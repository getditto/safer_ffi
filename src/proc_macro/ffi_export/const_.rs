use super::*;

#[derive(Default)]
pub(crate) struct Args {
    pub(crate) untyped: Option<Untyped>,
}

pub(crate) struct Untyped {
    pub(crate) _kw: kw::untyped,
}

mod kw {
    ::syn::custom_keyword!(untyped);
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Args> {
        let mut ret = Args::default();
        while input.is_empty().not() {
            let snoopy = input.lookahead1();
            match () {
                | _case if snoopy.peek(kw::untyped) => {
                    if ret.untyped.is_some() {
                        return Err(input.error("duplicate parameter"));
                    }
                    ret.untyped = Some(Untyped {
                        _kw: input.parse().unwrap(),
                    });
                },

                | _default => return Err(snoopy.error()),
            }
            let _: Option<Token![,]> = input.parse()?;
        }
        Ok(ret)
    }
}

#[allow(unexpected_cfgs)]
pub(super) fn handle(
    Args { untyped }: Args,
    input: ItemConst,
) -> Result<TokenStream2> {
    if cfg!(feature = "headers").not() {
        Ok(input.into_token_stream())
    } else {
        #[rustfmt::skip]
        #[apply(let_quote!)]
        use {
            ::safer_ffi::{
                __cfg_headers__,
                ඞ,
            },
            ::safer_ffi as krate,
        };

        let VAR @ _ = &input.ident;
        let VAR_str @ _ = &VAR.to_string();
        let Ty @ _ = &input.ty;
        let ref each_doc = utils::extract_docs(&input.attrs)?;
        let skip_type = untyped.is_some();

        Ok(quote!(
            #input

            #[cfg(not(target_arch = "wasm32"))]
            #ඞ::inventory::submit! {
                #ඞ::FfiExport {
                    name: #VAR_str,
                    gen_def: |
                        definer: &'_ mut dyn #ඞ::Definer,
                        lang: #ඞ::Language,
                    | {
                        use #krate::headers::{
                            Language,
                            languages::{self, HeaderLanguage},
                        };
                        let header_builder: &'static dyn HeaderLanguage =
                            #krate::__with_cfg_python__!(|$if_cfg_python| {
                                {
                                    match lang {
                                        | Language::C => &languages::C,
                                        | Language::CSharp => &languages::CSharp,
                                        | Language::Lua => &languages::Lua,
                                    $($($if_cfg_python)?
                                        | Language::Python => &languages::Python,
                                    )?
                                    }
                                }
                            })
                        ;

                        <#ඞ::CLayoutOf<#Ty> as #ඞ::CType>::define_self(
                            header_builder,
                            definer
                        )?;

                        header_builder.declare_constant(
                            header_builder,
                            definer,
                            &[ #(#each_doc),* ],
                            #VAR_str,
                            &#ඞ::PhantomData::<
                                #ඞ::CLayoutOf< #Ty >,
                            >,
                            #skip_type,
                            &#VAR,
                        )
                    },
                }
            }
        ))
    }
}
