#![cfg_attr(rustfmt, rustfmt::skip)]
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

#[allow(warnings)]
pub
struct screaming_case<'__>(
    pub &'__ str,
    pub &'__ str,
);

const _: () = {
    use ::core::fmt::{self, Display, Write};

    impl Display for screaming_case<'_> {
        fn fmt (self: &'_ Self, fmt: &'_ mut fmt::Formatter<'_>)
        -> fmt::Result
        {
            let mut not_first = false;
            [self.0, self.1].iter().copied().flat_map(|s| s.chars()).try_for_each(|c| {
                if true
                    && ::core::mem::replace(&mut not_first, true)
                    && c.is_ascii_uppercase()
                {
                    fmt.write_char('_')?;
                }
                fmt.write_char(c.to_ascii_uppercase())
            })
        }
    }
};
