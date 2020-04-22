#![allow(nonstandard_style, unused_imports)]

extern crate proc_macro;

use ::proc_macro::TokenStream;
#[cfg(feature = "proc_macros")]
use ::{
    proc_macro2::{
        Span,
        TokenStream as TokenStream2,
    },
    quote::{
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
        Result,
    },
};

include!("utils.rs");

#[cfg(feature = "proc_macros")]
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
    let ret = TokenStream::from(match data {
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
            quote! {
                ::repr_c::layout::#name! {
                    #(#attrs)*
                    #vis
                    #struct_ #ident
                                [#params]
                            where {
                                #(#bounds ,)*
                            }
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
    });
    #[cfg(feature = "verbose-expansions")]
    println!("{}", ret.to_string());
    ret
}

#[cfg(feature = "proc_macros")]
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

#[cfg(feature = "proc_macros")]
#[proc_macro_attribute] pub
fn derive_CType (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    if let Some(unexpected_tt) = attrs.into_iter().next() {
        return compile_error("Unexpected parameter", unexpected_tt.span());
    }
    feed_to_macro_rules(input, parse_quote!(CType))
}

#[proc_macro_attribute] pub
fn ffi_export (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    use ::proc_macro::{*, TokenTree as TT};
    if let Some(unexpected_tt) = attrs.into_iter().next() {
        return compile_error("Unexpected parameter", unexpected_tt.span());
    }
    #[cfg(feature = "proc_macros")] {
        let input = input.clone();
        let _: ItemFn = parse_macro_input!(input);
    }
    let span = Span::call_site();
    <TokenStream as ::std::iter::FromIterator<_>>::from_iter(vec![
        TT::Punct(Punct::new(':', Spacing::Joint)),
        TT::Punct(Punct::new(':', Spacing::Alone)),

        TT::Ident(Ident::new("repr_c", span)),

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
