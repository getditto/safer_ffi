use super::*;

pub(in crate)
fn mb_file_expanded (output: TokenStream2)
  -> TokenStream2
{
    let mut debug_macros_dir =
        match ::std::env::var_os("DEBUG_MACROS_LOCATION") {
            | Some(it) => ::std::path::PathBuf::from(it),
            | None => return output,
        }
    ;
    let hopefully_unique = {
        use ::std::hash::*;
        let ref mut hasher =
            ::std::collections::hash_map::RandomState::new()
                .build_hasher()
        ;
        hasher.finish()
    };

    debug_macros_dir.push("safer-ffi-debugged-proc-macros");
    ::std::fs::create_dir_all(&debug_macros_dir)
        .unwrap_or_else(|err| panic!(
            "`DEBUG_MACROS_LOCATION`-error: failed to create {}: {}",
            debug_macros_dir.display(), err,
        ))
    ;
    let ref file_name = {
        debug_macros_dir.push(format!("{:016x}.rs", hopefully_unique));
        debug_macros_dir
            .into_os_string()
            .into_string()
            .expect("`DEBUG_MACROS_LOCATION`-error: \
                non-UTF-8 paths are not supported\
            ")
    };

    ::std::fs::write(
        file_name,
        ::std::panic::catch_unwind(|| ::prettyplease::unparse(&parse_quote!(#output)))
            .unwrap_or_else(|_| quote!(#output).to_string())
        ,
    )
        .unwrap_or_else(|err| panic!(
            "`DEBUG_MACROS_LOCATION`-error: failed to write to `{}`: {}",
            file_name, err
        ))
    ;
    let warning =
        compile_warning(&quote!(), &format!(
            "Output emitted to {file_name}",
        ))
    ;
    quote!(
        #warning

        ::core::include! {
            #file_name
        }
    )
}
