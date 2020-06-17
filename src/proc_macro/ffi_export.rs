/// Export a function to be callable by C.
///
/// # Example
///
/// ```rust
/// use ::safer_ffi::prelude::ffi_export;
///
/// #[ffi_export]
/// /// Add two integers together.
/// fn add (x: i32, y: i32) -> i32
/// {
///     x + y
/// }
/// ```
///
///   - ensures that [the generated headers](/safer_ffi/headers/) will include the
///     following definition:
///
///     ```C
///     #include <stdint.h>
///
///     /* \brief
///      * Add two integers together.
///      */
///     int32_t add (int32_t x, int32_t y);
///     ```
///
///   - exports an `add` symbol pointing to the C-ABI compatible
///     `int32_t (*)(int32_t x, int32_t y)` function.
///
///     (The crate type needs to be `cdylib` or `staticlib` for this to work,
///     and, of course, the C compiler invocation needs to include
///     `-L path/to/the/compiled/library -l name_of_your_crate`)
///
///       - when in doubt, use `staticlib`.
///
/// # `ReprC`
///
/// [`ReprC`]: /safer_ffi/layout/trait.ReprC.html
///
/// You can use any Rust types in the singature of an `#[ffi_export]`-
/// function, provided each of the types involved in the signature is [`ReprC`].
///
/// Otherwise the layout of the involved types in the C world is **undefined**,
/// which `#[ffi_export]` will detect, leading to a compilation error.
///
/// To have custom structs implement [`ReprC`], it suffices to annotate the
/// `struct` definitions with the [`#[derive_ReprC]`](
/// /safer_ffi/layout/attr.derive_ReprC.html)
/// (on top of the obviously required `#[repr(C)]`).
#[proc_macro_attribute] pub
fn ffi_export (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    use ::proc_macro::{*, TokenTree as TT};
    if let Some(unexpected_tt) = attrs.into_iter().next() {
        return compile_error("Unexpected parameter", unexpected_tt.span());
    }
    // #[cfg(feature = "proc_macros")] {
    //     let input = input.clone();
    //     let _: ItemFn = parse_macro_input!(input);
    // }
    let span = Span::call_site();
    <TokenStream as ::std::iter::FromIterator<_>>::from_iter(vec![
        TT::Punct(Punct::new(':', Spacing::Joint)),
        TT::Punct(Punct::new(':', Spacing::Alone)),

        TT::Ident(Ident::new("safer_ffi", span)),

        TT::Punct(Punct::new(':', Spacing::Joint)),
        TT::Punct(Punct::new(':', Spacing::Alone)),

        TT::Ident(Ident::new("__ffi_export__", span)),

        TT::Punct(Punct::new('!', Spacing::Alone)),

        TT::Group(Group::new(
            Delimiter::Brace,
            input.into_iter().collect(),
        )),
    ])
}
