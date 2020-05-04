#[cfg(feature = "headers")]
#[macro_export] #[doc(hidden)]
macro_rules! __cfg_headers__ {(
    $($item:item)*
) => (
    $($item)*
)}
#[cfg(not(feature = "headers"))]
#[macro_export] #[doc(hidden)]
macro_rules! __cfg_headers__ {(
    $($item:item)*
) => (
    // nothing
)}

#[cfg(not(feature = "headers"))]
#[macro_export] #[doc(hidden)]
macro_rules! __cfg_not_headers__ {(
    $($item:item)*
) => (
    $($item)*
)}
#[cfg(feature = "headers")]
#[macro_export] #[doc(hidden)]
macro_rules! __cfg_not_headers__ {(
    $($item:item)*
) => (
    // nothing
)}

#[macro_export] #[doc(hidden)]
macro_rules! __with_doc__ {(
    #[doc = $doc:expr]
    $(#[$meta:meta])*
    $pub:vis
    struct
    $($rest:tt)*
) => (
    $(#[$meta])*
    #[doc = $doc]
    $pub
    struct
    $($rest)*
)}
/// Safely implement [`CType`][`trait@crate::layout::CType`]
/// for a `#[repr(C)]` struct **when all its fields are `CType`**.
///
/// Note: you rarely need to call this macro directly. Instead, look for the
/// [`ReprC!`] macro to safely implement [`ReprC`][`trait@crate::layout::ReprC`].
#[macro_export]
macro_rules! CType {(
    $(
        @doc_meta( $($doc_meta:tt)* )
    )?
    #[repr(C)]
    $(#[$($meta:tt)*])*
    $pub:vis
    struct $StructName:ident $(
        [
            $($lt:lifetime ,)*
            $($($generics:ident),+ $(,)?)?
        ]
            $(where { $($bounds:tt)* })?
    )?
    {
        $(
            $(#[$($field_meta:tt)*])*
            $field_pub:vis
            $field_name:ident : $field_ty:ty
        ),+ $(,)?
    }
) => (
    #[repr(C)]
    $(#[$($meta)*])*
    $pub
    struct $StructName
        $(<$($lt ,)* $($($generics),+)?> $(where $($bounds)* )?)?
    {
        $(
            $(#[$($field_meta)*])*
            $field_pub
            $field_name : $field_ty,
        )*
    }

    unsafe // Safety: struct is `#[repr(C)]` and contains `CType` fields
    impl $(<$($lt ,)* $($($generics),+)?>)? $crate::layout::CType
        for $StructName$(<$($lt ,)* $($($generics),+)?>)?
    where
        // $(
        //     $field_ty : $crate::layout::CType,
        // )*
        $(
            $($(
                $generics : $crate::layout::ReprC,
            )+)?
            $($($bounds)*)?
        )?
    { $crate::__cfg_headers__! {
        fn c_short_name_fmt (fmt: &'_ mut $crate::core::fmt::Formatter<'_>)
          -> $crate::core::fmt::Result
        {
            fmt.write_str($crate::core::stringify!($StructName))?;
            $($(
                $(
                    $crate::core::write!(fmt, "_{}",
                        <
                            <$generics as $crate::layout::ReprC>::CLayout
                            as
                            $crate::layout::CType
                        >::c_short_name()
                    )?;
                )+
            )?)?
            Ok(())
        }

        fn c_define_self (definer: &'_ mut dyn $crate::headers::Definer)
          -> $crate::std::io::Result<()>
        {
            assert_ne!(
                $crate::core::mem::size_of::<Self>(), 0,
                "C does not support zero-sized structs!",
            );
            let ref me =
                <Self as $crate::layout::CType>
                    ::c_short_name().to_string()
            ;
            definer.define_once(
                me,
                &mut |definer| {
                    $(
                        <$field_ty as $crate::layout::CType>::c_define_self(definer)?;
                    )*
                    let out = definer.out();
                    $(
                        $crate::__output_docs__!(out, "", $($doc_meta)*);
                    )?
                    $crate::__output_docs__!(out, "", $(#[$($meta)*])*);
                    $crate::core::writeln!(out, "typedef struct {{\n")?;
                    $(
                        if $crate::core::mem::size_of::<$field_ty>() > 0 {
                            // $crate::core::writeln!(out, "")?;
                            $crate::__output_docs__!(out, "    ",
                                $(#[$($field_meta)*])*
                            );
                            $crate::core::writeln!(out, "    {};\n",
                                <$field_ty as $crate::layout::CType>::c_var(
                                    $crate::core::stringify!($field_name),
                                ),
                            )?;
                        } else {
                            assert_eq!(
                                $crate::core::mem::align_of::<$field_ty>(),
                                1,
                                $crate::core::concat!(
                                    "Zero-sized fields must have an ",
                                    "alignment of `1`."
                                ),
                            );
                        }
                    )+
                    $crate::core::writeln!(out, "}} {}_t;\n", me)
                },
            )
        }

        fn c_var_fmt (
            fmt: &'_ mut $crate::core::fmt::Formatter<'_>,
            var_name: &'_ str,
        ) -> $crate::core::fmt::Result
        {
            $crate::core::write!(fmt,
                "{}_t{sep}{}",
                <Self as $crate::layout::CType>::c_short_name(),
                var_name,
                sep = if var_name.is_empty() { "" } else { " " },
            )
        }
    } type OPAQUE_KIND = $crate::layout::OpaqueKind::Concrete; }

    $crate::layout::from_CType_impl_ReprC! {
        $(@for [$($lt ,)* $($($generics),+)?])?
            $StructName
                $(<$($lt ,)* $($($generics),+)?>
                    where
                        $($($bounds)*)?
                )?
    }
)}

/// Safely implement [`ReprC`][`trait@crate::layout::ReprC`]
/// for a `#[repr(C)]` struct **when all its fields are `ReprC`**.
///
/// # Syntax
///
/// Note: given that this macro is implemented as a `macro_rules!` macro for
/// the sake of compilation speed, it cannot parse arbitrary generic parameters
/// and where clauses.
///
/// Instead, it expects a special syntax whereby the generic parameters are
/// written between square brackets, and only introduce `lifetime` parameters
/// and type parameters, with no bounds whatsoever (and a necessary trailing
/// comma for the lifetime parameters); the bounds must all be added to the
/// optional following `where { <where clauses here> }` (note the necessary
/// braces):
///
///   - Instead of:
///
///     ```rust,compile_fail
///     use ::repr_c::layout::ReprC;
///
///     ReprC! {
///         #[repr(C)]
///         struct GenericStruct<'lifetime, T : 'lifetime>
///         where
///             T : ReprC,
///         {
///             inner: &'lifetime T,
///         }
///     }
///     ```
///
///   - You need to write:
///
///     ```rust
///     use ::repr_c::layout::ReprC;
///
///     ReprC! {
///         #[repr(C)]
///         struct GenericStruct['lifetime, T]
///         where {
///             T : 'lifetime + ReprC,
///         }
///         {
///             inner: &'lifetime T,
///         }
///     }
///     # fn main () {}
///     ```
///
/// # `#[derive_ReprC]`
///
/// If all this looks cumbersome to you, and if you don't care about the
/// compilation-from-scratch time, then it is highly advised you enable
/// the `proc_macros` feature:
///
/// ```toml
/// [dependencies]
/// repr_c = { version = "...", features = ["proc_macros"] }
/// ```
///
/// and use the [`#[derive_ReprC]`](
/// /repr_c/layout/attr.derive_ReprC.html) attribute macro instead,
/// which will do the rewriting for you.
#[macro_export]
macro_rules! ReprC {
    // struct
    (
        $( @[doc = $doc:expr] )?
        #[repr(C)]
        $(#[$($meta:tt)*])*
        $pub:vis
        struct $StructName:ident $(
            [
                $($lt:lifetime ,)*
                $($($generics:ident),+ $(,)?)?
            ]
                $(where { $($bounds:tt)* })?
        )?
        {
            $(
                $(#[$($field_meta:tt)*])*
                $field_pub:vis
                $field_name:ident : $field_ty:ty
            ),+ $(,)?
        }
    ) => (
        $crate::__with_doc__! {
            #[doc = $crate::core::concat!(
                "  - [`",
                $crate::core::stringify!($StructName),
                "_Layout`]"
            )]
            #[repr(C)]
            $(#[doc = $doc])?
            $(#[$($meta)*])*
            /// # C Layout
            ///
            $pub
            struct $StructName $(
                <$($lt ,)* $($($generics),+)?> $(
                    where $($bounds)*
                )?
            )?
            {
                $(
                    $(#[$($field_meta)*])*
                    $field_pub
                    $field_name : $field_ty,
                )*
            }
        }

        $crate::paste::item! {
            #[allow(nonstandard_style)]
            $pub use
                [< __ $StructName _repr_c_mod >]::$StructName
                as
                [< $StructName _Layout >]
            ;
        }

        #[allow(trivial_bounds)]
        unsafe // Safety: struct is `#[repr(C)]` and contains `ReprC` fields
        impl $(<$($lt ,)* $($($generics),+)?>)? $crate::layout::ReprC
            for $StructName $(<$($lt ,)* $($($generics),+)?>)?
        where
            $(
                $field_ty : $crate::layout::ReprC,
            )*
            $(
                $($(
                    $generics : $crate::layout::ReprC,
                )+)?
                $($($bounds)*)?
            )?
        {

            type CLayout = $crate::paste::__item__! {
                [<$StructName _Layout>]
                    $(<$($lt ,)* $($($generics),+)?>)?
            };

            #[inline]
            fn is_valid (it: &'_ Self::CLayout)
                -> bool
            {
                let _ = it;
                true $(
                    && (
                        $crate::core::mem::size_of::<
                            <$field_ty as $crate::layout::ReprC>::CLayout
                        >() == 0
                        ||
                        <$field_ty as $crate::layout::ReprC>::is_valid(
                            &it.$field_name
                        )
                    )
                )*
            }
        }
        $crate::paste::item! {
            #[allow(nonstandard_style, trivial_bounds)]
            mod [< __ $StructName _repr_c_mod >] {
                #[allow(unused_imports)]
                use super::*;

                $crate::layout::CType! {
                    @doc_meta( $(#[$($meta)*])* )
                    #[repr(C)]
                    #[allow(missing_debug_implementations)]
                    // $(#[$meta])*
                    pub
                    struct $StructName
                        [$($($lt ,)* $($($generics),+)?)?]
                    where {
                        $(
                            $field_ty : $crate::layout::ReprC,
                        )*
                        $(
                            $($(
                                $generics : $crate::layout::ReprC,
                            )+)?
                            $($($bounds)*)?
                        )?
                    } {
                        $(
                            $(#[$($field_meta)*])*
                            pub
                            $field_name :
                                <$field_ty as $crate::layout::ReprC>::CLayout
                            ,
                        )*
                    }
                }
            }
        }
        const _: () = {
            $crate::paste::item! {
                use [< __ $StructName _repr_c_mod >]::*;
            }

            impl $(<$($lt ,)* $($($generics),+)?>)? $crate::core::marker::Copy
                for $StructName $(<$($lt ,)* $($($generics),+)?>)?
            where
                $(
                    $field_ty : $crate::layout::ReprC,
                )*
                $(
                    $($(
                        $generics : $crate::layout::ReprC,
                    )+)?
                    $($($bounds)*)?
                )?
            {}

            impl $(<$($lt ,)* $($($generics),+)?>)? $crate::core::clone::Clone
                for $StructName $(<$($lt ,)* $($($generics),+)?>)?
            where
                $(
                    $field_ty : $crate::layout::ReprC,
                )*
                $(
                    $($(
                        $generics : $crate::layout::ReprC,
                    )+)?
                    $($($bounds)*)?
                )?
            {
                #[inline]
                fn clone (self: &'_ Self)
                    -> Self
                {
                    *self
                }
            }
        };
    );

    // `#[repr(transparent)]`
    (
        $( @[doc = $doc:expr] )?
        #[repr(transparent)]
        $(#[$meta:meta])*
        $pub:vis
        struct $StructName:ident $(
            [$($generics:tt)*] $(
                where { $($bounds:tt)* }
            )?
        )?
        (
            $(#[$field_meta:meta])*
            $field_pub:vis
            $field_ty:ty $(,
            $($rest:tt)* )?
        );
    ) => (
        $crate::__with_doc__! {
            #[doc = $crate::core::concat!(
                " - [`",
                $crate::core::stringify!($field_ty),
                "`](#impl-ReprC)",
            )]
            #[repr(transparent)]
            $(#[doc = $doc])?
            $(#[$meta])*
            /// # C Layout
            ///
            $pub
            struct $StructName $(
                <$($generics)*>
            )?
            (
                $(#[$field_meta])*
                $field_pub
                $field_ty,
                $($($rest)*)?
            )
                $($(where $($bounds)*)?)?
            ;
        }

        #[allow(trivial_bounds)]
        unsafe // Safety: struct is `#[repr(C)]` and contains `ReprC` fields
        impl $(<$($generics)*>)? $crate::layout::ReprC
            for $StructName $(<$($generics)*>)?
        where
            $field_ty : $crate::layout::ReprC,
            $($(
                $($bounds)*
            )?)?
        {
            type CLayout = <$field_ty as $crate::layout::ReprC>::CLayout;

            #[inline]
            fn is_valid (it: &'_ Self::CLayout)
              -> bool
            {
                <$field_ty as $crate::layout::ReprC>::is_valid(
                    it
                )
            }
        }
    );

    // field-less `enum`
    (
        #[repr($Int:ident)]
        $(#[$($meta:tt)*])*
        $pub:vis
        enum $EnumName:ident {
            $(
                $(#[$variant_meta:meta])*
                $Variant:ident $(= $discriminant:expr)?
            ),+ $(,)?
        }
    ) => (
        $crate::layout::ReprC! {
            @validate_int_repr $Int
        }
        $crate::layout::ReprC! {
            @deny_C $Int
        }

        #[repr($Int)]
        $(#[$($meta)*])*
        $pub
        enum $EnumName {
            $(
                $(#[$variant_meta])*
                $Variant $(= $discriminant)? ,
            )+
        }

        ::paste::item! {
            #[repr(transparent)]
            #[derive(Clone, Copy, PartialEq, Eq)]
            pub
            struct [< $EnumName _Layout >] /* = */ (
                $crate::$Int,
            );

            impl $crate::core::convert::From<$crate::$Int>
                for [< $EnumName _Layout >]
            {
                #[inline]
                fn from (it: $crate::$Int)
                  -> Self
                {
                    Self(it)
                }
            }

            unsafe
            impl $crate::layout::CType
                for [< $EnumName _Layout >]
            { $crate::__cfg_headers__! {
                fn c_short_name_fmt (fmt: &'_ mut $crate::core::fmt::Formatter<'_>)
                  -> $crate::core::fmt::Result
                {
                    fmt.write_str($crate::core::stringify!($EnumName))
                }

                fn c_define_self (definer: &'_ mut dyn $crate::headers::Definer)
                  -> $crate::std::io::Result<()>
                {
                    let ref me =
                        <Self as $crate::layout::CType>
                            ::c_short_name().to_string()
                    ;
                    definer.define_once(
                        me,
                        &mut |definer| {
                            <$crate::$Int as $crate::layout::CType>::c_define_self(
                                definer,
                            )?;
                            let out = definer.out();
                            $crate::__output_docs__!(out, "", $(#[$($meta)*])*);
                            $crate::core::writeln!(out,
                                $crate::core::concat!(
                                    "enum {}_t {{",
                                    $(
                                      "    ",
                                        $crate::core::stringify!($EnumName),
                                        "_",
                                        $crate::core::stringify!($Variant),
                                        $( $crate::layout::ReprC! {
                                            @first(
                                                " = {}"
                                            ) $discriminant
                                        },)?
                                        ",",
                                    )*
                                    "}};\n",
                                    "\n",
                                    "typedef {int}_t;",
                                ),
                                me,
                                $($(
                                    $discriminant,
                                )?)*
                                int = <$crate::$Int as $crate::layout::CType>::c_var(
                                    me,
                                ),
                            )
                        },
                    )
                }

                fn c_var_fmt (
                    fmt: &'_ mut $crate::core::fmt::Formatter<'_>,
                    var_name: &'_ str,
                ) -> $crate::core::fmt::Result
                {
                    $crate::core::write!(fmt,
                        "{}_t{sep}{}",
                        <Self as $crate::layout::CType>::c_short_name(),
                        var_name,
                        sep = if var_name.is_empty() { "" } else { " " },
                    )
                }
            } type OPAQUE_KIND = $crate::layout::OpaqueKind::Concrete; }

            $crate::layout::from_CType_impl_ReprC! {
                [< $EnumName _Layout >]
            }

            unsafe
            impl $crate::layout::ReprC
                for $EnumName
            {
                type CLayout = [< $EnumName _Layout >];

                #[inline]
                fn is_valid (&discriminant: &'_ Self::CLayout)
                  -> bool
                {
                    #![allow(nonstandard_style)]
                    $(
                        const $Variant: $crate::$Int = $EnumName::$Variant as _;
                    )+
                    match discriminant.0 {
                        $( | $Variant )+ => true,
                        | _ => false,
                    }
                }
            }

            unsafe
            impl $crate::layout::__HasNiche__
                for $EnumName
            {
                #[inline]
                fn is_niche (it: &'_ <Self as $crate::layout::ReprC>::CLayout)
                  -> bool
                {
                    *it == unsafe { $crate::core::mem::transmute(
                        $crate::core::option::Option::None::<Self>
                    ) }
                }
            }
        }
    );

    // non-field-less repr-c-only enum
    (
        #[repr(C $(, $Int:ident)?)]
        $(#[$meta:meta])*
        $pub:vis
        enum $EnumName:ident {
            $($variants:tt)*
        }
    ) => (
        $crate::core::compile_error! {
            "Non field-less `enum`s are not supported yet."
        }
    );

    /* == Helpers == */

    (@validate_int_repr u8) => ();
    (@validate_int_repr u16) => ();
    (@validate_int_repr u32) => ();
    (@validate_int_repr u64) => ();
    (@validate_int_repr u128) => ();
    (@validate_int_repr i8) => ();
    (@validate_int_repr i16) => ();
    (@validate_int_repr i32) => ();
    (@validate_int_repr i64) => ();
    (@validate_int_repr i128) => ();

    (@deny_C C) => (
        $crate::core::compile_error!($crate::core::concat!(
            "A `#[repr(C)]` field-less `enum` is not supported,",
            " since the integer type of the discriminant is then",
            " platform dependent",
        ));
    );
    (@deny_C c_int) => (
        $crate::core::compile_error!($crate::core::concat!(
            "Type aliases in a `#[repr(...)]` are not supported by Rust.",
        ));
    );
    (@deny_C c_uint) => (
        $crate::core::compile_error!($crate::core::concat!(
            "Type aliases in a `#[repr(...)]` are not supported by Rust.",
        ));
    );

    (@deny_C $otherwise:tt) => ();

    (@first ($($fst:tt)*) $ignored:tt) => ($($fst)*);
}

#[cfg(feature = "headers")]
#[doc(hidden)] #[macro_export]
macro_rules! __output_docs__ {
    (
        $out:expr, $pad:expr,
    ) => (
        // Nothing
    );

    (
        $out:expr, $pad:expr,
            #[doc = $doc:expr]
            $(#[$($meta:tt)*])*
    ) => ({
        $crate::core::writeln!($out,
            "{pad}/** \\brief\n{pad} * {}", $doc,
            pad = $pad,
        )?;
        $crate::__output_docs__! {
            @opened
            $out, $pad, $(#[$($meta)*])*
        }
    });

    (
        $out:expr, $pad:expr,
            #[$not_doc_meta:meta]
            $(#[$($meta:tt)*])*
    ) => ({
        $crate::__output_docs__! {
            $out, $pad, $(#[$($meta)*])*
        }
    });

    (@opened
        $out:expr, $pad:expr, $(#[doc = $doc:expr])*
    ) => ({
        $(
            $crate::core::writeln!($out,
                "{pad} * {}", $doc,
                pad = $pad,
            )?;
        )*
        $crate::core::writeln!($out,
            "{pad} */",
            pad = $pad,
        )?;
    });

    (@opened
        $out:expr, $pad:expr,
            #[doc = $doc:expr]
            #[$not_doc_meta:meta]
            $(#[$($meta:tt)*])*
    ) => ({
        $crate::core::writeln!($out,
            "{pad} * {}", $doc,
            pad = $pad,
        )?;
        $crate::__output_docs__! {
            @opened
            $out, $pad, $(#[$($meta)*])*
        }
    });

    (@opened
        $out:expr, $pad:expr,
            #[$not_doc_meta:meta]
            $(#[$($meta:tt)*])*
    ) => ({
        $crate::__output_docs__! {
            @opened
            $out, $pad, $(#[$($meta)*])*
        }
    });
}

#[cfg(test)]
crate::layout::ReprC! {
    // #[derive_ReprC]
    #[repr(u8)]
    #[derive(Debug)]
    /// Some docstring
    pub
    enum MyBool {
        False = 42,
        True, // = 43
    }
}

#[cfg(any(test, docs))]
mod test {
    use crate::layout::ReprC;

    ReprC! {
        // #[derive_ReprC]
        #[repr(u8)]
        #[derive(Debug)]
        /// Some docstring
        pub
        enum MyBool {
            False = 42,
            True, // = 43
        }
    }

    ReprC! {
        #[repr(C)]
        struct GenericStruct['lifetime, T]
        where {
            T : 'lifetime,
        }
        {
            inner: &'lifetime T,
        }
    }

    cfg_proc_macros! { #[doc = "```rust"] #[doc = r#"
        use ::repr_c::prelude::*;

        #[derive_ReprC]
        #[repr(u8)]
        #[derive(Debug)]
        /// Some docstring
        pub
        enum MyBool {
            False = 42,
            True, // = 43
        }

        #[derive_ReprC]
        #[repr(C)]
        struct GenericStruct<'lifetime, T : 'lifetime>
        where
            T : ReprC,
        {
            inner: &'lifetime T,
        }

        fn main ()
        {}
    "#] extern {} }
}
