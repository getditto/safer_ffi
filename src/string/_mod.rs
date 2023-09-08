#![cfg_attr(rustfmt, rustfmt::skip)]
//! `Rust` string types with a defined `#[repr(C)]` layout, albeit not `char *`
//! compatible (_fat_ pointers).

use_prelude!();

pub use slice::*;
mod slice;

cfg_alloc! {
    use repr_c::Vec;

    ReprC! {
        #[repr(transparent)]
        #[cfg_attr(all(docs, feature = "nightly"), doc(cfg(feature = "alloc")))]
        /// Same as [`String`][`rust::String`], but with guaranteed `#[repr(C)]` layout
        pub
        struct String (
            Vec<u8>,
        );
    }

    /// Convert a [`std::string::String`] to a [`safer_ffi::String`].
    impl From<rust::String>
        for String
    {
        #[inline]
        fn from (s: rust::String) -> String
        {
            Self(rust::Vec::from(s).into())
        }
    }

    /// Convert a [`safer_ffi::String`] to a [`std::string::String`].
    impl From<String> for rust::String
    {
        #[inline]
        fn from(value: String) -> rust::String
        {
            unsafe {
                rust::String::from_utf8_unchecked(
                    value.0.into()
                )
            }
        }
    }

    impl Deref
        for String
    {
        type Target = str;

        fn deref (self: &'_ Self) -> &'_ Self::Target
        {
            unsafe {
                ::core::str::from_utf8_unchecked(&* self.0)
            }
        }
    }

    /// ```rust
    /// use ::safer_ffi::prelude::*;
    ///
    /// let s: repr_c::String = "".into();
    /// assert_eq!(format!("{s:?}"), "\"\"");
    /// ```
    impl fmt::Debug
        for String
    {
        fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
          -> fmt::Result
        {
            str::fmt(self, fmt)
        }
    }

    /// ```rust
    /// use ::safer_ffi::prelude::*;
    ///
    /// let s: repr_c::String = "".into();
    /// assert_eq!(format!("{s}"), "");
    /// ```
    impl fmt::Display
        for String
    {
        fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
          -> fmt::Result
        {
            str::fmt(self, fmt)
        }
    }

    impl From<&str> for repr_c::String {
        fn from(s: &str)
          -> repr_c::String
        {
            Self::from(rust::String::from(s))
        }
    }

    impl String {
        pub
        const EMPTY: Self = Self(Vec::EMPTY);

        pub
        fn with_rust_mut<R> (
            self: &'_ mut String,
            f: impl FnOnce(&'_ mut rust::String) -> R,
        ) -> R
        {
            self.0.with_rust_mut(|v: &'_ mut rust::Vec<u8>| {
                let s: &'_ mut rust::String = unsafe { mem::transmute(v) };
                f(s)
            })
        }
    }

    impl Clone for String {
        fn clone (
            self: &'_ Self,
        ) -> Self
        {
            Self(self.0.clone())
        }
    }
}
