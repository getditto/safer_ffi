#![cfg_attr(rustfmt, rustfmt::skip)]
use_prelude!();
use ::core::slice;
use crate::slice::*;

cfg_alloc! {
    ReprC! {
        #[repr(transparent)]
        #[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "alloc")))]
        /// Same as [`Box`][`rust::Box`]`<str>`, but with a guaranteed
        /// `#[repr(C)]` layout.
        pub
        struct str_boxed (
            slice_boxed<u8>,
        );
    }

    impl From<rust::Box<str>>
        for str_boxed
    {
        #[inline]
        fn from (boxed_str: rust::Box<str>)
          -> Self
        {
            let boxed_bytes: rust::Box<[u8]> = boxed_str.into();
            Self(boxed_bytes.into())
        }
    }

    impl From<rust::String>
        for str_boxed
    {
        #[inline]
        fn from (string: rust::String)
          -> Self
        {
            Self::from(string.into_boxed_str())
        }
    }

    impl<'lt> From<&'lt str>
        for str_boxed
    {
        #[inline]
        fn from (s: &'lt str)
          -> str_boxed
        {
            Self::from(rust::Box::<str>::from(s))
        }
    }

    impl str_boxed {
        #[inline]
        pub
        fn as_ref (self: &'_ str_boxed)
          -> str_ref<'_>
        {
            str_ref(self.0.as_ref())
        }
    }

    impl Deref
        for str_boxed
    {
        type Target = str;

        #[inline]
        fn deref (self: &'_ str_boxed)
          -> &'_ str
        {
            self.as_ref().as_str()
        }
    }

    impl AsRef<str>
        for str_boxed
    {
        #[inline]
        fn as_ref (self: &'_ str_boxed)
          -> &'_ str
        {
            &*self
        }
    }

    impl fmt::Debug
        for str_boxed
    {
        fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
          -> fmt::Result
        {
            <str as fmt::Debug>::fmt(self, fmt)
        }
    }

    impl Into<rust::Box<str>>
        for str_boxed
    {
        fn into (self: str_boxed)
          -> rust::Box<str>
        {
            unsafe {
                rust::Box::from_raw(rust::Box::<[u8]>::into_raw(
                    self.0.into()
                ) as _)
            }
        }
    }

    impl Into<rust::String>
        for str_boxed
    {
        fn into (self: str_boxed)
          -> rust::String
        {
            <rust::Box<str>>::into(self.into())
        }
    }
}

ReprC! {
    #[repr(transparent)]
    #[derive(Clone, Copy)]
    /// `&'lt str`, but with a guaranteed `#[repr(C)]` layout.
    pub
    struct str_ref['lt,] (
        slice_ref<'lt, u8>,
    );
}

impl<'lt> From<&'lt str>
    for str_ref<'lt>
{
    #[inline]
    fn from (s: &'lt str)
      -> str_ref<'lt>
    {
        let bytes = s.as_bytes();
        Self(
            bytes.into()
        )
    }
}

impl<'lt> str_ref<'lt> {
    #[inline]
    pub
    fn as_str (self: str_ref<'lt>)
      -> &'lt str
    {
        unsafe {
            ::core::str::from_utf8_unchecked(
                slice::from_raw_parts(
                    self.0.as_ptr(),
                    self.0.len(),
                )
            )
        }
    }
}

impl<'lt> Deref
    for str_ref<'lt>
{
    type Target = str;

    #[inline]
    fn deref (self: &'_ str_ref<'lt>)
      -> &'_ str
    {
        self.as_str()
    }
}

impl AsRef<str>
    for str_ref<'_>
{
    #[inline]
    fn as_ref (self: &'_ Self)
      -> &'_ str
    {
        self.as_str()
    }
}

impl fmt::Debug
    for str_ref<'_>
{
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        <str as fmt::Debug>::fmt(self, fmt)
    }
}
