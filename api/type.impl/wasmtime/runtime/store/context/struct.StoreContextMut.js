(function() {var type_impls = {
"wasmtime_c_api":[["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-AsContext-for-StoreContextMut%3C'_,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store/context.rs.html#148\">source</a><a href=\"#impl-AsContext-for-StoreContextMut%3C'_,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"trait\" href=\"wasmtime/runtime/store/context/trait.AsContext.html\" title=\"trait wasmtime::runtime::store::context::AsContext\">AsContext</a> for <a class=\"struct\" href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html\" title=\"struct wasmtime::runtime::store::context::StoreContextMut\">StoreContextMut</a>&lt;'_, T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Data\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Data\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"wasmtime/runtime/store/context/trait.AsContext.html#associatedtype.Data\" class=\"associatedtype\">Data</a> = T</h4></section></summary><div class='docblock'>The host information associated with the <a href=\"wasmtime/runtime/store/struct.Store.html\" title=\"struct wasmtime::runtime::store::Store\"><code>Store</code></a>, aka the <code>T</code> in\n<a href=\"wasmtime/runtime/store/struct.Store.html\" title=\"struct wasmtime::runtime::store::Store\"><code>Store&lt;T&gt;</code></a>.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_context\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store/context.rs.html#152\">source</a><a href=\"#method.as_context\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmtime/runtime/store/context/trait.AsContext.html#tymethod.as_context\" class=\"fn\">as_context</a>(&amp;self) -&gt; <a class=\"struct\" href=\"wasmtime/runtime/store/context/struct.StoreContext.html\" title=\"struct wasmtime::runtime::store::context::StoreContext\">StoreContext</a>&lt;'_, T&gt;</h4></section></summary><div class='docblock'>Returns the store context that this type provides access to.</div></details></div></details>","AsContext","wasmtime_c_api::store::CStoreContextMut"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-AsContextMut-for-StoreContextMut%3C'_,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store/context.rs.html#157\">source</a><a href=\"#impl-AsContextMut-for-StoreContextMut%3C'_,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;T&gt; <a class=\"trait\" href=\"wasmtime/runtime/store/context/trait.AsContextMut.html\" title=\"trait wasmtime::runtime::store::context::AsContextMut\">AsContextMut</a> for <a class=\"struct\" href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html\" title=\"struct wasmtime::runtime::store::context::StoreContextMut\">StoreContextMut</a>&lt;'_, T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.as_context_mut\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store/context.rs.html#159\">source</a><a href=\"#method.as_context_mut\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"wasmtime/runtime/store/context/trait.AsContextMut.html#tymethod.as_context_mut\" class=\"fn\">as_context_mut</a>(&amp;mut self) -&gt; <a class=\"struct\" href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html\" title=\"struct wasmtime::runtime::store::context::StoreContextMut\">StoreContextMut</a>&lt;'_, T&gt;</h4></section></summary><div class='docblock'>Returns the store context that this type provides access to.</div></details></div></details>","AsContextMut","wasmtime_c_api::store::CStoreContextMut"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-From%3C%26mut+T%3E-for-StoreContextMut%3C'a,+%3CT+as+AsContext%3E::Data%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store/context.rs.html#239\">source</a><a href=\"#impl-From%3C%26mut+T%3E-for-StoreContextMut%3C'a,+%3CT+as+AsContext%3E::Data%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html\" title=\"trait core::convert::From\">From</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;'a mut T</a>&gt; for <a class=\"struct\" href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html\" title=\"struct wasmtime::runtime::store::context::StoreContextMut\">StoreContextMut</a>&lt;'a, &lt;T as <a class=\"trait\" href=\"wasmtime/runtime/store/context/trait.AsContext.html\" title=\"trait wasmtime::runtime::store::context::AsContext\">AsContext</a>&gt;::<a class=\"associatedtype\" href=\"wasmtime/runtime/store/context/trait.AsContext.html#associatedtype.Data\" title=\"type wasmtime::runtime::store::context::AsContext::Data\">Data</a>&gt;<div class=\"where\">where\n    T: <a class=\"trait\" href=\"wasmtime/runtime/store/context/trait.AsContextMut.html\" title=\"trait wasmtime::runtime::store::context::AsContextMut\">AsContextMut</a>,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.from\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store/context.rs.html#240\">source</a><a href=\"#method.from\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/convert/trait.From.html#tymethod.from\" class=\"fn\">from</a>(t: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;'a mut T</a>) -&gt; <a class=\"struct\" href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html\" title=\"struct wasmtime::runtime::store::context::StoreContextMut\">StoreContextMut</a>&lt;'a, &lt;T as <a class=\"trait\" href=\"wasmtime/runtime/store/context/trait.AsContext.html\" title=\"trait wasmtime::runtime::store::context::AsContext\">AsContext</a>&gt;::<a class=\"associatedtype\" href=\"wasmtime/runtime/store/context/trait.AsContext.html#associatedtype.Data\" title=\"type wasmtime::runtime::store::context::AsContext::Data\">Data</a>&gt;</h4></section></summary><div class='docblock'>Converts to this type from the input type.</div></details></div></details>","From<&'a mut T>","wasmtime_c_api::store::CStoreContextMut"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-Index%3CI%3E-for-StoreContextMut%3C'_,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store/data.rs.html#163-165\">source</a><a href=\"#impl-Index%3CI%3E-for-StoreContextMut%3C'_,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;I, T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;I&gt; for <a class=\"struct\" href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html\" title=\"struct wasmtime::runtime::store::context::StoreContextMut\">StoreContextMut</a>&lt;'_, T&gt;<div class=\"where\">where\n    StoreData: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;I&gt;,</div></h3></section></summary><div class=\"impl-items\"><details class=\"toggle\" open><summary><section id=\"associatedtype.Output\" class=\"associatedtype trait-impl\"><a href=\"#associatedtype.Output\" class=\"anchor\">§</a><h4 class=\"code-header\">type <a href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html#associatedtype.Output\" class=\"associatedtype\">Output</a> = &lt;StoreData as <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;I&gt;&gt;::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html#associatedtype.Output\" title=\"type core::ops::index::Index::Output\">Output</a></h4></section></summary><div class='docblock'>The returned type after indexing.</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.index\" class=\"method trait-impl\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store/data.rs.html#170\">source</a><a href=\"#method.index\" class=\"anchor\">§</a><h4 class=\"code-header\">fn <a href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html#tymethod.index\" class=\"fn\">index</a>(&amp;self, index: I) -&gt; &amp;&lt;<a class=\"struct\" href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html\" title=\"struct wasmtime::runtime::store::context::StoreContextMut\">StoreContextMut</a>&lt;'_, T&gt; as <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html\" title=\"trait core::ops::index::Index\">Index</a>&lt;I&gt;&gt;::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html#associatedtype.Output\" title=\"type core::ops::index::Index::Output\">Output</a></h4></section></summary><div class='docblock'>Performs the indexing (<code>container[index]</code>) operation. <a href=\"https://doc.rust-lang.org/nightly/core/ops/index/trait.Index.html#tymethod.index\">Read more</a></div></details></div></details>","Index<I>","wasmtime_c_api::store::CStoreContextMut"],["<details class=\"toggle implementors-toggle\" open><summary><section id=\"impl-StoreContextMut%3C'a,+T%3E\" class=\"impl\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store.rs.html#1019\">source</a><a href=\"#impl-StoreContextMut%3C'a,+T%3E\" class=\"anchor\">§</a><h3 class=\"code-header\">impl&lt;'a, T&gt; <a class=\"struct\" href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html\" title=\"struct wasmtime::runtime::store::context::StoreContextMut\">StoreContextMut</a>&lt;'a, T&gt;</h3></section></summary><div class=\"impl-items\"><details class=\"toggle method-toggle\" open><summary><section id=\"method.data\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store.rs.html#1023\">source</a><h4 class=\"code-header\">pub fn <a href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html#tymethod.data\" class=\"fn\">data</a>(&amp;self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;T</a></h4></section></summary><div class=\"docblock\"><p>Access the underlying data owned by this <code>Store</code>.</p>\n<p>Same as <a href=\"wasmtime/runtime/store/struct.Store.html#method.data\" title=\"method wasmtime::runtime::store::Store::data\"><code>Store::data</code></a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.data_mut\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store.rs.html#1030\">source</a><h4 class=\"code-header\">pub fn <a href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html#tymethod.data_mut\" class=\"fn\">data_mut</a>(&amp;mut self) -&gt; <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.reference.html\">&amp;mut T</a></h4></section></summary><div class=\"docblock\"><p>Access the underlying data owned by this <code>Store</code>.</p>\n<p>Same as <a href=\"wasmtime/runtime/store/struct.Store.html#method.data_mut\" title=\"method wasmtime::runtime::store::Store::data_mut\"><code>Store::data_mut</code></a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.engine\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store.rs.html#1035\">source</a><h4 class=\"code-header\">pub fn <a href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html#tymethod.engine\" class=\"fn\">engine</a>(&amp;self) -&gt; &amp;<a class=\"struct\" href=\"wasmtime/engine/struct.Engine.html\" title=\"struct wasmtime::engine::Engine\">Engine</a></h4></section></summary><div class=\"docblock\"><p>Returns the underlying <a href=\"wasmtime/engine/struct.Engine.html\" title=\"struct wasmtime::engine::Engine\"><code>Engine</code></a> this store is connected to.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.gc\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store.rs.html#1045\">source</a><h4 class=\"code-header\">pub fn <a href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html#tymethod.gc\" class=\"fn\">gc</a>(&amp;mut self)</h4></section></summary><div class=\"docblock\"><p>Perform garbage collection of <code>ExternRef</code>s.</p>\n<p>Same as <a href=\"wasmtime/runtime/store/struct.Store.html#method.gc\" title=\"method wasmtime::runtime::store::Store::gc\"><code>Store::gc</code></a>.</p>\n<p>This method is only available when the <code>gc</code> Cargo feature is enabled.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.get_fuel\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store.rs.html#1052\">source</a><h4 class=\"code-header\">pub fn <a href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html#tymethod.get_fuel\" class=\"fn\">get_fuel</a>(&amp;self) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u64.html\">u64</a>, <a class=\"struct\" href=\"https://docs.rs/anyhow/1.0.75/anyhow/struct.Error.html\" title=\"struct anyhow::Error\">Error</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Returns remaining fuel in this store.</p>\n<p>For more information see <a href=\"wasmtime/runtime/store/struct.Store.html#method.get_fuel\" title=\"method wasmtime::runtime::store::Store::get_fuel\"><code>Store::get_fuel</code></a></p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.set_fuel\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store.rs.html#1059\">source</a><h4 class=\"code-header\">pub fn <a href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html#tymethod.set_fuel\" class=\"fn\">set_fuel</a>(&amp;mut self, fuel: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u64.html\">u64</a>) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://docs.rs/anyhow/1.0.75/anyhow/struct.Error.html\" title=\"struct anyhow::Error\">Error</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Set the amount of fuel in this store.</p>\n<p>For more information see <a href=\"wasmtime/runtime/store/struct.Store.html#method.set_fuel\" title=\"method wasmtime::runtime::store::Store::set_fuel\"><code>Store::set_fuel</code></a></p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.fuel_async_yield_interval\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store.rs.html#1066\">source</a><h4 class=\"code-header\">pub fn <a href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html#tymethod.fuel_async_yield_interval\" class=\"fn\">fuel_async_yield_interval</a>(\n    &amp;mut self,\n    interval: <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/option/enum.Option.html\" title=\"enum core::option::Option\">Option</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u64.html\">u64</a>&gt;\n) -&gt; <a class=\"enum\" href=\"https://doc.rust-lang.org/nightly/core/result/enum.Result.html\" title=\"enum core::result::Result\">Result</a>&lt;<a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.unit.html\">()</a>, <a class=\"struct\" href=\"https://docs.rs/anyhow/1.0.75/anyhow/struct.Error.html\" title=\"struct anyhow::Error\">Error</a>&gt;</h4></section></summary><div class=\"docblock\"><p>Configures this <code>Store</code> to periodically yield while executing futures.</p>\n<p>For more information see <a href=\"wasmtime/runtime/store/struct.Store.html#method.fuel_async_yield_interval\" title=\"method wasmtime::runtime::store::Store::fuel_async_yield_interval\"><code>Store::fuel_async_yield_interval</code></a></p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.set_epoch_deadline\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store.rs.html#1073\">source</a><h4 class=\"code-header\">pub fn <a href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html#tymethod.set_epoch_deadline\" class=\"fn\">set_epoch_deadline</a>(&amp;mut self, ticks_beyond_current: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u64.html\">u64</a>)</h4></section></summary><div class=\"docblock\"><p>Sets the epoch deadline to a certain number of ticks in the future.</p>\n<p>For more information see <a href=\"wasmtime/runtime/store/struct.Store.html#method.set_epoch_deadline\" title=\"method wasmtime::runtime::store::Store::set_epoch_deadline\"><code>Store::set_epoch_deadline</code></a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.epoch_deadline_trap\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store.rs.html#1080\">source</a><h4 class=\"code-header\">pub fn <a href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html#tymethod.epoch_deadline_trap\" class=\"fn\">epoch_deadline_trap</a>(&amp;mut self)</h4></section></summary><div class=\"docblock\"><p>Configures epoch-deadline expiration to trap.</p>\n<p>For more information see <a href=\"wasmtime/runtime/store/struct.Store.html#method.epoch_deadline_trap\" title=\"method wasmtime::runtime::store::Store::epoch_deadline_trap\"><code>Store::epoch_deadline_trap</code></a>.</p>\n</div></details><details class=\"toggle method-toggle\" open><summary><section id=\"method.epoch_deadline_async_yield_and_update\" class=\"method\"><a class=\"src rightside\" href=\"src/wasmtime/runtime/store.rs.html#1091\">source</a><h4 class=\"code-header\">pub fn <a href=\"wasmtime/runtime/store/context/struct.StoreContextMut.html#tymethod.epoch_deadline_async_yield_and_update\" class=\"fn\">epoch_deadline_async_yield_and_update</a>(&amp;mut self, delta: <a class=\"primitive\" href=\"https://doc.rust-lang.org/nightly/std/primitive.u64.html\">u64</a>)</h4></section></summary><div class=\"docblock\"><p>Configures epoch-deadline expiration to yield to the async\ncaller and the update the deadline.</p>\n<p>For more information see\n<a href=\"wasmtime/runtime/store/struct.Store.html#method.epoch_deadline_async_yield_and_update\" title=\"method wasmtime::runtime::store::Store::epoch_deadline_async_yield_and_update\"><code>Store::epoch_deadline_async_yield_and_update</code></a>.</p>\n</div></details></div></details>",0,"wasmtime_c_api::store::CStoreContextMut"]]
};if (window.register_type_impls) {window.register_type_impls(type_impls);} else {window.pending_type_impls = type_impls;}})()