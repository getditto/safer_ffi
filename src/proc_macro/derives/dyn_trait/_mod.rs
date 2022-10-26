#![cfg_attr(rustfmt, rustfmt::skip)]

use {
    ::core::{
        mem, slice,
    },
    super::*,
    self::{
        receiver_types::{
            ReceiverKind,
            ReceiverType,
        },
        vtable_entry::{
            VTableEntry,
            vtable_entries,
        },
    }
};

mod args;
mod receiver_types;
mod vtable_entry;

enum Input {
    Trait(ItemTrait),
    TokenStream(TokenStream2),
}

impl Parse for Input {
    fn parse (input: ParseStream<'_>)
      -> Result<Input>
    {Ok({
        let ref fork = input.fork();
        fork.parse::<ItemTrait>()
            .map(|trait_| {
                ::syn::parse::discouraged::Speculative::advance_to(input, fork);
                Input::Trait(trait_)
            })
            .unwrap_or_else(|_| Input::TokenStream(input.parse().unwrap()))
    })}
}

pub(in super)
fn try_handle_trait (
    args: &'_ mut TokenStream2,
    input: &'_ mut TokenStream2,
) -> Result< Option<TokenStream2> >
{
    #[apply(let_quote!)]
    use ::safer_ffi::{
        ඞ,
        ඞ::Box,
        dyn_traits::ErasedTy,
    };

    let ref mut trait_ = match parse2(mem::take(input)).unwrap() {
        | Input::TokenStream(ts) => {
            *input = ts.into();
            return Ok(None);
        },
        | Input::Trait(it) => {
            *input = it.to_token_stream().into();
            it
        },
    };
    let args: args::Args = parse2(mem::take(args))?;
    let Klone @ _ = args.clone.as_ref();
    let mut ret = TokenStream2::new();
    let ItemTrait {
        attrs: _,
        vis: ref pub_,
        unsafety: _, // FIXME
        auto_token: _,
        trait_token: _,
        ident: ref TraitName @ _,
        ref generics,
        colon_token: _,
        supertraits: _,
        brace_token: _,
        ref mut items,
    } = *trait_;

    let (_, fwd_trait_generics, trait_where_clause) =
        generics.split_for_impl()
    ;

    let ref each_vtable_entry = vtable_entries(items, &mut ret)?;
    let each_method_def = each_vtable_entry.vmap(VTableEntry::virtual_forwarding);
    let each_vtable_entry_name = each_vtable_entry.vmap(VTableEntry::name);
    let each_vtable_entry_attrs = each_vtable_entry.vmap(VTableEntry::attrs);
    let (EachVTableEntryType @ _, each_vtable_entry_value_f) =
        each_vtable_entry
            .iter()
            .map(VTableEntry::type_and_value)
            .unzip::<_, _, Vec<_>, Vec<_>>()
    ;
    let VTableName @ _ = format_ident!("{}VTable", TraitName);
    let impl_Trait = format_ident!("__impl_{}", TraitName);

    // Original generics but for introducing the `'usability` lifetime param.
    let mut trait_generics_and_lt = generics.clone();
    let lifetime = quote_spanned!(Span::mixed_site()=>
        '__usability
    );
    trait_generics_and_lt.params.insert(0, parse_quote!(
        #lifetime
    ));
    trait_generics_and_lt
        .make_where_clause()
        .predicates
        .extend_::<WherePredicate, _>(
            generics
                .type_params()
                .map(|it| &it.ident)
                .map(|T @ _| parse_quote!(
                    #T : #lifetime
                ))
        )
    ;
    let ref trait_generics_and_lt = trait_generics_and_lt;
    let (intro_trait_generics_and_lt, fwd_trait_generics_and_lt, _) =
        trait_generics_and_lt.split_for_impl()
    ;

    let EachToBeInvariantParam @ _ =
        Iterator::chain(
            trait_generics_and_lt
                .lifetimes()
                .map(|LifetimeDef { lifetime, .. }| quote!(
                    // We need something which *names* `lifetime`,
                    // but which "yields" / results in a 'static CType.
                    // So let's use the
                    // non-generic-assoc-type-of-a-generic-trait trick for this.
                    <u8 as #ඞ::IdentityIgnoring<#lifetime>>::ItSelf
                ))
            ,
            trait_generics_and_lt.type_params().map(|ty| {
                ty.ident.to_token_stream()
            })
        )
    ;

    let if_retain: Option<TokenStream2> = args.clone.as_ref().map(|_| quote!());
    // Since there is no `#( … )?`, use a slice to make it `#( … )*` usable.
    let if_retain: &[TokenStream2] = if_retain.as_ref().map_or(&[][..], slice::from_ref);

    // Emit the vtable type definition
    let vtable_def = quote_spanned!(Span::mixed_site()=>
        #[#ඞ::derive_ReprC]
        #[repr(C)]
        #pub_
        struct #VTableName #intro_trait_generics_and_lt
        #trait_where_clause
        {
            release_vptr:
                unsafe
                extern "C"
                fn (
                    _: ::safer_ffi::ptr::NonNullOwned< #ErasedTy >,
                )
            ,
            #(#if_retain
                retain_vptr:
                    unsafe
                    extern "C"
                    fn (
                        _: ::safer_ffi::ptr::NonNullRef< #ErasedTy >,
                    ) -> ::safer_ffi::ptr::NonNullOwned< #ErasedTy >
                ,
            )*
        #(
            #(#each_vtable_entry_attrs)*
            #each_vtable_entry_name: #EachVTableEntryType,
        )*
            _invariant:
                ::core::marker::PhantomData<
                    *mut (#(
                        #EachToBeInvariantParam,
                    )*)
                >
            ,
        }
    );

    let has_mutability = |mutability: bool| {
        each_vtable_entry.iter().any(|it| matches!(
            *it,
            VTableEntry::VirtualMethod {
                receiver: ReceiverType {
                    kind: ReceiverKind::Reference { mut_ },
                    ..
                },
                ..
            }
            if mut_ == mutability
        ))
    };
    let (has_ref, has_mut) = (has_mutability(false), has_mutability(true));

    let send_trait = &quote!(::core::marker::Send);
    let sync_trait = &quote!(::core::marker::Sync);
    let ref mut must_emit_generic_vtable_reference = true;
    for &(is_send, is_sync) in &[
        (false, false),
        (true, true),
        (true, false),
        /* given the presence of drop glue, thread-safe requires `Send`. */
        // (false, true),
    ]
    {
        if has_ref && is_send && is_sync.not() {
            // with a `&self` method, using `+ Send` only is a code smell too.
            continue;
        }
        // Make `Send` and `Sync` be `#( … )*` usable (no `#( … )?`).
        let Send @ _ = if is_send { slice::from_ref(send_trait) } else { &[] };
        let Sync @ _ = if is_sync { slice::from_ref(sync_trait) } else { &[] };
        let Trait @ _ = quote!(
            #lifetime #(+ #Send)* #(+ #Sync)* + #TraitName #fwd_trait_generics
        );

        // trait_generics_and_lt + `impl_Trait` generic parameter.
        let mut all_generics = trait_generics_and_lt.clone();
        let param_count = <usize as ::core::ops::Add>::add(
            all_generics.lifetimes().count(),
            all_generics.type_params().count(),
        );
        all_generics.params.insert(param_count, parse_quote!(
            #impl_Trait : #Trait
        ));
        let (intro_all_generics, fwd_all_generics, where_clause) =
            all_generics.split_for_impl()
        ;
            let mut storage = None;
        let where_clause_with_mb_clone = if args.clone.is_some() {
            Some(&*storage.get_or_insert(
                all_generics
                    .where_clause
                    .as_ref()
                    .map_or_else(|| parse_quote!(where), Clone::clone)
                    .also(|it| it
                        .predicates
                        .extend_one_::<WherePredicate>(
                            parse_quote!(
                                #impl_Trait : #Klone
                            )
                        )
                    )
            ))
        } else {
            all_generics.where_clause.as_ref()
        };

        let QSelf @ _ = quote!(
            <#impl_Trait as #TraitName #fwd_trait_generics>
        );

        let EACH_VTABLE_ENTRY_VALUE @ _ =
            each_vtable_entry_value_f.iter().map(|f| f(&QSelf, &all_generics))
        ;
        let VTABLE_EXPR @ _ = quote_spanned!(Span::mixed_site()=>
            &#VTableName {
                release_vptr: {
                    unsafe extern "C"
                    fn release_vptr #intro_all_generics (
                        ptr: ::safer_ffi::ptr::NonNullOwned< #ErasedTy >,
                    )
                    #where_clause
                    {
                        let ptr = ptr.cast::<#impl_Trait>();
                        ::core::mem::drop(
                            #Box::from_raw(
                                { ptr }.as_mut_ptr()
                            )
                        )
                    }
                    release_vptr::#fwd_all_generics // as …
                },
                #(#if_retain
                    retain_vptr: {
                        unsafe
                        extern "C"
                        fn unimplemented (_: #ඞ::ptr::NonNullRef<#ErasedTy>)
                          -> #ඞ::ptr::NonNullOwned<#ErasedTy>
                        {
                            ::std::process::abort();
                        }
                        unimplemented
                    },
                )*
            #(
                #(#each_vtable_entry_attrs)*
                #each_vtable_entry_name: #EACH_VTABLE_ENTRY_VALUE,
            )*
                _invariant: ::core::marker::PhantomData,
            }
        );
        if mem::take(must_emit_generic_vtable_reference) {
            ret.extend(quote_spanned!(Span::mixed_site()=>
                struct __GenericConst #intro_all_generics (
                    *mut Self,
                );

                impl #intro_all_generics
                    __GenericConst #fwd_all_generics
                #where_clause
                {
                    #[allow(unused_parens)]
                    const VALUE: &#lifetime #VTableName #fwd_trait_generics_and_lt = (
                        #VTABLE_EXPR
                    );
                }
            ));
        }
        let ref_and_mut = [quote!(), quote!(mut)];
        let mb_mut = {
            let from_shared = true
                && has_mut.not()
                && (
                    // FIXME: handle this with the proper bounds on the `From<&T>` impl.
                    is_send.not() || is_sync
                )
            ;
            let from_mut = args.clone.is_none();
            &ref_and_mut[
                match (from_shared, from_mut) {
                    (true, true) => 0..2,
                    (true, false) => 0..1,
                    (false, true) => 1..2,
                    (false, false) => 1..1,
                }
            ]
        };
        let retain_vptr = quote_spanned!(Span::mixed_site()=>
            #(#if_retain
                // if we are here, we are necessarily dealing with a `&T`,
                // which is Copy.
                retain_vptr: {
                    extern "C"
                    fn copy (
                        p: #ඞ::ptr::NonNullRef<#ErasedTy>,
                    ) -> #ඞ::ptr::NonNullOwned<#ErasedTy>
                    {
                        ::core::convert::Into::into(p.0)
                    }

                    copy
                },
            )*
        );
        ret.extend(quote_spanned!(Span::mixed_site()=>
            impl #intro_trait_generics_and_lt
                ::safer_ffi::dyn_traits::ReprCTrait
            for
                dyn #Trait
            #trait_where_clause
            {
                type VTable = #VTableName #fwd_trait_generics_and_lt;

                unsafe
                fn drop_ptr (
                    ptr: ::safer_ffi::ptr::NonNullOwned<#ErasedTy>,
                    &Self::VTable { release_vptr, .. }: &'_ Self::VTable,
                )
                {
                    release_vptr(ptr)
                }
            }

            #(#if_retain
                impl<#lifetime>
                    ::safer_ffi::dyn_traits::DynClone
                for
                    dyn #Trait
                {
                    fn dyn_clone (this: &::safer_ffi::dyn_traits::VirtualPtr<Self>)
                      -> ::safer_ffi::dyn_traits::VirtualPtr<Self>
                    {
                        let (ptr, vtable) = (this.__ptr(), this.__vtable());
                        unsafe {
                            let ptr = (vtable.retain_vptr)(ptr.into());
                            ::safer_ffi::dyn_traits::VirtualPtr::from_raw_parts(
                                ptr,
                                #VTableName { ..*vtable },
                            )
                        }
                    }
                }
            )*


            impl #intro_trait_generics_and_lt
                #TraitName #fwd_trait_generics
            for
                ::safer_ffi::dyn_traits::VirtualPtr<dyn #Trait>
            #trait_where_clause
            {
                #(#each_method_def)*
            }

            // Constructor / from impls:
            #(
                // `&T` and `&mut T`.
                impl #intro_all_generics
                    ::safer_ffi::dyn_traits::VirtualPtrFrom<
                        & #lifetime #mb_mut #impl_Trait
                    >
                for
                    dyn #Trait
                #where_clause
                {
                    fn into_virtual_ptr (
                        r: & #lifetime #mb_mut #impl_Trait,
                    ) -> ::safer_ffi::dyn_traits::VirtualPtr<dyn #Trait>
                    {
                        unsafe {
                            ::safer_ffi::dyn_traits::VirtualPtr::<dyn #Trait>::
                            from_raw_parts(
                                ::core::mem::transmute(r),
                                #VTableName {
                                    release_vptr: {
                                        extern "C"
                                        fn no_op (_: #ඞ::ptr::NonNullOwned<#ErasedTy>)
                                        {}

                                        no_op
                                    },
                                    #retain_vptr
                                    ..*<__GenericConst #fwd_all_generics>::VALUE
                                },
                            )
                        }
                    }
                }
            )*

            // `Box<T>`
            impl #intro_all_generics
                ::safer_ffi::dyn_traits::VirtualPtrFrom<#Box<#impl_Trait>>
            for
                dyn #Trait
            #where_clause_with_mb_clone
            {
                fn into_virtual_ptr (
                    boxed: #Box<#impl_Trait>
                ) -> ::safer_ffi::dyn_traits::VirtualPtr<dyn #Trait>
                {
                    unsafe {
                        ::safer_ffi::dyn_traits::VirtualPtr::<dyn #Trait>::
                        from_raw_parts(
                            ::core::mem::transmute(boxed),
                            #VTableName {
                                #(#if_retain
                                    retain_vptr: {
                                        unsafe extern "C"
                                        fn retain_vptr #intro_all_generics (
                                            ptr: ::safer_ffi::ptr::NonNullRef< #ErasedTy >,
                                        ) -> ::safer_ffi::ptr::NonNullOwned< #ErasedTy >
                                        #where_clause_with_mb_clone
                                        {
                                            let ptr = ptr.cast::<#impl_Trait>();
                                            ::core::mem::transmute(
                                                #Box::<#impl_Trait>::new(
                                                    ::core::clone::#Klone::clone(ptr.as_ref())
                                                )
                                            )
                                        }
                                        retain_vptr::#fwd_all_generics // as …
                                    },
                                )*
                                ..*<__GenericConst #fwd_all_generics>::VALUE },
                        )
                    }
                }
            }
        ));
    }
    drop(each_vtable_entry_value_f);
    ret = quote!(
        #trait_

        #vtable_def

        #[allow(warnings, clippy::all)]
        const _: () = {
            #ret
        };
    );
    Ok(Some(ret))
}

fn CType (ty: &'_ Type)
  -> TokenStream2
{
    /* replace lifetimes inside `T` with … `'static`?? */
    let mut T = ty.clone();
    ::syn::visit_mut::VisitMut::visit_type_mut(
        &mut RemapNonStaticLifetimesTo { new_lt_name: "static" },
        &mut T,
    );
    quote!(
        ::safer_ffi::ඞ::CLayoutOf<#T>
    )
}
