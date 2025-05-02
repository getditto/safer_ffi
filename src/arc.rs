//! `#[repr(C)]` [`Arc`][`rust::Arc`]ed types.

use_prelude!();

ReprC! {
    #[repr(transparent)]
    pub
    struct Arc_[T] (
        ptr::NonNullOwned<T>,
    );
}

impl<T> From<rust::Arc<T>> for Arc_<T> {
    #[inline]
    fn from(arced: rust::Arc<T>) -> Arc_<T> {
        let raw = rust::Arc::into_raw(arced);
        Self(ptr::NonNull::from(unsafe { &*raw }).into())
    }
}

impl<T> Arc_<T> {
    #[inline]
    pub fn new(value: T) -> Self {
        rust::Arc::new(value).into()
    }

    #[inline]
    pub fn into(self: Arc_<T>) -> rust::Arc<T> {
        let mut this = mem::ManuallyDrop::new(self);
        unsafe { rust::Arc::from_raw(this.0.as_mut_ptr()) }
    }
}

impl<T> Drop for Arc_<T> {
    #[inline]
    fn drop(self: &'_ mut Arc_<T>) {
        unsafe {
            drop::<rust::Arc<T>>(rust::Arc::from_raw(self.0.as_mut_ptr()));
        }
    }
}

impl<T> Deref for Arc_<T> {
    type Target = T;

    #[inline]
    fn deref(self: &'_ Arc_<T>) -> &'_ T {
        unsafe { &*self.0.as_ptr() }
    }
}

unsafe impl<T> Send for Arc_<T> where rust::Arc<T>: Send {}

unsafe impl<T> Sync for Arc_<T> where rust::Arc<T>: Sync {}

impl<T> Clone for Arc_<T> {
    #[inline]
    fn clone(self: &'_ Self) -> Self {
        let raw = self.0.as_ptr() as *mut T;
        unsafe {
            alloc::sync::Arc::increment_strong_count(raw);
        }
        Arc_(ptr::NonNull::from(unsafe { &*raw }).into())
    }
}

impl<T: fmt::Debug> fmt::Debug for Arc_<T> {
    fn fmt(
        self: &'_ Self,
        fmt: &'_ mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        T::fmt(self, fmt)
    }
}

pub type Arc<T> = <T as FitForCArc>::CArcWrapped;

pub trait FitForCArc {
    type CArcWrapped;
}

impl<T: Sized> FitForCArc for T {
    type CArcWrapped = Arc_<T>;
}
