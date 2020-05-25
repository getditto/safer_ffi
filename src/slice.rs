//! Logic common to all fat pointers.

use_prelude!();
use ::core::slice;

#[doc(no_inline)]
pub use self::{
    slice_ref as Ref,
    slice_mut as Mut,
};
cfg_alloc! {
    #[doc(no_inline)]
    pub use slice_boxed as Box;
}

/// The phantoms from the crate are not `ReprC`.
type PhantomCovariantLifetime<'lt> =
    PhantomData<&'lt ()>
;

ReprC! {
    #[repr(C)]
    /// Like [`slice_ref`] and [`slice_mut`], but with any lifetime attached
    /// whatsoever.
    ///
    /// It is only intended to be used as the parameter of a **callback** that
    /// locally borrows it, due to limitations of the [`ReprC`][
    /// `trait@crate::layout::ReprC`] design _w.r.t._ higher-rank trait bounds.
    ///
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
    /// use the `Option< slice_ptr<_> >` type.
    #[derive(Debug)]
    pub
    struct slice_raw[T] {
        /// Pointer to the first element (if any).
        pub
        ptr: ptr::NonNull<T>,

        /// Element count
        pub
        len: usize,
    }
}

impl<T> slice_raw<T> {
    /// # Safety
    ///
    ///   - For the duration of the `'borrow`, the pointer must point to the
    ///     beginning of a valid and immutable null-terminated slice of
    ///     `len` `T`s.
    #[inline]
    pub
    unsafe
    fn as_ref<'borrow> (self: &'borrow slice_raw<T>)
      -> slice_ref<'borrow, T>
    {
        slice_ref {
            ptr: self.ptr.into(),
            len: self.len,
            _lt: PhantomCovariantLifetime::default(),
        }
    }

    /// # Safety
    ///
    ///   - For the duration of the `'borrow`, the pointer must point to the
    ///     beginning of a valid and immutable null-terminated slice of
    ///     `len` `T`s.
    #[inline]
    pub
    unsafe
    fn as_mut<'borrow> (self: &'borrow mut slice_raw<T>)
      -> slice_mut<'borrow, T>
    {
        slice_mut {
            ptr: self.ptr.into(),
            len: self.len,
            _lt: PhantomCovariantLifetime::default(),
        }
    }
}

cfg_alloc! {
    ReprC! {
        #[repr(C)]
        #[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "alloc")))]
        /// [`Box`][`rust::Box`]`<[T]>` (fat pointer to a slice),
        /// but with a guaranteed `#[repr(C)]` layout.
        ///
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
        /// use the `Option< slice_ptr<_> >` type.
        #[derive(Debug)]
        pub
        struct slice_boxed[T] {
            /// Pointer to the first element (if any).
            pub(in crate)
            ptr: ptr::NonNullOwned<T>,

            /// Element count
            pub(in crate)
            len: usize,
        }
    }

    impl<T> slice_boxed<T> {
        #[inline]
        pub
        fn as_ref<'borrow> (self: &'borrow Self)
          -> slice_ref<'borrow, T>
        {
            Into::into(&self[..])
        }

        #[inline]
        pub
        fn as_mut<'borrow> (self: &'borrow mut Self)
          -> slice_mut<'borrow, T>
        {
            Into::into(&mut self[..])
        }

        #[inline]
        pub
        fn as_slice<'borrow> (self: &'borrow Self)
          -> &'borrow [T]
        {
            self.as_ref().as_slice()
        }

        #[inline]
        pub
        fn as_slice_mut<'borrow> (self: &'borrow mut Self)
          -> &'borrow mut [T]
        {
            self.as_mut().as_slice()
        }
    }

    impl<T> From<rust::Box<[T]>>
        for slice_boxed<T>
    {
        #[inline]
        fn from (boxed_slice: rust::Box<[T]>)
          -> Self
        {
            slice_boxed {
                len: boxed_slice.len(),
                ptr: unsafe {
                    ptr::NonNull::new_unchecked(
                        rust::Box::leak(boxed_slice).as_mut_ptr()
                    )
                }.into(),
            }
        }
    }

    impl<T> Into<rust::Box<[T]>>
        for slice_boxed<T>
    {
        #[inline]
        fn into (self: slice_boxed<T>)
          -> rust::Box<[T]>
        {
            let mut this = mem::ManuallyDrop::new(self);
            unsafe {
                rust::Box::from_raw(
                    slice::from_raw_parts_mut(
                        this.ptr.as_mut_ptr(),
                        this.len,
                    )
                )
            }
        }
    }

    impl<T> Drop
        for slice_boxed<T>
    {
        #[inline]
        fn drop (self: &'_ mut Self)
        {
            unsafe {
                drop::<rust::Box<[T]>>(
                    rust::Box::from_raw(
                        slice::from_raw_parts_mut(
                            self.ptr.as_mut_ptr(),
                            self.len,
                        )
                    )
                );
            }
        }
    }

    impl<T> Deref
        for slice_boxed<T>
    {
        type Target = [T];

        #[inline]
        fn deref (self: &'_ Self)
          -> &'_ Self::Target
        {
            unsafe {
                slice::from_raw_parts(self.ptr.as_ptr(), self.len)
            }
        }
    }
    impl<T> DerefMut
        for slice_boxed<T>
    {
        #[inline]
        fn deref_mut (self: &'_ mut Self)
          -> &'_ mut Self::Target
        {
            unsafe {
                slice::from_raw_parts_mut(self.ptr.as_mut_ptr(), self.len)
            }
        }
    }

    unsafe // Safety: equivalent to that of the `where` bound
        impl<T> Send
            for slice_boxed<T>
        where
            rust::Box<[T]> : Send,
        {}
    unsafe // Safety: equivalent to that of the `where` bound
        impl<T> Sync
            for slice_boxed<T>
        where
            rust::Box<[T]> : Sync,
        {}
}

ReprC! {
    #[repr(C)]
    /// `&'lt [T]` but with a guaranteed `#[repr(C)]` layout.
    ///
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
    /// use the `Option< slice_ptr<_> >` type.
    pub
    struct slice_ref['lt, T]
    where {
        T : 'lt,
    }
    {
        /// Pointer to the first element (if any).
        pub(in crate)
        ptr: ptr::NonNullRef<T>,

        /// Element count
        pub(in crate)
        len: usize,

        pub(in crate)
        _lt: PhantomCovariantLifetime<'lt>,
    }
}

impl<'lt, T : 'lt> From<&'lt [T]>
    for slice_ref<'lt, T>
{
    #[inline]
    fn from (slice: &'lt [T])
      -> slice_ref<'lt, T>
    {
        slice_ref {
            len: slice.len(),
            ptr: unsafe {
                ptr::NonNull::new_unchecked(slice.as_ptr() as _)
            }.into(),
            _lt: PhantomCovariantLifetime::default(),
        }
    }
}

impl<'lt, T : 'lt> slice_ref<'lt, T> {
    pub
    fn as_slice (self: slice_ref<'lt, T>)
      -> &'lt [T]
    {
        unsafe {
            slice::from_raw_parts(self.ptr.as_ptr(), self.len)
        }
    }
}

impl<'lt, T : 'lt> Copy
    for slice_ref<'lt, T>
{}

impl<'lt, T : 'lt> Clone
    for slice_ref<'lt, T>
{
    #[inline]
    fn clone (self: &'_ slice_ref<'lt, T>)
      -> slice_ref<'lt, T>
    {
        *self
    }
}

impl<'lt, T : 'lt> Deref
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
    impl<'lt, T : 'lt> Send
        for slice_ref<'lt, T>
    where
        &'lt [T] : Send,
    {}

unsafe // Safety: equivalent to that of the `where` bound
    impl<'lt, T : 'lt> Sync
        for slice_ref<'lt, T>
    where
        &'lt [T] : Sync,
    {}

impl<T : fmt::Debug> fmt::Debug
    for slice_ref<'_, T>
{
    #[inline]
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        <[T] as fmt::Debug>::fmt(self, fmt)
    }
}

impl<'lt, T : 'lt> From<slice_ref<'lt, T>>
    for slice_raw<T>
{
    #[inline]
    fn from (slice_ref { ptr, len, .. }: slice_ref<'lt, T>)
      -> slice_raw<T>
    {
        slice_raw { ptr: ptr.0, len }
    }
}

ReprC! {
    #[repr(C)]
    /// `&'lt mut [T]` but with a guaranteed `#[repr(C)]` layout.
    ///
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
    /// use the `Option< slice_ptr<_> >` type.
    pub
    struct slice_mut['lt, T]
    where {
        T : 'lt,
    }
    {
        /// Pointer to the first element (if any).
        pub(in crate)
        ptr: ptr::NonNullMut<T>,

        /// Element count
        pub(in crate)
        len: usize,

        pub(in crate)
        _lt: PhantomCovariantLifetime<'lt>,
    }
}

impl<'lt, T : 'lt> From<&'lt mut [T]>
    for slice_mut<'lt, T>
{
    #[inline]
    fn from (slice: &'lt mut [T])
      -> Self
    {
        slice_mut {
            len: slice.len(),
            ptr: unsafe {
                ptr::NonNull::new_unchecked(slice.as_mut_ptr())
            }.into(),
            _lt: PhantomCovariantLifetime::default(),
        }
    }
}

impl<'lt, T : 'lt> From<slice_mut<'lt, T>>
    for slice_ref<'lt, T>
{
    #[inline]
    fn from (it: slice_mut<'lt, T>)
      -> slice_ref<'lt, T>
    {
        (&*it.as_slice())
            .into()
    }
}

impl<T> Deref
    for slice_mut<'_, T>
{
    type Target = [T];

    #[inline]
    fn deref (self: &'_ Self)
      -> &'_ Self::Target
    {
        self.as_ref()
            .as_slice()
    }
}

impl<T> DerefMut
    for slice_mut<'_, T>
{
    #[inline]
    fn deref_mut (self: &'_ mut Self)
      -> &'_ mut Self::Target
    {
        self.as_mut()
            .as_slice()
    }
}

impl<'lt, T : 'lt> slice_mut<'lt, T> {
    #[inline]
    pub
    fn as_ref<'reborrow> (self: &'reborrow slice_mut<'lt, T>)
      -> slice_ref<'reborrow, T>
    where
        'lt : 'reborrow,
    {
        let &slice_mut { ptr: ptr::NonNullMut(ptr, ..), len, _lt } = self;
        slice_ref {
            ptr: ptr.into(),
            len,
            _lt,
        }
    }

    #[inline]
    pub
    fn as_mut<'reborrow> (self: &'reborrow mut slice_mut<'lt, T>)
      -> slice_mut<'reborrow, T>
    where
        'lt : 'reborrow,
    {
        let &mut slice_mut { ref mut ptr, len, _lt } = self;
        slice_mut {
            ptr: ptr.copy(),
            len,
            _lt,
        }
    }

    #[inline]
    pub
    fn as_slice (mut self: slice_mut<'lt, T>)
      -> &'lt mut [T]
    {
        unsafe {
            slice::from_raw_parts_mut(self.ptr.as_mut_ptr(), self.len)
        }
    }
}

unsafe // Safety: equivalent to that of the `where` bound
    impl<'lt, T : 'lt> Send
        for slice_mut<'lt, T>
    where
        &'lt mut [T] : Send,
    {}
unsafe // Safety: equivalent to that of the `where` bound
    impl<'lt, T : 'lt> Sync
        for slice_mut<'lt, T>
    where
        &'lt mut [T] : Sync,
    {}

impl<T : fmt::Debug> fmt::Debug
    for slice_mut<'_, T>
{
    #[inline]
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        <[T] as fmt::Debug>::fmt(self, fmt)
    }
}

impl<'lt, T : 'lt> From<slice_mut<'lt, T>>
    for slice_raw<T>
{
    #[inline]
    fn from (slice_mut { ptr, len, .. }: slice_mut<'lt, T>)
      -> slice_raw<T>
    {
        slice_raw { ptr: ptr.0, len }
    }
}

/// Extra traits for these `#[repr(C)]` slices.
const _: () = {
    use ::core::{
        hash::{Hash, Hasher},
        cmp::Ordering,
    };

    impl<T : Ord> Ord
        for slice_ref<'_, T>
    {
        #[inline]
        fn cmp (self: &'_ Self, other: &'_ Self)
          -> Ordering
        {
            self[..].cmp(&other[..])
        }
    }
    impl<T : PartialOrd> PartialOrd
        for slice_ref<'_, T>
    {
        #[inline]
        fn partial_cmp (self: &'_ Self, other: &'_ Self)
          -> Option<Ordering>
        {
            self[..].partial_cmp(&other[..])
        }
    }
    impl<T : Eq> Eq
        for slice_ref<'_, T>
    {}
    impl<T : PartialEq> PartialEq
        for slice_ref<'_, T>
    {
        #[inline]
        fn eq (self: &'_ Self, other: &'_ Self)
          -> bool
        {
            self[..] == other[..]
        }
    }
    impl<T : Hash> Hash
        for slice_ref<'_, T>
    {
        #[inline]
        fn hash<H : Hasher> (self: &'_ Self, hasher: &'_ mut H)
        {
            self[..].hash(hasher)
        }
    }
    impl<T> Default
        for slice_ref<'_, T>
    {
        #[inline]
        fn default ()
          -> Self
        {
            (&[][..]).into()
        }
    }

    impl<T : Ord> Ord
        for slice_mut<'_, T>
    {
        #[inline]
        fn cmp (self: &'_ Self, other: &'_ Self)
          -> Ordering
        {
            self[..].cmp(&other[..])
        }
    }
    impl<T : PartialOrd> PartialOrd
        for slice_mut<'_, T>
    {
        #[inline]
        fn partial_cmp (self: &'_ Self, other: &'_ Self)
          -> Option<Ordering>
        {
            self[..].partial_cmp(&other[..])
        }
    }
    impl<T : Eq> Eq
        for slice_mut<'_, T>
    {}
    impl<T : PartialEq> PartialEq
        for slice_mut<'_, T>
    {
        #[inline]
        fn eq (self: &'_ Self, other: &'_ Self)
          -> bool
        {
            self[..] == other[..]
        }
    }
    impl<T : Hash> Hash
        for slice_mut<'_, T>
    {
        #[inline]
        fn hash<H : Hasher> (self: &'_ Self, hasher: &'_ mut H)
        {
            self[..].hash(hasher)
        }
    }
    impl<T> Default
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
        impl<T : Ord> Ord
            for slice_boxed<T>
        {
            #[inline]
            fn cmp (self: &'_ Self, other: &'_ Self)
              -> Ordering
            {
                self[..].cmp(&other[..])
            }
        }
        impl<T : PartialOrd> PartialOrd
            for slice_boxed<T>
        {
            #[inline]
            fn partial_cmp (self: &'_ Self, other: &'_ Self)
              -> Option<Ordering>
            {
                self[..].partial_cmp(&other[..])
            }
        }
        impl<T : Eq> Eq
            for slice_boxed<T>
        {}
        impl<T : PartialEq> PartialEq
            for slice_boxed<T>
        {
            #[inline]
            fn eq (self: &'_ Self, other: &'_ Self)
              -> bool
            {
                self[..] == other[..]
            }
        }
        impl<T : Hash> Hash
            for slice_boxed<T>
        {
            #[inline]
            fn hash<H : Hasher> (self: &'_ Self, hasher: &'_ mut H)
            {
                self[..].hash(hasher)
            }
        }
        impl<T> Default
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
