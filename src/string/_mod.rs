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

    impl From<rust::String>
        for String
    {
        #[inline]
        fn from (s: rust::String) -> String
        {
            Self(rust::Vec::from(s).into())
        }
    }
    impl Into<rust::String>
        for String
    {
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

    impl fmt::Debug
        for String
    {
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
}
