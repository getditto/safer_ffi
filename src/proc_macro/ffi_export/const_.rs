use super::*;

#[derive(Default)]
pub(in crate)
struct Args {
    pub(in crate) untyped: Option<Untyped>,
}

pub(in crate)
struct Untyped {
    pub(in crate) _kw: kw::untyped,
}

mod kw {
    ::syn::custom_keyword!(untyped);
}

impl Parse for Args {
    fn parse (
        input: ParseStream<'_>,
    ) -> Result<Args>
    {
        let mut ret = Args::default();
        while input.is_empty().not() {
            let snoopy = input.lookahead1();
            match () {
                | _case if snoopy.peek(kw::untyped) => {
                    if ret.untyped.is_some() {
                        return Err(input.error("duplicate parameter"));
                    }
                    ret.untyped = Some(Untyped {
                        _kw: input.parse().unwrap()
                    });
                },

                | _default => return Err(snoopy.error()),
            }
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(ret)
    }
}


#[allow(unexpected_cfgs)]
pub(in super)
fn handle (
    Args { untyped }: Args,
    input: ItemConst,
) -> Result<TokenStream2>
{
    if cfg!(feature = "headers").not() {
        Ok(input.into_token_stream())
    } else {
        #[apply(let_quote!)]
        use {
            ::safer_ffi::{
                __cfg_headers__,
                ඞ,
            },
            ::safer_ffi as krate,
        };


        let VAR @ _ = &input.ident;
        let VAR_str @ _ = &VAR.to_string();
        let Ty @ _ = &input.ty;
        let ref each_doc = utils::extract_docs(&input.attrs)?;
        let skip_type = untyped.is_some();

        Ok(quote!(
            #input

            #[cfg(not(target_arch = "wasm32"))]
            #ඞ::inventory::submit! {
                #ඞ::FfiExport {
                    name: #VAR_str,
                    gen_def: |
                        definer: &'_ mut dyn #ඞ::Definer,
                        lang: #ඞ::Language,
                    | {
                        #krate::__with_cfg_python__!(|$if_cfg_python| {
                            use #krate::headers::{
                                Language,
                                languages::{self, HeaderLanguage},
                            };

                            let header_builder: &'static dyn HeaderLanguage = {
                                match lang {
                                    | Language::C => &languages::C,
                                    | Language::CSharp => &languages::CSharp,
                                    | Language::Metadata => &languages::Metadata,
                                $($($if_cfg_python)?
                                    | Language::Python => &languages::Python,
                                )?
                                }
                            };

                            <#ඞ::CLayoutOf<#Ty> as #ඞ::CType>::define_self(
                                header_builder,
                                definer
                            )?;

                            header_builder
                        }).emit_constant(
                            definer,
                            &[ #(#each_doc),* ],
                            #VAR_str,
                            &#ඞ::PhantomData::<
                                #ඞ::CLayoutOf< #Ty >,
                            >,
                            #skip_type,
                            &#VAR,
                        )
                    },
                }
            }
        ))
    }
}
