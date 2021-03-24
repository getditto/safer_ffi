#![cfg_attr(rustfmt, rustfmt::skip)]
//! Safer-ffi internals used by `#[ffi-export]` to make such
//! annotated functions be callable from Node.js
//!
//! The current implementation delegates a fair bunch to [`::napi`], given
//! the timing constraints around developing this feature.
//! It is expected that in the mid-to-long term, we'll be cutting this
//! middle-man. #FIXME

pub use ::napi::*;
pub use ::napi_derive::*;

pub use closures::*;
mod closures;

pub
mod ffi_helpers;

mod impls;

pub
mod registering;

/// Interconversion between `CType`s and js values
pub
trait ReprNapi : Sized /* : crate::layout::CType */ {
    type NapiValue : NapiValue + IntoUnknown;

    /// Conversion from a returned Rust value to a Node.js value.
    fn to_napi_value (
        self: Self,
        env: &'_ Env,
    ) -> Result< Self::NapiValue >
    ;

    /// Conversion from a Node.js parameter to a Rust value.
    fn from_napi_value (
        env: &'_ Env,
        napi_value: Self::NapiValue,
    ) -> Result<Self>
    ;
}

pub
fn extract_arg<T> (
    ctx: &'_ CallContext<'_>,
    idx: usize,
) -> Result<T>
where
    T : ReprNapi,
{
    T::from_napi_value(ctx.env, ctx.get::<T::NapiValue>(idx)?)
}

#[macro_export]
macro_rules! node_js_register_exported_functions {() => (
    const _: () = {
        use ::safer_ffi::node_js as napi;

        #[no_mangle] pub
        unsafe extern "C"
        fn napi_register_module_v1 (
            env: napi::sys::napi_env,
            exports: napi::sys::napi_value,
        ) -> napi::sys::napi_value
        {
            napi::registering::napi_register_module_v1(env, exports)
        }
    };
)}
pub use node_js_register_exported_functions as register_exported_functions;

pub
trait IntoUnknown : ::core::convert::TryFrom<JsUnknown> {
    fn into_unknown (self: Self)
      -> JsUnknown
    ;
}

match_! {(
    JsFunction,
    JsNumber,
    JsObject,
    JsBoolean,
    JsUnknown,
    JsUndefined,
    JsNull,
) {
    ( $($JsTy:ident),* $(,)? ) => (
        $(
            impl IntoUnknown for $JsTy {
                fn into_unknown (self: Self)
                  -> JsUnknown
                {
                    #![deny(unconditional_recursion)]
                    Self::into_unknown(self)
                }
            }
        )*
    );
}}
