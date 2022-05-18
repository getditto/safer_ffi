use super::*;

pub(in crate)
fn derive (
    args: Args,
    attrs: &'_ [Attribute],
    vis: &'_ Visibility,
    StructName @ _: &'_ Ident,
    generics: &'_ Generics,
    fields: &'_ Fields,
) -> Result<TokenStream2>
{
    if matches!(fields, Fields::Unnamed { .. } | Fields::Unit { .. }) {
        bail!("only braced structs are supported");
    }

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

    let impl_body = quote!(
        type OPAQUE_KIND = #OpaqueKind::Concrete;
    );

    #[cfg(feature = "headers")]
    let impl_body = {
        let EachGenericTy =
            generics.type_params().map(|it| &it.ident)
        ;
        let ref EachFieldTy =
            fields.iter().vmap(|Field { ty, .. }| ty)
        ;
        let ref each_field_name =
            fields.iter().vmap(|f| f.ident.as_ref().unwrap())
        ;
        let ref each_field_name_str =
            each_field_name.iter().vmap(ToString::to_string)
        ;
        let ref StructName_str =
            args.rename.map_or_else(
                || StructName.to_string().into_token_stream(),
                ToTokens::into_token_stream,
            )
        ;

        let mut impl_body = impl_body;
        impl_body.extend(quote!(
            fn short_name ()
              -> #ඞ::String
            {
                let mut _ret =
                    <#ඞ::String as #ඞ::From<_>>::from(#StructName_str)
                ;
                #(
                    #ඞ::fmt::Write::write_fmt(
                        &mut _ret,
                        #ඞ::format_args!(
                            "_{}",
                            <#CLayoutOf<#EachGenericTy> as #CType>::short_name(),
                        ),
                    ).unwrap();
                )*
                _ret
            }
        ));

        let ref struct_docs = utils::extract_docs(attrs)?;

        let ref each_field: Vec<Quote![ StructField ]> =
            (0..).zip(fields).try_vmap(|(i, f)| Result::Ok({
                let ref field_docs = utils::extract_docs(&f.attrs)?;
                let ref field_name_str = f.ident.as_ref().map_or_else(
                    || format!("_{i}"),
                    Ident::to_string,
                );
                let FieldTy = &f.ty;
                quote!(
                    #ඞ::StructField {
                        docs: &[#(#field_docs),*],
                        name: #field_name_str,
                        ty: &#ඞ::marker::PhantomData::<#FieldTy>,
                    }
                )
            }))?
        ;

        impl_body.extend(quote!(
            fn define_self__impl (
                language: &'_ dyn #headers::languages::HeaderLanguage,
                definer: &'_ mut dyn #headers::Definer,
            ) -> #ඞ::io::Result<()>
            {
            #(
                < #EachFieldTy as #CType >::define_self(language, definer)?;
            )*
                language.emit_struct(
                    definer,
                    &[#(#struct_docs),*],
                    &#ඞ::marker::PhantomData::<Self>,
                    &[#(#each_field),*],
                )
            }
        ));

        impl_body
    };

    ret.extend({
        let (intro_generics, fwd_generics, where_clauses) =
            generics.split_for_impl()
        ;

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

            unsafe
            impl #intro_generics
                #CType
            for
                #StructName #fwd_generics
            #where_clauses
            {
                #impl_body
            }

            // If it is CType, it trivially is ReprC.
            unsafe
            impl #intro_generics
                #ReprC
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
    });

    #[cfg(feature = "js")] {
        ret.extend(super::js::handle(/* … */)?);
    }

    Ok(ret)
}
