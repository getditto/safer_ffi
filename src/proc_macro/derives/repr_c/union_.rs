use syn::token::Union;

use super::*;

pub(crate) fn derive(
    args: Args,
    attrs: &'_ mut Vec<Attribute>,
    pub_: &'_ Visibility,
    UnionName @ _: &'_ Ident,
    generics: &'_ Generics,
    fields: &'_ FieldsNamed,
) -> Result<TokenStream2> {
    // at least one field in Union
    if fields.named.is_empty() {
        bail!("union must contain at least one field");
    }

    // NOTE: There’s a transparent_unions nightly feature to apply repr(transparent) to unions,
    // but it hasn’t been stabilized due to design concerns.

    if let Some(repr) = attrs.iter().find_map(|attr| {
        bool::then(attr.path().is_ident("repr"), || {
            attr.parse_args::<Ident>().ok()
        })
        .flatten()
    }) {
        match &repr.to_string()[..] {
            | "opaque" => return derive_opaque(args, attrs, pub_, UnionName, generics),

            | "C" => {},

            | _unsupported => bail! {
                "unsupported `repr`" => repr,
            },
        }
    } else {
        bail! {
            "missing explicit `#[repr(…)]` annotation"
        }
    }

    #[rustfmt::skip]
    #[apply(let_quote!)]
    use ::safer_ffi::{
        ඞ,
        layout::{
            ConcreteReprC,
            CLayoutOf,
            ReprC,
        },
    };

    let each_field_ty @ _ = || fields.named.iter().map(|Field { ty, .. }| ty);

    let each_field_name = || fields.named.iter().map(|f| f.ident.as_ref().unwrap());

    let ref ctype_generics = utils::ctype_generics(generics, &mut each_field_ty());

    let ref union_name_layout @ _ = format_ident!("{}_Layout", UnionName);

    let mut ret = quote!();

    let c_type_def = ItemUnion {
        attrs: docs_of(attrs)
            .cloned()
            .chain([
                parse_quote!(#[allow(nonstandard_style)]),
                parse_quote!(#[repr(C)]),
            ])
            .chain(
                attrs
                    .iter()
                    .filter(|a| a.path().is_ident("ffi_metadata"))
                    .cloned(),
            )
            .collect(),
        vis: {
            let pub_ = crate::respan(
                pub_.span().resolved_at(Span::mixed_site()),
                pub_.to_token_stream(),
            );
            parse_quote!(#pub_)
        },
        union_token: parse_quote!(union),
        ident: union_name_layout.clone(),
        generics: ctype_generics.clone(),
        fields: {
            let each_field_ty = each_field_ty();
            let each_field_name = each_field_name();
            let each_field_docs = fields.named.iter().map(|f| docs_of(&f.attrs).vec());

            parse_quote!({
                #(
                    #(#each_field_docs)*
                    pub #each_field_name: #CLayoutOf<#each_field_ty>
                ),*
            })
        },
    };

    ret.extend(c_type_def.into_token_stream());

    Ok(ret)
}

pub(crate) fn derive_opaque(
    args: Args,
    attrs: &'_ mut Vec<Attribute>,
    pub_: &'_ Visibility,
    StructName @ _: &'_ Ident,
    generics: &'_ Generics,
) -> Result<TokenStream2> {
    todo!()
}

fn docs_of(attrs: &'_ [Attribute]) -> impl '_ + Iterator<Item = &'_ Attribute> {
    attrs.iter().filter(|a| a.path().is_ident("doc"))
}
