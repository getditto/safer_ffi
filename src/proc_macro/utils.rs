fn compile_error (err_msg: &'_ str, span: Span)
  -> TokenStream
{
    use ::proc_macro::{*, TokenTree as TT};
    macro_rules! spanned {($expr:expr) => ({
        let mut it = $expr;
        it.set_span(span);
        it
    })}
    <TokenStream as ::std::iter::FromIterator<_>>::from_iter(vec![
        TT::Ident(Ident::new("compile_error", span)),
        TT::Punct(spanned!(Punct::new('!', Spacing::Alone))),
        TT::Group(spanned!(Group::new(
            Delimiter::Brace,
            ::core::iter::once(TT::Literal(
                spanned!(Literal::string(err_msg))
            )).collect(),
        ))),
    ])
}

#[cfg(feature = "proc_macros")]
trait MySplit {
    type Ret;
    fn my_split (self: &'_ Self)
      -> Self::Ret
    ;
}

#[cfg(feature = "proc_macros")]
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

// #[cfg(any())] /* Comment to enable */
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
