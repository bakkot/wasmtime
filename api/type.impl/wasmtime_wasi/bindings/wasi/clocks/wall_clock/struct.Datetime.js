(function() {var type_impls = {
"wasmtime_wasi":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Clone-for-Datetime\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime_wasi/lib.rs.html#86-169\">source</a><a href=\"#impl-Clone-for-Datetime\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> for <a class=\"struct\" href=\"wasmtime_wasi/bindings/wasi/clocks/wall_clock/struct.Datetime.html\" title=\"struct wasmtime_wasi::bindings::wasi::clocks::wall_clock::Datetime\">Datetime</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmtime_wasi/lib.rs.html#86-169\">source</a><a href=\"#method.clone\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\" class=\"fn\">clone</a>(&amp;self) -&gt; <a class=\"struct\" href=\"wasmtime_wasi/bindings/wasi/clocks/wall_clock/struct.Datetime.html\" title=\"struct wasmtime_wasi::bindings::wasi::clocks::wall_clock::Datetime\">Datetime</a></h4></section></summary><div class='docblock'>Returns a copy of the value. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#tymethod.clone\">Read more</a></div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.clone_from\" class=\"method trait-impl\"><span class=\"rightside\"><span class=\"since\" title=\"Stable since Rust version 1.0.0\">1.0.0</span> · <a class=\"src\" href=\"https://doc.rust-lang.org/nightly/src/core/clone.rs.html#169\">source</a></span><a href=\"#method.clone_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\" class=\"fn\">clone_from</a>(&amp;mut self, source: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;Self</a>)</h4></section></summary><div class='docblock'>Performs copy-assignment from <code>source</code>. <a href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html#method.clone_from\">Read more</a></div></details></div></details>","Clone","wasmtime_wasi::bindings::sync_io::_internal::wasi::filesystem::types::Datetime","wasmtime_wasi::bindings::wasi::filesystem::types::Datetime"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Debug-for-Datetime\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime_wasi/lib.rs.html#86-169\">source</a><a href=\"#impl-Debug-for-Datetime\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html\" title=\"trait core::fmt::Debug\">Debug</a> for <a class=\"struct\" href=\"wasmtime_wasi/bindings/wasi/clocks/wall_clock/struct.Datetime.html\" title=\"struct wasmtime_wasi::bindings::wasi::clocks::wall_clock::Datetime\">Datetime</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.fmt\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmtime_wasi/lib.rs.html#86-169\">source</a><a href=\"#method.fmt\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\" class=\"fn\">fmt</a>(&amp;self, f: &amp;mut <a class=\"struct\" href=\"https://doc.rust-lang.org/nightly/core/fmt/struct.Formatter.html\" title=\"struct core::fmt::Formatter\">Formatter</a>&lt;'_&gt;) -&gt; <a class=\"type\" href=\"https://doc.rust-lang.org/nightly/core/fmt/type.Result.html\" title=\"type core::fmt::Result\">Result</a></h4></section></summary><div class='docblock'>Formats the value using the given formatter. <a href=\"https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#tymethod.fmt\">Read more</a></div></details></div></details>","Debug","wasmtime_wasi::bindings::sync_io::_internal::wasi::filesystem::types::Datetime","wasmtime_wasi::bindings::wasi::filesystem::types::Datetime"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-TryFrom%3CSystemTime%3E-for-Datetime\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime_wasi/host/clocks.rs.html#13-25\">source</a><a href=\"#impl-TryFrom%3CSystemTime%3E-for-Datetime\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html\" title=\"trait core::convert::TryFrom\">TryFrom</a>&lt;SystemTime&gt; for <a class=\"struct\" href=\"wasmtime_wasi/bindings/wasi/clocks/wall_clock/struct.Datetime.html\" title=\"struct wasmtime_wasi::bindings::wasi::clocks::wall_clock::Datetime\">Datetime</a></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Error\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Error\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html#associatedtype.Error\" class=\"associatedtype\">Error</a> = <a class=\"struct\" href=\"https://docs.rs/anyhow/1.0.75/anyhow/struct.Error.html\" title=\"struct anyhow::Error\">Error</a></h4></section></summary><div class='docblock'>The type returned in the event of a conversion error.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.try_from\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmtime_wasi/host/clocks.rs.html#16-24\">source</a><a href=\"#method.try_from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html#tymethod.try_from\" class=\"fn\">try_from</a>(time: SystemTime) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;Self, Self::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.TryFrom.html#associatedtype.Error\" title=\"type core::convert::TryFrom::Error\">Error</a>&gt;</h4></section></summary><div class='docblock'>Performs the conversion.</div></details></div></details>","TryFrom<SystemTime>","wasmtime_wasi::bindings::sync_io::_internal::wasi::filesystem::types::Datetime","wasmtime_wasi::bindings::wasi::filesystem::types::Datetime"],["<section id=\"impl-ComponentType-for-Datetime\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime_wasi/lib.rs.html#86-169\">source</a><a href=\"#impl-ComponentType-for-Datetime\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"wasmtime/runtime/component/func/typed/trait.ComponentType.html\" title=\"trait wasmtime::runtime::component::func::typed::ComponentType\">ComponentType</a> for <a class=\"struct\" href=\"wasmtime_wasi/bindings/wasi/clocks/wall_clock/struct.Datetime.html\" title=\"struct wasmtime_wasi::bindings::wasi::clocks::wall_clock::Datetime\">Datetime</a></h3></section>","ComponentType","wasmtime_wasi::bindings::sync_io::_internal::wasi::filesystem::types::Datetime","wasmtime_wasi::bindings::wasi::filesystem::types::Datetime"],["<section id=\"impl-Copy-for-Datetime\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime_wasi/lib.rs.html#86-169\">source</a><a href=\"#impl-Copy-for-Datetime\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/marker/trait.Copy.html\" title=\"trait core::marker::Copy\">Copy</a> for <a class=\"struct\" href=\"wasmtime_wasi/bindings/wasi/clocks/wall_clock/struct.Datetime.html\" title=\"struct wasmtime_wasi::bindings::wasi::clocks::wall_clock::Datetime\">Datetime</a></h3></section>","Copy","wasmtime_wasi::bindings::sync_io::_internal::wasi::filesystem::types::Datetime","wasmtime_wasi::bindings::wasi::filesystem::types::Datetime"],["<section id=\"impl-Lift-for-Datetime\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime_wasi/lib.rs.html#86-169\">source</a><a href=\"#impl-Lift-for-Datetime\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"wasmtime/runtime/component/func/typed/trait.Lift.html\" title=\"trait wasmtime::runtime::component::func::typed::Lift\">Lift</a> for <a class=\"struct\" href=\"wasmtime_wasi/bindings/wasi/clocks/wall_clock/struct.Datetime.html\" title=\"struct wasmtime_wasi::bindings::wasi::clocks::wall_clock::Datetime\">Datetime</a></h3></section>","Lift","wasmtime_wasi::bindings::sync_io::_internal::wasi::filesystem::types::Datetime","wasmtime_wasi::bindings::wasi::filesystem::types::Datetime"],["<section id=\"impl-Lower-for-Datetime\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime_wasi/lib.rs.html#86-169\">source</a><a href=\"#impl-Lower-for-Datetime\" class=\"anchor\">§</a><h3 class=\"code-header\">impl <a class=\"trait\" href=\"wasmtime/runtime/component/func/typed/trait.Lower.html\" title=\"trait wasmtime::runtime::component::func::typed::Lower\">Lower</a> for <a class=\"struct\" href=\"wasmtime_wasi/bindings/wasi/clocks/wall_clock/struct.Datetime.html\" title=\"struct wasmtime_wasi::bindings::wasi::clocks::wall_clock::Datetime\">Datetime</a></h3></section>","Lower","wasmtime_wasi::bindings::sync_io::_internal::wasi::filesystem::types::Datetime","wasmtime_wasi::bindings::wasi::filesystem::types::Datetime"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()