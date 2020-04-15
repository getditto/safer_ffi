use_prelude!();

derive_ReprC! {
    #[repr(C)]
    /// Same as [`Vec<T>`][`rust::Vec`], but with guaranteed `#[repr(C)]` layout
    pub
    struct Vec[T] where { T : ReprC } {
        ptr: ptr::NonNull<T>,
        len: usize,

        cap: usize,
    }
}

impl<T : ReprC> Vec<T> {
    #[inline]
    pub
    fn as_ref (self: &'_ Self)
      -> RefSlice<'_, T>
    {
        let &Vec { ptr, len, .. } = self;
        RefSlice(
            crate::slice::SlicePtr { ptr, len },
            PhantomCovariantLifetime::default(),
        )
    }

    #[inline]
    pub
    fn as_mut (self: &'_ mut Self)
      -> MutSlice<'_, T>
    {
        let &mut Vec { ptr, len, .. } = self;
        MutSlice(
            crate::slice::SlicePtr { ptr, len },
            PhantomCovariantLifetime::default(),
            PhantomInvariant::<T>::default(),
        )
    }
}

impl<T : ReprC> From<rust::Vec<T>>
    for Vec<T>
{
    #[inline]
    fn from (vec: rust::Vec<T>)
      -> Vec<T>
    {
        let len = vec.len()/*.try_into().expect("Overflow")*/;
        let cap = vec.capacity()/*.try_into().expect("Overflow")*/;
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

impl<T : ReprC> Into<rust::Vec<T>>
    for Vec<T>
{
    #[inline]
    fn into (self: Vec<T>)
      -> rust::Vec<T>
    {
        let &mut Self { ptr, len, cap } = &mut *mem::ManuallyDrop::new(self);
        unsafe {
            rust::Vec::from_raw_parts(
                ptr.as_ptr(),
                len/*.try_into().expect("Overflow")*/,
                cap/*.try_into().expect("Overflow")*/,
            )
        }
    }
}

impl<T : ReprC> Drop
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

impl<T : ReprC> Deref
    for Vec<T>
{
    type Target = [T];

    fn deref (self: &'_ Vec<T>)
      -> &'_ [T]
    {
        unsafe {
            slice::from_raw_parts(
                self.ptr.as_ptr(),
                self.len(),
            )
        }
    }
}
impl<T : ReprC> DerefMut
    for Vec<T>
{
    fn deref_mut (self: &'_ mut Vec<T>)
      -> &'_ mut [T]
    {
        unsafe {
            slice::from_raw_parts_mut(
                self.ptr.as_ptr(),
                self.len(),
            )
        }
    }
}

unsafe impl<T : ReprC> Send
    for Vec<T>
where
    rust::Vec<T> : Send,
{}

unsafe impl<T : ReprC> Sync
    for Vec<T>
where
    rust::Vec<T> : Sync,
{}

impl<T : ReprC> Vec<T> {
    pub
    const EMPTY: Self = Self {
        ptr: ptr::NonNull::dangling(),
        len: 0,
        cap: 0,
    };

    pub
    fn with_rust_mut<R> (
        self: &'_ mut Self,
        f: impl FnOnce(&'_ mut rust::Vec<T>) -> R,
    ) -> R
    {
        let mut s: rust::Vec<T> = mem::replace(self, Self::EMPTY).into();
        let ret = f(&mut s);
        *self = s.into();
        ret
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
