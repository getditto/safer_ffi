use_prelude!();

/// Same as [`Vec<T>`][`rust::Vec`], but with guaranteed `#[repr(C)]` layout
#[repr(C)]
pub struct Vec<T> {
    ptr: ptr::NonNull<T>,
    len: size_t,

    cap: size_t,
}

impl<T> From<rust::Vec<T>> for Vec<T> {
    #[inline]
    fn from (vec: rust::Vec<T>) -> Vec<T>
    {
        let len = vec.len().try_into().expect("Overflow");
        let cap = vec.capacity().try_into().expect("Overflow");
        let ptr = mem::ManuallyDrop::new(vec).as_mut_ptr();
        Self {
            ptr: unsafe {
                ptr::NonNull::new_unchecked(ptr)
            },
            len,
            cap,
        }
    }
}

impl<T> Into<rust::Vec<T>> for Vec<T> {
    #[inline]
    fn into (self: Vec<T>) -> rust::Vec<T>
    {
        let &mut Self { ptr, len, cap } = &mut *mem::ManuallyDrop::new(self);
        unsafe {
            rust::Vec::from_raw_parts(
                ptr.as_ptr(),
                len.try_into().expect("Overflow"),
                cap.try_into().expect("Overflow"),
            )
        }
    }
}

impl<T> Drop for Vec<T> {
    #[inline]
    fn drop (self: &'_ mut Self)
    {
        unsafe {
            drop::<rust::Vec<T>>(
                ptr::read(self) // ManuallyDrop::take()
                    .into()
            )
        }
    }
}

impl<T> Deref for Vec<T> {
    type Target = MutSlice<'static, T>;

    fn deref (self: &'_ Self) -> &'_ Self::Target
    {
        unsafe {
            // Safety: this relies on Vec fields being
            // ordered as: ptr, len first.
            mem::transmute(self)
        }
    }
}
impl<T> DerefMut for Vec<T> {
    fn deref_mut (self: &'_ mut Self) -> &'_ mut Self::Target
    {
        unsafe {
            // Safety: this relies on Vec fields being
            // ordered as: ptr, len first.
            mem::transmute(self)
        }
    }
}

unsafe impl<T> Send for Vec<T> where
    rust::Vec<T> : Send,
{}

unsafe impl<T> Sync for Vec<T> where
    rust::Vec<T> : Sync,
{}

impl<T> Vec<T> {
    pub
    const EMPTY: Self = Self {
        ptr: ptr::NonNull::dangling(),
        len: 0,
        cap: 0,
    };

    pub
    fn with_rust_mut<R, F> (self: &'_ mut Self, f: F) -> R
    where
        F : FnOnce(&'_ mut rust::Vec<T>) -> R
    {
        let mut s: rust::Vec<T> = mem::replace(self, Self::EMPTY).into();
        let ret = f(&mut s);
        *self = s.into();
        ret
    }
}

impl<T : fmt::Debug> fmt::Debug for Vec<T> {
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        <[T] as fmt::Debug>::fmt(&self[..], fmt)
    }
}

