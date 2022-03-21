#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt::skip)]
#![allow(nonstandard_style, unused_imports)]

extern crate proc_macro;

use ::proc_macro::{Span, TokenStream};

#[cfg(feature = "proc_macros")]
use ::{
    proc_macro2::{
        Span as Span2,
        TokenStream as TokenStream2,
    },
    quote::{
        format_ident,
        quote,
        quote_spanned,
        ToTokens,
    },
    syn::{*,
        parse::{
            Parse,
            Parser,
        },
        punctuated::Punctuated,
        spanned::Spanned,
        visit_mut::VisitMut,
        Result,
    },
};

use ::core::ops::Not as _;

macro_rules! inline_mod {($modname:ident) => (
    include! { concat!(stringify!($modname), ".rs") }
)}

use utils::*;
mod utils;

#[cfg(feature = "proc_macros")]
inline_mod!(derives);

#[cfg(feature = "proc_macros")]
inline_mod!(c_str);

include!("ffi_export/_mod.rs");

#[cfg(feature = "headers")]
#[proc_macro_attribute] pub
fn cfg_headers (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    if let Some(unexpected_tt) = attrs.into_iter().next() {
        return compile_error("Unexpected parameter", unexpected_tt.span());
    }
    input
}

#[cfg(not(feature = "headers"))]
#[proc_macro_attribute] pub
fn cfg_headers (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    if let Some(unexpected_tt) = attrs.into_iter().next() {
        return compile_error("Unexpected parameter", unexpected_tt.span());
    }
    let _ = input;
    TokenStream::new()
}
