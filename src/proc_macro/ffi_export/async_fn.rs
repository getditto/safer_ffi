use super::*;

use ::proc_macro2::{Span, TokenStream as TokenStream2};

pub(in crate)
fn export (
    Attrs { block_on, node_js }: Attrs,
    fun: &'_ ItemFn,
) -> TokenStream
{
    let block_on = match (block_on, fun.sig.asyncness) {
        | (Some(block_on), Some(_asyncness)) => block_on,
        | (Some(block_on), None) => {
            return Error::new_spanned(block_on, "\
                `#[ffi_export(…)]`'s `executor` attribute \
                can only be applied to an `async fn`. \
            ").into_compile_error().into();
        },
        | (None, Some(asyncness)) => {
            return Error::new_spanned(asyncness, "\
                In order for `#[ffi_export(…)]` to support `async fn`, you \
                need to feed it an `executor = …` parameter and then use \
                `ffi_await!(…)` as the last expression of the function's body.\
            ").into_compile_error().into();
        },
        | (None, None) => unreachable!(),
    };
    // The body of the function is expected to be of the form:
    // ```rust
    // #[ffi_export(node_js, executor = …)]
    // async fn ffi_func (args…)
    //   -> Ret
    // {
    //     <stmts>
    //     ffi_await!(<a future>)
    // }
    // ```
    // where the `<stmts>` make up a prelude that allow to make `<a future>` be
    // `'static`.
    let (ref prelude, (ffi_await, ref async_body)): (Vec<Stmt>, (_, Expr)) = {
        let mut stmts = fun.block.stmts.clone();
        let mut async_body = None;
        if let Some(err_span) = (|| match stmts.pop() {
            | Some(Stmt::Local(Local { semi_token, .. }))
            | Some(Stmt::Semi(_, semi_token))
            | Some(Stmt::Item(
                Item::Macro(ItemMacro { semi_token: Some(semi_token), .. })
            )) => {
                Some(semi_token.span())
            },
            | Some(Stmt::Item(ref item)) => {
                Some(item.span())
            },
            | None => Some(Span::call_site()),

            | Some(Stmt::Expr(expr)) => {
                let span = expr.span();
                match expr {
                    | Expr::Macro(ExprMacro {
                        attrs: _,
                        mac: Macro {
                            path: ffi_await,
                            tokens,
                            ..
                        },
                    }) => if ffi_await.is_ident("ffi_await") {
                        if let Ok(expr) = parse2(tokens) {
                            async_body = Some((ffi_await.span(), expr));
                            return None;
                        }
                    },
                    | _ => {},
                }
                Some(span)
            },
        })()
        {
            return Error::new(err_span, "\
                `#[ffi_export(…, executor = …)]` expects the last \
                expression/statement to be an expression of the form: \
                `ffi_await!(<some future>)` such as:\n    \
                ffi_await!(async move {\n        …\n    })\n\
            ").into_compile_error().into();
        }
        (stmts, async_body.unwrap())
    };

    let block_on = respan(fun.block.span(), block_on.into_token_stream());

    let ret = if cfg!(feature = "node-js") {
        if node_js.is_none() {
            // Nothing to do in this branch:
            return fun.into_token_stream().into();
        }
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
            RemapNonStaticLifetimesTo { new_lt_name: "static" }
                .visit_type_mut(&mut ty)
            ;
            ty
        });
        let (each_lifetime, EachArgTyBounded): (Vec<_>, Vec<_>) =
            EachArgTy
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, mut ty)| {
                let new_lt_name = &format!(
                    "use_stmts_before_ffi_await_to_compute_owned_captures_{}",
                    i,
                );
                RemapNonStaticLifetimesTo { new_lt_name }
                    .visit_type_mut(&mut ty)
                ;
                (
                    Lifetime {
                        apostrophe: ty.span(),
                        ident: Ident::new(new_lt_name, ty.span()),
                    },
                    ty,
                )
            })
            .unzip()
        ;

        let safer_ffi_js_promise_spawn = quote_spanned!(ffi_await=>
            ::safer_ffi::node_js::JsPromise::spawn
        );
        let async_move = quote_spanned!(ffi_await=>
            async move
        );
        let mut js_future: ExprAsync = parse_quote!(
            #async_move {
                ::core::concat!(
                    "Use a `PhantomData` to make sure a",
                    " `", ::core::stringify!(#RetTy), "` ",
                    "is captured by the future, rendering it ",
                    "non-`Send`",
                );
                let ret: #RetTy =
                    match ::core::marker::PhantomData::<#RetTy> { _ => {
                        { #async_body }.await
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
            }
        );
        js_future.block.brace_token.span = ffi_await;
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
                    fn #fname< #(#each_lifetime,)* > (
                        __env__: &'_ ::safer_ffi::node_js::Env,
                        #(
                            #each_arg_name: #EachArgTyBounded
                        ),*
                    ) -> ::safer_ffi::node_js::Result<::safer_ffi::node_js::JsPromise>
                    {
                        #(#prelude)*
                        #safer_ffi_js_promise_spawn(
                            __env__, #js_future,
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
        let each_attr = &fun.attrs;
        fun_signature.asyncness = None;
        quote!(
            #[::safer_ffi::ffi_export]
            #(#each_attr)*
            #pub_ #fun_signature
            {
                #(#prelude)*
                #block_on(#async_body)
            }
        )
    };
    ret.into()
}

use ::syn::parse::{Parse, ParseStream};

#[derive(Default)]
pub(in super)
struct Attrs {
    pub(in super) node_js: Option<kw::node_js>,
    pub(in super) block_on: Option<Expr>,
}

mod kw {
    ::syn::custom_keyword!(node_js);
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

struct RemapNonStaticLifetimesTo<'__> {
    new_lt_name: &'__ str,
}
use visit_mut::VisitMut;
impl VisitMut for RemapNonStaticLifetimesTo<'_> {
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
