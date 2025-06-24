//! The logic around the dynamically self-registered Node.js functions.
//!
//! One of the possible approaches to do this with N-API is by exporting
//! a `napi_register_module_v1` special function.
//!
//! But that must be done from the downstream crate, so we export a macro
//! that just `#[unsafe(no_mangle)]`-wraps the definition of such function contained
//! in this module.
//!
//! Moreover, in order to support automagically getting access to the set
//! of all the `#[ffi_export(js)]`-annotated functions, we use
//! [`::inventory`] to define here a registry that will be extended by each
//! `#[ffi_export(js)]` annotation. Thanks to the magic of [`::inventory`],
//! we can then iterate over it here and it Just Works™.

#[allow(missing_copy_implementations, missing_debug_implementations)]
pub enum NapiRegistryEntry {
    NamedMethod {
        name: &'static str,
        method: ::napi::Callback,
    },
}

self::inventory::collect!(NapiRegistryEntry);

pub use crate::inventory::submit;
pub use crate::inventory::{self};

#[cold]
pub unsafe extern "C" fn napi_register_module_v1(
    raw_env: ::napi::sys::napi_env,
    raw_exports: ::napi::sys::napi_value,
) -> ::napi::sys::napi_value {
    // let env = ::napi::Env::from_raw(raw_env);
    let mut exports: ::napi::JsObject =
        unsafe { ::napi::NapiValue::from_raw_unchecked(raw_env, raw_exports) };
    match (|| {
        ::napi::Result::<_>::Ok({
            for entry in crate::inventory::iter::<NapiRegistryEntry> {
                match entry {
                    | &NapiRegistryEntry::NamedMethod { name, method } => {
                        let _ = exports.create_named_method(name, method);
                    },
                }
            }
        })
    })() {
        | Ok(()) => raw_exports,
        | Err(err) => unsafe {
            ::napi::sys::napi_throw_error(
                raw_env,
                crate::NULL!(),
                (
                    ::std::ffi::CString::new(format!("Error initializing module: {}", err))
                        .unwrap(),
                )
                    .0
                    .as_ptr(),
            );
            crate::NULL!()
        },
    }
}
