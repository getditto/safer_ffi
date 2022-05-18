use super::*;

pub(in crate)
fn derive (
    args: Args,
    attrs: &'_ mut Vec<Attribute>,
    pub_: &'_ Visibility,
    EnumName @ _: &'_ Ident,
    generics: &'_ Generics,
    variants: &'_ Punctuated<Variant, Token![,]>,
) -> Result<TokenStream2>
{
    let mut ret = quote!();

    if let Some(payload) =
        variants
            .iter()
            .find(|Variant { fields, .. }| matches!(
                fields,
                Fields::Unit,
            ).not())
    {
        bail! {
            "Non field-less `enum`s are not supported yet." => payload.fields,
        }
    }

    if variants.is_empty() {
        bail! {
            "C does not support empty enums!"
        }
    }

    let (mb_phantom_int, Int @ _) = parse_discriminant_type(attrs, &mut ret)?;

    let EnumName_Layout @ _ = format_ident!("{}_Layout", EnumName);

    #[apply(let_quote!)]
    use ::safer_ffi::{
        ඞ,
        ඞ::{
            mem,
        },
        layout::{
            // __HasNiche__,
            CLayoutOf,
            CType as CType,
            OpaqueKind,
            ReprC,
        },
    };

    ret.extend(quote!(
        #[allow(nonstandard_style)]
        #[repr(transparent)]
        #[#ඞ::derive(
            #ඞ::Clone, #ඞ::Copy,
            // #ඞ::PartialEq, #ඞ::Eq,
        )]
        #pub_
        struct #EnumName_Layout {
            #pub_
            discriminant: #Int,
        }

        impl
            #ඞ::From<#Int>
        for
            #EnumName_Layout
        {
            #[inline]
            fn from (discriminant: #Int)
              -> Self
            {
                Self { discriminant }
            }
        }
    ));

    let impl_body = quote!(
        type OPAQUE_KIND = #OpaqueKind::Concrete;
    );

    let ref each_doc = utils::extract_docs(attrs)?;

    #[cfg(feature = "headers")]
    let impl_body = {
        #[apply(let_quote!)]
        use ::safer_ffi::{
            ඞ::fmt,
            headers::{
                Definer,
                languages::{
                    HeaderLanguage,
                    EnumVariant,
                },
            },
        };

        let mut impl_body = impl_body;

        let ref EnumName_str =
            args.rename.map_or_else(
                || EnumName.to_string().into_token_stream(),
                ToTokens::into_token_stream,
            )
        ;
        let ref each_enum_variant =
            variants.try_vmap(|v| Result::Ok({
                let ref VariantName_str = v.ident.to_string();
                let discriminant = if let Some((_eq, disc)) = &v.discriminant {
                    quote!(
                        #ඞ::Some(&(#disc, ).0 as _)
                    )
                } else {
                    quote!(
                        #ඞ::None
                    )
                };
                let docs = utils::extract_docs(&v.attrs)?;
                quote!(
                    #EnumVariant {
                        docs: &[#(#docs),*],
                        name: #VariantName_str,
                        discriminant: #discriminant,
                    }
                )
            }))?
        ;

        impl_body.extend(quote!(
            fn short_name ()
              -> #ඞ::String
            {
                #EnumName_str.into()
            }

            fn define_self__impl (
                language: &'_ dyn #HeaderLanguage,
                definer: &'_ mut dyn #Definer,
            ) -> #ඞ::io::Result<()>
            {
                <#Int as #CType>::define_self(language, definer)?;
                language.emit_simple_enum(
                    definer,
                    &[#(#each_doc),*],
                    &#ඞ::marker::PhantomData::<Self>,
                    #mb_phantom_int,
                    &[#(#each_enum_variant),*],
                )
            }
        ));

        impl_body
    };

    ret.extend(quote!(
        unsafe
        impl
            #CType
        for
            #EnumName_Layout
        {
            #impl_body
        }
    ));

    ret.extend({
        let ref EachVariant @ _ = variants.iter().vmap(|it| &it.ident);
        quote!(
            unsafe
            impl #ReprC for #EnumName {
                type CLayout = #EnumName_Layout;

                #[inline]
                fn is_valid (
                    &#EnumName_Layout { discriminant }: &'_ Self::CLayout,
                ) -> #ඞ::bool
                {
                    #![allow(nonstandard_style)]
                #(
                    const #EachVariant: #Int = #EnumName::#EachVariant as _;
                )*
                    #ඞ::matches!(discriminant, #( #EachVariant )|*)
                }
            }
        )
    });

    // ret.extend(quote!(
    //     unsafe
    //     impl #__HasNiche__
    //     for
    //         #EnumName
    //     {
    //         #[inline]
    //         fn is_niche (
    //             &#EnumName_Layout { discriminant }: &'_ #CLayoutOf<Self>,
    //         ) -> #ඞ::bool
    //         {
    //             /// Safety: should this ever become ill-defined, it would fail to compile.
    //             const DISCRIMINANT_OF_NONE: #Int = unsafe {
    //                 #ඞ::mem::transmute::<_, #Int>(
    //                     #ඞ::None::<#EnumName>
    //                 )
    //             };

    //             #ඞ::matches!(discriminant, DISCRIMINANT_OF_NONE)
    //         }
    //     }
    // ));

    Ok(ret)
}

fn parse_discriminant_type (
    attrs: &'_ [Attribute],
    out_warnings: &mut TokenStream2,
) -> Result<(
        Quote![Option<&impl PhantomCType>],
        TokenStream2,
    )>
{
    let repr_attr =
        attrs
            .iter()
            .find(|attr| attr.path.is_ident("repr"))
            .ok_or(())
            .or_else(|()| bail!("missing `#[repr(…)]` annotation"))?
    ;
    let ref reprs = repr_attr.parse_args_with(
        Punctuated::<Ident, Token![,]>::parse_terminated,
    )?;
    if reprs.is_empty() {
        bail!("expected an integral `repr` specifier" => repr_attr);
    }
    let parsed_reprs = reprs.iter().try_vmap(Repr::try_from)?;
    let (c_type, repr) =
        match
            ::core::iter::zip(parsed_reprs, reprs)
                .find(|(parsed, ident)| matches!(parsed, Repr::C).not())
        {
            | Some((repr, ident)) => {
                let IntTy = quote!(
                    ::safer_ffi::ඞ::#ident
                );
                (
                    quote!(
                        ::safer_ffi::ඞ::Some(
                            &::safer_ffi::ඞ::marker::PhantomData::<#IntTy>
                        )
                    ),
                    IntTy,
                )
            },
            | None if reprs.iter().any(|repr| repr == "C") => {
                out_warnings.extend(utils::compile_warning(
                    &repr_attr,
                    "\
                        `#[repr(C)]` enums are not well-defined in C; \
                        it is thus ill-advised to use them \
                        in a multi-compiler scenario such as FFI\
                    ",
                ));
                let IntTy = quote!(
                    ::safer_ffi::ඞ::os::raw::c_int
                );
                (
                    quote!(
                        ::safer_ffi::ඞ::None
                    ),
                    IntTy,
                )
            },
            | None => bail! {
                "expected an integral `repr` annotation"
            },
        }
    ;

    Ok((c_type, repr))
}

match_! {(
    C,
    u8, u16, u32, u64, u128,
    i8, i16, i32, i64, i128,
) {(
    $($repr:ident),* $(,)?
) => (
    enum Repr {
        $($repr),*
    }

    impl Repr {
        fn try_from (ident: &'_ Ident)
          -> Result<Self>
        {
            match &ident.to_string()[..] {
            $(
                | stringify!($repr) => Ok(Self::$repr),
            )*
                | _ => bail! {
                    "unsupported `repr` annotation" => ident,
                },
            }
        }
    }
)}}
