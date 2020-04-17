//! Logic common to all fat pointers.

use_prelude!();

ReprC! {
    #[repr(C)]
    /// # C layout (for some given type T)
    ///
    /// ```c
    /// typedef struct {
    ///     // Cannot be NULL
    ///     T * ptr;
    ///     uintptr_t len;
    /// } slice_T;
    /// ```
    ///
    /// # Nullable pointer?
    ///
    /// If you want to support the above typedef, but where the `ptr` field is
    /// allowed to be `NULL` (with the contents of `len` then being undefined)
    /// use the `Option< slice_ptr<_> >` type.
    pub
    struct slice_ptr[T]
    where {
        T : ReprC,
    }
    {
        /// Pointer to the first element (if any)
        pub
        ptr: ptr::NonNull<T>, // /!\ Covariant /!\

        /// Element count
        pub
        len: usize,
    }
}

impl<T : ReprC> Copy
    for slice_ptr<T>
{}
impl<T : ReprC> Clone
    for slice_ptr<T>
{
    fn clone (self: &'_ Self)
      -> Self
    {
        *self
    }
}
impl<T : ReprC> fmt::Debug
    for slice_ptr<T>
{
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        fmt .debug_struct("slice_ptr")
            .field("ptr", &self.ptr)
            .field("len", &self.len)
            .finish()
    }
}
impl<T : ReprC> Eq
    for slice_ptr<T>
{}
impl<T : ReprC> PartialEq
    for slice_ptr<T>
{
    fn eq (self: &'_ Self, other: &'_ Self)
      -> bool
    {
        self.ptr == other.ptr && self.len == other.len
    }
}

impl<T : ReprC> slice_ptr<T> {
    pub
    unsafe
    fn as_slice<'lt> (self: slice_ptr<T>)
      -> &'lt [T]
    where
        T : 'lt,
    {
        let Self { ptr, len } = self;
        slice::from_raw_parts(
            ptr.as_ptr(),
            len
        )
    }

    pub
    unsafe
    fn as_slice_mut<'lt> (self: slice_ptr<T>)
      -> &'lt mut [T]
    where
        T : 'lt,
    {
        let Self { ptr, len } = self;
        slice::from_raw_parts_mut(
            ptr.as_ptr(),
            len,
        )
    }
}

impl<'lt, T : 'lt + ReprC> From<&'lt [T]>
    for slice_ptr<T>
{
    #[inline]
    fn from (slice: &'lt [T])
      -> Self
    {
        Self {
            len: slice.len(),
            ptr: unsafe {
                ptr::NonNull::new_unchecked(slice.as_ptr() as _)
            },
        }
    }
}

impl<'lt, T : 'lt + ReprC> From<&'lt mut [T]>
    for slice_ptr<T>
{
    #[inline]
    fn from (slice: &'lt mut [T])
      -> Self
    {
        Self {
            len: slice.len(),
            ptr: unsafe {
                ptr::NonNull::new_unchecked(slice.as_mut_ptr())
            },
        }
    }
}

cfg_alloc! {
    ReprC! {
        #[repr(transparent)]
        #[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "alloc")))]
        /// `rust::Box<[T]>` but with a guaranteed `#[repr(C)]` layout.
        #[derive(Debug)]
        pub
        struct slice_boxed[T]
        where {
            T : ReprC,
        }
        (
            // Variance OK because ownership
            slice_ptr<T>,
        );
    }

    impl<T : ReprC> slice_boxed<T> {
        #[inline]
        pub
        fn as_ref (self: &'_ Self)
          -> slice_ref<'_, T>
        {
            slice_ref(self.0, PhantomCovariantLifetime::default())
        }

        #[inline]
        pub
        fn as_mut (self: &'_ mut Self)
          -> slice_mut<'_, T>
        {
            slice_mut(
                self.0,
                PhantomCovariantLifetime::default(),
                PhantomInvariant::<T>::default(),
            )
        }
    }

    impl<T : ReprC> From<rust::Box<[T]>>
        for slice_boxed<T>
    {
        #[inline]
        fn from (boxed_slice: rust::Box<[T]>)
          -> Self
        {
            Self(slice_ptr::from(
                &mut **mem::ManuallyDrop::new(boxed_slice)
            ))
        }
    }

    impl<T : ReprC> Into<rust::Box<[T]>>
        for slice_boxed<T>
    {
        #[inline]
        fn into (self: slice_boxed<T>)
          -> rust::Box<[T]>
        {
            let this = mem::ManuallyDrop::new(self);
            unsafe {
                rust::Box::from_raw( this.0.as_slice_mut() )
            }
        }
    }

    impl<T : ReprC> Drop
        for slice_boxed<T>
    {
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

    impl<T : ReprC> Deref
        for slice_boxed<T>
    {
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
    impl<T : ReprC> DerefMut
        for slice_boxed<T>
    {
        #[inline]
        fn deref_mut (self: &'_ mut Self)
          -> &'_ mut Self::Target
        {
            unsafe {
                self.0.as_slice_mut()
            }
        }
    }

    unsafe impl<T : ReprC> Send
        for slice_boxed<T>
    where
        rust::Box<[T]> : Send,
    {}
    unsafe impl<T : ReprC> Sync
        for slice_boxed<T>
    where
        rust::Box<[T]> : Sync,
    {}
}

ReprC! {
    #[repr(transparent)]
    /// `&'lt mut [T]` but with a guaranteed `#[repr(C)]` layout.
    pub
    struct slice_mut['lt, T]
    where {
        T : ReprC + 'lt,
    }
    (
        pub(in crate)
        slice_ptr<T>, // /!\ not invariant /!\ -----+
                                                // |
        pub(in crate)                           // |
        PhantomCovariantLifetime<'lt>,          // |
                                                // |
        pub(in crate)                           // |
        PhantomInvariant<T>, // <------------------+
    );
}

impl<'lt, T : 'lt + ReprC> From<&'lt mut [T]>
    for slice_mut<'lt, T>
{
    #[inline]
    fn from (slice: &'lt mut [T])
      -> Self
    {
        Self(
            slice_ptr::from(slice),
            PhantomCovariantLifetime::default(),
            PhantomInvariant::<T>::default(),
        )
    }
}

impl<T : ReprC> Deref
    for slice_mut<'_, T>
{
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

impl<T : ReprC> DerefMut
    for slice_mut<'_, T>
{
    #[inline]
    fn deref_mut (self: &'_ mut Self)
      -> &'_ mut Self::Target
    {
        unsafe {
            self.0.as_slice_mut()
        }
    }
}

impl<'lt, T : 'lt + ReprC> AsRef<slice_ref<'lt, T>>
    for slice_mut<'lt, T>
{
    #[inline]
    fn as_ref (self: &'_ Self)
      -> &'_ slice_ref<'lt, T> // This would be unsound if slice_ref were Clone /!\
    {
        unsafe {
            mem::transmute(self)
        }
    }
}

unsafe impl<'lt, T : 'lt + ReprC> Send
    for slice_mut<'lt, T>
where
    &'lt mut [T] : Send,
{}
unsafe impl<'lt, T : 'lt + ReprC> Sync
    for slice_mut<'lt, T>
where
    &'lt mut [T] : Sync,
{}

impl<T : fmt::Debug + ReprC> fmt::Debug
    for slice_mut<'_, T>
{
    #[inline]
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        <[T] as fmt::Debug>::fmt(self, fmt)
    }
}

ReprC! {
    #[repr(transparent)]
    /// `&'lt [T]` but with a guaranteed `#[repr(C)]` layout.
    pub
    struct slice_ref['lt, T]
    where {
        T : ReprC + 'lt,
    }
    (
        pub(in crate)
        slice_ptr<T>,

        pub(in crate)
        PhantomCovariantLifetime<'lt>,
    );
}

impl<'lt, T : ReprC> slice_ref<'lt, T> {
    pub
    fn as_slice (self: slice_ref<'lt, T>)
      -> &'lt [T]
    {
        unsafe {
            self.0.as_slice()
        }
    }
}

impl<'lt, T : 'lt + ReprC> From<&'lt [T]>
    for slice_ref<'lt, T>
{
    #[inline]
    fn from (slice: &'lt [T])
      -> Self
    {
        Self(
            slice_ptr::from(slice),
            PhantomCovariantLifetime::default(),
        )
    }
}

impl<T : ReprC> Deref
    for slice_ref<'_, T>
{
    type Target = [T];

    #[inline]
    fn deref (self: &'_ Self)
      -> &'_ Self::Target
    {
        (*self).as_slice()
    }
}

unsafe
    impl<'lt, T : 'lt + ReprC> Send
        for slice_ref<'lt, T>
    where
        &'lt [T] : Send,
    {}

unsafe
    impl<'lt, T : 'lt + ReprC> Sync
        for slice_ref<'lt, T>
    where
        &'lt [T] : Sync,
    {}

impl<T : fmt::Debug + ReprC> fmt::Debug
    for slice_ref<'_, T>
{
    #[inline]
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        <[T] as fmt::Debug>::fmt(self, fmt)
    }
}

impl<T : ReprC> Copy
    for slice_ref<'_, T>
{}
impl<T : ReprC> Clone
    for slice_ref<'_, T>
{
    #[inline]
    fn clone (self: &'_ Self)
      -> Self
    {
        *self
    }
}
