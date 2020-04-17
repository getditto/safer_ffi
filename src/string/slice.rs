use_prelude!();

cfg_alloc! {
    ReprC! {
        #[repr(transparent)]
        #[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "alloc")))]
        /// Same as [`Box`][`rust::Box`]`<str>`, but with a guaranteed
        /// `#[repr(C)]` layout.
        pub
        struct str_boxed (
            slice_boxed<c_char>,
        );
    }

    impl From<rust::Box<str>>
        for str_boxed
    {
        #[inline]
        fn from (boxed_str: rust::Box<str>) -> Self
        {
            let boxed_bytes: rust::Box<[u8]> = boxed_str.into();
            boxed_bytes.into()
        }
    }

    impl From<rust::Box<[u8]>>
        for str_boxed
    {
        #[inline]
        fn from (boxed_bytes: rust::Box<[u8]>) -> Self
        {
            let boxed_bytes: rust::Box<[c_char]> = unsafe {
                // # Safety
                //
                //   - `c_char` is a `#[repr(transparent)]` wrapper around `u8`
                rust::Box::from_raw(rust::Box::into_raw(boxed_bytes) as _)
            };
            Self(boxed_bytes.into())
        }
    }

    impl From<rust::String>
        for str_boxed
    {
        #[inline]
        fn from (string: rust::String) -> Self
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

    impl Deref
        for str_boxed
    {
        type Target = str;

        #[inline]
        fn deref (self: &'_ str_boxed)
          -> &'_ str
        {
            unsafe {
                ::core::str::from_utf8_unchecked(
                    slice::from_raw_parts(
                        self.0.as_ptr().cast(),
                        self.0.len(),
                    )
                )
            }
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
}

ReprC! {
    #[repr(transparent)]
    /// `&'lt str`, but with a guaranteed `#[repr(C)]` layout.
    pub
    struct str_ref['lt,] (
        slice_ref<'lt, c_char>,
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
        unsafe { Self(
            slice::from_raw_parts(bytes.as_ptr().cast(), bytes.len())
                .into()
        )}
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
        unsafe {
            ::core::str::from_utf8_unchecked(
                slice::from_raw_parts(
                    self.0.as_ptr().cast(),
                    self.0.len(),
                )
            )
        }
    }
}

impl AsRef<str>
    for str_ref<'_>
{
    #[inline]
    fn as_ref (self: &'_ Self)
      -> &'_ str
    {
        &*self
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
