//! `char *`-compatible strings (slim pointers), for easier use from within C.
//
//! They thus do not support inner nulls, nor string appending.

use_prelude!();

ReprC! {
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    /// A `#[repr(c)]` null-terminated UTF-8 encoded string, for compatibility
    /// with both the C `char *` ABI and Rust's `str`.
    ///
    /// This is a **borrowed** version, _i.e._, with the semantics of
    /// `&'lt CStr` / `&'lt str`, but for it being a _slim_ pointer.
    pub
    struct c_str_ref['lt,] (
        ptr::NonNullRef<c_char>,
        PhantomCovariantLifetime<'lt>,
    );
}

const NUL: u8 = b'\0';

impl c_str_ref<'static> {
    pub
    const EMPTY: Self = unsafe {
        Self::from_ptr_unchecked(ptr::NonNull::new_unchecked({
            const IT: u8 = NUL;
            &IT as *const u8 as *mut u8
        }))
    };
}
impl<'lt> c_str_ref<'lt> {
    pub
    const
    unsafe
    fn from_ptr_unchecked (ptr: ptr::NonNull<u8>)
      -> Self
    {
        Self(
            ptr::NonNullRef(ptr.cast()),
            PhantomCovariantLifetime::<'static>(PhantomData),
        )
    }
}

#[derive(Debug)]
pub
struct InvalidNulTerminator<Payload> (
    pub Payload,
);

impl<T> fmt::Display
    for InvalidNulTerminator<T>
{
    fn fmt (self: &'_ Self, fmt : &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        fmt::Display::fmt(
            "Null byte not at the expected terminating position",
            fmt,
        )
    }
}

impl<'lt> TryFrom<&'lt str>
    for c_str_ref<'lt>
{
    type Error = InvalidNulTerminator<()>;

    fn try_from (s: &'lt str)
      -> Result<
            c_str_ref<'lt>,
            InvalidNulTerminator<()>,
        >
    {
        Ok(if let Some(len_minus_one) = s.len().checked_sub(1) {
            unsafe {
                if s.bytes().position(|b| b == NUL) != Some(len_minus_one) {
                    return Err(InvalidNulTerminator(()));
                }
                Self::from_ptr_unchecked(
                    ptr::NonNull::new(s.as_ptr() as _).unwrap()
                )
            }
        } else {
            c_str_ref::EMPTY
        })
    }
}

cfg_std! {
    impl<T : fmt::Debug> ::std::error::Error
        for InvalidNulTerminator<T>
    {}

    /// # Panic
    ///
    /// Panics if the `CStr` is not valid UTF-8.
    impl<'lt> From<&'lt ::std::ffi::CStr>
        for c_str_ref<'lt>
    {
        #[inline]
        fn from (s: &'lt ::std::ffi::CStr)
          -> c_str_ref<'lt>
        {
            unsafe {
                let _assert_valid_utf8 =
                    ::core::str::from_utf8(s.to_bytes())
                        .unwrap()
                ;
                Self::from_ptr_unchecked(
                    ptr::NonNull::new(s.as_ptr() as _)
                        .unwrap()
                )
            }
        }
    }
}

impl<'lt> c_str_ref<'lt> {
    #[inline]
    pub
    fn bytes (self: c_str_ref<'lt>)
      -> impl Iterator<Item = ::core::num::NonZeroU8> + 'lt
    {
        ::core::iter::from_fn({
            let mut ptr = self.0.as_ptr().cast::<u8>();
            move || {
                let ret = ::core::num::NonZeroU8::new(unsafe { ptr.read() });
                if ret.is_some() {
                    unsafe {
                        ptr = ptr.add(1);
                    }
                }
                ret
            }
        })
    }

    #[inline]
    pub
    fn to_bytes (self: c_str_ref<'lt>)
      -> &'lt [u8]
    {
        unsafe {
            slice::from_raw_parts(
                self.0.as_ptr().cast(),
                self.bytes().count(),
            )
        }
    }

    #[inline]
    pub
    fn to_nonzero_bytes (self: c_str_ref<'lt>)
      -> &'lt [::core::num::NonZeroU8]
    {
        unsafe {
            slice::from_raw_parts(
                self.0.as_ptr().cast(),
                self.bytes().count(),
            )
        }
    }

    #[inline]
    pub
    fn to_str (self: c_str_ref<'lt>)
      -> &'lt str
    {
        unsafe {
            ::core::str::from_utf8_unchecked(self.to_bytes())
        }
    }
}

ReprC! {
    #[repr(transparent)]
    /// Same as [`c_str_ref`], but without any lifetime attached whatsoever.
    ///
    /// It is only intended to be used as the parameter of a callback that
    /// locally borrows it, dues to limitations of the [`ReprC`][
    /// `crate::layout::ReprCTrait`] design _w.r.t._ higher-rank trait bounds.
    pub
    struct c_str_ref_ (
        ptr::NonNullRef<c_char>,
    );
}

impl c_str_ref_ {
    /// # Safety
    ///
    ///   - The `c_str` must remain valid and immutable for the duration of the
    ///     `'borrow`.
    pub
    unsafe
    fn assume_valid<'borrow> (self: &'borrow Self)
      -> c_str_ref<'borrow>
    {
        unsafe {
            // # Safety
            //
            //   - Same layout,
            //
            //   - Caller guarantees the validity of the borrow.
            ::core::mem::transmute(self.0)
        }
    }
}

cfg_alloc! {
    ReprC! {
        #[repr(transparent)]
        /// A `#[repr(c)]` null-terminated UTF-8 encoded string, for compatibility
        /// with both the `char *` C API and Rust `str`.
        ///
        /// This is an **owned** / heap-allocated version, much like `Box<str>`
        /// / `Box<CStr>` but for it being a _slim_ pointer.
        pub
        struct c_str_boxed (
            ptr::NonNullOwned<c_char>,
        );
    }

    unsafe // Safety: inherited from `Box<[u8]>`.
    impl Send
        for c_str_boxed
    where
        rust::Box<[u8]> : Send,
    {}

    unsafe // Safety: inherited from `Box<[u8]>`.
    impl Sync
        for c_str_boxed
    where
        rust::Box<[u8]> : Sync,
    {}

    /// We use a `static` rather than a `const` for the empty string case
    /// as its address serves as a sentinel value for a fake-boxed string.
    /// (Otherwise empty `c_str_boxed` would need to allocate to hold the
    /// `NUL` terminator).
    static EMPTY_SENTINEL: u8 = NUL;

    impl c_str_boxed {
        pub
        const
        unsafe
        fn from_ptr_unchecked (ptr: ptr::NonNull<u8>)
          -> Self
        {
            Self(
                ptr::NonNullOwned(ptr.cast()),
            )
        }

        #[inline]
        pub
        fn as_ref (self: &'_ c_str_boxed)
          -> c_str_ref<'_>
        {
            unsafe {
                mem::transmute(self)
            }
        }
    }

    impl TryFrom<rust::String> for c_str_boxed {
        type Error = InvalidNulTerminator<rust::String>;

        fn try_from (s: rust::String)
          -> Result<
                c_str_boxed,
                InvalidNulTerminator<rust::String>,
            >
        {
            Ok(if let Some(len_minus_one) = s.len().checked_sub(1) {
                unsafe {
                    if s.as_bytes()[.. len_minus_one].contains(&NUL) {
                        return Err(InvalidNulTerminator(s));
                    }
                    let mut s = s;
                    if s.as_bytes()[len_minus_one] != NUL {
                        s.reserve_exact(1);
                        s.push(NUL as _);
                    }
                    let s: rust::Box<[u8]> = s.into_boxed_str().into();
                    Self::from_ptr_unchecked(
                        ptr::NonNull::new(rust::Box::leak(s).as_mut_ptr())
                            .unwrap()
                    )
                }
            } else {
                unsafe {
                    Self::from_ptr_unchecked(ptr::NonNull::new_unchecked(
                        (&EMPTY_SENTINEL) as *const _ as *mut _
                    ))
                }
            })
        }
    }

    impl Drop for c_str_boxed {
        fn drop (self: &'_ mut c_str_boxed)
        {
            unsafe {
                if self.0 .0 == ptr::NonNull::from(&EMPTY_SENTINEL).cast() {
                    return;
                }
                let strlen = self.as_ref().bytes().count();
                drop::<rust::Box<[u8]>>(
                    rust::Box::from_raw(slice::from_raw_parts_mut(
                        self.0 .0.as_ptr().cast(),
                        strlen,
                    ))
                );
            }
        }
    }
}

cfg_std! {
    /// # Panic
    ///
    /// Panics if the `CString` is not valid UTF-8.
    impl From<::std::ffi::CString> for c_str_boxed {
        fn from (s: ::std::ffi::CString)
          -> c_str_boxed
        {
            let _assert_valid_utf8 =
                ::core::str::from_utf8(s.as_bytes())
                    .unwrap()
            ;
            let s: rust::Box<[u8]> =
                s   .into_bytes_with_nul()
                    .into_boxed_slice()
            ;
            unsafe {
                Self::from_ptr_unchecked(
                    ptr::NonNull::new(rust::Box::leak(s).as_mut_ptr())
                        .unwrap()
                )
            }
        }
    }
}

#[doc(no_inline)]
pub use {
    c_str_ref as Ref,
    c_str_boxed as Boxed,
    c_str_ref_ as Ref_,
};
