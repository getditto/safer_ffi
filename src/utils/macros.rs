#![allow(unused_macros)]

macro_rules! use_prelude { () => (
    #[allow(unused_imports)]
    use crate::utils::prelude::*;
)}

/// I really don't get the complexity of `cfg_if!`…
///
/// ```rust ,ignore
/// // USAGE:
/// match_cfg! {
///     <CFG PREDICATE> => {
///         <expansion if true>
///     },
///     // e.g.
///     feature = "foo" => { ... },
///     unix => { ... }
///     // trailing `_` possible for the final "else" branch
///     _ => { ... }
/// }
/// ```
macro_rules! match_cfg {
    (
        _ => { $($expansion:tt)* } $(,)?
    ) => (
        $($expansion)*
    );

    (
        $cfg:meta => $expansion:tt $(,
        $($($rest:tt)+)? )?
    ) => (
        #[cfg($cfg)]
        match_cfg! { _ => $expansion } $($(

        #[cfg(not($cfg))]
        match_cfg! { $($rest)+ } )?)?
    );

    // Bonus: expression-friendly syntax: `match_cfg!({ … })`
    ({
        $($input:tt)*
    }) => ({
        match_cfg! { $($input)* }
    });
}

match_cfg! {
    feature = "alloc" => {
        macro_rules! cfg_alloc {(
            $($item:item)*
        ) => (
            $(
                #[cfg_attr(all(feature = "docs"), doc(cfg(feature = "alloc")))]
                $item
            )*
        )}
    },
    _ => {
        macro_rules! cfg_alloc {(
            $($item:item)*
        ) => (
            // Nothing
        )}
    },
}

match_cfg! {
    feature = "std" => {
        macro_rules! cfg_std {(
            $($item:item)*
        ) => (
            $(
                #[cfg_attr(all(feature = "docs"), doc(cfg(feature = "std")))]
                $item
            )*
        )}
    },
    _ => {
        macro_rules! cfg_std {(
            $($item:item)*
        ) => (
            // Nothing
        )}
    },
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
) => (
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
        "<Param : ", ::core::stringify!($EnumName), "::T>"
    )]
    /// ```
    #[allow(warnings)]
    $pub mod $EnumName {
        #[doc(no_inline)]
        pub use $EnumName as T;

        #[doc = ::core::concat!(
            "See [`", ::core::stringify!($EnumName), "`]",
            "[super::", ::core::stringify!($EnumName), "]"
        )]
        pub
        trait $EnumName
        :
            __sealed::$EnumName +
            ::core::marker::Sized +
            'static +
        {
            const VALUE: __value::$EnumName;
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
                const VALUE: __value::$EnumName =
                    __value::$EnumName::$Variant
                ;
            }
            impl $Variant {
                pub const VALUE: __value::$EnumName =
                    __value::$EnumName::$Variant
                ;
            }
        )*
    }
)}

macro_rules! doc_test {
    ($name:ident :
        #![$attr:ident]
        $($code:tt)*
    ) => (
        const _: () = {
            #[doc = concat!(
                "```rust,", stringify!($attr), "\n",
                stringify!($($code)*), "\n",
            )]
            /// ```
            pub mod $name {}
        };
    );

    ($name:ident :
        $($code:tt)*
    ) => (
        const _: () = {
            #[doc = concat!(
                "```rust\n",
                stringify!($($code)*), "\n",
            )]
            /// ```
            pub mod $name {}
        };
    );
}

doc_test! { c_str:
    use ::safer_ffi::prelude::*;

    let _ = c!("Hello, World!");
}
doc_test! { c_str_inner_nul_byte:
    #![compile_fail]
    use ::safer_ffi::prelude::*;

    let _ = c!("Hell\0, World!");
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
