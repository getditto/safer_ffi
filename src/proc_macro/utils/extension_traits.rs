use super::*;

type Result<T, E = Error> = ::core::result::Result<T, E>;

pub(in crate)
trait CollectVec : Sized + IntoIterator {
    fn vec (self: Self)
      -> Vec<Self::Item>
    {
        #[expect(non_local_definitions)]
        impl<I : IntoIterator> CollectVec for I {}
        self.into_iter().collect()
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
        #[expect(non_local_definitions)]
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
        #[expect(non_local_definitions)]
        impl<T> Also for T {}
        tweak(&mut self);
        self
    }
}

/// Allows to convert a `bool` or an `Option<T>` into a `#( … )*`-usable
/// interpolable (to mock the `$( … )?` from `macro_rules!` macros).
pub
trait Kleene<'r> {
    type Ret;
    fn kleenable (self: &'r Self)
      -> Self::Ret
    ;
}
impl<'r, T : 'r + ToTokens> Kleene<'r> for Option<T> {
    type Ret = &'r [T];
    fn kleenable (self: &'r Option<T>)
      -> &'r [T]
    {
        self.as_ref().map_or(&[], slice::from_ref)
    }
}
// `bool` can be viewed as a `Option<EmptyTs>`.
impl Kleene<'_> for bool {
    type Ret = &'static [EmptyTs];
    fn kleenable (self: &'_ bool)
      -> &'static [EmptyTs]
    {
        if let true = self {
            &[EmptyTs]
        } else {
            &[]
        }
    }
}
pub
struct EmptyTs;
impl ToTokens for EmptyTs {
    fn to_tokens (self: &'_ EmptyTs, _: &mut TokenStream2)
    {}
}
