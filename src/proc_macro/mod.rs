#![allow(nonstandard_style)]

extern crate proc_macro;

use ::proc_macro::TokenStream;
use ::proc_macro2::{
    Span,
    TokenStream as TokenStream2,
};
use ::quote::{
    quote,
    quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{
        Parse,
        Parser,
    },
    punctuated::Punctuated,
    spanned::Spanned,
    Result,
};

include!("utils.rs");

fn feed_to_macro_rules (input: TokenStream, name: Ident)
  -> TokenStream
{
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = parse_macro_input!(input);
    TokenStream::from(match data {
        | Data::Enum(DataEnum {
            enum_token: ref enum_,
            ref variants,
            ..
        }) => quote! {
            ::repr_c::layout::ReprC! {
                #(#attrs)*
                #vis
                #enum_ #ident {
                    #variants
                }
            }
        },
        | Data::Struct(DataStruct {
            struct_token: ref struct_,
            ref fields,
            semi_token: ref maybe_semi_colon,
        }) => {
            let (params, bounds) = generics.my_split();
            let generics = quote!();
            quote! {
                ::repr_c::layout::#name! {
                    #(#attrs)*
                    #vis
                    #struct_ #ident
                                [#params] where { #(#bounds ,)* }
                        #fields
                    #maybe_semi_colon
                }
            }
        },
        | Data::Union(ref union_) => {
            Error::new_spanned(
                union_.union_token,
                "`union`s are not supported yet."
            ).to_compile_error()
        },
    })
}

#[proc_macro_attribute] pub
fn derive_ReprC (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    if let Some(tt) = TokenStream2::from(attrs).into_iter().next() {
        return Error::new_spanned(tt,
            "Unexpected parameter",
        ).to_compile_error().into();
    }
    feed_to_macro_rules(input, parse_quote!(ReprC))
}

#[proc_macro_attribute] pub
fn derive_CType (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    if let Some(tt) = TokenStream2::from(attrs).into_iter().next() {
        return Error::new_spanned(tt,
            "Unexpected parameter",
        ).to_compile_error().into();
    }
    feed_to_macro_rules(input, parse_quote!(CType))
}

#[proc_macro_attribute] pub
fn ffi_export (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    dbg!(&input);
    let _: ItemFn = parse_macro_input!(input);
    TokenStream::new()
}
