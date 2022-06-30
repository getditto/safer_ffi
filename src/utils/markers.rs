#![cfg_attr(rustfmt, rustfmt::skip)]
#![allow(missing_debug_implementations)]

#[derive(Default, Clone, Copy)]
pub
struct PhantomCovariantLifetime<'lt> (
    pub
    ::core::marker::PhantomData<
        &'lt ()
    >,
);

pub
struct PhantomInvariant<T : ?Sized> (
    pub
    ::core::marker::PhantomData<
        fn(&T) -> &T,
    >,
);

impl<T : ?Sized> Default
    for PhantomInvariant<T>
{
    #[inline]
    fn default () -> Self
    {
        Self(::core::marker::PhantomData)
    }
}

impl<T : ?Sized> Copy
    for PhantomInvariant<T>
{}
impl<T : ?Sized> Clone
    for PhantomInvariant<T>
{
    #[inline]
    fn clone (self: &'_ Self) -> Self
    {
        *self
    }
}
