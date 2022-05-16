use super::*;

pub(in crate)
fn derive (
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
            "Non field-less `enum`s are not supported yet." => payload,
        }
    }

    let (Int @ _, repr) = parse_discriminant_type(attrs, &mut ret)?;

    let EnumName_Layout @ _ = format_ident!("{}_Layout", EnumName);

    #[apply(let_quote!)]
    use ::safer_ffi::{
        ඞ,
        ඞ::{
            mem,
        },
        layout::{
            __HasNiche__,
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

        let ref EnumName_str = EnumName.to_string();
        let enum_size = repr.to_enum_size();
        let each_enum_variant =
            variants.iter().map(|v| {
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
                quote!(
                    #EnumVariant {
                        docs: &[], // FIXME TODO
                        name: #VariantName_str,
                        discriminant: #discriminant,
                    }
                )
            })
        ;

        let ref EnumName_t_str = format!("{EnumName}_t");
        impl_body.extend(quote!(
            fn short_name ()
              -> #ඞ::String
            {
                #EnumName_str.into()
            }

            fn define_self (
                language: &'_ dyn #HeaderLanguage,
                definer: &'_ mut dyn #Definer,
            ) -> #ඞ::io::Result<()>
            {
                definer.define_once(
                    #EnumName_t_str,
                    &mut |definer| {
                        <#Int as #CType>::define_self(language, definer)?;
                        language.emit_simple_enum(
                            definer,
                            &[#(#each_doc),*],
                            #EnumName_str,
                            #enum_size,
                            &[#(#each_enum_variant),*],
                        )
                    },
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

    ret.extend(quote!(
        unsafe
        impl #__HasNiche__
        for
            #EnumName
        {
            #[inline]
            fn is_niche (
                &#EnumName_Layout { discriminant }: &'_ #CLayoutOf<Self>,
            ) -> #ඞ::bool
            {
                /// Safety: this is either well-defined, or fails to compile.
                const DISCRIMINANT_OF_NONE: #Int = unsafe {
                    #ඞ::mem::transmute::<_, #Int>(
                        #ඞ::None::<#EnumName>
                    )
                };

                #ඞ::matches!(discriminant, DISCRIMINANT_OF_NONE)
            }
        }
    ));

    Ok(ret)
}

fn parse_discriminant_type (
    attrs: &'_ [Attribute],
    out_warnings: &mut TokenStream2,
) -> Result<(TokenStream2, Repr)>
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
                (
                    quote!(
                        ::safer_ffi::ඞ::#ident
                    ),
                    repr,
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
                (
                    quote!(
                        ::safer_ffi::c_int
                    ),
                    Repr::C,
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

        fn to_enum_size (&self)
          -> Quote![Option<(bool, u8)>]
        {
            use {
                ::{
                    core::primitive as builtin,
                },
                self::{
                    Repr::*,
                },
            };
            let mb_signed_bitwidth: Option<(bool, builtin::u8)> = match *self {
                | C => None,
                | u8 | u16 | u32 | u64 | u128 => Some((
                    false,
                    match *self {
                        | u8 => 8,
                        | u16 => 16,
                        | u32 => 32,
                        | u64 => 64,
                        | u128 => 128,
                        | _ => unreachable!(),
                    },
                )),
                | i8 | i16 | i32 | i64 | i128 => Some((
                    true,
                    match *self {
                        | i8 => 8,
                        | i16 => 16,
                        | i32 => 32,
                        | i64 => 64,
                        | i128 => 128,
                        | _ => unreachable!(),
                    },
                )),
            };
            let ret: Quote![Option<(bool, builtin::u8)>] = {
                match mb_signed_bitwidth {
                    | None => quote!(
                        ::safer_ffi::ඞ::None
                    ),
                    | Some((signed, bitwidth)) => quote!(
                        ::safer_ffi::ඞ::Some((#signed, #bitwidth))
                    ),
                }
            };
            ret
        }
    }
)}}
