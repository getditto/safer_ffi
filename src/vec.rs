#![cfg_attr(rustfmt, rustfmt::skip)]

use_prelude!();
use ::core::slice;
use crate::slice::*;

ReprC! {
    #[repr(C, js)]
    #[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "alloc")))]
    /// Same as [`Vec<T>`][`rust::Vec`], but with guaranteed `#[repr(C)]` layout
    pub
    struct Vec[T] {
        ptr: ptr::NonNullOwned<T>,
        len: usize,

        cap: usize,
    }
}

impl<T> Vec<T> {
    #[inline]
    pub
    fn as_ref (self: &'_ Self)
      -> slice_ref<'_, T>
    {
        // should optimize to a `transmute_copy`.
        crate::slice::slice_ref {
            ptr: self.ptr.0.into(),
            len: self.len,
            _lt: Default::default(),
        }
    }

    #[inline]
    pub
    fn as_mut (self: &'_ mut Self)
      -> slice_mut<'_, T>
    {
        // should optimize to a `transmute_copy`.
        crate::slice::slice_mut {
            ptr: self.ptr.0.into(),
            len: self.len,
            _lt: Default::default(),
        }
    }
}

/// Convert a [`std::vec::Vec`] to a [`safer_ffi::Vec`].
impl<T> From<rust::Vec<T>>
    for Vec<T>
{
    #[inline]
    fn from (vec: rust::Vec<T>)
      -> Vec<T>
    {
        let len = vec.len();
        let cap = vec.capacity();
        let ptr = mem::ManuallyDrop::new(vec).as_mut_ptr();
        Self {
            ptr: unsafe {
                // Safety: `Vec` guarantees its pointer is nonnull.
                ptr::NonNull::new_unchecked(ptr)
            }.into(),
            len,
            cap,
        }
    }
}

/// Convert a [`safer_ffi::Vec`] to a [`std::vec::Vec`].
impl<T> From<Vec<T>> for rust::Vec<T>
{
    #[inline]
    fn from (value: Vec<T>)
      -> rust::Vec<T>
    {
        let mut this = mem::ManuallyDrop::new(value);
        unsafe {
            // Safety: pointers originate from `Vec`.
            rust::Vec::from_raw_parts(
                this.ptr.as_mut_ptr(),
                this.len,
                this.cap,
            )
        }
    }
}

impl<T> Drop
    for Vec<T>
{
    #[inline]
    fn drop (self: &'_ mut Vec<T>)
    {
        unsafe {
            drop::<rust::Vec<T>>(
                ptr::read(self) // ManuallyDrop::take()
                    .into()
            )
        }
    }
}

impl<T> Deref
    for Vec<T>
{
    type Target = [T];

    fn deref (self: &'_ Vec<T>)
      -> &'_ [T]
    {
        unsafe {
            slice::from_raw_parts(
                self.ptr.as_ptr(),
                self.len,
            )
        }
    }
}
impl<T> DerefMut
    for Vec<T>
{
    fn deref_mut (self: &'_ mut Vec<T>)
      -> &'_ mut [T]
    {
        unsafe {
            slice::from_raw_parts_mut(
                self.ptr.as_mut_ptr(),
                self.len(),
            )
        }
    }
}

unsafe // Safety: from delegation
    impl<T> Send
        for Vec<T>
    where
        rust::Vec<T> : Send,
    {}

unsafe // Safety: from delegation
    impl<T> Sync
        for Vec<T>
    where
        rust::Vec<T> : Sync,
    {}

impl<T> Vec<T> {
    pub
    const EMPTY: Self = Self {
        ptr: ptr::NonNullOwned(ptr::NonNull::dangling(), PhantomData),
        len: 0,
        cap: 0,
    };

    pub
    fn with_rust_mut<R> (
        self: &'_ mut repr_c::Vec<T>,
        f: impl FnOnce(&'_ mut rust::Vec<T>) -> R,
    ) -> R
    {
        let at_c_vec: *mut repr_c::Vec<T> = self;
        unsafe {
            ::unwind_safe::with_state::<rust::Vec<T>>(at_c_vec.read().into())
                .try_eval(f)
                .finally(|rust_vec| {
                    at_c_vec.write(rust_vec.into());
                })
        }
    }
}

impl<T : fmt::Debug + ReprC> fmt::Debug
    for Vec<T>
{
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        <[T] as fmt::Debug>::fmt(&self[..], fmt)
    }
}

#[macro_export]
macro_rules! c_vec { [$($input:tt)*] => (
    $crate::prelude::repr_c::Vec::from($crate::ඞ::vec![ $($input)* ])
)}
