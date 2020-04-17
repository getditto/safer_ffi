//! `#[repr(C)]` [`Box`][`rust::Box`]ed types.

use_prelude!();

ReprC! {
    #[repr(transparent)]
    /// Same as [`Box<T>`][`rust::Box`], (_e.g._, same `#[repr(C)]` layout), but
    /// with **no non-aliasing guarantee**.
    pub
    struct Box[T] (
        ptr::NonNullOwned<T>,
    );
}

impl<T> From<rust::Box<T>>
    for Box<T>
{
    #[inline]
    fn from (boxed: rust::Box<T>)
      -> Box<T>
    {
        Self(
            ptr::NonNull::from(rust::Box::leak(boxed))
                .into()
        )
    }
}

impl<T> Box<T> {
    #[inline]
    pub
    fn into (self: Box<T>)
      -> rust::Box<T>
    {
        let mut this = mem::ManuallyDrop::new(self);
        unsafe {
            rust::Box::from_raw(this.0.as_mut_ptr())
        }
    }
}

impl<T> Drop
    for Box<T>
{
    #[inline]
    fn drop (self: &'_ mut Box<T>)
    {
        unsafe {
            drop::<rust::Box<T>>(
                rust::Box::from_raw(self.0.as_mut_ptr())
            );
        }
    }
}

impl<T> Deref
    for Box<T>
{
    type Target = T;

    #[inline]
    fn deref (self: &'_ Box<T>)
      -> &'_ T
    {
        unsafe {
            &*self.0.as_ptr()
        }
    }
}

impl<T> DerefMut
    for Box<T>
{
    #[inline]
    fn deref_mut (self: &'_ mut Box<T>)
      -> &'_ mut T
    {
        unsafe {
            &mut *(self.0.as_mut_ptr())
        }
    }
}

unsafe impl<T> Send
    for Box<T>
where
    rust::Box<T> : Send,
{}

unsafe impl<T> Sync
    for Box<T>
where
    rust::Box<T> : Sync,
{}

impl<T : fmt::Debug> fmt::Debug
    for Box<T>
{
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        T::fmt(self, fmt)
    }
}

#[doc(no_inline)]
pub use crate::slice::slice_boxed;

#[doc(no_inline)]
pub use crate::string::str_boxed;
