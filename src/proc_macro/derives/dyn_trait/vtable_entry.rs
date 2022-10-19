#![cfg_attr(rustfmt, rustfmt::skip)]

use super::*;

pub(in super)
enum VTableEntry<'trait_> {
    VirtualMethod {
        src: &'trait_ TraitItemMethod,
        name: &'trait_ Ident,
        each_for_lifetime: Vec<&'trait_ Lifetime>,
        each_arg_name: Vec<Ident>,
        ErasedSelf: Type,
        EachArgTy: Vec<&'trait_ Type>,
        OutputTy: &'trait_ [Type],
    },
}

impl<'trait_> VTableEntry<'trait_> {
    pub(in super)
    fn name (self: &'_ VTableEntry<'trait_>)
      -> &'trait_ Ident
    {
        match self {
            | Self::VirtualMethod {
                name,
                ..
            } => name,
        }
    }

    pub(in super)
    fn virtual_forwarding<'r> (
        self: &'r VTableEntry<'trait_>
    ) -> TokenStream2
    {
        match *self {
            | Self::VirtualMethod {
                name,
                each_for_lifetime: _,
                ref each_arg_name,
                ErasedSelf: _,
                EachArgTy: _,
                OutputTy: _,
                src: &TraitItemMethod {
                    sig: ref full_signature,
                    ref attrs,
                    ..
                },
            } => {
                let mut signature = full_signature.clone();
                signature
                    .inputs
                    .iter_mut()
                    .skip(1)
                    .zip(each_arg_name)
                    .for_each(|(fn_arg, arg_name)| match *fn_arg {
                        | FnArg::Typed(PatType { ref mut pat, .. }) => {
                            // let arg_name = format_ident!("__arg_{}", i, span = ty.span());
                            **pat = parse_quote!( #arg_name );
                        },
                        | _ => unreachable!(),
                    })
                ;
                quote!(
                    #(#attrs)*
                    #[inline]
                    #signature
                    {
                        unsafe {
                            ::core::mem::transmute(
                                (self.__vtable().#name)(
                                    // FIXME: use traits to feature .into_raw() / .from_raw()
                                    ::core::mem::transmute(self.__ptr()),
                                    #(
                                        ::core::mem::transmute(#each_arg_name),
                                    )*
                                )
                            )
                        }
                    }
                )
            },
        }
    }

    pub(in super)
    fn attrs<'r> (
        self: &'r VTableEntry<'trait_>
    ) -> &'trait_ Vec<Attribute>
    {
        match self {
            | Self::VirtualMethod {
                src: &TraitItemMethod {
                    ref attrs,
                    ..
                },
                ..
            } => attrs,
        }
    }

    pub(in super)
    fn type_and_value<'r> (
        self: &'r VTableEntry<'trait_>,
    ) -> (
            TokenStream2,
            impl 'r + Fn(
                /* QSelf: */ &dyn ToTokens,
                /* trait_generics: */ &'_ Generics,
            ) -> TokenStream2,
        )
    {
        match self {
            | Self::VirtualMethod {
                name,
                each_for_lifetime,
                each_arg_name,
                ErasedSelf,
                EachArgTy,
                OutputTy,
                src: _,
            } => {
                let span = Span::mixed_site().located_at(name.span());
                let EachArgTy @ _ = EachArgTy.iter().copied().vmap(CType);
                let OutputTy @ _ = CType(OutputTy.get(0).unwrap_or(&parse_quote!( () )));
                let type_ = quote_spanned!(span=>
                    for<#(#each_for_lifetime),*>
                    unsafe
                    extern "C"
                    fn(
                        #ErasedSelf,
                        #(#EachArgTy ,)*
                    ) -> #OutputTy

                );
                let value = {
                    // let type_ = type_.clone(); /* may not be necessary */
                    move
                    |
                        QSelf @ _: &dyn ToTokens,
                        trait_generics: &Generics,
                    | {
                        // What happens here is quite subtle:
                        //  1. we are dealing with the function signature of a trait's method
                        //  2. the trait may have generic lifetime params,
                        //  3. and the method may have its own generic lifetime params
                        //       - which we'll currently assume to be higher-order / late-bound
                        //         since they'll most likely be (and writing a heuristic to detect
                        //         these with a macro seems overkill to begin with)
                        //  4. we want to end up with a `for<'higher_order_lts…> fn…` kind of
                        //     function pointer, but still be able to name the types of the method
                        //     signature, even if those may refer to the trait's generic.
                        //  5. since it has to be an `extern fn` pointer, we can't use closures
                        //     to implicitly get access to those, so we need to:
                        //       - REINJECT the trait generics into the helper fn def;
                        //       - TURBOFISH-FEED those immediately after, when instanciating it.
                        //  6. But… the outer lifetime generics will be problematic:
                        //       - They are not to be higher-order / late-bound, but early-bound.
                        //       - if a combination of both such kinds lifetime params occurs,
                        //         no lifetime parameter may be turbofished, at all.
                        //  7. We tackle the former problem by ensuring the outer lifetime parameters
                        //     are early-bound by ensuring there is a `:` after their definition 😅
                        //  8. And tackle the latter by not turbofishing the early-bound lifetimes,
                        //     since those can always be left inferred.
                        let mut vfn_generics = trait_generics.clone();
                        // Step 7: ensure the outer generics introduce early-bound lifetimes.
                        vfn_generics
                            .make_where_clause()
                            .predicates
                            .extend(
                                trait_generics
                                    .lifetimes()
                                    .map(|LifetimeDef { lifetime, .. }| -> WherePredicate {
                                        parse_quote!(
                                            #lifetime :
                                        )
                                    })
                            )
                        ;
                        vfn_generics.params = Iterator::chain(
                            each_for_lifetime.iter().map(|lt| -> GenericParam {
                                parse_quote!( #lt )
                            }),
                            ::core::mem::take(&mut vfn_generics.params)
                        ).collect();
                        // we can't use fwd_generics since we want to skip the lts.
                        let (intro_generics, _, where_clause) =
                            vfn_generics.split_for_impl()
                        ;
                        let fwd_generics = Iterator::chain(
                            vfn_generics.type_params().map(|it| &it.ident),
                            vfn_generics.const_params().map(|it| &it.ident),
                        );
                        quote_spanned!(span=> {
                            unsafe
                            extern "C"
                            fn #name #intro_generics (
                                __this: #ErasedSelf,
                                #(#each_arg_name: #EachArgTy ,)*
                            ) -> #OutputTy
                            #where_clause
                            {
                                ::safer_ffi::layout::into_raw(#QSelf::#name(
                                    ::core::mem::transmute(__this) #(,
                                    ::safer_ffi::layout::from_raw_unchecked(
                                        #each_arg_name
                                    ) )*
                                ))
                            }

                            #name ::< #(#fwd_generics),* > // as #type_
                        })
                    }
                };
                (type_, value)
            },
        }
    }
}

pub(in super)
fn vtable_entries<'trait_> (
    trait_items: &'trait_ mut [TraitItem],
    emit: &mut TokenStream2,
) -> Result<Vec<VTableEntry<'trait_>>>
{
    use ::quote::format_ident as ident;
    // let mut Sized @ _ = None;
    // let mut skip_attrs_found = vec![];
    macro_rules! failwith {( $err_msg:expr => $at:expr $(,)? ) => (
        return Some(Err(Error::new_spanned($at, $err_msg)))
    )}
    macro_rules! continue_ {() => (
        return None
    )}
    trait_items.iter_mut().filter_map(|it| Some(Result::Ok(match *it {
        | TraitItem::Method(ref trait_item_method @ TraitItemMethod {
            attrs: _,
            sig: ref sig @ Signature {
                constness: ref const_,
                asyncness: ref async_,
                unsafety: _, // ref unsafe_,
                abi: _, // ref extern_,
                fn_token: _,
                ident: ref method_name,
                ref generics,
                ref paren_token,
                ref inputs,
                variadic: ref variadic,
                output: ref RetTy @ _,
            },
            default: _,
            semi_token: _,
        }) => {
            // // Is there a `Self : Sized` opt-out-of-`dyn` clause?
            // if matches!(
            //     generics.where_clause, Some(ref where_clause)
            //     if where_clause.predicates.iter().any(|clause| matches!(
            //         *clause, WherePredicate::Type(PredicateType {
            //             lifetimes: ref _for,
            //             bounded_ty: Type::Path(TypePath {
            //                 qself: None,
            //                 path: ref BoundedTy @ _,
            //             }),
            //             colon_token: _,
            //             ref bounds,
            //         })
            //         if BoundedTy.is_ident("Self")
            //         && bounds.iter().any(|Bound @ _| matches!(
            //             *Bound, TypeParamBound::Trait(TraitBound {
            //                 path: ref Super @ _,
            //                 ..
            //             })
            //             if Super.is_ident("Sized")
            //         ))
            //     ))
            // )
            // {
            //     // If so, skip it, it did opt out after all.
            //     continue_!()
            // }
            match_! {(const_ async_ variadic) {( $($it:tt)* ) => (
                $(
                    if $it.is_some() {
                        failwith! {
                            "not `dyn`-safe" => $it,
                        }
                    }

                )*
            )}}
            let ref mut storage = None;
            let lifetime_of_and = move |and: &Token![&], mb_lt| {
                let _: &Option<Lifetime> = mb_lt;
                mb_lt.as_ref().unwrap_or_else(|| {
                    { storage }.get_or_insert(
                        Lifetime::new("'_", and.span)
                    )
                })
            };
            let receiver = if let Some(fn_arg) = sig.receiver() {
                match ReceiverType::from_fn_arg(fn_arg) {
                    | Ok(it) => it,
                    | Err(err) => return Some(Err(err)),
                }
            } else {
                return Some(Err(Error::new(
                    paren_token.span,
                    "\
                        `dyn` trait requires a `self` receiver on this method. \
                        Else opt-out of `dyn` trait support by adding a \
                        `where Self : Sized` clause.\
                    ",
                )));
            };
            let self_lt = Some { 0: &Lifetime::new("'_", Span::mixed_site()) };
            /* From now on, we'll assume "no funky stuff", _e.g._, no generics, etc.
             * since at the time of this writing, this kind of funky stuff is denied for
             * `dyn Trait`s, and we're gonna emit a `dyn_safe(true)` assertion beforehand.
             * we can thus allow to skip checks when we consider the resulting diagnostic
             * noise to be bearable. */
            VTableEntry::VirtualMethod {
                name: method_name,
                each_for_lifetime: if false {
                    // Since CTypes are `'static`, we shouldn't need those lifetimes
                    // when writing the function pointer definitions.
                    generics
                        .lifetimes()
                        .map(|it| &it.lifetime)
                        .collect()
                } else {
                    vec![]
                },
                each_arg_name:
                    inputs
                        .iter()
                        .enumerate()
                        .skip(1)
                        .map(|(i, arg)| ident!("__arg{}", i, span = match *arg {
                            | FnArg::Receiver(_) => {
                                unreachable!("Skipped receiver")
                            },
                            | FnArg::Typed(PatType { ref pat, .. }) => {
                                pat.span()
                            },
                        }))
                        .collect()
                ,
                ErasedSelf: if let Some(_) = self_lt {
                    parse_quote!(
                        // ::safer_ffi::dyn_traits::ErasedRef<#lt>
                        ::safer_ffi::ptr::NonNull<
                            ::safer_ffi::dyn_traits::ErasedTy,
                        >
                    )
                } else {
                    parse_quote!(
                        ::safer_ffi::ptr::NonNull<
                            ::safer_ffi::dyn_traits::ErasedTy,
                        >
                    )
                },
                EachArgTy:
                    inputs
                        .iter()
                        .skip(1)
                        .map(|it| match *it {
                            | FnArg::Receiver(_) => unreachable!(),
                            | FnArg::Typed(PatType { ref ty, .. }) => &**ty,
                        })
                        .collect()
                ,
                OutputTy: match RetTy {
                    | ReturnType::Type(_, it) => ::core::slice::from_ref(it),
                    | ReturnType::Default => &[],
                },
                src: trait_item_method,
            }
        },

        // | TraitItem::Const(TraitItemConst { ref mut attrs, .. })
        // | TraitItem::Type(TraitItemType { ref mut attrs, .. })
        //     if {
        //         skip_attrs_found =
        //             attrs
        //                 .iter()
        //                 .enumerate()
        //                 .filter_map(|(i, attr)| (
        //                     attr.path.is_ident("safer_ffi")
        //                     &&
        //                     attr.parse_args_with(|input: ParseStream<'_>| {
        //                         ::syn::custom_keyword!(skip);
        //                         let _: skip = input.parse()?;
        //                         let _: Option<Token![,]> = input.parse()?;
        //                         Ok(())
        //                     }).is_ok()
        //                 ).then(|| i))
        //                 .collect()
        //         ;
        //         skip_attrs_found.is_empty().not()
        //     }
        // => {
        //     // perform the drain (hack needed since you can't mutate a binding in an `if` guard)
        //     let mut enumerate = 0..;
        //     attrs.retain(|_| skip_attrs_found.contains(&enumerate.next().unwrap()).not());
        //     // skip the current item from the `repr(c) dyn` processing altogether.
        //     continue_!();
        // },

        | TraitItem::Type(_) => {
            failwith!("not supported yet (TBD)" => it);
        },

        | TraitItem::Const(_)
        | TraitItem::Macro(_)
        | TraitItem::Verbatim(_)
        | _
        => failwith!("unsupported" => it),
    })))
    .collect()
}
