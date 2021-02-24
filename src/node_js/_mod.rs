//! Safer-ffi internals used by `#[ffi-export]` to make such
//! annotated functions be callable from Node.js
//!
//! The current implementation delegates a fair bunch to [`::napi`], given
//! the timing constraints around developing this feature.
//! It is expected that in the mid-to-long term, we'll be cutting this
//! middle-man. #FIXME

pub use ::napi::*;
pub use ::napi_derive::*;

pub
mod ffi_helpers;

mod impls;

pub
mod registering;

/// Conversion from a Node.js parameter to a Rust value.
pub
trait FromNapi : Sized {
    type NapiValue : NapiValue;

    fn from_napi_value (
        env: &'_ Env,
        napi_value: Self::NapiValue,
    ) -> Result<Self>
    ;
}

/// Conversion from a returned Rust value to a Node.js value.
pub
trait ToNapi : Sized {
    type NapiValue : NapiValue;

    fn to_napi_value (
        self: Self,
        env: &'_ Env,
    ) -> Result<Self::NapiValue>
    ;
}

/// Convenience trait to implement both traits in one block.
pub
trait ReprNapi : ToNapi + FromNapi {
    type NapiValue : NapiValue;

    fn to_napi_value (
        self: Self,
        env: &'_ Env,
    ) -> Result< <Self as ToNapi>::NapiValue >
    ;
    fn from_napi_value (
        env: &'_ Env,
        napi_value: <Self as FromNapi>::NapiValue,
    ) -> Result<Self>
    ;
}

impl<T : ReprNapi> ToNapi for T {
    type NapiValue = <Self as ReprNapi>::NapiValue;

    fn to_napi_value (
        self: Self,
        env: &'_ Env,
    ) -> Result<Self::NapiValue>
    {
        <Self as ReprNapi>::to_napi_value(self, env)
    }
}

impl<T : ReprNapi> FromNapi for T {
    type NapiValue = <Self as ReprNapi>::NapiValue;

    fn from_napi_value (
        env: &'_ Env,
        napi_value: <Self as FromNapi>::NapiValue,
    ) -> Result<Self>
    {
        <Self as ReprNapi>::from_napi_value(env, napi_value)
    }
}

pub
fn extract_arg<T> (
    ctx: &'_ CallContext<'_>,
    idx: usize,
) -> Result<T>
where
    T : FromNapi,
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
