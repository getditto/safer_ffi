use super::*;

pub(in super)
fn handle (
    args: parse::Nothing,
    Ty @ _: &'_ Ident,
    generics: &'_ Generics,
    input: &dyn ToTokens,
) -> Result<TokenStream2>
{
    if let Some(extraneous) = generics.params.first() {
        bail! {
            "generic parameters not allowed here" => extraneous,
        }
    }
    if let Some(clause) = &generics.where_clause {
        bail! {
            "`where` clauses not allowed here" => clause.where_token,
        }
    }
    let ref Ty_str @ _ = Ty.to_string();
    Ok(quote!(
        #input

        #[cfg(not(target_arch = "wasm32"))]
        ::safer_ffi::__cfg_headers__! {
            ::safer_ffi::inventory::submit! {
                #![crate = ::safer_ffi]

                ::safer_ffi::FfiExport {
                    name: #Ty_str,
                    gen_def: ::safer_ffi::headers::__define_self__::<#Ty>,
                }
            }
        }
    ))
}
