//! `#[repr(C)]` [`Arc`][`rust::Arc`]ed types.
//!
//! Like [`crate::boxed`], but for [`Arc`][`rust::Arc`].
//!
//! # When to use [`repr_c::Arc<T>`] _vs._ [`ThinArc<T>`]?
//!
//! In `fn` signatures, prefer [`repr_c::Arc<T>`], since it reads more nicely.
//!
//! But [`repr_c::Arc<T>`], by virtue of having been defined as a type alias rather than its own,
//! dedicated, new-type wrapper `struct` (for historical reasons, the author wishes to fix this in
//! some future version), comes with a couple limitations of which to be mindful:
//!
//!   - you cannot use associated functions on [`repr_c::Arc<T>`]:
//!
//!     ```rust ,compile_fail
//!     use ::safer_ffi::prelude::*;
//!
//!     repr_c::Arc::new(42); // Error!
//!     ```
//!
//!     ```rust ,ignore
//!     # /*
//!     error[E0599]: no function or associated item named `new` found for type
//!     `_` in the current scope --> src/arc.rs:19:14
//!         |
//!     7   | repr_c::Arc::new(42); // Error!
//!         |              ^^^ function or associated item not found in `_`
//!         |
//!         = help: items from traits can only be used if the trait is in scope
//!     # */
//!     ```
//!
//!   - you should not `impl Trait for repr_c::Arc<T> {`. Indeed, the `type` alias is defined as an
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
//!     // `safer-ffi` were to define the assoc type of `FitForCArc` as `i32`?
//!     //
//!     // Anyhow, this is treated the same as if it had been:
//!     // `impl<T> MyFancyTrait for T {}`, hence the overlap with `i32` above.
//!     impl MyFancyTrait for repr_c::Arc<bool> {}
//!     ```
//!
//!     ```rust ,ignore
//!     # /*
//!     error[E0119]: conflicting implementations of trait `MyFancyTrait` for type `i32`
//!       --> src/arc.rs:44:1
//!        |
//!     9  | impl MyFancyTrait for i32 {}
//!        | ------------------------- first implementation here
//!     ...
//!     16 | impl MyFancyTrait for repr_c::Arc<bool> {}
//!        | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ conflicting implementation for `i32`
//!     # */
//!     ```
//!
//!       - To be completely honest, I don't feel like this failing is _that_ justified, and it may
//!         just be a bug in _coherence_, but oh well, it is what it is 🤷.
//!
//! In either of these situations, you may want to directly target [`ThinArc<T>`] instead.
//!
//! [tr]: [`FitForCArc`]

use_prelude!();

ReprC! {
    /// An FFI-safe representation of a standard-library `Arc<T>`, as a thin pointer to its `T`.
    ///
    /// # When to use [`repr_c::Arc<T>`] _vs._ [`ThinArc<T>`]?
    ///
    /// See [the module docs][self]
    #[repr(transparent)]
    pub
    struct ThinArc[T] (
        ptr::NonNullOwned<T>,
    );
}

impl<T> From<rust::Arc<T>> for ThinArc<T> {
    #[inline]
    fn from(arced: rust::Arc<T>) -> Arc<T> {
        let raw = rust::Arc::into_raw(arced);
        Self(ptr::NonNull::new(raw.cast_mut()).unwrap().into())
    }
}

impl<T> ThinArc<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        rust::Arc::new(value).into()
    }

    #[inline]
    pub fn into(self: ThinArc<T>) -> rust::Arc<T> {
        let mut this = mem::ManuallyDrop::new(self);
        unsafe { rust::Arc::from_raw(this.0.as_mut_ptr()) }
    }

    /// See [`rust::Arc<T>::as_ptr()`].
    #[inline]
    pub fn as_ptr(this: &Self) -> *const T {
        this.0.as_ptr()
    }

    /// See [`rust::Arc<T>::into_raw()`].
    #[inline]
    pub fn into_raw(self) -> *const T {
        Self::as_ptr(&*::core::mem::ManuallyDrop::new(self))
    }

    /// See [`rust::Arc<T>::from_raw()`].
    #[inline]
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        Self(unsafe { ptr::NonNull::new_unchecked(ptr.cast_mut()) }.into())
    }

    /// Morally, a <code>\&[ThinArc\<T\>][`ThinArc`] -> \&[Arc\<T\>][`rust::Arc`]</code> conversion.
    ///
    /// For lifetime reasons, this is exposed as a scoped / callback / CPS API.
    ///
    /// # Example
    ///
    /// ```rust
    /// use ::safer_ffi::prelude::*;
    /// use ::std::sync::Arc;
    ///
    /// fn increment_strong_count<T>(a: &repr_c::Arc<T>) {
    ///     a.with_rust(|a: &Arc<T>| unsafe {
    ///         # //
    ///         Arc::<T>::increment_strong_count(Arc::as_ptr(&a))
    ///     })
    /// }
    /// ```
    #[inline]
    pub fn with_rust<R>(
        &self,
        scope: impl FnOnce(&rust::Arc<T>) -> R,
    ) -> R {
        let yield_ = scope;
        let arc: &rust::Arc<T> =
            &*::core::mem::ManuallyDrop::new(unsafe { rust::Arc::from_raw(ThinArc::as_ptr(self)) });
        yield_(arc)
    }
}

impl<T> Drop for ThinArc<T> {
    #[inline]
    fn drop(self: &'_ mut ThinArc<T>) {
        unsafe {
            drop::<rust::Arc<T>>(rust::Arc::from_raw(self.0.as_mut_ptr()));
        }
    }
}

impl<T> Deref for ThinArc<T> {
    type Target = T;

    #[inline]
    fn deref(self: &'_ ThinArc<T>) -> &'_ T {
        unsafe { &*self.0.as_ptr() }
    }
}

unsafe impl<T> Send for ThinArc<T> where rust::Arc<T>: Send {}

unsafe impl<T> Sync for ThinArc<T> where rust::Arc<T>: Sync {}

impl<T> Clone for ThinArc<T> {
    #[inline]
    fn clone(self: &'_ Self) -> Self {
        self.with_rust(rust::Arc::clone).into()
    }
}

impl<T: fmt::Debug> fmt::Debug for ThinArc<T> {
    fn fmt(
        self: &'_ Self,
        fmt: &'_ mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        T::fmt(&**self, fmt)
    }
}

/// A `?Sized`-aware alias, for convenience:
///
///   - when `T : Sized`, this is [`ThinArc<T>`];
///   - when `T = dyn 'static + Send + Sync + Fn(…) -> _`, this is [the dedicated hand-rolled
///     FFI-safe `dyn` "closure" struct of the given arity][crate::closure::arc].
///
/// # When to use [`repr_c::Arc<T>`] _vs._ [`ThinArc<T>`]?
///
/// See [the module docs][self]
pub type Arc<T> = <T as FitForCArc>::CArcWrapped;

/// Helper trait enabling the definition of [`Arc<T>`].
pub trait FitForCArc {
    type CArcWrapped;
}

impl<T: Sized> FitForCArc for T {
    type CArcWrapped = ThinArc<T>;
}
