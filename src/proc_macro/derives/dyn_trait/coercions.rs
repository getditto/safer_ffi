//! Types which can be coerced to a `VirtualPtr<dyn Trait…>`.

use super::*;

macro_rules! quote {( $($tt:tt)* ) => (
    quote_spanned! { Span::mixed_site()=>
        $($tt)*
    }
)}

pub
enum Coercible {
    Box,
    Rc { thread_safe: bool },
    Ref { mut_: bool },
}

impl Coercible {
    pub
    fn is_eligible (
        self: &'_ Coercible,
        e: &'_ Env<'_>,
    ) -> bool
    {
        match *self {
            | Self::Box { .. } => true,
            | Self::Rc { thread_safe } => e.has_mut.not() && (
                thread_safe == e.is_send && e.is_send == e.is_sync
            ),
            | Self::Ref { mut_: true } => e.Clone.is_none(),
            | Self::Ref { mut_: false } => e.has_mut.not() && (
                // TODO: handle the `-> dyn Send` case for `&`, by amending
                // the default `impl_Trait : Send` as `impl_Trait : Sync`.
                // Until then, just skip that case altogether.
                e.is_sync == e.is_send
            ),
        }
    }

    pub
    fn into_raw (
        self: &'_ Coercible,
        e @ Env { ඞ, .. }: &'_ Env<'_>,
    ) -> TokenStream2
    {
        let this = if e.has_pin {
            squote!(
                #ඞ::pin::Pin::into_inner_unchecked(this)
            )
        } else {
            squote!( this )
        };
        match *self {
            | _ if self.is_eligible(e).not() => unreachable!(),
            | Self::Box => {
                squote!(
                    #ඞ::boxed::Box::into_raw(#this)
                )
            },
            | Self::Rc { thread_safe } => {
                let Rc @ _ = if thread_safe {
                    squote!( #ඞ::sync::Arc )
                } else {
                    squote!( #ඞ::rc::Rc )
                };
                squote!(
                    #Rc::into_raw(#this)
                )
            },
            | Self::Ref { mut_: true } => {
                squote!(
                    #ඞ::ptr::addr_of_mut!(*#this)
                )
            },
            | Self::Ref { mut_: false } => {
                squote!(
                    #ඞ::ptr::addr_of!(*#this)
                )
            },
        }
    }

    pub
    fn release_vptr (
        self: &'_ Coercible,
        e @ Env { ඞ, .. }: &'_ Env<'_>,
    ) -> TokenStream2
    {
        match *self {
            | _ if self.is_eligible(e).not() => <_>::default(),
            | Self::Box => {
                squote!(
                    release_vptr: {
                        unsafe extern "C"
                        fn drop_box #{ e.intro_all_generics } (
                            ptr: #ඞ::ptr::NonNullOwned< #{ e.ErasedTy } >,
                        )
                        #{ e.where_clause }
                        {
                            #ඞ::mem::drop(
                                #{ e.Box }::from_raw(
                                    ptr .cast::<#{ e.impl_Trait }>()
                                        .as_mut_ptr()
                                )
                            );
                        }

                        drop_box::#{ e.fwd_all_generics }
                    },
                )
            },
            Self::Rc { thread_safe } => {
                let Rc @ _ = if thread_safe {
                    squote!( #ඞ::sync::Arc )
                } else {
                    squote!( #ඞ::rc::Rc )
                };
                squote!(
                    release_vptr: {
                        unsafe extern "C"
                        fn drop_ref_count #{ e.intro_all_generics } (
                            ptr: #ඞ::ptr::NonNullOwned< #{ e.ErasedTy } >,
                        )
                        #{ e.where_clause }
                        {
                            #Rc::decrement_strong_count(
                                ptr .cast::<#{ e.impl_Trait }>()
                                    .as_ptr()
                            )
                        }

                        drop_ref_count::#{ e.fwd_all_generics }
                    },
                )
            },
            Self::Ref { .. } => {
                squote!(
                    release_vptr: {
                        unsafe extern "C"
                        fn no_op (
                            ptr: #ඞ::ptr::NonNullOwned< #{ e.ErasedTy } >,
                        )
                        {}

                        no_op
                    },
                )
            },
        }
    }

    pub
    fn retain_vptr (
        self: &'_ Coercible,
        e @ Env { ඞ, .. }: &Env<'_>,
    ) -> TokenStream2
    {
        match *self {
            | _ if self.is_eligible(e).not() => <_>::default(),
            | Self::Box => {
                squote!(
                    retain_vptr: {
                        unsafe extern "C"
                        fn clone_box #{ e.intro_all_generics } (
                            ptr: #ඞ::ptr::NonNullRef< #{ e.ErasedTy } >,
                        ) -> #ඞ::ptr::NonNullOwned< #{ e.ErasedTy } >
                        #{ e.where_clause_with_mb_clone }
                        {
                            #ඞ::mem::transmute(
                                #{ e.Box }::into_raw(#{ e.Box }::new(
                                    <#{ e.impl_Trait } as #ඞ::clone::#{ e.Clone }>::clone(
                                        &*
                                        ptr .cast::<#{ e.impl_Trait }>()
                                            .as_ptr()
                                    )
                                ))
                            )
                        }

                        clone_box::#{ e.fwd_all_generics }
                    },
                )
            },
            Self::Rc { thread_safe } => {
                let Rc @ _ = if thread_safe {
                    squote!( #ඞ::sync::Arc )
                } else {
                    squote!( #ඞ::rc::Rc )
                };
                squote!(
                    retain_vptr: {
                        unsafe extern "C"
                        fn inc_ref_count #{ e.intro_all_generics } (
                            ptr: #ඞ::ptr::NonNullRef< #{ e.ErasedTy } >,
                        ) -> #ඞ::ptr::NonNullOwned< #{ e.ErasedTy } >
                        #{ e.where_clause }
                        {
                            #Rc::increment_strong_count(
                                ptr .cast::<#{ e.impl_Trait }>()
                                    .as_ptr()
                            );
                            /// Now that the ref-count has been incremented, we
                            /// can just copy the ptr.
                            {
                                ptr.0.into()
                            }
                        }

                        inc_ref_count::#{ e.fwd_all_generics }
                    },
                )
            },
            // `&mut` can't be `Clone`.
            Self::Ref { mut_: true } => <_>::default(),
            Self::Ref { mut_: false } => {
                squote!(
                    retain_vptr: {
                        unsafe extern "C"
                        fn copy (
                            ptr: #ඞ::ptr::NonNullRef< #{ e.ErasedTy } >,
                        ) -> #ඞ::ptr::NonNullOwned< #{ e.ErasedTy } >
                        {
                            ptr.0.into()
                        }

                        copy
                    },
                )
            },
        }
    }

    pub
    fn as_type (
        self: &'_ Coercible,
        e @ Env { ඞ, .. }: &Env<'_>,
    ) -> TokenStream2
    {
        let ty = match *self {
            | Self::Box => {
                squote!(
                    #ඞ::boxed::Box<#{ e.impl_Trait }>
                )
            },
            | Self::Rc { thread_safe } => {
                let Rc @ _ = if thread_safe {
                    squote!( #ඞ::sync::Arc )
                } else {
                    squote!( #ඞ::rc::Rc )
                };
                squote!(
                    #Rc<#{ e.impl_Trait }>
                )
            },
            | Self::Ref { mut_ } => {
                let mut_ = mut_.then(|| squote!(mut));
                squote!(
                    & #{ e.lifetime } #mut_ #{ e.impl_Trait }
                )
            },
        };
        if e.has_pin {
            squote!(
                #ඞ::pin::Pin< #ty >
            )
        } else {
            ty
        }
    }
}

// Env.
match_! {(
    ඞ,
    Box, ErasedTy, Trait,
    impl_Trait, lifetime,
    intro_all_generics, intro_trait_generics_and_lt,
    fwd_all_generics, fwd_trait_generics_and_lt,
    where_clause, trait_where_clause, where_clause_with_mb_clone,
) {( $($field:ident),* $(,)? ) => (
    pub
    struct Env<'r> {
        // quote-interpolable/friendly stuff from the caller.
    $(
        pub
        $field : &'r dyn ToTokens,
    )*
        /// Whether we have been provided the `Clone` / `retain` preference.
        pub
        Clone: Option<&'r super::args::kw::Clone>,

        /// Whether we are dealing with `dyn 'lt + Send + … + Trait` (len 0 or 1).
        pub
        is_send: bool,

        /// Whether we are dealing with `dyn 'lt + … + Sync + Trait` (len 0 or 1).
        pub
        is_sync: bool,

        /// Whether there was any occurrence (besides the destructor) of a
        /// method in the trait definition using a `&mut self`-receiver.
        pub
        has_mut: bool,

        /// Whether `self: Pin<…>` occurred.
        pub
        has_pin: bool,
    }
)}}
