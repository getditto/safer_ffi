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
