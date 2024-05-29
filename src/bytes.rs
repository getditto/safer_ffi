use core::{
    borrow::Borrow,
    fmt::Debug,
    hash::Hash,
    ops::{Deref, Not, RangeBounds},
};

use alloc::sync::Arc;
use safer_ffi_proc_macros::derive_ReprC;

/// A slice of bytes optimized for sharing ownership of it and its subslices.
///
/// Typically, [`Bytes`] can constructed from `&[u8]`, `Arc<[u8]>` or `Arc<T: AsRef<[u8]>>`.
#[derive_ReprC]
#[repr(C)]
pub struct Bytes<'a> {
    start: &'a u8,
    len: usize,
    data: *const (),
    capacity: usize,
    vtable: &'a BytesVt,
}

extern "C" fn noop(_: *const (), _: usize) {}
impl<'a> Bytes<'a> {
    /// Constructs an empty slice.
    ///
    /// Since [`Bytes`] are immutable slices, this mostly serves to create default values.
    pub const fn empty() -> Self {
        Self::from_static([].as_slice())
    }
    /// Constructs a [`Bytes`] referring to static data.
    ///
    /// This is preferable to `<Bytes as From<&'static [u8]>>::from` in the sense that even if the value is cast to a non-static lifetime, [`Self::upgrade`] won't need to reallocate to recover the `'static` lifetime.
    pub const fn from_static(data: &'static [u8]) -> Self {
        const VT: BytesVt = BytesVt {
            retain: Some(noop),
            release: Some(noop),
        };
        Self {
            start: unsafe { &*data.as_ptr() },
            len: data.len(),
            data: data.as_ptr().cast(),
            capacity: data.len(),
            vtable: &VT,
        }
    }
    /// Slices `self` in-place:
    /// ```
    /// let data = b"Hello there".as_slice();
    /// let bytes = Bytes::from_static(data);
    /// bytes.subslice(3..7);
    /// assert_eq!(&data[3..7], bytes.as_slice());
    /// ```
    /// # Panics
    /// If the range's end is out of bounds, or if the range's start is greater than its end.
    pub fn subslice<R: RangeBounds<usize>>(&mut self, range: R) {
        let start = match range.start_bound() {
            core::ops::Bound::Included(i) => *i,
            core::ops::Bound::Excluded(i) => *i + 1,
            core::ops::Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            core::ops::Bound::Included(i) => *i + 1,
            core::ops::Bound::Excluded(i) => *i,
            core::ops::Bound::Unbounded => self.len,
        };
        assert!(start <= end);
        assert!(end <= self.len);
        let len = end - start;
        self.start = unsafe { &*(self.start as *const u8).add(start) };
        self.len = len;
    }
    /// Slices `self` in-place, returning it.
    /// ```
    /// let data = b"Hello there".as_slice();
    /// let bytes = Bytes::from_static(data);
    /// assert_eq!(&data[3..7], bytes.subsliced(3..7).as_slice());
    /// ```
    /// # Panics
    /// If the range's end is out of bounds, or if the range's start is greater than its end.
    pub fn subsliced<R: RangeBounds<usize>>(mut self, range: R) -> Self {
        self.subslice(range);
        self
    }
    /// Splits the slice at `index`.
    /// ```
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
        if index <= self.len {
            let mut left = self.clone();
            let mut right = left.clone();
            left.len = index;
            right.len -= index;
            right.start = unsafe { &*(self.start as *const u8).add(index) };
            Ok((left, right))
        } else {
            Err(self)
        }
    }
    /// Returns the slice's contents.
    pub const fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.start, self.len) }
    }
    /// Proves that the slice can be held onto for arbitrary durations, or copies it into a new one that does.
    pub fn upgrade(self) -> Bytes<'static> {
        if !self.vtable.is_borrowed() {
            return unsafe { core::mem::transmute(self) };
        }
        Arc::<[u8]>::from(self.as_slice()).into()
    }
    /// Attempts to prove that the slice has a static lifetime.
    /// # Errors
    /// Returns the original instance if it couldn't be proven to be `'static`.
    pub fn noalloc_upgrade(self) -> Result<Bytes<'static>, Self> {
        if !self.vtable.is_borrowed() {
            Ok(unsafe { core::mem::transmute(self) })
        } else {
            Err(self)
        }
    }
    /// Only calls [`Clone::clone`] if no reallocation would be necessary for it, returning `None` if it would have been.
    pub fn noalloc_clone(&self) -> Option<Self> {
        self.clone_will_allocate().not().then(|| self.clone())
    }
    /// Returns `true` if a call to [`Self::upgrade`] would cause an allocation.
    pub const fn upgrade_will_allocate(&self) -> bool {
        self.vtable.is_borrowed()
    }
    /// Returns `true` if a call to [`Clone::clone`] would cause an allocation.
    pub const fn clone_will_allocate(&self) -> bool {
        self.vtable.retain.is_none()
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
        f.debug_struct("Bytes")
            .field("data", &self.as_slice())
            .field("owned", &!self.upgrade_will_allocate())
            .field("shared", &!self.clone_will_allocate())
            .finish()
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
impl<'a> From<&'a [u8]> for Bytes<'a> {
    fn from(data: &'a [u8]) -> Self {
        static VT: BytesVt = BytesVt {
            release: None,
            retain: Some(noop),
        };
        Bytes {
            start: unsafe { &*data.as_ptr() },
            len: data.len(),
            data: data.as_ptr().cast(),
            capacity: data.len(),
            vtable: &VT,
        }
    }
}
impl From<Arc<[u8]>> for Bytes<'static> {
    fn from(data: Arc<[u8]>) -> Self {
        extern "C" fn retain(this: *const (), capacity: usize) {
            unsafe {
                Arc::increment_strong_count(core::ptr::slice_from_raw_parts(
                    this.cast::<u8>(),
                    capacity,
                ))
            }
        }
        extern "C" fn release(this: *const (), capacity: usize) {
            unsafe {
                Arc::decrement_strong_count(core::ptr::slice_from_raw_parts(
                    this.cast::<u8>(),
                    capacity,
                ))
            }
        }
        static VT: BytesVt = BytesVt {
            release: Some(release),
            retain: Some(retain),
        };
        let capacity = data.len();
        Bytes {
            start: unsafe { &*data.as_ptr() },
            len: data.len(),
            data: Arc::into_raw(data) as *const (),
            capacity,
            vtable: &VT,
        }
    }
}
impl<T: Sized + AsRef<[u8]> + Send + Sync> From<Arc<T>> for Bytes<'static> {
    fn from(value: Arc<T>) -> Self {
        extern "C" fn retain<T: Sized>(this: *const (), _: usize) {
            unsafe { Arc::increment_strong_count(this.cast::<T>()) }
        }
        extern "C" fn release<T: Sized>(this: *const (), _: usize) {
            unsafe { Arc::decrement_strong_count(this.cast::<T>()) }
        }
        let data: &[u8] = value.as_ref().as_ref();
        Bytes {
            start: unsafe { &*data.as_ptr() },
            len: data.len(),
            capacity: data.len(),
            data: Arc::into_raw(value) as *const (),
            vtable: &BytesVt {
                release: Some(release::<T>),
                retain: Some(retain::<T>),
            },
        }
    }
}
unsafe impl Send for Bytes<'_> {}
unsafe impl Sync for Bytes<'_> {}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BytesVt {
    retain: Option<extern "C" fn(*const (), usize)>,
    release: Option<extern "C" fn(*const (), usize)>,
}
impl BytesVt {
    const fn is_borrowed(&self) -> bool {
        self.release.is_none()
    }
}

impl Clone for Bytes<'_> {
    fn clone(&self) -> Self {
        if let Some(retain) = &self.vtable.retain {
            retain(self.data, self.capacity);
            Self {
                start: self.start,
                len: self.len,
                data: self.data,
                capacity: self.capacity,
                vtable: self.vtable,
            }
        } else {
            Arc::<[u8]>::from(self.as_slice()).into()
        }
    }
}
impl Drop for Bytes<'_> {
    fn drop(&mut self) {
        if let Some(release) = &self.vtable.release {
            release(self.data, self.capacity)
        }
    }
}
