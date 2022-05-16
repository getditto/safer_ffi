use super::*;

pub(in crate)
fn derive (
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
        let ref StructName_str = StructName.to_string();

        #[cfg(feature = "csharp")]
        let impl_body = quote!(
            fn csharp_define_self (
                definer: &'_ mut dyn #headers::Definer,
            ) -> #io::Result<()>
            {
                use #io::Write as _;
                #core::assert_ne!(
                    #mem::size_of::<Self>(), 0,
                    "C# does not support zero-sized structs!",
                );
                let ref me = <Self as #CType>::csharp_ty();
            #(
                <#EachFieldTy as #CType>::csharp_define_self(definer)?;
            )*
                definer.define_once(me, &mut |definer| #writeln!(definer.out(),
                    #concat!(
                        "[StructLayout(LayoutKind.Sequential, Size = {size})]\n",
                        "public unsafe struct {me} {{\n",
                        #(
                            "{}{", #each_field_name_str, "}",
                        )*
                        "}}\n",
                    ),
                #(
                    <#EachFieldTy as #CType>::csharp_marshaler()
                        .map(|m| #format!("    [MarshalAs({})]\n", m))
                        .as_deref()
                        .unwrap_or("")
                    ,
                )*
                    size = #mem::size_of::<Self>(),
                #(
                    #each_field_name = {
                        if #mem::size_of::<#EachFieldTy>() > 0 {
                            #format!(
                                "    public {};\n",
                                <#EachFieldTy as #CType>::csharp_var(
                                    #each_field_name_str,
                                ),
                            )
                        } else {
                            #assert_eq!(
                                #mem::align_of::<#EachFieldTy>(),
                                1,
                                "\
                                    Zero-sized fields must have an \
                                    alignment of `1`.\
                                ",
                            );
                            String::new()
                        }
                    },
                )*
                    me = me,
                ))
            }
        );

        let mut impl_body = impl_body;
        impl_body.extend(quote!(
            fn short_name ()
              -> #ඞ::String
            {
                let mut ret: #ඞ::String = StructName_str.into();
                #(
                    #ඞ::fmt::Write::write_fmt(
                        &mut ret,
                        #ඞ::format_args!(
                            "_{}",
                            <#CLayoutOf<#EachGenericTy> as #CType>::c_short_name(),
                        ),
                    ).unwrap();
                )*
                ret
            }
        ));

        let each_field =
            fields.try_vmap(|f| Result::Ok({
                let ref docs = utils::extract_docs(attrs)?;
                let ref name = f.ident.as_ref().expect("BRACED STRUCT").to_string();
                let FieldTy = &f.ty;
                let emit_unindented = quote!(
                    &|language, definer| #ඞ::io::Write::write_fmt(
                        definer.out(),
                        #ඞ::format_args!(
                            "{}",
                            <#FieldTy as #CType>::name_wrapping_var(
                                language,
                                #name,
                            ),
                        ),
                    )
                );
                quote!(
                    #ඞ::FunctionArg {
                        docs: &[#(#docs),*],
                        name: #name,
                        emit_unindented: #emit_unindented,
                    }
                )
            }))?
        ;

        let ref docs = utils::extract_docs(attrs)?;
        impl_body.extend(quote!(
            fn define_self (
                language: &'_ dyn #headers::HeaderLanguage,
                definer: &'_ mut dyn #headers::Definer,
            ) -> #ඞ::io::Result<()>
            {
                let ref me =
                    <Self as #CType>::short_name()
                        .to_string()
                ;
                definer.define_once(me, &mut |definer| {
                    language.emit_struct(
                        definer,
                        me,
                        &[#(#docs),*],
                        #ඞ::mem::size_of::<Self>(),
                        &[#(#each_field),*],
                    )
                })

                //     #(
                //         < #EachFieldTy as #CType >::c_define_self(definer)?;
                //     )*
                //     // /* FIXME: handle docs */
                //     // // let out = definer.out();
                //     // // $(
                //     // //     $crate::__output_docs__!(out, "", $($doc_meta)*);
                //     // // )?
                //     // // $crate::__output_docs__!(out, "", $(#[$($meta)*])*);
                //     // #core::writeln!(out, "typedef struct {{\n")?;
                //     #(
                //         if #ඞ::mem::size_of::< #EachFieldTy >() > 0 {
                //             // $crate::core::writeln!(out, "")?;
                //             /* FIXME: docs */
                //             // $crate::__output_docs__!(out, "    ",
                //             //     $(#[$($field_meta)*])*
                //             // );
                //             #ඞ::writeln!(out,
                //                 "    {};\n",
                //                 < #EachFieldTy as #CType >::c_var(
                //                     #each_field_name_str,
                //                 ),
                //             )?;
                //         } else {
                //             #ඞ::assert_eq!(
                //                 #ඞ::mem::align_of::< #EachFieldTy >(),
                //                 1,
                //                 "\
                //                     Zero-sized fields must have an \
                //                     alignment of `1`.\
                //                 ",
                //             );
                //         }
                //     )*
                //     #ඞ::writeln!(out, "}} {};\n", me)
                // })
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
                #CType
            for
                #StructName #fwd_generics
            where
                #where_clauses
            {
                #impl_body
            }

            // If it is CType, it trivially is ReprC.
            impl #intro_generics
                #ReprC
            for
                #StructName #fwd_generics
            where
                #where_clauses
            {
                type CLayout = Self;

                #[inline]
                fn is_valid (
                    self: &'_ Self::CLayout,
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
