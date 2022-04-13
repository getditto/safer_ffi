#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt::skip)]
#![allow(nonstandard_style, unused_imports)]

use {
    ::core::{
        ops::Not as _,
    },
    ::proc_macro::{
        TokenStream,
    },
    ::proc_macro2::{
        Span,
        TokenStream as TokenStream2,
        TokenTree as TT,
    },
    ::quote::{
        format_ident,
        quote,
        quote_spanned,
        ToTokens,
    },
    ::syn::{*,
        parse::{
            Parse,
            Parser,
            ParseStream,
        },
        punctuated::Punctuated,
        spanned::Spanned,
        Result,
    },
    crate::utils::{
        *,
    },
};

mod c_str;

#[path = "derives/_mod.rs"]
mod derives;

#[path = "ffi_export/_mod.rs"]
mod ffi_export;

#[path = "utils/_mod.rs"]
mod utils;

#[proc_macro_attribute] pub
fn cfg_headers (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    parse_macro_input!(attrs as parse::Nothing);
    if cfg!(feature = "headers") {
        input
    } else {
        <_>::default()
    }
}

#[proc_macro] pub
fn c_str (input: TokenStream)
  -> TokenStream
{
    unwrap!(c_str::c_str(input.into()))
}

#[proc_macro_attribute] pub
fn ffi_export (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    unwrap!(ffi_export::ffi_export(attrs.into(), input.into()))
}

#[proc_macro_attribute] pub
fn derive_ReprC (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    unwrap!(derives::derive_ReprC(attrs.into(), input.into()))
}

#[proc_macro_attribute] pub
fn derive_CType (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    unwrap!(derives::derive_CType(attrs.into(), input.into()))
}
