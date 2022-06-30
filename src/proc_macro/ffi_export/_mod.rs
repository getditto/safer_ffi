use super::*;

#[allow(unused_macros)]
macro_rules! emit {( $($tt:tt)* ) => ( $($tt)* )}

pub(in super)
fn ffi_export (
    attrs: TokenStream2,
    input: TokenStream2,
) -> Result<TokenStream2>
{
    use ::proc_macro2::*;

    if let Ok(input) = parse2::<DeriveInput>(input.clone()) {
        let _: parse::Nothing = parse2(attrs)?;
        return Ok(::quote::quote!(
            ::safer_ffi::__ffi_export__! {
                #input
            }
        ));
    }

    if let Ok(input) = parse2::<ItemConst>(input.clone()) {
        let _: parse::Nothing = parse2(attrs)?;
        return Ok(::quote::quote!(
            ::safer_ffi::__ffi_export__! {
                #input
            }
        ));
    }

    #[cfg(feature = "async-fn")] emit! {
        let fun: ItemFn = parse2(input.clone())?;
        match parse2::<async_fn::Attrs>(attrs.clone()) {
            | Ok(attrs)
                if attrs.block_on.is_some()
                || fun.sig.asyncness.is_some()
            => {
                return async_fn::export(attrs, &fun);
            },
            | _ => {},
        }
    }
    let ref mut attr_tokens = attrs.into_iter().peekable();
    #[cfg(feature = "node-js")]
    let mut node_js = None;
    loop {
        match attr_tokens.next() {
            | Some(TT::Ident(kw)) if kw.to_string() == "node_js" => {
                let mut is_async_worker = false;
                match attr_tokens.peek() {
                    | Some(TT::Group(g)) if matches!(g.delimiter(), Delimiter::Parenthesis) => {
                        let mut tts = g.stream().into_iter().peekable();
                        loop {
                            match tts.next() {
                                | None => break,
                                | Some(TT::Ident(id)) if id.to_string() == "async_worker" => {
                                    is_async_worker = true;
                                },
                                | Some(extraneous_tt) => return Err(Error::new_spanned(
                                    extraneous_tt,
                                    "Unexpected parameter",
                                )),
                            }
                            if matches!(
                                tts.peek(),
                                Some(TT::Punct(p)) if p.as_char() == ','
                            )
                            {
                                let _ = tts.next();
                            }
                        }
                        let _consume_group = attr_tokens.next();
                    },
                    | _ => {},
                }
                let _ = is_async_worker;
                #[cfg(feature = "node-js")] {
                    let prev = node_js.replace((
                        ::proc_macro2::Literal::usize_unsuffixed(fun.sig.inputs.len()),
                        is_async_worker,
                    ));
                    if prev.is_some() {
                        bail! {
                            "Duplicate `nodejs` parameter" => kw
                        }
                    }
                }
            },
            | Some(unexpected_tt) => bail! {
                "Unexpected parameter" => unexpected_tt
            },
            | None => break,
        }
        if matches!(attr_tokens.peek(), Some(TT::Punct(p)) if p.as_char() == ',') {
            let _ = attr_tokens.next();
        }
    }
    #[cfg(feature = "node-js")]
    let input = if let Some((arg_count, is_async_worker)) = node_js {
        let is_async_worker = if is_async_worker {
            Some(::quote::quote!(
                "async_worker",
            ))
        } else {
            None
        };
        let mut ts = TokenStream::from(::quote::quote!(
            @[node_js(#arg_count, #is_async_worker)]
        ));
        ts.extend(input);
        ts
    } else {
        input
    };
    let span = Span::call_site();
    Ok(TokenStream2::from_iter(vec![
        TT::Punct(Punct::new(':', Spacing::Joint)),
        TT::Punct(Punct::new(':', Spacing::Alone)),

        TT::Ident(Ident::new("safer_ffi", span)),

        TT::Punct(Punct::new(':', Spacing::Joint)),
        TT::Punct(Punct::new(':', Spacing::Alone)),

        TT::Ident(Ident::new("__ffi_export__", span)),

        TT::Punct(Punct::new('!', Spacing::Alone)),

        TT::Group(Group::new(
            Delimiter::Brace,
            input.into_iter().collect(),
        )),
    ]))
}

#[cfg(feature = "async-fn")]
mod async_fn;
