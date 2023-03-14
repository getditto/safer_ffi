(function() {var implementors = {
"futures_executor":[["impl&lt;S:&nbsp;<a class=\"trait\" href=\"futures_core/stream/trait.Stream.html\" title=\"trait futures_core::stream::Stream\">Stream</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"futures_executor/struct.BlockingStream.html\" title=\"struct futures_executor::BlockingStream\">BlockingStream</a>&lt;S&gt;"]],
"futures_task":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"futures_task/struct.WakerRef.html\" title=\"struct futures_task::WakerRef\">WakerRef</a>&lt;'_&gt;"]],
"futures_util":[["impl&lt;T:&nbsp;?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"futures_util/lock/struct.OwnedMutexGuard.html\" title=\"struct futures_util::lock::OwnedMutexGuard\">OwnedMutexGuard</a>&lt;T&gt;"],["impl&lt;T:&nbsp;?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"futures_util/lock/struct.MutexGuard.html\" title=\"struct futures_util::lock::MutexGuard\">MutexGuard</a>&lt;'_, T&gt;"],["impl&lt;T:&nbsp;?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>, U:&nbsp;?<a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/marker/trait.Sized.html\" title=\"trait core::marker::Sized\">Sized</a>&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"futures_util/lock/struct.MappedMutexGuard.html\" title=\"struct futures_util::lock::MappedMutexGuard\">MappedMutexGuard</a>&lt;'_, T, U&gt;"]],
"once_cell":[["impl&lt;T, F:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/function/trait.FnOnce.html\" title=\"trait core::ops::function::FnOnce\">FnOnce</a>() -&gt; T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"once_cell/unsync/struct.Lazy.html\" title=\"struct once_cell::unsync::Lazy\">Lazy</a>&lt;T, F&gt;"],["impl&lt;T, F:&nbsp;<a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/function/trait.FnOnce.html\" title=\"trait core::ops::function::FnOnce\">FnOnce</a>() -&gt; T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"once_cell/sync/struct.Lazy.html\" title=\"struct once_cell::sync::Lazy\">Lazy</a>&lt;T, F&gt;"]],
"safer_ffi":[["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/layout/struct.Opaque.html\" title=\"struct safer_ffi::layout::Opaque\">Opaque</a>&lt;T&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/boxed/struct.Box_.html\" title=\"struct safer_ffi::boxed::Box_\">Box_</a>&lt;T&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/ptr/struct.NonNullRef.html\" title=\"struct safer_ffi::ptr::NonNullRef\">NonNullRef</a>&lt;T&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/ptr/struct.NonNullMut.html\" title=\"struct safer_ffi::ptr::NonNullMut\">NonNullMut</a>&lt;T&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/ptr/struct.NonNullOwned.html\" title=\"struct safer_ffi::ptr::NonNullOwned\">NonNullOwned</a>&lt;T&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/slice/struct.slice_boxed.html\" title=\"struct safer_ffi::slice::slice_boxed\">slice_boxed</a>&lt;T&gt;"],["impl&lt;'lt, T:&nbsp;'lt&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/slice/struct.slice_ref.html\" title=\"struct safer_ffi::slice::slice_ref\">slice_ref</a>&lt;'lt, T&gt;"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/slice/struct.slice_mut.html\" title=\"struct safer_ffi::slice::slice_mut\">slice_mut</a>&lt;'_, T&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/string/struct.str_boxed.html\" title=\"struct safer_ffi::string::str_boxed\">str_boxed</a>"],["impl&lt;'lt&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/string/struct.str_ref.html\" title=\"struct safer_ffi::string::str_ref\">str_ref</a>&lt;'lt&gt;"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/struct.String.html\" title=\"struct safer_ffi::String\">String</a>"],["impl&lt;T&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"safer_ffi/struct.Vec.html\" title=\"struct safer_ffi::Vec\">Vec</a>&lt;T&gt;"]],
"scopeguard":[["impl&lt;T, F, S&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"scopeguard/struct.ScopeGuard.html\" title=\"struct scopeguard::ScopeGuard\">ScopeGuard</a>&lt;T, F, S&gt;<span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;F: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/function/trait.FnOnce.html\" title=\"trait core::ops::function::FnOnce\">FnOnce</a>(T),<br>&nbsp;&nbsp;&nbsp;&nbsp;S: <a class=\"trait\" href=\"scopeguard/trait.Strategy.html\" title=\"trait scopeguard::Strategy\">Strategy</a>,</span>"]],
"syn":[["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Underscore.html\" title=\"struct syn::token::Underscore\">Underscore</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Add.html\" title=\"struct syn::token::Add\">Add</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.And.html\" title=\"struct syn::token::And\">And</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.At.html\" title=\"struct syn::token::At\">At</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Bang.html\" title=\"struct syn::token::Bang\">Bang</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Caret.html\" title=\"struct syn::token::Caret\">Caret</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Colon.html\" title=\"struct syn::token::Colon\">Colon</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Comma.html\" title=\"struct syn::token::Comma\">Comma</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Div.html\" title=\"struct syn::token::Div\">Div</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Dollar.html\" title=\"struct syn::token::Dollar\">Dollar</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Dot.html\" title=\"struct syn::token::Dot\">Dot</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Eq.html\" title=\"struct syn::token::Eq\">Eq</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Gt.html\" title=\"struct syn::token::Gt\">Gt</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Lt.html\" title=\"struct syn::token::Lt\">Lt</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Or.html\" title=\"struct syn::token::Or\">Or</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Pound.html\" title=\"struct syn::token::Pound\">Pound</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Question.html\" title=\"struct syn::token::Question\">Question</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Rem.html\" title=\"struct syn::token::Rem\">Rem</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Semi.html\" title=\"struct syn::token::Semi\">Semi</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Star.html\" title=\"struct syn::token::Star\">Star</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Sub.html\" title=\"struct syn::token::Sub\">Sub</a>"],["impl <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/token/struct.Tilde.html\" title=\"struct syn::token::Tilde\">Tilde</a>"],["impl&lt;'c, 'a&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"syn/parse/struct.StepCursor.html\" title=\"struct syn::parse::StepCursor\">StepCursor</a>&lt;'c, 'a&gt;"]],
"uninit":[["impl&lt;'out, T:&nbsp;'out&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/1.66.1/core/ops/deref/trait.Deref.html\" title=\"trait core::ops::deref::Deref\">Deref</a> for <a class=\"struct\" href=\"uninit/out_ref/struct.Out.html\" title=\"struct uninit::out_ref::Out\">Out</a>&lt;'out, <a class=\"primitive\" href=\"https://doc.rust-lang.org/1.66.1/core/primitive.slice.html\">[T]</a>&gt;"]]
};if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()