use super::*;

#[derive(Default)]
pub(in crate)
struct Args {
    pub(in crate) node_js: Option<NodeJs>,
    pub(in crate) executor: Option<Executor>,
    pub(in crate) rename: Option<Rename>,
}

#[cfg_attr(not(feature = "js"),
    allow(dead_code),
)]
pub(in crate)
struct NodeJs {
    pub(in crate) kw: kw::node_js,
    pub(in crate) async_worker: Option<kw::async_worker>,
}

pub(in crate)
struct Executor {
    pub(in crate) kw: kw::executor,
    pub(in crate) _eq: Token![=],
    #[cfg_attr(not(feature = "async_fn"),
        allow(dead_code),
    )]
    pub(in crate) block_on: Expr,
}

pub(in crate)
struct Rename {
    pub(in crate) _kw: kw::rename,
    pub(in crate) _eq: Token![=],
    pub(in crate) new_name: LitStr,
}

mod kw {
    ::syn::custom_keyword!(async_worker);
    ::syn::custom_keyword!(executor);
    ::syn::custom_keyword!(node_js);
    ::syn::custom_keyword!(rename);
}

impl Parse for Args {
    fn parse (
        input: ParseStream<'_>,
    ) -> Result<Args>
    {
        let mut ret = Args::default();
        while input.is_empty().not() {
            let snoopy = input.lookahead1();
            match () {
                | _case if snoopy.peek(kw::executor) => {
                    if ret.executor.is_some() {
                        return Err(input.error("duplicate parameter"));
                    }
                    ret.executor = Some(Executor {
                        kw: input.parse().unwrap(),
                        _eq: input.parse()?,
                        block_on: input.parse()?,
                    });
                },

                | _case if snoopy.peek(kw::node_js) => {
                    if ret.node_js.is_some() {
                        return Err(input.error("duplicate parameter"));
                    }
                    ret.node_js = Some(NodeJs {
                        kw: input.parse().unwrap(),
                        async_worker: if input.peek(token::Paren) {
                            utils::parenthesized(input, |_paren, input| Ok({
                                let it: Option<_> = input.parse()?;
                                if it.is_some() {
                                    let _: Option<Token![,]> = input.parse()?;
                                }
                                it
                            }))?
                        } else {
                            None
                        },
                    });
                },

                | _case if snoopy.peek(kw::rename) => {
                    if ret.rename.is_some() {
                        return Err(input.error("duplicate parameter"));
                    }
                    ret.rename = Some(Rename {
                        _kw: input.parse().unwrap(),
                        _eq: input.parse()?,
                        new_name: {
                            let it = input.parse::<LitStr>()?;
                            if it.parse::<Ident>().is_err() {
                                bail! {
                                    "expected a function name (identifier)" => it,
                                }
                            }
                            it
                        },
                    });
                },

                | _default => return Err(snoopy.error()),
            }
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(ret)
    }
}
