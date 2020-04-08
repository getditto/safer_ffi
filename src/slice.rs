use_prelude!();

/// # C layout (for some given type T)
///
/// ```c
/// typedef struct {
///     // Cannot be NULL
///     T * ptr;
///     size_t len;
/// } slice_T;
/// ```
///
/// # Nullable pointer?
///
/// If you want to support the above typedef, but where the `ptr` field is
/// allowed to be `NULL` (with the contents of `len` then being undefined)
/// use the `Option< SlicePtr<_> >` type.
// Note: this struct is **covariant** in `T`
#[repr(C)]
pub
struct SlicePtr<T> {
    /// Pointer to the first element (if any)
    pub
    ptr: ptr::NonNull<T>, // /!\ Covariant /!\
    /// Element count
    pub
    len: size_t,
}

impl<T> Copy for SlicePtr<T> {}
impl<T> Clone for SlicePtr<T> {
    fn clone (self: &'_ Self)
      -> Self
    {
        *self
    }
}
impl<T> fmt::Debug for SlicePtr<T> {
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        fmt .debug_struct("SlicePtr")
            .field("ptr", &self.ptr)
            .field("len", &self.len)
            .finish()
    }
}
impl<T> Eq for SlicePtr<T> {}
impl<T> PartialEq for SlicePtr<T> {
    fn eq (self: &'_ Self, other: &'_ Self)
      -> bool
    {
        self.ptr == other.ptr && self.len == other.len
    }
}

impl<T> SlicePtr<T> {
    pub
    unsafe
    fn as_slice<'lt> (self: SlicePtr<T>)
      -> &'lt [T]
    where
        T : 'lt,
    {
        let Self { ptr, len } = self;
        slice::from_raw_parts(
            ptr.as_ptr(),
            len.try_into().expect("Overflow")
        )
    }

    pub
    unsafe
    fn as_slice_mut<'lt> (self: SlicePtr<T>)
      -> &'lt mut [T]
    where
        T : 'lt,
    {
        let Self { ptr, len } = self;
        slice::from_raw_parts_mut(
            ptr.as_ptr(),
            len.try_into().expect("Overflow")
        )
    }
}

impl<'lt, T : 'lt> From<&'lt [T]> for SlicePtr<T> {
    #[inline]
    fn from (slice: &'lt [T])
      -> Self
    {
        Self {
            len: slice.len().try_into().expect("Overflow"),
            ptr: unsafe {
                ptr::NonNull::new_unchecked(slice.as_ptr() as _)
            },
        }
    }
}

impl<'lt, T : 'lt> From<&'lt mut [T]> for SlicePtr<T> {
    #[inline]
    fn from (slice: &'lt mut [T])
      -> Self
    {
        Self {
            len: slice.len().try_into().expect("Overflow"),
            ptr: unsafe {
                ptr::NonNull::new_unchecked(slice.as_mut_ptr())
            },
        }
    }
}

cfg_alloc! {
    /// `rust::Box<[T]>` but with a guaranteed `#[repr(C)]` layout.
    #[derive(Debug)]
    #[repr(transparent)]
    pub
    struct BoxedSlice<T> (
        SlicePtr<T>, // Variance OK because ownership
    );

    impl<T> From<rust::Box<[T]>> for BoxedSlice<T> {
        #[inline]
        fn from (boxed_slice: rust::Box<[T]>)
          -> Self
        {
            Self(SlicePtr::from(
                &mut **mem::ManuallyDrop::new(boxed_slice)
            ))
        }
    }

    impl<T> Into<rust::Box<[T]>> for BoxedSlice<T> {
        #[inline]
        fn into (self: BoxedSlice<T>)
          -> rust::Box<[T]>
        {
            let this = mem::ManuallyDrop::new(self);
            unsafe {
                rust::Box::from_raw( this.0.as_slice_mut() )
            }
        }
    }

    impl<T> Drop for BoxedSlice<T> {
        #[inline]
        fn drop (self: &'_ mut Self)
        {
            unsafe {
                drop::<rust::Box<[T]>>(
                    rust::Box::from_raw(
                        self.0.as_slice_mut()
                    )
                );
            }
        }
    }

    impl<T> Deref for BoxedSlice<T> {
        type Target = MutSlice<'static, T>;

        #[inline]
        fn deref (self: &'_ Self)
          -> &'_ Self::Target
        {
            unsafe {
                mem::transmute(self)
            }
        }
    }
    impl<T> DerefMut for BoxedSlice<T> {
        #[inline]
        fn deref_mut (self: &'_ mut Self)
          -> &'_ mut Self::Target
        {
            unsafe {
                mem::transmute(self)
            }
        }
    }

    unsafe impl<T> Send for BoxedSlice<T>
    where
        rust::Box<[T]> : Send,
    {}
    unsafe impl<T> Sync for BoxedSlice<T>
    where
        rust::Box<[T]> : Sync,
    {}
}

/// `&'lt mut [T]` but with a guaranteed `#[repr(C)]` layout.
#[repr(transparent)]
pub
struct MutSlice<'lt, T> (
    SlicePtr<T>, // /!\ not invariant /!\ -----+
    PhantomCovariantLifetime<'lt>,          // |
    PhantomInvariant<T>, // <------------------+
);

impl<'lt, T : 'lt> From<&'lt mut [T]> for MutSlice<'lt, T> {
    #[inline]
    fn from (slice: &'lt mut [T])
      -> Self
    {
        Self(
            SlicePtr::from(slice),
            PhantomCovariantLifetime::default(),
            PhantomInvariant::<T>::default(),
        )
    }
}

impl<T> Deref for MutSlice<'_, T> {
    type Target = [T];

    #[inline]
    fn deref (self: &'_ Self)
      -> &'_ Self::Target
    {
        unsafe {
            self.0.as_slice()
        }
    }
}

impl<T> DerefMut for MutSlice<'_, T> {
    #[inline]
    fn deref_mut (self: &'_ mut Self)
      -> &'_ mut Self::Target
    {
        unsafe {
            self.0.as_slice_mut()
        }
    }
}

impl<'lt, T : 'lt> AsRef<RefSlice<'lt, T>> for MutSlice<'lt, T> {
    #[inline]
    fn as_ref (self: &'_ Self)
      -> &'_ RefSlice<'lt, T> // This would be unsound if RefSlice were Clone /!\
    {
        unsafe {
            mem::transmute(self)
        }
    }
}

unsafe impl<'lt, T : 'lt> Send for MutSlice<'lt, T>
    where &'lt mut [T] : Send
{}
unsafe impl<'lt, T : 'lt> Sync for MutSlice<'lt, T>
    where &'lt mut [T] : Sync
{}

impl<T : fmt::Debug> fmt::Debug for MutSlice<'_, T> {
    #[inline]
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        <[T] as fmt::Debug>::fmt(self, fmt)
    }
}

/// `&'lt [T]` but with a guaranteed `#[repr(C)]` layout.
#[repr(C)]
pub
struct RefSlice<'lt, T : 'lt> (
    SlicePtr<T>,
    PhantomCovariantLifetime<'lt>,
);

impl<'lt, T : 'lt> From<&'lt [T]> for RefSlice<'lt, T> {
    #[inline]
    fn from (slice: &'lt [T])
      -> Self
    {
        Self(
            SlicePtr::from(slice),
            PhantomCovariantLifetime::default(),
        )
    }
}

impl<T> Deref for RefSlice<'_, T> {
    type Target = [T];

    #[inline]
    fn deref (self: &'_ Self)
      -> &'_ Self::Target
    {
        unsafe {
            self.0.as_slice()
        }
    }
}

unsafe impl<'lt, T : 'lt> Send for RefSlice<'lt, T>
    where &'lt [T] : Send
{}
unsafe impl<'lt, T : 'lt> Sync for RefSlice<'lt, T>
    where &'lt [T] : Sync
{}

impl<T : fmt::Debug> fmt::Debug for RefSlice<'_, T> {
    #[inline]
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        <[T] as fmt::Debug>::fmt(self, fmt)
    }
}
