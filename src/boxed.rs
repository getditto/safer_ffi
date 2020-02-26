use_prelude!();

/// Same as [`Box<T>`][`rust::Box`], (_e.g._, same `#[repr(C)]` layout), but
/// with **no non-aliasing guarantee**.
#[repr(transparent)]
pub
struct Box<T> (
    ptr::NonNull<T>, // variance is OK because ownership
);

impl<T> From<rust::Box<T>> for Box<T> {
    #[inline]
    fn from (boxed: rust::Box<T>) -> Self
    {
        Self(
            ptr::NonNull::from(rust::Box::leak(boxed))
        )
    }
}

impl<T> Box<T> {
    #[inline]
    pub
    fn into (self: Box<T>) -> rust::Box<T>
    {
        let this = mem::ManuallyDrop::new(self);
        unsafe {
            rust::Box::from_raw(this.0.as_ptr())
        }
    }
}

impl<T> Drop for Box<T> {
    #[inline]
    fn drop (self: &'_ mut Self)
    {
        unsafe {
            drop::<rust::Box<T>>(
                rust::Box::from_raw(self.0.as_ptr())
            );
        }
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    #[inline]
    fn deref (self: &'_ Self) -> &'_ Self::Target
    {
        unsafe {
            &*self.0.as_ptr()
        }
    }
}

impl<T> DerefMut for Box<T> {
    #[inline]
    fn deref_mut (self: &'_ mut Self)
      -> &'_ mut Self::Target
    {
        unsafe {
            &mut *(self.0.as_ptr())
        }
    }
}

unsafe impl<T> Send for Box<T> where
    rust::Box<T> : Send,
{}

unsafe impl<T> Sync for Box<T> where
    rust::Box<T> : Sync,
{}

impl<T : fmt::Debug> fmt::Debug for Box<T> {
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        T::fmt(self, fmt)
    }
}

#[doc(inline)]
pub use super::slice::BoxedSlice;

#[doc(inline)]
pub use super::str::BoxedStr;
