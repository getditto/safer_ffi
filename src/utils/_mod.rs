#[macro_use]
mod macros;

pub(crate) mod extension_traits;

pub(crate) mod prelude;

pub(crate) mod markers;

pub(crate) unsafe fn transmute_unchecked<T, U>(ref it: T) -> U {
    use ::core::ops::Not as _;
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
    unsafe { ::core::mem::transmute_copy(it) }
}

#[allow(warnings)]
pub struct screaming_case<'__>(pub &'__ str, pub &'__ str);

const _: () = {
    use ::core::fmt::Display;
    use ::core::fmt::Write;
    use ::core::fmt::{self};
    use ::core::ops::Not;

    impl Display for screaming_case<'_> {
        fn fmt(
            self: &'_ Self,
            fmt: &'_ mut fmt::Formatter<'_>,
        ) -> fmt::Result {
            let mut first = true;
            [self.0, self.1]
                .iter()
                .flat_map(|&s| s.chars())
                .try_for_each(|c| {
                    if ::core::mem::take(&mut first).not() && c.is_ascii_uppercase() {
                        fmt.write_char('_')?;
                    }
                    fmt.write_char(c.to_ascii_uppercase())
                })
        }
    }
};

#[cfg(feature = "headers")]
#[allow(missing_debug_implementations)]
pub struct DisplayFromFn<F>(pub F)
where
    F: Fn(&mut dyn ::std::io::Write) -> ::std::io::Result<()>;

#[cfg(feature = "headers")]
impl<F> ::core::fmt::Display for DisplayFromFn<F>
where
    F: Fn(&mut dyn ::std::io::Write) -> ::std::io::Result<()>,
{
    fn fmt(
        &self,
        f: &mut ::core::fmt::Formatter<'_>,
    ) -> ::core::fmt::Result {
        struct IoToFmtBridge<W>(W);

        impl<W> ::std::io::Write for IoToFmtBridge<W>
        where
            W: ::core::fmt::Write,
        {
            fn write(
                &mut self,
                buf: &[u8],
            ) -> std::io::Result<usize> {
                self.0
                    .write_str(
                        ::core::str::from_utf8(buf).expect("only UTF-8 writes in `DisplayFromFn`"),
                    )
                    .map_err(::std::io::Error::other)?;
                Ok(buf.len())
            }

            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }

        self.0(&mut IoToFmtBridge(f)).map_err(|_| ::core::fmt::Error)
    }
}
