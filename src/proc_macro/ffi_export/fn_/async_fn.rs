use ::syn::visit_mut::VisitMut;

use super::*;

pub(super) fn export(
    Args {
        executor,
        js,
        rename,
    }: Args,
    fun: &'_ ItemFn,
) -> Result<TokenStream2> {
    let block_on = match (executor, fun.sig.asyncness) {
        | (Some(Executor { block_on, .. }), Some(_asyncness)) => block_on,
        | (Some(Executor { kw: executor, .. }), None) => bail!(
            "\
                `#[ffi_export(…)]`'s `executor` attribute \
                can only be applied to an `async fn`. \
            " => executor,
        ),
        | (None, Some(asyncness)) => bail!(
            "\
                In order for `#[ffi_export(…)]` to support `async fn`, you \
                need to feed it an `executor = …` parameter and then use \
                `ffi_await!(…)` as the last expression of the function's body.\
            " => asyncness,
        ),
        | (None, None) => unreachable!(),
    };
    // The body of the function is expected to be of the form:
    // ```rust
    // #[ffi_export(js, executor = …)]
    // async fn ffi_func (args…)
    //   -> Ret
    // {
    //     <stmts>
    //     ffi_await!(<a future>)
    // }
    // ```
    // where the `<stmts>` make up a prelude that allow to make `<a future>` be
    // `'static`.
    let (ref prelude, ffi_await, ref async_body): (Vec<Stmt>, _, Expr) = {
        let mut stmts = fun.block.stmts.clone();
        let ill_formed_ffi_await_at = |problematic_tts: &dyn ToTokens| {
            bail! {
            "\
                `#[ffi_export(…, executor = …)]` expects the last \
                expression/statement to be an expression of the form: \
                `ffi_await!(<some future>)` such as:\n    \
                ffi_await!(async move {\n        …\n    })\n\
            " => problematic_tts
        }
        };
        match stmts.pop() {
            // no trailing expression case (block ending with a `;`):
            | Some(Stmt::Local(Local { semi_token, .. }))
            | Some(Stmt::Expr(_, Some(semi_token)))
            | Some(Stmt::Item(Item::Macro(ItemMacro {
                semi_token: Some(semi_token),
                ..
            }))) => {
                return ill_formed_ffi_await_at(&semi_token);
            },

            // `ffi_await!( tokens… )` or `ffi_await! { tokens… }` cases:
            | Some(Stmt::Expr(
                Expr::Macro(ExprMacro {
                    mac:
                        Macro {
                            path: ffi_await,
                            tokens,
                            ..
                        },
                    ..
                }),
                None,
            ))
            | Some(Stmt::Item(Item::Macro(ItemMacro {
                semi_token: None,
                mac:
                    Macro {
                        path: ffi_await,
                        tokens,
                        ..
                    },
                ..
            }))) if ffi_await.is_ident("ffi_await") => {
                // In order to guarantee good UX when using this fake
                // macro, we want to assert that these `tokens…`
                // represent a valid expression.
                //
                // While we could let `syn` parse them and produce its
                // own syntactic error when ill-formed, since we don't
                // need to, ourselves, inspect that expression in and of
                // itself (we just treat it as an opaque expression
                // which we trust to be an `impl Future`), we will
                // similarly trust its syntactic well-formedness and let
                // Rust complain at the final expansion.
                //
                // However, since things such as `let x = #expr;` can
                // actually succeed when `expr` is not an expression but
                // e.g. `42; let x = 27`, we still would like some
                // safeguard against that. We could use `(#expr)`, but then:
                //   - we need to disable the `unused_parens` lint…
                //   - in case of `cargo expand` people will see the parens.
                //
                // We instead use `identity_expr_macro!(#expr)`, which
                // achieves the same safeguard, but with invis parens.
                //
                // At that point, we may as well have a well-typed
                // `Expr` rather than a raw `TokenStream2`, which we can
                // cheaply achieve with the `Verbatim` variant.
                let expr = Expr::Verbatim(quote!(
                    ::safer_ffi::ඞ::assert_expr!(#tokens)
                ));
                (stmts, ffi_await.span(), expr)
            },
            // Other trailing stmt case, _e.g._, `mod some_item { … }`
            | Some(other_trailing_stmt) => return ill_formed_ffi_await_at(&other_trailing_stmt),
            // completely empty fn body!
            | None => return ill_formed_ffi_await_at(&reified_span(None)),
        }
    };

    let block_on = respan(fun.block.span(), block_on.into_token_stream());

    let ret = if cfg!(feature = "js") {
        if js.is_none() {
            // Nothing to do in this branch:
            return Ok(fun.into_token_stream());
        }
        let fname = &fun.sig.ident;
        let mut storage = None;
        let export_name = if let Some(Rename { new_name, .. }) = &rename {
            storage.get_or_insert(new_name.parse().expect("checked when parsing args"))
        } else {
            fname
        };
        let mut fun_signature = fun.sig.clone();
        fun_signature.asyncness = None;
        fun_signature.ident = Ident::new("__js", fname.span().resolved_at(Span::call_site()));
        let RetTy @ _ = match ::core::mem::replace(
            &mut fun_signature.output,
            parse_quote!(
                  -> ::safer_ffi::js::Result<
                        ::safer_ffi::js::JsUnknown
                    >
                ),
        ) {
            | ReturnType::Type(_, ty) => *ty,
            | ReturnType::Default => parse_quote!( () ),
        };
        let (each_arg_name, EachArgTy @ _) = fun_signature
            .inputs
            .iter_mut()
            .map(|fn_arg| match fn_arg {
                | FnArg::Receiver(_) => unimplemented!("`self` receivers"),
                | FnArg::Typed(PatType { pat, ty, .. }) => match **pat {
                    | Pat::Ident(PatIdent { ref ident, .. }) => (
                        ident.clone(),
                        ::core::mem::replace(
                            ty,
                            parse_quote!(
                            __ty_aliases::#ident
                        ),
                        ),
                    ),
                    | _ => unimplemented!("Non-trivial function param patterns",),
                },
            })
            .unzip::<_, _, Vec<_>, Vec<_>>();
        let EachArgTyStatic = EachArgTy.iter().cloned().map(|mut ty| {
            RemapNonStaticLifetimesTo {
                new_lt_name: "static",
            }
            .visit_type_mut(&mut ty);
            ty
        });
        let (each_lifetime, EachArgTyBounded): (Vec<_>, Vec<_>) = EachArgTy
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, mut ty)| {
                let new_lt_name =
                    &format!("use_stmts_before_ffi_await_to_compute_owned_captures_{}", i,);
                RemapNonStaticLifetimesTo { new_lt_name }.visit_type_mut(&mut ty);
                (
                    Lifetime {
                        apostrophe: ty.span(),
                        ident: Ident::new(new_lt_name, ty.span()),
                    },
                    ty,
                )
            })
            .unzip();

        let safer_ffi_js_promise_spawn = quote_spanned!(ffi_await=>
            ::safer_ffi::js::JsPromise::spawn
        );
        let js_future_async_body = quote! {
            ::core::concat!(
                "Use a `PhantomData` to make sure a",
                " `", ::core::stringify!(#RetTy), "` ",
                "is captured by the future, rendering it ",
                "non-`Send`",
            );
            let ret: #RetTy =
                match ::core::marker::PhantomData::<#RetTy> { _ => {
                    #async_body.await
                }}
            ;
            unsafe {
                "Safety: \
                since the corresponding `ReprC` type is \
                already captured by the future, the `CType` \
                wrapper can be assumed to be `Send`.";
                ::safer_ffi::js::UnsafeAssertSend::new(
                    ::safer_ffi::layout::into_raw(ret)
                )
            }
        };
        let js_future: ExprAsync = parse_quote_spanned!(ffi_await=>
            async move {
                #js_future_async_body
            }
        );
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
                                ::safer_ffi::js::ReprNapi
                            >::NapiValue
                        ;
                    )*
                }

                #[::safer_ffi::js::derive::js_export(js_name = #export_name)]
                #fun_signature
                {
                    #[inline(never)]
                    fn #fname< #(#each_lifetime,)* > (
                        __env__: &'_ ::safer_ffi::js::Env,
                        #(
                            #each_arg_name: #EachArgTyBounded
                        ),*
                    ) -> ::safer_ffi::js::Result<::safer_ffi::js::JsPromise>
                    {
                        #(#prelude)*
                        #safer_ffi_js_promise_spawn(
                            __env__, #js_future,
                        )
                        .map(|it| it.resolve_into_unknown())
                    }

                    let __ctx__ = ::safer_ffi::js::derive::__js_ctx!();
                    #fname(
                        __ctx__.env,
                        #({
                            let #each_arg_name: <#EachArgTy as ::safer_ffi::layout::ReprC>::CLayout =
                                ::safer_ffi::js::ReprNapi::from_napi_value(
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
    Ok(ret)
}

fn respan(
    span: Span,
    tokens: TokenStream2,
) -> TokenStream2 {
    use ::proc_macro2::*;

    tokens
        .into_iter()
        .map(|tt| match tt {
            | TT::Group(g) => TT::Group(Group::new(g.delimiter(), respan(span, g.stream()))),
            | mut tt => {
                tt.set_span(tt.span().resolved_at(span));
                tt
            },
        })
        .collect()
}
