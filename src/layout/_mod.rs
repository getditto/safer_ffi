#![cfg_attr(rustfmt, rustfmt::skip)]
//! Trait abstractions describing the semantics of "being `#[repr(C)]`"

use_prelude!();

__cfg_headers__! {
    use crate::headers::{
        Definer,
        languages::*,
    };
}

pub(in crate)
mod macros;

#[doc(inline)]
pub use crate::{from_CType_impl_ReprC, ReprC, CType};

pub use crate::{
    derive_ReprC,
};

type_level_enum! {
    pub
    enum OpaqueKind {
        Concrete,
        Opaque,
    }
}

/// Safety (non-exhaustive list at the moment):
///   - `::core::mem::zeroed::<Self>()` must be sound to use.
pub
unsafe
trait CType
:
    Sized +
    Copy +
{
    type OPAQUE_KIND : OpaqueKind::T;

    fn zeroed() -> Self {
        unsafe {
            ::core::mem::zeroed()
        }
    }

    __cfg_headers__! {
        fn short_name ()
          -> String
        ;

        #[allow(nonstandard_style)]
        fn define_self__impl (
            language: &'_ dyn HeaderLanguage,
            definer: &'_ mut dyn Definer,
        ) -> io::Result<()>
        ;

        fn define_self (
            language: &'_ dyn HeaderLanguage,
            definer: &'_ mut dyn Definer,
        ) -> io::Result<()>
        {
            definer.define_once(
                &Self::name(language),
                &mut |definer| Self::define_self__impl(language, definer),
            )
        }

        fn name (
            _language: &'_ dyn HeaderLanguage,
        ) -> String
        {
            format!("{}_t", Self::short_name())
        }

        fn name_wrapping_var (
            language: &'_ dyn HeaderLanguage,
            var_name: &'_ str,
        ) -> String
        {
            let sep = if var_name.is_empty() { "" } else { " " };
            format!("{}{sep}{var_name}", Self::name(language))
        }

        /// Optional marshaler attached to the type (_e.g._,
        /// `[MarshalAs(UnmanagedType.FunctionPtr)]`)
        fn csharp_marshaler ()
          -> Option<String>
        {
            None
        }

        fn metadata_type_usage () -> String;
    }
}

unsafe
impl<T : LegacyCType> CType for T {
    type OPAQUE_KIND = <T as LegacyCType>::OPAQUE_KIND;

    __cfg_headers__! {
        #[inline]
        fn short_name ()
          -> String
        {
            <Self as LegacyCType>::c_short_name().to_string()
        }

        #[inline]
        fn define_self__impl (
            _: &'_ dyn HeaderLanguage,
            _: &'_ mut dyn Definer,
        ) -> io::Result<()>
        {
            unimplemented!()
        }

        fn define_self (
            language: &'_ dyn HeaderLanguage,
            definer: &'_ mut dyn Definer,
        ) -> io::Result<()>
        {
            match () {
                | _case if language.is::<C>() => {
                    <Self as LegacyCType>::c_define_self(definer)
                },
                | _case if language.is::<CSharp>() => {
                    <Self as LegacyCType>::csharp_define_self(definer)
                },
                | _case if language.is::<Metadata>() => {
                    <Self as LegacyCType>::metadata_define_self(definer)
                },
                #[cfg(feature = "python-headers")]
                | _case if language.is::<Python>() => {
                    <Self as LegacyCType>::c_define_self(definer)
                },
                | _ => unimplemented!(),
            }
        }

        #[inline]
        fn name (
            language: &'_ dyn HeaderLanguage,
        ) -> String
        {
            Self::name_wrapping_var(language, "")
        }

        #[inline]
        fn name_wrapping_var (
            language: &'_ dyn HeaderLanguage,
            var_name: &'_ str,
        ) -> String
        {
            match () {
                | _case if language.is::<C>() => {
                    <Self as LegacyCType>::c_var(var_name).to_string()
                },
                | _case if language.is::<CSharp>() => {
                    let sep = if var_name.is_empty() { "" } else { " " };
                    format!("{}{sep}{var_name}", Self::csharp_ty())
                },
                | _case if language.is::<Metadata>() => {
                    <Self as LegacyCType>::c_var(var_name).to_string()
                },
                #[cfg(feature = "python-headers")]
                | _case if language.is::<Python>() => {
                    <Self as LegacyCType>::c_var(var_name).to_string()
                },
                | _ => unimplemented!(),
            }
        }

        fn metadata_type_usage() -> String {
            <T as LegacyCType>::metadata_type_usage()
        }

        #[inline]
        fn csharp_marshaler ()
          -> Option<String>
        {
            <T as LegacyCType>::legacy_csharp_marshaler()
        }
    }
}

pub
type CLayoutOf<ImplReprC> = <ImplReprC as ReprC>::CLayout;

/// One of the two core traits of this crate (with [`ReprC`][`trait@ReprC`]).
///
/// `CType` is an `unsafe` trait that binds a Rust type to a C typedef.
///
/// To optimise compile-times, the C typedef part is gated behind the `headers`
/// cargo feature, so when that feature is not enabled, the trait may "look"
/// like a marker trait, but it isn't.
///
/// That's why **manually implementing this trait is strongly discouraged**,
/// although not forbidden:
///
///   - If you trully want a manual implementation of `CType` (_e.g._, for an
///     "opaque type" pattern, _i.e._, a forward declaration), then, to
///     implement the trait so that it works no matter the status of
///     the `safer_ffi/headers` feature, one must define the methods as if
///     feature was present, but with a `#[::safer_ffi::cfg_headers]` gate slapped
///     on _each_ method.
///
/// # Safety
///
/// The Rust type in an `extern "C"` function must have the same layout and ABI
/// as the defined C type, and all the bit-patterns representing any instance
/// of such C type must be valid and safe bit-patterns for the Rust type.
///
/// For the most common types, there are only two reasons to correctly be a
/// `CType`:
///
///   - being a primitive type, such as an integer type or a (slim) pointer.
///
///       - This crates provides as many of these implementations as possible.
///
///   - and recursively, a non-zero-sized `#[repr(C)]` struct of `CType` fields.
///
///       - the [`CType!`] macro can be used to wrap a `#[repr(C)]` struct
///         definition to _safely_ and automagically implement the trait
///         when it is sound to do so.
///
/// Note that types such as Rust's [`bool`] are ruled out by this definition,
/// since it has the ABI of a `u8 <-> uint8_t`, and yet there are many
/// bit-patterns for the `uint8_t` type that do not make _valid_ `bool`s.
///
/// For such types, see the [`ReprC`][`trait@ReprC`] trait.
pub
unsafe trait LegacyCType
:
    Sized +
    Copy +
    CType +
{
    type OPAQUE_KIND : OpaqueKind::T;
    __cfg_headers__! {
        /// A short-name description of the type, mainly used to fill
        /// "placeholders" such as when monomorphising generics structs or
        /// arrays.
        ///
        /// This provides the implementation used by [`LegacyCType::c_short_name`]`()`.
        ///
        /// There are no bad implementations of this method, except,
        /// of course, for the obligation to provide a valid identifier chunk,
        /// _i.e._, the output must only contain alphanumeric digits and
        /// underscores.
        ///
        /// For instance, given `T : CType` and `const N: usize > 0`, the type
        /// `[T; N]` (inline fixed-size array of `N` consecutive elements of
        /// type `T`) will be typedef-named as:
        ///
        /// ```rust,ignore
        /// write!(fmt, "{}_{}_array", <T as CType>::c_short_name(), N)
        /// ```
        ///
        /// Generally, typedefs with a trailing `_t` will see that `_t` trimmed
        /// when used as a `short_name`.
        ///
        /// ## Implementation by [`CType!`]:
        ///
        /// A non generic struct such as:
        ///
        /// ```rust,ignore
        /// CType! {
        ///     #[repr(C)]
        ///     struct Foo { /* fields */ }
        /// }
        /// ```
        ///
        /// will have `Foo` as its `short_name`.
        ///
        /// A generic struct such as:
        ///
        /// ```rust,ignore
        /// CType! {
        ///     #[repr(C)]
        ///     struct Foo[T] where { T : CType } { /* fields */ }
        /// }
        /// ```
        ///
        /// will have `Foo_xxx` as its `short_name`, with `xxx` being `T`'s
        /// `short_name`.
        fn c_short_name_fmt (fmt: &'_ mut fmt::Formatter<'_>)
          -> fmt::Result
        ;
        // {
        //     Self::short_name_fmt(&C, fmt)
        // }

        // fn short_name_fmt (
        //     language: &'_ dyn HeaderLanguage,
        //     fmt: &'_ mut fmt::Formatter<'_>,
        // ) -> fmt::Result
        // {
        //     match () {
        //         | _case if language.is::<C>() => Self::c_short_name_fmt(fmt),
        //         // | _case if language.is::<CSharp>() => Self::csharp_short_name_fmt(fmt),
        //         | _ => unimplemented!(),
        //     }
        // }

        /// Convenience function for _callers_ / users of types implementing
        /// [`CType`][`trait@CType`].
        ///
        /// The `Display` logic is auto-derived from the implementation of
        /// [`LegacyCType::c_short_name_fmt`]`()`.
        #[inline]
        fn c_short_name ()
          -> short_name_impl_display::ImplDisplay<Self>
        {
            short_name_impl_display::ImplDisplay { _phantom: PhantomData }
        }

        /// Necessary one-time code for [`LegacyCType::c_var`]`()` to make sense.
        ///
        /// Some types, such as `char`, are part of the language, and can be
        /// used directly by [`LegacyCType::c_var`]`()`.
        /// In that case, there is nothing else to _define_, and all is fine.
        ///
        ///   - That is the default implementation of this method: doing
        ///     nothing.
        ///
        /// But most often than not, a `typedef` or an `#include` is required.
        ///
        /// In that case, here is the place to put it, with the help of the
        /// provided `Definer`.
        ///
        /// # Idempotent
        ///
        /// Given some `definer: &mut dyn Definer`, **the `c_define_self(definer)`
        /// call must be idempotent _w.r.t._ code generated**. In other words,
        /// two or more such calls must not generate any extra code _w.r.t_ the
        /// first call.
        ///
        /// This is easy to achieve thanks to `definer`:
        ///
        /// ```rust,ignore
        /// // This ensures the idempotency requirements are met.
        /// definer.define_once(
        ///     // some unique `&str`, ideally the C name being defined:
        ///     "my_super_type_t",
        ///     // Actual code generation logic, writing to `definer.out()`
        ///     &mut |definer| {
        ///         // If the typdef recursively needs other types being defined,
        ///         // ensure it is the case by explicitly calling
        ///         // `c_define_self(definer)` on those types.
        ///         OtherType::c_define_self(definer)?;
        ///         write!(definer.out(), "typedef ... my_super_type_t;", ...)
        ///     },
        /// )?
        /// ```
        ///
        /// # Safety
        ///
        /// Given that the defined types may be used by [`LegacyCType::c_var_fmt`]`()`,
        /// the same safety disclaimers apply.
        ///
        /// ## Examples
        ///
        /// #### `i32`
        ///
        /// The corresponding type for `i32` in C is `int32_t`, but such type
        /// definition is not part of the language, it is brought by a library
        /// instead: `<stdint.h>` (or `<inttypes.h>` since it includes it).
        ///
        /// ```rust,ignore
        /// unsafe impl CType for i32 {
        ///     #[::safer_ffi::cfg_headers]
        ///     fn c_define_self (definer: &'_ mut dyn Definer)
        ///       -> io::Result<()>
        ///     {
        ///         definer.define_once("<stdint.h>", &mut |definer| {
        ///             write!(definer.out(), "\n#include <stdint.h>\n")
        ///         })
        ///     }
        ///
        ///     // ...
        /// }
        /// ```
        ///
        /// #### `#[repr(C)] struct Foo { x: i32 }`
        ///
        /// ```rust,ignore
        /// #[repr(C)]
        /// struct Foo {
        ///     x: i32,
        /// }
        ///
        /// unsafe impl CType for i32 {
        ///     #[::safer_ffi::cfg_headers]
        ///     fn c_define_self (definer: &'_ mut dyn Definer)
        ///       -> io::Result<()>
        ///     {
        ///         definer.define_once("Foo_t", &mut |definer| {
        ///             // ensure int32_t makes sense
        ///             <i32 as CType>::c_define_self(definer)?;
        ///             write!(definer.out(),
        ///                 "typedef struct {{ {}; }} Foo_t;",
        ///                 <i32 as CType>::c_var("x"),
        ///             )
        ///         })
        ///     }
        ///
        ///     // ...
        /// }
        /// ```
        fn c_define_self (definer: &'_ mut dyn Definer)
          -> io::Result<()>
        ;
        // {
        //     Self::define_self(&C, definer)
        // }

        // #[inline]
        // fn define_self__impl (
        //     language: &'_ dyn HeaderLanguage,
        //     definer: &'_ mut dyn Definer,
        // ) -> io::Result<()>
        // {
        //     let _ = (language, definer);
        //     Ok(())
        // }

        /// The core method of the trait: it provides the implementation to be
        /// used by [`LegacyCType::c_var`], by bringing a `Formatter` in scope.
        ///
        /// This provides the implementation used by [`LegacyCType::c_var`]`()`.
        ///
        /// The implementations are thus much like any classic `Display` impl,
        /// except that:
        ///
        ///   - it must output valid C code representing the type corresponding
        ///     to the Rust type.
        ///
        ///   - a `var_name` may be supplied, in which case the type must
        ///     use that as its "variable name" (C being how it is, the var
        ///     name may need to be inserted in the middle of the types, such as
        ///     with arrays and function pointers).
        ///
        /// # Safety
        ///
        /// Here is where the meat of the safety happens: associating a Rust
        /// type to a non-corresponding C definition will cause Undefined
        /// Behavior when a function using such type in its ABI is called.
        ///
        /// ## Examples
        ///
        /// #### `i32`
        ///
        /// ```rust,ignore
        /// unsafe impl CType for i32 {
        ///     #[::safer_ffi::cfg_headers]
        ///     fn c_var_fmt (
        ///         fmt: &'_ mut fmt::Formatter<'_>,
        ///         var_name: &'_ str,
        ///     ) -> fmt::Result
        ///     {
        ///         write!(fmt, "int32_t {}", var_name)
        ///     }
        ///
        ///     // ...
        /// }
        /// ```
        ///
        /// #### `Option<extern "C" fn (i32) -> u32>`
        ///
        /// ```rust,ignore
        /// unsafe impl CType for Option<extern "C" fn (i32) -> u32> {
        ///     #[::safer_ffi::cfg_headers]
        ///     fn c_var_fmt (
        ///         fmt: &'_ mut fmt::Formatter<'_>,
        ///         var_name: &'_ str,
        ///     ) -> fmt::Result
        ///     {
        ///         write!(fmt, "uint32_t (*{})(int32_t)", var_name)
        ///     }
        ///
        ///     // ...
        /// }
        /// ```
        ///
        /// #### `[i32; 42]`
        ///
        /// ```rust,ignore
        /// unsafe impl CType for [i32; 42] {
        ///     #[::safer_ffi::cfg_headers]
        ///     fn c_var_fmt (
        ///         fmt: &'_ mut fmt::Formatter<'_>,
        ///         var_name: &'_ str,
        ///     ) -> fmt::Result
        ///     {
        ///         let typedef_name = format_args!("{}_t", Self::c_short_name());
        ///         write!(fmt, "{} {}", typedef_name, var_name)
        ///     }
        ///
        ///     // Since `c_var_fmt()` requires a one-time typedef, overriding
        ///     // `c_define_self()` is necessary:
        ///     #[::safer_ffi::cfg_headers]
        ///     fn c_define_self (definer: &'_ mut dyn Definer)
        ///       -> fmt::Result
        ///     {
        ///         let typedef_name = &format!("{}_t", Self::c_short_name());
        ///         definer.define_once(typedef_name, &mut |definer| {
        ///             // ensure the array element type is defined
        ///             i32::c_define_self(definer)?;
        ///             write!(definer.out(),
        ///                 "typedef struct {{ {0}; }} {1};\n",
        ///                 i32::c_var("arr[42]"), // `int32_t arr[42]`
        ///                 typedef_name,
        ///             )
        ///         })
        ///     }
        ///
        ///     // etc.
        /// }
        /// ```
        fn c_var_fmt (
            fmt: &'_ mut fmt::Formatter<'_>,
            var_name: &'_ str,
        ) -> fmt::Result
        ;

        /// Convenience function for _callers_ / users of types implementing
        /// [`LegacyCType`][`trait@LegacyCType`].
        ///
        /// The `Display` logic is auto-derived from the implementation of
        /// [`LegacyCType::c_var_fmt`]`()`.
        #[inline]
        fn c_var (
            var_name: &'_ str,
        ) -> var_impl_display::ImplDisplay<'_, Self>
        {
            var_impl_display::ImplDisplay {
                var_name,
                _phantom: Default::default(),
            }
        }

        fn metadata_define_self (definer: &'_ mut dyn Definer) -> io::Result<()>;

        fn metadata_type_usage () -> String;

        __cfg_csharp__! {
            /// Extra typedef code (_e.g._ `[LayoutKind.Sequential] struct ...`)
            fn csharp_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            ;
            // {
            //     Self::define_self(
            //         &CSharp,
            //         definer,
            //     )
            // }

            /// Optional marshaler attached to the type (_e.g._,
            /// `[MarshalAs(UnmanagedType.FunctionPtr)]`)
            fn legacy_csharp_marshaler ()
              -> Option<rust::String>
            {
                None
            }

            // TODO: Optimize out those unnecessary heap-allocations
            /// Type name (_e.g._, `int`, `string`, `IntPtr`)
            fn csharp_ty ()
              -> rust::String
            {
                Self::c_var("").to_string()
            }

            /// Convenience function for formatting `{ty} {var}` in CSharp.
            fn csharp_var (var_name: &'_ str)
              -> rust::String
            {
                format!(
                    "{}{sep}{}",
                    Self::csharp_ty(), var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            }
        }
    }
}

__cfg_headers__! {
    mod var_impl_display {
        use super::*;
        use fmt::*;

        #[allow(missing_debug_implementations)]
        pub
        struct ImplDisplay<'__, T : LegacyCType> {
            pub(in super)
            var_name: &'__ str,

            pub(in super)
            _phantom: ::core::marker::PhantomData<T>,
        }

        impl<T : LegacyCType> Display
            for ImplDisplay<'_, T>
        {
            #[inline]
            fn fmt (self: &'_ Self, fmt: &'_ mut Formatter<'_>)
              -> Result
            {
                T::c_var_fmt(fmt, self.var_name)
            }
        }
    }

    mod short_name_impl_display {
        use super::*;
        use fmt::*;

        #[allow(missing_debug_implementations)]
        pub
        struct ImplDisplay<T : LegacyCType> {
            pub(in super)
            _phantom: ::core::marker::PhantomData<T>,
        }

        impl<T : LegacyCType> Display
            for ImplDisplay<T>
        {
            #[inline]
            fn fmt (self: &'_ Self, fmt: &'_ mut Formatter<'_>)
              -> Result
            {
                T::c_short_name_fmt(fmt)
            }
        }
    }
}

/// The meat of the crate. _The_ trait.
/// This trait describes that **a type has a defined / fixed `#[repr(C)]`
/// layout**.
///
/// This is expressed at the type level by the `unsafe` (trait) type
/// association of `ReprC::CLayout`, which must be a [`CType`][`trait@CType`].
///
/// Because of that property, the type may be used in the API of an
/// `#[ffi_export]`-ed function, where ABI-wise it will be replaced by its
/// equivalent [C layout][`ReprC::CLayout`].
///
/// Then, `#[ffi_export]` will transmute the `CType` parameters back to the
/// provided `ReprC` types, using [`from_raw_unchecked`].
///
/// Although, from a pure point of view, no checks are performed at this step
/// whatsoever, in practice, when `debug_assertions` are enabled, some "sanity
/// checks" are performed on the input parameters: [`ReprC::is_valid`] is
/// called in that case (as part of the implementation of [`from_raw`]).
///
///   - Although that may look innocent, it is actually pretty powerful tool:
///
///     **For instance, a non-null pointer coming from C can, this way, be
///     automatically checked and unwrapped, and the same applies for
///     enumerations having a finite number of valid bit-patterns.**
///
/// # Safety
///
/// It must be sound to transmute from a `ReprC::CLayout` instance when the
/// bit pattern represents a _safe_ instance of `Self`.
///
/// # Implementing `ReprC`
///
/// It is generally recommended to avoid manually (and `unsafe`-ly)
/// implementing the [`ReprC`] trait. Instead, the recommended and blessed way
/// is to use the [`#[derive_ReprC]`](/safer_ffi/layout/attr.derive_ReprC.html)
/// attribute on your `#[repr(C)] struct` (or your field-less
/// `#[repr(<integer>)] enum`).
///
/// [`ReprC`]: `trait@ReprC`
///
/// ## Examples
///
/// #### Simple `struct`
///
/// ```rust,no_run
/// # fn main () {}
/// use ::safer_ffi::prelude::*;
///
/// #[derive_ReprC]
/// #[repr(C)]
/// struct Instant {
///     seconds: u64,
///     nanos: u32,
/// }
/// ```
///
///   - corresponding to the following C definition:
///
///     ```C
///     typedef struct {
///         uint64_t seconds;
///         uint32_t nanos;
///     } Instant_t;
///     ```
///
/// #### Field-less `enum`
///
/// ```rust,no_run
/// # fn main () {}
/// use ::safer_ffi::prelude::*;
///
/// #[derive_ReprC]
/// #[repr(u8)]
/// enum Status {
///     Ok = 0,
///     Busy,
///     NotInTheMood,
///     OnStrike,
///     OhNo,
/// }
/// ```
///
///   - corresponding to the following C definition:
///
///     ```C
///     typedef uint8_t Status_t; enum {
///         STATUS_OK = 0,
///         STATUS_BUSY,
///         STATUS_NOT_IN_THE_MOOD,
///         STATUS_ON_STRIKE,
///         STATUS_OH_NO,
///     }
///     ```
///
/// #### Generic `struct`
///
/// ```rust,no_run
/// # fn main () {}
/// use ::safer_ffi::prelude::*;
///
/// #[derive_ReprC]
/// #[repr(C)]
/// struct Point<Coordinate : ReprC> {
///     x: Coordinate,
///     y: Coordinate,
/// }
/// ```
///
/// Each monomorphization leads to its own C definition:
///
///   - **`Point<i32>`**
///
///     ```C
///     typedef struct {
///         int32_t x;
///         int32_t y;
///     } Point_int32_t;
///     ```
///
///   - **`Point<f64>`**
///
///     ```C
///     typedef struct {
///         double x;
///         double y;
///     } Point_double_t;
///     ```
pub
unsafe
trait ReprC : Sized {
    /// The `CType` having the same layout as `Self`.
    type CLayout : CType;

    /// Sanity checks that can be performed on an instance of the `CType`
    /// layout.
    ///
    /// Such checks are performed when calling [`from_raw`], or equivalently
    /// (⚠️ only with `debug_assertions` enabled ⚠️), [`from_raw_unchecked`].
    ///
    /// Implementation-wise, this function is only a "sanity check" step:
    ///
    ///   - It is valid (although rather pointless) for this function to always
    ///     return `true`, even if the input may be `unsafe` to transmute to
    ///     `Self`, or even be an _invalid_ value of type `Self`.
    ///
    ///   - In the other direction, it is not unsound, although it would be a
    ///     logic error, to always return `false`.
    ///
    ///   - This is because it is impossible to have a function that for any
    ///     type is able to tell if a given bit pattern is a safe value of that
    ///     type.
    ///
    /// In practice, if this function returns `false`, then such result must be
    /// trusted, _i.e._, transmuting such instance to the `Self` type will,
    /// at the very least, break a _safety_ invariant, and it will even most
    /// probably break a _validity_ invariant.
    ///
    /// On the other hand, if the function returns `true`, then the result is
    /// inconclusive; there is no explicit reason to stop going on, but that
    /// doesn't necessarily make it sound.
    ///
    /// # TL,DR
    ///
    /// > This function **may yield false positives** but no false negatives.
    ///
    /// ## Example: `Self = &'borrow i32`
    ///
    /// When `Self = &'borrow i32`, we know that the backing pointer is
    /// necessarily non-null and well-aligned.
    ///
    /// This means that bit-patterns such as `0 as *const i32` or
    /// `37 as *const i32` are "blatantly unsound" to transmute to a
    /// `&'borrow i32`, and thus `<&'borrow i32 as ReprC>::is_valid` will
    /// return `false` in such cases.
    ///
    /// But if given `4 as *const i32`, or if given `{ let p = &*Box::new(42)
    /// as *const i32; p }`, then `is_valid` will return `true` in both cases,
    /// since it doesn't know better.
    ///
    /// ## Example: `bool` or `#[repr(u8)] enum Foo { A, B }`
    ///
    /// In the case of `bool`, or in the case of a `#[repr(<integer>)]`
    /// field-less enum, then the valid bit-patterns and the invalid
    /// bit-patterns are all known and finite.
    ///
    /// In that case, `ReprC::is_valid` will return a `bool` that truly
    /// represents the validity of the bit-pattern, in both directions
    ///
    ///   - _i.e._, no false positives (_validity_-wise);
    ///
    /// Still, there may be _safety_ invariants involved with custom types,
    /// so even then it is unclear.
    fn is_valid (it: &'_ Self::CLayout)
      -> bool
    ;
}

#[doc(hidden)] /** For clarity;
                   this macro may be stabilized
                   if downstream users find it useful
                **/
#[macro_export]
macro_rules! from_CType_impl_ReprC {(
    $(@for[$($generics:tt)*])? $T:ty $(where $($bounds:tt)*)?
) => (
    unsafe
    impl$(<$($generics)*>)? $crate::layout::ReprC
        for $T
    where
        $($($bounds)*)?
    {
        type CLayout = Self;

        #[inline]
        fn is_valid (_: &'_ Self::CLayout)
          -> bool
        {
            true
        }
    }
)}

#[inline]
pub
unsafe
fn from_raw_unchecked<T : ReprC> (c_layout: T::CLayout)
  -> T
{
    if let Some(it) = from_raw::<T>(c_layout) { it } else {
        if cfg!(debug_assertions) || cfg!(test) {
            panic!(
                "Error: not a valid bit-pattern for the type `{}`",
                // c_layout,
                ::core::any::type_name::<T>(),
            );
        } else {
            ::core::hint::unreachable_unchecked()
        }
    }
}

#[deny(unsafe_op_in_unsafe_fn)]
#[inline]
pub
unsafe
fn from_raw<T : ReprC> (c_layout: T::CLayout)
  -> Option<T>
{
    if <T as ReprC>::is_valid(&c_layout).not() {
        None
    } else {
        Some(unsafe {
            const_assert! {
                for [T]
                    [T : ReprC] => [T::CLayout : Copy]
            }
            crate::utils::transmute_unchecked(c_layout)
        })
    }
}

#[deny(unsafe_op_in_unsafe_fn)]
#[inline]
pub
unsafe // May not be sound when input has uninit bytes that the output does not
       // have.
fn into_raw<T : ReprC> (it: T)
  -> T::CLayout
{
    unsafe {
        crate::utils::transmute_unchecked(
            ::core::mem::ManuallyDrop::new(it)
        )
    }
}

pub use impls::Opaque;
pub(in crate)
mod impls;

mod niche;

#[apply(hidden_export)]
use niche::HasNiche as __HasNiche__;

#[apply(hidden_export)]
trait Is { type EqTo : ?Sized; }
impl<T : ?Sized> Is for T { type EqTo = Self; }

/// Alias for `ReprC where Self::CLayout::OPAQUE_KIND = OpaqueKind::Concrete`
pub
trait ConcreteReprC
where
    Self : ReprC,
{
    type ConcreteCLayout
    :
        Is<EqTo = CLayoutOf<Self>> +
        CType<OPAQUE_KIND = OpaqueKind::Concrete> +
    ;
}
impl<T : ?Sized> ConcreteReprC for T
where
    Self : ReprC,
    CLayoutOf<Self> : CType<OPAQUE_KIND = OpaqueKind::Concrete>,
{
    type ConcreteCLayout = CLayoutOf<Self>;
}

#[apply(hidden_export)]
fn __assert_concrete__<T> ()
where
    T : ConcreteReprC,
{}
