//! Safer-ffi internals used by `#[ffi-export]` to make such
//! annotated functions be callable from Node.js
//!
//! The current implementation delegates a fair bunch to [`::napi`], given
//! the timing constraints around developing this feature.
//! It is expected that in the mid-to-long term, we'll be cutting this
//! middle-man. #FIXME

use ::core::future::Future;

extern crate napi;

pub use ::napi::*;

// pub use ::napi_derive::js_function;

pub use closures::*;

#[cfg_attr(target_arch = "wasm32", path = "closures/wasm.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path = "closures/node_js.rs")]
mod closures;

pub
mod ffi_helpers;

mod impls;

cfg_not_wasm! {
    pub
    mod registering;
}

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

pub use adhoc::AdhocToReprNapi;
mod adhoc {
    use super::*;

    pub
    trait ToReprNapiFnOnce
    where
        Self : FnOnce(&'_ Env) -> Result< Self::NapiValue >,
    {
        type NapiValue : NapiValue + IntoUnknown;
    }

    impl<F, NapiValue>
        ToReprNapiFnOnce
    for
        F
    where
        Self : FnOnce(&'_ Env) -> Result< NapiValue >,
        NapiValue : super::NapiValue + IntoUnknown,
    {
        type NapiValue = NapiValue;
    }

    #[allow(missing_debug_implementations)]
    pub
    struct AdhocToReprNapi<F : ToReprNapiFnOnce>(
        pub F,
    )
    where
        F : FnOnce(&Env) -> Result< F::NapiValue >,
    ;

    impl<F : ToReprNapiFnOnce> ReprNapi for AdhocToReprNapi<F> {
        type NapiValue = F::NapiValue;

        /// Conversion from a returned Rust value to a Node.js value.
        fn to_napi_value (
            self: Self,
            env: &'_ Env,
        ) -> Result< F::NapiValue >
        {
            (self.0)(env)
        }

        /// Conversion from a Node.js parameter to a Rust value.
        fn from_napi_value (
            _: &'_ Env,
            _: Self::NapiValue,
        ) -> Result<Self>
        {
            unimplemented!("ToReprNapiFnOnce")
        }
    }
}

cfg_not_wasm! {
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
}

#[macro_export]
#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! js_register_exported_functions {() => (
    #[cfg(not(target_arch = "wasm32"))]
    const _: () = {
        use ::safer_ffi::js as napi;

        #[unsafe(no_mangle)] pub
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
pub use js_register_exported_functions as register_exported_functions;

pub
trait IntoUnknown : ::core::convert::TryFrom<JsUnknown> {
    fn into_unknown (self: Self)
      -> JsUnknown
    ;
}

match_! {
    (
        JsBuffer,
        JsFunction,
        JsNumber,
        JsObject,
        JsBoolean,
        JsUnknown,
        JsUndefined,
        JsNull,
        JsString,
    )
{(
    $(
        $JsTy:ident
    ),* $(,)?
) => (
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
)}}

#[allow(missing_debug_implementations)]
pub
struct JsPromise<ResolvesTo = JsUnknown> /* = */ (
    JsObject,
    ::core::marker::PhantomData<ResolvesTo>,
);

cfg_wasm! {
    mod hidden {
        pub trait Send {}
    }
    use hidden::Send;
    impl<T> Send for T {}
}

impl<ResolvesTo> JsPromise<ResolvesTo> {
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

    #[cfg(target_arch = "wasm32")]
    pub
    fn resolve (value: &'_ ::napi::__::wasm_bindgen::JsValue)
      -> JsPromise<ResolvesTo>
    {
        Self(
            ::napi::__::js_sys::Promise::resolve(value)
                .unchecked_into()
            ,
            Default::default(),
        )
    }

    pub
    fn spawn<Fut> (
        env: &'_ Env,
        fut: Fut,
    ) -> Result< JsPromise<ResolvesTo> >
    where
        ResolvesTo : 'static + NapiValue,
        Fut : 'static + Send + Future,
        <Fut as Future>::Output : Send + ReprNapi<NapiValue = ResolvesTo>,
    {
        #[cfg(target_arch = "wasm32")]
        let ret = {
            let _ = env;
            let promise = ::napi::__::wasm_bindgen_futures::future_to_promise(
                async move {
                    fut .await
                        .to_napi_value(&Env::__new())
                        .map(|it| it.unchecked_into())
                }
            );
            Ok(JsPromise(promise.unchecked_into(), Default::default()))
        };
        #[cfg(not(target_arch = "wasm32"))]
        let ret =
            env .execute_tokio_future(
                    async { Ok(fut.await) },
                    |env, fut_output| fut_output.to_napi_value(env),
                )
                .map(|promise| JsPromise(promise, Default::default()))
        ;
        ret
    }
}

cfg_not_wasm! {
    impl<ResolvesTo> NapiValue for JsPromise<ResolvesTo> {
        unsafe
        fn from_raw (env: sys::napi_env, value: sys::napi_value)
          -> Result<Self>
        {
            unsafe { JsObject::from_raw(env, value) }
                .map(|obj| Self(obj, Default::default()))
        }

        unsafe
        fn from_raw_unchecked (env: sys::napi_env, value: sys::napi_value)
          -> Self
        {
            Self(
                unsafe { JsObject::from_raw_unchecked(env, value) },
                Default::default(),
            )
        }

        unsafe
        fn raw (self: &'_ Self)
          -> sys::napi_value
        {
            unsafe { self.0.raw() }
        }
    }

    #[derive(Debug)]
    pub
    struct AsyncWorkerTask<Worker, ThenMainJs> {
        pub on_worker: Option<Worker>,
        pub then_on_main_js: ThenMainJs,
    }

    impl<Worker, ThenMainJs, R, JsValue> ::napi::Task
        for AsyncWorkerTask<Worker, ThenMainJs>
    where
        Worker : 'static + Send + FnOnce() -> R,
        R : 'static + Send,
        ThenMainJs : 'static + Send + FnOnce(&'_ Env, R) -> Result<JsValue>,
        JsValue : NapiValue,
    {
        type Output = R;

        type JsValue = JsValue;

        fn compute (self: &'_ mut AsyncWorkerTask<Worker, ThenMainJs>)
          -> Result<R>
        {
            self.on_worker
                .take()
                .ok_or_else(|| Error::from_reason("\
                    Attempted to perform the background (worker pool) \
                    `::napi::Task::compute`-ation more than once!\
                ".into()))
                .map(|f| f())
        }

        fn resolve (
            self: AsyncWorkerTask<Worker, ThenMainJs>,
            ref env: Env,
            output: R,
        ) -> Result<JsValue>
        {
            (self.then_on_main_js)(env, output)
        }
    }

    impl<Worker, ThenMainJs, R, JsValue> AsyncWorkerTask<Worker, ThenMainJs>
    where
        Worker : 'static + Send + FnOnce() -> R,
        R : 'static + Send,
        ThenMainJs : 'static + Send + FnOnce(&'_ Env, R) -> Result<JsValue>,
        JsValue : NapiValue,
    {
        pub
        fn spawn (
            self: AsyncWorkerTask<Worker, ThenMainJs>,
            env: &'_ Env,
        ) -> Result<JsPromise< JsValue >>
        {
            env .spawn(self)
                .map(|async_work_promise| JsPromise(
                    async_work_promise.promise_object(),
                    Default::default(),
                ))
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
            ResolvesTo : NapiValue + IntoUnknown,
        {
            AsyncWorkerTask {
                on_worker: Some(move || UnsafeAssertSend(task())),
                then_on_main_js: |env: &'_ _, UnsafeAssertSend(output)| unsafe {
                    crate::layout::into_raw::<R>(output)
                        .to_napi_value(env)
                },
            }.spawn(env)
        }
    }
}

#[allow(missing_debug_implementations)]
#[repr(transparent)]
pub
struct UnsafeAssertSend<T> /* = */ (
    T,
);

impl<T> UnsafeAssertSend<T> {
    #[inline]
    pub
    unsafe
    fn new (value: T)
      -> UnsafeAssertSend<T>
    {
        UnsafeAssertSend(value)
    }

    pub
    fn into_inner (self: UnsafeAssertSend<T>)
      -> T
    {
        let UnsafeAssertSend(value) = self;
        value
    }
}

unsafe
impl<T> ::core::marker::Send for UnsafeAssertSend<T>
{}

impl<T : ReprNapi> ReprNapi for UnsafeAssertSend<T> {
    type NapiValue = T::NapiValue;

    /// Conversion from a returned Rust value to a Node.js value.
    #[inline]
    fn to_napi_value (
        self: UnsafeAssertSend<T>,
        env: &'_ Env,
    ) -> Result< T::NapiValue >
    {
        self.into_inner().to_napi_value(env)
    }

    /// Conversion from a Node.js parameter to a Rust value.
    #[inline]
    fn from_napi_value (
        _: &'_ Env,
        _: T::NapiValue,
    ) -> Result<UnsafeAssertSend<T>>
    {
        unimplemented!("\
            Cannot produce an `UnsafeAssertSend` without `unsafe` code.\
        ");
    }
}
