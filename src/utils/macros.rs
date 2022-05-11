#![cfg_attr(rustfmt, rustfmt::skip)]
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

macro_rules! cfg_wasm {( $($item:item)* ) => (
    $(
        #[cfg(target_arch = "wasm32")]
        $item
    )*
)}
macro_rules! cfg_not_wasm {( $($item:item)* ) => (
    $(
        #[cfg(not(target_arch = "wasm32"))]
        $item
    )*
)}

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
    $( #[doc = $doc:tt] )*
    $pub:vis
    enum $EnumName:ident {
        $(
            $( #[doc = $doc_variant:tt] )*
            $Variant:ident
        ),* $(,)?
    }
) => (type_level_enum! { // This requires the macro to be in scope when called.
    with_docs! {
        $( #[doc = $doc] )*
        ///
        /// ### Type-level `enum`
        ///
        /// Until `const_generics` can handle custom `enum`s, this pattern must be
        /// implemented at the type level.
        ///
        /// We thus end up with:
        ///
        /// ```rust,ignore
        /// #[type_level_enum]
        #[doc = ::core::concat!(
            " enum ", ::core::stringify!($EnumName), " {",
        )]
        $(
            #[doc = ::core::concat!(
                "         ", ::core::stringify!($Variant), ",",
            )]
        )*
        #[doc = " }"]
        /// ```
        ///
        #[doc = ::core::concat!(
            "With [`", ::core::stringify!($EnumName), "::T`](#reexports) \
            being the type-level \"enum type\":",
        )]
        ///
        /// ```rust,ignore
        #[doc = ::core::concat!(
            "<Param: ", ::core::stringify!($EnumName), "::T>"
        )]
        /// ```
    }
    #[allow(warnings)]
    $pub mod $EnumName {
        #[doc(no_inline)]
        pub use $EnumName as T;

        type_level_enum! {
            with_docs! {
                #[doc = ::core::concat!(
                    "See [`", ::core::stringify!($EnumName), "`]\
                    [super::", ::core::stringify!($EnumName), "]"
                )]
            }
            pub trait $EnumName : __sealed::$EnumName + ::core::marker::Sized + 'static {
                const VALUE: __value::$EnumName;
            }
        }

        mod __sealed { pub trait $EnumName {} }

        mod __value {
            #[derive(Debug, PartialEq, Eq)]
            pub enum $EnumName { $( $Variant ),* }
        }

        $(
            $( #[doc = $doc_variant] )*
            pub enum $Variant {}
            impl __sealed::$EnumName for $Variant {}
            impl $EnumName for $Variant {
                const VALUE: __value::$EnumName = __value::$EnumName::$Variant;
            }
            impl $Variant {
                pub const VALUE: __value::$EnumName = __value::$EnumName::$Variant;
            }
        )*
    }
});(
    with_docs! {
        $( #[doc = $doc:expr] )*
    }
    $item:item
) => (
    $( #[doc = $doc] )*
    $item
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

/// Items exported through this macro are internal implementation details
/// that exported macros may need access to.
///
/// Users of this crate should not directly use them (unless the pinpoint the
/// version dependency), since they are considered to be semver-exempt and
/// could thus cause breakage.
macro_rules! hidden_export {
    (
        $(#[$attr:meta])*
        macro_rules! $($rest:tt)*
    ) => (
        $(#[$attr])*
        #[doc(hidden)] /** Not part of the public API **/ #[macro_export]
        macro_rules! $($rest)*
    );

    (
        @attrs[ $($attr:tt)* ]
        #[$current:meta]
        $($rest:tt)*
    ) => (
        hidden_export! {
            @attrs[ $($attr)* $current ]
            $($rest)*
        }
    );

    (
        @attrs[ $($attr:tt)* ]
        $($item:tt)*
    ) => (
        $(#[$attr])*
        #[doc(hidden)] /** Not part of the public API **/ pub
        $($item)*
    );

    (
        $($input:tt)*
    ) => (
        hidden_export! {
            @attrs[]
            $($input)*
        }
    )
}

macro_rules! match_ {(
    ( $($scrutinee:tt)* ) $rules:tt
) => (
    macro_rules! __recurse__ $rules
    __recurse__! { $($scrutinee)* }
)}

macro_rules! emit {( $($tt:tt)* ) => (
    $($tt)*
)}
