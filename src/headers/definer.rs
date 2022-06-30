#![cfg_attr(rustfmt, rustfmt::skip)]
use super::*;

/// Helper for the generation of C headers.
///
/// Defining C headers requires _two_ abstractions:
///
///   - set-like lookup by name, to ensure each type is defined at most once;
///
///   - a [`Write`][`::std::io::Write`]able "out stream", where the headers
///     should be written to.
///
/// This trait minimally combines both abstractions, and in exchange offers an
/// auto-implemented non-overridable
/// [`Definer::define_once`]`()`.
pub
trait Definer : define_once_seal::__ {
    /// Must return `true` iff an actual `insert` happened.
    fn insert (self: &'_ mut Self, name: &'_ str)
      -> bool
    ;

    /// Yields a handle to the underlying [`Write`][`io::Write`]r
    fn out (self: &'_ mut Self)
      -> &'_ mut dyn io::Write
    ;

    #[cfg(docs)]
    /// Convenience method to perform an [`.insert()`][`Definer::insert`] so
    /// that if it succeeds (thus guaranteeing the call happens for the first
    /// time), it calls `write_typedef` on itself.
    ///
    /// **This method cannot be overriden**.
    fn define_once (
        self: &'_ mut Self,
        name: &'_ str,
        write_typedef: &'_ mut dyn
            FnMut (&'_ mut dyn Definer) -> io::Result<()>
        ,
    ) -> io::Result<()>
    { unreachable!("See `define_once_seal::__::define_once` for the impl") }
}

mod define_once_seal {
    use super::*;

    pub
    trait __ {
        fn define_once (
            self: &'_ mut Self,
            name: &'_ str,
            write_typedef: &'_ mut dyn
                FnMut(&'_ mut dyn Definer) -> io::Result<()>
            ,
        ) -> io::Result<()>
        ;
    }

    impl<T : Definer> __
        for T
    {
        fn define_once (
            self: &'_ mut Self,
            name: &'_ str,
            write_typedef: &'_ mut dyn
                FnMut(&'_ mut dyn Definer) -> io::Result<()>
            ,
        ) -> io::Result<()>
        {
            if self.insert(name) {
                write_typedef(self)?;
            }
            Ok(())
        }
    }
}

/// Simplest implementation of a [`Definer`]:
/// a `HashSet<String>, &'_ mut dyn Write` pair.
pub
struct HashSetDefiner<'out> {
    pub
    defines_set: HashSet<String>,

    pub
    out: &'out mut dyn io::Write,
}

impl Definer
    for HashSetDefiner<'_>
{
    fn insert (self: &'_ mut Self, name: &'_ str)
      -> bool
    {
        self.defines_set
            .insert(name.to_owned())
    }

    fn out (self: &'_ mut Self)
      -> &'_ mut dyn io::Write
    {
        &mut *self.out
    }
}
