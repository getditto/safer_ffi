use super::*;

pub(in super) use args::Args;
mod args;

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
    let args: Args = parse2(attrs)?;

    let mut input: DeriveInput = parse2(input)?;
    if let Some(ret) = super::handle_fptr::try_handle_fptr(&input) {
        return ret;
    }
    let DeriveInput {
        ref mut attrs,
        ref vis,
        ref ident,
        ref generics,
        ref data,
    } = input;
    let ret = match *data {
        | Data::Struct(DataStruct { ref fields, .. }) => struct_::derive(
            args,
            attrs,
            vis,
            ident,
            generics,
            fields,
        ),
        | Data::Enum(DataEnum { ref variants, .. }) => enum_::derive(
            args,
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
