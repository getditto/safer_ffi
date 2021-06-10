#![cfg_attr(rustfmt, rustfmt::skip)]

use super::*;

use ::{
    std::os::raw::c_void,
};
use crate::{
    layout::ReprC,
};

/// Define `Closure<fn(A) -> B>` to be sugar for:
/// `Closure_<(<A as ReprC>::CLayout,), <B as ReprC>::CLayout>`
#[derive(Debug)]
pub
struct Closure<fn_sig> {
    js_fun: JsFunction,
    _phantom: ::core::marker::PhantomData<fn_sig>,
}

impl<fn_sig> ReprNapi
    for Closure<fn_sig>
{
    type NapiValue = JsFunction;

    fn from_napi_value (
        _: &'_ Env,
        js_fun: JsFunction,
    ) -> Result<Self>
    {
        Ok(Self {
            js_fun,
            _phantom: Default::default(),
        })
    }

    fn to_napi_value (
        self: Closure<fn_sig>,
        _: &'_ Env,
    ) -> Result<JsFunction>
    {
        Ok(self.js_fun)
    }
}

// Since variadic generics to support arbitrary function arities are not
// available yet, we use macros to generate implementations for many hard-coded
// arities. In this instance, functions of up to 6 parameters.
match_! {(
    _6, _5, _4, _3, _2, _1,
) {(
    $(  $_0:ident $(,
        $_k:ident )* $(,)? )?
) => (
    $(
        __recurse__! { $($_k),* }
    )?

    impl<
    $(  $_0 : 'static + /* Send + */ ReprC, $(
        $_k : 'static + /* Send + */ ReprC, )*)?
        Ret : 'static + /* Send + */ ReprC,
    >
        Closure<fn( $($_0 $(, $_k)*)? ) -> Ret>
    where
    $(  <$_0 as ReprC>::CLayout : ReprNapi, $(
        <$_k as ReprC>::CLayout : ReprNapi, )*)?
        <Ret as ReprC>::CLayout : ReprNapi,
    {
        pub
        fn make_nodejs_wait_for_this_to_be_dropped (
            self: &'_ mut Self,
            _nodejs_should_wait: bool,
        ) -> Result<()>
        {
            Ok(())
        }

        pub
        fn as_raw_parts (
            self: &'_ Self,
        ) -> (
            *mut c_void,

            unsafe extern "C"
            fn(this: *mut c_void $(,
                $_0: <$_0 as ReprC>::CLayout $(,
                $_k: <$_k as ReprC>::CLayout )*)?
            ) -> <Ret as ReprC>::CLayout,
        )
        {
            (
                self as *const _ as *mut _,
                Self::call_trampoline,
            )
        }

        pub
        fn into_raw_parts (self: ::std::sync::Arc<Self>)
          -> ArcClosureRawParts<
            unsafe extern "C"
            fn(
                this: *mut c_void $(,
                $_0: <$_0 as ReprC>::CLayout, $(
                $_k: <$_k as ReprC>::CLayout, )*)?
            ) -> <Ret as ReprC>::CLayout,
        >
        {
            use ::std::sync::Arc;
            ArcClosureRawParts {
                data: Arc::into_raw(self) as _,
                call: Self::call_trampoline,
                release: {
                    unsafe extern "C"
                    fn release_arc<Self_> (data: *mut c_void)
                    {
                        drop(Arc::<Self_>::from_raw(data.cast()))
                    }

                    release_arc::<Self>
                },
                retain: {
                    unsafe extern "C"
                    fn retain_arc<Self_> (data: *mut c_void)
                    {
                        let arc: &Arc<Self_> = &(
                            ::core::mem::ManuallyDrop::new(
                                Arc::<Self_>::from_raw(data.cast())
                            )
                        );
                        ::core::mem::forget(arc.clone());
                    }

                    retain_arc::<Self>
                },
            }
        }

        /* Helpers */
        unsafe extern "C"
        fn call_trampoline (
            this: *mut c_void $(,
            $_0: <$_0 as ReprC>::CLayout $(,
            $_k: <$_k as ReprC>::CLayout )*)?
        ) -> <Ret as ReprC>::CLayout
        {
            ::scopeguard::defer_on_unwind! {
                eprintln!("\
                    Attempted to panic through an `extern \"C\"` boundary, \
                    which is undefined behavior. \
                    Aborting for soundness.\
                ");
                ::std::process::abort();
            }

            let     &Self {
                ref js_fun,
                ..
            } = {
                this.cast::<Self>().as_ref().expect("Got NULL")
            };
            let ref env = Env::__new();
            js_fun.call(
                // this
                None,
                // params
                &[
                    // send_ret
                    {
                        use crate::node_js::__::wasm_bindgen;

                        #[wasm_bindgen(inline_js = r#"
                            export function mk_send_ret() {
                                return function send_ret(arg) {
                                    return arg;
                                };
                            }
                        "#)]
                        extern {
                            fn mk_send_ret () -> JsUnknown;
                        }
                        mk_send_ret()
                    },

                $(  ReprNapi::to_napi_value($_0, env)
                        .expect("Conversion from C param to closure param failed")
                        .into_unknown()
                    , $(
                    ReprNapi::to_napi_value($_k, env)
                        .expect("Conversion from C param to closure param failed")
                        .into_unknown()
                    , )*)?
                ],
            )
                // FIXME: use `.and_then(… ….dyn_into())` as done in `node_js.rs`.
                .map(|js_unknown| js_unknown.unchecked_into())
                .and_then(|r| ReprNapi::from_napi_value(env, r))
                .expect("Cannot throw a js exception within an FFI callback")
        }
    }
)}}

include!("common.rs");
