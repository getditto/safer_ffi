#[proc_macro_attribute] pub
fn ffi_export (attrs: TokenStream, input: TokenStream)
  -> TokenStream
{
    use ::proc_macro::{*, TokenTree as TT};
    if let Some(unexpected_tt) = attrs.into_iter().next() {
        return compile_error("Unexpected parameter", unexpected_tt.span());
    }
    #[cfg(feature = "proc_macros")] {
        let input = input.clone();
        let _: ItemFn = parse_macro_input!(input);
    }
    let span = Span::call_site();
    <TokenStream as ::std::iter::FromIterator<_>>::from_iter(vec![
        TT::Punct(Punct::new(':', Spacing::Joint)),
        TT::Punct(Punct::new(':', Spacing::Alone)),

        TT::Ident(Ident::new("repr_c", span)),

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
