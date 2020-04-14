use super::*;

#[cfg(feature = "headers")]
#[macro_export] #[doc(hidden)]
macro_rules! cfg_headers {(
    $($it:item)*
) => (
    $($it)*
)}
#[cfg(not(feature = "headers"))]
#[macro_export] #[doc(hidden)]
macro_rules! cfg_headers {(
    $($it:item)*
) => (
    // nothing
)}

#[macro_export]
macro_rules! derive_CType {(
    #[repr(C)]
    $(#[$meta:meta])*
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
            $(#[$field_meta:meta])*
            $field_pub:vis
            $field_name:ident : $field_ty:ty
        ),+ $(,)?
    }
) => (
    #[repr(C)]
    $(#[$meta])*
    $pub
    struct $StructName
        $(<$($lt ,)* $($($generics),+)?> $(where $($bounds)* )?)?
    {
        $(
            $(#[$field_meta])*
            $field_pub
            $field_name : $field_ty,
        )*
    }

    unsafe // Safety: struct is `#[repr(C)]` and contains `CType` fields
    impl $(<$($lt ,)* $($($generics),+)?>)? $crate::layout::CType
        for $StructName$(<$($lt ,)* $($($generics),+)?>)?
    where
        $(
            $field_ty : $crate::layout::CType,
        )*
        $(
            $($($bounds)*)?
        )?
    { $crate::cfg_headers! {
        fn with_short_name<R> (
            ret: impl
                $crate::core::ops::FnOnce(&'_ dyn $crate::core::fmt::Display)
                  -> R
            ,
        ) -> R
        {
            ret(&{
                let ret = stringify!($StructName);
                $($(
                    let mut ret = ret.to_string();
                    $(
                        <
                            <$generics as $crate::layout::ReprC>::CLayout
                            as
                            $crate::layout::CType
                        >::with_short_name(|it| {
                            use $crate::core::fmt::Write;
                            $crate::core::write!(ret, "_{}", it)
                                .unwrap()
                        });
                    )+
                )?)?
                ret
            })
        }

        fn c_define_self (definer: &'_ mut dyn $crate::layout::Definer)
          -> $crate::std::io::Result<()>
        {
            let ref me =
                <Self as $crate::layout::CType>
                    ::with_short_name(|it| it.to_string())
            ;
            definer.define(
                me,
                &mut |definer| {
                    $(
                        <$field_ty as $crate::layout::CType>::c_define_self(definer)?;
                    )*
                    let out = definer.out();
                    write!(out, "typedef struct {{\n")?;
                    $(
                        write!(out, "    {};\n",
                            <$field_ty as $crate::layout::CType>::c_display(
                                stringify!($field_name),
                            ),
                        )?;
                    )*
                    $crate::core::write!(out, "}} {}_t;\n\n", me)
                },
            )
        }

        fn c_fmt (
            fmt: &'_ mut $crate::core::fmt::Formatter<'_>,
            var_name: &'_ str,
        ) -> $crate::core::fmt::Result
        {
            <Self as $crate::layout::CType>::with_short_name(|me| {
                write!(fmt,
                    "{}_t{sep}{}",
                    me, var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            })
        }
    }}

    $crate::layout::from_CType_impl_ReprC! {
        $(@for [$($lt ,)* $($($generics),+)?])?
            $StructName
                $(<$($lt ,)* $($($generics),+)?>
                    where
                        $($($bounds)*)?
                )?
    }
)}

#[macro_export]
macro_rules! derive_ReprC {
    // struct
    (
        $( @[doc = $doc:expr] )?
        #[repr(C)]
        $(#[$meta:meta])*
        $pub:vis
        struct $StructName:ident $(
            [$($generics:tt)*] $(
                where { $($bounds:tt)* }
            )?
        )?
        {
            $(
                $(#[$field_meta:meta])*
                $field_pub:vis
                $field_name:ident : $field_ty:ty
            ),+ $(,)?
        }
    ) => (
        #[repr(C)]
        $(#[doc = $doc])?
        $(#[$meta])*
        $pub
        struct $StructName $(
            <$($generics)*> $(
                where $($bounds)*
            )?
        )?
        {
            $(
                $(#[$field_meta])*
                $field_pub
                $field_name : $field_ty,
            )*
        }

        ::paste::item! {
            pub type [< $StructName _Layout >]$(<$($generics)*>)?
                = <$StructName $(<$($generics)*>)? as $crate::layout::ReprC>::CLayout
            ;
        }
        const _: () = {
            type __ReprC_StructName__ $(<$($generics)*>)? =
                $StructName $(<$($generics)*>)?
            ;
            const _: () = {
                unsafe // Safety: struct is `#[repr(C)]` and contains `ReprC` fields
                impl $(<$($generics)*>)? $crate::layout::ReprC
                    for __ReprC_StructName__ $(<$($generics)*>)?
                where
                    $(
                        $field_ty : $crate::layout::ReprC,
                    )*
                    $($(
                        $($bounds)*
                    )?)?
                {
                    type CLayout =
                        $StructName
                            $(<$($generics)*>)?
                    ;

                    #[inline]
                    fn is_valid (it: &'_ Self::CLayout)
                      -> bool
                    {
                        true $(
                            && <$field_ty as $crate::layout::ReprC>::is_valid(
                                &it.$field_name
                            )
                        )*
                    }
                }

                $crate::layout::derive_CType! {
                    #[repr(C)]
                    // $(#[$meta])*
                    pub
                    struct $StructName
                        [$($($generics)*)?]
                    where {
                        $(
                            $field_ty : $crate::layout::ReprC,
                        )*
                        $($(
                            $($bounds)*
                        )?)?
                    } {
                        $(
                            // $(#[$field_meta])*
                            pub
                            $field_name :
                                <$field_ty as $crate::layout::ReprC>::CLayout
                            ,
                        )*
                    }
                }

                impl $(<$($generics)*>)? $crate::core::marker::Copy
                    for $StructName $(<$($generics)*>)?
                where
                    $(
                        $field_ty : $crate::layout::ReprC,
                    )*
                    $($(
                        $($bounds)*
                    )?)?
                {}

                impl $(<$($generics)*>)? $crate::core::clone::Clone
                    for $StructName $(<$($generics)*>)?
                where
                    $(
                        $field_ty : $crate::layout::ReprC,
                    )*
                    $($(
                        $($bounds)*
                    )?)?
                {
                    #[inline]
                    fn clone (self: &'_ Self)
                      -> Self
                    {
                        *self
                    }
                }
            };
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
        #[repr(transparent)]
        $(#[doc = $doc])?
        $(#[$meta])*
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
        $(#[$meta:meta])*
        $pub:vis
        enum $EnumName:ident {
            $(
                $(#[$variant_meta:meta])*
                $Variant:ident $(= $discriminant:expr)?
            ),+ $(,)?
        }
    ) => (
        $crate::layout::derive_ReprC! {
            @validate_int_repr $Int
        }
        $crate::layout::derive_ReprC! {
            @deny_C $Int
        }

        #[repr($Int)]
        $(#[$meta])*
        $pub
        enum $EnumName {
            $(
                $(#[$variant_meta])*
                $Variant $(= $discriminant)? ,
            )+
        }

        ::paste::item! {
            #[repr(transparent)]
            #[derive(Clone, Copy)]
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
            impl $crate::layout::CType for [< $EnumName _Layout >]
            { $crate::cfg_headers! {
                fn with_short_name<R> (
                    ret: impl FnOnce(&'_ dyn $crate::core::fmt::Display) -> R,
                ) -> R
                {
                    ret(&concat!(stringify!($EnumName)))
                }

                fn c_define_self (definer: &'_ mut $crate::layout::Definer)
                  -> $crate::std::io::Result<()>
                {
                    let ref me =
                        <Self as $crate::layout::CType>
                            ::with_short_name(|it| it.to_string())
                    ;
                    definer.define(
                        me,
                        &mut |definer| {
                            <$crate::$Int as $crate::layout::CType>::c_define_self(
                                definer,
                            )?;
                            use $crate::std::io::Write;
                            write!(definer.out(),
                                concat!(
                                    "enum {}_t {{\n",
                                    $(
                                      "    ",
                                        stringify!($EnumName),
                                        "_",
                                        stringify!($Variant),
                                        $( $crate::layout::derive_ReprC! {
                                            @first(
                                                " = {}"
                                            ) $discriminant
                                        },)?
                                        ",\n",
                                    )*
                                    "}};\n",
                                    "\n",
                                    "typedef {int}_t",
                                    ";\n",
                                ),
                                me,
                                $($(
                                    $discriminant,
                                )?)*
                                int = <$crate::$Int as $crate::layout::CType>::c_display(
                                    me,
                                ),
                            )
                        },
                    )
                }

                fn c_fmt (
                    fmt: &'_ mut $crate::core::fmt::Formatter<'_>,
                    var_name: &'_ str,
                ) -> $crate::core::fmt::Result
                {
                    <Self as $crate::layout::CType>::with_short_name(|me| {
                        write!(fmt,
                            "{}_t{sep}{}",
                            me, var_name,
                            sep = if var_name.is_empty() { "" } else { " " },
                        )
                    })
                }
            }}
            $crate::layout::from_CType_impl_ReprC! {
                [< $EnumName _Layout >]
            }

            unsafe
            impl $crate::layout::ReprC for $EnumName {
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
        compile_error! {
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
        compile_error!(concat!(
            "A `#[repr(C)]` field-less `enum` is not supported,",
            " since the integer type of the discriminant is then",
            " platform dependent",
        ));
    );
    (@deny_C c_int) => (
        compile_error!(concat!(
            "Type aliases in a `#[repr(...)]` are not supported by Rust.",
        ));
    );
    (@deny_C c_uint) => (
        compile_error!(concat!(
            "Type aliases in a `#[repr(...)]` are not supported by Rust.",
        ));
    );

    (@deny_C $otherwise:tt) => ();

    (@first ($($fst:tt)*) $ignored:tt) => ($($fst)*);
}

derive_ReprC! {
    #[repr(u8)]
    /// Some docstring
    pub
    enum MyBool {
        False = 42,
        True = 27,
    }
}
