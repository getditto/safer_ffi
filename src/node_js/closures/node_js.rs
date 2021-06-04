#![cfg_attr(rustfmt, rustfmt::skip)]
use super::*;

use ::napi::threadsafe_function::*;
use ::std::{
    os::raw::c_void,
    sync::Arc,
};

/// Define `Closure<fn(A) -> B>` to be sugar for:
/// `Closure_<(<A as ReprC>::CLayout,), <B as ReprC>::CLayout>`
pub type Closure<fn_sig> = <fn_sig as TypeAliasHelper>::T;
pub trait TypeAliasHelper { type T; }

use safety_boundary::ThreadTiedJsFunction;

mod safety_boundary {
    use super::*;

    pub
    struct ThreadTiedJsFunction {
        // A simple `func` field would not be `Send` nor `Sync` since it can't
        // be `call`ed from another thread.
        // We thus forge our own `SendWrapper` tailored for our use case.
        main_nodejs_thread: ::std::thread::ThreadId,
        env: Env,
        // We also need to enable some ref-counting mechanisms in case the
        // closure is called after the initial `JsFunction` is valid (_e.g._,
        // when the N-API call that fed the JsFunction to us returns).
        raw_ref_handle: ::napi::sys::napi_ref,
    }

    // Objective
    unsafe
        impl Send for ThreadTiedJsFunction {}
    unsafe
        impl Sync for ThreadTiedJsFunction {}

    impl ThreadTiedJsFunction {
        pub
        fn new (func: &'_ JsFunction, env: Env)
          -> ThreadTiedJsFunction
        {
            // call N-API's `ref`-counting functions:
            let mut raw_ref_handle = NULL!();
            unsafe {
                assert_eq!(0, ::napi::sys::napi_create_reference(
                    env.raw(),
                    func.raw(),
                    1,
                    &mut raw_ref_handle,
                ));
            }

            impl Drop for ThreadTiedJsFunction {
                fn drop (self: &'_ mut ThreadTiedJsFunction)
                {
                    // Note: since Self is `Send`,
                    // this may be called in a non-Node.js thread.
                    // It appears the ref-counting functions are thread-safe.
                    let Self { ref env, raw_ref_handle, .. } = *self;
                    unsafe {
                        /* Decrementing the ref-count before destroying it does
                         * not seem to be necessary. */
                        // ::napi::sys::napi_reference_unref(
                        //     env.raw(), raw_ref_handle, &mut 0,
                        // );
                        let _ignored_status =
                            ::napi::sys::napi_delete_reference(
                                env.raw(),
                                raw_ref_handle,
                            )
                        ;
                    }
                }
            }

            Self {
                env,
                raw_ref_handle,
                // `JsFunction`s can only be forged from within a Node.js thread.
                main_nodejs_thread: ::std::thread::current().id(),
            }
        }

        pub
        fn get_thread_tied_func (self: &'_ Self)
          -> Option<JsFunction>
        {
            if ::std::thread::current().id() == self.main_nodejs_thread {
                Some(unsafe {
                    let mut raw_func = NULL!();
                    assert_eq!(0, sys::napi_get_reference_value(
                        self.env.raw(),
                        self.raw_ref_handle,
                        &mut raw_func,
                    ));
                    JsFunction::from_raw_unchecked(self.env.raw(), raw_func)
                })
            } else {
                None
            }
        }
    }
}

/// A type-safe and _fully_ thread-safe wrapper around a `JsFunction`.
///
/// In N-API, a `JsFunction` can only be (synchronously) called from the thread
/// whence it originated.
///
/// The only workaround for that is using a wrapper where the `JsFunction`'s
/// return value is lost (run in "the background"): `ThreadsafeFunction`.
///
/// To solve that, the closure can be tweaked to send its return value through
/// a channel, and then have the caller wait on the receiving end of the
/// channel.
///
/// But this makes the pattern unusable from within the main `Node.js` thread,
/// since that's the actual "background" that runs the computation, and it only
/// does so *after* returning from an FFI call. So if the FFI call were to wait
/// on the receiving end of the channel, it would be blocking the `Node.js`
/// thread, which would be unable to return and actually produce the return
/// value that needs to be sent across that very channel: a deadlock.
///
/// In order to avoid that, we also bundle a `ThreadTiedJsFunction` to fallback
/// to a classic call when we detect we are within the main Node.js thread.
pub
struct Closure_<Args : 'static, Ret : 'static> {
    /// A `JsFunction` that can only be called from the thread whence it
    /// originated.
    local_func: ThreadTiedJsFunction,

    /// An N-API provided wrapper around a `JsFunction` to "make it thread-safe"
    /// (this allows using a special API that can be called from an *external*
    /// thread).
    /// In order for the return value produced by the wrapped `JsFunction` not
    /// to be lost, this wrapper bundles the `Sender` part of a `channel`,
    /// and somehow the return value of the closure is sent through that channel
    /// (the current implementation expects the Js side to cooperate and call
    /// the first parameter it receives on the produced value, but this is
    /// expected to be done by us in the FFI so as to hide that implementation
    /// detail).
    ts_fun: ThreadsafeFunction<
        (
            ::std::sync::mpsc::SyncSender< Result<Ret> >,
            Args,
        ),
        ErrorStrategy::Fatal,
    >,

    /// We cache an `Env` mainly to be able to keep interacting with the
    /// ref-counting APIs.
    env: Env,
}

impl<Args : 'static, Ret : 'static> ::core::fmt::Debug
    for Closure_<Args, Ret>
{
    fn fmt (
        self: &'_ Self,
        fmt: &'_ mut ::core::fmt::Formatter<'_>,
    ) -> ::core::fmt::Result
    {
        ::core::fmt::Display::fmt(
            ::core::any::type_name::<Self>(),
            fmt,
        )
    }
}

unsafe
    impl<Args : 'static, Ret : 'static> Send for Closure_<Args, Ret>
   /*
    * FIXME: these bounds seem plausible in order to make sure our API is
    * sound, but since raw pointers aren't `Send`, in practice it will be
    * too cumbersome. Since the current design with
    * ReprC-to-CType-that-is-ReprNapi is not final anyways (ideally, we'd
    * be dealing with `ReprC + ReprNapi` types), let's not worry about this
    * yet…
    **/
    // where
        // Args : Send,
        // Ret : Send,
    {}

unsafe
    impl<Args : 'static, Ret : 'static> Sync for Closure_<Args, Ret>
   /*
    * FIXME: same as above, but for the sub-bounds still being `Send`.
    * This is intended / not a typo: Args and Ret are never shared, so this
    * is, AFAIK, the correct bound.
    **/
    // where
        // Args : Send,
        // Ret : Send,
    {}

// Since variadic generics to support arbitrary function arities are not
// available yet, we use macros to generate implementations for many hard-coded
// arities. In this instance, functions of up to 6 parameters.
impls! { (_6, _5, _4, _3, _2, _1) }
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
        TypeAliasHelper
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
        Ret : 'static + ReprNapi + Send,
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
            &env: &'_ Env,
            ref func: JsFunction,
        ) -> Result<Self>
        {
            let thread_tied_func = ThreadTiedJsFunction::new(func, env);

            // let (ret_sender, ret_receiver) = mpsc::sync_channel::<Result<Ret>>(0);
            // // Note: when using a `ThreadsafeFunction` wrapper, we lose
            // // the ability to get the value returned by the wrapped `JsFunction`.
            // // We thus circumvent this limitation by wrapping the received
            // // `JsFunction` into one of our making that uses a special returner
            // // slot. The `receiver` is then then handle through which we can get
            // // the so-"returned" value.
            // let (func, ret_receiver) = {
            //     use ::std::sync::mpsc;
            //     let (ret_sender, ret_receiver) = mpsc::sync_channel::<Result<Ret>>(1);

            //     let func = ::scopeguard::guard(
            //         // call ref
            //         unsafe {
            //             let mut raw_ref = crate::NULL!();
            //             dbg!(::napi::sys::napi_create_reference(
            //                 env.raw(),
            //                 func.raw(),
            //                 1,
            //                 &mut raw_ref,
            //             ));
            //             (func, raw_ref, env.raw())
            //         },
            //         // drop: call unref
            //         |(_, raw_ref, raw_env)| unsafe {
            //             // dbg!(::napi::sys::napi_reference_unref(
            //             //     raw_env, raw_ref, &mut 0,
            //             // ));
            //             // dbg!(::napi::sys::napi_delete_reference(
            //             //     raw_env, raw_ref,
            //             // ));
            //         },
            //     );

            //     let func = UnsafeAssertSendSync(func);
            //     struct UnsafeAssertSendSync<T>(T);
            //     unsafe impl<T> Send for UnsafeAssertSendSync<T> {}
            //     unsafe impl<T> Sync for UnsafeAssertSendSync<T> {}
            //     let wrapped_func = env.create_function_from_closure("<name>", move |ctx: CallContext<'_>| Ok({
            //         drop(&func);
            //         let _ =
            //             ret_sender.send(
            //                 (unsafe {
            //                     JsFunction::from_raw_unchecked(
            //                         ctx.env.raw(),
            //                         func.0.0.raw(),
            //                     )
            //                     // func.0.0
            //                 }).call(
            //                     ctx.this::<JsObject>().ok().as_ref(),
            //                     &ctx.get_all(),
            //                 )
            //                 .and_then(|js_unknown| {
            //                     use ::core::convert::TryInto;
            //                     js_unknown
            //                         .try_into()
            //                         .map_or_else(
            //                             |_| todo!(),
            //                             |napi_val| ReprNapi::from_napi_value(ctx.env, napi_val),
            //                         )
            //                 })
            //             )
            //         ;
            //         ctx .env
            //             .get_undefined()
            //             .unwrap()
            //     }))?;
            //     (wrapped_func, ret_receiver)
            // };

            let mut ts_fun = ThreadsafeFunction::create(
                env.raw(),
                func,
                // max_queue_size /* use `0` for a sane default */
                1,
                // callback
                Self::handle_params,
            )?;
            // By default, let's avoid keeping the main loop spinning while
            // this entity is alive.
            // A `.make_nodejs_wait_for_this_to_be_dropped(true)` helper is
            // available to opt-out of that.
            ts_fun.unref(&env)?;

            Ok(Self {
                ts_fun,
                local_func: thread_tied_func,
                env,
            })
        }
    }

    impl<
    $(  $_0 : 'static + ReprNapi, $(
        $_k : 'static + ReprNapi, )*)?
        CRet : 'static + ReprNapi,
    >
        Closure_<($($_0, $($_k ,)*)?), CRet>
    {
        /// Make Node.js's main event loop wait for the `Closure` to be
        /// dropped before ending.
        ///
        /// If used incorrectly, this can make Node.js keep spinning infinitely
        /// at the end of the program execution.
        ///
        /// On the other hand, if you have a `Closure` that _needs_ to be
        /// called (or dropped), and which you know _will be **dropped**_ before
        /// Node.js reaches the end of its main loop, then you can call this
        /// function with `true`.
        pub
        fn make_nodejs_wait_for_this_to_be_dropped (
            self: &'_ mut Self,
            nodejs_should_wait: bool,
        ) -> Result<()>
        {
            if nodejs_should_wait {
                self.ts_fun.refer(&self.env)
            } else {
                self.ts_fun.unref(&self.env)
            }
        }

        pub
        fn as_raw_parts (self: &'_ Self)
          -> (
                *mut c_void,

                unsafe extern "C"
                fn(this: *mut c_void $(, $_0: $_0, $( $_k: $_k, )*)?)
                  -> CRet
                ,
            )
        {
            (
                self as *const _ as *mut _,
                Self::call_trampoline,
            )
        }

        pub
        fn into_raw_parts (self: Arc<Self>)
          -> ArcClosureRawParts<
                unsafe extern "C"
                fn(
                    this: *mut c_void $(,
                    $_0: $_0, $(
                    $_k: $_k, )*)?
                ) -> CRet,
            >
        {
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
            $_0: $_0, $(
            $_k: $_k, )*)?
        ) -> CRet
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
                ref ts_fun,
                env: ref orig_env,
                local_func: ref fun,
                    } = {
                this.cast::<Self>().as_ref().expect("Got NULL")
            };
            match fun.get_thread_tied_func() {
                | None => {
                    // If we are not called from the thread whence the `Closure`
                    // was created, we are not allowed to call `func` directly:
                    // we thus politely ask the main js thread to call us, and
                    // then patiently wait to receive the return value back.
                    //
                    // ⚠️ Note: this wait could still back-propagate to the main
                    // js thread (if the caller happens to try to sync with it),
                    // causing a deadlock. ⚠️
                    //
                    // The key idea, implementation-wise, is to rely (FIXME) on
                    // some co-operation from the JsFunction: such a function
                    // shall instead of returning a value, call its first
                    // received parameter on it.
                    //
                    // That is, the JsFunction is expected to have been wrapped
                    // in a `wrap_cb_for_ffi` call, where:
                    //
                    // ```js
                    // function wrap_cb_for_ffi(f) {
                    //     return (send_ret, ...args) => {
                    //         try {
                    //             return send_ret(f(...args));
                    //         } catch (e) {
                    //             console.error(e);
                    //         }
                    //     };
                    // }
                    // ```
                    let (ret_sender, ret_receiver) =
                        ::std::sync::mpsc::sync_channel(0)
                    ;
                    let status = ts_fun.call(
                        // Note: this params are handled by `fn convert_params`
                        (
                            ret_sender,
                            ( $( $_0, $($_k, )*)? ),
                        ),
                        ThreadsafeFunctionCallMode::Blocking,
                    );
                    debug_assert_eq!(status, Status::Ok);

                    ret_receiver
                        .recv_timeout(::std::time::Duration::from_secs(5))
                        .expect("Channel closed or timeout (deadlock?)")
                },

                | Some(ref func) => {
                    // Otherwise, it means we are within the main nodejs thread,
                    // so we can't "schedule the call and wait for it to be run",
                    // lest we deadlock. We thus directly perform the call.
                    func.call(
                        // this
                        None,
                        // params
                        &[
                            orig_env
                                .create_function_from_closure(
                                    "send_ret",
                                    move |ctx: CallContext<'_>| Ok({
                                        ctx.get::<JsUnknown>(0)?
                                    }),
                                )
                                .expect("Creation of `send_ret` failed")
                                .into_unknown()
                            ,
                        $(  ReprNapi::to_napi_value($_0, orig_env)
                                .expect("Conversion from C param to closure param failed")
                                .into_unknown()
                            , $(
                            ReprNapi::to_napi_value($_k, orig_env)
                                .expect("Conversion from C param to closure param failed")
                                .into_unknown()
                            , )*)?
                        ],
                    )
                    .and_then(|js_unknown| {
                        use ::core::convert::TryInto;
                        let ty = js_unknown.get_type();
                        js_unknown
                            .try_into()
                            .map_err(|_| Error::from_reason(format!(
                                "\
                                    Expected the js callback to return a {}, \
                                    got `{:?}` instead.\
                                ",
                                ::core::any::type_name::<
                                    <CRet as ReprNapi>::NapiValue
                                >(),
                                ty.as_ref().map_or(
                                    &"Failed to query the type of the JsUnknown" as &dyn ::core::fmt::Debug,
                                    |ty| ty,
                                ),
                            )))
                    })
                    .and_then(|r| ReprNapi::from_napi_value(orig_env, r))
                },
            }
            .expect("Cannot throw a js exception within an FFI callback")
        }

        fn handle_params(
            ThreadSafeCallContext {
                env,
                // FFI args
                value: (
                    sender,
                    ( $( $_0, $($_k ,)* )? ),
                ),
            }: ThreadSafeCallContext<(
                ::std::sync::mpsc::SyncSender< Result<CRet> >,
                ( $( $_0, $( $_k, )* )? ),
            )>,
        ) -> Result<Vec<JsUnknown>> // Node.js args
        where
            CRet : Send,
        {
            // `let sender = js_closure!(move |value| sender.send(value));`
            let sender = env.create_function_from_closure(
                "ret sender",
                move |ctx: CallContext<'_>| Ok({
                    let arg: Result<CRet> = if ctx.length == 0 {
                        unreachable!(
                            "JsFunction was incorrectly wrapped"
                        );
                    } else {
                        super::extract_arg::<CRet>(&ctx, 0)
                    };
                    let _ = sender.send(arg);
                    ctx.env.get_undefined()?
                })
            )?;

        $(  let $_0 = ReprNapi::to_napi_value($_0, &env)?; $(
            let $_k = ReprNapi::to_napi_value($_k, &env)?; )*)?
            let mut args = Vec::<JsUnknown>::with_capacity(
                1
            $(  + { let _ = $_0; 1 } $(
                + { let _ = $_k; 1 } )*)?
            );
            args.push(sender.into_unknown());
        $(  args.push($_0.into_unknown()); $(
            args.push($_k.into_unknown()); )*)?
            Ok(args)
        }
    }
)} use impls;

include!("common.rs");
