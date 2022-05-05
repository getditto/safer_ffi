/// Export a function to be callable by C.
///
/// # Example
///
/// ```rust
/// use ::safer_ffi::prelude::ffi_export;
///
/// #[ffi_export]
/// /// Add two integers together.
/// fn add (x: i32, y: i32) -> i32
/// {
///     x + y
/// }
/// ```
///
///   - ensures that [the generated headers](/safer_ffi/headers/) will include the
///     following definition:
///
///     ```C
///     #include <stdint.h>
///
///     /* \brief
///      * Add two integers together.
///      */
///     int32_t add (int32_t x, int32_t y);
///     ```
///
///   - exports an `add` symbol pointing to the C-ABI compatible
///     `int32_t (*)(int32_t x, int32_t y)` function.
///
///     (The crate type needs to be `cdylib` or `staticlib` for this to work,
///     and, of course, the C compiler invocation needs to include
///     `-L path/to/the/compiled/library -l name_of_your_crate`)
///
///       - when in doubt, use `staticlib`.
///
/// # `ReprC`
///
/// [`ReprC`]: /safer_ffi/layout/trait.ReprC.html
///
/// You can use any Rust types in the singature of an `#[ffi_export]`-
/// function, provided each of the types involved in the signature is [`ReprC`].
///
/// Otherwise the layout of the involved types in the C world is **undefined**,
/// which `#[ffi_export]` will detect, leading to a compilation error.
///
/// To have custom structs implement [`ReprC`], it suffices to annotate the
/// `struct` definitions with the [`#[derive_ReprC]`](
/// /safer_ffi/layout/attr.derive_ReprC.html)
/// (on top of the obviously required `#[repr(C)]`).
#[proc_macro_attribute] pub
fn ffi_export (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    use ::proc_macro::{*, TokenTree as TT};
    #[cfg(feature = "proc_macros")]
    if let Ok(input) = parse::<DeriveInput>(input.clone()) {
        parse_macro_input!(attrs as parse::Nothing);
        return ::quote::quote!(
            ::safer_ffi::__ffi_export__! {
                #input
            }
        ).into();
    }
    #[cfg(feature = "async-fn")]
    let fun: ItemFn = {
        let input = input.clone();
        parse_macro_input!(input)
    };
    #[cfg(feature = "async-fn")] {
        let attrs = attrs.clone();
        match parse::<async_fn::Attrs>(attrs) {
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
                                | Some(extraneous_tt) => return compile_error(
                                    "Unexpected parameter",
                                    extraneous_tt.span(),
                                ),
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
                        return compile_error(
                            "Duplicate `nodejs` parameter",
                            kw.span(),
                        );
                    }
                }
            },
            | Some(unexpected_tt) => return compile_error(
                "Unexpected parameter", unexpected_tt.span(),
            ),
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
    <TokenStream as ::std::iter::FromIterator<_>>::from_iter(vec![
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
    ])
}

#[cfg(feature = "async-fn")]
mod async_fn;
