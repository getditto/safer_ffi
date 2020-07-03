#[doc(hidden)] #[macro_export]
macro_rules! __ffi_export__ {
(
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

    #[allow(dead_code, nonstandard_style, unused_parens)]
    const _: () = {
        $($(#[doc = $doc])+)?
        #[no_mangle]
        pub
        $(unsafe $(@$hack@)?)? /* Safety: function is not visible but to the linker */
        extern "C"
        fn $fname $(<$($lt $(: $sup_lt)?),*>)? (
            $(
                $arg_name : <$arg_ty as $crate::layout::ReprC>::CLayout,
            )*
        ) $(-> $Ret)?
        where
            $( $($bounds)* )?
        {{
            let body = /* #[inline(always)] */ || {
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
            let ret = body();
            $crate::core::mem::forget(guard);
            ret
        }}
    };

    $crate::__cfg_headers__! {
        $crate::inventory::submit! {
            #![crate = $crate]
            $crate::FfiExport({
                #[allow(unused_parens)]
                fn typedef $(<$($lt $(: $sup_lt)?),*>)? (
                    definer: &'_ mut dyn $crate::headers::Definer,
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
                        <
                            <$arg_ty as $crate::layout::ReprC>::CLayout
                            as
                            $crate::layout::CType
                        >::c_define_self(definer)?;
                    )*
                    $(
                        <
                            <$Ret as $crate::layout::ReprC>::CLayout
                            as
                            $crate::layout::CType
                        >::c_define_self(definer)?;
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

                    $crate::core::write!(out,
                        "{} (",
                        <
                            <($($Ret)?) as $crate::layout::ReprC>::CLayout
                            as
                            $crate::layout::CType
                        >::c_var($crate::core::stringify!($fname)),
                    )?;
                    // $crate::std::io::Write::write_all(out,
                    //     $crate::core::concat!($crate::core::stringify!($fname), " (")
                    //         .as_bytes()
                    //     ,
                    // )?;
                    let mut has_args = false; has_args = has_args;
                    $(
                        $crate::core::write!(out,
                            "{comma}\n    {arg}",
                            comma = if has_args { "," } else { "" },
                            arg = <
                                    <$arg_ty as $crate::layout::ReprC>::CLayout
                                    as
                                    $crate::layout::CType
                                >::c_var({
                                    let it = stringify!($arg_name);
                                    if it == "_" { "" } else { it }
                                })
                            ,
                        )?;
                        has_args |= true;
                    )*
                    if has_args.not() {
                        out.write_all(b"void")?;
                    }
                    drop(has_args);
                    $crate::std::io::Write::write_all(out,
                        ");\n\n"
                            .as_bytes()
                        ,
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
);

(
    $(#[doc = $doc:expr])*
    $pub:vis static $VAR:ident : $T:ty = $value:expr;
) => (
    $(#[doc = $doc])*
    #[no_mangle]
    $pub static $VAR : $T = $value;

    $crate::__cfg_headers__! {
        $crate::inventory::submit! {
            #![crate = $crate]
            $crate::FfiExport(|definer: &mut dyn $crate::headers::Definer| {
                write!(definer.out(), "extern {};\n",
                    <
                        <$T as $crate::layout::ReprC>::CLayout
                        as
                        $crate::layout::CType
                    >::c_var(stringify!($VAR))
                )
            })
        }
    }
);

} // End of `ffi_export!` macro

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
