use super::*;

#[allow(unexpected_cfgs)]
pub(crate) fn derive(
    args: Args,
    attrs: &'_ [Attribute],
    pub_: &'_ Visibility,
    StructName @ _: &'_ Ident,
    generics: &'_ Generics,
    fields: &'_ Fields,
) -> Result<TokenStream2> {
    if let Some(repr) = attrs.iter().find_map(|attr| {
        bool::then(attr.path().is_ident("repr"), || {
            attr.parse_args::<Ident>().ok()
        })
        .flatten()
    }) {
        if repr.to_string() == "transparent" {
            return derive_transparent(args, attrs, pub_, StructName, generics, fields);
        }
    } else {
        bail!("Missing `#[repr]`!");
    }

    #[rustfmt::skip]
    #[apply(let_quote!)]
    use ::safer_ffi::{
        ඞ,
        headers,
        layout::{
            CLayoutOf,
            CType as CType,
            OpaqueKind,
            ReprC,
        },
    };

    let mut ret = quote!();

    if matches!(fields, Fields::Unnamed { .. } | Fields::Unit { .. }) {
        bail!("only braced structs are supported");
    }

    if cfg!(feature = "js") && args.js.is_some() {
        // invoke the legacy `CType!` macro which is the one currently featuring
        // the js FFI glue generating logic.
        let (params, bounds) = generics.my_split();
        ret.extend(quote!(
            ::safer_ffi::layout::CType! {
                #[repr(C, js)]
                #pub_
                struct #StructName
                    [#params]
                where {
                    #(#bounds ,)*
                }
                #fields
            }
        ))
    }

    let mut impl_body = quote!(
        type OPAQUE_KIND = #OpaqueKind::Concrete;
    );

    if cfg!(feature = "headers") {
        let EachGenericTy = generics.type_params().map(|it| &it.ident);
        let EachConstParam = generics.const_params().map(|param| &param.ident);
        let ref EachFieldTy = fields.iter().vmap(|Field { ty, .. }| ty);
        let ref StructName_str = args.rename.map_or_else(
            || StructName.to_string().into_token_stream(),
            ToTokens::into_token_stream,
        );

        impl_body.extend(quote!(
            fn short_name ()
              -> #ඞ::String
            {
                let mut _ret = #ඞ::format!("{}", #StructName_str);
                #(
                    _ret.push_str(&#ඞ::format!("_{}", <#CLayoutOf<#EachGenericTy> as #CType>::short_name()));
                )*
                #(
                    _ret.push_str(&#ඞ::format!("_{}", #EachConstParam));
                )*
                _ret
            }
        ));

        let ref struct_docs = utils::extract_docs(attrs)?;

        let ref each_field: Vec<Quote![StructField]> = (0..).zip(fields).try_vmap(|(i, f)| {
            Result::Ok({
                let ref field_docs = utils::extract_docs(&f.attrs)?;
                let ref field_name_str = f
                    .ident
                    .as_ref()
                    .map_or_else(|| format!("_{i}"), Ident::to_string);
                let FieldTy = &f.ty;
                quote!(
                    #ඞ::StructField {
                        docs: &[#(#field_docs),*],
                        name: #field_name_str,
                        ty: &#ඞ::marker::PhantomData::<#FieldTy>,
                    }
                )
            })
        })?;

        let ffi_metadata = attrs.iter().find(|attr| { attr.path.is_ident("ffi_metadata") });

        if let Some(ffi_metadata) = ffi_metadata {
            let ptr_type = fields
                .iter()
                .find(|field| field.ident.as_ref().map_or(false, |ident| ident == "ptr"))
                .map(|field| &field.ty)
                .unwrap_or_else(|| panic!("Struct annotated with ffi_metadata attribute does not have field 'ptr'."));

            let result = ffi_metadata.parse_args::<Ident>();

            if let Some(kind) = result.ok() {
                let kind_string = kind.to_string();

                impl_body.extend(quote_spanned!(Span::mixed_site()=>
                    fn metadata_type_usage() -> String {
                        let nested_type = <#ptr_type as #CType>::metadata_type_usage();

                        let indented_nested_type = nested_type
                            .lines()
                            .map(|line| format!("    {}", line))
                            .collect::<alloc::vec::Vec<alloc::string::String>>()
                            .join("\n");

                        format!(
                            "\"kind\": \"{}\",\n\"backingTypeName\": \"{}\",\n\"type\": {{\n{}\n}}",
                            #kind_string,
                            Self::short_name(),
                            indented_nested_type,
                        )
                    }
                ));
            } else {
                bail!("Failed to parse ffi_metadata attribute.");
            }
        } else {
            impl_body.extend(quote_spanned!(Span::mixed_site()=>
                fn metadata_type_usage() -> String {
                    format!("\"kind\": \"{}\",\n\"name\": \"{}\"", "Struct", Self::short_name())
                }
            ));
        }

        let is_built_in_struct = ffi_metadata.is_some();

        impl_body.extend(quote_spanned!(Span::mixed_site()=>
            #[allow(nonstandard_style)]
            fn define_self__impl (
                language: &'_ dyn #headers::languages::HeaderLanguage,
                definer: &'_ mut dyn #headers::Definer,
            ) -> #ඞ::io::Result<()>
            {
            #(
                < #EachFieldTy as #CType >::define_self(language, definer)?;
            )*
                if #is_built_in_struct && !language.must_declare_built_in_types() {
                    return Ok(())
                }

                language.declare_struct(
                    language,
                    definer,
                    &[#(#struct_docs),*],
                    &#ඞ::marker::PhantomData::<Self>,
                    &[#(#each_field),*],
                )
            }
        ));
    }

    ret.extend({
        let (intro_generics, fwd_generics, where_clauses) = &generics.split_for_impl();

        let trivial_impls = trivial_impls(intro_generics, fwd_generics, where_clauses, StructName);

        quote!(
            unsafe
            impl #intro_generics
                #CType
            for
                #StructName #fwd_generics
            #where_clauses
            {
                #impl_body
            }

            #trivial_impls
        )
    });

    // #[cfg(feature = "js")] {
    //     ret.extend(super::js::handle(/* … */)?);
    // }

    Ok(ret)
}

pub(crate) fn derive_transparent(
    args: Args,
    attrs: &'_ [Attribute],
    _pub: &'_ Visibility,
    StructName_Layout @ _: &'_ Ident,
    generics: &'_ Generics,
    fields: &'_ Fields,
) -> Result<TokenStream2> {
    // Example input:
    #[cfg(any())]
    #[derive_CType(js, rename = "dittoffi_string")]
    #[repr(transparent)]
    struct FfiString_Layout(CLayoutOf<char_p::Box>)
    where
        char_p::Box: ReprC;

    #[rustfmt::skip]
    #[apply(let_quote)]
    use ::safer_ffi::ඞ;

    let mut ret = quote!();

    let Args { rename, .. } = &args;

    let docs = utils::extract_docs(attrs)?;

    let CFieldTy @ _ = match fields.iter().next() {
        | Some(f) => &f.ty,
        | None => bail! {
            "`#[repr(transparent)]` requires at least one field" => fields,
        },
    };

    let (intro_generics, fwd_generics, where_clauses) = &generics.split_for_impl();

    ret.extend(quote!(
        unsafe
        impl #intro_generics
            #ඞ::CType
        for
            #StructName_Layout #fwd_generics
        #where_clauses
        {
            type OPAQUE_KIND = <#CFieldTy as #ඞ::CType>::OPAQUE_KIND;

            ::safer_ffi::__cfg_headers__! {
                fn short_name ()
                  -> #ඞ::String
                {
                    #ඞ::String::from(#rename)
                }

                #[allow(nonstandard_style)]
                fn define_self__impl (
                    language: &'_ dyn #ඞ::HeaderLanguage,
                    definer: &'_ mut dyn #ඞ::Definer,
                ) -> #ඞ::io::Result<()>
                {
                    ::core::unimplemented!("directly handled in `define_self()`");
                }

                fn define_self (
                    language: &'_ dyn #ඞ::HeaderLanguage,
                    definer: &'_ mut dyn #ඞ::Definer,
                ) -> #ඞ::io::Result<()>
                {
                    // We have to manyally override `define_self()`, since the
                    // idempotency logic here is a bit more subtle: we may be dealing
                    // with a language which does not support type aliases.
                    //
                    // In that case, we will just be inlining the semantics of the inner
                    // type, completely bypassing the existence of the alias
                    // (in other words, we will be "eagerly" expanding the type alias to
                    // its aliasee).
                    //
                    // Among other things, in that case, the `{short_,}name()` will be that
                    // of the inner type.
                    //
                    // And this is where the subtlety lies: it's the same name
                    // being used as the "idempotency_id" in `define_{once,self}()`.
                    //
                    // So we need to make sure to *first* define the inner type, and only
                    // then self-guard the rest with our extra info.
                    <#CFieldTy as #ඞ::CType>::define_self(language, definer)?;

                    definer.define_once(
                        &Self::name(language),
                        &mut |definer| {
                            if let #ඞ::Some(language) = language.supports_type_aliases() {
                                language.declare_type_alias(
                                    definer,
                                    &[#(#docs),*],
                                    &#ඞ::PhantomData::<Self>,
                                    &#ඞ::PhantomData::<#CFieldTy>,
                                )?;
                            }
                            Ok(())
                        },
                    )?;

                    Ok(())
                }

                // fn metadata_type_usage() -> String {
                //     <#CFieldTy as #ඞ::CType>::metadata_type_usage()
                // }

                fn name (
                    language: &'_ dyn #ඞ::HeaderLanguage,
                ) -> #ඞ::String
                {
                    if language.supports_type_aliases().is_some() {
                        #ඞ::std::format!("{}_t", Self::short_name())
                    } else {
                        <#CFieldTy as #ඞ::CType>::name(language)
                    }
                }

                fn render(
                    out: &'_ mut dyn #ඞ::io::Write,
                    language: &'_ dyn #ඞ::HeaderLanguage,
                ) -> #ඞ::io::Result<()>
                {
                    Self::render_wrapping_var(out, language, #ඞ::None {})
                }

                fn render_wrapping_var(
                    out: &'_ mut dyn #ඞ::io::Write,
                    language: &'_ dyn #ඞ::HeaderLanguage,
                    var_name: #ඞ::Option<&dyn #ඞ::fmt::Display>,
                ) -> #ඞ::io::Result<()>
                {
                    if language.supports_type_aliases().is_some() {
                        #ඞ::write!(
                            out,
                            "{ty}{sep}{var_name}",
                            ty = Self::name(language),
                            sep = if var_name.is_none() { "" } else { " " },
                            var_name = var_name.unwrap_or(&""),
                        )
                    } else {
                        <#CFieldTy as #ඞ::CType>::render_wrapping_var(out, language, var_name)
                    }
                }
            }
        }
    ));

    ret.extend(trivial_impls(
        intro_generics,
        fwd_generics,
        where_clauses,
        StructName_Layout,
    ));

    if cfg!(feature = "js") && args.js.is_some() {
        #[rustfmt::skip]
        #[apply(let_quote)]
        use ::safer_ffi::js;

        ret.extend(quote!(
            impl #intro_generics
                #js::ReprNapi
            for
                #StructName_Layout #fwd_generics
            #where_clauses
            {
                type NapiValue = <#CFieldTy as #js::ReprNapi>::NapiValue;

                fn to_napi_value (
                    self: Self,
                    env: &'_ #js::Env,
                ) -> #js::Result< Self::NapiValue >
                {
                    <#CFieldTy as #js::ReprNapi>::to_napi_value(self.0, env)
                }

                fn from_napi_value (
                    env: &'_ #js::Env,
                    napi_value: Self::NapiValue,
                ) -> #js::Result<Self>
                {
                    let inner = <#CFieldTy as #js::ReprNapi>::from_napi_value(env, napi_value)?;
                    #js::Result::Ok(unsafe { #ඞ::core::mem::transmute::<#CFieldTy, Self>(inner) })
                }
            }
        ));
    }

    Ok(ret)
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
