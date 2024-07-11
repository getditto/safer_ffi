use super::*;

pub(in super)
fn handle (
    _args: parse::Nothing,
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

        Ok(quote!(
            #input

            #[cfg(not(target_arch = "wasm32"))]
            #ඞ::inventory::submit! {
                #ඞ::FfiExport {
                    name: #VAR_str,
                    gen_def: |
                        definer: &'_ mut dyn #ඞ::Definer,
                        lang_config: &'_ #ඞ::LanguageConfig
                    | {
                        #krate::__with_cfg_python__!(|$if_cfg_python| {
                            use #krate::headers::{
                                Language,
                                languages::{self, HeaderLanguage},
                            };

                            let header_builder: &'static dyn HeaderLanguage = {
                                match lang_config {
                                    | Language::C(_) => &languages::C,
                                    | Language::CSharp(_) => &languages::CSharp,
                                $($($if_cfg_python)?
                                    | Language::Python(_) => &languages::Python,
                                )?
                                }
                            };

                            header_builder
                        }).emit_constant(
                            definer,
                            lang_config,
                            &[ #(#each_doc),* ],
                            #VAR_str,
                            &#ඞ::PhantomData::<
                                #ඞ::CLayoutOf< #Ty >,
                            >,
                            &#VAR,
                        )
                    },
                }
            }
        ))
    }
}
