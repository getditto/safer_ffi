//! Trait abstractions describing the semantics of "being `#[repr(C)]`"

use_prelude!();

#[macro_use]
mod macros;

#[doc(inline)]
pub use crate::{from_CType_impl_ReprC, ReprC, CType};

cfg_proc_macros! {
    pub use ::proc_macro::{
        derive_ReprC,
    };
}

__cfg_headers__! {
    #[cfg_attr(feature = "nightly",
        doc(cfg(feature = "headers")),
    )]
    /// Helper for the generation of C headers.
    ///
    /// Defining C headers requires _two_ abstractions:
    ///
    ///   - set-like lookup by name, to ensure each type is defined at most once;
    ///
    ///   - a [`Write`][`::std::io::Write`]able "out stream", where the headers
    ///     should be written to.
    ///
    /// This trait minimally combines both abstractions.
    pub
    trait Definer : definer_ext::__ {
        /// Must return `true` iff an actual `insert` happened.
        fn insert (self: &'_ mut Self, name: &'_ str)
          -> bool
        ;

        fn out (self: &'_ mut Self)
          -> &'_ mut dyn io::Write
        ;
    }

    mod definer_ext {
        use super::*;

        pub
        trait __ {
            fn define (
                self: &'_ mut Self,
                name: &'_ str,
                write_typedef: &'_ mut dyn
                    FnMut (&'_ mut dyn Definer) -> io::Result<()>
                ,
            ) -> io::Result<()>
            ;
        }

        impl<T : Definer> __
            for T
        {
            fn define (
                self: &'_ mut Self,
                name: &'_ str,
                write_typedef: &'_ mut dyn
                    FnMut (&'_ mut dyn Definer) -> io::Result<()>
                ,
            ) -> io::Result<()>
            {
                if self.insert(name) {
                    write_typedef(self)?;
                }
                Ok(())
            }
        }
    }
}

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
///     the `repr_c/headers` feature, one must define the methods as if
///     feature was present, but with a `#[::repr_c::cfg_headers]` gate slapped
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
///   - an recursively, a non-zero-sized `#[repr(C)]` struct of `CType` fields.
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
unsafe trait CType
:
    Sized +
    Copy +
{
    __cfg_headers__! {
        /// A short-name description of the type, mainly used to fill
        /// "placeholders" such as when monomorphising generics structs or
        /// arrays.
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
        /// <T as CType>::with_c_short_name(|it| format!("{}_{}_array", it, N))
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
        ///
        /// ## `with` ?
        ///
        /// To avoid performing too many allocations when recursing, this
        /// function uses the Continuation Passing Style pattern to allow
        /// using its own stack to allocate the result, and thus allow to:
        ///
        /// ```rust,ignore
        /// ret(&format_args!(...))
        /// ```
        ///
        /// when implementing the method.
        fn with_c_short_name<R> (ret: impl FnOnce(&'_ dyn fmt::Display) -> R)
          -> R
        ;

        /// Necessary one-time code for `c_fmt` to make sense.
        ///
        /// Some types, such as `char`, are part of the language, and can be
        /// used directly by `c_fmt`. In that case, there is nothing else
        /// to _define_, and all is fine.
        ///
        ///   - That is the default implementation of this method: doing
        ///     nothing.
        ///
        /// But most often than not, a `typedef` or an `#include` is required.
        ///
        /// In that case, here is the place to put it, using a provided
        /// `Definer`.
        ///
        /// # Safety
        ///
        /// Given that the name outputted by `c_fmt` may refer to a definition
        /// from here, the same safety disclaimers apply.
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
        ///     #[::repr_c::cfg_headers]
        ///     fn c_define_self (definer: &'_ mut dyn Definer)
        ///       -> io::Result<()>
        ///     {
        ///         definer.define("<stdint.h>", &mut |definer| {
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
        ///     #[::repr_c::cfg_headers]
        ///     fn c_define_self (definer: &'_ mut dyn Definer)
        ///       -> io::Result<()>
        ///     {
        ///         definer.define("Foo_t", &mut |definer| {
        ///             // ensure int32_t makes sense
        ///             <i32 as CType>::c_define_self(definer)?;
        ///             write!(definer.out(),
        ///                 "typedef struct {{ {}; }} Foo_t;",
        ///                 <i32 as CType>::c_display("x"),
        ///             )
        ///         })
        ///     }
        ///
        ///     // ...
        /// }
        /// ```
        #[inline]
        fn c_define_self (definer: &'_ mut dyn Definer)
          -> io::Result<()>
        {
            let _ = definer;
            Ok(())
        }

        /// The core method of the trait: it provides the implementation to be
        /// used by [`CType::c_display`], by bringing a `Formatter` in scope.
        ///
        /// The implementations are thus much like any classic `Display` impl,
        /// except that:
        ///
        ///   - it must output valid C code representing the type corresponding
        ///     to the Rust type.
        ///
        ///   - a `var_name` may be supplied, in which case the type must
        ///     use that as its "variable name" (C being how it is, the var
        //      name may need to be inserted in the middle of the types, such as
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
        ///     #[::repr_c::cfg_headers]
        ///     fn c_fmt (
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
        /// #### `[i32; 42]`
        ///
        /// ```rust,ignore
        /// unsafe impl CType for [i32; 42] {
        ///     #[::repr_c::cfg_headers]
        ///     fn c_fmt (
        ///         fmt: &'_ mut fmt::Formatter<'_>,
        ///         var_name: &'_ str,
        ///     ) -> fmt::Result
        ///     {
        ///         write!(fmt, "int32_t {}[42]", var_name)
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
        ///     #[::repr_c::cfg_headers]
        ///     fn c_fmt (
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
        /// #### More advanced types
        ///
        /// In this case, an actual `typedef` will need to be used to define the
        /// type, using [`CType::c_define_self`]`()`, and then using `c_fmt` is
        /// just a matter of outputing the name of the newly defined type next
        /// to the `var_name`, like with `int32_t`.
        fn c_fmt (
            fmt: &'_ mut fmt::Formatter<'_>,
            var_name: &'_ str,
        ) -> fmt::Result
        ;

        /// Convenience function for _callers_ / users of types implementing
        /// [`CType`][`trait@CType`].
        ///
        /// Indeed, since the function implemented in [`CType::c_fmt`] is not
        /// `Display`, one cannot directly use `c_fmt` with `"{}"` formatting.
        ///
        /// Instead, `c_display` implements as way to derive a `Display` impl
        /// by "capturing" the `var_name` parameter.
        #[inline]
        fn c_display<'__> (
            var_name: &'__ str,
        ) -> impl_display::ImplDisplay<'__, Self>
        {
            impl_display::ImplDisplay {
                var_name,
                _phantom: Default::default(),
            }
        }
    }
}

__cfg_headers__! {
    mod impl_display {
        use super::*;
        use fmt::*;

        #[allow(missing_debug_implementations)]
        pub
        struct ImplDisplay<'__, T : CType> {
            pub(in super)
            var_name: &'__ str,

            pub(in super)
            _phantom: ::core::marker::PhantomData<T>,
        }

        impl<T : CType> Display
            for ImplDisplay<'_, T>
        {
            fn fmt (self: &'_ Self, fmt: &'_ mut Formatter<'_>)
              -> Result
            {
                T::c_fmt(fmt, self.var_name)
            }
        }
    }
}

#[cfg(docs)]
pub(in crate) use ReprC as ReprCTrait;

/// The meat of the crate. _The_ trait.
/// This trait describes that **a type has a defined / fixed `#[repr(C)]`
/// layout**.
///
/// This is expressed at the type level by the `unsafe` (trait) type
/// association of `ReprC::CLayout`, which must be a [`CType`][`trait@CType`].
///
/// Because of that property, the type may be used in the API of an
/// `#[ffi_export]`-ed function, where ABI-wise it will can replaced by its
/// equivalent [C layout][`ReprC::CLayout`].
///
/// Then, `#[ffi_export]` will transmute the `CType` parameters back to the
/// provided `ReprC` types, using [`from_raw_unchecked`].
///
/// Although from a pure point of view, no checks are performed at this step
/// whatsoever, in practice, when `debug_assertions` are enabled some "sanity
/// checks" are performed on the input parameters: [`ReprC::is_valid`] is
/// called in that case (as part of the implementation of [`from_raw`]).
///
///   - Although that may look innocent, it is actually pretty powerful tool:
///
///     For instance, a non-null pointer coming from C can, this way, be
///     automatically checked and unwrapped, and the same applies for
///     enumerations having a finite number of valid bit-patterns.
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
    /// # Tl,DR
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

#[cfg_attr(all(feature = "proc_macros", not(docs)),
    require_unsafe_in_body,
)]
#[cfg_attr(not(feature = "proc_macros"),
    allow(unused_unsafe),
)]
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

#[cfg_attr(all(feature = "proc_macros", not(docs)),
    require_unsafe_in_body,
)]
#[cfg_attr(not(feature = "proc_macros"),
    allow(unused_unsafe),
)]
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

mod impls;

mod niche;

#[doc(hidden)] /* Not part of the public API */ pub
use niche::HasNiche as __HasNiche__;
