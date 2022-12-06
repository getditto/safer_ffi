#![cfg_attr(rustfmt, rustfmt::skip)]
//! `#[repr(C)]` [`Box`][`rust::Box`]ed types.

use_prelude!();

ReprC! {
    #[repr(transparent)]
    /// Same as [`Box<T>`][`rust::Box`], (_e.g._, same `#[repr(C)]` layout), but
    /// with **no non-aliasing guarantee**.
    pub
    struct Box_[T] (
        ptr::NonNullOwned<T>,
    );
}

impl<T> From<rust::Box<T>>
    for Box_<T>
{
    #[inline]
    fn from (boxed: rust::Box<T>)
      -> Box_<T>
    {
        Self(
            ptr::NonNull::from(rust::Box::leak(boxed))
                .into()
        )
    }
}

impl<T> Box_<T> {
    #[inline]
    pub
    fn new (value: T)
      -> Self
    {
        rust::Box::new(value)
            .into()
    }

    #[inline]
    pub
    fn into (self: Box_<T>)
      -> rust::Box<T>
    {
        let mut this = mem::ManuallyDrop::new(self);
        unsafe {
            rust::Box::from_raw(this.0.as_mut_ptr())
        }
    }
}

impl<T> Drop
    for Box_<T>
{
    #[inline]
    fn drop (self: &'_ mut Box_<T>)
    {
        unsafe {
            drop::<rust::Box<T>>(
                rust::Box::from_raw(self.0.as_mut_ptr())
            );
        }
    }
}

impl<T> Deref
    for Box_<T>
{
    type Target = T;

    #[inline]
    fn deref (self: &'_ Box_<T>)
      -> &'_ T
    {
        unsafe {
            &*self.0.as_ptr()
        }
    }
}

impl<T> DerefMut
    for Box_<T>
{
    #[inline]
    fn deref_mut (self: &'_ mut Box_<T>)
      -> &'_ mut T
    {
        unsafe {
            &mut *(self.0.as_mut_ptr())
        }
    }
}

unsafe impl<T> Send
    for Box_<T>
where
    rust::Box<T> : Send,
{}

unsafe impl<T> Sync
    for Box_<T>
where
    rust::Box<T> : Sync,
{}

impl<T : fmt::Debug> fmt::Debug
    for Box_<T>
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

pub
type Box<T> = <T as FitForCBox>::CBoxWrapped;

pub
trait FitForCBox {
    type CBoxWrapped;
}

impl<T : Sized> FitForCBox for T {
    type CBoxWrapped = Box_<T>;
}

impl<T : Sized> FitForCBox for [T] {
    type CBoxWrapped = c_slice::Box<T>;
}

pub
trait FitForCArc {
    type CArcWrapped;
}
