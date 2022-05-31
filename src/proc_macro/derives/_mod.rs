use super::*;

#[path = "c_type/_mod.rs"]
mod c_type;

#[path = "repr_c/_mod.rs"]
pub(in crate)
mod repr_c;

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
    if let Some(result) = handle_fptr::try_handle_fptr(&input) {
        return result;
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

#[allow(unused_mut)]
pub(in crate)
fn derive_ReprC (
    mut args: TokenStream2,
    mut input: TokenStream2,
) -> Result<TokenStream2>
{
    #[cfg(feature = "dyn-traits")]
    if let Some(output) = dyn_trait::try_handle_trait(&mut args, &mut input)? {
        return Ok(utils::mb_file_expanded(output));
    }

    // Compatibility with legacy mode (`nodejs` annotations):
    let (mut attrs, rest) = Parser::parse2(
        |input: ParseStream<'_>| Result::<_>::Ok(
            (
                Attribute::parse_outer(input)?,
                input.parse::<TokenStream2>().unwrap(),
            )
        ),
        input,
    )?;
    if let Some(attr) = attrs.iter_mut().find(|a| a.path.is_ident("repr")) {
        let mut idents =
            attr.parse_args_with(
                    Punctuated::<Ident, Token![,]>::parse_terminated,
                )
                .unwrap()
                .vec()
        ;
        if let Some(i) =
            idents
                .iter()
                .position(|repr| repr == "nodejs" || repr == "js")
        {
            // `repr(C, nodejs)` case.
            // Are we targetting js *right now*?
            if cfg!(feature = "js") {
                // Legacy mode.
                if let Some(tt) = TokenStream2::from(args).into_iter().next() {
                    return Err(Error::new_spanned(tt, "Unexpected parameter"));
                }

                input = quote!(#(#attrs)* #rest);
                return feed_to_macro_rules(input, parse_quote!(ReprC)); // .map(utils::mb_file_expanded);
            } else {
                // Otherwise, we might as well not have been covering js to begin with.
                drop(idents.swap_remove(i));
            }
        }
        *attr = parse_quote!(
            #[repr(#(#idents),*)]
        );
    }
    input = quote!(#(#attrs)* #rest);

    derives::repr_c::derive(args, input)
}
