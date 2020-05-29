#![allow(unused_macros)]

macro_rules! use_prelude { () => (
    #[allow(unused_imports)]
    use crate::utils::prelude::*;
)}

#[macro_use]
mod cfg_alloc {
    #[cfg(
        feature = "alloc",
    )]
    macro_rules! cfg_alloc {(
        $($item:item)*
    ) => (
        $(
            #[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "alloc")))]
            $item
        )*
    )}

    #[cfg(not(
        feature = "alloc",
    ))]
    macro_rules! cfg_alloc {(
        $($item:item)*
    ) => (
        // Nothing
    )}
}

#[macro_use]
mod cfg_std {
    #[cfg(
        feature = "std",
    )]
    macro_rules! cfg_std {(
        $($item:item)*
    ) => (
        $(
            #[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "std")))]
            $item
        )*
    )}

    #[cfg(not(
        feature = "std",
    ))]
    macro_rules! cfg_std {(
        $($item:item)*
    ) => (
        // Nothing
    )}
}

#[macro_use]
mod cfg_proc_macros {
    #[cfg(
        feature = "proc_macros",
    )]
    macro_rules! cfg_proc_macros {(
        $($item:item)*
    ) => (
        $(
            #[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "proc_macros")))]
            $item
        )*
    )}

    #[cfg(not(
        feature = "proc_macros",
    ))]
    macro_rules! cfg_proc_macros {(
        $($item:item)*
    ) => (
        // Nothing
    )}
}

macro_rules! const_assert {
    (
        for [$($generics:tt)*]
        [$($($pre:tt)+)?] => [$($post:tt)*]
    ) => (
        const _: () = {
            fn check<$($generics)*> ()
            where
                $($($pre)+)?
            {
                fn const_assert<$($generics)*> ()
                where
                    $($($pre)* ,)?
                    $($post)*
                {}
                let _ = const_assert::<$($generics)*>;
            }
        };
    );

    (
        $cond:expr
    ) => (
        const _: [(); 1] = [(); {
            const COND: bool = $cond;
            COND
        } as usize];
    );
}

macro_rules! type_level_enum {(
    $(#[$meta:meta])*
    $pub:vis
    enum $EnumName:ident {
        $(
            $(#[$variant_meta:meta])*
            $Variant:ident
        ),+ $(,)?
    }
) => (
    #[allow(
        bad_style,
        missing_copy_implementations,
        missing_debug_implementations,
    )]
    $(#[$meta])*
    $pub
    mod $EnumName {
        use private::Sealed; mod private { pub trait Sealed {} }
        pub trait __ : Sealed {}
        $(
            $(#[$variant_meta])*
            pub
            enum $Variant {}
                impl __ for $Variant {} impl Sealed for $Variant {}
        )+
    }
)}

macro_rules! with_doc {(
    #[doc = $doc:expr]
    $($rest:tt)*
) => (
    #[doc = $doc]
    $($rest)*
)}

macro_rules! doc_test {
    ($name:ident :
        #![$attr:ident]
        $($code:tt)*
    ) => (
        with_doc! {
            #[doc = concat!(
                "```rust,", stringify!($attr), "\n",
                stringify!($($code)*),
                "\n```\n",
            )]
            pub mod $name {}
        }
    );

    ($name:ident :
        $($code:tt)*
    ) => (
        with_doc! {
            #[doc = concat!(
                "```rust\n",
                stringify!($($code)*),
                "\n```\n",
            )]
            pub mod $name {}
        }
    );
}

cfg_proc_macros! {
    doc_test! { c_str:
        use ::safer_ffi::prelude::*;

        let _ = c!("Hello, World!");
    }
    doc_test! { c_str_inner_nul_byte:
        #![compile_fail]
        use ::safer_ffi::prelude::*;

        let _ = c!("Hell\0, World!");
    }
}
