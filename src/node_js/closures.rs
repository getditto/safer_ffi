use super::*;

use ::napi::threadsafe_function::*;

pub trait Helper { type T; }
pub type Closure<fn_sig> = <fn_sig as Helper>::T;

#[allow(missing_debug_implementations)]
pub
struct Closure_<Args : 'static, Ret> {
    ts_fun: ThreadsafeFunction<Args>,
    _ret: ::core::marker::PhantomData<Ret>,
}

impls! { (_5, _4, _3, _2, _1) }
macro_rules! impls {(
    ($( $_0:ident $(, $_k:ident)* $(,)? )?)
) => (
    $(
        impls! {
            ($($_k),*)
        }
    )?

    impl<
    $(  $_0 : 'static + crate::layout::ReprC, $(
        $_k : 'static + crate::layout::ReprC, )*)?
        Ret : 'static + crate::layout::ReprC,
    >
        Helper
    for
        fn($($_0, $($_k ,)*)?) -> Ret
    {
        type T = Closure_<
            ($(
                <$_0 as crate::layout::ReprC>::CLayout, $(
                <$_k as crate::layout::ReprC>::CLayout, )*)?
            ),
            <Ret as crate::layout::ReprC>::CLayout,
        >;
    }

    impl<
    $(  $_0 : 'static + ReprNapi, $(
        $_k : 'static + ReprNapi, )*)?
        Ret : 'static + ReprNapi,
    >
        ReprNapi
    for
        Closure_<($($_0, $($_k ,)*)?), Ret>
    {
        type NapiValue = JsFunction;

        /// Conversion from a returned Rust value to a Node.js value.
        fn to_napi_value (
            self: Self,
            _: &'_ Env,
        ) -> Result< JsFunction >
        {
            unimplemented!(
                "<{} as ReprNapi>::to_napi_value",
                ::core::any::type_name::<Self>(),
            );
        }

        /// Conversion from a Node.js parameter to a Rust value.
        fn from_napi_value (
            env: &'_ Env,
            ref func: JsFunction,
        ) -> Result< Self >
        {Ok({
            let mut ts_fun = ThreadsafeFunction::create(
                env.raw(),
                func,
                // max_queue_size
                0,
                // callback
                |ctx: ThreadSafeCallContext<($($_0, $($_k ,)*)?)>| Ok({
                    let ThreadSafeCallContext {
                        ref env,
                        value: ($($_0, $($_k ,)*)?),
                    } = ctx;
                $(  let $_0 = ReprNapi::to_napi_value($_0, env)?; $(
                    let $_k = ReprNapi::to_napi_value($_k, env)?; )*)?
                    let mut _args = Vec::<JsUnknown>::with_capacity(
                        0
                    $(  + { let _ = $_0; 1 } $(
                        + { let _ = $_k; 1 } )*)?
                    );
                $(  _args.push($_0.into_unknown()); $(
                    _args.push($_k.into_unknown()); )*)?
                    _args
                })
            )?;
            // ts_fun.refer(env)?; // Keep the main loop alive while this entity is.

            /* No need to inc ref-count thanks to single-ownership in Rust */
            // unsafe {
            //     extern "C" {
            //         fn napi_acquire_threadsafe_function (
            //             func: ::napi::sys::napi_threadsafe_function,
            //         ) -> ::napi::sys::napi_status;
            //     }
            //     napi_acquire_threadsafe_function(ts_fun.raw());
            // }

            Self {
                ts_fun,
                _ret: ::core::marker::PhantomData,
            }
        })}
    }

    impl<
    $(  $_0 : 'static + ReprNapi, $(
        $_k : 'static + ReprNapi, )*)?
        Ret : 'static + ReprNapi,
    >
        Closure_<($($_0, $($_k ,)*)?), Ret>
    {
        pub
        unsafe extern "C"
        fn cb (
            this: *mut ::std::os::raw::c_void $(,
            $_0: $_0, $(
            $_k: $_k, )*)?
        ) -> Ret
        {
            ::scopeguard::defer_on_unwind! {
                eprintln!("\
                    Attempted to panic through an `extern \"C\"` boundary, \
                    which is undefined behavior. Aborting for soundness.\
                ");
                ::std::process::abort();
            }

            let this: &Self = this.cast::<Self>().as_ref().expect("Got NULL");

            let todo_handle_status = dbg!(this.ts_fun.call( // FIXME
                Ok(($($_0, $($_k ,)*)?)),
                ThreadsafeFunctionCallMode::Blocking, // Should it be non-blocking?
            ));

            if  ::core::any::TypeId::of::<Ret>() ==
                ::core::any::TypeId::of::< crate::tuple::CVoid >()
            {
                return unsafe {
                    ::core::mem::transmute_copy(&())
                };
            }
            unimplemented!("Get the instance of type `Ret` back");
        }
    }
)} use impls;
