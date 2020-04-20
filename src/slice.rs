//! Logic common to all fat pointers.

use_prelude!();

ReprC! {
    #[repr(C)]
    /// The C layout of a (fat) pointer to a slice: a `(ptr, len)` pair.
    ///
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
        /// Pointer to the first element (if any).
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
    fn as_mut_slice<'lt> (self: slice_ptr<T>)
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
        /// [`Box`][`rust::Box`]`<[T]>` (fat pointer to a slice),
        /// but with a guaranteed `#[repr(C)]` layout.
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
        fn as_ref<'borrow> (self: &'borrow Self)
          -> slice_ref<'borrow, T>
        {
            slice_ref(self.0, PhantomCovariantLifetime::default())
        }

        #[inline]
        pub
        fn as_mut<'borrow> (self: &'borrow mut Self)
          -> slice_mut<'borrow, T>
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
                rust::Box::from_raw(
                    this.0.as_mut_slice()
                )
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
                        self.0.as_mut_slice()
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
                self.0.as_mut_slice()
            }
        }
    }

    unsafe // Safety: equivalent to that of the `where` bound
        impl<T : ReprC> Send
            for slice_boxed<T>
        where
            rust::Box<[T]> : Send,
        {}
    unsafe // Safety: equivalent to that of the `where` bound
        impl<T : ReprC> Sync
            for slice_boxed<T>
        where
            rust::Box<[T]> : Sync,
        {}
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

impl<'lt, T : 'lt + ReprC> From<&'lt [T]>
    for slice_ref<'lt, T>
{
    #[inline]
    fn from (slice: &'lt [T])
      -> slice_ref<'lt, T>
    {
        slice_ref(
            slice_ptr::from(slice),
            PhantomCovariantLifetime::default(),
        )
    }
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

impl<'lt, T : 'lt + ReprC> Copy
    for slice_ref<'lt, T>
{}

impl<'lt, T : 'lt + ReprC> Clone
    for slice_ref<'lt, T>
{
    #[inline]
    fn clone (self: &'_ slice_ref<'lt, T>)
      -> slice_ref<'lt, T>
    {
        *self
    }
}

impl<'lt, T : 'lt + ReprC> Deref
    for slice_ref<'lt, T>
{
    type Target = [T];

    #[inline]
    fn deref (self: &'_ slice_ref<'lt, T>)
      -> &'_ [T]
    {
        (*self).as_slice()
    }
}

unsafe // Safety: equivalent to that of the `where` bound
    impl<'lt, T : 'lt + ReprC> Send
        for slice_ref<'lt, T>
    where
        &'lt [T] : Send,
    {}

unsafe // Safety: equivalent to that of the `where` bound
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
        slice_ptr<T>, // /!\ not invariant /!\ ----+
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

impl<'lt, T : 'lt + ReprC> From<slice_mut<'lt, T>>
    for slice_ref<'lt, T>
{
    #[inline]
    fn from (it: slice_mut<'lt, T>)
      -> slice_ref<'lt, T>
    {
        unsafe {
            (&*it.as_slice())
                .into()
        }
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
            self.as_ref()
                .as_slice()
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
            self.as_mut()
                .as_slice()
        }
    }
}

impl<'lt, T : 'lt + ReprC> slice_mut<'lt, T> {
    #[inline]
    pub
    fn as_ref<'reborrow> (self: &'reborrow slice_mut<'lt, T>)
      -> slice_ref<'reborrow, T>
    where
        'lt : 'reborrow,
    {
        unsafe {
            self.0
                .as_slice()
                .into()
        }
    }

    #[inline]
    pub
    fn as_mut<'reborrow> (self: &'reborrow mut slice_mut<'lt, T>)
      -> slice_mut<'reborrow, T>
    where
        'lt : 'reborrow,
    {
        unsafe {
            slice_mut { .. *self }
        }
    }

    #[inline]
    pub
    fn as_slice (self: slice_mut<'lt, T>)
      -> &'lt mut [T]
    {
        unsafe {
            slice::from_raw_parts_mut(self.0.ptr.as_ptr(), self.0.len)
        }
    }
}

unsafe // Safety: equivalent to that of the `where` bound
    impl<'lt, T : 'lt + ReprC> Send
        for slice_mut<'lt, T>
    where
        &'lt mut [T] : Send,
    {}
unsafe // Safety: equivalent to that of the `where` bound
    impl<'lt, T : 'lt + ReprC> Sync
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

/// Extra traits for these `#[repr(C)]` slices.
const _: () = {
    use ::core::{
        hash::{Hash, Hasher},
        cmp::Ordering,
    };

    impl<T : ReprC + Ord> Ord
        for slice_ref<'_, T>
    {
        #[inline]
        fn cmp (self: &'_ Self, other: &'_ Self)
          -> Ordering
        {
            self[..].cmp(&other[..])
        }
    }
    impl<T : ReprC + PartialOrd> PartialOrd
        for slice_ref<'_, T>
    {
        #[inline]
        fn partial_cmp (self: &'_ Self, other: &'_ Self)
          -> Option<Ordering>
        {
            self[..].partial_cmp(&other[..])
        }
    }
    impl<T : ReprC + Eq> Eq
        for slice_ref<'_, T>
    {}
    impl<T : ReprC + PartialEq> PartialEq
        for slice_ref<'_, T>
    {
        #[inline]
        fn eq (self: &'_ Self, other: &'_ Self)
          -> bool
        {
            self[..] == other[..]
        }
    }
    impl<T : ReprC + Hash> Hash
        for slice_ref<'_, T>
    {
        #[inline]
        fn hash<H : Hasher> (self: &'_ Self, hasher: &'_ mut H)
        {
            self[..].hash(hasher)
        }
    }
    impl<T : ReprC> Default
        for slice_ref<'_, T>
    {
        #[inline]
        fn default ()
          -> Self
        {
            (&[][..]).into()
        }
    }

    impl<T : ReprC + Ord> Ord
        for slice_mut<'_, T>
    {
        #[inline]
        fn cmp (self: &'_ Self, other: &'_ Self)
          -> Ordering
        {
            self[..].cmp(&other[..])
        }
    }
    impl<T : ReprC + PartialOrd> PartialOrd
        for slice_mut<'_, T>
    {
        #[inline]
        fn partial_cmp (self: &'_ Self, other: &'_ Self)
          -> Option<Ordering>
        {
            self[..].partial_cmp(&other[..])
        }
    }
    impl<T : ReprC + Eq> Eq
        for slice_mut<'_, T>
    {}
    impl<T : ReprC + PartialEq> PartialEq
        for slice_mut<'_, T>
    {
        #[inline]
        fn eq (self: &'_ Self, other: &'_ Self)
          -> bool
        {
            self[..] == other[..]
        }
    }
    impl<T : ReprC + Hash> Hash
        for slice_mut<'_, T>
    {
        #[inline]
        fn hash<H : Hasher> (self: &'_ Self, hasher: &'_ mut H)
        {
            self[..].hash(hasher)
        }
    }
    impl<T : ReprC> Default
        for slice_mut<'_, T>
    {
        #[inline]
        fn default ()
          -> Self
        {
            (&mut [][..]).into()
        }
    }

    cfg_alloc! {
        impl<T : ReprC + Ord> Ord
            for slice_boxed<T>
        {
            #[inline]
            fn cmp (self: &'_ Self, other: &'_ Self)
              -> Ordering
            {
                self[..].cmp(&other[..])
            }
        }
        impl<T : ReprC + PartialOrd> PartialOrd
            for slice_boxed<T>
        {
            #[inline]
            fn partial_cmp (self: &'_ Self, other: &'_ Self)
              -> Option<Ordering>
            {
                self[..].partial_cmp(&other[..])
            }
        }
        impl<T : ReprC + Eq> Eq
            for slice_boxed<T>
        {}
        impl<T : ReprC + PartialEq> PartialEq
            for slice_boxed<T>
        {
            #[inline]
            fn eq (self: &'_ Self, other: &'_ Self)
              -> bool
            {
                self[..] == other[..]
            }
        }
        impl<T : ReprC + Hash> Hash
            for slice_boxed<T>
        {
            #[inline]
            fn hash<H : Hasher> (self: &'_ Self, hasher: &'_ mut H)
            {
                self[..].hash(hasher)
            }
        }
        impl<T : ReprC> Default
            for slice_boxed<T>
        {
            #[inline]
            fn default ()
              -> Self
            {
                <rust::Box<[_]>>::into(rust::Box::new([]))
            }
        }
    }
};
