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

pub(in crate)
trait Extend_ {
    fn extend_<A, I> (
        &mut self,
        iterable: I,
    )
    where
        Self : Extend<A>,
        I : IntoIterator<Item = A>,
    {
        impl<T> Extend_ for T {}
        self.extend(iterable)
    }

    fn extend_one_<A> (
        &mut self,
        item: A,
    )
    where
        Self : Extend<A>,
    {
        self.extend([item])
    }
}

pub
trait Also : Sized {
    fn also (mut self, tweak: impl FnOnce(&mut Self))
      -> Self
    {
        impl<T> Also for T {}
        tweak(&mut self);
        self
    }
}
