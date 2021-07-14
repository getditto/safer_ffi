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

    let ret = if cfg!(feature = "node-js") {
        if node_js.is_none() {
            // Nothing to do in this branch:
            return fun.into_token_stream().into();
        }
        let fun_body = &fun.block;
        let fname = &fun.sig.ident;
        let mut fun_signature = fun.sig.clone();
        fun_signature.asyncness = None;
        fun_signature.ident = Ident::new(
            "__node_js",
            fname.span().resolved_at(Span::call_site()),
        );
        let RetTy @ _ =
            match ::core::mem::replace(
                &mut fun_signature.output,
                parse_quote!(
                  -> ::safer_ffi::node_js::Result<
                        ::safer_ffi::node_js::JsUnknown
                    >
                ),
            )
            {
                ReturnType::Type(_, ty) => *ty,
                ReturnType::Default => parse_quote!( () ),
            }
        ;
        let (each_arg_name, EachArgTy @ _) =
            fun_signature
            .inputs
            .iter_mut()
            .map(|fn_arg| match fn_arg {
                | FnArg::Receiver(_) => unimplemented!("`self` receivers"),
                | FnArg::Typed(PatType { pat, ty, .. }) => match **pat {
                    | Pat::Ident(PatIdent { ref ident, .. }) => (
                        ident.clone(),
                        ::core::mem::replace(ty, parse_quote!(
                            __ty_aliases::#ident
                        )),
                    ),
                    | _ => unimplemented!(
                        "Non-trivial function param patterns",
                    ),
                },
            })
            .unzip::<_, _, Vec<_>, Vec<_>>()
        ;
        let EachArgTyStatic = EachArgTy.iter().cloned().map(|mut ty| {
            RemapNonStaticLifetimesTo {
                new_lt_name: "static"
            }.visit_type_mut(&mut ty);
            ty
        });
        let EachArgTyBounded = EachArgTy.iter().cloned().map(|mut ty| {
            RemapNonStaticLifetimesTo {
                new_lt_name: "use__eager_prelude__to_compute_owned_variants",
            }.visit_type_mut(&mut ty);
            ty
        });

        quote!(
            const _: () = {
                // We want to use `type #arg_name = <$arg_ty as …>::Assoc;`
                // (with the lifetimes appearing there having been replaced with
                // `'static`, to soothe `#[wasm_bindgen]`).
                //
                // To avoid polluting the namespace with that many `#arg_name`s,
                // we will namespace those type aliases.
                mod __ty_aliases {
                    #![allow(nonstandard_style, unused_parens)]
                    use super::*;
                    #(
                        // Incidentally, the usage of a `type` alias ensures
                        // `__make_all_lifetimes_static!` is not missing hidden
                        // lifetime parameters in paths (_e.g._, `Cow<str>`, or
                        // more on point, `char_p::Ref`). Indeed, when one does
                        // that inside a type alias, a very nice error message
                        // will complain about it.
                        pub(in super)
                        type #each_arg_name =
                            <
                                <#EachArgTyStatic as ::safer_ffi::layout::ReprC>::CLayout
                                as
                                ::safer_ffi::node_js::ReprNapi
                            >::NapiValue
                        ;
                    )*
                }

                #[::safer_ffi::node_js::derive::js_export(js_name = #fname)]
                #fun_signature
                {
                    #[inline(never)]
                    fn #fname<'use__eager_prelude__to_compute_owned_variants> (
                        __env__: &'_ ::safer_ffi::node_js::Env,
                        #(
                            #each_arg_name: #EachArgTyBounded
                        ),*
                    ) -> ::safer_ffi::node_js::Result<::safer_ffi::node_js::JsPromise>
                    {
                        #prelude
                        ::safer_ffi::node_js::JsPromise::spawn(
                            __env__,
                            async move {
                                ::core::concat!(
                                    "Use a `PhantomData` to make sure a",
                                    " `", ::core::stringify!(#RetTy), "` ",
                                    "is captured by the future, rendering it ",
                                    "non-`Send`",
                                );
                                let ret: #RetTy =
                                    match ::core::marker::PhantomData::<#RetTy> { _ => {
                                        async #fun_body.await
                                    }}
                                ;
                                unsafe {
                                    "Safety: \
                                    since the corresponding `ReprC` type is \
                                    already captured by the future, the `CType` \
                                    wrapper can be assumed to be `Send`.";
                                    ::safer_ffi::node_js::UnsafeAssertSend::new(
                                        ::safer_ffi::layout::into_raw(ret)
                                    )
                                }
                            },
                        )
                        .map(|it| it.resolve_into_unknown())
                    }

                    let __ctx__ = ::safer_ffi::node_js::derive::__js_ctx!();
                    #fname(
                        __ctx__.env,
                        #({
                            let #each_arg_name: <#EachArgTy as ::safer_ffi::layout::ReprC>::CLayout =
                                ::safer_ffi::node_js::ReprNapi::from_napi_value(
                                    __ctx__.env,
                                    #each_arg_name,
                                )?
                            ;
                            unsafe {
                                ::safer_ffi::layout::from_raw_unchecked(#each_arg_name)
                            }
                        },)*
                    )
                    .map(|it| it.into_unknown())
                }
            };
        )
    } else {
        let mut fun_signature = fun.sig.clone();
        let pub_ = &fun.vis;
        let fun_body = &fun.block;
        fun_signature.asyncness = None;
        quote!(
            #[::safer_ffi::ffi_export]
            #pub_ #fun_signature
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
    ::syn::custom_keyword!(eager_prelude);
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
                | _case if snoopy.peek(kw::eager_prelude) => match ret.prelude {
                    | Some(_) => return Err(input.error("duplicate attribute")),
                    | ref mut it @ None => {
                        let _: kw::eager_prelude = input.parse().unwrap();
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

struct RemapNonStaticLifetimesTo {
    new_lt_name: &'static str,
}
use visit_mut::VisitMut;
impl VisitMut for RemapNonStaticLifetimesTo {
    fn visit_lifetime_mut (
        self: &'_ mut Self,
        lifetime: &'_ mut Lifetime,
    )
    {
        if lifetime.ident != "static" {
            lifetime.ident = Ident::new(
                self.new_lt_name,
                lifetime.ident.span(),
            );
        }
    }

    fn visit_type_reference_mut (
        self: &'_ mut Self,
        ty_ref: &'_ mut TypeReference,
    )
    {
        // 1 – sub-recurse
        visit_mut::visit_type_reference_mut(self, ty_ref);
        // 2 – handle the implicitly elided case.
        if ty_ref.lifetime.is_none() {
            ty_ref.lifetime = Some(Lifetime::new(
                &["'", self.new_lt_name].concat(),
                ty_ref.and_token.span,
            ));
        }
    }

    fn visit_parenthesized_generic_arguments_mut (
        self: &'_ mut Self,
        _: &'_ mut ParenthesizedGenericArguments,
    )
    {
        // Elided lifetimes in `fn(…)` or `Fn…(…)` are higher order:
        /* do not subrecurse */
    }
}
