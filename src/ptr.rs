//! Wrappers around `NonNull` to better express the semantics of such pointer.
//!
//! Useful when manually defining custom low-level `ReprC` types.

use_prelude!();

#[doc(no_inline)]
/// Foo
pub use ::core::ptr::*;

#[repr(transparent)]
pub
struct NonNullRef<T> (
    pub
    ptr::NonNull<T>, // Variance OK because immutable
);

#[repr(transparent)]
pub
struct NonNullMut<T> (
    pub
    ptr::NonNull<T>,

    pub
    PhantomInvariant<T>, // Must be invariant because non-owning mutable.
);

#[repr(transparent)]
pub
struct NonNullOwned<T> (
    pub
    ptr::NonNull<T>, // Variance OK because ownership

    pub
    PhantomData<T>, // Express ownership to dropck
);

macro_rules! impl_for_each {(
    [$($T:ident),* $(,)?]
        .impl_for_each!(|$dol:tt $NonNull:ident| {
            $($expansion:tt)*
        })
    ;
) => (
    // const _: () = {
        macro_rules! helper {(
            $dol $NonNull : ident
        ) => (
            $($expansion)*
        )}
        $(
            helper! { $T }
        )*
    // };
)}

impl_for_each! {
    [NonNullRef, NonNullMut, NonNullOwned].impl_for_each!(|$NonNull| {
        impl<T> From<NonNull<T>>
            for $NonNull<T>
        {
            #[inline]
            fn from (it: NonNull<T>)
              -> Self
            {
                unsafe { ::core::mem::transmute(it) }
            }
        }

        impl<T> ::core::ops::Deref
            for $NonNull<T>
        {
            type Target = ptr::NonNull<T>;

            #[inline]
            fn deref (self: &'_ $NonNull<T>)
            -> &'_ ptr::NonNull<T>
            {
                &self.0
            }
        }

        impl<T> fmt::Debug
            for $NonNull<T>
        {
            fn fmt (self: &'_ $NonNull<T>, fmt: &'_ mut fmt::Formatter<'_>)
              -> fmt::Result
            {
                fmt .debug_tuple(stringify!($NonNull))
                    .field(&self.0)
                    .finish()
            }
        }

        impl<T> $NonNull<T> {
            #[inline]
            pub
            fn as_ptr (self: &'_ Self)
              -> *const T
            {
                self.0.as_ptr()
            }

            #[inline]
            pub
            fn cast<U> (self: $NonNull<T>)
              -> $NonNull<U>
            {
                unsafe { ::core::mem::transmute(self) }
            }
        }
    });
}

impl_for_each! {
    [NonNullMut, NonNullOwned].impl_for_each!(|$NonNull| {
        impl<T> ::core::ops::DerefMut
            for $NonNull<T>
        {
            #[inline]
            fn deref_mut (self: &'_ mut $NonNull<T>)
              -> &'_ mut ptr::NonNull<T>
            {
                &mut self.0
            }
        }

        impl<T> $NonNull<T> {
            #[inline]
            pub
            fn as_mut_ptr (self: &'_ mut Self)
              -> *mut T
            {
                self.0.as_ptr()
            }

            #[inline]
            pub
            fn copy (self: &'_ mut $NonNull<T>)
              -> $NonNull<T>
            {
                $NonNull::<T> { .. *self }
            }
        }
    });
}
impl_for_each! {
    [NonNullMut, NonNullRef].impl_for_each!(|$NonNull| {
        impl<'lt, T : 'lt> From<&'lt mut T>
            for $NonNull<T>
        {
            #[inline]
            fn from (it: &'lt mut T)
              -> $NonNull<T>
            {
                $NonNull::from(NonNull::from(it))
            }
        }
    });
}
impl<'lt, T : 'lt> From<&'lt T>
    for NonNullRef<T>
{
    #[inline]
    fn from (it: &'lt T)
      -> NonNullRef<T>
    {
        NonNullRef::from(NonNull::from(it))
    }
}

impl<__> NonNullOwned<__> {
    cfg_alloc! {
        #[inline]
        pub
        unsafe
        fn dealloc<T> (self)
        {
            if ::core::mem::size_of::<T>() == 0 {
                return;
            }
            ::alloc::alloc::dealloc(
                self.0.as_ptr().cast(),
                ::alloc::alloc::Layout::new::<T>(),
            );
        }

        #[inline]
        pub
        unsafe
        fn drop_in_place_and_dealloc<T> (mut self)
        {
            drop_in_place::<T>(self.copy().cast().as_mut());
            self.dealloc::<T>();
        }
    }

    #[inline]
    pub
    unsafe
    fn drop_in_place<T> (self)
    {
        drop_in_place::<T>(self.0.cast().as_mut());
    }
}

impl<__> Copy
    for NonNullRef<__>
{}
impl<__> Clone
    for NonNullRef<__>
{
    #[inline]
    fn clone (self: &'_ Self)
      -> Self
    {
        *self
    }
}
