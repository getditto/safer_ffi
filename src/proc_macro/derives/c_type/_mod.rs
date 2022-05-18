use super::*;

use args::Args;
mod args;

#[cfg(feature = "js")]
pub(in crate)
mod js;

pub(in crate)
mod struct_;

pub(in crate)
fn derive (
    args: TokenStream2,
    input: TokenStream2
) -> Result<TokenStream2>
{
    let args: Args = parse2(args)?;

    let input: DeriveInput = parse2(input)?;
    let DeriveInput {
        ref attrs,
        ref vis,
        ref ident,
        ref generics,
        ref data,
    } = input;
    let mut ret = match data {
        | Data::Struct(DataStruct { fields, .. }) => struct_::derive(
            args,
            attrs,
            vis,
            ident,
            generics,
            fields,
        ),
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
