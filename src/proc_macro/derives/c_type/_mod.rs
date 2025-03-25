use args::Args;

use super::*;
mod args;

#[cfg(feature = "js")]
pub(crate) mod js;

pub(crate) mod struct_;

pub(crate) fn derive(
    args: TokenStream2,
    input: TokenStream2,
) -> Result<TokenStream2> {
    let args: Args = parse2(args)?;

    let input: DeriveInput = parse2(input)?;
    let DeriveInput {
        ref attrs,
        ref vis,
        ref ident,
        ref generics,
        ref data,
    } = input;
    let ret = match data {
        | Data::Struct(DataStruct { fields, .. }) => {
            struct_::derive(args, attrs, vis, ident, generics, fields)
        },
        | Data::Enum(DataEnum { enum_token, .. }) => bail! {
            "\
                an `enum` does not have a *fully safe* backing `CType`; \
                did you mean to implement `ReprC` instead?\
            " => enum_token
        },
        | Data::Union(DataUnion { union_token, .. }) => bail! {
            "`union`s are not supported yet" => union_token
        },
    }?;
    Ok(quote!(
        #input

        #ret
    ))
}
