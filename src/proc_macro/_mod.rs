#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt::skip)]
#![allow(nonstandard_style, unused_imports)]
#![allow(warnings)] // #![feature(proc_macro_span)]

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

macro_rules! todo {(
    $( $fmt:expr $(, $($rest:tt)* )? )?
) => (
    ::core::todo! {
        concat!(file!(), ":", line!(), ":", column!(), " ({})"),
        ::core::format_args!(
            concat!($($fmt)?),
            $($( $($rest)* )?)?
        ),
    }
)}

#[macro_use]
extern crate macro_rules_attribute;

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
    // Compatibility with legacy mode.
    if cfg!(feature = "js")
    && {
        fn parse_attrs (input: ParseStream<'_>)
          -> Result<Vec<Attribute>>
        {
            Ok((
                Attribute::parse_outer(input)?,
                input.parse::<TokenStream2>()
            ).0)
        }

        let attrs = {
            let input = input.clone();
            parse_macro_input!(input with parse_attrs)
        };
        attrs.iter().any(|attr| {
            attr.path.is_ident("repr")
            &&
            attr.parse_args_with(Punctuated::<Ident, Token![,]>::parse_terminated)
                .unwrap()
                .iter()
                .any(|repr| repr == "nodejs")
        })
    }
    {
        return unwrap!(derives::derive_ReprC(attrs.into(), input.into()));
    }
    unwrap!(
        derives::repr_c::derive(attrs.into(), input.into())
            .map(utils::mb_file_expanded)
    )

}

// #[proc_macro_attribute] pub
// fn derive_CType (
//     attrs: TokenStream,
//     input: TokenStream,
// ) -> TokenStream
// {
//     unwrap!(derives::derive_CType(attrs.into(), input.into()))
// }

#[proc_macro_attribute] pub
fn derive_ReprC2 (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    unwrap!(
        derives::repr_c::derive(attrs.into(), input.into())
            .map(utils::mb_file_expanded)
    )
}

#[doc(hidden)] /** Not part of the public API */ #[proc_macro] pub
fn __respan (
    input: TokenStream,
) -> TokenStream
{
    // use ::proc_macro::*;
    use ::proc_macro2::{*, TokenStream as TokenStream2};
    let parser = |input: ParseStream<'_>| Result::Ok({
        let mut contents;
        ({
            parenthesized!(contents in input);
            let tt: TokenTree = contents.parse()?;
            let _: TokenStream2 = contents.parse()?;
            tt.span()
        }, {
            parenthesized!(contents in input);
            contents.parse::<TokenStream2>()?
        })
    });
    let (span, tts) = parse_macro_input!(input with parser);
    return respan(span, tts.into()).into();
    // where:
    fn respan (
        span: Span,
        tts: TokenStream2,
    ) -> TokenStream2
    {
        tts.into_iter().map(|mut tt| {
            if let TokenTree::Group(ref mut g) = tt {
                let g_span = g.span();
                *g = Group::new(
                    g.delimiter(),
                    respan(span, g.stream()),
                );
                g.set_span(/* span.located_at */ g_span)
            } else {
                tt.set_span(
                    span //.located_at(tt.span())
                );
            }
            tt
        }).collect()
    }
}
