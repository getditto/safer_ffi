#[proc_macro] pub
fn __make_all_lifetimes_static (
    input: TokenStream,
) -> TokenStream
{
    let mut input: Type = parse_macro_input!(input);
    struct Visitor;
    impl visit_mut::VisitMut for Visitor {
        fn visit_lifetime_mut (
            self: &'_ mut Visitor,
            lifetime: &'_ mut Lifetime,
        )
        {
            lifetime.ident = Ident::new("static", lifetime.ident.span());
        }

        fn visit_type_reference_mut (
            self: &'_ mut Visitor,
            reference: &'_ mut TypeReference,
        )
        {
            // Sub-recurse!
            visit_mut::visit_type_reference_mut(self, reference);
            if reference.lifetime.is_none() {
                reference.lifetime = Some(Lifetime::new(
                    "'static",
                    reference.and_token.span,
                ));
            }
        }
    }
    visit_mut::VisitMut::visit_type_mut(&mut Visitor, &mut input);
    input.into_token_stream().into()
}

struct Attrs {
    js_name: Option<Ident>,
    skip_napi_import: Option<()>,
}

impl Parse for Attrs {
    fn parse (input: ParseStream<'_>)
      -> Result<Attrs>
    {
        mod kw {
            ::syn::custom_keyword!(js_name);
            ::syn::custom_keyword!(__skip_napi_import);
        }
        let mut js_name = None;
        let mut skip_napi_import = None;
        while input.is_empty().not() {
            let lookahead = input.lookahead1();
            match () {
                | _case if lookahead.peek(kw::js_name) => {
                    let _: kw::js_name = input.parse().unwrap();
                    let _: Token![=] = input.parse()?;
                    let prev = js_name.replace(input.parse()?);
                    if prev.is_some() {
                        return Err(input.error("Duplicate attribute"));
                    }
                },
                | _case if lookahead.peek(kw::__skip_napi_import) => {
                    let _: kw::__skip_napi_import = input.parse().unwrap();
                    skip_napi_import = Some(());
                },
                | _default => return Err(lookahead.error()),
            }
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(Attrs {
            js_name,
            skip_napi_import,
        })
    }
}
