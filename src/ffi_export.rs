#[macro_export]
macro_rules! ffi_export_ {(
    $($(#[doc = $doc:expr])+)?
    // $(#[$meta:meta])*
    $pub:vis
    $(unsafe $(@$hack:ident@)?)?
    $(extern $($abi:literal)?)?
    fn $fname:ident $(<$($lt:lifetime),* $(,)?>)? (
        $(
            $arg_name:tt : $arg_ty:ty
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
    $(unsafe $($hack)?)?
    $(extern $($abi)?)?
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

    /// Guard against some param or return type not being `ReprC`
    #[allow(dead_code, nonstandard_style)]
    const _: () = {
        // introduce lifetime parameters
        fn __ $(<$($lt),*>)? ()
        {
            $(
                let _: <$arg_ty as $crate::layout::ReprC>::CLayout;
            )*
            $(
                let _: <$Ret as $crate::layout::ReprC>::CLayout;
            )?
        }
    };

    $crate::cfg_headers! {
        $crate::inventory::submit! {
            $crate::TypeDef(|definer: &'_ mut dyn $crate::layout::Definer| Ok({
                // FIXME: this merges the value namespace with the type
                // namespace...
                if ! definer.insert($crate::core::stringify!($fname)) {
                    return $crate::core::result::Result::Err(
                        $crate::std::io::Error::new(
                            $crate::std::io::ErrorKind::AlreadyExists,
                            $crate::core::concat!(
                                "Error, attempted to declar `",
                                stringify!($fname),
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
                        b"/**\n",
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
            }))
        }
    }
)}

// ffi_export_! {
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
