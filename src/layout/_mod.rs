//! Trait abstractions describing the semantics of "being `#[repr(C)]`"

use_prelude!();

use ::core::fmt;

#[macro_use]
mod macros;

#[doc(inline)]
pub use crate::{from_CType_impl_ReprC, ReprC, CType};

cfg_proc_macros! {
    pub use ::proc_macro::{
        derive_ReprC,
    };
}

cfg_headers! {
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

        impl<T : Definer> DefinerExt
            for T
        {
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
}

/// Marker trait for a type whose
pub
unsafe trait CType
:
    Sized +
    Copy +
{
    cfg_headers! {
        fn with_short_name<R> (ret: impl FnOnce(&'_ dyn fmt::Display) -> R)
          -> R
        ;

        #[inline]
        fn c_define_self (definer: &'_ mut dyn Definer)
          -> io::Result<()>
        {
            let _ = definer;
            Ok(())
        }

        fn c_fmt (
            fmt: &'_ mut fmt::Formatter<'_>,
            var_name: &'_ str,
        ) -> fmt::Result
        ;

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
}

cfg_headers! {
    mod impl_display {
        use super::*;
        use fmt::*;

        #[allow(missing_debug_implementations)]
        pub
        struct ImplDisplay<'__, T : CType> {
            pub(in super)
            var_name: &'__ str,

            pub(in super)
            _phantom: ::core::marker::PhantomData<T>,
        }

        impl<T : CType> Display
            for ImplDisplay<'_, T>
        {
            fn fmt (self: &'_ Self, fmt: &'_ mut Formatter<'_>)
            -> Result
            {
                T::c_fmt(fmt, self.var_name)
            }
        }
    }
}

#[cfg(docs)]
pub(in crate) use ReprC as ReprCTrait;

pub
unsafe
trait ReprC : Sized {
    type CLayout : CType;

    fn is_valid (it: &'_ Self::CLayout)
      -> bool
    ;
}

#[doc(hidden)] /** For clarity;
                   this macro may be stabilized
                   if downstream users find it useful
                **/
#[macro_export]
macro_rules! from_CType_impl_ReprC {(
    $(@for[$($generics:tt)*])? $T:ty $(where $($bounds:tt)*)?
) => (
    unsafe
    impl$(<$($generics)*>)? $crate::layout::ReprC
        for $T
    where
        $($($bounds)*)?
    {
        type CLayout = Self;

        #[inline]
        fn is_valid (_: &'_ Self::CLayout)
          -> bool
        {
            true
        }
    }
)}

#[inline]
pub
unsafe
fn from_raw_unchecked<T : ReprC> (c_layout: T::CLayout)
  -> T
{
    if let Some(it) = from_raw::<T>(c_layout) { it } else {
        if cfg!(debug_assertions) || cfg!(test) {
            panic!(
                "Error: not a valid bit-pattern for the type `{}`",
                // c_layout,
                ::core::any::type_name::<T>(),
            );
        } else {
            ::core::hint::unreachable_unchecked()
        }
    }
}

#[cfg_attr(feature = "proc_macros",
    require_unsafe_in_body,
)]
#[cfg_attr(not(feature = "proc_macros"),
    allow(unused_unsafe),
)]
#[inline]
pub
unsafe
fn from_raw<T : ReprC> (c_layout: T::CLayout)
  -> Option<T>
{
    if <T as ReprC>::is_valid(&c_layout).not() {
        None
    } else {
        Some(unsafe {
            const_assert! {
                for [T]
                    [T : ReprC] => [T::CLayout : Copy]
            }
            crate::utils::transmute_unchecked(c_layout)
        })
    }
}

#[inline]
pub
fn into_raw<T : ReprC> (it: T)
  -> T::CLayout
{
    unsafe {
        crate::utils::transmute_unchecked(
            ::core::mem::ManuallyDrop::new(it)
        )
    }
}

mod impls;
