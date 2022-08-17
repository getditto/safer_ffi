#![warn(warnings)] // Prevent `-Dwarnings` from causing breakage.
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt::skip)]
#![allow(nonstandard_style, unused_imports)]

use ::core::{mem, ops::Not as _};
use ::proc_macro::TokenStream;
use ::proc_macro2::{Span, TokenTree as TT, TokenStream as TokenStream2};
use ::quote::{
    format_ident,
    quote,
    quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    Result, // explicitly shadow Result
};

#[proc_macro] pub
fn __js_ctx (input: TokenStream)
  -> TokenStream
{
    let _: parse::Nothing = parse_macro_input!(input);
    quote!(
        __js_ctx
    ).into()
}

#[proc_macro_attribute] pub
fn js_export (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    let Attrs { js_name, skip_napi_import } = parse_macro_input!(attrs);
    let mut fun: ItemFn = parse_macro_input!(input);
    // TODO: implement support for non-trivial patterns?
    if let Some(span_of_unsupported_param) =
        fun .sig
            .inputs
            .iter()
            .filter_map(|arg| match arg {
                | &FnArg::Typed(PatType {
                    ref pat,
                    ..
                }) if matches!(**pat, Pat::Ident(_)).not()
                => {
                    Some(pat.span())
                },
                | &FnArg::Receiver(_) => Some(arg.span()),
                | _ => None,
            })
            .next()
    {
        return Error::new(
            span_of_unsupported_param,
            "Unsupported parameter",
        ).into_compile_error().into();
    }
    let inputs_count = ::proc_macro2::Literal::usize_unsuffixed(fun.sig.inputs.len());
    let js_ctx = TokenStream2::from(__js_ctx(TokenStream::new()));
    fun.block.stmts =
        mem::replace(
            &mut fun.sig.inputs,
            parse_quote!(
                #js_ctx: napi::CallContext<'_>,
            ),
        )
        .into_iter()
        .enumerate()
        .map(|(i, binding)| -> Stmt { parse_quote!(
            let #binding = ::safer_ffi::js::derive::#js_ctx!().get(#i)?;
        )})
        .chain(mem::take(&mut fun.block.stmts))
        .collect()
    ;
    // We need to launder the spans for some obscure span resolution bug…
    // fun.block = laundered(fun.block.into_token_stream());
    // let s = fun.block.into_token_stream().to_string();
    // println!("{}", s);
    // fun.block = parse_str(&s).expect("BUG");
    let ref name = fun.sig.ident;
    let js_name = js_name.as_ref().unwrap_or(name);
    let napi_import = if skip_napi_import.is_some() {
        quote!()
    } else {
        quote!(
            use ::safer_ffi::js as napi;
        )
    };
    let stmts = &fun.block.stmts;
    let krate_annotation = cfg!(not(feature = "inventory-0-3-1")).then(|| {
        quote!( #![crate = ::safer_ffi::js::registering] )
    });
    let ret = quote!(
        const _: () = {
            #napi_import

            #[::safer_ffi::js::derive::js_function(#inputs_count)]
            // #fun
            fn #name (
                #js_ctx: napi::CallContext<'_>,
            ) -> napi::Result<impl napi::NapiValue>
            {
                #(#stmts)*
            }

            ::safer_ffi::js::registering::submit! {
                #krate_annotation

                ::safer_ffi::js::registering::NapiRegistryEntry::NamedMethod {
                    name: ::core::stringify!(#js_name),
                    method: #name,
                }
            }
        };
    );
    // ret.to_string().parse().unwrap()
    ret.into()
}

include!("../../../js_export_common.rs");
