use super::*;

#[cfg(feature = "dyn-traits")]
#[path = "dyn_trait/_mod.rs"]
mod dyn_trait;

mod handle_fptr;

fn feed_to_macro_rules (
    input: TokenStream2,
    name: Ident,
) -> Result<TokenStream2>
{
    let input = parse2::<DeriveInput>(input)?;
    if let Some(expansion) = handle_fptr::try_handle_fptr(&input) {
        return Ok(expansion);
    }
    let DeriveInput {
        attrs,
        vis,
        ident,
        generics,
        data,
    } = input;
    let ret = match data {
        | Data::Enum(DataEnum {
            enum_token: ref enum_,
            ref variants,
            ..
        }) => quote! {
            ::safer_ffi::layout::ReprC! {
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
                ::safer_ffi::layout::#name! {
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
    };
    #[cfg(feature = "verbose-expansions")]
    println!("{}", ret);
    Ok(ret)
}

pub(in crate)
fn derive_ReprC (
    mut attrs: TokenStream2,
    mut input: TokenStream2,
) -> Result<TokenStream2>
{
    #![cfg_attr(not(feature = "dyn-traits"), allow(unused_mut))]
    #[cfg(feature = "dyn-traits")]
    if let Some(output) = dyn_trait::try_handle_trait(&mut attrs, &mut input)? {
        return Ok(utils::mb_file_expanded(output));
    }
    //     | Err(mut err) => {
    //         // Prefix error messages with `derive_ReprC`.
    //         {
    //             let mut errors =
    //                 err .into_iter()
    //                     .map(|err| Error::new_spanned(
    //                         err.to_compile_error(),
    //                         format_args!("`#[safer_ffi::derive_ReprC]`: {}", err),
    //                     ))
    //             ;
    //             err = errors.next().unwrap();
    //             errors.for_each(|cur| err.combine(cur));
    //         }
    //         input.extend(TokenStream::from(err.to_compile_error()));
    //         return input;
    //     },
    //     | Ok(None) => {},
    // }
    if let Some(tt) = TokenStream2::from(attrs).into_iter().next() {
        return Err(Error::new_spanned(tt, "Unexpected parameter"));
    }
    feed_to_macro_rules(input, parse_quote!(ReprC))
}

pub(in crate)
fn derive_CType (
    attrs: TokenStream2,
    input: TokenStream2,
) -> Result<TokenStream2>
{
    if let Some(unexpected_tt) = attrs.into_iter().next() {
        return Err(Error::new_spanned(unexpected_tt, "Unexpected parameter"));
    }
    feed_to_macro_rules(input, parse_quote!(CType))
}
