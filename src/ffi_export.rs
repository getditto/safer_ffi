#[doc(hidden)] #[macro_export]
macro_rules! __ffi_export__ {(
    $($(#[doc = $doc:expr])+)?
    // $(#[$meta:meta])*
    $pub:vis
    $(unsafe $(@$hack:ident@)?)?
    // $(extern $($abi:literal)?)?
    fn $fname:ident $(<$($lt:lifetime),* $(,)?>)? (
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
    // $(extern $($abi)?)?
    fn $fname $(<$($lt),*>)? (
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
        unsafe extern "C"
        fn $fname $(<$($lt),*>)? (
            $(
                $arg_name : <$arg_ty as $crate::layout::ReprC>::CLayout,
            )*
        ) $(-> $Ret)?
        $(
            where
                $($bounds)*
        )?
        {{
            // let body = #[inline(always)] || {
                $(
                    let _: <$Ret as $crate::layout::ReprC>::CLayout;
                )?
                $(
                    let $arg_name: $arg_ty = $crate::layout::from_raw_unchecked::<$arg_ty>(
                        $arg_name,
                    );
                )*
                $body
            // };
            // let guard = {
            //     struct $fname;
            //     impl $crate::core::ops::Drop
            //         for $fname
            //     {
            //         fn drop (self: &'_ mut Self)
            //         {
            //             $crate::core::panic!($crate::core::concat!(
            //                 "Error, attempted to panic across the FFI ",
            //                 "boundary of `",
            //                 $crate::core::stringify!($fname),
            //                 "()`, ",
            //                 "which is Undefined Behavior.\n",
            //                 "Aborting for soundness.",
            //             ));
            //         }
            //     }
            //     $fname
            // };
            // let ret = body();
            // $crate::core::mem::forget(guard);
            // ret
        }}
    };

    $crate::__cfg_headers__! {
        $crate::inventory::submit! {
            #![crate = $crate]
            $crate::FfiExport({
                fn typedef $(<$($lt),*>)? (
                    definer: &'_ mut dyn $crate::layout::Definer,
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
                        >::c_display($crate::core::stringify!($fname)),
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
                                >::c_display({
                                    let it = stringify!($arg_name);
                                    if it == "_" { "" } else { it }
                                })
                            ,
                        )?;
                        has_args |= true;
                    )*
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
//     fn max<'a> (
//         ints: crate::slice::slice_ref<'a, i32>
//     ) -> Option<&'a i32>
//     {
//         ints.as_slice().iter().max()
//     }
// }
