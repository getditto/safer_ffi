pub(in crate)
trait CollectVec : Sized + Iterator {
    fn vec (self: Self)
      -> Vec<Self::Item>
    {
        impl<I : Iterator> CollectVec for I {}
        self.collect()
    }
}

pub(in crate)
trait VMap : Sized + IntoIterator {
    fn vmap<T> (
        self: Self,
        f: impl FnMut(Self::Item) -> T,
    ) -> Vec<T>
    {
        self.into_iter().map(f).collect()
    }

    fn try_vmap<T, E> (
        self: Self,
        f: impl FnMut(Self::Item) -> Result<T, E>
    ) -> Result<Vec<T>, E>
    {
        self.into_iter().map(f).collect()
    }
}

impl<I : ?Sized> VMap for I
where
    Self : Sized + IntoIterator,
{}
