#![allow(dead_code)] // for now

// Since we want this to be linear-time, we never want to iterate over a `Vec`. `ArrayVec`s have a hard,
// small maximum size and so we can consider iterating over them to be essentially constant-time.
use arrayvec::ArrayVec;

use dynasmrt::x64::Assembler;
use dynasmrt::{AssemblyOffset, DynamicLabel, DynasmApi, DynasmLabelApi, ExecutableBuffer};
use error::Error;
use std::{iter, mem};

use module::{RuntimeFunc, VmCtx};

/// Size of a pointer on the target in bytes.
const WORD_SIZE: u32 = 8;

type GPR = u8;

#[derive(Debug, Copy, Clone)]
struct GPRs {
    bits: u16,
}

impl GPRs {
    fn new() -> Self {
        Self { bits: 0 }
    }
}

const RAX: u8 = 0;
const RCX: u8 = 1;
const RDX: u8 = 2;
const RBX: u8 = 3;
const RSP: u8 = 4;
const RBP: u8 = 5;
const RSI: u8 = 6;
const RDI: u8 = 7;
const R8: u8 = 8;
const R9: u8 = 9;
const R10: u8 = 10;
const R11: u8 = 11;
const R12: u8 = 12;
const R13: u8 = 13;
const R14: u8 = 14;
const R15: u8 = 15;
const NUM_GPRS: u8 = 16;

extern "sysv64" fn println(len: u64, args: *const u8) {
    println!("{}", unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(args, len as usize))
    });
}

macro_rules! asm_println {
    ($asm:expr, $($args:tt)*) => {{
        use std::mem;

        let mut args = format!($($args)*).into_bytes();

        let len = args.len();
        let ptr = args.as_mut_ptr();
        mem::forget(args);

        dynasm!($asm
            ; push rdi
            ; push rsi
            ; push rdx
            ; push rcx
            ; push r8
            ; push r9
            ; push r10
            ; push r11

            ; mov rax, QWORD println as *const u8 as i64
            ; mov rdi, QWORD len as i64
            ; mov rsi, QWORD ptr as i64

            ; mov r11, rsp
            ; and r11, 0b1111
            ; test r11, r11
            ; jnz >with_adjusted_stack_ptr

            ; call rax
            ; jmp >pop_rest

            ; with_adjusted_stack_ptr:
            ; push 1
            ; call rax
            ; pop r11

            ; pop_rest:
            ; pop r11
            ; pop r10
            ; pop r9
            ; pop r8
            ; pop rcx
            ; pop rdx
            ; pop rsi
            ; pop rdi
        );
    }}
}

impl GPRs {
    fn take(&mut self) -> GPR {
        let lz = self.bits.trailing_zeros();
        debug_assert!(lz < 16, "ran out of free GPRs");
        let gpr = lz as GPR;
        self.mark_used(gpr);
        gpr
    }

    fn mark_used(&mut self, gpr: GPR) {
        self.bits &= !(1 << gpr as u16);
    }

    fn release(&mut self, gpr: GPR) {
        debug_assert!(
            !self.is_free(gpr),
            "released register {} was already free",
            gpr
        );
        self.bits |= 1 << gpr;
    }

    fn free_count(&self) -> u32 {
        self.bits.count_ones()
    }

    fn is_free(&self, gpr: GPR) -> bool {
        (self.bits & (1 << gpr)) != 0
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Registers {
    scratch: GPRs,
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

impl Registers {
    pub fn new() -> Self {
        let mut result = Self {
            scratch: GPRs::new(),
        };
        // Give ourselves a few scratch registers to work with, for now.
        for &scratch in SCRATCH_REGS {
            result.release_scratch_gpr(scratch);
        }

        result
    }

    pub fn mark_used(&mut self, gpr: GPR) {
        self.scratch.mark_used(gpr);
    }

    // TODO: Add function that takes a scratch register if possible
    //       but otherwise gives a fresh stack location.
    pub fn take_scratch_gpr(&mut self) -> GPR {
        self.scratch.take()
    }

    pub fn release_scratch_gpr(&mut self, gpr: GPR) {
        self.scratch.release(gpr);
    }

    pub fn is_free(&self, gpr: GPR) -> bool {
        self.scratch.is_free(gpr)
    }

    pub fn free_scratch(&self) -> u32 {
        self.scratch.free_count()
    }
}

// TODO: Allow pushing condition codes to stack? We'd have to immediately
//       materialise them into a register if anything is pushed above them.
/// Describes location of a value.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ValueLocation {
    /// Value exists in a register.
    Reg(GPR),
    /// Value exists on the stack. This is an offset relative to the
    /// first local, and so will have to be adjusted with `adjusted_offset`
    /// before reading (as RSP may have been changed by `push`/`pop`).
    Stack(i32),
    /// Value is a literal (TODO: Support more than just `i32`)
    Immediate(i64),
}

// TODO: This assumes only system-v calling convention.
// In system-v calling convention the first 6 arguments are passed via registers.
// All rest arguments are passed on the stack.
const ARGS_IN_GPRS: &[GPR] = &[RDI, RSI, RDX, RCX, R8, R9];
// List of scratch registers taken from https://wiki.osdev.org/System_V_ABI
const SCRATCH_REGS: &[GPR] = &[RAX, R10, R11];

#[must_use]
pub struct FunctionEnd {
    should_generate_epilogue: bool,
}

pub struct CodeGenSession {
    assembler: Assembler,
    func_starts: Vec<(Option<AssemblyOffset>, DynamicLabel)>,
    has_memory: bool,
}

impl CodeGenSession {
    pub fn new(func_count: u32, has_memory: bool) -> Self {
        let mut assembler = Assembler::new().unwrap();
        let func_starts = iter::repeat_with(|| (None, assembler.new_dynamic_label()))
            .take(func_count as usize)
            .collect::<Vec<_>>();

        CodeGenSession {
            assembler,
            func_starts,
            has_memory,
        }
    }

    pub fn new_context(&mut self, func_idx: u32) -> Context {
        {
            let func_start = &mut self.func_starts[func_idx as usize];

            // At this point we know the exact start address of this function. Save it
            // and define dynamic label at this location.
            func_start.0 = Some(self.assembler.offset());
            self.assembler.dynamic_label(func_start.1);
        }

        Context {
            asm: &mut self.assembler,
            func_starts: &self.func_starts,
            has_memory: self.has_memory,
            block_state: Default::default(),
        }
    }

    pub fn into_translated_code_section(self) -> Result<TranslatedCodeSection, Error> {
        let exec_buf = self
            .assembler
            .finalize()
            .map_err(|_asm| Error::Assembler("assembler error".to_owned()))?;
        let func_starts = self
            .func_starts
            .iter()
            .map(|(offset, _)| offset.unwrap())
            .collect::<Vec<_>>();
        Ok(TranslatedCodeSection {
            exec_buf,
            func_starts,
            // TODO
            relocatable_accesses: vec![],
        })
    }
}

#[derive(Debug)]
struct RelocateAddress {
    reg: Option<GPR>,
    imm: usize,
}

#[derive(Debug)]
struct RelocateAccess {
    position: AssemblyOffset,
    dst_reg: GPR,
    address: RelocateAddress,
}

#[derive(Debug)]
pub struct UninitializedCodeSection(TranslatedCodeSection);

#[derive(Debug)]
pub struct TranslatedCodeSection {
    exec_buf: ExecutableBuffer,
    func_starts: Vec<AssemblyOffset>,
    relocatable_accesses: Vec<RelocateAccess>,
}

impl TranslatedCodeSection {
    pub fn func_start(&self, idx: usize) -> *const u8 {
        let offset = self.func_starts[idx];
        self.exec_buf.ptr(offset)
    }

    pub fn disassemble(&self) {
        ::disassemble::disassemble(&*self.exec_buf).unwrap();
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Value {
    Local(u32),
    Temp(GPR),
    Immediate(i64),
}

impl Value {
    fn immediate(&self) -> Option<i64> {
        match *self {
            Value::Immediate(i) => Some(i),
            _ => None,
        }
    }

    fn location(&self, locals: &Locals) -> ValueLocation {
        match *self {
            Value::Local(loc) => locals.get(loc),
            Value::Temp(reg) => ValueLocation::Reg(reg),
            Value::Immediate(reg) => ValueLocation::Immediate(reg),
        }
    }
}

/// A value on the logical stack. The logical stack is the value stack as it
/// is visible to the WebAssembly, whereas the physical stack is the stack as
/// it exists on the machine (i.e. as offsets in memory relative to `rsp`).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum StackValue {
    /// A local (arguments are also locals). Note that when setting a local
    /// any values of this local stored on the stack are rendered invalid.
    /// See `manifest_local` for how we manage this.
    Local(u32),
    /// A temporary, stored in a register
    Temp(GPR),
    /// An immediate value, created with `i32.const`/`i64.const` etc.
    Immediate(i64),
    /// This value is on the physical stack and so should be accessed
    /// with the `pop` instruction.
    Pop,
}

impl StackValue {
    /// Returns either the location that this value can be accessed at
    /// if possible. If this value is `Pop`, you can only access it by
    /// popping the physical stack and so this function returns `None`.
    ///
    /// Of course, we could calculate the location of the value on the
    /// physical stack, but that would be unncessary computation for
    /// our usecases.
    fn location(&self, locals: &Locals) -> Option<ValueLocation> {
        match *self {
            StackValue::Local(loc) => Some(locals.get(loc)),
            StackValue::Immediate(i) => Some(ValueLocation::Immediate(i)),
            StackValue::Temp(reg) => Some(ValueLocation::Reg(reg)),
            StackValue::Pop => None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum ArgLoc {
    Register(GPR),
    Stack(i32),
    Both(GPR, i32),
}

impl ArgLoc {
    fn from_loc(loc: ValueLocation) -> Self {
        match loc {
            ValueLocation::Stack(offset) => ArgLoc::Stack(offset),
            ValueLocation::Reg(gpr) => ArgLoc::Register(gpr),
            _ => panic!("Unsupported local location"),
        }
    }

    fn union(&mut self, other: ArgLoc) {
        for l in other.locs() {
            self.add_loc(l);
        }
    }

    fn locs(&self) -> impl Iterator<Item = ValueLocation> {
        self.reg()
            .map(ValueLocation::Reg)
            .into_iter()
            .chain(self.stack().map(ValueLocation::Stack))
    }

    fn reg(&self) -> Option<GPR> {
        match *self {
            ArgLoc::Register(gpr) | ArgLoc::Both(gpr, _) => Some(gpr),
            ArgLoc::Stack(_) => None,
        }
    }

    fn stack(&self) -> Option<i32> {
        match *self {
            ArgLoc::Stack(o) | ArgLoc::Both(_, o) => Some(o),
            ArgLoc::Register(_) => None,
        }
    }

    fn add_loc(&mut self, loc: ValueLocation) {
        match loc {
            ValueLocation::Stack(offset) => self.add_stack(offset),
            ValueLocation::Reg(gpr) => self.add_reg(gpr),
            _ => panic!("Unsupported local location"),
        }
    }

    fn add_stack(&mut self, offset: i32) {
        *self = match *self {
            ArgLoc::Register(gpr) | ArgLoc::Both(gpr, _) => ArgLoc::Both(gpr, offset),
            ArgLoc::Stack(_) => ArgLoc::Stack(offset),
        };
    }

    fn add_reg(&mut self, gpr: GPR) {
        *self = match *self {
            ArgLoc::Stack(offset) | ArgLoc::Both(_, offset) => ArgLoc::Both(gpr, offset),
            ArgLoc::Register(_) => ArgLoc::Register(gpr),
        };
    }

    fn best_loc(&self) -> ValueLocation {
        match *self {
            ArgLoc::Register(gpr) | ArgLoc::Both(gpr, _) => ValueLocation::Reg(gpr),
            ArgLoc::Stack(offset) => ValueLocation::Stack(offset),
        }
    }
}

/// A store for the local values for our function, including arguments.
#[derive(Debug, Default, Clone)]
pub struct Locals {
    // TODO: Store all places that the value can be read, so we can optimise
    //       passing (register) arguments along into a noop after saving their
    //       values.
    /// All arguments in registers, check `ARGS_IN_GPRS` for the list of
    /// registers that this can contain. If we need to move the argument
    /// out of a register (for example, because we're calling a function)
    /// we note that down here, so we don't have to move it back afterwards.
    register_locals: ArrayVec<[ArgLoc; ARGS_IN_GPRS.len()]>,
    /// The number of arguments stored on the stack.
    num_stack_args: u32,
    /// The number of local stack slots, i.e. the amount of stack space reserved for locals.
    num_local_stack_slots: u32,
}

impl Locals {
    fn register(&self, index: u32) -> Option<GPR> {
        if index < self.register_locals.len() as u32 {
            Some(ARGS_IN_GPRS[index as usize])
        } else {
            None
        }
    }

    fn add_pos(&mut self, index: u32, loc: ValueLocation) {
        self.register_locals[index as usize].add_loc(loc);
    }

    fn set_pos(&mut self, index: u32, loc: ValueLocation) {
        self.register_locals[index as usize] = ArgLoc::from_loc(loc);
    }

    fn get(&self, index: u32) -> ValueLocation {
        self.register_locals
            .get(index as usize)
            .map(ArgLoc::best_loc)
            .unwrap_or_else(|| {
                let stack_index = index - self.register_locals.len() as u32;
                if stack_index < self.num_stack_args {
                    ValueLocation::Stack(
                        ((stack_index + self.num_local_stack_slots + 2) * WORD_SIZE) as _,
                    )
                } else {
                    let stack_index = stack_index - self.num_stack_args;
                    ValueLocation::Stack((stack_index * WORD_SIZE) as _)
                }
            })
    }

    fn num_args(&self) -> u32 {
        self.register_locals.len() as u32 + self.num_stack_args
    }

    fn vmctx_index(&self) -> u32 {
        0
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlockState {
    stack: Stack,
    // TODO: `BitVec`
    stack_map: Vec<bool>,
    depth: StackDepth,
    return_register: Option<GPR>,
    regs: Registers,
    /// This is the _current_ locals, since we can shuffle them about during function calls.
    pub locals: Locals,
    /// In non-linear control flow (ifs and loops) we have to set the locals to the state that
    /// the next block that we enter will expect them in.
    pub end_locals: Option<Locals>,
}

type Stack = Vec<StackValue>;

pub enum MemoryAccessMode {
    /// This is slower than using `Unchecked` mode, but works in
    /// any scenario (the most important scenario being when we're
    /// running on a system that can't index much more memory than
    /// the Wasm).
    Checked,
    /// This means that checks are _not emitted by the compiler_!
    /// If you're using WebAssembly to run untrusted code, you
    /// _must_ delegate bounds checking somehow (probably by
    /// allocating 2^33 bytes of memory with the second half set
    /// to unreadable/unwriteable/unexecutable)
    Unchecked,
}

pub struct Context<'a> {
    asm: &'a mut Assembler,
    func_starts: &'a Vec<(Option<AssemblyOffset>, DynamicLabel)>,
    /// Each push and pop on the value stack increments or decrements this value by 1 respectively.
    pub block_state: BlockState,
    has_memory: bool,
}

/// Label in code.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Label(DynamicLabel);

/// Offset from starting value of SP counted in words.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct StackDepth(u32);

impl StackDepth {
    pub fn reserve(&mut self, slots: u32) {
        self.0 += slots;
    }

    pub fn free(&mut self, slots: u32) {
        self.0 -= slots;
    }
}

macro_rules! unop {
    ($name:ident, $instr:ident, $reg_ty:ident, $typ:ty, $const_fallback:expr) => {
        pub fn $name(&mut self) {
            let val = self.pop();

            let out_val = match val.location(&self.block_state.locals) {
                ValueLocation::Immediate(imm) => Value::Immediate($const_fallback(imm as $typ) as _),
                ValueLocation::Stack(offset) => {
                    let offset = self.adjusted_offset(offset);
                    let temp = self.block_state.regs.take_scratch_gpr();
                    dynasm!(self.asm
                        ; $instr $reg_ty(temp), [rsp + offset]
                    );
                    Value::Temp(temp)
                }
                ValueLocation::Reg(reg) => {
                    let temp = self.block_state.regs.take_scratch_gpr();
                    dynasm!(self.asm
                        ; $instr $reg_ty(temp), $reg_ty(reg)
                    );
                    Value::Temp(temp)
                }
            };

            self.push(out_val);
        }
    }
}

macro_rules! cmp_i32 {
    ($name:ident, $instr:ident, $reverse_instr:ident, $const_fallback:expr) => {
        pub fn $name(&mut self) {
            let right = self.pop();
            let left = self.pop();

            let out = if let Some(i) = left.immediate() {
                match right.location(&self.block_state.locals) {
                    ValueLocation::Stack(offset) => {
                        let result = self.block_state.regs.take_scratch_gpr();
                        let offset = self.adjusted_offset(offset);
                        dynasm!(self.asm
                            ; xor Rd(result), Rd(result)
                            ; cmp DWORD [rsp + offset], i as i32
                            ; $reverse_instr Rb(result)
                        );
                        Value::Temp(result)
                    }
                    ValueLocation::Reg(rreg) => {
                        let result = self.block_state.regs.take_scratch_gpr();
                        dynasm!(self.asm
                            ; xor Rd(result), Rd(result)
                            ; cmp Rd(rreg), i as i32
                            ; $reverse_instr Rb(result)
                        );
                        Value::Temp(result)
                    }
                    ValueLocation::Immediate(right) => {
                        Value::Immediate(if $const_fallback(i as i32, right as i32) { 1 } else { 0 })
                    }
                }
            } else {
                let (lreg, lreg_needs_free) = self.into_reg(left);
                let result = self.block_state.regs.take_scratch_gpr();

                match right.location(&self.block_state.locals) {
                    ValueLocation::Stack(offset) => {
                        let offset = self.adjusted_offset(offset);
                        dynasm!(self.asm
                            ; xor Rd(result), Rd(result)
                            ; cmp Rd(lreg), [rsp + offset]
                            ; $instr Rb(result)
                        );
                    }
                    ValueLocation::Reg(rreg) => {
                        dynasm!(self.asm
                            ; xor Rd(result), Rd(result)
                            ; cmp Rd(lreg), Rd(rreg)
                            ; $instr Rb(result)
                        );
                    }
                    ValueLocation::Immediate(i) => {
                        dynasm!(self.asm
                            ; xor Rd(result), Rd(result)
                            ; cmp Rd(lreg), i as i32
                            ; $instr Rb(result)
                        );
                    }
                }

                if left != Value::Temp(lreg) && lreg_needs_free {
                    self.block_state.regs.release_scratch_gpr(lreg);
                }

                Value::Temp(result)
            };

            self.free_value(left);
            self.free_value(right);

            self.push(out);
        }
    }
}

macro_rules! cmp_i64 {
    ($name:ident, $instr:ident, $reverse_instr:ident, $const_fallback:expr) => {
        pub fn $name(&mut self) {
            let right = self.pop();
            let left = self.pop();

            let out = if let Some(i) = left.immediate() {
                match right.location(&self.block_state.locals) {
                    ValueLocation::Stack(offset) => {
                        let result = self.block_state.regs.take_scratch_gpr();
                        let offset = self.adjusted_offset(offset);
                        if let Some(i) = i.try_into() {
                            dynasm!(self.asm
                                ; xor Rd(result), Rd(result)
                                ; cmp QWORD [rsp + offset], i
                                ; $reverse_instr Rb(result)
                            );
                        } else {
                            unimplemented!("Unsupported `cmp` with large 64-bit immediate operand");
                        }
                        Value::Temp(result)
                    }
                    ValueLocation::Reg(rreg) => {
                        let result = self.block_state.regs.take_scratch_gpr();
                        if let Some(i) = i.try_into() {
                            dynasm!(self.asm
                                ; xor Rd(result), Rd(result)
                                ; cmp Rq(rreg), i
                                ; $reverse_instr Rb(result)
                            );
                        } else {
                            unimplemented!("Unsupported `cmp` with large 64-bit immediate operand");
                        }
                        Value::Temp(result)
                    }
                    ValueLocation::Immediate(right) => {
                        Value::Immediate(if $const_fallback(i, right) { 1 } else { 0 })
                    }
                }
            } else {
                let (lreg, lreg_needs_free) = self.into_reg(left);
                let result = self.block_state.regs.take_scratch_gpr();

                match right.location(&self.block_state.locals) {
                    ValueLocation::Stack(offset) => {
                        let offset = self.adjusted_offset(offset);
                        dynasm!(self.asm
                            ; xor Rd(result), Rd(result)
                            ; cmp Rq(lreg), [rsp + offset]
                            ; $instr Rb(result)
                        );
                    }
                    ValueLocation::Reg(rreg) => {
                        dynasm!(self.asm
                            ; xor Rd(result), Rd(result)
                            ; cmp Rq(lreg), Rq(rreg)
                            ; $instr Rb(result)
                        );
                    }
                    ValueLocation::Immediate(i) => {
                        if let Some(i) = i.try_into() {
                            dynasm!(self.asm
                                ; xor Rd(result), Rd(result)
                                ; cmp Rq(lreg), i
                                ; $instr Rb(result)
                            );
                        } else {
                            unimplemented!("Unsupported `cmp` with large 64-bit immediate operand");
                        }
                    }
                }

                if left != Value::Temp(lreg) && lreg_needs_free {
                    self.block_state.regs.release_scratch_gpr(lreg);
                }

                Value::Temp(result)
            };

            self.free_value(left);
            self.free_value(right);
            self.push(out);
        }
    }
}

#[must_use]
pub struct CallCleanup {
    restore_registers: ArrayVec<[GPR; SCRATCH_REGS.len()]>,
    stack_depth: i32,
}

macro_rules! commutative_binop_i32 {
    ($name:ident, $instr:ident, $const_fallback:expr) => {
        pub fn $name(&mut self) {
            let op0 = self.pop();
            let op1 = self.pop();

            if let Some(i1) = op1.immediate() {
                if let Some(i0) = op0.immediate() {
                    self.block_state.stack.push(StackValue::Immediate($const_fallback(i1 as i32, i0 as i32) as _));
                    return;
                }
            }

            let (op1, op0) = match op1 {
                Value::Temp(reg) => (reg, op0),
                _ => if op0.immediate().is_some() {
                    (self.into_temp_reg(op1), op0)
                } else {
                    (self.into_temp_reg(op0), op1)
                }
            };

            match op0.location(&self.block_state.locals) {
                ValueLocation::Reg(reg) => {
                    dynasm!(self.asm
                        ; $instr Rd(op1), Rd(reg)
                    );
                }
                ValueLocation::Stack(offset) => {
                    let offset = self.adjusted_offset(offset);
                    dynasm!(self.asm
                        ; $instr Rd(op1), [rsp + offset]
                    );
                }
                ValueLocation::Immediate(i) => {
                    dynasm!(self.asm
                        ; $instr Rd(op1), i as i32
                    );
                }
            }

            self.free_value(op0);
            self.push(Value::Temp(op1));
        }
    }
}

macro_rules! commutative_binop_i64 {
    ($name:ident, $instr:ident, $const_fallback:expr) => {
        pub fn $name(&mut self) {
            let op0 = self.pop();
            let op1 = self.pop();

            if let Some(i1) = op1.immediate() {
                if let Some(i0) = op0.immediate() {
                    self.block_state.stack.push(StackValue::Immediate($const_fallback(i1, i0)));
                    return;
                }
            }

            let (op1, op0) = match op1 {
                Value::Temp(reg) => (reg, op0),
                _ => if op0.immediate().is_some() {
                    (self.into_temp_reg(op1), op0)
                } else {
                    (self.into_temp_reg(op0), op1)
                }
            };

            match op0.location(&self.block_state.locals) {
                ValueLocation::Reg(reg) => {
                    dynasm!(self.asm
                        ; $instr Rq(op1), Rq(reg)
                    );
                }
                ValueLocation::Stack(offset) => {
                    let offset = self.adjusted_offset(offset);
                    dynasm!(self.asm
                        ; $instr Rq(op1), [rsp + offset]
                    );
                }
                ValueLocation::Immediate(i) => {
                    if let Some(i) = i.try_into() {
                        dynasm!(self.asm
                            ; $instr Rq(op1), i
                        );
                    } else {
                        let scratch = self.block_state.regs.take_scratch_gpr();

                        dynasm!(self.asm
                            ; mov Rq(scratch), QWORD i
                            ; $instr Rq(op1), Rq(scratch)
                        );

                        self.block_state.regs.release_scratch_gpr(scratch);
                    }
                }
            }

            self.free_value(op0);
            self.push(Value::Temp(op1));
        }
    }
}

macro_rules! load {
    ($name:ident, $reg_ty:ident, $instruction_name:expr) => {
        pub fn $name(&mut self, offset: u32) -> Result<(), Error> {
            fn load_to_reg(
                ctx: &mut Context,
                dst: GPR,
                vmctx: GPR,
                (offset, runtime_offset): (i32, Result<i32, GPR>)
            ) {
                let vmctx_mem_offset = VmCtx::offset_of_memory() as i32;
                match runtime_offset {
                    Ok(imm) => {
                        dynasm!(ctx.asm
                            ; mov $reg_ty(dst), [Rq(vmctx) + offset + imm + vmctx_mem_offset]
                        );
                    }
                    Err(offset_reg) => {
                        dynasm!(ctx.asm
                            ; mov $reg_ty(dst), [Rq(vmctx) + Rq(offset_reg) + offset + vmctx_mem_offset]
                        );
                    }
                }
            }

            assert!(offset <= i32::max_value() as u32);

            if !self.has_memory {
                return Err(Error::Input(
                    concat!(
                        "Unexpected ",
                        $instruction_name,
                        ", this module has no memory section"
                    ).into()
                ));
            }

            let base = self.pop();
            let vmctx_idx = self.block_state.locals.vmctx_index();

            let (vmctx, needs_release) = self.into_reg(Value::Local(vmctx_idx));

            let temp = self.block_state.regs.take_scratch_gpr();

            match base.location(&self.block_state.locals) {
                // TODO: Do compilers (to wasm) actually emit load-with-immediate when doing
                //       constant loads? There isn't a `load` variant that _doesn't_ take a
                //       runtime parameter.
                ValueLocation::Immediate(i) => {
                    let val = if let Some(i) = i.try_into() {
                        Ok(i)
                    } else {
                        Err(self.into_temp_reg(base))
                    };

                    load_to_reg(self, temp, vmctx, (offset as _, val));

                    if let Err(r) = val {
                        self.block_state.regs.release_scratch_gpr(r);
                    }

                    // TODO: Push relocation
                }
                ValueLocation::Reg(gpr) => {
                    load_to_reg(self, temp, vmctx, (offset as _, Err(gpr)));
                    // TODO: Push relocation
                }
                ValueLocation::Stack(_) => {
                    let gpr = self.into_temp_reg(base);
                    load_to_reg(self, temp, vmctx, (offset as _, Err(gpr)));
                    self.block_state.regs.release_scratch_gpr(gpr);
                    // TODO: Push relocation
                }
            }

            self.free_value(base);
            if needs_release {
                self.block_state.regs.release_scratch_gpr(vmctx);
            }
            self.push(Value::Temp(temp));

            Ok(())
        }
    }
}

macro_rules! store {
    ($name:ident, $reg_ty:ident, $size:ident, $instruction_name:expr) => {
        pub fn $name(&mut self, offset: u32) -> Result<(), Error> {
            fn store_from_reg(
                ctx: &mut Context,
                src: GPR,
                vmctx: GPR,
                (offset, runtime_offset): (i32, Result<i32, GPR>)
            ) {
                let vmctx_mem_offset = VmCtx::offset_of_memory() as i32;
                match runtime_offset {
                    Ok(imm) => {
                        dynasm!(ctx.asm
                            ; mov [Rq(vmctx) + offset + imm + vmctx_mem_offset], $reg_ty(src)
                        );
                    }
                    Err(offset_reg) => {
                        dynasm!(ctx.asm
                            ; mov [Rq(vmctx) + Rq(offset_reg) + offset + vmctx_mem_offset], $reg_ty(src)
                        );
                    }
                }
            }

            assert!(offset <= i32::max_value() as u32);

            if !self.has_memory {
                return Err(Error::Input(
                    concat!(
                        "Unexpected ",
                        $instruction_name,
                        ", this module has no memory section"
                    ).into()
                ));
            }

            let src = self.pop();
            let base = self.pop();
            let vmctx_idx = self.block_state.locals.vmctx_index();

            let (vmctx, needs_release) = self.into_reg(Value::Local(vmctx_idx));

            let (src_reg, src_needs_free) = self.into_reg(src);

            match base.location(&self.block_state.locals) {
                // TODO: Do compilers (to wasm) actually emit load-with-immediate when doing
                //       constant loads? There isn't a `load` variant that _doesn't_ take a
                //       runtime parameter.
                ValueLocation::Immediate(i) => {
                    let val = if let Some(i) = i.try_into() {
                        Ok(i)
                    } else {
                        Err(self.into_temp_reg(base))
                    };

                    store_from_reg(self, src_reg, vmctx, (offset as i32, val));

                    if let Err(r) = val {
                        self.block_state.regs.release_scratch_gpr(r);
                    }

                    // TODO: Push relocation
                }
                ValueLocation::Reg(gpr) => {
                    store_from_reg(self, src_reg, vmctx, (offset as i32, Err(gpr)));
                    // TODO: Push relocation
                }
                ValueLocation::Stack(_) => {
                    let gpr = self.into_temp_reg(base);
                    store_from_reg(self, src_reg, vmctx, (offset as i32, Err(gpr)));
                    self.block_state.regs.release_scratch_gpr(gpr);
                    // TODO: Push relocation
                }
            }

            self.free_value(base);
            if src_needs_free {
                self.block_state.regs.release_scratch_gpr(src_reg);
            }
            if needs_release {
                self.block_state.regs.release_scratch_gpr(vmctx);
            }

            Ok(())
        }
    }
}

trait TryInto<O> {
    fn try_into(self) -> Option<O>;
}

impl TryInto<i64> for u64 {
    fn try_into(self) -> Option<i64> {
        let max = i64::max_value() as u64;

        if self <= max {
            Some(self as i64)
        } else {
            None
        }
    }
}

impl TryInto<i32> for i64 {
    fn try_into(self) -> Option<i32> {
        let min = i32::min_value() as i64;
        let max = i32::max_value() as i64;

        if self >= min && self <= max {
            Some(self as i32)
        } else {
            None
        }
    }
}

impl Context<'_> {
    /// Create a new undefined label.
    pub fn create_label(&mut self) -> Label {
        Label(self.asm.new_dynamic_label())
    }

    pub fn define_host_fn(&mut self, host_fn: *const u8) {
        dynasm!(self.asm
            ; mov rax, QWORD host_fn as i64
            ; call rax
            ; ret
        );
    }

    fn adjusted_offset(&self, offset: i32) -> i32 {
        (self.block_state.depth.0 * WORD_SIZE) as i32 + offset
    }

    cmp_i32!(i32_eq, sete, sete, |a, b| a == b);
    cmp_i32!(i32_neq, setne, setne, |a, b| a != b);
    // `dynasm-rs` inexplicably doesn't support setb but `setnae` (and `setc`) are synonymous
    cmp_i32!(i32_lt_u, setnae, seta, |a, b| (a as u32) < (b as u32));
    cmp_i32!(i32_le_u, setbe, setae, |a, b| (a as u32) <= (b as u32));
    cmp_i32!(i32_gt_u, seta, setnae, |a, b| (a as u32) > (b as u32));
    cmp_i32!(i32_ge_u, setae, setna, |a, b| (a as u32) >= (b as u32));
    cmp_i32!(i32_lt_s, setl, setnle, |a, b| a < b);
    cmp_i32!(i32_le_s, setle, setnl, |a, b| a <= b);
    cmp_i32!(i32_gt_s, setg, setnge, |a, b| a > b);
    cmp_i32!(i32_ge_s, setge, setng, |a, b| a >= b);

    cmp_i64!(i64_eq, sete, sete, |a, b| a == b);
    cmp_i64!(i64_neq, setne, setne, |a, b| a != b);
    // `dynasm-rs` inexplicably doesn't support setb but `setnae` (and `setc`) are synonymous
    cmp_i64!(i64_lt_u, setnae, seta, |a, b| (a as u64) < (b as u64));
    cmp_i64!(i64_le_u, setbe, setae, |a, b| (a as u64) <= (b as u64));
    cmp_i64!(i64_gt_u, seta, setnae, |a, b| (a as u64) > (b as u64));
    cmp_i64!(i64_ge_u, setae, setna, |a, b| (a as u64) >= (b as u64));
    cmp_i64!(i64_lt_s, setl, setnle, |a, b| a < b);
    cmp_i64!(i64_le_s, setle, setnl, |a, b| a <= b);
    cmp_i64!(i64_gt_s, setg, setnge, |a, b| a > b);
    cmp_i64!(i64_ge_s, setge, setng, |a, b| a >= b);

    // TODO: Should we do this logic in `eq` and just have this delegate to `eq`?
    //       That would mean that `eqz` and `eq` with a const 0 argument don't
    //       result in different code.
    pub fn i32_eqz(&mut self) {
        let val = self.pop();

        if let Value::Immediate(i) = val {
            self.push(Value::Immediate(if i == 0 { 1 } else { 0 }));
            return;
        }

        let (reg, needs_release) = self.into_reg(val);
        let out = self.block_state.regs.take_scratch_gpr();

        dynasm!(self.asm
            ; xor Rd(out), Rd(out)
            ; test Rd(reg), Rd(reg)
            ; setz Rb(out)
        );

        if needs_release {
            self.block_state.regs.release_scratch_gpr(reg);
        }

        self.push(Value::Temp(out));
    }

    pub fn i64_eqz(&mut self) {
        let val = self.pop();

        if let Value::Immediate(i) = val {
            self.push(Value::Immediate(if i == 0 { 1 } else { 0 }));
            return;
        }

        let (reg, needs_release) = self.into_reg(val);
        let out = self.block_state.regs.take_scratch_gpr();

        dynasm!(self.asm
            ; xor Rd(out), Rd(out)
            ; test Rq(reg), Rq(reg)
            ; setz Rb(out)
        );

        if needs_release {
            self.block_state.regs.release_scratch_gpr(reg);
        }

        self.push(Value::Temp(out));
    }

    /// Pops i32 predicate and branches to the specified label
    /// if the predicate is equal to zero.
    pub fn jump_if_false(&mut self, label: Label) {
        let val = self.pop();
        let predicate = self.into_temp_reg(val);
        dynasm!(self.asm
            ; test Rd(predicate), Rd(predicate)
            ; je =>label.0
        );
        self.block_state.regs.release_scratch_gpr(predicate);
    }

    /// Branch unconditionally to the specified label.
    pub fn br(&mut self, label: Label) {
        dynasm!(self.asm
            ; jmp =>label.0
        );
    }

    fn immediate_to_reg(&mut self, reg: GPR, val: i64) {
        if (val as u64) <= u32::max_value() as u64 {
            dynasm!(self.asm
                ; mov Rd(reg), val as i32
            );
        } else {
            dynasm!(self.asm
                ; mov Rq(reg), QWORD val
            );
        }
    }

    fn copy_value(&mut self, src: ValueLocation, dst: ValueLocation) {
        match (src, dst) {
            (ValueLocation::Stack(in_offset), ValueLocation::Stack(out_offset)) => {
                let in_offset = self.adjusted_offset(in_offset);
                let out_offset = self.adjusted_offset(out_offset);
                if in_offset != out_offset {
                    let gpr = self.block_state.regs.take_scratch_gpr();
                    dynasm!(self.asm
                        ; mov Rq(gpr), [rsp + in_offset]
                        ; mov [rsp + out_offset], Rq(gpr)
                    );
                    self.block_state.regs.release_scratch_gpr(gpr);
                }
            }
            (ValueLocation::Reg(in_reg), ValueLocation::Stack(out_offset)) => {
                let out_offset = self.adjusted_offset(out_offset);
                dynasm!(self.asm
                    ; mov [rsp + out_offset], Rq(in_reg)
                );
            }
            (ValueLocation::Immediate(i), ValueLocation::Stack(out_offset)) => {
                let out_offset = self.adjusted_offset(out_offset);
                if (i as u64) <= u32::max_value() as u64 {
                    dynasm!(self.asm
                        ; mov DWORD [rsp + out_offset], i as i32
                    );
                } else {
                    let scratch = self.block_state.regs.take_scratch_gpr();

                    dynasm!(self.asm
                        ; mov Rq(scratch), QWORD i
                        ; mov [rsp + out_offset], Rq(scratch)
                    );

                    self.block_state.regs.release_scratch_gpr(scratch);
                }
            }
            (ValueLocation::Stack(in_offset), ValueLocation::Reg(out_reg)) => {
                let in_offset = self.adjusted_offset(in_offset);
                dynasm!(self.asm
                    ; mov Rq(out_reg), [rsp + in_offset]
                );
            }
            (ValueLocation::Reg(in_reg), ValueLocation::Reg(out_reg)) => {
                if in_reg != out_reg {
                    dynasm!(self.asm
                        ; mov Rq(out_reg), Rq(in_reg)
                    );
                }
            }
            (ValueLocation::Immediate(i), ValueLocation::Reg(out_reg)) => {
                self.immediate_to_reg(out_reg, i);
            }
            // TODO: Have separate `ReadLocation` and `WriteLocation`?
            (_, ValueLocation::Immediate(_)) => panic!("Tried to copy to an immediate value!"),
        }
    }

    /// Define the given label at the current position.
    ///
    /// Multiple labels can be defined at the same position. However, a label
    /// can be defined only once.
    pub fn define_label(&mut self, label: Label) {
        self.asm.dynamic_label(label.0);
    }

    fn expand_stack(&mut self, by: u32) {
        use std::iter;

        if by == 0 {
            return;
        }

        let new_stack_size = (self.block_state.stack_map.len() + by as usize).next_power_of_two();
        let additional_elements = new_stack_size - self.block_state.stack_map.len();
        self.block_state
            .stack_map
            .extend(iter::repeat(false).take(additional_elements));

        dynasm!(self.asm
            ; sub rsp, additional_elements as i32
        );
    }

    // TODO: Make this generic over `Vec` or `ArrayVec`?
    fn stack_slots(&mut self, count: u32) -> Vec<i32> {
        let mut out = Vec::with_capacity(count as usize);

        let offset_if_taken = |(i, is_taken): (usize, bool)| {
            if !is_taken {
                Some(i as i32 * WORD_SIZE as i32)
            } else {
                None
            }
        };

        out.extend(
            self.block_state
                .stack_map
                .iter()
                .cloned()
                .enumerate()
                .filter_map(offset_if_taken),
        );

        let remaining = count as usize - out.len();

        if remaining > 0 {
            self.expand_stack(remaining as u32);
            out.extend(
                self.block_state
                    .stack_map
                    .iter()
                    .cloned()
                    .enumerate()
                    .filter_map(offset_if_taken),
            );
        }

        out
    }

    fn stack_slot(&mut self) -> i32 {
        if let Some(pos) = self
            .block_state
            .stack_map
            .iter()
            .position(|is_taken| !is_taken)
        {
            self.block_state.stack_map[pos] = true;
            pos as i32 * WORD_SIZE as i32
        } else {
            self.expand_stack(1);
            self.stack_slot()
        }
    }

    // We use `put` instead of `pop` since with `BrIf` it's possible
    // that the block will continue after returning.
    pub fn return_from_block(&mut self, arity: u32) {
        if arity == 0 {
            return;
        }

        let stack_top = *self.block_state.stack.last().expect("Stack is empty");
        if let Some(reg) = self.block_state.return_register {
            self.put_stack_val_into(stack_top, ValueLocation::Reg(reg));
        } else {
            let out_reg = match stack_top {
                StackValue::Temp(r) => r,
                other => {
                    let new_scratch = self.block_state.regs.take_scratch_gpr();
                    self.put_stack_val_into(other, ValueLocation::Reg(new_scratch));
                    new_scratch
                }
            };

            self.block_state.return_register = Some(out_reg);
        }
    }

    pub fn start_block(&mut self) -> BlockState {
        use std::mem;

        // OPTIMISATION: We cannot use the parent's stack values (it is disallowed by the spec)
        //               so we start a new stack, using `mem::replace` to ensure that we never
        //               clone or deallocate anything.
        //
        //               I believe that it would be possible to cause a compiler bomb if we did
        //               not do this, since cloning iterates over the whole `Vec`.
        let out_stack = mem::replace(&mut self.block_state.stack, vec![]);
        let mut current_state = self.block_state.clone();
        current_state.stack = out_stack;

        self.block_state.return_register = None;
        current_state
    }

    // To start the next subblock of a block (for `if..then..else..end`).
    // The only difference is that choices we made in the first subblock
    // (for now only the return register) must be maintained in the next
    // subblocks.
    pub fn reset_block(&mut self, parent_block_state: BlockState) {
        let return_reg = self.block_state.return_register;
        let locals = mem::replace(&mut self.block_state.locals, Default::default());

        self.block_state = parent_block_state;
        self.block_state.end_locals = Some(locals);

        self.block_state.return_register = return_reg;
    }

    pub fn end_block(
        &mut self,
        parent_block_state: BlockState,
        before_push_return: impl FnOnce(&mut Self),
    ) {
        debug_assert_eq!(
            self.block_state.depth, parent_block_state.depth,
            "Imbalanced pushes and pops"
        );
        // TODO: This should currently never be called, but is important for if we want to
        //       have a more complex stack spilling scheme.
        // TODO: This should use an `end_locals`-style system where we only do this when
        //       control flow splits.
        if self.block_state.depth != parent_block_state.depth {
            dynasm!(self.asm
                ; add rsp, ((self.block_state.depth.0 - parent_block_state.depth.0) * WORD_SIZE) as i32
            );
        }

        let return_reg = self.block_state.return_register;
        let locals = mem::replace(&mut self.block_state.locals, Default::default());
        self.block_state = parent_block_state;
        self.block_state.locals = locals;

        before_push_return(self);

        if let Some(reg) = return_reg {
            self.block_state.regs.mark_used(reg);
            self.block_state.stack.push(StackValue::Temp(reg));
        }
    }

    pub fn restore_locals(&mut self) {
        if let Some(end_locals) = self.block_state.end_locals.clone() {
            self.restore_locals_to(&end_locals);
        }
    }

    pub fn restore_locals_to(&mut self, locals: &Locals) {
        for (src, dst) in self
            .block_state
            .locals
            .register_locals
            .clone()
            .iter()
            .zip(&locals.register_locals)
        {
            self.copy_value(src.best_loc(), dst.best_loc());
        }

        for (src, dst) in self
            .block_state
            .locals
            .register_locals
            .iter_mut()
            .zip(&locals.register_locals)
        {
            src.union(*dst);
        }
    }

    load!(i32_load, Rd, "i32.load");
    load!(i64_load, Rq, "i64.load");
    store!(i32_store, Rd, DWORD, "i32.store");
    store!(i64_store, Rq, QWORD, "i64.store");

    fn push(&mut self, value: Value) {
        let stack_loc = match value {
            Value::Local(loc) => StackValue::Local(loc),
            Value::Immediate(i) => StackValue::Immediate(i),
            Value::Temp(gpr) => {
                if self.block_state.regs.free_scratch() >= 3 {
                    StackValue::Temp(gpr)
                } else {
                    self.block_state.depth.reserve(1);
                    // TODO: Proper stack allocation scheme
                    dynasm!(self.asm
                        ; push Rq(gpr)
                    );
                    self.block_state.regs.release_scratch_gpr(gpr);
                    StackValue::Pop
                }
            }
        };

        self.block_state.stack.push(stack_loc);
    }

    fn pop(&mut self) -> Value {
        match self.block_state.stack.pop().expect("Stack is empty") {
            StackValue::Local(loc) => Value::Local(loc),
            StackValue::Immediate(i) => Value::Immediate(i),
            StackValue::Temp(reg) => Value::Temp(reg),
            StackValue::Pop => {
                self.block_state.depth.free(1);
                let gpr = self.block_state.regs.take_scratch_gpr();
                dynasm!(self.asm
                    ; pop Rq(gpr)
                );
                Value::Temp(gpr)
            }
        }
    }

    /// Warning: this _will_ pop the runtime stack, but will _not_ pop the compile-time
    ///          stack. It's specifically for mid-block breaks like `Br` and `BrIf`.
    fn put_stack_val_into(&mut self, val: StackValue, dst: ValueLocation) {
        let to_move = match val {
            StackValue::Local(loc) => Value::Local(loc),
            StackValue::Immediate(i) => Value::Immediate(i),
            StackValue::Temp(reg) => Value::Temp(reg),
            StackValue::Pop => {
                self.block_state.depth.free(1);
                match dst {
                    ValueLocation::Reg(r) => dynasm!(self.asm
                        ; pop Rq(r)
                    ),
                    ValueLocation::Stack(offset) => {
                        let offset = self.adjusted_offset(offset);
                        dynasm!(self.asm
                            ; pop QWORD [rsp + offset]
                        )
                    }
                    ValueLocation::Immediate(_) => panic!("Tried to write to literal!"),
                }

                // DO NOT DO A `copy_val`
                return;
            }
        };

        let src = to_move.location(&self.block_state.locals);
        self.copy_value(src, dst);
        if src != dst {
            self.free_value(to_move);
        }
    }

    pub fn drop(&mut self) {
        match self.block_state.stack.pop().expect("Stack is empty") {
            StackValue::Pop => {
                self.block_state.depth.free(1);
                dynasm!(self.asm
                    ; add rsp, WORD_SIZE as i32
                );
            }
            StackValue::Temp(gpr) => self.free_value(Value::Temp(gpr)),
            StackValue::Local(loc) => self.free_value(Value::Local(loc)),
            StackValue::Immediate(imm) => self.free_value(Value::Immediate(imm)),
        }
    }

    fn pop_into(&mut self, dst: ValueLocation) {
        let val = self.block_state.stack.pop().expect("Stack is empty");
        self.put_stack_val_into(val, dst);
    }

    fn free_value(&mut self, val: Value) {
        match val {
            Value::Temp(reg) => self.block_state.regs.release_scratch_gpr(reg),
            Value::Local(_) | Value::Immediate(_) => {}
        }
    }

    /// Puts this value into a register so that it can be efficiently read
    fn into_reg(&mut self, val: Value) -> (GPR, bool) {
        match val {
            Value::Local(idx) => match self.block_state.locals.get(idx) {
                ValueLocation::Stack(offset) => {
                    let offset = self.adjusted_offset(offset);
                    let (reg, needs_release) =
                        if let Some(reg) = self.block_state.locals.register(idx) {
                            self.block_state
                                .locals
                                .set_pos(idx, ValueLocation::Reg(reg));
                            (reg, false)
                        } else {
                            (self.block_state.regs.take_scratch_gpr(), true)
                        };
                    dynasm!(self.asm
                        ; mov Rq(reg), [rsp + offset]
                    );
                    (reg, needs_release)
                }
                ValueLocation::Reg(reg) => (reg, false),
                ValueLocation::Immediate(..) => {
                    panic!("Currently immediates in locals are unsupported")
                }
            },
            Value::Immediate(i) => {
                let scratch = self.block_state.regs.take_scratch_gpr();
                self.immediate_to_reg(scratch, i);
                (scratch, true)
            }
            Value::Temp(reg) => (reg, true),
        }
    }

    /// Puts this value into a temporary register so that operations
    /// on that register don't write to a local.
    fn into_temp_reg(&mut self, val: Value) -> GPR {
        match val {
            Value::Local(loc) => {
                let scratch = self.block_state.regs.take_scratch_gpr();

                match self.block_state.locals.get(loc) {
                    ValueLocation::Stack(offset) => {
                        let offset = self.adjusted_offset(offset);
                        dynasm!(self.asm
                            ; mov Rq(scratch), [rsp + offset]
                        );
                    }
                    ValueLocation::Reg(reg) => {
                        dynasm!(self.asm
                            ; mov Rq(scratch), Rq(reg)
                        );
                    }
                    ValueLocation::Immediate(_) => {
                        panic!("We shouldn't be storing immediates in locals for now")
                    }
                }

                scratch
            }
            Value::Immediate(i) => {
                let scratch = self.block_state.regs.take_scratch_gpr();

                self.immediate_to_reg(scratch, i);

                scratch
            }
            Value::Temp(reg) => reg,
        }
    }

    unop!(i32_clz, lzcnt, Rd, u32, u32::leading_zeros);
    unop!(i64_clz, lzcnt, Rq, u64, |a: u64| a.leading_zeros() as u64);
    unop!(i32_ctz, tzcnt, Rd, u32, u32::trailing_zeros);
    unop!(i64_ctz, tzcnt, Rq, u64, |a: u64| a.trailing_zeros() as u64);
    unop!(i32_popcnt, popcnt, Rd, u32, u32::count_ones);
    unop!(i64_popcnt, popcnt, Rq, u64, |a: u64| a.count_ones() as u64);

    // TODO: Use `lea` when the LHS operand isn't a temporary but both of the operands
    //       are in registers.
    commutative_binop_i32!(i32_add, add, |a, b| (a as i32).wrapping_add(b as i32));
    commutative_binop_i32!(i32_and, and, |a, b| a & b);
    commutative_binop_i32!(i32_or, or, |a, b| a | b);
    commutative_binop_i32!(i32_xor, xor, |a, b| a ^ b);

    commutative_binop_i64!(i64_add, add, i64::wrapping_add);
    commutative_binop_i64!(i64_and, and, |a, b| a & b);
    commutative_binop_i64!(i64_or, or, |a, b| a | b);
    commutative_binop_i64!(i64_xor, xor, |a, b| a ^ b);

    // `sub` is not commutative, so we have to handle it differently (we _must_ use the `op1`
    // temp register as the output)
    pub fn i64_sub(&mut self) {
        let op0 = self.pop();
        let op1 = self.pop();

        if let Some(i1) = op1.immediate() {
            if let Some(i0) = op0.immediate() {
                self.block_state.stack.push(StackValue::Immediate(i1 - i0));
                return;
            }
        }

        let op1 = self.into_temp_reg(op1);
        match op0.location(&self.block_state.locals) {
            ValueLocation::Reg(reg) => {
                dynasm!(self.asm
                    ; sub Rq(op1), Rq(reg)
                );
            }
            ValueLocation::Stack(offset) => {
                let offset = self.adjusted_offset(offset);
                dynasm!(self.asm
                    ; sub Rq(op1), [rsp + offset]
                );
            }
            ValueLocation::Immediate(i) => {
                if let Some(i) = i.try_into() {
                    dynasm!(self.asm
                        ; sub Rq(op1), i
                    );
                } else {
                    unimplemented!(concat!(
                        "Unsupported `sub` with large 64-bit immediate operand"
                    ));
                }
            }
        }

        self.push(Value::Temp(op1));
        self.free_value(op0);
    }

    // `i64_mul` needs to be seperate because the immediate form of the instruction
    // has a different syntax to the immediate form of the other instructions.
    pub fn i64_mul(&mut self) {
        let op0 = self.pop();
        let op1 = self.pop();

        if let Some(i1) = op1.immediate() {
            if let Some(i0) = op0.immediate() {
                self.block_state
                    .stack
                    .push(StackValue::Immediate(i64::wrapping_mul(i1, i0)));
                return;
            }
        }

        let (op1, op0) = match op1 {
            Value::Temp(reg) => (reg, op0),
            _ => {
                if op0.immediate().is_some() {
                    (self.into_temp_reg(op1), op0)
                } else {
                    (self.into_temp_reg(op0), op1)
                }
            }
        };

        match op0.location(&self.block_state.locals) {
            ValueLocation::Reg(reg) => {
                dynasm!(self.asm
                    ; imul Rq(op1), Rq(reg)
                );
            }
            ValueLocation::Stack(offset) => {
                let offset = self.adjusted_offset(offset);
                dynasm!(self.asm
                    ; imul Rq(op1), [rsp + offset]
                );
            }
            ValueLocation::Immediate(i) => {
                if let Some(i) = i.try_into() {
                    dynasm!(self.asm
                        ; imul Rq(op1), Rq(op1), i
                    );
                } else {
                    unimplemented!(concat!(
                        "Unsupported `imul` with large 64-bit immediate operand"
                    ));
                }
            }
        }

        self.push(Value::Temp(op1));
        self.free_value(op0);
    }

    // `sub` is not commutative, so we have to handle it differently (we _must_ use the `op1`
    // temp register as the output)
    pub fn i32_sub(&mut self) {
        let op0 = self.pop();
        let op1 = self.pop();

        if let Some(i1) = op1.immediate() {
            if let Some(i0) = op0.immediate() {
                self.block_state.stack.push(StackValue::Immediate(i1 - i0));
                return;
            }
        }

        let op1 = self.into_temp_reg(op1);
        match op0.location(&self.block_state.locals) {
            ValueLocation::Reg(reg) => {
                dynasm!(self.asm
                    ; sub Rd(op1), Rd(reg)
                );
            }
            ValueLocation::Stack(offset) => {
                let offset = self.adjusted_offset(offset);
                dynasm!(self.asm
                    ; sub Rd(op1), [rsp + offset]
                );
            }
            ValueLocation::Immediate(i) => {
                dynasm!(self.asm
                    ; sub Rd(op1), i as i32
                );
            }
        }

        self.push(Value::Temp(op1));
        self.free_value(op0);
    }

    // `i32_mul` needs to be seperate because the immediate form of the instruction
    // has a different syntax to the immediate form of the other instructions.
    pub fn i32_mul(&mut self) {
        let op0 = self.pop();
        let op1 = self.pop();

        if let Some(i1) = op1.immediate() {
            if let Some(i0) = op0.immediate() {
                self.block_state
                    .stack
                    .push(StackValue::Immediate(
                        i32::wrapping_mul(i1 as i32, i0 as i32) as _,
                    ));
                return;
            }
        }

        let (op1, op0) = match op1 {
            Value::Temp(reg) => (reg, op0),
            _ => {
                if op0.immediate().is_some() {
                    (self.into_temp_reg(op1), op0)
                } else {
                    (self.into_temp_reg(op0), op1)
                }
            }
        };

        match op0.location(&self.block_state.locals) {
            ValueLocation::Reg(reg) => {
                dynasm!(self.asm
                    ; imul Rd(op1), Rd(reg)
                );
            }
            ValueLocation::Stack(offset) => {
                let offset = self.adjusted_offset(offset);
                dynasm!(self.asm
                    ; imul Rd(op1), [rsp + offset]
                );
            }
            ValueLocation::Immediate(i) => {
                dynasm!(self.asm
                    ; imul Rd(op1), Rd(op1), i as i32
                );
            }
        }

        self.push(Value::Temp(op1));
        self.free_value(op0);
    }

    fn adjusted_local_idx(&self, index: u32) -> u32 {
        if index >= self.block_state.locals.vmctx_index() {
            index + 1
        } else {
            index
        }
    }

    pub fn get_local(&mut self, local_idx: u32) {
        let index = self.adjusted_local_idx(local_idx);
        self.push(Value::Local(index));
    }

    fn local_write_loc(&self, local_idx: u32) -> ValueLocation {
        self.block_state
            .end_locals
            .as_ref()
            .map(|l| l.get(local_idx))
            .or_else(|| {
                self.block_state
                    .locals
                    .register(local_idx)
                    .map(ValueLocation::Reg)
            })
            .unwrap_or_else(|| self.block_state.locals.get(local_idx))
    }

    // TODO: We can put locals that were spilled to the stack
    //       back into registers here.
    pub fn set_local(&mut self, local_idx: u32) {
        let local_idx = self.adjusted_local_idx(local_idx);
        let val = self.pop();
        let val_loc = val.location(&self.block_state.locals);
        let dst_loc = self.local_write_loc(local_idx);

        self.materialize_local(local_idx);

        // TODO: Abstract this somehow
        if let Some(cur) = self
            .block_state
            .locals
            .register_locals
            .get_mut(local_idx as usize)
        {
            *cur = ArgLoc::from_loc(dst_loc);
        }

        self.copy_value(val_loc, dst_loc);
        self.free_value(val);
    }

    pub fn tee_local(&mut self, local_idx: u32) {
        let local_idx = self.adjusted_local_idx(local_idx);
        let val = self.pop();
        let val_loc = val.location(&self.block_state.locals);
        let dst_loc = self.local_write_loc(local_idx);

        self.materialize_local(local_idx);

        if let Some(cur) = self
            .block_state
            .locals
            .register_locals
            .get_mut(local_idx as usize)
        {
            *cur = ArgLoc::from_loc(dst_loc);
        }

        self.copy_value(val_loc, dst_loc);

        match (val_loc, dst_loc) {
            (ValueLocation::Stack(_), ValueLocation::Reg(_)) => {
                self.free_value(val);
                self.block_state.stack.push(StackValue::Local(local_idx))
            }
            _ => self.push(val),
        }
    }

    fn materialize_local(&mut self, local_idx: u32) {
        // TODO: With real stack allocation we can make this constant-time. We can have a kind of
        //       on-the-fly SSA transformation where we mark each `StackValue::Local` with an ID
        //       that increases with each assignment (this can be stored in block state and so
        //       is reset when the block ends). We then refcount the storage associated with each
        //       "value ID" and in `pop` we free up slots whose refcount hits 0. This means we
        //       can have even cleaner assembly than we currently do while giving us back
        //       linear runtime.
        let mut highest_stack_index = None;
        let mut highest_pop_index = None;

        for index in (0..self.block_state.stack.len()).rev() {
            match self.block_state.stack[index] {
                // For now it's impossible for a local to be in RAX but that might be
                // possible in the future, so we check both cases.
                StackValue::Local(i) if i == local_idx => {
                    if highest_stack_index.is_none() {
                        highest_stack_index = Some(index);
                    }

                    self.block_state.depth.reserve(1);
                    self.block_state.stack[index] = StackValue::Pop;
                    match self.block_state.locals.get(local_idx) {
                        ValueLocation::Reg(r) => dynasm!(self.asm
                            ; push Rq(r)
                        ),
                        ValueLocation::Stack(offset) => {
                            let offset = self.adjusted_offset(offset);
                            dynasm!(self.asm
                                ; push QWORD [rsp + offset]
                            )
                        }
                        _ => unreachable!(),
                    }
                }
                StackValue::Pop => {
                    if highest_pop_index.is_none() {
                        highest_pop_index = Some(index);
                    }
                }
                _ => {}
            }
        }

        if let (Some(stack), Some(pop)) = (highest_stack_index, highest_pop_index) {
            if stack < pop {
                panic!("Tried to materialize local but the stack already contains elements");
            }
        }
    }

    pub fn i32_literal(&mut self, imm: i32) {
        self.push(Value::Immediate(imm as _));
    }

    pub fn i64_literal(&mut self, imm: i64) {
        self.push(Value::Immediate(imm));
    }

    /// Make sure that any argument registers that will be used by the call are free
    /// by storing them to the stack.
    ///
    /// Unfortunately, we can't elide this store if we're just passing arguments on
    /// because these registers are caller-saved and so the callee can use them as
    /// scratch space.
    fn free_arg_registers(&mut self, exclude: Option<u32>) {
        // This is bound to the maximum size of the `ArrayVec` amd so can be considered to have constant
        // runtime
        for i in (0..self.block_state.locals.register_locals.len())
            .filter(|i| exclude != Some(*i as u32))
        {
            match self.block_state.locals.register_locals[i] {
                ArgLoc::Register(reg) => {
                    if ARGS_IN_GPRS.contains(&reg) {
                        let offset =
                            ((self.block_state.locals.num_local_stack_slots - 1 - i as u32)
                                * WORD_SIZE) as _;
                        let dst = ValueLocation::Stack(offset);
                        self.copy_value(ValueLocation::Reg(reg), dst);
                        self.block_state.locals.register_locals[i].add_stack(offset);
                    }
                }
                _ => {}
            }
        }
    }

    fn free_return_register(&mut self, count: u32) {
        if count == 0 {
            return;
        }

        self.free_register(RAX);
    }

    fn free_register(&mut self, reg: GPR) {
        let mut to_repush = 0;
        let mut out = None;

        if self.block_state.regs.is_free(reg) {
            return;
        }

        // TODO: With real stack allocation we can make this constant-time
        for stack_val in self.block_state.stack.iter_mut().rev() {
            match stack_val.location(&self.block_state.locals) {
                // For now it's impossible for a local to be in RAX but that might be
                // possible in the future, so we check both cases.
                Some(ValueLocation::Reg(r)) if r == reg => {
                    *stack_val = StackValue::Pop;

                    out = Some(*stack_val);

                    break;
                }
                Some(_) => {}
                None => {
                    to_repush += 1;
                }
            }
        }

        if let Some(out) = out {
            match out {
                StackValue::Temp(gpr) => {
                    dynasm!(self.asm
                        ; mov Rq(gpr), rax
                    );
                }
                StackValue::Pop => {
                    self.block_state.depth.reserve(1);
                    // TODO: Ideally we should do proper stack allocation so we
                    //       don't have to check this at all (i.e. order on the
                    //       physical stack and order on the logical stack should
                    //       be independent).
                    debug_assert_eq!(to_repush, 0);
                    dynasm!(self.asm
                        ; push Rq(reg)
                    );
                }
                _ => unreachable!(),
            }
            self.block_state.regs.release_scratch_gpr(reg);
        }
    }

    // TODO: Use `ArrayVec`?
    /// Saves volatile (i.e. caller-saved) registers before a function call, if they are used.
    fn save_volatile(&mut self) -> ArrayVec<[GPR; SCRATCH_REGS.len()]> {
        let mut out = ArrayVec::new();

        for &reg in SCRATCH_REGS.iter() {
            if self
                .block_state
                .stack
                .iter()
                .filter_map(|v| v.location(&self.block_state.locals))
                .any(|p| p == ValueLocation::Reg(reg))
            {
                dynasm!(self.asm
                    ; push Rq(reg)
                );
                out.push(reg);
            }
        }

        out
    }

    /// Write the arguments to the callee to the registers and the stack using the SystemV
    /// calling convention.
    fn pass_outgoing_args(
        &mut self,
        arity: u32,
        return_arity: u32,
        has_vmctx: bool,
    ) -> CallCleanup {
        let num_stack_args = (arity as usize).saturating_sub(ARGS_IN_GPRS.len()) as i32;

        self.free_arg_registers(if has_vmctx { Some(0) } else { None });

        // We pop stack arguments first - arguments are RTL
        if num_stack_args > 0 {
            let size = num_stack_args * WORD_SIZE as i32;

            // Reserve space for the outgoing stack arguments (so we don't
            // stomp on any locals or the value stack).
            dynasm!(self.asm
                ; sub rsp, size
            );
            self.block_state.depth.reserve(num_stack_args as u32);

            for stack_slot in (0..num_stack_args).rev() {
                // Since the stack offset is from the bottom of the locals
                // and we want to start from the actual RSP (so `offset = 0`
                // writes to `[rsp]`), we subtract our current depth.
                //
                // We might want to do this in the future by having a separate
                // `AbsoluteValueLocation` and `RelativeValueLocation`.
                let offset = stack_slot * WORD_SIZE as i32
                    - self.block_state.depth.0 as i32 * WORD_SIZE as i32;
                self.pop_into(ValueLocation::Stack(offset));
            }
        }

        for reg in ARGS_IN_GPRS[..(arity as usize).min(ARGS_IN_GPRS.len())]
            .iter()
            .rev()
        {
            self.pop_into(ValueLocation::Reg(*reg));
        }

        // We do this before doing `save_volatile`, since otherwise we'll trample the return value
        // of the call when we pop back.
        self.free_return_register(return_arity);

        CallCleanup {
            stack_depth: num_stack_args,
            restore_registers: self.save_volatile(),
        }
    }

    /// Frees up the stack space used for stack-passed arguments and restores the value
    /// of volatile (i.e. caller-saved) registers to the state that they were in before
    /// the call.
    fn post_call_cleanup(&mut self, mut cleanup: CallCleanup) {
        if cleanup.stack_depth > 0 {
            let size = cleanup.stack_depth * WORD_SIZE as i32;
            self.block_state.depth.free(cleanup.stack_depth as _);
            dynasm!(self.asm
                ; add rsp, size
            );
        }

        // If these values were in register they've now been invalidated, since
        // the callee can use them as scratch.
        for loc in self.block_state.locals.register_locals.iter_mut() {
            if let Some(offset) = loc.stack() {
                *loc = ArgLoc::Stack(offset);
            }
        }

        for reg in cleanup.restore_registers.drain(..).rev() {
            dynasm!(self.asm
                ; pop Rq(reg)
            );
        }
    }

    fn push_function_return(&mut self, arity: u32) {
        if arity == 0 {
            return;
        }
        debug_assert_eq!(arity, 1);
        self.block_state.regs.mark_used(RAX);
        self.push(Value::Temp(RAX));
    }

    pub fn call_indirect(&mut self, signature_hash: u32, arg_arity: u32, return_arity: u32) {
        debug_assert!(
            return_arity == 0 || return_arity == 1,
            "We don't support multiple return yet"
        );

        let callee = self.pop();
        let callee = self.into_temp_reg(callee);

        let vmctx_idx = self.block_state.locals.vmctx_index();
        let (vmctx_reg, should_release_vmctx_reg) = self.into_reg(Value::Local(vmctx_idx));

        let signature_matches = self.create_label();
        let temp0 = self.block_state.regs.take_scratch_gpr();
        let temp1 = self.block_state.regs.take_scratch_gpr();

        dynasm!(self.asm
            ; imul Rq(callee), Rq(callee), mem::size_of::<RuntimeFunc>() as i32
            ; mov Rq(temp0), [Rq(vmctx_reg) + VmCtx::offset_of_funcs_ptr() as i32]
            ; mov Rd(temp1), [
                Rq(temp0) +
                    Rq(callee) +
                    RuntimeFunc::offset_of_sig_hash() as i32
            ]
            ; cmp Rd(temp1), signature_hash as i32
            ; je =>signature_matches.0
        );

        self.trap();

        self.define_label(signature_matches);
        self.block_state.regs.release_scratch_gpr(temp1);

        // TODO: I believe that this can't cause quadratic runtime but I'm not
        //       certain.
        let vmctx = StackValue::Local(vmctx_idx);

        let count = self.block_state.stack.len();
        self.block_state
            .stack
            .insert(count - arg_arity as usize, vmctx);
        let cleanup = self.pass_outgoing_args(arg_arity + 1, return_arity, true);

        dynasm!(self.asm
            ; call QWORD [
                Rq(temp0) +
                    Rq(callee) +
                    RuntimeFunc::offset_of_func_start() as i32
            ]
        );

        self.block_state.regs.release_scratch_gpr(temp0);
        self.block_state.regs.release_scratch_gpr(callee);

        if should_release_vmctx_reg {
            self.block_state.regs.release_scratch_gpr(vmctx_reg);
        }

        self.post_call_cleanup(cleanup);
        self.push_function_return(return_arity);
    }

    /// Call a function with the given index
    pub fn call_direct(&mut self, index: u32, arg_arity: u32, return_arity: u32) {
        debug_assert!(
            return_arity == 0 || return_arity == 1,
            "We don't support multiple return yet"
        );

        let vmctx = StackValue::Local(self.block_state.locals.vmctx_index());
        let count = self.block_state.stack.len();

        // TODO: I believe that this can't cause quadratic runtime but I'm not
        //       certain.
        self.block_state
            .stack
            .insert(count - arg_arity as usize, vmctx);
        let cleanup = self.pass_outgoing_args(arg_arity + 1, return_arity, true);

        let label = &self.func_starts[index as usize].1;
        dynasm!(self.asm
            ; call =>*label
        );

        self.post_call_cleanup(cleanup);
        self.push_function_return(return_arity);
    }

    // TODO: Reserve space to store RBX, RBP, and R12..R15 so we can use them
    //       as scratch registers
    // TODO: Allow use of unused argument registers as scratch registers.
    /// Writes the function prologue and stores the arguments as locals
    pub fn start_function(&mut self, arguments: u32, locals: u32) -> FunctionEnd {
        // To support `vmctx`
        let arguments = arguments + 1;
        let (reg_args, locals_in_gprs) =
            ARGS_IN_GPRS.split_at((arguments as usize).min(ARGS_IN_GPRS.len()));
        let (reg_locals, temps) =
            locals_in_gprs.split_at((locals as usize).min(locals_in_gprs.len()));

        for temp in temps {
            self.block_state.regs.release_scratch_gpr(*temp);
        }

        // We need space to store the register arguments if we need to call a function
        // and overwrite these registers so we add `reg_args.len()`
        let stack_slots = locals + reg_args.len() as u32;
        // Align stack slots to the nearest even number. This is required
        // by x86-64 ABI.
        let aligned_stack_slots = (stack_slots + 1) & !1;
        let frame_size: i32 = aligned_stack_slots as i32 * WORD_SIZE as i32;

        self.block_state.locals.register_locals = reg_args
            .iter()
            .chain(reg_locals)
            .cloned()
            .map(ArgLoc::Register)
            .collect();
        self.block_state.locals.num_stack_args = arguments.saturating_sub(ARGS_IN_GPRS.len() as _);
        self.block_state.locals.num_local_stack_slots = stack_slots;
        self.block_state.return_register = Some(RAX);

        // self.block_state.depth.reserve(aligned_stack_slots - locals);
        let should_generate_epilogue = frame_size > 0;
        if should_generate_epilogue {
            dynasm!(self.asm
                ; push rbp
                ; mov rbp, rsp
                ; sub rsp, frame_size
            );
        }

        FunctionEnd {
            should_generate_epilogue,
        }
    }

    /// Writes the function epilogue, restoring the stack pointer and returning to the
    /// caller.
    pub fn epilogue(&mut self, func: FunctionEnd) {
        // We don't need to clean up the stack - RSP is restored and
        // the calling function has its own register stack and will
        // stomp on the registers from our stack if necessary.
        if func.should_generate_epilogue {
            dynasm!(self.asm
                ; mov rsp, rbp
                ; pop rbp
            );
        }

        dynasm!(self.asm
            ; ret
        );
    }

    pub fn trap(&mut self) {
        dynasm!(self.asm
            ; ud2
        );
    }
}

