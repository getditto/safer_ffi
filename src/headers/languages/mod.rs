#![cfg_attr(rustfmt, rustfmt::skip)]
#![allow(irrefutable_let_patterns)]

use_prelude!();
use {
    ::std::io::{
        self,
        Write as _,
    },
    super::{
        Definer,
    },
};

pub use c::C;
mod c;

__cfg_csharp__! {
    pub use csharp::CSharp;
    mod csharp;
}

__cfg_python__! {
    pub use python::Python;
    mod python;
}

__cfg_lua__! {
    pub use lua::Lua;
    mod lua;
}

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

type Docs<'lt> = &'lt [&'lt str];

pub
trait HeaderLanguage : UpcastAny {
    fn language_name(self: &'_ Self) -> &'static str {
        ::core::any::type_name::<Self>()
    }

    fn supports_type_aliases(self: &'_ Self)
      -> Option<&'_ dyn HeaderLanguageSupportingTypeAliases>
    {
        None
    }

    fn emit_simple_enum (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        backing_integer: Option<&'_ dyn PhantomCType>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()>
    ;

    fn emit_struct (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        fields: &'_ [StructField<'_>]
    ) -> io::Result<()>
    ;

    fn emit_opaque_type (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()>
    ;

    fn emit_function (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        fname: &'_ str,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()>
    ;

    fn emit_constant (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        name: &'_ str,
        ty: &'_ dyn PhantomCType,
        skip_type: bool,
        value: &'_ dyn ::core::fmt::Debug,
    ) -> io::Result<()>
    ;

    fn emit_docs (
        self: &'_ Self,
        _ctx: &'_ mut dyn Definer,
        _docs: Docs<'_>,
        _indentation: &'_ Indentation,
    ) -> io::Result<()>
    {
        // This function is just offered as a convenience helper;
        // it is not directly called by the framework.
        Ok(())
    }
}

pub
trait HeaderLanguageSupportingTypeAliases : HeaderLanguage {
    fn emit_type_alias(
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        inner_ty: &'_ dyn PhantomCType,
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
    ty: &'lt dyn PhantomCType,
}

pub
struct FunctionArg<'lt> {
    // pub
    // docs: Docs<'lt>,

    pub
    name: &'lt str,

    pub
    ty: &'lt dyn PhantomCType,
}

/// `T::assoc_func()` -> `PhantomData::<T>.method()` conversion
/// so as to become `dyn`-friendly (you can't pass a heterogeneous array of
/// *distinct* `T : Trait`s *types* to a function, but you can pass a slice of
/// `PhantomData`-materialized `dyn Trait`s).
///
/// In other words, we are projecting a compile-time type-level knowledge
/// of an array / struct / "table" of a type's associated functions
/// into a _runtime_ table of such, thence allowing runtime / `dyn`amic
/// unification within a heterogeneous collection.
pub
trait PhantomCType {
    fn short_name (
        self: &'_ Self,
    ) -> String
    ;

    fn name_wrapping_var (
        self: &'_ Self,
        language: &'_ dyn HeaderLanguage,
        var_name: &'_ str,
    ) -> String
    ;

    fn name (
        self: &'_ Self,
        language: &'_ dyn HeaderLanguage,
    ) -> String
    ;

    fn csharp_marshaler (
        self: &'_ Self,
    ) -> Option<String>
    ;

    fn size (
        self: &'_ Self,
    ) -> usize
    ;

    fn align (
        self: &'_ Self,
    ) -> usize
    ;
}

impl<T : ?Sized>
    PhantomCType
for
    ::core::marker::PhantomData<T>
where
    T : CType,
{
    fn short_name (
        self: &'_ Self,
    ) -> String
    {
        <T as CType>::short_name()
    }

    fn name_wrapping_var (
        self: &'_ Self,
        language: &'_ dyn HeaderLanguage,
        var_name: &'_ str,
    ) -> String
    {
        T::name_wrapping_var(language, var_name)
    }

    fn name (
        self: &'_ Self,
        language: &'_ dyn HeaderLanguage,
    ) -> String
    {
        T::name(language)
    }

    fn csharp_marshaler (
        self: &'_ Self,
    ) -> Option<String>
    {
        T::csharp_marshaler()
    }

    fn size (
        self: &'_ Self,
    ) -> usize
    {
        ::core::mem::size_of::<T>()
    }

    fn align (
        self: &'_ Self,
    ) -> usize
    {
        ::core::mem::align_of::<T>()
    }
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
        // $indent:tt,
        $out:expr $(,)?
    ) => (
        mk_out! { $indent_name /* $indent */ $out $ }
    );

    (
        $indent_name:tt /* $indent:tt */ $out:tt $_:tt
    ) => (
        macro_rules! out {
            (
                ($_(
                    $line:tt
                )*) $_($rest:tt)*
            ) => (
                // we have to use eager expansion of `concat!` coupled with
                // span manipulation for the implicit format args to work…
                ::with_builtin_macros::with_builtin! {
                    let $concat = concat!($_(
                        "{", stringify!($indent_name), "}",
                        $line,
                        "\n",
                    )*) in {
                        ::safer_ffi_proc_macros::__respan! {
                            // take the (first) span of the format string
                            // literals provided by the caller…
                            ( $_($line)* ) (
                                // …and replace, with it, the spans of the whole
                                //  `write!(` invocation.
                                for line in
                                    format!(
                                        $concat
                                        $_($rest)*
                                    )
                                    .split_inclusive('\n')
                                {
                                    let new_line = if line.ends_with('\n') { "\n" } else { "" };
                                    write!($out, "{}{new_line}", line.trim_end())?;
                                }
                            )
                        }
                    }
                }
                /* for reference, here is the "simple usecase" I'd have expected
                 * to Just Work™: */
                // write!(
                //     $out,
                //     concat!($_(
                //         // "{", stringify!($indent), "}",
                //         $indent,
                //         $line,
                //         "\n",
                //     )*)
                //     // , $indent_name = $indent_name
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
      -> &'_ dyn ::core::any::Any
    ;
}
impl<T : 'static> UpcastAny for T {
    fn upcast_any (self: &'_ Self)
      -> &'_ dyn ::core::any::Any
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
