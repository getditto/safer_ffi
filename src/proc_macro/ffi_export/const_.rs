use super::*;

pub(in super)
fn handle (
    args: super::fn_::Args,
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
        let skip_type =  matches!(args.raw_const, Some(true));

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
