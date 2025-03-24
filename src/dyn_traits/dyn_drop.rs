use super::*;

#[derive_ReprC(dyn)]
pub
trait DropGlue {}

/// We need to use a new type to avoid the trait-coherence issues of
/// an otherwise too blanket-y impl.
#[derive(Debug)]
#[repr(transparent)]
pub
struct ImplDropGlue<T>(pub T);

impl<T> DropGlue for ImplDropGlue<T> {}

impl DynDrop {
    pub
    fn new (value: impl 'static + Send + Sync)
      -> DynDrop
    {
        Self(::std::sync::Arc::new(ImplDropGlue(value)).into())
    }
}

/// Convenience shorthand around
/// `VirtualPtr<dyn 'static + Send + Sync + DropGlue>`.
#[derive(Debug, Clone)]
#[derive_ReprC]
#[repr(transparent)]
pub
struct DynDrop /* = */ (
    pub VirtualPtr<dyn 'static + Send + Sync + StaticDropGlue>,
);

#[derive_ReprC(dyn, Clone)]
pub
trait StaticDropGlue : Send + Sync {}

impl<T> StaticDropGlue for ImplDropGlue<T>
where
    T : 'static + Send + Sync,
{}
