use_prelude!();

use ::core::fmt;

#[macro_use]
mod macros;

#[doc(inline)]
pub use crate::{from_CType_impl_ReprC, derive_ReprC, derive_CType};

#[cfg(feature = "headers")]
pub
trait Definer : definer_ext::DefinerExt {
    /// Must return `true` iff an actual `insert` happened.
    fn insert (self: &'_ mut Self, name: &'_ str)
      -> bool
    ;

    fn out (self: &'_ mut Self)
      -> &'_ mut dyn io::Write
    ;
}

#[cfg(feature = "headers")]
mod definer_ext {
    use super::*;

    pub
    trait DefinerExt {
        fn define (
            self: &'_ mut Self,
            name: &'_ str,
            write_typedef: &'_ mut dyn
                FnMut (&'_ mut dyn Definer) -> io::Result<()>
            ,
        ) -> io::Result<()>
        ;
    }

    impl<T : Definer> DefinerExt for T {
        fn define (
            self: &'_ mut Self,
            name: &'_ str,
            write_typedef: &'_ mut dyn
                FnMut (&'_ mut dyn Definer) -> io::Result<()>
            ,
        ) -> io::Result<()>
        {
            if self.insert(name) {
                write_typedef(self)?;
            }
            Ok(())
        }
    }
}

/// Marker trait for a type whose
#[doc(spotlight)]
#[fundamental]
pub
unsafe trait CType
:
    Sized +
    Copy +
{
    #[cfg(feature = "headers")]
    fn with_short_name<R> (ret: impl FnOnce(&'_ dyn fmt::Display) -> R)
      -> R
    ;

    #[inline]
    #[cfg(feature = "headers")]
    fn c_define_self (definer: &'_ mut dyn Definer)
      -> io::Result<()>
    {
        let _ = definer;
        Ok(())
    }

    #[cfg(feature = "headers")]
    fn c_fmt (
        fmt: &'_ mut fmt::Formatter<'_>,
        var_name: &'_ str,
    ) -> fmt::Result
    ;

    #[cfg(feature = "headers")]
    #[inline]
    fn c_display<'__> (
        var_name: &'__ str,
    ) -> impl_display::ImplDisplay<'__, Self>
    {
        impl_display::ImplDisplay {
            var_name,
            _phantom: Default::default(),
        }
    }
}

#[cfg(feature = "headers")]
mod impl_display {
    use super::*;
    use fmt::*;

    pub
    struct ImplDisplay<'__, T : CType> {
        pub(in super)
        var_name: &'__ str,

        pub(in super)
        _phantom: ::core::marker::PhantomData<T>,
    }

    impl<T : CType> Display for ImplDisplay<'_, T> {
        fn fmt (self: &'_ Self, into: &'_ mut Formatter<'_>)
          -> Result
        {
            T::c_fmt(into, self.var_name)
        }
    }
}

#[doc(spotlight)]
pub
unsafe
trait ReprC : Sized {
    type CLayout : CType;

    fn is_valid (it: &'_ Self::CLayout)
      -> bool
    ;
}

#[macro_export]
macro_rules! from_CType_impl_ReprC {(
    $(@for[$($generics:tt)*])? $T:ty $(where $($bounds:tt)*)?
) => (
    unsafe
    impl$(<$($generics)*>)? $crate::layout::ReprC for $T
    where
        $($($bounds)*)?
    {
        type CLayout = Self;

        #[inline]
        fn is_valid (it: &'_ Self::CLayout)
          -> bool
        {
            true
        }
    }
)}

#[inline]
pub
unsafe
fn from_raw<T : ReprC> (c_layout: T::CLayout) -> T
{
    if cfg!(debug_assertions) || cfg!(test) {
        if <T as ReprC>::is_valid(&c_layout).not() {
            panic!(
                "Error: not a valid bit-pattern for the type `{}`",
                // c_layout,
                ::core::any::type_name::<T>(),
            );
        }
    }
    unsafe {
        const_assert! {
            for [T]
                [T : ReprC] => [T::CLayout : Copy]
        }
        crate::utils::transmute_unchecked(c_layout)
    }
}

#[inline]
pub
fn into_raw<T : ReprC> (it: T) -> T::CLayout
{
    unsafe {
        crate::utils::transmute_unchecked(
            ::core::mem::ManuallyDrop::new(it)
        )
    }
}

mod impls;

/*

pub
unsafe
trait ToBytes
:
    Sized +
    Copy +
{}
impl<T : ToBytes> AsBytes for T {}
pub
trait AsBytes : ToBytes {
    #[inline]
    fn as_bytes (self: &'_ Self)
      -> &'_ [u8]
    {
        unsafe {
            ::core::slice::from_raw_parts(
                <*const _>::cast::<u8>(self),
                ::core::mem::size_of::<Self>(),
            )
        }
    }
}

pub
unsafe
trait FromAnyBytes
:
    Sized +
    Copy +
{}
impl<T : FromAnyBytes> AsBytesMut for T {}
impl<T : FromAnyBytes> FromBytes for T {}
pub
trait AsBytesMut : FromAnyBytes {
    #[inline]
    fn as_bytes_mut (self: &'_ mut Self)
      -> &'_ mut [u8]
    {
        unsafe {
            ::core::slice::from_raw_parts_mut(
                <*mut _>::cast::<u8>(self),
                ::core::mem::size_of::<Self>(),
            )
        }
    }
}
pub
trait FromBytes : FromAnyBytes {
    #[inline]
    fn from_bytes (bytes: &'_ [u8])
      -> &'_ Self
    {
        assert!(bytes.len() >= ::core::mem::size_of::<Self>());
        unsafe {
            ::core::mem::transmute(bytes.as_ptr())
        }
    }
    #[inline]
    fn from_mut_bytes (bytes: &'_ mut [u8])
      -> &'_ mut Self
    where
        // I guess writing to `Self` could otherwise write padding bytes
        Self : ToBytes,
    {
        assert!(bytes.len() >= ::core::mem::size_of::<Self>());
        unsafe {
            ::core::mem::transmute(bytes.as_mut_ptr())
        }
    }
}

#[inline]
#[doc(hidden)] pub
fn assert_FromAnyBytes<T : FromAnyBytes> (_: *const T)
{}

#[inline]
#[doc(hidden)] pub
fn assert_ToBytes<T : ToBytes> (_: *const T)
{}

mod private { #[macro_export]
macro_rules! transmute_bytes {
    (
        $value:expr => [u8; $N:expr]
    ) => ({
        let it = $value;
        type Ret = [$crate::u8; $N];
        {
            let ref it = it;
            $crate::layout::assert_ToBytes(it);
        }
        unsafe {
            /// Safety: from `CType` contract
            extern {}

            $crate::core::mem::transmute::<_, Ret>(it)
        }
    });

    (
        $bytes:expr => $T:ty
    ) => ({
        let bytes: [$crate::u8; $crate::core::mem::size_of::<$T>()] = $bytes;
        let _ = $crate::layout::assert_FromAnyBytes::<$T>;
        unsafe {
            $crate::core::mem::transmute::<_, $T>(bytes)
        }
    });
}} #[doc(inline)] pub use crate::transmute_bytes;

*/
