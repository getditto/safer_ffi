use_prelude!();
use ::core::slice;
use crate::slice::*;

ReprC! {
    #[repr(C)]
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

impl<T> Into<rust::Vec<T>>
    for Vec<T>
{
    #[inline]
    fn into (self: Vec<T>)
      -> rust::Vec<T>
    {
        let mut this = mem::ManuallyDrop::new(self);
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
        self: &'_ mut Self,
        f: impl FnOnce(&'_ mut rust::Vec<T>) -> R,
    ) -> R
    {
        use mem::ManuallyDrop as MD;
        let this: &'_ mut MD<Self> = unsafe {
            mem::transmute(self)
        };
        let rust_vec: rust::Vec<T> =
            unsafe { MD::take(this) }
                .into()
        ;
        // f(&mut *::scopeguard::guard(rust_vec, |it| this.write(it.into())))
        return f(&mut Guard(MD::new(rust_vec), this).0);
        // where
        struct Guard<'__, T> (
            MD<rust::Vec<T>>,
            &'__ mut MD<Vec<T>>,
        );
        impl<T> Drop for Guard<'_, T> {
            fn drop (self: &'_ mut Self)
            {
                unsafe {
                    *self.1 = MD::new(MD::take(&mut self.0).into())
                }
            }
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
    $crate::Vec::from($crate::std::vec![ $($input)* ])
)}
