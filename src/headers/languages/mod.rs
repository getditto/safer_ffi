#![allow(irrefutable_let_patterns)]

use_prelude!();
use {
    ::std::io::{
        self,
        Write as _,
    },
    super::{
        Definer,
        Language,
    },
};

pub use c::C;
mod c;

pub use csharp::CSharp;
mod csharp;

pub
struct Indentation {
    depth: ::core::cell::Cell<usize>,
    width: usize,
}

impl Indentation {
    pub
    fn new (width: usize)
      -> Indentation
    {
        Self { depth: 0.into(), width }
    }

    pub
    fn scope (self: &'_ Self)
      -> impl '_ + Sized
    {
        self.depth.set(self.depth.get() + 1);
        ::scopeguard::guard((), move |()| {
            self.depth.set(self.depth.get() - 1);
        })
    }
}

impl ::core::fmt::Display for Indentation {
    fn fmt (
        self: &'_ Indentation,
        fmt: &'_ mut ::core::fmt::Formatter<'_>,
    ) -> ::core::fmt::Result
    {
        write!(fmt, "{: <indent$}", "", indent = self.depth.get() * self.width)
    }
}

#[derive(
    Debug,
    Copy, Clone,
    Eq, PartialEq, Ord, PartialOrd,
    Hash,
)]
pub
enum EnumSize {
    Default,
    Unsigned { bitwidth: u8 },
    Signed { bitwidth: u8 },
}

#[::safer_ffi_proc_macros::derive_ReprC2]
#[repr(u8)]
enum Foo { A, B = 12, C }

pub
type Docs<'lt> = &'lt [&'lt str];

pub
trait HeaderLanguage : UpcastAny {
    fn emit_docs (
        self: &'_ Self,
        out: &'_ mut dyn Definer,
        docs: Docs<'_>,
        indentation: &'_ Indentation,
    ) -> io::Result<()>
    ;

    fn emit_simple_enum (
        self: &'_ Self,
        out: &'_ mut dyn Definer,
        docs: Docs<'_>,
        name: &'_ str,
        // `(is_signed, bitwidth)`
        size: Option<(bool, u8)>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()>
    ;

    fn emit_struct (
        self: &'_ Self,
        out: &'_ mut dyn Definer,
        docs: Docs<'_>,
        name: &'_ str,
        size: usize,
        fields: &'_ [StructField<'_>]
    ) -> io::Result<()>
    ;

    fn emit_function (
        self: &'_ Self,
        out: &'_ mut dyn Definer,
        docs: Docs<'_>,
        fname: &'_ str,
        arg_names: &'_ [FunctionArg<'_>],
        ret_ty: &'_ str,
    ) -> io::Result<()>
    ;
}

pub
struct EnumVariant<'lt> {
    pub
    docs: Docs<'lt>,

    pub
    name: &'lt str,

    pub
    discriminant: Option<&'lt dyn ::core::fmt::Debug>,
}

pub
struct StructField<'lt> {
    pub
    docs: Docs<'lt>,

    pub
    name: &'lt str,

    pub
    emit_unindented: &'lt dyn
        Fn(&'_ dyn HeaderLanguage, &'_ mut dyn Definer) -> io::Result<()>
    ,

    pub
    layout: ::std::alloc::Layout,
}

pub
struct FunctionArg<'lt> {
    pub
    docs: Docs<'lt>,

    pub
    name: &'lt str,

    pub
    emit_unindented: &'lt dyn
        Fn(&'_ dyn HeaderLanguage, &'_ mut dyn Definer) -> io::Result<()>
    ,
}

/// Generates an `out!` macro.
///
/// Important: the `out!` macro accepts a `("foo" "bar" "baz")` shorthand
/// for the format literal parameter, to automatically convert it to:
///
/** ```rust ,ignore
concat!(
    "{indent}foo\n",
    "{indent}bar\n",
    "{indent}baz\n",
)
``` */
///
/// where `"{indent}"` is the first parameter passed to `mk_out!`,
/// and the second parameter is the `impl Write` the `write!`s will
/// be outputting to.
macro_rules! mk_out {
    (
        $indent_name:ident,
        $indent:tt,
        $out:expr $(,)?
    ) => (
        mk_out! { $indent_name $indent $out $ }
    );

    (
        $indent_name:tt $indent:tt $out:tt $_:tt
    ) => (
        macro_rules! out {
            (
                ($_(
                    $line:tt
                )*) $_($rest:tt)*
            ) => (
                with_builtin! {
                    let $concat = concat!($_(
                        $indent,
                        $line,
                        "\n",
                    )*) in {
                        ::safer_ffi_proc_macros::__respan! {
                            ( $_($line)* )
                            (
                                write!(
                                    $out,
                                    $concat
                                    // , $indent_name = $indent_name
                                    $_($rest)*
                                )?
                            )
                        }
                    }
                }
                // write!(
                //     $out,
                //     concat!($_(
                //         // "{", stringify!($indent), "}",
                //         $indent,
                //         $line,
                //         "\n",
                //     )*)
                //     , $indent_name = $indent_name
                //     $_($rest)*
                // )
            );

            ( $_($tt:tt)* ) => (
                write!($out, $_($tt)*)?
            )
        }
    );
} use mk_out;

pub
trait UpcastAny : 'static {
    fn upcast_any (self: &'_ Self)
      -> &dyn ::core::any::Any
    ;
}
impl<T : 'static> UpcastAny for T {
    fn upcast_any (self: &'_ Self)
      -> &dyn ::core::any::Any
    {
        self
    }
}

impl dyn HeaderLanguage {
    pub
    fn is<Concrete : HeaderLanguage> (
        self: &'_ Self,
    ) -> bool
    {
        self.upcast_any().is::<Concrete>()
    }

    pub
    fn downcast_ref<Concrete : HeaderLanguage> (
        self: &'_ Self,
    ) -> Option<&'_ Concrete>
    {
        self.upcast_any()
            .downcast_ref()
    }
}
