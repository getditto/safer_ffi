use super::*;

use args::*;
mod args;

#[cfg(feature = "async-fn")]
mod async_fn;

const SUPPORTED_ABIS: &[&str] = &[
    "C",
];

pub(in super)
fn handle (
    args: Args,
    mut fun: ItemFn,
) -> Result<TokenStream2>
{
    // async fn case.
    if args.executor.is_some() || fun.sig.asyncness.is_some() {
        #[cfg(feature = "async-fn")]
        return async_fn::export(args, &fun);
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

    // *We* handle the C-safety heuristics in a more accurate manner than
    // rustc's lint, so let's disable it to prevent it from firing false
    // positives against us.
    fun.attrs.push(parse_quote!(
        #[allow(improper_ctypes_definitions)]
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
                        // // ensure user-provided names cannot collide with the
                        // // ones we generate.
                        // if arg_name.to_string().starts_with(ARG_PREFIX).not()
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
    let ref fname_str = fname.to_string();
    ffi_fun.sig.ident = format_ident!("{}__ffi_export__", fname);
    ffi_fun.attrs.push(parse_quote!(
        #[export_name = #fname_str]
    ));
    #[apply(let_quote!)]
    use ::safer_ffi::{
        ඞ,
        layout,
    };
    *ffi_fun.block = parse_quote_spanned!(Span::mixed_site()=> {
        let abort_on_unwind_guard;
        (
            abort_on_unwind_guard = #ඞ::UnwindGuard(#fname_str),
            unsafe {
                #layout::into_raw(
                    #fname( #(#layout::from_raw_unchecked(#each_arg)),* )
                )
            },
            #ඞ::mem::forget(abort_on_unwind_guard),
        ).1
    });

    let mut fun = fun;
    fun.block.stmts.insert(0, parse_quote!(
        { #ffi_fun }
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
        let ref EachArgTy @ _ =
            fun.sig.inputs.iter().vmap(|fn_arg| match *fn_arg {
                | FnArg::Typed(PatType { ref ty, .. }) => ty,
                | FnArg::Receiver(_) => unreachable!(),
            })
        ;
        let each_doc = utils::extract_docs(&fun.attrs)?;
        let (generics, _, where_clause) = fun.sig.generics.split_for_impl();
        ret.extend(quote!(
            #[cfg(not(target_arch = "wasm32"))]
            #ඞ::inventory::submit! {
                #![crate = #ඞ]
                #ඞ::FfiExport {
                    name: #fname_str,
                    gen_def: {
                        fn gen_def #generics (
                            definer: &'_ mut dyn #ඞ::Definer,
                            lang: #headers::Language,
                        ) -> #ඞ::io::Result<()>
                        #where_clause
                        {#ඞ::io::Result::<()>::Ok({
                            // FIXME: this merges the value namespace with the type
                            // namespace...
                            if ! definer.insert(#fname_str) {
                                return #ඞ::result::Result::Err(
                                    #ඞ::io::Error::new(
                                        #ඞ::io::ErrorKind::AlreadyExists,
                                        #ඞ::concat!(
                                            "Error, attempted to declare `",
                                            #fname_str,
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
                                #fname_str,
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

fn concrete_c_type (T @ _: &'_ Type)
  -> Type
{
    parse_quote_spanned!(T.span()=>
        <#T as ::safer_ffi::ඞ::ConcreteReprC>::ConcreteCLayout
    )
}
