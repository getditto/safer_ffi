use super::*;

macro_rules! define_kws {(
    $_:tt $($kw:ident),* $(,)?
) => (
    pub
    mod kw {
        $(
            ::syn::custom_keyword!($kw);
        )*
    }

    macro_rules! sym {
        $(
            ( $kw ) => ( kw::$kw );
        )*
        (
            $_($otherwise:tt)*
        ) => (
            ::syn::Token![ $_($otherwise)* ]
        );
    }
)}

define_kws! {$
    Clone,
}

pub
struct Args {
    pub dyn_: sym![dyn],
    pub clone: Option<sym![Clone]>,
}

impl Parse for Args {
    fn parse (input: ParseStream<'_>)
      -> Result<Args>
    {
        // We special case the `dyn` parameter. It does not provide any
        // information to the macro invocation; it just makes said invocation
        // more readable. We thus require it once, and at the beginning of the
        // args: the idea is that users could grep for `derive_ReprC(dyn`.
        let dyn_ = match input.parse() {
            | Ok(it) => it,
            | Err(err) => return Err(Error::new(
                err.span(),
                &format!("{err} — usage: `#[derive_ReprC(dyn, …)]`"),
            )),
        };
        let mut clone = None;
        let _: Option<Token![,]> = input.parse()?;
        while input.is_empty().not() {
            let snoopy = input.lookahead1();
            match () {
                | _case if input.peek(sym![dyn]) => {
                    return Err(input.error("duplicate parameter"));
                },
                | _case if snoopy.peek(sym![Clone]) => {
                    if clone.is_some() {
                        input.error("duplicate parameter");
                    }
                    clone = Some(input.parse().unwrap());
                },
                | _default => return Err(snoopy.error()),
            }
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(Self {
            dyn_,
            clone,
        })
    }
}
