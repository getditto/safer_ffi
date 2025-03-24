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
        "Error, size mismatch. \
        This is a soundness bug, please report an issue ASAP",
    );
    assert!(
        ::core::mem::needs_drop::<T>().not(),
        "Error, input has drop glue. \
        This is a soundness bug, please report an issue ASAP",
    );
    unsafe {
        ::core::mem::transmute_copy(it)
    }
}

#[allow(warnings)]
pub
struct screaming_case<'__> (
    pub &'__ str,
    pub &'__ str,
);

const _: () = {
    use ::core::{fmt::{self, Display, Write}, ops::Not};

    impl Display for screaming_case<'_> {
        fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
          -> fmt::Result
        {
            let mut first = true;
            [self.0, self.1].iter().flat_map(|&s| s.chars()).try_for_each(|c| {
                if ::core::mem::take(&mut first).not()
                && c.is_ascii_uppercase()
                {
                    fmt.write_char('_')?;
                }
                fmt.write_char(c.to_ascii_uppercase())
            })
        }
    }
};
