use_prelude!();

/// Same as [`String`][`rust::String`], but with guaranteed `#[repr(C)]` layout
#[repr(transparent)]
pub
struct String (
    Vec<u8>,
);

impl From<rust::String> for String {
    #[inline]
    fn from (s: rust::String) -> String
    {
        Self(rust::Vec::from(s).into())
    }
}
impl Into<rust::String> for String {
    #[inline]
    fn into (self: String) -> rust::String
    {
        unsafe {
            rust::String::from_utf8_unchecked(
                self.0.into()
            )
        }
    }
}

impl Deref for String {
    type Target = RefStr<'static>;

    fn deref (self: &'_ Self) -> &'_ Self::Target
    {
        unsafe {
            mem::transmute::<
                &RefSlice<'static, u8>,
                &RefStr<'static>,
            >(self.0.as_ref())
        }
    }
}

impl fmt::Debug for String {
    fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
      -> fmt::Result
    {
        <str as fmt::Debug>::fmt(self, fmt)
    }
}

impl String {
    pub
    const EMPTY: Self = Self(Vec::EMPTY);

    pub
    fn with_rust_mut<R, F> (self: &'_ mut Self, f: F) -> R
    where
        F : FnOnce(&'_ mut rust::String) -> R
    {
        let mut s: rust::String = mem::replace(self, Self::EMPTY).into();
        let ret = f(&mut s);
        *self = s.into();
        ret
    }
}

