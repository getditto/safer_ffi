//! Trait abstractions describing the semantics of "being `#[repr(C)]`"

use_prelude!();

pub(crate) mod macros;

#[doc(inline)]
pub use crate::CType;
#[doc(inline)]
pub use crate::ReprC;
pub use crate::derive_ReprC;
#[doc(inline)]
pub use crate::from_CType_impl_ReprC;

type_level_enum! {
    pub
    enum OpaqueKind {
        Concrete,
        Opaque,
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
///   - If you truly want a manual implementation of `CType` (_e.g._, for an "opaque type" pattern,
///     _i.e._, a forward declaration), then, to implement the trait so that it works no matter the
///     status of the `safer_ffi/headers` feature, one must define the methods as if feature was
///     present, but with a `#[::safer_ffi::cfg_headers]` gate slapped on _each_ method.
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
///       - the [`CType!`] macro can be used to wrap a `#[repr(C)]` struct definition to _safely_
///         and automagically implement the trait when it is sound to do so.
///
/// Note that types such as Rust's [`bool`] are ruled out by this definition,
/// since it has the ABI of a `u8 <-> uint8_t`, and yet there are many
/// bit-patterns for the `uint8_t` type that do not make _valid_ `bool`s.
///
/// For such types, see the [`ReprC`][`trait@ReprC`] trait.
///
/// Safety (non-exhaustive list at the moment):
///   - `::core::mem::zeroed::<Self>()` must be sound to use.
pub unsafe trait CType: Sized + Copy {
    type OPAQUE_KIND: OpaqueKind::T;

    fn zeroed() -> Self {
        unsafe { ::core::mem::zeroed() }
    }

    #[apply(__cfg_headers__!)]
    /// Necessary one-time code for [`CType::name()`] to make sense.
    ///
    /// Some types, such as `char`, are part of the language, and can be
    /// used directly by [`CType::name()`].
    /// In that case, there is nothing else to _define_, and all is fine.
    ///
    ///   - That is the default implementation of this method: doing nothing.
    ///
    /// But most often than not, a `typedef` or an `#include` is required.
    ///
    /// In that case, here is the place to put it, with the help of the
    /// provided `Definer`.
    ///
    /// # Idempotency?
    ///
    /// Given some `definer: &mut dyn Definer`, **the `define_self__impl(definer)`
    /// call is not to be called more than once, thanks to the convenience
    /// method [`Self::define_self()`], which is the one to guarantee idempotency**
    /// (thanks to the [`Definer`]'s [`.define_once()`][`Definer::define_once()`] helper).
    ///
    /// # Safety
    ///
    /// Given that the defined types may be used by [`CType::name_wrapping_var()`],
    /// and [`CType::name()`], the same safety disclaimers apply.
    ///
    /// ## Examples
    ///
    /// ### `#[repr(C)] struct Foo { x: i32 }`
    ///
    /// ```rust
    /// use ::safer_ffi::headers::Definer;
    /// use ::safer_ffi::headers::languages::HeaderLanguage;
    /// use ::safer_ffi::layout::CType;
    /// use ::safer_ffi::layout::OpaqueKind;
    /// use ::std::io;
    /// use ::std::marker::PhantomData;
    ///
    /// #[derive(Clone, Copy)]
    /// #[repr(C)]
    /// struct Foo {
    ///     x: i32,
    /// }
    ///
    /// unsafe impl CType for Foo {
    ///     #[::safer_ffi::cfg_headers]
    ///     fn define_self__impl(
    ///         language: &'_ dyn HeaderLanguage,
    ///         definer: &'_ mut dyn Definer,
    ///     ) -> io::Result<()> {
    ///         // ensure int32_t makes sense
    ///         <i32 as CType>::define_self(language, definer)?;
    ///         language.declare_struct(
    ///             language,
    ///             definer,
    ///             // no docs.
    ///             &[],
    ///             &PhantomData::<Self>,
    ///             &[::safer_ffi::headers::languages::StructField {
    ///                 docs: &[],
    ///                 name: "x",
    ///                 ty: &PhantomData::<i32>,
    ///             }],
    ///         )?;
    ///         Ok(())
    ///     }
    ///
    ///     #[::safer_ffi::cfg_headers]
    ///     fn short_name() -> String {
    ///         "Foo".into()
    ///     }
    ///
    ///     type OPAQUE_KIND = OpaqueKind::Concrete;
    ///
    ///     // ...
    /// }
    /// ```
    #[allow(nonstandard_style)]
    fn define_self__impl(
        language: &'_ dyn HeaderLanguage,
        definer: &'_ mut dyn Definer,
    ) -> io::Result<()>;

    #[apply(__cfg_headers__!)]
    fn define_self(
        language: &'_ dyn HeaderLanguage,
        definer: &'_ mut dyn Definer,
    ) -> io::Result<()> {
        definer.define_once(
            &F(|out| Self::render(out, language)).to_string(),
            &mut |definer| Self::define_self__impl(language, definer),
        )
    }

    #[apply(__cfg_headers__!)]
    /// A short-name description of the type, mainly used to fill
    /// "placeholders" such as when monomorphising generics structs or
    /// arrays.
    ///
    /// This provides the implementation used by [`CType::short_name`]`()`.
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
    /// write!(fmt, "{}_{}_array", <T as CType>::short_name(), N)
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
    fn short_name() -> String;

    /// Display itself as header code which refers to this C type.
    ///
    /// This can be:
    ///
    ///   - either through some direct syntactical construct derived off some other stuff, e.g., in
    ///     C, `{} const*` for the `*const T` case,
    ///
    ///   - or simply by given a simple/single identifier name from a helper type alias or type
    ///     definition having occurred in `define_self` (common case).
    ///
    ///     In this case, the name is probably going to be equal to `Self::short_name() + "_t"`.
    ///
    ///     **The default implementation does this.**
    #[apply(__cfg_headers__!)]
    fn render(
        out: &'_ mut dyn io::Write,
        _language: &'_ dyn HeaderLanguage,
    ) -> io::Result<()> {
        write!(out, "{}_t", Self::short_name())
    }

    /// Convenience directly-`String`-outputting version of [`Self::render()`].
    #[apply(__cfg_headers__!)]
    fn name(language: &dyn HeaderLanguage) -> String {
        F(|out| Self::render(out, language)).to_string()
    }

    /// Same as [`Self::render()`], but for "appending the varname/fn-name after it, with
    /// whitespace.
    ///
    /// This, on its own would be a silly thing for which to dedicate a whole function.
    ///
    /// However, C being how it is, in the non-simple/single-typename ident cases for
    /// `Self::render()`, mainly and most notably, in the array and `fn` pointer cases, the
    /// varname/fn-name does not just go after the whole type.
    ///
    /// Instead, **it has to be interspersed in the middle of the type**.
    ///
    /// For instance, in the `fn(c_int) -> u8` case, it would have to be:
    ///
    /// > `uint8_t (*{var_name})(int)`.
    ///
    /// So such cases need to override the default implementation here, do the right thing, and then
    /// override the other simpler non-`wrapping_var` versions thereof, to delegate to this function
    /// with `var_name = ""`.
    ///
    /// > ⚠️ **NOTE**: when overriding this default impl, remember to `Self::render()` as
    /// `Self::render_wrapping_var(…, "")`.
    ///
    /// ---
    ///
    /// Default implementation is to simply emit `{self.render()}{sep}{var_name}`, in pseudo-code
    /// parlance.
    #[apply(__cfg_headers__!)]
    fn render_wrapping_var(
        out: &'_ mut dyn io::Write,
        language: &'_ dyn HeaderLanguage,
        // Either a `&&str`, or a `&fmt::Arguments<'_>`, for instance.
        var_name: Option<&dyn ::core::fmt::Display>,
    ) -> io::Result<()> {
        write!(
            out,
            "{}{sep}{var_name}",
            F(|out| Self::render(out, language)),
            sep = var_name.sep(),
            var_name = var_name.or_empty(),
        )?;
        Ok(())
    }

    #[apply(__cfg_headers__!)]
    /// The core method of the trait: it provides the code to emit in the target
    /// [`HeaderLanguage`] in order to refer to the corresponding C type.
    ///
    /// The implementations are thus much like any classic `.to_string()` impl,
    /// except that:
    ///
    ///   - it must output valid C code representing the type corresponding to the Rust type.
    ///
    ///   - a `var_name` may be supplied, in which case the type must use that as its "variable
    ///     name" (C being how it is, the var name may need to be inserted in the middle of the
    ///     types, such as with arrays and function pointers).
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
    /// ```rust ,ignore
    /// # #[repr(transparent)] struct i32(::core::primitive::i32);
    ///
    /// use ::safer_ffi::{headers::languages::HeaderLanguage, layout::CType};
    ///
    /// unsafe impl CType for i32 {
    ///     #[::safer_ffi::cfg_headers]
    ///     fn name_wrapping_var (
    ///         header_language: &dyn HeaderLanguage,
    ///         var_name: Option<&dyn ::core::fmt::Display>,
    ///     ) -> String
    ///     {
    ///         // Usually this kind of logic for primitive types is
    ///         // provided by the `HeaderLanguage` itself, rather than hard-coded by the type…
    ///         assert_eq!(header_language.language_name(), "C");
    ///
    ///         let sep = if var_name { " " } else { "" };
    ///         format!("int32_t{sep}{var_name}")
    ///     }
    ///
    ///     // ...
    /// }
    /// ```
    fn name_wrapping_var(
        language: &'_ dyn HeaderLanguage,
        var_name: Option<&dyn ::core::fmt::Display>,
    ) -> String {
        F(|out| Self::render_wrapping_var(out, language, var_name)).to_string()
    }

    #[apply(__cfg_headers__!)]
    /// Optional language-specific metadata attached to the type (_e.g._,
    /// some `[MarshalAs(UnmanagedType.FunctionPtr)]` annotation for C#).
    ///
    /// To be done using:
    ///
    /// <code>\&[provide_with]\(|req| req.give_if_requested::\<[CSharpMarshaler]\>(…))</code>
    ///
    /// [CSharpMarshaler]: `crate::headers::languages::CSharpMarshaler`
    fn metadata() -> &'static dyn Provider {
        &None
    }

    #[apply(__cfg_headers__!)]
    fn metadata_type_usage() -> String;
}

/// The meat of the crate. _The_ trait.
///
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
/// struct Point<Coordinate: ReprC> {
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
pub unsafe trait ReprC: Sized {
    /// The `CType` having the same layout as `Self`.
    type CLayout: CType;

    /// Sanity checks that can be performed on an instance of the `CType`
    /// layout.
    ///
    /// Such checks are performed when calling [`from_raw`], or equivalently
    /// (⚠️ only with `debug_assertions` enabled ⚠️), [`from_raw_unchecked`].
    ///
    /// Implementation-wise, this function is only a "sanity check" step:
    ///
    ///   - It is valid (although rather pointless) for this function to always return `true`, even
    ///     if the input may be `unsafe` to transmute to `Self`, or even be an _invalid_ value of
    ///     type `Self`.
    ///
    ///   - In the other direction, it is not unsound, although it would be a logic error, to always
    ///     return `false`.
    ///
    ///   - This is because it is impossible to have a function that for any type is able to tell if
    ///     a given bit pattern is a safe value of that type.
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
    fn is_valid(it: &'_ Self::CLayout) -> bool;
}

pub type CLayoutOf<ImplReprC> = <ImplReprC as ReprC>::CLayout;

#[doc(hidden)] /** For clarity;
                   this macro may be stabilized
                   if downstream users find it useful
                **/
#[macro_export]
#[cfg_attr(rustfmt, rustfmt::skip)]
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
pub unsafe fn from_raw_unchecked<T: ReprC>(c_layout: T::CLayout) -> T {
    if let Some(it) = unsafe { from_raw::<T>(c_layout) } {
        it
    } else {
        if cfg!(debug_assertions) || cfg!(test) {
            panic!(
                "Error: not a valid bit-pattern for the type `{}`",
                // c_layout,
                ::core::any::type_name::<T>(),
            );
        } else {
            unsafe { ::core::hint::unreachable_unchecked() }
        }
    }
}

#[deny(unsafe_op_in_unsafe_fn)]
#[inline]
pub unsafe fn from_raw<T: ReprC>(c_layout: T::CLayout) -> Option<T> {
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
pub unsafe fn into_raw<T: ReprC>(it: T) -> T::CLayout {
    unsafe { crate::utils::transmute_unchecked(::core::mem::ManuallyDrop::new(it)) }
}

pub use impls::Opaque;
pub(crate) mod impls;

mod niche;

#[apply(hidden_export)]
use niche::HasNiche as __HasNiche__;

#[apply(hidden_export)]
trait Is {
    type EqTo: ?Sized;
}
impl<T: ?Sized> Is for T {
    type EqTo = Self;
}

/// Alias for `ReprC where Self::CLayout::OPAQUE_KIND = OpaqueKind::Concrete`
pub trait ConcreteReprC
where
    Self: ReprC,
{
    type ConcreteCLayout: Is<EqTo = CLayoutOf<Self>> + CType<OPAQUE_KIND = OpaqueKind::Concrete>;
}
impl<T: ?Sized> ConcreteReprC for T
where
    Self: ReprC,
    CLayoutOf<Self>: CType<OPAQUE_KIND = OpaqueKind::Concrete>,
{
    type ConcreteCLayout = CLayoutOf<Self>;
}

#[apply(hidden_export)]
fn __assert_concrete__<T>()
where
    T: ConcreteReprC,
{
}
