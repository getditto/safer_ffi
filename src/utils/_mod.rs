#[macro_use]
mod macros;

pub(in crate)
mod prelude;

pub(in crate)
mod markers;

pub(in crate)
unsafe
fn transmute_unchecked<T, U> (ref it: T)
  -> U
{
    use ::core::ops::{Not as _};
    assert!(
        ::core::mem::size_of::<T>() == ::core::mem::size_of::<U>(),
        concat!(
            "Error, size mismatch.",
            " This is a soundness bug, please report an issue ASAP",
        )
    );
    assert!(
        ::core::mem::needs_drop::<T>().not(),
        concat!(
            "Error, input has drop glue.",
            " This is a soundness bug, please report an issue ASAP",
        ),
    );
    ::core::mem::transmute_copy(it)
}
