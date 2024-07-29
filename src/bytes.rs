use core::{
    borrow::Borrow,
    fmt::Debug,
    hash::Hash,
    ops::{Deref, RangeBounds},
    ptr::NonNull,
};

#[cfg(feature = "alloc")]
use alloc::sync::Arc;
use safer_ffi_proc_macros::derive_ReprC;

/// A slice of bytes optimized for sharing ownership of it and its subslices.
///
/// Typically, [`Bytes`] can constructed from `&'static [u8]`, `Arc<[u8]>` or `Arc<T: AsRef<[u8]>>`.
///
/// [`Bytes`] can also "intern" small enough slices: that is, if the slice is more than one byte smaller than
/// [`Bytes`] memory layout (which is 40 bytes on 64bit architectures), it may be store directly in that memory
/// instead of through indirection.
#[derive_ReprC]
#[repr(C)]
#[cfg_attr(feature = "stabby", stabby::stabby)]
pub struct Bytes<'a> {
    start: *const u8,
    len: usize,
    data: *const (),
    capacity: usize,
    vtable: NonNull<BytesVt>,
    marker: core::marker::PhantomData<&'a [u8]>,
}
#[cfg(not(feature = "stabby"))]
unsafe impl<'a> crate::layout::__HasNiche__ for Bytes<'a> {
    fn is_niche(it: &'_ <Self as crate::ඞ::ReprC>::CLayout) -> bool {
        !it.vtable.is_null()
    }
}
#[cfg(feature = "stabby")]
unsafe impl<'a> crate::layout::__HasNiche__ for Bytes<'a>
where
    Self: stabby::IStable<HasExactlyOneNiche = stabby::abi::B1>,
{
    fn is_niche(it: &'_ <Self as crate::ඞ::ReprC>::CLayout) -> bool {
        !it.vtable.is_null()
    }
}

const _: () = {
    #[cfg(feature = "stabby")]
    const fn check_single_niche<
        T: stabby::IStable<HasExactlyOneNiche = stabby::abi::B1> + crate::layout::__HasNiche__,
    >() -> u64 {
        T::ID
    }
    #[cfg(not(feature = "stabby"))]
    const fn check_single_niche<T: crate::layout::__HasNiche__>() -> () {}
    _ = check_single_niche::<Bytes<'static>>();
};

extern "C" fn noop(_: *const (), _: usize) {}

impl<'a> Bytes<'a> {
    /// Constructs an empty slice.
    ///
    /// Since [`Bytes`] are immutable slices, this mostly serves to create default values.
    pub const fn empty() -> Self {
        Self::from_static([].as_slice())
    }
    const fn is_interned(&self) -> bool {
        (unsafe {
            (self as *const Self)
                .cast::<u8>()
                .add(core::mem::size_of::<Self>() - 1)
                .read()
        } & 1)
            != 0
    }
    fn vtable(&self) -> Option<&'a BytesVt> {
        if self.is_interned() {
            None
        } else {
            unsafe { Some(self.vtable.as_ref()) }
        }
    }

    /// Constructs a [`Bytes`] referring to static data.
    ///
    /// This is equivalent to `<Bytes as From<&'static [u8]>>::from`, guaranteeing that [`Self::upgrade`] won't need to reallocate to recover the `'static` lifetime through [`Bytes::upgrade`].
    /// ```
    /// # use safer_ffi::bytes::Bytes;
    /// let data = "Hello there, this string is long enough that it'll cross the inline-threshold (core::mem::size_of::<Bytes>() - 1) on all supported platforms";
    /// let mut bytes = Bytes::from_static(data.as_bytes());
    /// assert!(!bytes.upgrade_will_allocate());
    /// ```
    pub const fn from_static(data: &'static [u8]) -> Self {
        const VT: BytesVt = BytesVt {
            retain: Some(noop),
            release: Some(noop),
        };
        Self {
            start: data.as_ptr().cast_mut(),
            len: data.len(),
            data: data.as_ptr().cast(),
            capacity: data.len(),
            vtable: unsafe {
                NonNull::new_unchecked(&VT as &'static BytesVt as *const BytesVt as *mut BytesVt)
            },
            marker: core::marker::PhantomData,
        }
    }

    /// Constructs a [`Bytes`] referring to a slice.
    ///
    /// Unlike [`Bytes::from_static`] and `<Bytes as From<&'static ..>>::from` implementations, the resulting [`Bytes`] cannot tell that it's referring to `'static` data even if it is, and will therefore perform a copy when [`Bytes::upgrade`]ing.
    /// ```
    /// # use safer_ffi::bytes::Bytes;
    /// let data = "Hello there, this string is long enough that it'll cross the inline-threshold (core::mem::size_of::<Bytes>() - 1) on all supported platforms";
    /// let mut bytes = Bytes::from_slice(data.as_bytes());
    /// assert!(bytes.upgrade_will_allocate());
    /// let data = "This string isn't";
    /// let mut bytes = Bytes::from_slice(data.as_bytes());
    /// assert!(!bytes.upgrade_will_allocate());
    /// ```
    ///
    /// Note that if the slice is small enough to be interned in the [`Bytes`], it'll be, allowing for free upgrades. You may use the [`Bytes::intern_slice`] constructor instead of this one if you want to be able to handle interning not being possible.
    pub fn from_slice(data: &'a [u8]) -> Self {
        static VT: BytesVt = BytesVt {
            release: None,
            retain: Some(noop),
        };
        Self::intern_slice(data).unwrap_or_else(|| Self {
            start: data.as_ptr().cast(),
            len: data.len(),
            data: data.as_ptr().cast(),
            capacity: data.len(),
            vtable: unsafe {
                NonNull::new_unchecked(&VT as &'static BytesVt as *const BytesVt as *mut BytesVt)
            },
            marker: core::marker::PhantomData,
        })
    }

    /// Constructs a [`Bytes`] from a short slice by interning it, untying [`Bytes`]'s lifetime from `slice`'s.
    ///
    /// If the slice is too long to be interned, this constructor returns `None` instead.
    ///
    /// A slice may be interned if it is at least one byte smaller than [`Bytes`], which has the same size as `[usize; 5]`.
    ///
    /// ```
    /// # use safer_ffi::bytes::Bytes;
    /// let interned = Bytes::intern_slice("Hi".as_bytes()).unwrap();
    /// assert_eq!(interned.as_slice(), "Hi".as_bytes());
    /// assert!(!interned.clone_will_allocate());
    /// assert!(!interned.upgrade_will_allocate());
    /// assert!(Bytes::intern_slice("This slice is too long to intern, even on architectures with 128 bit pointer-size.".as_bytes()).is_none())
    /// ```
    pub fn intern_slice(slice: &[u8]) -> Option<Bytes<'static>> {
        let mut buffer = [0; core::mem::size_of::<Self>()];
        if slice.len() >= buffer.len() {
            return None;
        }
        buffer[..slice.len()].copy_from_slice(slice);
        buffer[buffer.len() - 1] = 1 | ((slice.len() as u8) << 1);
        unsafe { core::mem::transmute(buffer) }
    }

    /// Slices `self` in-place:
    /// ```
    /// # use safer_ffi::bytes::Bytes;
    /// let data = b"Hello there";
    /// let mut bytes = Bytes::from(data);
    /// bytes.shrink_to(3..7);
    /// assert_eq!(&data[3..7], bytes.as_slice());
    /// ```
    /// # Panics
    /// If the range's end is out of bounds, or if the range's start is greater than its end.
    pub fn shrink_to<R: RangeBounds<usize>>(&mut self, range: R) {
        let start = match range.start_bound() {
            core::ops::Bound::Included(i) => *i,
            core::ops::Bound::Excluded(i) => *i + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(i) => *i + 1,
            core::ops::Bound::Excluded(i) => *i,
            core::ops::Bound::Unbounded => self.len(),
        };
        assert!(start <= end);
        assert!(end <= self.len);
        let len = end - start;
        if len < core::mem::size_of::<Self>() {
            *self = Self::intern_slice(&self[start..end]).unwrap()
        } else {
            self.start = unsafe { self.start.add(start) };
            self.len = len;
        }
    }

    /// Convenience around [`Self::shrink_to`] for better method chaining.
    /// ```
    /// # use safer_ffi::bytes::Bytes;
    /// let data = b"Hello there".as_slice();
    /// let bytes = Bytes::from_static(data);
    /// assert_eq!(&data[3..7], bytes.subsliced(3..7).as_slice());
    /// ```
    /// # Panics
    /// If the range's end is out of bounds, or if the range's start is greater than its end.
    pub fn subsliced<R: RangeBounds<usize>>(mut self, range: R) -> Self {
        self.shrink_to(range);
        self
    }

    #[cfg(feature = "alloc")]
    /// Splits the slice at `index`.
    /// ```
    /// # use safer_ffi::bytes::Bytes;
    /// let data = b"Hello there".as_slice();
    /// let index = 5;
    /// let (l, r) = data.split_at(index);
    /// let (bl, br) = Bytes::from_static(data).split_at(index).unwrap();
    /// assert_eq!((l, r), (bl.as_slice(), br.as_slice()));
    /// ```
    ///
    /// If re-allocating was necessary to create a second owner, both returned subslices will refer to a common buffer.
    ///
    /// # Errors
    /// Returns `self` if `index` is out of bounds.
    pub fn split_at(self, index: usize) -> Result<(Self, Self), Self> {
        if index <= self.len() {
            if index < core::mem::size_of::<Self>() {
                let left = Self::intern_slice(&self[..index]).unwrap();
                Ok((left, self.subsliced(index..)))
            } else {
                if self.len() - index < core::mem::size_of::<Self>() {
                    let right = Self::intern_slice(&self[index..]).unwrap();
                    Ok((self.subsliced(..index), right))
                } else {
                    let mut left = self.clone();
                    let mut right = left.clone();
                    left.len = index;
                    right.len -= index;
                    right.start = unsafe { right.start.add(index) };
                    Ok((left, right))
                }
            }
        } else {
            Err(self)
        }
    }

    /// Returns the length of the slice.
    pub const fn len(&self) -> usize {
        if self.is_interned() {
            (unsafe { core::mem::transmute::<_, [u8; core::mem::size_of::<usize>()]>(self.vtable) }
                [core::mem::size_of::<usize>() - 1] as usize)
                >> 1
        } else {
            self.len
        }
    }

    /// Returns the slice's contents.
    pub const fn as_slice(&self) -> &[u8] {
        if self.is_interned() {
            unsafe { core::slice::from_raw_parts(self as *const Self as *const u8, self.len()) }
        } else {
            unsafe { core::slice::from_raw_parts(self.start, self.len) }
        }
    }

    #[cfg(any(feature = "alloc", feature = "stabby"))]
    /// Copies the slice into an `stabby::sync::ArcSlice<u8>` if `stabby` is enabled or a `Arc<[u8]>` otherwise before wrapping it in [`Bytes`].
    ///
    /// If the slice is small enough to do so, it will be [interned](Bytes::intern_slice) instead, saving the need to allocate.
    pub fn copied_from_slice(slice: &[u8]) -> Bytes<'static> {
        match_cfg! {
            feature = "stabby" => {
                use ::stabby::sync::ArcSlice;
            },
            _ => {
                type ArcSlice<T> = Arc<[T]>;
            },
        }
        Self::intern_slice(slice).unwrap_or_else(|| ArcSlice::<u8>::from(slice).into())
    }

    #[cfg(any(feature = "alloc", feature = "stabby"))]
    /// Proves that the slice can be held onto for arbitrary durations, or copies it into a new `Arc<[u8]>` that does.
    ///
    /// Note that `feature = "stabby"` being enabled will cause a `stabby::sync::ArcSlice<u8>` to be used instead of `Arc<[u8]>`.
    pub fn upgrade(self: Bytes<'a>) -> Bytes<'static> {
        if self.vtable().map_or(true, |vt| !vt.is_borrowed()) {
            return unsafe { core::mem::transmute(self) };
        }
        Self::copied_from_slice(&self)
    }

    /// Attempts to prove that the slice has a static lifetime.
    /// # Errors
    /// Returns the original instance if it couldn't be proven to be `'static`.
    pub fn noalloc_upgrade(self: Bytes<'a>) -> Result<Bytes<'static>, Self> {
        if !self.vtable().map_or(true, |vt| !vt.is_borrowed()) {
            Ok(unsafe { core::mem::transmute(self) })
        } else {
            Self::intern_slice(&self).ok_or(self)
        }
    }

    /// Only calls [`Clone::clone`] if no reallocation would be necessary for it, returning `None` if it would have been.
    pub fn noalloc_clone(&self) -> Option<Self> {
        let Some(vtable) = self.vtable() else {
            // SAFETY: `Bytes` is `Copy` if it is interned.
            return Some(unsafe { core::ptr::read(self) });
        };
        let retain = vtable.retain?;
        unsafe { retain(self.data, self.capacity) };
        Some(Self {
            start: self.start,
            len: self.len,
            data: self.data,
            capacity: self.capacity,
            vtable: self.vtable,
            marker: core::marker::PhantomData,
        })
    }

    /// Returns `true` if a call to [`Self::upgrade`] would cause an allocation.
    pub fn upgrade_will_allocate(&self) -> bool {
        match self.vtable() {
            Some(t) => t.is_borrowed() && self.len() >= core::mem::size_of::<Self>(),
            None => false,
        }
    }

    /// Returns `true` if a call to [`Clone::clone`] would cause an allocation.
    pub fn clone_will_allocate(&self) -> bool {
        match self.vtable() {
            Some(t) => t.retain.is_none() && self.len() >= core::mem::size_of::<Self>(),
            None => false,
        }
    }
}

impl Default for Bytes<'_> {
    fn default() -> Self {
        Bytes::empty()
    }
}
impl AsRef<[u8]> for Bytes<'_> {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
impl Deref for Bytes<'_> {
    type Target = [u8];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl Borrow<[u8]> for Bytes<'_> {
    fn borrow(&self) -> &[u8] {
        self.as_slice()
    }
}
impl Debug for Bytes<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if f.alternate() {
            f.debug_struct("Bytes")
                .field("data", &self.as_slice())
                .field("owned", &!self.upgrade_will_allocate())
                .field("shared", &!self.clone_will_allocate())
                .finish()
        } else {
            Debug::fmt(self.as_slice(), f)
        }
    }
}
impl Hash for Bytes<'_> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state)
    }
}
impl<T: AsRef<[u8]>> PartialEq<T> for Bytes<'_> {
    fn eq(&self, other: &T) -> bool {
        self.as_slice() == other.as_ref()
    }
}
impl Eq for Bytes<'_> {}
impl<T: AsRef<[u8]>> PartialOrd<T> for Bytes<'_> {
    fn partial_cmp(&self, other: &T) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_ref())
    }
}
impl Ord for Bytes<'_> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other)
    }
}

#[cfg(feature = "std")]
impl std::io::Read for Bytes<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let len = self.len().min(buf.len());
        buf[..len].copy_from_slice(&self[..len]);
        self.shrink_to(len..);
        Ok(len)
    }
}

impl<'a> From<&'static [u8]> for Bytes<'a> {
    fn from(data: &'static [u8]) -> Self {
        Self::from_static(data)
    }
}
impl<'a, const N: usize> From<&'static [u8; N]> for Bytes<'a> {
    fn from(data: &'static [u8; N]) -> Self {
        Self::from_static(data.as_slice())
    }
}
impl<'a> From<&'static str> for Bytes<'a> {
    fn from(data: &'static str) -> Self {
        Self::from_static(data.as_bytes())
    }
}
#[cfg(feature = "alloc")]
unsafe extern "C" fn retain_arc_bytes(this: *const (), capacity: usize) {
    Arc::increment_strong_count(core::ptr::slice_from_raw_parts(this.cast::<u8>(), capacity))
}
#[cfg(feature = "alloc")]
unsafe extern "C" fn release_arc_bytes(this: *const (), capacity: usize) {
    Arc::decrement_strong_count(core::ptr::slice_from_raw_parts(this.cast::<u8>(), capacity))
}
#[cfg(feature = "alloc")]
static ARC_BYTES_VT: BytesVt = BytesVt {
    release: Some(release_arc_bytes),
    retain: Some(retain_arc_bytes),
};
#[cfg(feature = "alloc")]
impl From<Arc<[u8]>> for Bytes<'static> {
    fn from(data: Arc<[u8]>) -> Self {
        let capacity = data.len();
        Bytes {
            start: data.as_ref().as_ptr().cast(),
            len: data.len(),
            data: Arc::into_raw(data) as *const (),
            capacity,
            vtable: unsafe {
                NonNull::new_unchecked(
                    &ARC_BYTES_VT as &'static BytesVt as *const BytesVt as *mut BytesVt,
                )
            },
            marker: core::marker::PhantomData,
        }
    }
}
#[cfg(feature = "alloc")]
impl<'a, T: Sized + AsRef<[u8]> + Send + Sync + 'a> From<Arc<T>> for Bytes<'a> {
    fn from(value: Arc<T>) -> Self {
        unsafe extern "C" fn retain<T: Sized>(this: *const (), _: usize) {
            unsafe { Arc::increment_strong_count(this.cast::<T>()) }
        }
        unsafe extern "C" fn release<T: Sized>(this: *const (), _: usize) {
            unsafe { Arc::decrement_strong_count(this.cast::<T>()) }
        }
        let data: &[u8] = value.as_ref().as_ref();
        Bytes {
            start: data.as_ptr().cast(),
            len: data.len(),
            capacity: data.len(),
            data: Arc::into_raw(value) as *const (),
            vtable: unsafe {
                NonNull::new_unchecked(&BytesVt {
                    release: Some(release::<T>),
                    retain: Some(retain::<T>),
                } as &'static BytesVt as *const BytesVt
                    as *mut BytesVt)
            },
            marker: core::marker::PhantomData,
        }
    }
}
#[cfg(feature = "alloc")]
impl From<alloc::boxed::Box<[u8]>> for Bytes<'_> {
    fn from(value: alloc::boxed::Box<[u8]>) -> Self {
        unsafe extern "C" fn release_box_bytes(this: *const (), capacity: usize) {
            core::mem::drop(alloc::boxed::Box::from_raw(
                core::ptr::slice_from_raw_parts_mut(this.cast::<u8>().cast_mut(), capacity),
            ))
        }
        let bytes: &[u8] = &*value;
        let len = bytes.len();
        let start = bytes.as_ptr().cast();
        let data = alloc::boxed::Box::into_raw(value).cast::<()>();
        Bytes {
            start,
            len,
            capacity: len,
            data,
            vtable: unsafe {
                NonNull::new_unchecked(&BytesVt {
                    release: Some(release_box_bytes),
                    retain: None,
                } as &'static BytesVt as *const BytesVt
                    as *mut BytesVt)
            },
            marker: core::marker::PhantomData,
        }
    }
}
#[cfg(feature = "alloc")]
impl From<alloc::vec::Vec<u8>> for Bytes<'_> {
    fn from(value: alloc::vec::Vec<u8>) -> Self {
        alloc::boxed::Box::<[u8]>::from(value).into()
    }
}

#[cfg(feature = "stabby")]
unsafe extern "C" fn retain_stabby_arc_bytes(this: *const (), capacity: usize) {
    let this: stabby::sync::ArcSlice<u8> = unsafe {
        stabby::sync::ArcSlice::from_raw(stabby::alloc::AllocSlice {
            start: core::mem::transmute(this),
            end: core::mem::transmute(capacity),
        })
    };
    // we don't own any `Arc` in this function:
    let this = &*::core::mem::ManuallyDrop::new(this);
    // time to do the increment:
    core::mem::forget(this.clone());
}
#[cfg(feature = "stabby")]
unsafe extern "C" fn release_stabby_arc_bytes(this: *const (), capacity: usize) {
    let this: stabby::sync::ArcSlice<u8> = unsafe {
        stabby::sync::ArcSlice::from_raw(stabby::alloc::AllocSlice {
            start: core::mem::transmute(this),
            end: core::mem::transmute(capacity),
        })
    };
    core::mem::drop(this);
}
#[cfg(feature = "stabby")]
static STABBY_ARCSLICE_BYTESVT: BytesVt = BytesVt {
    retain: Some(retain_stabby_arc_bytes),
    release: Some(release_stabby_arc_bytes),
};
#[cfg(feature = "stabby")]
impl From<stabby::sync::ArcSlice<u8>> for Bytes<'static> {
    fn from(data: stabby::sync::ArcSlice<u8>) -> Self {
        let slice = data.as_slice();
        let start = slice.as_ptr().cast();
        let len = data.len();
        let data = stabby::sync::ArcSlice::into_raw(data);
        unsafe {
            Bytes {
                start,
                len,
                data: core::mem::transmute(data.start),
                capacity: core::mem::transmute(data.end),
                vtable: NonNull::new_unchecked(
                    &STABBY_ARCSLICE_BYTESVT as &'static BytesVt as *const BytesVt as *mut BytesVt,
                ),
                marker: core::marker::PhantomData,
            }
        }
    }
}
#[cfg(feature = "stabby")]
impl<T: Sized + AsRef<[u8]> + Send + Sync + 'static> From<stabby::sync::Arc<T>> for Bytes<'static> {
    fn from(value: stabby::sync::Arc<T>) -> Self {
        unsafe extern "C" fn retain_stabby_arc<T: Sized>(this: *const (), _: usize) {
            let this: stabby::sync::Arc<T> =
                unsafe { stabby::sync::Arc::from_raw(core::mem::transmute(this)) };
            let this = core::mem::ManuallyDrop::new(this);
            core::mem::forget(this.clone());
        }
        unsafe extern "C" fn release_stabby_arc<T: Sized>(this: *const (), _: usize) {
            let this: stabby::sync::Arc<T> =
                unsafe { stabby::sync::Arc::from_raw(core::mem::transmute(this)) };
            core::mem::drop(this)
        }
        let data: &[u8] = value.as_ref().as_ref();
        Bytes {
            start: data.as_ptr().cast(),
            len: data.len(),
            capacity: data.len(),
            data: unsafe { core::mem::transmute(stabby::sync::Arc::into_raw(value)) },
            vtable: unsafe {
                NonNull::new_unchecked(&BytesVt {
                    release: Some(release_stabby_arc::<T>),
                    retain: Some(retain_stabby_arc::<T>),
                } as &'static BytesVt as *const BytesVt
                    as *mut BytesVt)
            },
            marker: core::marker::PhantomData,
        }
    }
}
#[cfg(feature = "stabby")]
impl From<stabby::boxed::BoxedSlice<u8>> for Bytes<'_> {
    fn from(value: stabby::boxed::BoxedSlice<u8>) -> Self {
        stabby::vec::Vec::<u8>::from(value).into()
    }
}
#[cfg(feature = "stabby")]
impl From<stabby::vec::Vec<u8>> for Bytes<'_> {
    fn from(value: stabby::vec::Vec<u8>) -> Self {
        stabby::sync::ArcSlice::<u8>::from(value).into()
    }
}

#[cfg(feature = "alloc")]
unsafe impl<'a> Send for Bytes<'a>
where
    &'a [u8]: Send,
    Arc<[u8]>: Send,
    alloc::boxed::Box<[u8]>: Send,
    Arc<dyn 'a + AsRef<[u8]> + Send + Sync>: Send,
{
}
#[cfg(feature = "alloc")]
unsafe impl<'a> Sync for Bytes<'a>
where
    &'a [u8]: Sync,
    Arc<[u8]>: Sync,
    alloc::boxed::Box<[u8]>: Sync,
    Arc<dyn 'a + AsRef<[u8]> + Send + Sync>: Sync,
{
}

#[cfg(not(feature = "alloc"))]
unsafe impl<'a> Send for Bytes<'a> where &'a [u8]: Send {}
#[cfg(not(feature = "alloc"))]
unsafe impl<'a> Sync for Bytes<'a> where &'a [u8]: Send {}

#[cfg_attr(feature = "stabby", stabby::stabby)]
#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BytesVt {
    retain: Option<unsafe extern "C" fn(*const (), usize)>,
    release: Option<unsafe extern "C" fn(*const (), usize)>,
}
impl BytesVt {
    const fn is_borrowed(&self) -> bool {
        self.release.is_none()
    }
}

#[cfg(any(feature = "alloc", feature = "stabby"))]
/// Clones `self` by either:
/// - Performing a reference count increment (or a no-op for references) if possible
/// - Using [`Bytes::copied_from_slice`]
impl Clone for Bytes<'_> {
    fn clone(&self) -> Self {
        self.noalloc_clone()
            .unwrap_or_else(|| Self::copied_from_slice(self.as_slice()))
    }
}
impl Drop for Bytes<'_> {
    fn drop(&mut self) {
        if let Some(release) = self.vtable().and_then(|vt| vt.release) {
            unsafe { release(self.data, self.capacity) }
        }
    }
}

#[cfg(feature = "alloc")]
/// Attempts to downcast the [`Bytes`] into its inner `Arc<[u8]>`.
///
/// This requires the `value` to be backed by an `Arc<[u8]>`
/// and not have been shrunk or cloned from a shrunk value.
///
/// Note that unless `stabby` support is enabled, cloning or upgrading a [`Bytes`]
/// that was backed by a type that requires an allocation to do so will result
/// in an instance where this is guaranteed to succeed.
impl<'a> TryFrom<Bytes<'a>> for Arc<[u8]> {
    type Error = Bytes<'a>;
    fn try_from(value: Bytes<'a>) -> Result<Self, Self::Error> {
        let data = value.data.cast();
        let Some(vtable) = value.vtable() else {
            return Err(value);
        };
        match core::ptr::eq(vtable, &ARC_BYTES_VT)
            && core::ptr::eq(value.start, data)
            && value.len == value.capacity
        {
            true => unsafe {
                let arc = Arc::from_raw(core::ptr::slice_from_raw_parts(data, value.capacity));
                core::mem::forget(value);
                Ok(arc)
            },
            false => Err(value),
        }
    }
}

#[cfg(feature = "stabby")]
/// Attempts to downcast the [`Bytes`] into its inner [`stabby::sync::ArcSlice<u8>`](stabby::sync::ArcSlice).
///
/// This requires the `value` to be backed by an [`stabby::sync::ArcSlice<u8>`](stabby::sync::ArcSlice)
/// and not have been shrunk or cloned from a shrunk value.
///
/// Note that cloning or upgrading a [`Bytes`] that was backed by a
/// type that requires an allocation to do so will result
/// in an instance where this is guaranteed to succeed.
impl<'a> TryFrom<Bytes<'a>> for stabby::sync::ArcSlice<u8> {
    type Error = Bytes<'a>;
    fn try_from(value: Bytes<'a>) -> Result<Self, Self::Error> {
        let data = value.data.cast();
        let Some(vtable) = value.vtable() else {
            return Err(value);
        };
        match core::ptr::eq(vtable, &STABBY_ARCSLICE_BYTESVT) && core::ptr::eq(value.start, data) {
            true => unsafe {
                let value = core::mem::ManuallyDrop::new(value);
                let arc = stabby::sync::ArcSlice::from_raw(stabby::alloc::AllocSlice {
                    start: core::mem::transmute(data),
                    end: core::mem::transmute(data.add(value.len)),
                });
                Ok(arc)
            },
            false => Err(value),
        }
    }
}

#[cfg(feature = "alloc")]
#[test]
fn fuzz() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let data = (0..rng.gen_range(10..100))
            .map(|_| rng.gen())
            .collect::<Arc<[u8]>>();
        let bytes: Bytes<'_> = data.clone().into();
        assert_eq!(bytes.as_slice(), &*data);
        for _ in 0..100 {
            let start = rng.gen_range(0..data.len());
            let end = rng.gen_range(start..=data.len());
            assert_eq!(bytes.clone().subsliced(start..end), &data[start..end]);
            let (l, r) = bytes.clone().split_at(start).unwrap();
            assert_eq!((l.as_slice(), r.as_slice()), data.split_at(start));
        }
    }
}

#[cfg(feature = "stabby")]
#[test]
fn fuzz_stabby() {
    use rand::Rng;
    use stabby::{sync::ArcSlice, vec::Vec};
    let mut rng = rand::thread_rng();
    for _ in 0..1000 {
        let data: ArcSlice<u8> = (0..rng.gen_range(10..100))
            .map(|_| rng.gen())
            .collect::<Vec<u8>>()
            .into();
        println!("{data:?}");
        let bytes: Bytes<'_> = data.clone().into();
        assert_eq!(bytes.as_slice(), &*data, "Bytes construction went wrong");
        for _ in 0..100 {
            assert_eq!(bytes.clone().as_slice(), bytes.as_slice());
            let start = rng.gen_range(0..data.len());
            let end = rng.gen_range(start..=data.len());
            assert_eq!(
                bytes.clone().subsliced(start..end),
                &data[start..end],
                "subsliced went wrong"
            );
            let (l, r) = bytes.clone().split_at(start).unwrap();
            assert_eq!(
                (l.as_slice(), r.as_slice()),
                data.split_at(start),
                "split went wrong"
            );
        }
    }
}

#[cfg(feature = "serde")]
impl<'a> serde::Serialize for Bytes<'a> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.as_slice())
    }
}

#[cfg(feature = "serde")]
impl<'a, 'de: 'a> serde::Deserialize<'de> for Bytes<'a> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        serde::Deserialize::deserialize(deserializer).map(|x: &[u8]| Bytes::from(x))
    }
}
