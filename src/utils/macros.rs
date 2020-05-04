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

macro_rules! trait_alias {(
    $(#[$meta:meta])*
    $pub:vis
    trait $TraitName:ident
    where
        $($bounds:tt)*
) => (
    $(#[$meta])*
    $pub
    trait $TraitName
    where
        $($bounds)*

    impl<__ : ?Sized> $TraitName for __
    where
        $($bounds)*

)}
