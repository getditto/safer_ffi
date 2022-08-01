#![cfg_attr(rustfmt, rustfmt::skip)]
#![allow(unused)]
#![warn(unused_must_use)]

use super::*;

pub(in crate) use extension_traits::*;
mod extension_traits;

pub(in crate) use macros::*;
mod macros;

pub(in crate) use mb_file_expanded::*;
mod mb_file_expanded;

pub(in crate) use trait_impl_shenanigans::*;
mod trait_impl_shenanigans;

pub(in crate)
trait MySplit {
    type Ret;
    fn my_split (self: &'_ Self)
      -> Self::Ret
    ;
}

impl MySplit for Generics {
    type Ret = (TokenStream2, Vec<WherePredicate>);

    fn my_split (self: &'_ Generics)
      -> Self::Ret
    {
        let cap = self.params.iter().len();
        let mut lts = Vec::with_capacity(cap);
        let mut tys = Vec::with_capacity(cap);
        let mut predicates =
            self.split_for_impl()
                .2
                .map_or(vec![], |wc| wc.predicates.iter().cloned().collect())
        ;
        self.params
            .iter()
            .cloned()
            .for_each(|it| match it {
                | GenericParam::Type(mut ty) => {
                    let ty_param = &ty.ident;
                    ::core::mem::take(&mut ty.bounds)
                        .into_iter()
                        .for_each(|bound: TypeParamBound| {
                            predicates.push(parse_quote! {
                                #ty_param : #bound
                            });
                        })
                    ;
                    tys.push(ty);
                },
                | GenericParam::Lifetime(mut lt) => {
                    let lt_param = &lt.lifetime;
                    ::core::mem::take(&mut lt.bounds)
                        .into_iter()
                        .for_each(|bound: Lifetime| {
                            predicates.push(parse_quote! {
                                #lt_param : #bound
                            });
                        })
                    ;
                    lts.push(lt);
                },
                | GenericParam::Const(_) => {
                    unimplemented!("const generics")
                },
            })
        ;
        (
            quote!(
                #(#lts ,)*
                #(#tys),*
            ),

            predicates
        )
    }
}

#[cfg(any())] /* Comment to enable (requires `cargo add bat`) */
pub(in crate)
fn pretty_print_tokenstream (
    code: &'_ TokenStream,
    fname: &'_ str,
)
{
    fn try_format (input: &'_ str)
      -> Option<String>
    {Some({
        let mut child =
            ::std::process::Command::new("rustfmt")
                .stdin(::std::process::Stdio::piped())
                .stdout(::std::process::Stdio::piped())
                .stderr(::std::process::Stdio::piped())
                .spawn()
                .ok()?
        ;
        match child.stdin.take().unwrap() { ref mut stdin => {
            ::std::io::Write::write_all(stdin, input.as_bytes()).ok()?;
        }}
        let mut stdout = String::new();
        ::std::io::Read::read_to_string(
            &mut child.stdout.take().unwrap(),
            &mut stdout,
        ).ok()?;
        if child.wait().ok()?.success().not() { return None; }
        stdout
    })}

    if  ::std::env::var("SAFER_FFI_DEBUG_FILTER")
            .ok()
            .map_or(true, |ref filter| fname.contains(filter))
    {
        if let Some(ref formatted) = try_format(&code.to_string()) {
            // It's formatted, now let's try to also colorize it:
            if  ::bat::PrettyPrinter::new()
                    .input_from_bytes(formatted.as_ref())
                    .language("rust")
                    .true_color(false)
                    .print()
                    .is_err()
            {
                // Fallback to non-colorized-but-formatted output.
                println!("{}", formatted);
            }
        } else {
            // Fallback to raw output.
            println!("{}", code);
        }
    }
}

pub(in crate)
struct RemapNonStaticLifetimesTo<'__> {
    pub(in crate)
    new_lt_name: &'__ str,
}

impl ::syn::visit_mut::VisitMut
    for RemapNonStaticLifetimesTo<'_>
{
    fn visit_lifetime_mut (
        self: &'_ mut Self,
        lifetime: &'_ mut Lifetime,
    )
    {
        if lifetime.ident != "static" {
            lifetime.ident = Ident::new(
                self.new_lt_name,
                lifetime.ident.span(),
            );
        }
    }

    fn visit_type_reference_mut (
        self: &'_ mut Self,
        ty_ref: &'_ mut TypeReference,
    )
    {
        // 1 – sub-recurse
        visit_mut::visit_type_reference_mut(self, ty_ref);
        // 2 – handle the implicitly elided case.
        if ty_ref.lifetime.is_none() {
            ty_ref.lifetime = Some(Lifetime::new(
                &["'", self.new_lt_name].concat(),
                ty_ref.and_token.span,
            ));
        }
    }

    fn visit_parenthesized_generic_arguments_mut (
        self: &'_ mut Self,
        _: &'_ mut ParenthesizedGenericArguments,
    )
    {
        // Elided lifetimes in `fn(…)` or `Fn…(…)` are higher order:
        /* do not subrecurse */
    }
}

pub(in crate)
fn compile_warning (
    span: &dyn ToTokens,
    message: &str,
) -> TokenStream2
{
    let mut spans = span.to_token_stream().into_iter().map(|tt| tt.span());
    let fst = spans.next().unwrap_or_else(Span::call_site);
    let lst = spans.fold(fst, |_, cur| cur);
    let ref message = ["\n", message].concat();
    let warning = Ident::new("warning", fst);
    quote_spanned!(lst=>
        #[allow(nonstandard_style, clippy::all)]
        const _: () = {
            #[allow(nonstandard_style)]
            struct safer_ffi_ {
                #[deprecated(note = #message)]
                #warning: ()
            }
            //                     fst    lst
            let _ = safer_ffi_ { #warning: () };
            //                   ^^^^^^^^^^^^
        };
    )
}

pub(in crate)
fn extract_docs (
    attrs: &'_ [Attribute]
) -> Result<Vec<Expr>>
{
    attrs.iter().filter_map(|attr| {
        attr.path
            .is_ident("doc")
            .then(|| Parser::parse2(
                |input: ParseStream<'_>| Ok(
                    if input.peek(Token![=]) {
                        let _: Token![=] = input.parse::<Token![=]>().unwrap();
                        let doc_str: Expr = input.parse()?;
                        let _: Option<Token![,]> = input.parse()?;
                        Some(doc_str)
                    } else {
                        let _ = input.parse::<TokenStream2>();
                        None
                    }
                ),
                attr.tokens.clone(),
            )
            .transpose())
            .flatten()
        })
        .collect()
}

pub(crate)
struct LazyQuote(
    pub(crate) fn() -> TokenStream2,
    pub(crate) ::core::cell::RefCell<Option<TokenStream2>>,
);

impl ::quote::ToTokens for LazyQuote {
    fn to_tokens (self: &'_ LazyQuote, tokens: &'_ mut TokenStream2)
    {
        self.1
            .borrow_mut()
            .get_or_insert_with(self.0)
            .to_tokens(tokens)
    }
}

pub(in crate)
fn parenthesized<T> (
    input: ParseStream<'_>,
    scope: impl FnOnce(token::Paren, ParseStream<'_>) -> Result<T>,
) -> Result<T>
{
    let contents;
    scope(parenthesized!(contents in input), &contents)
}
