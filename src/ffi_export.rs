#![cfg_attr(rustfmt, rustfmt::skip)]
#[doc(hidden)] #[macro_export]
macro_rules! __ffi_export__ {(
    $( @[node_js($node_js_arg_count:literal)] )?

    $($(#[doc = $doc:expr])+)?
    // $(#[$meta:meta])*
    $pub:vis
    $(unsafe $(@$hack:ident@)?)?
    $(extern $("C")?)?
    fn $fname:ident $(<$($lt:lifetime $(: $sup_lt:lifetime)?),* $(,)?>)? (
        $(
            $arg_name:ident : $arg_ty:ty
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
            $arg_name : $arg_ty,
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

    #[allow(dead_code, nonstandard_style, unused_parens)]
    const _: () = {
        $($(#[doc = $doc])+)?
        #[allow(improper_ctypes_definitions)]
        #[no_mangle]
        pub
        $(unsafe $(@$hack@)?)? /* Safety: function is not visible but to the linker */
        extern "C"
        fn $fname $(<$($lt $(: $sup_lt)?),*>)? (
            $(
                $arg_name : <$arg_ty as $crate::layout::ReprC>::CLayout,
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
                            T : $crate::layout::ReprC,
                            <T as $crate::layout::ReprC>::CLayout
                            :
                            $crate::layout::CType<
                                OPAQUE_KIND = $crate::layout::OpaqueKind::Concrete,
                            >,
                        {}
                        let _ = __return_type__::<$Ret>;
                    }
                    let _: <$Ret as $crate::layout::ReprC>::CLayout;
                )?
                $(
                    {
                        mod __parameter__ {
                            pub(in super)
                            fn $arg_name<T> (_: T)
                            where
                                T : $crate::layout::ReprC,
                                <T as $crate::layout::ReprC>::CLayout
                                :
                                $crate::layout::CType<
                                    OPAQUE_KIND = $crate::layout::OpaqueKind::Concrete,
                                >,
                            {}
                        }
                        let _ = __parameter__::$arg_name::<$arg_ty>;
                    }
                    #[allow(unused_unsafe)]
                    let $arg_name: $arg_ty = unsafe {
                        $crate::layout::from_raw_unchecked::<$arg_ty>(
                            $arg_name,
                        )
                    };
                )*
                $body
            };
            let guard = {
                struct $fname;
                impl $crate::core::ops::Drop
                    for $fname
                {
                    fn drop (self: &'_ mut Self)
                    {
                        $crate::__abort_with_msg__!($crate::core::concat!(
                            "Error, attempted to panic across the FFI ",
                            "boundary of `",
                            $crate::core::stringify!($fname),
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
            $crate::core::mem::forget(guard);
            ret
        }}

        #[cfg(any(
            $(
                all(),
                __hack = $node_js_arg_count
            )?
        ))]
        /// Define the N-API wrapping function.
        const _: () = {
            use ::safer_ffi::node_js as napi;

            $( #[napi::js_function($node_js_arg_count)] )?
            fn __node_js $(<$($lt $(: $sup_lt)?),*>)? (ctx: napi::CallContext<'_>)
              -> napi::Result<impl napi::NapiValue>
            {
                let mut __nodejs_arg_idx = 0;
                $(
                    let $arg_name: <$arg_ty as $crate::layout::ReprC>::CLayout =
                        napi::extract_arg(&ctx, __nodejs_arg_idx)?
                    ;
                    let __nodejs_arg_idx = __nodejs_arg_idx + 1;
                )*
                let ret = unsafe {
                    $fname($($arg_name),*)
                };
                napi::ReprNapi::to_napi_value(ret, ctx.env)
            }

            /// Register the N-API defined function.
            ::safer_ffi::node_js::registering::submit! {
                #![crate = ::safer_ffi::node_js::registering]
                ::safer_ffi::node_js::registering::NapiRegistryEntry::NamedMethod {
                    name: stringify!($fname),
                    method: __node_js,
                }
            }
        };
    };

    $crate::__cfg_headers__! {
        $crate::inventory::submit! {
            #![crate = $crate]
            $crate::FfiExport({
                #[allow(unused_parens)]
                fn typedef $(<$($lt $(: $sup_lt)?),*>)? (
                    definer: &'_ mut dyn $crate::headers::Definer,
                    lang: $crate::headers::Language,
                ) -> $crate::std::io::Result<()>
                {Ok({
                    // FIXME: this merges the value namespace with the type
                    // namespace...
                    if ! definer.insert($crate::core::stringify!($fname)) {
                        return $crate::core::result::Result::Err(
                            $crate::std::io::Error::new(
                                $crate::std::io::ErrorKind::AlreadyExists,
                                $crate::core::concat!(
                                    "Error, attempted to declare `",
                                    $crate::core::stringify!($fname),
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
                    let out = definer.out();
                    $(
                        $crate::std::io::Write::write_all(out,
                            b"/** \\brief\n",
                        )?;
                        $(
                            $crate::core::write!(out,
                                " * {}\n", $doc,
                            )?;
                        )+
                        $crate::std::io::Write::write_all(out,
                            b" */\n",
                        )?;
                    )?
                    drop(out); // of school?

                    let mut fname_and_args = String::new();
                    $crate::headers::__define_fn__::name(
                        &mut fname_and_args,
                        $crate::core::stringify!($fname),
                        lang,
                    );
                    $(
                        $crate::headers::__define_fn__::arg::<$arg_ty>(
                            &mut fname_and_args,
                            $crate::core::stringify!($arg_name),
                            lang,
                        );
                    )*
                    $crate::headers::__define_fn__::ret::<($($Ret)?)>(
                        definer,
                        lang,
                        fname_and_args,
                    )?;
                })};
                typedef
            })
        }
    }
);

(
    $(#[doc = $doc:expr])*
    $pub:vis const $VAR:ident : $T:ty = $value:expr;
) => (
    $(#[doc = $doc])*
    $pub const $VAR : $T = $value;

    $crate::__cfg_headers__! {
        $crate::inventory::submit! {
            #![crate = $crate]
            $crate::FfiExport(|definer: &mut dyn $crate::headers::Definer| {
                write!(
                    definer.out(),
                    concat!(
                        "\n#define ",
                        stringify!($VAR),
                        " (({ty_cast}) ({expr}))\n\n",
                    ),
                    ty_cast =
                        <
                            <$T as $crate::layout::ReprC>::CLayout
                            as
                            $crate::layout::CType
                        >::c_var("")
                    ,
                    expr = stringify!($value),
                )
            })
        }
    }
)}

// __ffi_export__! {
//     /// Concatenate two strings
//     fn concat (
//         fst: crate::char_p::char_p_ref<'_>,
//         snd: crate::char_p::char_p_ref<'_>,
//     ) -> crate::char_p::char_p_boxed
//     {
//         use ::core::convert::TryInto;
//         format!("{}{}\0", fst.to_str(), snd.to_str())
//             .try_into()
//             .unwrap()
//     }
// }

// __ffi_export__! {
//     /// Some docstring
//     fn max<'a, 'b : 'a> (
//         ints: crate::slice::slice_ref<'a, i32>
//     ) -> Option<&'a i32>
//     {
//         ints.as_slice().iter().max()
//     }
// }
