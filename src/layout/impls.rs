use super::*;
use crate::ptr::NonNullPtrCLayout;

__cfg_headers__! {
    use crate::headers::languages::{
        CSharpMarshaler,
        FunctionArg,
        MetadataTypeData,
    };
}

#[cfg(not(any(target_arch = "wasm32", not(feature = "std"))))] // no libc on WASM nor no_std
const_assert! {
    ::core::mem::size_of::<crate::libc::uintptr_t>()
    ==
    ::core::mem::size_of::<crate::libc::size_t>()
}

const _: () = {
    macro_rules! impl_CTypes {
    () => (
        impl_CTypes! { @pointers }
        impl_CTypes! { @zsts }
        impl_CTypes! { @floats
            unsafe
            f32 => "float",

            unsafe
            f64 => "double",
        }
        impl_CTypes! { @integers
            // C# safety: equivalence based onhttps://docs.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/built-in-types

            unsafe // Safety: trivial integer equivalence.
            u8 => "uint8" "byte",

            unsafe // Safety: trivial integer equivalence.
            u16 => "uint16" "UInt16",

            unsafe // Safety: trivial integer equivalence.
            u32 => "uint32" "UInt32",

            unsafe // Safety: trivial integer equivalence.
            u64 => "uint64" "UInt64",

            // unsafe u128 => "uint128",

            unsafe // Safety: Contrary to what most people think,
                   // `usize` is not a `size_t` but an `uintptr_t`,
                   // since it has a guaranteed non-`unsafe` transmute (`as`)
                   // with pointers.
                   //
                   // That being said, many places where Rust uses `usize`
                   // C would expect a `size_t` instead, so there is definitely
                   // a confusion going on with Rust in that regard.
                   //
                   // In practice, it looks like Rust will therefore never
                   // support a platform where `size_t != uintptr_t`.
                   //
                   // Given that, and given how `size_t` for, for instance,
                   // slice lengths, feels far more natural and idiomatic,
                   // this crate makes the opinionated choice not to support
                   // such a platform, so as to use `size_t` instead.
                   //
                   // To ensure soundness in case Rust were to support such as
                   // platform, a compile-time assertion is added, that
                   // ensure the crate will not compile on such platforms.
                   // (search for `size_of` in this file).
            usize => "size" "UIntPtr",


            unsafe // Safety: trivial integer equivalence.
            i8 => "int8" "sbyte",

            unsafe // Safety: trivial integer equivalence.
            i16 => "int16" "Int16",

            unsafe // Safety: trivial integer equivalence.
            i32 => "int32" "Int32",

            unsafe // Safety: trivial integer equivalence.
            i64 => "int64" "Int64",

            // unsafe i128 => "int128",

            unsafe // Safety: See `usize`'s
            isize => "ssize" "IntPtr",
        }
        #[cfg(docs)] impl_CTypes! { @fns (A1, A2) } #[cfg(not(docs))]
        impl_CTypes! { @fns
            (A10, A9, A8, A7, A6, A5, A4, A3, A2, A1)
        }
    );


    (@fns
        (
            $(
                $An:ident $(,
                $Ai:ident)* $(,)?
            )?
        )
    ) => (
        // recurse
        $(
            impl_CTypes! {
                @fns
                ($($Ai ,)*)
            }
        )?

        unsafe
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> $crate::layout::__HasNiche__
        for
            unsafe extern "C" fn ($($An, $($Ai ,)*)?) -> Ret
        {}

        unsafe
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> $crate::layout::__HasNiche__
        for
            /*unsafe*/ extern "C" fn ($($An, $($Ai ,)*)?) -> Ret
        {}

        // LegacyCType
        /// Simplified for lighter documentation, but the actual impls include
        /// **up to 9 function parameters**.
        unsafe // Safety: this is the "blessed" type recommended across Rust
               // literature. Still the alignment of function pointers is not
               // as well-defined, as one would wish.
        impl<
            Ret : CType, $(
            $An : CType, $(
            $Ai : CType,
        )*)?> CType
            for Option<unsafe extern "C" fn ($($An, $($Ai ,)*)?) -> Ret>
        {
            type OPAQUE_KIND = OpaqueKind::Concrete;
            __cfg_headers__! {
                fn short_name() -> String {
                    // ret_arg1_arg2_fptr
                    F(|out| {
                        write!(out, "{}", Ret::short_name())?; $(
                        write!(out, "_{}", $An::short_name())?; $(
                        write!(out, "_{}", $Ai::short_name())?; )*)?
                        write!(out, "_fptr")?;
                        Ok(())
                    })
                    .to_string()
                }

                fn define_self__impl (
                    language: &'_ dyn HeaderLanguage,
                    definer: &'_ mut dyn Definer,
                ) -> io::Result<()>
                {
                    Ret::define_self(language, definer)?; $(
                    $An::define_self(language, definer)?; $(
                    $Ai::define_self(language, definer)?; )*)?
                    language.define_function_ptr_ty(
                        language,
                        definer,
                        &PhantomData::<Self>,
                        &[$(
                            FunctionArg {
                                // FIXME: should this be `_{i}`?
                                name: "",
                                ty: &PhantomData::<$An>,
                            }, $(
                            FunctionArg {
                                // FIXME: should this be `_{i}`?
                                name: "",
                                ty: &PhantomData::<$Ai>,
                            }, )*)?
                        ],
                        &PhantomData::<Ret>,
                    )?;
                    Ok(())
                }

                fn render(
                    out: &mut dyn io::Write,
                    language: &dyn HeaderLanguage,
                ) -> io::Result<()>
                {
                    Self::render_wrapping_var(out, language, None)
                }

                fn render_wrapping_var(
                    out: &'_ mut dyn io::Write,
                    language: &'_ dyn HeaderLanguage,
                    var_name: Option<&dyn ::core::fmt::Display>,
                ) -> io::Result<()>
                {
                    language.emit_function_ptr_ty(
                        language,
                        out,
                        &(Self::short_name() + "_t"),
                        var_name,
                        &[$(
                            FunctionArg {
                                // FIXME: should this be `_{i}`?
                                name: "",
                                ty: &PhantomData::<$An>,
                            }, $(
                            FunctionArg {
                                // FIXME: should this be `_{i}`?
                                name: "",
                                ty: &PhantomData::<$Ai>,
                            }, )*)?
                        ],
                        &PhantomData::<Ret>,
                    )?;
                    Ok(())
                }

                fn metadata() -> &'static dyn Provider {
                    &provide_with(|request| {
                        request.give_if_requested::<CSharpMarshaler>(|| {
                            // This assumes the calling convention from the above
                            // `UnmanagedFunctionPointer` attribute.
                            CSharpMarshaler("UnmanagedType.FunctionPtr".into())
                        });
                        request.give_if_requested::<MetadataTypeData>(|| {
                            let return_type = metadata_nested_type_usage::<Ret>();

                            #[allow(unused_mut)]
                            let mut value_parameters = String::new();

                            $(
                                let n_type = metadata_n_nested_type_usage::<$An>(2);
                                value_parameters.push_str("\n    {\n");
                                value_parameters.push_str(&n_type);

                                $(
                                    let i_type = metadata_n_nested_type_usage::<$Ai>(2);
                                    value_parameters.push_str("\n    },\n    {\n");
                                    value_parameters.push_str(&i_type);
                                )*

                                value_parameters.push_str("\n    }\n");
                            )?

                            MetadataTypeData(format!(
                                "\"kind\": \"{}\",\n\"valueParameters\": [{}],\n\"returnType\": {{\n{}\n}}",
                                "Function",
                                value_parameters,
                                return_type,
                            ))
                        });
                    })
                }
            }
        }

        /* == ReprC for Option-less == */

        /// Simplified for lighter documentation, but the actual impls include
        /// **up to 9 function parameters**.
        unsafe // Safety: byte-wise the layout is the same, but the safety
               // invariants will still have to be checked at each site.
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> ReprC
            for unsafe extern "C" fn ($($An, $($Ai ,)*)?) -> Ret
        {
            type CLayout = Option<
                unsafe extern "C"
                fn ($($An::CLayout, $($Ai::CLayout ,)*)?) -> Ret::CLayout
            >;

            #[inline]
            fn is_valid (c_layout: &'_ Self::CLayout)
              -> bool
            {
                c_layout.is_some()
            }
        }

        /// Simplified for lighter documentation, but the actual impls include
        /// **up to 9 function parameters**.
        unsafe // Safety: byte-wise the layout is the same, but the safety
               // invariants will still have to be checked at each site.
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> ReprC
            for /*unsafe*/ extern "C" fn ($($An, $($Ai ,)*)?) -> Ret
        {
            type CLayout = Option<
                unsafe extern "C"
                fn ($($An::CLayout, $($Ai::CLayout ,)*)?) -> Ret::CLayout
            >;

            #[inline]
            fn is_valid (c_layout: &'_ Self::CLayout)
              -> bool
            {
                c_layout.is_some()
            }
        }

        // Improve the error message when encountering a non-`extern "C"` fn
        // wrapped in an `Option` (otherwise `rustc` tunnelvisions _w.r.t_
        // the lack of Niche).
        unsafe // Safety: `Self : ReprC` is not met so this impl never happens
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> crate::layout::__HasNiche__
            for /*unsafe*/ /*extern "C"*/ fn ($($An, $($Ai ,)*)?) -> Ret
        where
            Self : ReprC, // bound not met
        {
            #[inline]
            fn is_niche (_: &'_ Self::CLayout)
              -> bool
            {
                unreachable!()
            }
        }
        unsafe // Safety: `Self : ReprC` is not met so this impl never happens
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> crate::layout::__HasNiche__
            for unsafe /*extern "C"*/ fn ($($An, $($Ai ,)*)?) -> Ret
        where
            Self : ReprC, // bound not met
        {
            #[inline]
            fn is_niche (_: &'_ Self::CLayout)
              -> bool
            {
                unreachable!()
            }
        }
    );

    (@integers
        $(
            $unsafe:tt
            $RustInt:ident => $CInt:literal $CSharpInt:literal,
        )*
    ) => ($(
        $unsafe // Safety: guaranteed by the caller of the macro
        impl CType
            for $RustInt
        {
            type OPAQUE_KIND = OpaqueKind::Concrete;
            __cfg_headers__! {
                fn short_name () -> String {
                    $CInt.into()
                }

                fn define_self__impl (
                    _language: &'_ dyn HeaderLanguage,
                    _definer: &'_ mut dyn Definer,
                ) -> io::Result<()>
                {
                    unimplemented!("directly did `define_self`")
                }

                fn define_self (
                    language: &'_ dyn HeaderLanguage,
                    definer: &'_ mut dyn Definer,
                ) -> io::Result<()>
                {
                    language.define_primitive_ty(
                        language,
                        definer,
                        primitives::Primitive::Integer {
                            signed: { #[allow(unused_comparisons)] {
                                Self::MIN < 0
                            }},
                            bitwidth: primitives::IntBitWidth::Fixed(
                                primitives::FixedIntBitWidth::from_raw(
                                    Self::BITS as _,
                                ).unwrap()
                            ),
                        }
                    )
                }

                fn metadata() -> &'static dyn Provider {
                    &provide_with(|request| {
                        request
                            .give_if_requested::<MetadataTypeData>(|| {
                                MetadataTypeData(format!(r#""kind": "{}""#, stringify!($RustInt)))
                            })
                        ;
                    })
                }

                fn render(
                    out: &'_ mut dyn io::Write,
                    language: &'_ dyn HeaderLanguage,
                ) -> io::Result<()>
                {
                    language.emit_primitive_ty(
                        out,
                        primitives::Primitive::Integer {
                            signed: { #[allow(unused_comparisons)] {
                                Self::MIN < 0
                            }},
                            bitwidth: match stringify!($RustInt) {
                                | "isize" | "usize" => primitives::IntBitWidth::PointerSized,
                                _ => primitives::IntBitWidth::Fixed(
                                    primitives::FixedIntBitWidth::from_raw(
                                        Self::BITS as _,
                                    ).unwrap()
                                ),
                            },
                        },
                    )?;
                    Ok(())
                }
            }
        }
        from_CType_impl_ReprC! { $RustInt }
    )*);

    (@floats
        $(
            $unsafe:tt
            $fN:ident => $Cty:literal,
        )*
    ) => ($(
        $unsafe // Safety: guaranteed by the caller of the macro
        impl CType
            for $fN
        {
            type OPAQUE_KIND = OpaqueKind::Concrete;
            __cfg_headers__! {
                fn short_name () -> String {
                    $Cty.into()
                }

                fn define_self__impl (
                    _language: &'_ dyn HeaderLanguage,
                    _definer: &'_ mut dyn Definer,
                ) -> io::Result<()>
                {
                    Ok(())
                }

                fn metadata() -> &'static dyn Provider {
                    &provide_with(|request| {
                        request.give_if_requested::<MetadataTypeData>(|| {
                            MetadataTypeData(format!(r#""kind": "{}""#, stringify!($fN)))
                        });
                    })
                }

                fn render(
                    out: &'_ mut dyn io::Write,
                    language: &'_ dyn HeaderLanguage,
                ) -> io::Result<()>
                {
                    language.emit_primitive_ty(
                        out,
                        primitives::Primitive::Float {
                            bitwidth: match size_of::<Self>() {
                                4 => primitives::FloatBitWidth::_32,
                                8 => primitives::FloatBitWidth::_64,
                                _ => unreachable!(),
                            },
                        },
                    )?;
                    Ok(())
                }
            }
        }
        from_CType_impl_ReprC! { $fN }
    )*);

    (
        @pointers
    ) => (
        unsafe
        impl<T : CType> CType
            for *const T
        {
            type OPAQUE_KIND = OpaqueKind::Concrete;

            __cfg_headers__! {
                fn short_name () -> String {
                    format!("{}_const_ptr", T::short_name())
                }

                fn define_self__impl (
                    language: &'_ dyn HeaderLanguage,
                    definer: &'_ mut dyn Definer,
                ) -> io::Result<()>
                {
                    T::define_self(language, definer)
                }

                fn metadata() -> &'static dyn Provider {
                    &provide_with(|request| {
                        request.give_if_requested::<MetadataTypeData>(|| {
                            let nested_type = metadata_nested_type_usage::<T>();

                            MetadataTypeData(format!(
                                "\"kind\": \"{}\",\n\"isMutable\": {},\n\"type\": {{\n{}\n}}",
                                "Pointer",
                                "false",
                                nested_type,
                            ))
                        });
                    })
                }

                fn render(
                    out: &'_ mut dyn io::Write,
                    language: &'_ dyn HeaderLanguage,
                ) -> io::Result<()>
                {
                    const IMMUTABLE: bool = true;
                    language.emit_pointer_ty(
                        language,
                        out,
                        IMMUTABLE,
                        &PhantomData::<T>,
                    )?;
                    Ok(())
                }
            }
        }

        unsafe
        impl<T : ReprC> ReprC
        for *const T
        {
            type CLayout = *const T::CLayout;

            #[inline]
            fn is_valid (_: &'_ Self::CLayout)
              -> bool
            {
                true
            }
        }

        unsafe
        impl<T : CType> CType
            for *mut T
        {
            type OPAQUE_KIND = OpaqueKind::Concrete;

            __cfg_headers__! {
                fn short_name () -> String {
                    format!("{}_ptr", T::short_name())
                }

                fn define_self__impl (
                    language: &'_ dyn HeaderLanguage,
                    definer: &'_ mut dyn Definer,
                ) -> io::Result<()>
                {
                    T::define_self(language, definer)
                }

                fn metadata() -> &'static dyn Provider {
                    &provide_with(|request| {
                        request.give_if_requested::<MetadataTypeData>(|| {
                            let nested_type = metadata_nested_type_usage::<T>();

                            MetadataTypeData(format!(
                                "\"kind\": \"{}\",\n\"isMutable\": {},\n\"type\": {{\n{}\n}}",
                                "Pointer",
                                "true",
                                nested_type,
                            ))
                        });
                    })
                }

                fn render(
                    out: &'_ mut dyn io::Write,
                    language: &'_ dyn HeaderLanguage,
                ) -> io::Result<()>
                {
                    const IMMUTABLE: bool = false;
                    language.emit_pointer_ty(
                        language,
                        out,
                        IMMUTABLE,
                        &PhantomData::<T>,
                    )?;
                    Ok(())
                }
            }
        }

        unsafe
        impl<T : ReprC> ReprC
            for *mut T
        {
            type CLayout = *mut T::CLayout;

            #[inline]
            fn is_valid (_: &'_ Self::CLayout)
              -> bool
            {
                true
            }
        }
    );

    (
        @zsts
    ) => (
        // needed for compatibility with functions returning `()`
        // FIXME: Use special impls in `@fns` for `-> ()` instead.
        unsafe
        impl ReprC
            for ()
        {
            type CLayout = CVoid;

            #[inline]
            fn is_valid (_: &'_ CVoid)
              -> bool
            {
                panic!("It is a logic error to try and get a ZST from C");
            }
        }
        // Needed for structs containing a `PhantomData` field.
        unsafe
        impl<T : ?Sized> ReprC
            for PhantomData<T>
        {
            type CLayout = CVoid;

            #[inline]
            fn is_valid (_: &'_ CVoid)
              -> bool
            {
                panic!("It is a logic error to try and get a ZST from C");
            }
        }
    );
}
    impl_CTypes! {}
};

#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! impl_ReprC_for {(
    $unsafe:tt {
        $(
            $(@for [$($generics:tt)+])? $T:ty
                => |ref $it:tt : $Layout:ty| $expr:expr
        ),* $(,)?
    }
) => (
    $(
        $unsafe
        impl $(<$($generics)+>)? ReprC
            for $T
        {
            type CLayout = $Layout;

            #[inline]
            fn is_valid (it: &'_ $Layout)
              -> bool
            {
                let $it = it;
                if $expr {
                    true
                } else {
                    #[cfg(feature = "log")]
                    ::log::error!(
                        "{:#x?} is not a _valid_ bit pattern for the type `{}`",
                        unsafe {
                            ::core::slice::from_raw_parts(
                                <*const _>::cast::<u8>(it),
                                ::core::mem::size_of_val(it),
                            )
                        },
                        ::core::any::type_name::<Self>(),
                    );
                    false
                }
            }
        }
    )*
)}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(missing_debug_implementations)]
pub struct Bool(u8);

#[cfg(feature = "js")]
const _: () = {
    use crate::js::*;

    impl ReprNapi for Bool {
        type NapiValue = JsBoolean;

        fn to_napi_value(
            self: Self,
            env: &'_ Env,
        ) -> Result<JsBoolean> {
            env.get_boolean(match self.0 {
                | 0 => false,
                | 1 => true,
                | bad => unreachable!("({:#x}: Bool) != 0x0, 0x1", bad),
            })
        }

        fn from_napi_value(
            _env: &'_ Env,
            napi_value: JsBoolean,
        ) -> Result<Self> {
            napi_value.get_value().map(|b: bool| Self(b as _))
        }
    }
};

unsafe impl CType for Bool {
    type OPAQUE_KIND = OpaqueKind::Concrete;
    __cfg_headers__! {
        fn short_name() -> String {
            "bool".into()
        }
        fn define_self__impl(
            _language: &'_ dyn HeaderLanguage,
            _definer: &'_ mut dyn Definer,
        ) -> io::Result<()>
        {
            unimplemented!("directly did `define_self()`");
        }

        fn define_self(
            language: &'_ dyn HeaderLanguage,
            definer: &'_ mut dyn Definer,
        ) -> io::Result<()>
        {
            language.define_primitive_ty(
                language,
                definer,
                primitives::Primitive::Bool,
            )
        }

        fn render(
            out: &'_ mut dyn io::Write,
            language: &'_ dyn HeaderLanguage,
        ) -> io::Result<()>
        {
            language.emit_primitive_ty(
                out,
                primitives::Primitive::Bool,
            )?;
            Ok(())
        }

        fn metadata() -> &'static dyn Provider {
            &provide_with(|request| {
                request
                    .give_if_requested::<CSharpMarshaler>(|| {
                        CSharpMarshaler("UnmanagedType.U1")
                    })
                    .give_if_requested::<MetadataTypeData>(|| {
                        MetadataTypeData(format!("\"kind\": \"{}\"", "bool"))
                    })
                ;
            })
        }
    }
}
from_CType_impl_ReprC! { Bool }

/// A `ReprC` _standalone_ type with the same layout and ABI as
/// [`::libc::c_int`][crate::libc::c_int].
///
/// By _standalone_, the idea is that this is defined as a (`transparent`) _newtype_ `struct`,
/// rather than as a _`type` alias_, which is error-prone and yields less-portable headers (since
/// the header generation will resolve the type alias and emit, for instance, `int32_t`, ⚠️).
///
/// By using this type, you guarantee that the C `int` type be used in the headers.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct c_int(pub crate::libc::c_int);

impl ::core::fmt::Debug for c_int {
    fn fmt(
        self: &'_ c_int,
        fmt: &'_ mut ::core::fmt::Formatter<'_>,
    ) -> ::core::fmt::Result {
        ::core::fmt::Debug::fmt(&self.0, fmt)
    }
}

unsafe impl CType for c_int {
    type OPAQUE_KIND = OpaqueKind::Concrete;
    __cfg_headers__! {
        fn short_name() -> String {
            "int".into()
        }

        fn define_self__impl (
            _language: &'_ dyn HeaderLanguage,
            _definer: &'_ mut dyn Definer,
        ) -> io::Result<()>
        {
            // TODO.
            Ok(())
        }

        fn render(
            out: &'_ mut dyn io::Write,
            language: &'_ dyn HeaderLanguage,
        ) -> io::Result<()>
        {
            language.emit_primitive_ty(
                out,
                primitives::Primitive::Integer {
                    signed: true,
                    bitwidth: primitives::IntBitWidth::CInt,
                },
            )?;
            Ok(())
        }

        fn metadata() -> &'static dyn Provider {
            &provide_with(|request| {
                request
                    .give_if_requested::<CSharpMarshaler>(|| {
                        CSharpMarshaler("UnmanagedType.SysInt")
                    })
                    .give_if_requested::<MetadataTypeData>(|| {
                        MetadataTypeData(format!("\"kind\": \"{}\"", "int"))
                    })
                ;
            })
        }
    }
}

impl<T: CType> From<*const T> for NonNullPtrCLayout<*const T> {
    #[inline]
    fn from(ptr: *const T) -> Self {
        debug_assert!(ptr.is_null().not());
        NonNullPtrCLayout(ptr)
    }
}

impl<T: CType> From<*mut T> for NonNullPtrCLayout<*mut T> {
    #[inline]
    fn from(ptr: *mut T) -> Self {
        debug_assert!(ptr.is_null().not());
        NonNullPtrCLayout(ptr)
    }
}

#[cfg(feature = "js")]
const _: () = {
    use crate::js::*;

    impl<Ptr: CType + ReprNapi> ReprNapi for NonNullPtrCLayout<Ptr> {
        type NapiValue = Ptr::NapiValue;

        fn to_napi_value(
            self: Self,
            env: &'_ Env,
        ) -> Result<Self::NapiValue> {
            Ptr::to_napi_value(self.0, env)
        }

        fn from_napi_value(
            env: &'_ Env,
            napi_value: Self::NapiValue,
        ) -> Result<Self> {
            Ptr::from_napi_value(env, napi_value).map(NonNullPtrCLayout)
        }
    }
};

unsafe impl<Ptr: CType> CType for NonNullPtrCLayout<Ptr> {
    type OPAQUE_KIND = Ptr::OPAQUE_KIND;

    __cfg_headers__! {
        // THE MAIN POINT OF THIS WHOLE WRAPPER:
        fn metadata() -> &'static dyn Provider {
            &provide_with(|request| {
                // Note that this shadows / trumps whatever `T::metadata()` would have provided for
                // `MetadataTypeData`.
                request.give_if_requested::<MetadataTypeData>(|| {
                    let nested_type = metadata_nested_type_usage::<Ptr>();

                    MetadataTypeData(format!("\"kind\": \"{}\",\n\"type\": {{\n{}\n}}", "NonNull", nested_type))
                });
                Ptr::metadata().provide_to(request);
            })
        }

        // -- From here on, we just delegate.

        fn short_name() -> String {
            Ptr::short_name()
        }

        fn render(
            out: &'_ mut dyn io::Write,
            language: &'_ dyn HeaderLanguage,
        ) -> io::Result<()> {
            Ptr::render(out, language)
        }

        fn render_wrapping_var(
            out: &'_ mut dyn io::Write,
            language: &'_ dyn HeaderLanguage,
            // Either a `&&str`, or a `&fmt::Arguments<'_>`, for instance.
            var_name: Option<&dyn ::core::fmt::Display>,
        ) -> io::Result<()> {
            Ptr::render_wrapping_var(out, language, var_name)
        }

        fn define_self__impl(language: &'_ dyn HeaderLanguage, definer: &'_ mut dyn Definer) -> io::Result<()> {
            Ptr::define_self__impl(language, definer)
        }

        fn define_self(language: &'_ dyn HeaderLanguage, definer: &'_ mut dyn Definer) -> io::Result<()> {
            Ptr::define_self(language, definer)
        }

        fn name(language: &'_ dyn HeaderLanguage) -> String {
            Ptr::name(language)
        }

        fn name_wrapping_var(language: &'_ dyn HeaderLanguage, var_name: Option<&dyn fmt::Display>) -> String {
            Ptr::name_wrapping_var(language, var_name)
        }
    }
}

// Interestingly enough, `make -C ffi_tests` ICEs without this…
unsafe impl<T: CType> ReprC for NonNullPtrCLayout<*mut T> {
    type CLayout = *mut T;

    fn is_valid(&ptr: &*mut T) -> bool {
        ptr.is_null().not() && ptr.is_aligned()
    }
}

impl<T: CType> NonNullPtrCLayout<*mut T> {
    #[inline]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    pub fn as_ptr(&self) -> *const T {
        self.0
    }

    pub fn align_offset(
        &self,
        align: usize,
    ) -> usize {
        let addr = self.as_ptr() as usize;
        let misalignment = addr % align;
        if misalignment == 0 {
            0
        } else {
            align - misalignment
        }
    }
}

// Interestingly enough, `make -C ffi_tests` ICEs without this…
unsafe impl<T: CType> ReprC for NonNullPtrCLayout<*const T> {
    type CLayout = *const T;

    fn is_valid(&ptr: &*const T) -> bool {
        ptr.is_null().not() && ptr.is_aligned()
    }
}

impl<T: CType> NonNullPtrCLayout<*const T> {
    #[inline]
    pub fn is_null(self) -> bool {
        self.0.is_null()
    }

    pub fn as_ptr(&self) -> *const T {
        self.0
    }

    pub fn align_offset(
        &self,
        align: usize,
    ) -> usize {
        let addr = self.as_ptr() as usize;
        let misalignment = addr % align;
        if misalignment == 0 {
            0
        } else {
            align - misalignment
        }
    }
}

impl_ReprC_for! { unsafe {
    bool
        => |ref byte: Bool| (byte.0 & !0b1) == 0
    ,

    @for[T : ReprC]
    ptr::NonNull<T>
        => |ref it: NonNullPtrCLayout<*mut T::CLayout>| {
            it.is_null().not() &&
            (it.0 as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for[T : ReprC]
    ptr::NonNullRef<T>
        => |ref it: NonNullPtrCLayout<*const T::CLayout>| {
            it.is_null().not() &&
            (it.0 as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for[T : ReprC]
    ptr::NonNullMut<T>
        => |ref it: NonNullPtrCLayout<*mut T::CLayout>| {
            it.is_null().not() &&
            (it.0 as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for[T : ReprC]
    ptr::NonNullOwned<T>
        => |ref it: NonNullPtrCLayout<*mut T::CLayout>| {
            it.is_null().not() &&
            (it.0 as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for['a, T : 'a + ReprC]
    &'a T
        => |ref it: NonNullPtrCLayout<*const T::CLayout>| {
            it.is_null().not() &&
            (it.0 as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for['a, T : 'a + ReprC]
    &'a mut T
        => |ref it: NonNullPtrCLayout<*mut T::CLayout>| {
            it.is_null().not() &&
            (it.0 as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
}}

/* `HasNiche` from `niche.rs` impls `ReprC` for `Option<ptr>` types. */

impl_ReprC_for! { unsafe {
    @for['out, T : 'out + Sized + ReprC]
    Out<'out, T>
        => |ref it: NonNullPtrCLayout<*mut T::CLayout>| {
            (it.0 as usize) % ::core::mem::align_of::<T>() == 0
        },
}}

pub type OpaqueLayout<T> = OpaqueLayout_<::core::marker::PhantomData<T>>;

#[derive(Debug, Clone, Copy)]
pub struct OpaqueLayout_<Phantom>(Phantom);

from_CType_impl_ReprC!(@for[T] OpaqueLayout<T>);
unsafe impl<T> CType for OpaqueLayout<T> {
    type OPAQUE_KIND = OpaqueKind::Opaque;

    __cfg_headers__! {
        fn short_name ()
          -> String
        {
            let mut it = String::from("Opaque");
            crate::ඞ::append_unqualified_name(&mut it, ::core::any::type_name::<T>());
            it
        }

        fn define_self__impl (
            language: &'_ dyn HeaderLanguage,
            definer: &'_ mut dyn Definer,
        ) -> io::Result<()>
        {
            language.declare_opaque_type(
                language,
                definer,
                &[
                    &format!(
                        "The layout of `{}` is opaque/subject to changes.",
                        ::core::any::type_name::<T>(),
                    ),
                ],
                &PhantomData::<Self>,
            )
        }

        fn metadata() -> &'static dyn Provider {
            &provide_with(|request| {
                request.give_if_requested::<MetadataTypeData>(|| {
                    MetadataTypeData(format!("\"kind\": \"{}\",\n\"name\": \"{}\"", "Opaque", Self::short_name()))
                });
            })
        }
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct Opaque<T> {
    pub concrete: T,
}

impl<T> ::core::ops::Deref for Opaque<T> {
    type Target = T;

    fn deref(self: &'_ Opaque<T>) -> &'_ T {
        &self.concrete
    }
}

impl<T> ::core::ops::DerefMut for Opaque<T> {
    fn deref_mut(self: &'_ mut Opaque<T>) -> &'_ mut T {
        &mut self.concrete
    }
}

#[apply(cfg_alloc)]
impl<T> From<rust::Box<T>> for rust::Box<Opaque<T>> {
    fn from(b: rust::Box<T>) -> rust::Box<Opaque<T>> {
        unsafe { rust::Box::from_raw(rust::Box::into_raw(b).cast()) }
    }
}

#[apply(cfg_alloc)]
impl<T> From<rust::Box<T>> for ThinBox<Opaque<T>> {
    fn from(b: rust::Box<T>) -> repr_c::Box<Opaque<T>> {
        rust::Box::<Opaque<T>>::from(b).into()
    }
}

impl<'r, T> From<&'r T> for &'r Opaque<T> {
    fn from(r: &'r T) -> &'r Opaque<T> {
        unsafe { &*<*const _>::cast::<Opaque<T>>(r) }
    }
}

impl<'r, T> From<&'r mut T> for &'r mut Opaque<T> {
    fn from(r: &'r mut T) -> &'r mut Opaque<T> {
        unsafe { &mut *<*mut _>::cast::<Opaque<T>>(r) }
    }
}

unsafe impl<T> ReprC for Opaque<T> {
    type CLayout = OpaqueLayout<T>;

    fn is_valid(_: &'_ Self::CLayout) -> bool {
        unreachable! {"\
            wondering about the validity of an opaque type \
            makes no sense\
        "};
    }
}

opaque_impls! {
    @for['c] ::core::task::Context<'c>,
    @for['r] &'r str,
    @for['r, T] &'r [T],
    @for['r, T] &'r mut [T],
    @for[T] ::core::cell::RefCell<T>,
}

#[cfg(feature = "alloc")]
opaque_impls! {
    rust::String,
    @for[T] rust::Box<T>,
    @for[T] ::alloc::rc::Rc<T>,
    @for[T] rust::Vec<T>,
    @for[K, V] ::alloc::collections::BTreeMap<K, V>,
}

#[cfg(feature = "std")]
opaque_impls! {
    @for[K, V] ::std::collections::HashMap<K, V>,
    @for[T] ::std::sync::Arc<T>,
    @for[T] ::std::sync::Mutex<T>,
    @for[T] ::std::sync::RwLock<T>,
}

// where
#[cfg_attr(rustfmt, rustfmt::skip)]
macro_rules! opaque_impls {(
    $(
        $(@for[$($generics:tt)*])? $T:ty
    ),* $(,)?
) => (
    $(
        unsafe
        impl<$($($generics)*)?>
            ReprC
        for
            $T
        where
            // …
        {
            type CLayout = OpaqueLayout<$T>;

            fn is_valid (_: &'_ Self::CLayout)
              -> bool
            {
                unreachable! {"\
                    wondering about the validity of an opaque type \
                    makes no sense\
                "};
            }
        }
    )*
)}
use opaque_impls;

/// Arrays of const size `N`
unsafe impl<Item: CType, const N: usize> CType for [Item; N] {
    type OPAQUE_KIND = OpaqueKind::Concrete;
    __cfg_headers__! {
        fn short_name() -> String {
            // item_N_array
            format!("{}_{}_array", Item::short_name(), N)
        }

        fn define_self__impl(
            language: &'_ dyn HeaderLanguage,
            definer: &'_ mut dyn Definer,
        ) -> io::Result<()>
        {
            Item::define_self(language, definer)?;
            language.define_array_ty(
                language,
                definer,
                &PhantomData::<Self>,
                &PhantomData::<Item>,
                N,
            )?;
            Ok(())
        }

        fn render(
            out: &mut dyn io::Write,
            language: &dyn HeaderLanguage,
        ) -> io::Result<()>
        {
            Self::render_wrapping_var(out, language, None)
        }

        fn render_wrapping_var(
            out: &'_ mut dyn io::Write,
            language: &'_ dyn HeaderLanguage,
            var_name: Option<&dyn ::core::fmt::Display>,
        ) -> io::Result<()>
        {
            language.emit_array_ty(
                language,
                out,
                var_name,
                &(Self::short_name() + "_t"),
                &PhantomData::<Item>,
                N,
            )
        }

        fn metadata() -> &'static dyn Provider {
            &provide_with(|request| {
                request.give_if_requested::<MetadataTypeData>(|| {
                    let nested_type = metadata_nested_type_usage::<Item>();

                    MetadataTypeData(format!("\"kind\": \"{}\",\n\"backingTypeName\": \"{}\",\n\"size\": {},\n\"type\": {{\n{}\n}}",
                        "StaticArray",
                        Self::short_name() + "_t",
                        N,
                        nested_type,
                    ))
                });
            })
        }
    }
}

unsafe impl<Item: ReprC, const N: usize> ReprC for [Item; N] {
    type CLayout = [Item::CLayout; N];

    #[inline]
    fn is_valid(it: &'_ Self::CLayout) -> bool {
        it.iter().all(Item::is_valid)
    }
}

__cfg_headers__! {

    pub(super)
    fn metadata_nested_type_usage<Type: CType>() -> String {
        metadata_n_nested_type_usage::<Type>(1)
    }

    pub(super)
    fn metadata_n_nested_type_usage<Type: CType>(nesting: usize) -> String {
        if let Some(MetadataTypeData(nested_type)) = Type::metadata().dyn_request() {
            nested_type
                .lines()
                .map(|line| format!("{}{}", "    ".repeat(nesting), line))
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            <_>::default()
        }
    }
}
