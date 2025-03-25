#![allow(dead_code)] // FIXME.

use super::*;

pub(super) enum ReceiverKind {
    Reference { mut_: bool },
    Box,
    Arc,
}

pub(super) struct ReceiverType {
    pub(super) kind: ReceiverKind,

    pub(super) pinned: bool,
}

impl ReceiverType {
    pub(crate) fn from_fn_arg(fn_arg: &'_ mut FnArg) -> Result<ReceiverType> {
        let pinned = false;
        let mut storage = None;
        Self::from_type_of_self(
            match fn_arg {
                | &mut FnArg::Receiver(Receiver {
                    attrs: _,
                    reference: ref ref_,
                    mutability: ref mut_,
                    self_token: token::SelfValue { span },
                }) => storage.get_or_insert({
                    let Self_ = Ident::new(
                        "Self", span, // .resolved_at(Span::mixed_site()),
                    );
                    if let Some((and, mb_lt)) = ref_ {
                        parse_quote!(
                            #and #mb_lt #mut_ #Self_
                        )
                    } else {
                        parse_quote!(
                            #Self_
                        )
                    }
                }),
                | FnArg::Typed(PatType { pat, ty, .. }) => match **pat {
                    | Pat::Ident(PatIdent { ref ident, .. }) if ident == "self" => ty,
                    | _ => bail! {
                        "expected `self`" => pat,
                    },
                },
            },
            pinned,
        )
    }

    fn from_type_of_self(
        type_of_self: &'_ mut Type,
        pinned: bool,
    ) -> Result<ReceiverType> {
        // let ref mut storage = None;
        // let lifetime_of_and = move |and: &Token![&], mb_lt: &'i Option<Lifetime>| {
        //     mb_lt.as_ref().unwrap_or_else(|| {
        //         { storage }.get_or_insert(
        //             Lifetime::new("'_", and.span)
        //         )
        //     })
        // };

        let is_Self = |T: &Type| {
            matches!(
                *T, Type::Path(TypePath {
                    qself: None, ref path,
                }) if path.is_ident("Self")
            )
        };

        Ok(match *type_of_self {
            // `: Self`
            | _ if is_Self(type_of_self) => bail! {
                "owned `Self` receiver is not `dyn` safe" => type_of_self,
            },

            // `: &[mut] Self`
            | Type::Reference(TypeReference {
                // and_token: ref and,
                mutability: ref mut_,
                // lifetime: ref mb_lt,
                elem: ref Pointee @ _,
                ..
            }) if is_Self(Pointee) => ReceiverType {
                pinned,
                kind: ReceiverKind::Reference {
                    mut_: mut_.is_some(),
                },
            },

            // `: path::to::SomeWrapper<…>`
            | Type::Path(TypePath {
                qself: None,
                path: ref mut ty_path,
            }) => {
                use AngleBracketedGenericArguments as Generic;
                fn extract_generic_ty(args: &'_ mut syn::PathArguments) -> Option<&'_ mut Type> {
                    match args {
                        | PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                            args,
                            ..
                        }) if args.len() == 1 => match args[0] {
                            | GenericArgument::Type(ref mut inner) => Some(inner),
                            | _ => None,
                        },
                        | _ => None,
                    }
                }

                // `SomeWrapper<inner>`
                let last = ty_path.segments.last_mut().unwrap();
                let ret = match (
                    &last.ident.to_string()[..],
                    extract_generic_ty(&mut last.arguments),
                ) {
                    // `Box<Self>`
                    | ("Box", Some(inner)) if is_Self(inner) => Self {
                        pinned,
                        kind: ReceiverKind::Box,
                    },
                    // `Arc<Self>`
                    | ("Arc", Some(inner)) if is_Self(inner) => Self {
                        pinned,
                        kind: ReceiverKind::Arc,
                    },
                    | _ if pinned => bail! {
                        "\
                            expected one of `&`, `&mut`, `Box<_>`, or `Arc<_>` \
                            (more complex `Self` types are not supported)\
                        " => last,
                    },
                    // `Pin<…>`
                    | ("Pin", Some(inner)) => {
                        let pinned = true;
                        Self::from_type_of_self(inner, pinned)?
                    },
                    | _ => bail! {
                        "\
                            expected one of `&`, `&mut`, `Box<_>`, `Arc<_>`, or `Pin<_>` \
                            (more complex `Self` types are not supported)\
                        " => last,
                    },
                };
                // Replace any encountered `Box`,`Arc`,`Pin`, with *our* fully qualified to the
                // expected item, to guard against silly shadowings.
                ty_path.leading_colon = Some(token::Colon2 {
                    spans: [last.span(), last.span()],
                });
                ty_path.segments = Punctuated::parse_separated_nonempty
                    .parse2(quote_spanned!(last.span()=>
                        safer_ffi::ඞ::#last
                    ))
                    .unwrap();
                ret
            },

            // `([<Something as Complex>::Assoc; 3], bool)`
            | _ => bail! {
                "arbitrary `Self` types are not supported" => type_of_self,
            },
        })
    }
}
