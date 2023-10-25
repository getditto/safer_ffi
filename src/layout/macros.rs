#![cfg_attr(rustfmt, rustfmt::skip)]

export_cfgs! {$
    "headers" => __cfg_headers__!,
    "headers" => __cfg_csharp__!,
    "js" => __cfg_js__!,
    "python-headers" => __cfg_python__!,
}
/// Generates convenience `__cfg_xxx__! { … }` macros with the semantics of
/// `#[cfg(feature = "xxx")] emit! { … }`, but usable by downstream code, such
/// as the one emitted by our own derive macros and whatnot.
macro_rules! export_cfgs {(
    $_:tt
    $(
        $feature:tt => $macro_name:ident !
    ),* $(,)?
) => (
    $(
        match_cfg! {
            feature = $feature => {
                #[doc(hidden)] /** not part of the public API */ #[macro_export]
                macro_rules! $macro_name {(
                    $_($item:item)*
                ) => (
                    $_($item)*
                )}
            },
            _ => {
                #[doc(hidden)] /** not part of the public API */ #[macro_export]
                macro_rules! $macro_name {(
                    $_($item:item)*
                ) => (
                    // nothing
                )}
            },
        }
        #[allow(unused)]
        pub(in crate) use $macro_name;
    )*
)} use export_cfgs;

// Defines a special `__with_cfg_python__!` for-downstream-code, based on the
// usable-everywhere `$($($if_cfg_python)? … )?` conditional expansion trick:
//
// ```rust
// __with_cfg_python__!(|$if_cfg_python| {
//     match … {
//         Language::C => …,
//         $($($if_cfg_python)?
//             Language::Python => …,
//         )?
//     }
// })
// ```
match_cfg! {
    feature = "python-headers" => {
        #[doc(hidden)] /** Not part of the public API */ #[macro_export]
        macro_rules! __with_cfg_python__ {(
            |$_:tt $if_python:ident $(, $__:tt $dol:ident)?| $body:tt
        ) => ({
            macro_rules! __emit__ {
                (
                    [$( $__ $dol : tt )? $_($__rest:tt)*]
                    $_(true $_($_ $if_python:tt)?)?
                    $_(false)?
                ) => $body
            }
            __emit__! { [$] true }
        })}
    },
    _ => {
        #[doc(hidden)] /** Not part of the public API */ #[macro_export]
        macro_rules! __with_cfg_python__ {(
            |$_:tt $if_python:ident $(, $__:tt $dol:ident)?| $body:tt
        ) => ({
            macro_rules! __emit__ {
                (
                    [$( $__ $dol : tt )? $_($__rest:tt)*]
                    $_(true $_($_ $if_python:tt)?)?
                    $_(false)?
                ) => $body
            }
            __emit__! { [$] false }
        })}
    },
}

/// Safely implement [`CType`][`trait@crate::layout::LegacyCType`]
/// for a `#[repr(C)]` struct **when all its fields are `CType`**.
///
/// Note: you rarely need to call this macro directly. Instead, look for the
/// [`ReprC!`] macro to safely implement [`ReprC`][`trait@crate::layout::ReprC`].
#[macro_export]
macro_rules! CType {(
    $(
        @doc_meta( $($doc_meta:tt)* )
    )?
    #[repr(C $(, js $(@$js:tt)?)? $(,)?)]
    $(#[$($meta:tt)*])*
    $pub:vis
    struct $StructName:ident $(
        [
            $($lt:lifetime ,)*
            $($($generics:ident $(= $Default:ty)?),+ $(,)?)?
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
        impl $(<$($lt ,)* $($($generics),+)?>)?
            $crate::js::ReprNapi
        for
            $StructName $(<$($lt ,)* $($($generics),+)?>)?
        where
            Self : 'static,
            $(
                $field_ty : $crate::layout::ReprC,
                <$field_ty as $crate::layout::ReprC>::CLayout : $crate::js::ReprNapi,
            )*
            $(
                $($(
                    $generics : $crate::layout::ReprC,
                    <$generics as $crate::layout::ReprC>::CLayout : $crate::js::ReprNapi,
                )+)?
                $($($bounds)*)?
            )?
        {
            type NapiValue = $crate::js::JsUnknown;

            fn to_napi_value (
                self: Self,
                env: &'_ $crate::js::Env,
            ) -> $crate::js::Result<$crate::js::JsUnknown>
            {
                let mut _obj = env.create_object()?;
                $(
                    _obj.set_named_property(
                        $crate::ඞ::stringify!($field_name),
                        <
                            <$field_ty as $crate::layout::ReprC>::CLayout
                            as
                            $crate::js::ReprNapi
                        >::to_napi_value(
                            unsafe { $crate::layout::into_raw(self.$field_name) },
                            env,
                        )?,
                    )?;
                )*
                $crate::js::Result::Ok(_obj.into_unknown())
            }

            fn from_napi_value (
                env: &'_ $crate::js::Env,
                obj: $crate::js::JsUnknown,
            ) -> $crate::js::Result<Self>
            {
                use $crate::ඞ::convert::TryFrom as _;
                let mut is_buffer = false;
                // Poor man's specialization.
                if  $crate::ඞ::any::TypeId::of::<Self>()
                    ==
                    $crate::ඞ::any::TypeId::of::<$crate::slice::slice_ref_Layout<'_, u8>>()
                &&  (
                        { is_buffer = obj.is_buffer()?; is_buffer }
                        ||
                        $crate::ඞ::matches!(
                            obj.get_type(),
                            $crate::js::Result::Ok($crate::js::ValueType::Null)
                        )
                    )
                {
                    return if is_buffer {
                        let js_buffer = $crate::js::JsBuffer::try_from(obj)?;
                        let (buf, _storage): (&[u8], _);
                        #[cfg(target_arch = "wasm32")] {
                            _storage = ();
                            let bytes = js_buffer.into_value()?.into_boxed_slice();
                            let raw = $crate::ඞ::boxed::Box::into_raw(bytes);
                            env.__push_drop_glue($crate::ඞ::scopeguard::guard(raw, |raw| unsafe {
                                $crate::ඞ::mem::drop($crate::ඞ::boxed::Box::from_raw(raw))
                            }));
                            buf = unsafe { &*raw };
                        } /* else */
                        #[cfg(not(target_arch = "wasm32"))] {
                            _storage = js_buffer.into_value()?;
                            buf = &_storage;
                        }
                        let xs = buf;
                        $crate::js::Result::Ok(unsafe { $crate::ඞ::mem::transmute_copy(&{
                            $crate::slice::slice_raw_Layout::<u8> {
                                ptr: xs.as_ptr() as _,
                                len: xs.len(),
                            }
                        })})
                    } else { // it's NULL
                        $crate::js::Result::Ok(unsafe { $crate::ඞ::mem::transmute_copy::<_, Self>(&{
                            $crate::slice::slice_raw_Layout::<u8> {
                                ptr: $crate::NULL!(),
                                len: 0xbad000,
                            }
                        })})
                    };
                }
                let obj = $crate::js::JsObject::try_from(obj)?;
                $crate::js::Result::Ok(Self {
                    $(
                        $field_name: unsafe { $crate::layout::from_raw_unchecked(
                            <
                                <$field_ty as $crate::layout::ReprC>::CLayout
                                as
                                $crate::js::ReprNapi
                            >::from_napi_value(
                                env,
                                obj.get_named_property($crate::ඞ::stringify!($field_name))?,
                            )?
                        )},
                    )*
                })
            }
        }
); (
    @js_enum
    $Enum_Layout:ident {
        $(
            $Variant:ident = $Discriminant:expr
        ),* $(,)?
    }
) => (
    #[allow(nonstandard_style)]
    const _: () = {
        impl $Enum_Layout {
            $(
                pub const $Variant: $Enum_Layout = $Discriminant;
            )*
        }

        impl $crate::js::ReprNapi for $Enum_Layout {
            type NapiValue = $crate::js::JsString;

            fn to_napi_value (
                self: Self,
                env: &'_ $crate::js::Env,
            ) -> $crate::js::Result< $crate::js::JsString >
            {
                env.create_string(match self {
                $(
                    | $Enum_Layout::$Variant => $crate::ඞ::stringify!($Variant),
                )*
                    | _ => $crate::ඞ::panic!(
                        "ill-formed enum variant ({:?}) for type `{}`",
                        &self.discriminant,
                        <$Enum_Layout as $crate::layout::CType>::short_name(),
                    ),
                })
            }

            fn from_napi_value (
                env: &'_ $crate::js::Env,
                js_string: $crate::js::JsString,
            ) -> $crate::js::Result<Self>
            {
                match js_string.into_utf8()?.as_str()? {
                $(
                    | $crate::ඞ::stringify!($Variant) => $crate::js::Result::Ok($Enum_Layout::$Variant),
                )*
                    | _ => $crate::js::Result::Err($crate::js::Error::new(
                        // status
                        $crate::js::Status::InvalidArg,
                        // reason
                        $crate::ඞ::concat!(
                            "Expected one of: "
                            $(
                                , "`", $crate::ඞ::stringify!($Variant), "`",
                            )", "*
                        ).into(),
                    ).into()),
                }
            }
        }
    };
)}

/// Transitioning helper macro: still uses the old `ReprC!` syntax, but just to
/// forward to the new `#[derive_ReprC2($(js)?)]` one.
#[macro_export]
macro_rules! ReprC {(
    $(
        @[doc = $doc:expr]
    )?
    $(
        #[doc = $doc2:expr]
    )*
    #[repr(
        $C_or_transparent:ident $(,
            $($(@$if_js:tt)?
        js $(,)?
            )?
        )?
    )]
    $(
        #[$attr:meta]
    )*
    $pub:vis
    struct $StructName:ident $([$($generics:tt)*])?
    $(
        where { $($wc:tt)* }
    )?
    $({
        $($body:tt)*
    })?
    $((
        $($body2:tt)*
    );)?
) => (
    #[$crate::prelude::derive_ReprC2($($($($if_js)? js)?)?)]
    $(
        #[doc = $doc]
    )?
    $(
        #[doc = $doc2]
    )*
    #[repr($C_or_transparent)]
    $(
        #[$attr]
    )*
    $pub
    struct $StructName $(<$($generics)*>)?
    $(
        where $($wc)*
    )?
    $({
        $($body)*
    })?
    $((
        $($body2)*
    );)?
)}

#[cfg(test)]
#[crate::derive_ReprC]
#[repr(u8)]
#[derive(Debug)]
/// Some docstring
pub
enum MyBool {
    /// Some variant docstring
    False = 42,
    True, // = 43
}

#[cfg(any(test, docs))]
mod test {
    use crate::layout::ReprC;

    #[crate::derive_ReprC]
    /// Some docstring before
    #[repr(u8)]
    #[derive(Debug)]
    /// Some docstring after
    pub
    enum MyBool {
        False = 42,
        True, // = 43
    }

    ReprC! {
        #[repr(opaque)]
        struct Opaque
        {}
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

    doc_test! { derive_ReprC_supports_generics:
        fn main () {}

        use ::safer_ffi::prelude::*;

        /// Some docstring before
        #[derive_ReprC]
        #[repr(u8)]
        #[derive(Debug)]
        /// Some docstring after
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
    }

    mod opaque {
        doc_test! { unused:
            fn main () {}

            use ::safer_ffi::prelude::*;

            ReprC! {
                #[repr(opaque)]
                struct Foo {}
            }
        }

        doc_test! { with_indirection:
            fn main () {}

            use ::safer_ffi::prelude::*;

            ReprC! {
                #[repr(opaque)]
                pub
                struct Foo {}
            }

            #[ffi_export]
            fn foo (_it: &'_ Foo)
            {}
        }

        doc_test! { without_indirection:
            #![compile_fail]
            fn main () {}

            use ::safer_ffi::prelude::*;

            ReprC! {
                #[repr(opaque)]
                pub
                struct Foo {}
            }

            #[ffi_export]
            fn foo (it: Foo)
            {}
        }
    }
}
