use super::*;

pub(in crate)
mod enum_;

#[cfg(feature = "js")]
pub(in crate)
mod js;

pub(in crate)
mod struct_;

pub(in crate)
fn derive (
    attrs: TokenStream2,
    input: TokenStream2
) -> Result<TokenStream2>
{
    let _: parse::Nothing = parse2(attrs)?;

    let mut input: DeriveInput = parse2(input)?;
    let DeriveInput {
        ref mut attrs,
        ref vis,
        ref ident,
        ref generics,
        ref data,
    } = input;
    let mut ret = match *data {
        | Data::Struct(DataStruct { ref fields, .. }) => struct_::derive(
            attrs,
            vis,
            ident,
            generics,
            fields,
        ),
        | Data::Enum(DataEnum { ref variants, .. }) => enum_::derive(
            attrs,
            vis,
            ident,
            generics,
            variants,
        ),
        | Data::Union(DataUnion { ref union_token, .. }) => bail! {
            "`union`s are not supported yet" => union_token
        },
    }?;
    Ok(quote!(
        #input

        #ret
    ))
}
