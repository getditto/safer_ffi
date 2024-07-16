#![cfg_attr(rustfmt, rustfmt::skip)]

use super::*;

fn docs_of (attrs: &'_ [Attribute])
  -> impl '_ + Iterator<Item = &'_ Attribute>
{
    attrs
        .iter()
        .filter(|a| a.path.is_ident("doc"))
}

pub(in crate)
fn derive (
    args: Args,
    attrs: &'_ mut Vec<Attribute>,
    pub_: &'_ Visibility,
    StructName @ _: &'_ Ident,
    generics: &'_ Generics,
    fields: &'_ Fields,
) -> Result<TokenStream2>
{
    if let Some(repr) = attrs.iter().find_map(|attr| {
        bool::then(
            attr.path.is_ident("repr"),
            || attr.parse_args::<Ident>().ok()
        ).flatten()
    })
    {
        match &repr.to_string()[..] {
            | "transparent" => return derive_transparent(
                args,
                attrs,
                pub_,
                StructName,
                generics,
                fields,
            ),

            | "opaque" => return derive_opaque(
                args,
                attrs,
                pub_,
                StructName,
                generics,
            ),

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

    if fields.is_empty() {
        bail!("C requires that structs have at least one field");
    }

    let mut ret = quote!();

    #[apply(let_quote!)]
    use ::safer_ffi::{
        ඞ,
        layout::{
            ConcreteReprC,
            CLayoutOf,
            ReprC,
        },
    };

    let EachFieldTy @ _ = || fields.iter().map(|Field { ty, .. }| ty);
    let each_field_name = || (0..).zip(fields).map(|(i, f)| match f.ident {
        | Some(ref ident) => ident.to_token_stream(),
        | None => Index { index: i, span: f.ty.span() }.into_token_stream(),
    });

    let ref StructName_Layout @ _ = format_ident!("{}_Layout", StructName);

    let ref ctype_generics =
        utils::ctype_generics(generics, &mut EachFieldTy())
    ;
    // define the CType
    ret.extend({
        let c_type_def = ItemStruct {
            attrs: docs_of(attrs).cloned()
                    .chain([
                        parse_quote!(
                            #[allow(nonstandard_style)]
                        ),
                        parse_quote!(
                            #[repr(C)]
                        ),
                    ])
                    .collect()
            ,
            vis: {
                let pub_ = crate::respan(
                    pub_.span().resolved_at(Span::mixed_site()),
                    pub_.to_token_stream(),
                );
                parse_quote!(#pub_)
            },
            struct_token: parse_quote!(struct),
            ident: StructName_Layout.clone(),
            generics: ctype_generics.clone(),
            fields: Fields::Named({
                let EachFieldTy = EachFieldTy();
                let each_field_name = (0_u8..).zip(fields).map(|(i, f)| {
                    match f.ident {
                        | Some(ref ident) => ident.clone(),
                        | None => format_ident!("_{}", i),
                    }
                });
                let each_field_docs = fields.iter().map(|f| {
                    docs_of(&f.attrs).vec()
                });
                parse_quote!({
                    #(
                        #(#each_field_docs)*
                        pub
                        #each_field_name: #CLayoutOf<#EachFieldTy>
                    ),*
                })
            }),
            semi_token: None,
        };

        let rename = args.rename.unwrap_or_else(|| {
            let s = StructName.to_string();
            parse_quote!(#s)
        });

        // allow using `#()*` as `#()?`.
        let js = args.js.as_ref().map_or(&[][..], ::core::slice::from_ref);

        crate::derives::c_type::derive(
            quote!(
                #(#js ,)*
                rename = #rename,
            ),
            c_type_def.into_token_stream(),
        )?
    });

    // Impl ReprC to point to the just defined type
    ret.extend({
        let EachFieldTy @ _ = EachFieldTy();
        let each_field_name = each_field_name();
        let (intro_generics, fwd_generics, where_clauses) =
            ctype_generics.split_for_impl()
        ;
        quote!(
            #[allow(trivial_bounds)]
            unsafe
            impl #intro_generics
                #ReprC
            for
                #StructName #fwd_generics
            #where_clauses
            {
                type CLayout = #StructName_Layout #fwd_generics;

                #[inline]
                fn is_valid (_it: &'_ Self::CLayout)
                  -> #ඞ::bool
                {
                    let mut _ret = true;
                    #(
                        if #ඞ::mem::size_of::<#EachFieldTy>() != 0
                        && <#EachFieldTy as #ReprC>::is_valid(
                            &_it.#each_field_name
                        ) == false
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
        )
    });

    // Add docs about C layout.
    attrs.extend_::<Attribute, _>([
        parse_quote!(
            /// # C Layout
        ),
        parse_quote!(
            ///
        ),
        {
            let line = format!(
                "{}  - [`{StructName}_Layout`](#impl-ReprC)", " ",
            );
            parse_quote!(#[doc = #line])
        },
    ]);

    Ok(ret)
}

pub(in crate)
fn derive_transparent (
    args: Args,
    attrs: &'_ mut Vec<Attribute>,
    pub_: &'_ Visibility,
    StructName @ _: &'_ Ident,
    generics: &'_ Generics,
    fields: &'_ Fields,
) -> Result<TokenStream2>
{
    #[apply(let_quote)]
    use ::safer_ffi::ඞ;

    let mut ret = quote!();

    let FieldTy = match fields.iter().next() {
        | Some(f) => &f.ty,
        | None => bail! {
            "`#[repr(transparent)]` requires at least one field" => fields,
        },
    };

    let ref impl_generics = generics.clone().also(|g| {
        g   .make_where_clause()
            .predicates
            .push(parse_quote!(
                #FieldTy : #ඞ::ReprC
            ))
        ;
    });

    let (intro_generics, fwd_generics, where_clauses) =
        impl_generics.split_for_impl()
    ;

    let inner;
    if let Some(rename) = &args.rename {
        // define the CType
        ret.extend({
            let ref StructName_Layout @ _ = format_ident!("{}_Layout", StructName);

            let c_type_def = ItemStruct {
                attrs:
                    docs_of(attrs)
                        .cloned()
                        .chain([
                            parse_quote!(
                                #[repr(transparent)]
                            ),
                            parse_quote!(
                                #[allow(nonstandard_style)]
                            ),
                        ])
                        .collect()
                ,
                vis: {
                    let pub_ = crate::respan(
                        pub_.span().resolved_at(Span::mixed_site()),
                        pub_.to_token_stream(),
                    );
                    parse_quote!(#pub_)
                },
                struct_token: parse_quote!(struct),
                ident: StructName_Layout.clone(),
                generics: impl_generics.clone(),
                fields: Fields::Unnamed(parse_quote!((
                    #ඞ::CLayoutOf<#FieldTy>,
                    #ඞ::CLayoutOf<::core::marker::PhantomData<fn(&()) -> &mut Self>>,
                ))),
                semi_token: Some(parse_quote!(
                    ;
                )),
            };

            // allow using `#()*` as `#()?`.
            let js = args.js.as_ref().map_or(&[][..], ::core::slice::from_ref);

            let derive_output = crate::derives::c_type::derive(
                quote!(
                    #(#js, )*
                    rename = #rename,
                ),
                c_type_def.to_token_stream(),
            )?;

            quote!(
                #derive_output

                unsafe
                impl #intro_generics
                    #ඞ::ReprC
                for
                    #StructName #fwd_generics
                #where_clauses
                {
                    type CLayout = #StructName_Layout #fwd_generics;

                    #[inline]
                    fn is_valid (it: &'_ Self::CLayout)
                      -> #ඞ::bool
                    {
                        <#FieldTy as #ඞ::ReprC>::is_valid(&it.0)
                    }
                }
            )
        });
        inner = quote!(&it.0);
    } else {
        // Forward ReprC to point to the `CLayoutOf` its first type.
        ret.extend(quote!(
            unsafe
            impl #intro_generics
                #ඞ::ReprC
            for
                #StructName #fwd_generics
            #where_clauses
            {
                type CLayout = #ඞ::CLayoutOf<#FieldTy>;

                #[inline]
                fn is_valid (it: &'_ Self::CLayout)
                  -> #ඞ::bool
                {
                    <#FieldTy as #ඞ::ReprC>::is_valid(it)
                }
            }
        ));
        inner = quote!(it);
    }

    // let mut ret = debug_macro(ret);

    // add niche where applicable.
    ret.extend({
        let niche_generics = impl_generics.clone().also(|g| {
            g   .make_where_clause()
                .predicates
                .push(utils::allowing_trivial_bound(parse_quote!(
                    #FieldTy : #ඞ::__HasNiche__
                )))
        });
        let (intro_generics, fwd_generics, where_clauses) =
            niche_generics.split_for_impl()
        ;
        quote!(
            unsafe
            impl #intro_generics
                #ඞ::__HasNiche__
            for
                #StructName #fwd_generics
            #where_clauses
            {
                #[inline]
                fn is_niche (
                    it: &'_ #ඞ::CLayoutOf<Self>,
                ) -> #ඞ::bool
                {
                    <
                        #FieldTy
                        as
                        #ඞ::__HasNiche__
                    >::is_niche(#inner)
                }
            }
        )
    });

    // Add docs about C layout.
    attrs.extend_::<Attribute, _>([
        parse_quote!(
            /// # C Layout
        ),
        parse_quote!(
            ///
        ),
        {
            let line = format!(
                "{}  - [`{ty}`](#impl-ReprC)", " ",
                ty = FieldTy.to_token_stream(),
            );
            parse_quote!(#[doc = #line])
        },
    ]);

    Ok(ret)
}

pub(in crate)
fn derive_opaque (
    args: Args,
    attrs: &'_ mut Vec<Attribute>,
    pub_: &'_ Visibility,
    StructName @ _: &'_ Ident,
    generics: &'_ Generics,
) -> Result<TokenStream2>
{
    #[apply(let_quote)]
    use ::safer_ffi::ඞ;

    // Strip the `repr(opaque)`
    attrs.retain(|attr| bool::not({
        mod kw { ::syn::custom_keyword!(opaque); }
        attr.path.is_ident("repr")
        && attr.parse_args::<kw::opaque>().is_ok()
    }));

    let mut ret = quote!();

    if let Some(js) = &args.js {
        ret.extend(utils::compile_warning(
            js,
            "`js` annotation is ignored for `repr(opaque)`",
        ));
    }

    let OpaqueStructName = format_ident!(
        "__opaque_{}", StructName,
    );

    let (intro_generics, fwd_generics, where_clauses) =
        generics.split_for_impl()
    ;

    // emit the ReprC
    ret.extend(quote!(
        unsafe
        impl #intro_generics
            #ඞ::ReprC
        for
            #StructName #fwd_generics
        #where_clauses
        {
            type CLayout = #OpaqueStructName #fwd_generics;

            fn is_valid (it: &'_ Self::CLayout)
              -> #ඞ::bool
            {
                match it._void {}
            }
        }
    ));

    // emit the CType
    ret.extend({
        let header_generation = quote!();
        #[cfg(feature = "headers")]
        let header_generation = {
            drop(header_generation);
            let ref short_name: Quote![ String ] = match args.rename {
                | Some(string_expr) => quote!(
                    #ඞ::From::from(#string_expr)
                ),
                | None => {
                    let ref StructName_str = StructName.to_string();
                    let EachGenericParam = generics.type_params().map(|p| &p.ident);
                    quote!(
                        let mut _ret =
                            <#ඞ::String as #ඞ::From<_>>::from(#StructName_str)
                        ;
                        #(
                            #ඞ::append_unqualified_name(
                                &mut _ret,
                                #ඞ::any::type_name::<#EachGenericParam>(),
                            );
                        )*
                        _ret
                    )
                },
            };
            let docs = utils::extract_docs(attrs)?;
            quote!(
                fn short_name ()
                  -> #ඞ::String
                {
                    #short_name
                }

                #[allow(nonstandard_style)]
                fn define_self__impl (
                    language: &'_ dyn #ඞ::HeaderLanguage,
                    definer: &'_ mut dyn #ඞ::Definer,
                ) -> #ඞ::io::Result<()>
                {
                    language.emit_opaque_type(
                        definer,
                        &[#(#docs),*],
                        &#ඞ::PhantomData::<Self>,
                    )
                }
            )
        };

        quote!(
            #[allow(nonstandard_style)]
            #pub_
            struct #OpaqueStructName #intro_generics
            #where_clauses
            {
                _marker: *mut Self,
                _void: #ඞ::convert::Infallible,
            }

            impl #intro_generics
                #ඞ::marker::Copy
            for
                #OpaqueStructName #fwd_generics
            #where_clauses
            {}

            impl #intro_generics
                #ඞ::clone::Clone
            for
                #OpaqueStructName #fwd_generics
            #where_clauses
            {
                fn clone (self: &'_ Self)
                  -> Self
                {
                    *self
                }
            }

            unsafe
            impl #intro_generics
                #ඞ::CType
            for
                #OpaqueStructName #fwd_generics
            #where_clauses
            {
                type OPAQUE_KIND = #ඞ::OpaqueKind::Opaque;

                #header_generation
            }

            unsafe
            impl #intro_generics
                #ඞ::ReprC
            for
                #OpaqueStructName #fwd_generics
            #where_clauses
            {
                type CLayout = Self;

                fn is_valid (
                    _: &'_ Self::CLayout,
                ) -> #ඞ::bool
                {
                    true
                }
            }
        )
    });

    Ok(quote!(
        const _: () = { #ret };
    ))
}
