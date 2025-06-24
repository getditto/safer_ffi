use super::*;

mod kw {
    ::syn::custom_keyword!(js);
    ::syn::custom_keyword!(rename);
}

pub(crate) struct Args {
    pub(crate) rename: Option<Expr![String]>,

    pub(crate) js: Option<kw::js>,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Args> {
        let mut ret = Args {
            js: None,
            rename: None,
        };

        while input.is_empty().not() {
            let snoopy = input.lookahead1();
            match () {
                | _case if snoopy.peek(kw::rename) => {
                    let _: kw::rename = input.parse().unwrap();
                    let _: Token![=] = input.parse()?;
                    if ret.rename.replace(input.parse()?).is_some() {
                        return Err(input.error("duplicate attribute"));
                    }
                },
                | _case if snoopy.peek(kw::js) => {
                    if ret.js.replace(input.parse().unwrap()).is_some() {
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
