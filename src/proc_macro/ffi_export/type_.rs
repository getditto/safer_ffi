use super::*;

pub(in super)
fn handle (
    _args: parse::Nothing,
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
    let inventory_krate = cfg!(not(feature = "inventory-0-3-1")).then(|| {
        quote!( #![crate = ::safer_ffi] )
    });
    Ok(quote!(
        #input

        #[cfg(not(target_arch = "wasm32"))]
        ::safer_ffi::__cfg_headers__! {
            ::safer_ffi::inventory::submit! {
                #inventory_krate

                ::safer_ffi::FfiExport {
                    name: #Ty_str,
                    gen_def: ::safer_ffi::headers::__define_self__::<#Ty>,
                }
            }
        }
    ))
}
