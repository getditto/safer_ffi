use super::*;

#[cfg(feature = "js")]
pub(in crate)
mod js;

pub(in crate)
mod struct_;

pub
struct Args {
    rename: Option<Ident>,
}

impl Parse for Args {
    fn parse (input: ParseStream<'_>)
      -> Result<Args>
    {
        let mut ret = Args {
            rename: None,
        };

        let snoopy = input.lookahead1();
        while input.is_empty().not() {
            mod kw {
                ::syn::custom_keyword!(rename);
            }
            match () {
                | _case if snoopy.peek(kw::rename) => {
                    let _: kw::rename = input.parse().unwrap();
                    let _: Token![=] = input.parse()?;
                    if ret.rename.replace(input.parse()?).is_some() {
                        return Err(input.error("duplicate attribute"));
                    }
                },
                | _default => return Err(snoopy.error()),
            }
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(ret)
    }
}

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
