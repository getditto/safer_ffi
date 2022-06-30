#![cfg_attr(rustfmt, rustfmt::skip)]

#[doc(hidden)] #[macro_export]
macro_rules! __ffi_export__ {
(
    $(#[$($meta:tt)*])*
    $pub:vis
    $enum_or_struct:ident
    $name:ident
    {
        $($tt:tt)*
    }
) => (
    $(#[$($meta)*])*
    $pub
    $enum_or_struct
    $name
    {
        $($tt)*
    }

    #[cfg(not(target_arch = "wasm32"))]
    $crate::__cfg_headers__! {
        $crate::inventory::submit! {
            #![crate = $crate]
            $crate::FfiExport {
                name: $crate::ඞ::stringify!($name),
                gen_def: $crate::headers::__define_self__::<$name>,
            }
        }
    }
);

(
    $( @[node_js(
        $node_js_arg_count:literal,
        $($async_worker:literal $(,)?)?
    )] )?

    $($(#[doc = $doc:expr])+)?
    // $(#[$meta:meta])*
    $pub:vis
    $(unsafe $(@$hack:ident@)?)?
    $(extern $("C")?)?
    fn $fname:ident $(<$($lt:lifetime $(: $sup_lt:lifetime)?),* $(,)?>)? (
        $(
            $($arg_name:ident)+ : $arg_ty:ty
        ),* $(,)?
    ) $(-> $Ret:ty)?
    $( where {
        $($bounds:tt)*
    } )?
        $body:block
) => (
    $($(#[doc = $doc])+)?
    // $(#[$meta])*
    #[allow(improper_ctypes_definitions)]
    $pub
    $(unsafe $(@$hack@)?)?
    extern "C"
    fn $fname $(<$($lt $(: $sup_lt)?),*>)? (
        $(
            $($arg_name)+ : $arg_ty,
        )*
    ) $(-> $Ret)?
    $(
        where
            $($bounds)*
    )?
        $body

    $crate::paste::item! {
        use $fname as [< $fname __orig >];
    }

    #[allow(dead_code, nonstandard_style, unused_parens, clippy::all)]
    const _: () = {
        $($(#[doc = $doc])+)?
        #[allow(improper_ctypes_definitions)]
        #[cfg_attr(not(target_arch = "wasm32"),
            no_mangle,
        )]
        pub
        $(unsafe $(@$hack@)?)? /* Safety: function is not visible but to the linker */
        extern "C"
        fn $fname $(<$($lt $(: $sup_lt)?),*>)? (
            $(
                $($arg_name)+ : <$arg_ty as $crate::layout::ReprC>::CLayout,
            )*
        ) -> <($($Ret)?) as $crate::layout::ReprC>::CLayout
        where
            $( $($bounds)* )?
        {{
            $crate::paste::item! {
                use [< $fname __orig >] as $fname;
            }
            let body = /* #[inline(always)] */ || -> ($($Ret)?) {
                $(
                    {
                        fn __return_type__<T> (_: T)
                        where
                            T : $crate::layout::ConcreteReprC,
                        {}
                        let _ = __return_type__::<$Ret>;
                    }
                )?
                $(
                    $crate::__last__! { $(({
                        mod __parameter__ {
                            pub(in super)
                            fn $arg_name<T> (_: T)
                            where
                                T : $crate::layout::ConcreteReprC,
                            {}
                        }
                        let _ = __parameter__::$arg_name::<$arg_ty>;
                    }))+ }
                    #[allow(unused_unsafe)]
                    let $($arg_name)+: $arg_ty = unsafe {
                        $crate::layout::from_raw_unchecked::<$arg_ty>(
                            $crate::__last__! { $(( $arg_name ))+ },
                        )
                    };
                )*
                $body
            };
            let guard = {
                struct $fname;
                impl
                    $crate::ඞ::ops::Drop
                for
                    $fname
                {
                    fn drop (self: &'_ mut Self)
                    {
                        $crate::__abort_with_msg__!($crate::ඞ::concat!(
                            "Error, attempted to panic across the FFI ",
                            "boundary of `",
                            $crate::ඞ::stringify!($fname),
                            "()`, ",
                            "which is Undefined Behavior.\n",
                            "Aborting for soundness.",
                        ));
                    }
                }
                $fname
            };
            let ret = unsafe {
                $crate::layout::into_raw(body())
            };
            $crate::ඞ::mem::forget(guard);
            ret
        }}

        $crate::paste::item! {
        /// Define the N-API wrapping function.
        #[cfg(any(
            $(
                all(),
                [< __hack_ $node_js_arg_count >] = "",
            )?
        ))]
        const _: () = {
            // We want to use `type $arg_name = <$arg_ty as …>::Assoc;`
            // (with the lifetimes appearing there having been replaced with
            // `'static`, to soothe `#[wasm_bindgen]`).
            //
            // To avoid polluting the namespace with that many `$arg_name`s,
            // we will namespace those type aliases.
            mod __ty_aliases {
                #![allow(nonstandard_style, unused_parens)]
                use super::*;
                $($crate::__last__! { $((
                    // Incidentally, the usage of a `type` alias ensures
                    // `__make_all_lifetimes_static!` is not missing hidden
                    // lifetime parameters in paths (_e.g._, `Cow<str>`, or
                    // more on point, `char_p::Ref`). Indeed, when one does
                    // that inside a type alias, a very nice error message
                    // will complain about it.
                    pub(in super)
                    type $arg_name =
                        $crate::node_js::derive::__make_all_lifetimes_static!(
                            <
                                <$arg_ty as $crate::layout::ReprC>::CLayout
                                as
                                $crate::node_js::ReprNapi
                            >::NapiValue
                        )
                    ;
                ))*})*
            }
            #[$crate::node_js::derive::js_export(js_name = $fname)]
            fn __node_js $(<$($lt $(: $sup_lt)?),*>)? (
                $(
                    $($arg_name)*: $crate::__last__! { $((__ty_aliases::$arg_name))* },
                )*
            ) -> $crate::node_js::Result<$crate::node_js::JsUnknown>
            {
                let __ctx__ = $crate::node_js::derive::__js_ctx!();

                $($crate::__last__! { $((
                    let $arg_name: <$arg_ty as $crate::layout::ReprC>::CLayout =
                        $crate::node_js::ReprNapi::from_napi_value(
                            __ctx__.env,
                            $arg_name,
                        )?
                    ;
                ))*})*

                #[cfg(any(
                    $($(
                        not(target_arch = "wasm32"),
                        __hack = $async_worker,
                    )?)?
                ))] {
                    fn __assert_send<__T : $crate::ඞ::marker::Send> ()
                    {}
                    $($crate::__last__! { $((
                        let $arg_name = unsafe {
                            // The raw `CType` may not be `Send` (_e.g._, it
                            // may be a raw pointer), but we can turn off the
                            // lint if the `ReprC` whence it originated is
                            // `Send`.
                            let _ = __assert_send::<$arg_ty>;
                            $crate::node_js::UnsafeAssertSend::new($arg_name)
                        };
                    ))*})*
                    return
                        napi::JsPromise::from_task_spawned_on_worker_pool(__ctx__.env, move || unsafe {
                            $fname(
                                $(
                                    $crate::node_js::UnsafeAssertSend::into_inner(
                                        $crate::__last__! { $(($arg_name))* }
                                    )
                                ),*
                            )
                        })
                        .map(|it| it.into_unknown());
                }

                #[cfg(all(
                    $($(
                        target_arch = "wasm32",
                        not(__hack = $async_worker),
                    )?)?
                ))] {
                    let ret = unsafe {
                        $fname(
                            $(
                                $crate::__last__! { $(($arg_name))* }
                            ),*
                        )
                    };
                    return
                        napi::ReprNapi::to_napi_value(ret, __ctx__.env)
                        $($(
                            .map(|it| {
                                $crate::ඞ::stringify!($async_worker);
                                $crate::node_js::JsPromise::<napi::JsUnknown>::resolve(it.as_ref())
                            })
                        )?)?
                            .map(|it| it.into_unknown())
                    ;
                }
            }
        };
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    $crate::__cfg_headers__! {
        $crate::inventory::submit! {
            #![crate = $crate]
            $crate::FfiExport { name: $crate::ඞ::stringify!($fname), gen_def: {
                #[allow(unused_parens, clippy::all)]
                fn typedef $(<$($lt $(: $sup_lt)?),*>)? (
                    definer: &'_ mut dyn $crate::ඞ::Definer,
                    lang: $crate::headers::Language,
                ) -> $crate::ඞ::io::Result<()>
                {Ok({
                    // FIXME: this merges the value namespace with the type
                    // namespace...
                    if ! definer.insert($crate::ඞ::stringify!($fname)) {
                        return $crate::ඞ::result::Result::Err(
                            $crate::ඞ::io::Error::new(
                                $crate::ඞ::io::ErrorKind::AlreadyExists,
                                $crate::ඞ::concat!(
                                    "Error, attempted to declare `",
                                    $crate::ඞ::stringify!($fname),
                                    "` while another declaration already exists",
                                ),
                            )
                        );
                    }
                    $(
                        $crate::headers::__define_self__::<$arg_ty>(definer, lang)?;
                    )*
                    $(
                        $crate::headers::__define_self__::<$Ret>(definer, lang)?;
                    )?
                    $crate::headers::__define_fn__(
                        definer,
                        lang,
                        &[
                            $($($doc),*)?
                        ],
                        $crate::ඞ::stringify!($fname),
                        &[
                            $(
                                $crate::ඞ::FunctionArg {
                                    name: $crate::__last__! {$((
                                        $crate::ඞ::stringify!($arg_name)
                                    ))*},
                                    ty: &$crate::ඞ::PhantomData::<
                                        $crate::ඞ::CLayoutOf<$arg_ty>,
                                    >,
                                }
                            ),*
                        ],
                        &$crate::ඞ::PhantomData::<
                            $crate::ඞ::CLayoutOf< ($($Ret)?) >,
                        >,
                    )?;
                })};
                typedef
            }}
        }
    }
);

(
    $(#[doc = $doc:expr])*
    $pub:vis const $VAR:ident : $T:ty = $value:expr;
) => (
    $(#[doc = $doc])*
    $pub const $VAR : $T = $value;

    #[cfg(not(target_arch = "wasm32"))]
    $crate::__cfg_headers__! {
        $crate::inventory::submit! {
            #![crate = $crate]
            $crate::FfiExport {
                name: $crate::ඞ::stringify!($VAR),
                gen_def: {
                    |
                        definer: &'_ mut dyn $crate::ඞ::Definer,
                        lang: $crate::headers::Language,
                    |
                    {
                        {
                            use $crate::headers::{
                                Language,
                                languages::{self, HeaderLanguage},
                            };
                            let header_builder: &'static dyn HeaderLanguage =
                                match lang {
                                    | Language::C => &languages::C,
                                    | Language::CSharp => &languages::CSharp,
                                }
                            ;
                            header_builder
                        }.emit_constant(
                            definer,
                            &[
                                $($($doc),*)?
                            ],
                            $crate::ඞ::stringify!($VAR),
                            &$crate::ඞ::PhantomData::<
                                $crate::ඞ::CLayoutOf< $T >,
                            >,
                            &$VAR,
                        )
                    }
                },
            }
        }
    }
)}

#[doc(hidden)] #[macro_export]
macro_rules! __last__ {
    (
        $not_last:tt
        $($rest:tt)+
    ) => (
        $crate::__last__! {
            $($rest)*
        }
    );

    (
        // last
        (
            $($contents:tt)*
        )
    ) => (
        $($contents)*
    );
}
