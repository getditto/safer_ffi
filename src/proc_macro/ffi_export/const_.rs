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

        let inventory_krate = cfg!(not(feature = "inventory-0-3-1")).then(|| {
            quote!( #![crate = #ඞ] )
        });
        Ok(quote!(
            #input

            #[cfg(not(target_arch = "wasm32"))]
            #ඞ::inventory::submit! {
                #inventory_krate

                #ඞ::FfiExport {
                    name: #VAR_str,
                    gen_def: |
                        definer: &'_ mut dyn #ඞ::Definer,
                        lang: #ඞ::Language,
                    | {
                        {
                            use #krate::headers::{
                                Language,
                                languages::{self, HeaderLanguage},
                            };
                            let header_builder: &'static dyn HeaderLanguage =
                                match lang {
                                    | Language::C => &languages::C,
                                    | Language::CSharp => &languages::CSharp,
                                    | Language::Python => &languages::Python,
                                }
                            ;
                            header_builder
                        }.emit_constant(
                            definer,
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
