use crate::*;

pub(in crate)
fn allowing_trivial_bound (
    mut where_predicate: WherePredicate
) -> WherePredicate
{
    if let WherePredicate::Type(PredicateType {
        ref mut lifetimes,
        ref mut bounded_ty,
        ..
    }) = where_predicate
    {
        lifetimes
            .get_or_insert_with(|| parse_quote!(for<>))
            .lifetimes
            .push(parse_quote!('__trivial_bound_hack))
        ;
        *bounded_ty = parse_quote!(
            ::safer_ffi::__::Identity<'__trivial_bound_hack, #bounded_ty>
        );
    } else {
        panic!("Invalid `where_predicate` arg");
    }
    where_predicate
}

pub(in crate)
fn ctype_generics (
    generics: &'_ Generics,
    EachFieldTy @ _: &mut dyn Iterator<Item = &'_ Type>,
) -> Generics
{
    #[apply(let_quote!)]
    use ::safer_ffi::ඞ::{
        ConcreteReprC,
        CLayoutOf,
        CType,
        OpaqueKind,
        ReprC,
    };
    generics.clone().also(|it| {
        it
        .make_where_clause()
        .predicates
        .extend_::<WherePredicate, _>(Iterator::chain(
            generics
                .type_params()
                .map(|TypeParam { ident: T, .. }| parse_quote!(
                    #T : #ReprC
                ))
            ,
            EachFieldTy
                .map(|FieldTy @_ | parse_quote!(
                    #FieldTy : #ConcreteReprC
                ))
                // .map(utils::allowing_trivial_bound)
            ,
        ))
    })
}
