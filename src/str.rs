use_prelude!();

cfg_alloc! {
    #[repr(transparent)]
    pub struct BoxedStr (
        BoxedSlice<u8>,
    );

    impl From<rust::Box<str>> for BoxedStr {
        #[inline]
        fn from (boxed_str: rust::Box<str>) -> Self
        {
            let boxed_bytes: rust::Box<[u8]> = boxed_str.into();
            Self(boxed_bytes.into())
        }
    }
    
    impl From<rust::String> for BoxedStr {
        #[inline]
        fn from (string: rust::String) -> Self
        {
            Self::from(string.into_boxed_str())
        }
    }
    
    impl<'lt> From<&'lt str> for BoxedStr {
        #[inline]
        fn from (s: &'lt str) -> Self
        {
            Self::from(rust::Box::<str>::from(s))
        }
    }

    impl Deref for BoxedStr {
        type Target = RefStr<'static>;
    
        #[inline]
        fn deref (self: &'_ Self) -> &'_ Self::Target
        {
            unsafe {
                mem::transmute(self)
            }
        }
    }
    
    impl AsRef<str> for BoxedStr {
        #[inline]
        fn as_ref (self: &'_ Self) -> &'_ str
        {
            &*self
        }
    }

    impl fmt::Debug for BoxedStr {
        fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
          -> fmt::Result
        {
            <str as fmt::Debug>::fmt(self, fmt)
        }
    }
}

#[repr(transparent)]
pub struct RefStr<'lt> (
    RefSlice<'lt, u8>,
);

impl<'lt> From<&'lt str> for RefStr<'lt> {
    #[inline]
    fn from (s: &'lt str) -> Self
    {
        Self(s.as_bytes().into())
    }
}

impl Deref for RefStr<'_> {
    type Target = str;

    #[inline]
    fn deref (self: &'_ Self) -> &'_ Self::Target
    {
        unsafe {
            ::core::str::from_utf8_unchecked(&*self.0)
        }
    }
}

impl AsRef<str> for RefStr<'_> {
    #[inline]
    fn as_ref (self: &'_ Self) -> &'_ str
    {
        &*self
    }
}

impl fmt::Debug for RefStr<'_> {
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        <str as fmt::Debug>::fmt(self, fmt)
    }
}
