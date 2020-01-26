use ::std::{
    ffi::{CStr, CString},
    fmt,
    ops::{Deref, Not},
    os::raw::{
        c_char,
    },
    sync::Arc,
};

#[derive(Debug)]
#[repr(transparent)]
pub struct SharedCStr /* = */ (
    /// Owned raw version of a Arc<CStr>
    pub *const c_char,
);

unsafe impl Send for SharedCStr
    where Arc<CStr> : Send
{}
unsafe impl Sync for SharedCStr
    where Arc<CStr> : Sync
{}

impl From<Arc<CStr>> for SharedCStr {
    #[inline]
    fn from (s: Arc<CStr>) -> Self
    {
        if let Err(err) = s.to_str() {
            panic!("`SharedCStr` expects a valid UTF-8 string: {}", err);
        };
        let fat_ptr = Arc::into_raw(s);
        Self(fat_ptr as *const c_char)
    }
}

impl<'a> From<&'a str> for SharedCStr {
    #[inline]
    fn from (s: &'a str) -> Self
    {
        let c_str =
            CString::new(s.as_bytes())
                .expect("`SharedCStr` does not support inner nul bytes")
        ;
        Self::from(c_str)
    }
}

impl From<CString> for SharedCStr {
    #[inline]
    fn from (s: CString) -> Self
    {
        let arc: Arc<CStr> = s.into();
        Self::from(arc)
    }
}

impl<'a> From<&'a CStr> for SharedCStr {
    #[inline]
    fn from (s: &'a CStr) -> Self
    {
        let arc: Arc<CStr> = s.into();
        Self::from(arc)
    }
}

impl Drop for SharedCStr {
    #[inline]
    fn drop (&mut self)
    {
        let &mut Self(ptr) = self;
        debug_assert!(ptr.is_null().not());
        unsafe {
            let c_str: &CStr = CStr::from_ptr(ptr);
            let arc: Arc<CStr> = Arc::from_raw(c_str);
            drop(arc);
        }
    }
}

impl Clone for SharedCStr {
    #[inline]
    fn clone (&self) -> Self
    {
        use ::std::mem::ManuallyDrop;
        let &Self(ptr) = self;
        debug_assert!(ptr.is_null().not());
        let c_str = unsafe { CStr::from_ptr(ptr) };
        let arc = ManuallyDrop::new(unsafe {
            Arc::from_raw(c_str)
        });
        Self::from(Arc::clone(&arc))
    }
}

impl Deref for SharedCStr {
    type Target = str;

    #[inline]
    fn deref (self: &'_ Self) -> &'_ Self::Target
    {
        let &Self(ptr) = self;
        debug_assert!(ptr.is_null().not());
        let c_str = unsafe { CStr::from_ptr(ptr) };
        c_str.to_str().unwrap_or_else(|_| unsafe {
            ::std::hint::unreachable_unchecked()
        })
    }
}

impl AsRef<str> for SharedCStr {
    #[inline]
    fn as_ref (self: &'_ Self) -> &'_ str
    {
        &*self
    }
}

impl fmt::Display for SharedCStr {
    #[inline]
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>) -> fmt::Result
    {
        <str as fmt::Display>::fmt(&self, fmt)
    }
}

// TODO: add Eq, PartialEq, Hash, PartialOrd, Ord

