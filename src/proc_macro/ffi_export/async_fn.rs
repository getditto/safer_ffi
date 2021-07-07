use super::*;

use ::proc_macro2::{Span, TokenStream as TokenStream2};

pub(in crate)
fn export (
    attrs: TokenStream,
    fun: &'_ ItemFn,
) -> TokenStream
{
    assert!(fun.sig.asyncness.is_some());
    let asyncness = fun.sig.asyncness.as_ref().unwrap();
    let Attrs { block_on, node_js, prelude } = parse_macro_input!(attrs);
    let prelude = prelude.map_or_else(TokenStream2::new, |stmts| {
        respan(fun.block.span(), quote!( #(#stmts)* ))
    });
    let block_on = if let Some(it) = block_on { it } else {
        return Error::new_spanned(
            asyncness,
            "\
                `#[ffi_export(…)]` on an `async fn` needs a \
                `executor = …` attribute, such as:\n \
                 - #[ffi_export(executor = ::futures::executor::block_on)]\n\
                or:\n \
                 - #[ffi_export(executor = arg1.runtime_handle.block_on)]\n\
            ",
        ).into_compile_error().into();
    };
    let block_on = respan(fun.block.span(), block_on.into_token_stream());
    let ret = if cfg!(feature = "node_js") {
        if node_js.is_some() {
            todo!(stringify!(
                #[cfg(feature = "node_js")] fn ffi_export::async_fn::export()
            ));
        } else {
            fun.into_token_stream().into()
        }
    } else {
        let mut fun_signature = fun.sig.clone();
        let fun_body = &fun.block;
        fun_signature.asyncness = None;

        quote!(
            #[::safer_ffi::ffi_export]
            #fun_signature
            {
                #prelude
                #block_on(async move #fun_body)
            }
        )
    };
    // println!("{}", ret);
    ret.into()
}

use ::syn::parse::{Parse, ParseStream};

#[derive(Default)]
struct Attrs {
    node_js: Option<kw::node_js>,
    prelude: Option<Vec<Stmt>>,
    block_on: Option<Expr>,
}

mod kw {
    ::syn::custom_keyword!(node_js);
    ::syn::custom_keyword!(prelude);
    ::syn::custom_keyword!(executor);
}

impl Parse for Attrs {
    fn parse (input: ParseStream<'_>)
      -> Result<Attrs>
    {
        let mut ret = Attrs::default();
        while input.is_empty().not() {
            let snoopy = input.lookahead1();
            match () {
                | _case if snoopy.peek(kw::executor) => match ret.block_on {
                    | Some(_) => return Err(input.error("duplicate attribute")),
                    | ref mut it @ None => {
                        let _: kw::executor = input.parse().unwrap();
                        let _: Token![ = ] = input.parse()?;
                        *it = Some(input.parse()?);
                    },
                },
                | _case if snoopy.peek(kw::node_js) => match ret.node_js {
                    | Some(_) => return Err(input.error("duplicate attribute")),
                    | ref mut it @ None => {
                        *it = Some(input.parse().unwrap());
                    },
                },
                | _case if snoopy.peek(kw::prelude) => match ret.prelude {
                    | Some(_) => return Err(input.error("duplicate attribute")),
                    | ref mut it @ None => {
                        let _: kw::prelude = input.parse().unwrap();
                        let _: Token![ = ] = input.parse()?;
                        *it = Some(input.parse::<Block>()?.stmts);
                    },
                },
                | _default => return Err(snoopy.error()),
            }
            let _: Option<Token![ , ]> = input.parse()?;
        }
        Ok(ret)
    }
}

fn respan (span: Span, tokens: TokenStream2)
  -> TokenStream2
{
  use ::proc_macro2::{Group, TokenTree as TT};
  tokens.into_iter().map(|tt| match tt {
      | TT::Group(g) => TT::Group(
          Group::new(g.delimiter(), respan(span, g.stream()))
      ),
      | mut tt => {
          tt.set_span(tt.span().resolved_at(span));
          tt
      },
  }).collect()
}
