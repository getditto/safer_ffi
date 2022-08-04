use super::*;

use args::*;
mod args;

#[cfg(feature = "async-fn")]
mod async_fn;

const SUPPORTED_ABIS: &[&str] = &[
    "C",
];

fn concrete_c_type (T @ _: &'_ Type)
  -> Type
{
    parse_quote_spanned!(T.span()=>
        <#T as ::safer_ffi::ඞ::ConcreteReprC>::ConcreteCLayout
    )
}

pub(in super)
fn handle (
    args: Args,
    mut fun: ItemFn,
) -> Result<TokenStream2>
{
    // async fn case.
    if args.executor.is_some() || fun.sig.asyncness.is_some() {
        if true {
            #[cfg(feature = "async-fn")]
            return async_fn::export(args, &fun);
        }
        bail! {
            "\
                Support for `#[ffi_export(executor = …)] async fn` requires \
                that the `async-fn` Cargo feature of `safer-ffi` be enabled.\
            " => Option::<&dyn ToTokens>::or(
                fun.sig.asyncness.as_ref().map(|it| it as _),
                args.executor.as_ref().map(|it| &it.kw as _),
            ).unwrap(),
        }
    }

    let mut storage = None;
    let export_name_str: &LitStr =
        if let Some(Rename { new_name, .. }) = &args.rename {
            new_name
        } else {
            storage.get_or_insert(
                LitStr::new(
                    &fun.sig.ident.to_string(),
                    fun.sig.ident.span(),
                )
            )
        }
    ;

    // *We* handle the C-safety heuristics in a more accurate manner than
    // rustc's lint, so let's disable it to prevent it from firing false
    // positives against us.
    fun.attrs.push(parse_quote!(
        #[allow(improper_ctypes_definitions)]
    ));
    fun.attrs.push(parse_quote!(
        #[forbid(elided_lifetimes_in_paths)]
    ));
    // Ergonomics: lack-of-`extern` defaults to `extern "C"`.
    fun.sig.abi.get_or_insert_with(|| parse_quote!(
        extern "C"
    ));
    // No more changes to the original function:
    let fun = fun;

    let extern_ = fun.sig.abi.as_ref().unwrap();
    if matches!(
        &extern_.name, Some(abi)
        if SUPPORTED_ABIS.contains(&abi.value().as_str()).not()
    )
    {
        return Err(Error::new_spanned(
            &extern_.name,
            &format!(
                "unsupported abi, expected one of {:?}", SUPPORTED_ABIS,
            ),
        ));
    }

    if let Some(receiver) = fun.sig.receiver() {
        bail! {
            "methods are not supported" => receiver,
        }
    }

    // The actually ffi-exported function: a shim around the given input.
    let mut ffi_fun = fun.clone();
    let each_arg = &ffi_fun.sig.inputs.iter_mut().enumerate().vmap(|(i, arg)| {
        match *arg {
            | FnArg::Receiver(_) => unreachable!(),
            | FnArg::Typed(PatType { ref mut pat, ref mut ty, .. }) => {
                // C-ize each arg type.
                **ty = concrete_c_type(ty);

                // Normalize the arg name: strip `ref`s and `mut` if ident, else
                // fall back to a `ARG_PREFIX{i}` override.
                const ARG_PREFIX: &str = "__arg_";
                match **pat {
                    | Pat::Ident(PatIdent {
                        ref mut by_ref,
                        ref mut mutability,
                        ident: ref arg_name,
                        ..
                    })
                        /* should not be needed thanks to mixed site hygiene */
                        // ensure user-provided names cannot collide with the
                        // ones we generate.
                        //   - While mixed-site hygiene would help on the Rust
                        //     side, it won't on the generated code side:
                        //     `./generated.h:144:13: error: redefinition of parameter '__arg_1'`
                        if arg_name.to_string().starts_with(ARG_PREFIX).not()
                    => {
                        *by_ref = None;
                        *mutability = None;
                        arg_name.clone()
                    },

                    | _ => {
                        let arg_name = Ident::new(
                            &format!("{}{}", ARG_PREFIX, i),
                            Span::mixed_site().located_at(pat.span()),
                        );
                        **pat = parse_quote!( #arg_name );
                        arg_name
                    },
                }
            }
        }
    });
    fn arg_tys (fun: &ItemFn)
      -> impl Iterator<Item = &'_ Type>
    {
        fun.sig.inputs.iter().map(|fn_arg| match *fn_arg {
            | FnArg::Typed(PatType { ref ty, .. }) => &**ty,
            | FnArg::Receiver(_) => unreachable!(),
        })
    }
    // C-ize the return type.
    match ffi_fun.sig.output {
        ref mut out @ ReturnType::Default => *out = parse_quote!(
            ->
            ::safer_ffi::ඞ::CLayoutOf<()>
        ),
        ReturnType::Type(_, ref mut ty) => **ty = concrete_c_type(ty),
    }

    let ItemFn {
        sig: Signature {
            ident: ref fname,
            ..
        },
        ..
    } = fun;
    ffi_fun.sig.ident = format_ident!(
        "{}__ffi_export__", fname,
        span = fname.span().resolved_at(Span::mixed_site()),
    );
    ffi_fun.attrs.push(parse_quote!(
        #[cfg_attr(not(target_arch = "wasm32"),
            export_name = #export_name_str,
        )]
    ));
    #[apply(let_quote!)]
    use ::safer_ffi::{
        ඞ,
        layout,
    };
    *ffi_fun.block = parse_quote_spanned!(Span::mixed_site()=> {
        let abort_on_unwind_guard;
        (
            abort_on_unwind_guard = #ඞ::UnwindGuard(#export_name_str),
            unsafe {
                #layout::into_raw(
                    #fname( #(#layout::from_raw_unchecked(#each_arg)),* )
                )
            },
            #ඞ::mem::forget(abort_on_unwind_guard),
        ).1
    });

    #[cfg_attr(not(feature = "js"), allow(unused))]
    let mut js_body = quote!();
    #[cfg(feature = "js")]
    if let Some(args_node_js) = &args.node_js {
        #[apply(let_quote!)]
        use ::safer_ffi::{
            ඞ,
            layout,
            node_js,
            node_js::{napi, ReprNapi},
        };

        let span = Span::mixed_site().located_at(args_node_js.kw.span());
        let fname = &ffi_fun.sig.ident;
        let mut storage = None;
        let export_name =
            if let Some(Rename { ref new_name, .. }) = args.rename {
                storage.get_or_insert(
                    new_name
                        .parse()
                        .expect("checked when parsing args")
                )
            } else {
                &fun.sig.ident
            }
        ;
        let EachArgTyStatic @ _ =
            arg_tys(&fun).vmap(|ty| {
                let mut ty = ty.clone();
                visit_mut::VisitMut::visit_type_mut(
                    &mut utils::RemapNonStaticLifetimesTo { new_lt_name: "static" },
                    &mut ty,
                );
                ty
            })
        ;
        let EachArgTyJs @ _ =
            EachArgTyStatic.iter().map(|ty| quote!(
                <#ඞ::CLayoutOf<#ty> as #ReprNapi>::NapiValue
            ))
        ;
        let each_arg_spanned_at_fun = each_arg.iter().map(|arg| {
            format_ident!("{}", arg, span = arg.span().located_at(fun.sig.ident.span()))
        });
        let ty_aliases = quote_spanned!(span=>
            // We want to use `type $arg_name = <$arg_ty as …>::Assoc;`
            // (with the lifetimes appearing there having been replaced with
            // `'static`, to soothe `#[wasm_bindgen]`).
            //
            // To avoid polluting the namespace with that many `$arg_name`s,
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
                    // (this is for versions of Rust somehow ignoring the
                    // `elided_lifetimes_in_paths` lint)
                    pub(in super)
                    type #each_arg_spanned_at_fun = #EachArgTyJs;
                )*
            }
        );
        let EachArgTyJs = each_arg.iter().map(|arg| quote!(
            __ty_aliases::#arg
        ));
        let (generics, _, where_clause) = fun.sig.generics.split_for_impl();
        let body = |call_and_return| quote_spanned!(span=>
            const _: () = {
                #ty_aliases

                #[#node_js::derive::js_export(js_name = #export_name)]
                fn __node_js #generics (
                    #( #each_arg: #EachArgTyJs ),*
                ) -> #node_js::Result<#node_js::JsUnknown>
                #where_clause
                {
                    let __ctx__ = #node_js::derive::__js_ctx!();
                    #(
                        let #each_arg: #ඞ::CLayoutOf<#EachArgTyStatic> =
                            #node_js::ReprNapi::from_napi_value(
                                __ctx__.env,
                                #each_arg,
                            )?
                        ;
                    )*

                    #call_and_return
                }
            };
        );
        // where
        let call_and_return = if let Some(async_worker) = &args_node_js.async_worker {
            quote_spanned!(Span::mixed_site().located_at(async_worker.span())=>
                #[cfg(not(target_arch = "wasm32"))] {
                    #node_js::JsPromise::from_task_spawned_on_worker_pool(
                        __ctx__.env,
                        unsafe {
                            fn __assert_send<__T : #ඞ::marker::Send>() {}
                            #(
                                let #each_arg = {
                                    // The raw `CType` may not be `Send` (_e.g._, it
                                    // may be a raw pointer), but we can turn off the
                                    // lint if the `ReprC` whence it originated is
                                    // `Send`.
                                    let _ = __assert_send::<#EachArgTyStatic>;
                                    #node_js::UnsafeAssertSend::new(#each_arg)
                                };
                            )*
                            move || {
                                #(
                                    let #each_arg = {
                                        #node_js::UnsafeAssertSend::into_inner(#each_arg)
                                    };
                                )*
                                #fname( #(#each_arg),* )
                            }
                        },
                    )
                    .map(|it| it.into_unknown())
                }

                #[cfg(target_arch = "wasm32")] {
                    let ret = unsafe {
                        #fname( #(#each_arg),* )
                    };
                    #ReprNapi::to_napi_value(ret, __ctx__.env)
                        .map(|it| #node_js::JsPromise::<#node_js::JsUnknown>::resolve(
                            it.as_ref(),
                        ))
                        .map(|it| it.into_unknown())
                }
            )
        } else {
            quote_spanned!(Span::mixed_site()=>
                let ret = unsafe {
                    #fname( #(#each_arg),* )
                };
                #ReprNapi::to_napi_value(ret, __ctx__.env)
                    .map(|it| it.into_unknown())
            )
        };
        js_body.extend(body(call_and_return));
    };

    let mut fun = fun;
    fun.block.stmts.insert(0, parse_quote!(
        {
            #ffi_fun
            #js_body
        }
    ));

    let mut ret = fun.to_token_stream();

    if cfg!(feature = "headers") {
        let_quote!(use ::safer_ffi::headers);
        let mut storage = None;
        let RetTy @ _ = match fun.sig.output {
            | ReturnType::Default => &*storage.get_or_insert(
                Type::Verbatim(quote!( () ))
            ),
            | ReturnType::Type(_, ref ty) => &**ty,
        };
        let ref EachArgTy @ _ = arg_tys(&fun).vec();
        let each_doc = utils::extract_docs(&fun.attrs)?;
        let (generics, _, where_clause) = fun.sig.generics.split_for_impl();
        ret.extend(quote!(
            #[cfg(not(target_arch = "wasm32"))]
            #ඞ::inventory::submit! {
                #![crate = #ඞ]
                #ඞ::FfiExport {
                    name: #export_name_str,
                    gen_def: {
                        fn gen_def #generics (
                            definer: &'_ mut dyn #ඞ::Definer,
                            lang: #headers::Language,
                        ) -> #ඞ::io::Result<()>
                        #where_clause
                        {#ඞ::io::Result::<()>::Ok({
                            // FIXME: this merges the value namespace with the type
                            // namespace...
                            if ! definer.insert(#export_name_str) {
                                return #ඞ::result::Result::Err(
                                    #ඞ::io::Error::new(
                                        #ඞ::io::ErrorKind::AlreadyExists,
                                        #ඞ::concat!(
                                            "Error, attempted to declare `",
                                            #export_name_str,
                                            "` while another declaration already exists",
                                        ),
                                    )
                                );
                            }
                        #(
                            #headers::__define_self__::<#EachArgTy>(definer, lang)?;
                        )*
                            #headers::__define_self__::<#RetTy>(definer, lang)?;
                            #headers::__define_fn__(
                                definer,
                                lang,
                                &[ #(#each_doc),* ],
                                #export_name_str,
                                &[
                                    #(
                                        #ඞ::FunctionArg {
                                            name: #ඞ::stringify!(#each_arg),
                                            ty: &#ඞ::PhantomData::<
                                                #ඞ::CLayoutOf<#EachArgTy>,
                                            >,
                                        }
                                    ),*
                                ],
                                &#ඞ::PhantomData::<
                                    #ඞ::CLayoutOf< #RetTy >,
                                >,
                            )?;
                        })}
                        gen_def
                    },
                }
            }
        ));
    }

    Ok(ret)
}
