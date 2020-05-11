//! `char *`-compatible strings (slim pointers), for easier use from within C.
//
//! They thus do not support inner nulls, nor string appending.

use_prelude!();
use ::core::slice;

ReprC! {
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    /// A `#[repr(c)]` null-terminated UTF-8 encoded string, for compatibility
    /// with both the C `char *` API and Rust's `str`.
    ///
    /// This is a **borrowed** version, _i.e._, with the semantics of
    /// `&'lt CStr` / `&'lt str`, but for it being a _slim_ pointer.
    pub
    struct char_p_ref['lt,] (
        ptr::NonNullRef<c_char>,
        PhantomCovariantLifetime<'lt>,
    );
}

const NUL: u8 = b'\0';

impl char_p_ref<'static> {
    pub
    const EMPTY: Self = unsafe {
        Self::from_ptr_unchecked(ptr::NonNull::new_unchecked({
            const IT: u8 = NUL;
            &IT as *const u8 as *mut u8
        }))
    };
}
impl<'lt> char_p_ref<'lt> {
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

impl fmt::Debug
    for char_p_ref<'_>
{
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        fmt::Debug::fmt(self.to_str(), fmt)
    }
}
impl fmt::Display
    for char_p_ref<'_>
{
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        fmt::Display::fmt(self.to_str(), fmt)
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
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        fmt::Display::fmt(
            "Null byte not at the expected terminating position",
            fmt,
        )
    }
}

impl<'lt> TryFrom<&'lt str>
    for char_p_ref<'lt>
{
    type Error = InvalidNulTerminator<()>;

    fn try_from (s: &'lt str)
      -> Result<
            char_p_ref<'lt>,
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
            char_p_ref::EMPTY
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
        for char_p_ref<'lt>
    {
        #[inline]
        fn from (s: &'lt ::std::ffi::CStr)
          -> char_p_ref<'lt>
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

impl<'lt> char_p_ref<'lt> {
    #[inline]
    pub
    fn bytes (self: char_p_ref<'lt>)
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
    fn to_nonzero_bytes (self: char_p_ref<'lt>)
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
    fn to_bytes (self: char_p_ref<'lt>)
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
    fn to_bytes_with_null (self: char_p_ref<'lt>)
      -> &'lt [u8]
    {
        unsafe {
            slice::from_raw_parts(
                self.0.as_ptr().cast(),
                self.bytes().count() + 1,
            )
        }
    }

    #[inline]
    pub
    fn to_str (self: char_p_ref<'lt>)
      -> &'lt str
    {
        unsafe {
            ::core::str::from_utf8_unchecked(self.to_bytes())
        }
    }

    #[inline]
    pub
    fn to_str_with_null (self: char_p_ref<'lt>)
      -> &'lt str
    {
        unsafe {
            ::core::str::from_utf8_unchecked(self.to_bytes_with_null())
        }
    }
}

impl<'lt> Eq for char_p_ref<'lt> {}
impl<'lt> PartialEq for char_p_ref<'lt> {
    #[inline]
    fn eq (self: &'_ Self, other: &'_ Self)
      -> bool
    {
        *self.to_str() == *other.to_str()
    }
}

ReprC! {
    #[repr(transparent)]
    #[allow(missing_copy_implementations)]
    /// Same as [`char_p_ref`], but without any lifetime attached whatsoever.
    ///
    /// It is only intended to be used as the parameter of a **callback** that
    /// locally borrows it, due to limitations of the [`ReprC`][
    /// `trait@crate::layout::ReprC`] design _w.r.t._ higher-rank trait bounds.
    pub
    struct char_p_raw (
        ptr::NonNullRef<c_char>,
    );
}

#[cfg_attr(feature = "proc_macros",
    require_unsafe_in_bodies,
)]
#[cfg_attr(not(feature = "proc_macros"),
    allow(unused_unsafe),
)]
impl char_p_raw {
    /// # Safety
    ///
    ///   - For the duration of the `'borrow`, the pointer must point to the
    ///     beginning of a valid and immutable null-terminated slice of
    ///     `c_char`s.
    pub
    unsafe
    fn as_ref<'borrow> (self: &'borrow Self)
      -> char_p_ref<'borrow>
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

impl<'lt> From<char_p_ref<'lt>>
    for char_p_raw
{
    #[inline]
    fn from (it: char_p_ref<'lt>)
      -> char_p_raw
    {
        unsafe {
            mem::transmute(it)
        }
    }
}

impl fmt::Debug
    for char_p_raw
{
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        fmt .debug_tuple("char_p_raw")
            .field(&self.0)
            .finish()
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
        struct char_p_boxed (
            ptr::NonNullOwned<c_char>,
        );
    }

    unsafe // Safety: inherited from `Box<[u8]>`.
    impl Send
        for char_p_boxed
    where
        rust::Box<[u8]> : Send,
    {}

    unsafe // Safety: inherited from `Box<[u8]>`.
    impl Sync
        for char_p_boxed
    where
        rust::Box<[u8]> : Sync,
    {}

    impl fmt::Debug
        for char_p_boxed
    {
        fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
          -> fmt::Result
        {
            fmt::Debug::fmt(&self.as_ref(), fmt)
        }
    }
    impl fmt::Display
        for char_p_boxed
    {
        fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
          -> fmt::Result
        {
            fmt::Display::fmt(&self.as_ref(), fmt)
        }
    }

    /// We use a `static` rather than a `const` for the empty string case
    /// as its address serves as a sentinel value for a fake-boxed string.
    /// (Otherwise empty `char_p_boxed` would need to allocate to hold the
    /// `NUL` terminator).
    static EMPTY_SENTINEL: u8 = NUL;

    impl char_p_boxed {
        #[inline]
        pub
        const
        unsafe
        fn from_ptr_unchecked (ptr: ptr::NonNull<u8>)
          -> Self
        {
            Self(
                ptr::NonNullOwned(ptr.cast(), PhantomData),
            )
        }

        #[inline]
        pub
        fn as_ref (self: &'_ char_p_boxed)
          -> char_p_ref<'_>
        {
            unsafe {
                mem::transmute(self.0.as_ref())
            }
        }
    }

    impl TryFrom<rust::String> for char_p_boxed {
        type Error = InvalidNulTerminator<rust::String>;

        fn try_from (s: rust::String)
          -> Result<
                char_p_boxed,
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

    impl Drop for char_p_boxed {
        fn drop (self: &'_ mut char_p_boxed)
        {
            unsafe {
                if ptr::eq(self.0.as_mut_ptr().cast(), &EMPTY_SENTINEL) {
                    return;
                }
                let num_bytes = self.to_bytes_with_null().len();
                drop::<rust::Box<[u8]>>(
                    rust::Box::from_raw(slice::from_raw_parts_mut(
                        self.0.as_mut_ptr().cast(),
                        num_bytes,
                    ))
                );
            }
        }
    }

    impl char_p_boxed {
        #[inline]
        pub
        fn bytes<'lt> (self: &'lt char_p_boxed)
          -> impl Iterator<Item = ::core::num::NonZeroU8> + 'lt
        {
            self.as_ref().bytes()
        }

        #[inline]
        pub
        fn to_nonzero_bytes (self: &'_ char_p_boxed)
          -> &'_ [::core::num::NonZeroU8]
        {
            self.as_ref().to_nonzero_bytes()
        }

        #[inline]
        pub
        fn to_bytes (self: &'_ char_p_boxed)
          -> &'_ [u8]
        {
            self.as_ref().to_bytes()
        }

        #[inline]
        pub
        fn to_bytes_with_null (self: &'_ char_p_boxed)
          -> &'_ [u8]
        {
            self.as_ref().to_bytes_with_null()
        }

        #[inline]
        pub
        fn to_str (self: &'_ char_p_boxed)
          -> &'_ str
        {
            self.as_ref().to_str()
        }

        #[inline]
        pub
        fn to_str_with_null (self: &'_ char_p_boxed)
          -> &'_ str
        {
            self.as_ref().to_str_with_null()
        }

        pub
        fn into_vec (mut self: char_p_boxed)
          -> rust::Vec<u8>
        {
            if ptr::eq(self.0.as_mut_ptr().cast(), &EMPTY_SENTINEL) {
                return vec![];
            }
            let num_bytes = self.to_bytes_with_null().len();
            let ptr = mem::ManuallyDrop::new(self).0.as_mut_ptr();
            let boxed_bytes = unsafe {
                rust::Box::from_raw(slice::from_raw_parts_mut(
                    ptr.cast(),
                    num_bytes,
                ))
            };
            let mut vec = rust::Vec::from(boxed_bytes);
            vec.pop();
            vec
        }

        #[inline]
        pub
        fn into_string (self: char_p_boxed)
          -> rust::String
        {
            unsafe {
                rust::String::from_utf8_unchecked(
                    self.into_vec()
                )
            }
        }
    }

    impl Eq for char_p_boxed {}
    impl PartialEq for char_p_boxed {
        #[inline]
        fn eq (self: &'_ Self, other: &'_ Self)
          -> bool
        {
            self.as_ref() == other.as_ref()
        }
    }
}

cfg_std! {
    /// # Panic
    ///
    /// Panics if the `CString` is not valid UTF-8.
    impl From<::std::ffi::CString> for char_p_boxed {
        fn from (s: ::std::ffi::CString)
          -> char_p_boxed
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
