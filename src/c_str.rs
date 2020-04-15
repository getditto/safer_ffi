use_prelude!();

derive_ReprC! {
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    pub
    struct c_str['lt,] (
        ptr::NonNull<c_char>,
        PhantomCovariantLifetime<'lt>,
    );
}

const NUL: u8 = b'\0';

impl c_str<'static> {
    pub
    const EMPTY: Self = unsafe {
        Self::from_ptr_unchecked(ptr::NonNull::new_unchecked({
            const IT: u8 = NUL;
            &IT as *const u8 as *mut u8
        }))
    };
}
impl<'lt> c_str<'lt> {
    pub
    const
    unsafe
    fn from_ptr_unchecked (ptr: ptr::NonNull<u8>)
      -> Self
    {
        Self(
            ptr.cast(),
            PhantomCovariantLifetime::<'static>(PhantomData),
        )
    }
}

#[derive(Debug)]
pub
struct InvalidNulTerminator;

impl fmt::Display
    for InvalidNulTerminator
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

#[cfg(feature = "std")]
impl ::std::error::Error
    for InvalidNulTerminator
{}

impl<'lt> TryFrom<&'lt str>
    for c_str<'lt>
{
    type Error = InvalidNulTerminator;

    fn try_from (s: &'lt str)
      -> Result<c_str<'lt>, InvalidNulTerminator>
    {
        Ok(if let Some(len_minus_one) = s.len().checked_sub(1) {
            unsafe {
                if s.bytes().position(|b| b == NUL) != Some(len_minus_one) {
                    return Err(InvalidNulTerminator);
                }
                Self::from_ptr_unchecked(
                    ptr::NonNull::new(s.as_ptr() as _).unwrap()
                )
            }
        } else {
            c_str::EMPTY
        })
    }
}

/// # Panic
///
/// Panics if the `CStr` is not valid UTF-8.
#[cfg(feature = "std")]
impl<'lt> From<&'lt ::std::ffi::CStr>
    for c_str<'lt>
{
    #[inline]
    fn from (s: &'lt ::std::ffi::CStr)
      -> c_str<'lt>
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

impl<'lt> c_str<'lt> {
    #[inline]
    pub
    fn bytes (self: c_str<'lt>)
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
    fn to_bytes (self: c_str<'lt>)
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
    fn to_nonzero_bytes (self: c_str<'lt>)
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
    fn to_str (self: c_str<'lt>)
      -> &'lt str
    {
        unsafe {
            ::core::str::from_utf8_unchecked(self.to_bytes())
        }
    }
}

derive_ReprC! {
    #[repr(transparent)]
    /// Same as [`c_str`], but without any lifetime attached whatsoever.
    ///
    /// It is only intended to be used as the parameter of a callback that
    /// locally borrows it, dues to limitations of the [`ReprC`] design _w.r.t._
    /// higher-rank trait bounds.
    pub
    struct c_str_ (
        ptr::NonNull<c_char>,
    );
}

impl c_str_ {
    /// # Safety
    ///
    ///   - The `c_str` must remain valid and immutable for the duration of the
    ///     `'borrow`.
    pub
    unsafe
    fn assume_valid<'borrow> (self: &'borrow Self)
      -> c_str<'borrow>
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
