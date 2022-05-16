use super::*;

pub(in crate)
struct Args {
    pub(in crate)
    rename: Option<Expr![String]>,
}

impl Parse for Args {
    fn parse (input: ParseStream<'_>)
      -> Result<Args>
    {
        let mut ret = Args {
            rename: None,
        };

        let snoopy = input.lookahead1();
        while input.is_empty().not() {
            mod kw {
                ::syn::custom_keyword!(rename);
            }
            match () {
                | _case if snoopy.peek(kw::rename) => {
                    let _: kw::rename = input.parse().unwrap();
                    let _: Token![=] = input.parse()?;
                    if ret.rename.replace(input.parse()?).is_some() {
                        return Err(input.error("duplicate attribute"));
                    }
                },
                | _default => return Err(snoopy.error()),
            }
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(ret)
    }
}
