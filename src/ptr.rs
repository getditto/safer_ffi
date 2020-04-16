use_prelude!();

#[repr(transparent)]
#[derive(Clone, Copy)]
pub
struct NonNullRef<T> (
    pub
    ptr::NonNull<T>,
);

impl<T> ::core::ops::Deref
    for NonNullRef<T>
{
    type Target = ptr::NonNull<T>;

    #[inline]
    fn deref (self: &'_ Self)
      -> &'_ ptr::NonNull<T>
    {
        &self.0
    }
}

#[repr(transparent)]
pub
struct NonNullMut<T> (
    pub
    ptr::NonNull<T>,

    pub
    PhantomInvariant<T>,
);

impl<T> ::core::ops::Deref
    for NonNullMut<T>
{
    type Target = ptr::NonNull<T>;

    #[inline]
    fn deref (self: &'_ Self)
      -> &'_ ptr::NonNull<T>
    {
        &self.0
    }
}
impl<T> ::core::ops::DerefMut
    for NonNullMut<T>
{
    #[inline]
    fn deref_mut (self: &'_ mut Self)
      -> &'_ mut ptr::NonNull<T>
    {
        &mut self.0
    }
}

#[repr(transparent)]
pub
struct NonNullOwned<T> (
    pub
    ptr::NonNull<T>,
);

impl<T> ::core::ops::Deref
    for NonNullOwned<T>
{
    type Target = ptr::NonNull<T>;

    #[inline]
    fn deref (self: &'_ Self)
      -> &'_ ptr::NonNull<T>
    {
        &self.0
    }
}
impl<T> ::core::ops::DerefMut
    for NonNullOwned<T>
{
    #[inline]
    fn deref_mut (self: &'_ mut Self)
      -> &'_ mut ptr::NonNull<T>
    {
        &mut self.0
    }
}
