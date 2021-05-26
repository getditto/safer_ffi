#![warn(warnings)] // Prevent `-Dwarnings` from causing breakage.
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt::skip)]
#![allow(nonstandard_style, unused_imports)]

use ::core::{mem, ops::Not as _};
use ::proc_macro::TokenStream;
use ::proc_macro2::{Span, TokenStream as TokenStream2};
use ::quote::{
    format_ident,
    quote,
    quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    Result, // explicitly shadow Result
};

#[proc_macro] pub
fn __js_ctx (
    input: TokenStream,
) -> TokenStream
{
    let _: parse::Nothing = parse_macro_input!(input);
    quote!(
        ::safer_ffi::node_js::CallContext::__new()
    ).into()
}

#[proc_macro_attribute] pub
fn js_export (
    attrs: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    let Attrs { js_name, skip_napi_import } = parse_macro_input!(attrs);
    let ref mut input: ItemFn = parse_macro_input!(input);
    if matches!(input.vis, Visibility::Public(_)).not() {
        input.vis = Visibility::Public(VisPublic {
            pub_token: Token![pub](Span::call_site()),
        });
    }
    replace_lifetimes(input);
    let _ = skip_napi_import;
    let js_name = js_name.map(|it| quote!(js_name = #it));
    quote!(
        const _: () = {
            use ::safer_ffi::{
                node_js as napi,
                node_js::__::wasm_bindgen,
            };

            #[wasm_bindgen(#js_name)]
            #input
        };
    ).into()
}

include!("../../../js_export_common.rs");

/// Since `#[wasm_bindgen]` does not want to have to deal
/// with short-lived stuff for soundness reasons, they implement
/// a heuristic they deny any non-`'static` lifetime that appear
/// in the function signature.
///
/// So to "force" our way in without patching `#[wasm_bindgen]`,
/// we replace any lifetime occurrences with `'static`.
fn replace_lifetimes (input: &'_ mut ItemFn)
{
    struct Visitor;
    /// Behold, the power of `syn`!
    impl visit_mut::VisitMut for Visitor {
        fn visit_path_arguments_mut (
            self: &'_ mut Visitor,
            path_args: &'_ mut PathArguments,
        )
        {
            #[allow(nonstandard_style)]
            mod Retain {
                pub const Drop: bool = false;
                pub const Keep: bool = true;
            }

            // Sub-recurse!
            visit_mut::visit_path_arguments_mut(self, path_args);
            // Now handle the top-most level:
            match *path_args {
                | PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    ref mut args,
                    ..
                }) => *args = {
                    mem::take(args)
                        .into_iter()
                        .filter(|generic_arg| match *generic_arg {
                            | GenericArgument::Lifetime(_) => Retain::Drop,
                            | _ => Retain::Keep,
                        })
                        .collect()
                },
                | PathArguments::Parenthesized(_) => {
                    /* TODO? For now, do nothing */
                }
                | PathArguments::None => {},
            }
        }

        fn visit_type_reference_mut (
            self: &'_ mut Self,
            reference: &'_ mut TypeReference,
        )
        {
            // Sub-recurse!
            visit_mut::visit_type_reference_mut(self, reference);
            // Now handle the outer-most level:
            reference.lifetime = None;
        }
    }
    visit_mut::VisitMut::visit_signature_mut(&mut Visitor, &mut input.sig);
    // And voilà!

    // // Now, if we ever _introduced_ generic lifetime parameters, remove that:
    // // it won't make any sense to be introducing the `'static` lifetime.
    // input.sig.generics.params =
    //     mem::take(&mut input.sig.generics.params)
    //         .into_iter()
    //         .filter(|generic_param| {
    //             matches!(
    //                 generic_param,
    //                 GenericParam::Lifetime(_),
    //             )
    //             .not()
    //         })
    //         .collect()
    // ;
}
