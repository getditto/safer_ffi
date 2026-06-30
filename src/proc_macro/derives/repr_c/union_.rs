use syn::token::{Union, Unsafe};

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
            | "transparent" => todo!(),

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
            CType,
            ReprC,
            OpaqueKind
        },
    };

    let each_field_ty = || fields.named.iter().map(|Field { ty, .. }| ty);

    let each_field_name = || fields.named.iter().map(|f| f.ident.as_ref().unwrap());

    let ref ctype_generics = utils::ctype_generics(generics, &mut each_field_ty());

    let ref union_name_layout = format_ident!("{}_Layout", UnionName);

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

    let EachFieldTy = each_field_ty();
    let each_field_name = each_field_name();
    let (intro_generics, fwd_generics, where_clauses) = ctype_generics.split_for_impl();

    let c_type_impl = quote! {
        #[allow(trivial_bounds)]
        unsafe
        impl #intro_generics
            #ReprC
        for
            #UnionName #fwd_generics
        #where_clauses
        {
            type CLayout = #union_name_layout #fwd_generics;

            #[inline]
            fn is_valid (_it: &'_ Self::CLayout)
              -> #ඞ::bool
            {
                let mut _ret = true;
                #(
                    if #ඞ::mem::size_of::<#EachFieldTy>() != 0
                    && unsafe {
                            <#EachFieldTy as #ReprC>::is_valid(
                            &_it.#each_field_name
                        ) == false
                    }
                    {
                        #ඞ::__error__!(
                            "\
                                Encountered invalid bit-pattern \
                                for field `.{}` \
                                of type `{}`: \
                                got `{:02x?}`\
                            ",
                            #ඞ::stringify!(#each_field_name),
                            #ඞ::any::type_name::<#EachFieldTy>(),
                            unsafe {
                                #ඞ::slice::from_raw_parts(
                                    <*const _>::cast::<#ඞ::u8>(&_it.#each_field_name),
                                    #ඞ::mem::size_of_val(&_it.#each_field_name),
                                )
                            },
                        );
                        _ret = false;
                    }
                )*
                _ret
            }
        }
    };

    let (intro_generics, fwd_generics, where_clauses) = &generics.split_for_impl();

    let trivial_impls = trivial_impls(
        intro_generics,
        fwd_generics,
        where_clauses,
        union_name_layout,
    );

    let mut impl_body = quote!(
        type OPAQUE_KIND = #OpaqueKind::Concrete;
    );

    let c_type_layout_impl = quote! {
        unsafe
            impl #intro_generics
                #CType
            for
                #union_name_layout #fwd_generics
            #where_clauses
            {
                #impl_body
            }

            #trivial_impls
    };

    ret.extend(c_type_def.into_token_stream());

    ret.extend(c_type_impl);

    ret.extend(c_type_layout_impl);

    attrs.extend_::<Attribute, _>([
        parse_quote!(
            /// # C Layout
        ),
        parse_quote!(
            ///
        ),
        {
            let line = format!("{}  - [`{UnionName}_Layout`](#impl-ReprC)", " ",);
            parse_quote!(#[doc = #line])
        },
    ]);

    Ok(ret)
}

pub(crate) fn derive_opaque(
    args: Args,
    attrs: &'_ mut Vec<Attribute>,
    pub_: &'_ Visibility,
    StructName @ _: &'_ Ident,
    generics: &'_ Generics,
) -> Result<TokenStream2> {
    todo!("hawdawda")
}

fn docs_of(attrs: &'_ [Attribute]) -> impl '_ + Iterator<Item = &'_ Attribute> {
    attrs.iter().filter(|a| a.path().is_ident("doc"))
}

fn trivial_impls(
    intro_generics: &dyn ToTokens,
    fwd_generics: &dyn ToTokens,
    where_clauses: &dyn ToTokens,
    StructName @ _: &dyn ToTokens,
) -> TokenStream2 {
    #[rustfmt::skip]
    #[apply(let_quote)]
    use ::safer_ffi::ඞ;

    quote!(
        impl #intro_generics
            #ඞ::Clone
        for
            #StructName #fwd_generics
        #where_clauses
        {
            #[inline]
            fn clone (self: &'_ Self)
              -> Self
            {
                *self
            }
        }

        impl #intro_generics
            #ඞ::Copy
        for
            #StructName #fwd_generics
        #where_clauses
        {}

        // If it is CType, it trivially is ReprC.
        unsafe
        impl #intro_generics
            #ඞ::ReprC
        for
            #StructName #fwd_generics
        #where_clauses
        {
            type CLayout = Self;

            #[inline]
            fn is_valid (
                _: &'_ Self::CLayout,
            ) -> #ඞ::bool
            {
                true
            }
        }
    )
}
