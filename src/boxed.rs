//! `#[repr(C)]` [`Box`][`rust::Box`]ed types.
//!
//! # When to use [`repr_c::Box<T>`] _vs._ [`ThinBox<T>`]?
//!
//! In `fn` signatures, prefer [`repr_c::Box<T>`], since it reads more nicely.
//!
//! But [`repr_c::Box<T>`], by virtue of having been defined as a type alias rather than its own,
//! dedicated, new-type wrapper `struct` (for historical reasons, the author wishes to fix this in
//! some future version), comes with a couple limitations of which to be mindful:
//!
//!   - you cannot use associated functions on [`repr_c::Box<T>`]:
//!
//!     ```rust ,compile_fail
//!     use ::safer_ffi::prelude::*;
//!
//!     repr_c::Box::new(42); // Error!
//!     ```
//!
//!     ```rust ,ignore
//!     # /*
//!     error[E0599]: no function or associated item named `new` found for type
//!     `_` in the current scope --> src/arc.rs:19:14
//!         |
//!     7   | repr_c::Box::new(42); // Error!
//!         |              ^^^ function or associated item not found in `_`
//!         |
//!         = help: items from traits can only be used if the trait is in scope
//!     # */
//!     ```
//!
//!   - you should not `impl Trait for repr_c::Box<T> {`. Indeed, the `type` alias is defined as an
//!     associated type "output" [through a `trait`][tr], and from the point of view of _coherence_
//!     (the SemVer-aware overlapping-`impl`s checker), when from within a downstream crate, such as
//!     yours (🫵), this is treated as a fully _blanket_ (generic) type param.
//!
//!     ```rust ,compile_fail
//!     use ::safer_ffi::prelude::*;
//!
//!     trait MyFancyTrait {}
//!
//!     impl MyFancyTrait for i32 {}
//!
//!     // Error, *potentially*-overlapping `impl` in the future: what if
//!     // `safer-ffi` were to define the assoc type of `FitForCBox` as `i32`?
//!     //
//!     // Anyhow, this is treated the same as if it had been:
//!     // `impl<T> MyFancyTrait for T {}`, hence the overlap with `i32` above.
//!     impl MyFancyTrait for repr_c::Box<bool> {}
//!     ```
//!
//!     ```rust ,ignore
//!     # /*
//!     error[E0119]: conflicting implementations of trait `MyFancyTrait` for type `i32`
//!       --> src/boxed.rs:44:1
//!        |
//!     9  | impl MyFancyTrait for i32 {}
//!        | ------------------------- first implementation here
//!     ...
//!     16 | impl MyFancyTrait for repr_c::Box<bool> {}
//!        | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `i32`
//!     # */
//!     ```
//!
//!       - To be completely honest, I don't feel like this failing is _that_ justified, and it may
//!         just be a bug in _coherence_, but oh well, it is what it is 🤷.
//!
//! In either of these situations, you may want to directly target [`ThinBox<T>`] instead.
//!
//! [tr]: [`FitForCBox`]
use_prelude!();

ReprC! {
    #[repr(transparent)]
    /// An FFI-safe representation of a standard-library `Box<T>`, as a thin pointer.
    ///
    /// (It is thus the same as [`Box<T>`][`rust::Box`], (_e.g._, same `#[repr(C)]` layout), but
    /// with **no non-aliasing guarantee**.)
    ///
    /// # When to use [`repr_c::Box<T>`] _vs._ [`ThinBox<T>`]?
    ///
    /// See [the module docs][self]
    pub
    struct ThinBox[T] (
        ptr::NonNullOwned<T>,
    );
}

impl<T> From<rust::Box<T>> for ThinBox<T> {
    #[inline]
    fn from(boxed: rust::Box<T>) -> ThinBox<T> {
        Self(ptr::NonNull::from(rust::Box::leak(boxed)).into())
    }
}

impl<T> ThinBox<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        rust::Box::new(value).into()
    }

    #[inline]
    pub fn into(self: ThinBox<T>) -> rust::Box<T> {
        let mut this = mem::ManuallyDrop::new(self);
        unsafe { rust::Box::from_raw(this.0.as_mut_ptr()) }
    }
}

impl<T> Drop for ThinBox<T> {
    #[inline]
    fn drop(self: &'_ mut ThinBox<T>) {
        unsafe {
            drop::<rust::Box<T>>(rust::Box::from_raw(self.0.as_mut_ptr()));
        }
    }
}

impl<T> Deref for ThinBox<T> {
    type Target = T;

    #[inline]
    fn deref(self: &'_ ThinBox<T>) -> &'_ T {
        unsafe { &*self.0.as_ptr() }
    }
}

impl<T> DerefMut for ThinBox<T> {
    #[inline]
    fn deref_mut(self: &'_ mut ThinBox<T>) -> &'_ mut T {
        unsafe { &mut *(self.0.as_mut_ptr()) }
    }
}

unsafe impl<T> Send for ThinBox<T> where rust::Box<T>: Send {}

unsafe impl<T> Sync for ThinBox<T> where rust::Box<T>: Sync {}

impl<T: Clone> Clone for ThinBox<T> {
    #[inline]
    fn clone(self: &'_ Self) -> Self {
        Self::new(T::clone(self))
    }
}

impl<T: fmt::Debug> fmt::Debug for ThinBox<T> {
    fn fmt(
        self: &'_ Self,
        fmt: &'_ mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        T::fmt(self, fmt)
    }
}

#[doc(no_inline)]
pub use crate::slice::slice_boxed;
#[doc(no_inline)]
pub use crate::string::str_boxed;

/// A `?Sized`-aware alias, for convenience:
///
///   - when `T : Sized`, this is [`ThinBox<T>`];
///   - when `T = [U]`, this is [`c_slice::Box<U>`];
///   - when `T = dyn 'static + Send + FnMut(…) -> _`, this is [the dedicated hand-rolled FFI-safe
///     `dyn` "closure" struct of the given arity][crate::closure::boxed].
///
/// # When to use [`repr_c::Box<T>`] _vs._ [`ThinBox<T>`]?
///
/// See [the module docs][self]
pub type Box<T> = <T as FitForCBox>::CBoxWrapped;

#[doc(hidden)]
#[deprecated = "Use `ThinBox<T>` instead"]
pub type Box_<T> = ThinBox<T>;

/// Helper trait enabling the definition of [`Box<T>`].
pub trait FitForCBox {
    type CBoxWrapped;
}

impl<T: Sized> FitForCBox for T {
    type CBoxWrapped = ThinBox<T>;
}

impl<T: Sized> FitForCBox for [T] {
    type CBoxWrapped = c_slice::Box<T>;
}
