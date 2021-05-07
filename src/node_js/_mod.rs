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
    JsString,
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

#[allow(missing_debug_implementations)]
pub
struct JsPromise<ResolvesTo = JsUndefined> /* = */ (
    JsObject,
    ::core::marker::PhantomData<ResolvesTo>,
);

impl<ResolvesTo> NapiValue for JsPromise<ResolvesTo> {
    unsafe
    fn from_raw (env: sys::napi_env, value: sys::napi_value)
      -> Result<Self>
    {
        JsObject::from_raw(env, value)
            .map(|obj| Self(obj, Default::default()))
    }

    unsafe
    fn from_raw_unchecked (env: sys::napi_env, value: sys::napi_value)
      -> Self
    {
        Self(JsObject::from_raw_unchecked(env, value), Default::default())
    }

    unsafe
    fn raw (self: &'_ Self)
      -> sys::napi_value
    {
        self.0.raw()
    }
}

impl<ResolvesTo> JsPromise<ResolvesTo> {
    pub
    fn from_task_spawned_on_worker_pool<R, F> (
        env: &'_ Env,
        task: F,
    ) -> Result<JsPromise<ResolvesTo>>
    where
        F : 'static + Send + FnOnce() -> R,
        R : 'static + Send + crate::layout::ReprC,
        <R as crate::layout::ReprC>::CLayout : ReprNapi<NapiValue = ResolvesTo>,
    {
        struct UnsafeAssertSend<T>(T);

        unsafe
        impl<T> Send for UnsafeAssertSend<T>
        {}

        struct NapiTask<F> /* = */ (
            Option<F>,
        );

        impl<R, F> ::napi::Task for NapiTask<F>
        where
            F : 'static + Send + FnOnce() -> R,
            R : 'static + Send + crate::layout::ReprC,
            <R as crate::layout::ReprC>::CLayout : ReprNapi,
        {
            type Output = UnsafeAssertSend<
                <R as crate::layout::ReprC>::CLayout
            >;

            type JsValue =
                <
                    <R as crate::layout::ReprC>::CLayout
                    as
                    ReprNapi
                >::NapiValue
            ;

            fn compute (self: &'_ mut NapiTask<F>)
              -> Result<Self::Output>
            {
                self.0
                    .take()
                    .ok_or_else(|| Error::from_reason("\
                        Attempted to perform the background (worker pool) \
                        `::napi::Task::compute`-ation more than once!\
                    ".into()))
                    .map(|f| f())
                    .map(|repr_c| unsafe {
                        crate::layout::into_raw(repr_c)
                    })
                    .map(UnsafeAssertSend)
            }

            fn resolve (
                self: NapiTask<F>,
                env: Env,
                UnsafeAssertSend(output): Self::Output,
            ) -> Result<Self::JsValue>
            {
                output.to_napi_value(&env)
            }
        }

        let async_work_promise = env.spawn(NapiTask(Some(task)))?;
        Ok(JsPromise(async_work_promise.promise_object(), Default::default()))
    }

    pub
    fn resolve_into_unknown (self: JsPromise<ResolvesTo>)
      -> JsPromise<JsUnknown>
    {
        JsPromise(self.0, Default::default())
    }

    pub
    fn into_unknown (self: JsPromise<ResolvesTo>)
      -> JsUnknown
    {
        self.0
            .into_unknown()
    }
}
