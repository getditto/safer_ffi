use {
    ::core::{
        mem, slice,
    },
    super::*,
    self::{
        coercions::{
            Coercible,
        },
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
mod coercions;
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
        dyn_traits::{ErasedTy, VirtualPtr, VirtualPtrFrom},
    };

    let ref coercibles = [
        Coercible::Box,
        Coercible::Ref { mut_: false },
        Coercible::Ref { mut_: true },
        Coercible::Rc { thread_safe: true },
        Coercible::Rc { thread_safe: false },
    ];

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
        &generics.split_for_impl()
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
    let ref impl_Trait = format_ident!("__impl_{}", TraitName);

    // Original generics but for introducing the `'usability` lifetime param.
    let mut trait_generics_and_lt = generics.clone();
    let ref lifetime = squote!(
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
        &trait_generics_and_lt.split_for_impl()
    ;

    let EachToBeInvariantParam @ _ =
        Iterator::chain(
            trait_generics_and_lt
                .lifetimes()
                .map(|LifetimeDef { lifetime, .. }| squote!(
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

    let if_retain = args.clone.is_some().kleenable();

    // Emit the vtable type definition
    let vtable_def = squote!(
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
                    _: #ඞ::ptr::NonNullOwned< #ErasedTy >,
                )
            ,
            #(#if_retain
                retain_vptr:
                    unsafe
                    extern "C"
                    fn (
                        _: #ඞ::ptr::NonNullRef< #ErasedTy >,
                    ) -> #ඞ::ptr::NonNullOwned< #ErasedTy >
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
    let (mut has_pin, mut has_non_pin_non_ref) = (None, None);
    each_vtable_entry
        .iter()
        .filter_map(|it| match *it {
            | VTableEntry::VirtualMethod {
                ref receiver,
                ref src,
                ..
            } => {
                Some((receiver, || src.sig.ident.clone()))
            },
        })
        .for_each(|(receiver, fname)| match receiver {
            | ReceiverType {
                pinned: true,
                ..
            } => {
                has_pin.get_or_insert_with(fname);
            },
            // Tolerate `&self` refs mixed with `Pin`s:
            | ReceiverType {
                kind: ReceiverKind::Reference { mut_: false },
                ..
            } => {
                /* OK */
            },
            | _otherwise => {
                has_non_pin_non_ref.get_or_insert_with(fname);
            },
        })
    ;
    match (&has_pin, has_non_pin_non_ref) {
        | (Some(pinned_span), Some(non_pin_non_ref_span)) => bail! {
            "`Pin<>` receivers can only be mixed with `&self` receivers"
            => quote!(#pinned_span #non_pin_non_ref_span)
        },
        | _ => {},
    }
    let has_pin = has_pin.is_some();
    let (has_ref, has_mut) = (has_mutability(false), has_mutability(true));

    let send_trait = &squote!( #ඞ::marker::Send );
    let sync_trait = &squote!( #ඞ::marker::Sync );
    for &(is_send, is_sync) in &[
        (false, false),
        (true, true),
        (true, false),
        (false, true),
    ]
    {
        // given the *unconditional* presence of drop glue, the thread-safe
        // intent of `Sync` without `Send` is a code smell:
        if is_sync && is_send.not() {
            continue;
        }
        // with at least one `&self` method, the dual applies as well:
        if has_ref && is_send && is_sync.not() {
            continue;
        }
        let mb_send = is_send.then(|| send_trait);
        let mb_sync = is_sync.then(|| sync_trait);
        // Make `Send` and `Sync` be `#( … )*` usable (no `#( … )?`).
        let Send @ _ = mb_send.kleenable();
        let Sync @ _ = mb_sync.kleenable();
        let ref Trait @ _ = squote!(
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
            &all_generics.split_for_impl()
        ;
            let mut storage = None;
        let ref where_clause_with_mb_clone = if args.clone.is_some() {
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

        let QSelf @ _ = squote!(
            <#impl_Trait as #TraitName #fwd_trait_generics>
        );

        let ref EACH_VTABLE_ENTRY_VALUE @ _ =
            each_vtable_entry_value_f.iter().vmap(|f| f(&QSelf, &all_generics))
        ;
        let ref e = coercions::Env {
            ඞ,
            Box, ErasedTy, Trait,
            impl_Trait, lifetime,
            intro_all_generics, intro_trait_generics_and_lt,
            fwd_all_generics, fwd_trait_generics_and_lt,
            where_clause, trait_where_clause, where_clause_with_mb_clone,

            Clone: args.clone.as_ref(),
            is_send,
            is_sync,
            has_mut,
            has_pin,
        };
        let vtable_expr = |src: &Coercible| squote!(
            #VTableName {
                #{ src.release_vptr(e) }
                #(#if_retain
                    #{ src.retain_vptr(e) }
                )*
            #(
                #(#each_vtable_entry_attrs)*
                #each_vtable_entry_name: #EACH_VTABLE_ENTRY_VALUE,
            )*
                _invariant: #ඞ::marker::PhantomData,
            }
        );
        let constructors: TokenStream2 =
            coercibles
            .iter()
            .filter(|it| it.is_eligible(e))
            .map(|src: &Coercible| squote!(
                impl #intro_all_generics
                    #VirtualPtrFrom<
                        #{ src.as_type(e) }
                    >
                for
                    dyn #Trait
                                        #{
                                            if matches!(src, Coercible::Box) {
                where_clause_with_mb_clone
                                            } else {
                where_clause
                                            }
                                        }
                {
                    fn into_virtual_ptr (
                        this: #{ src.as_type(e) },
                    ) -> #VirtualPtr<dyn #Trait>
                    {
                        unsafe {
                            #VirtualPtr::<dyn #Trait>::from_raw_parts(
                                #ඞ::mem::transmute(
                                    #{ src.into_raw(e) }
                                ),
                                #{ vtable_expr(src) },
                            )
                        }
                    }
                }
            ))
            .collect()
        ;
        ret.extend(squote!(
            impl #intro_trait_generics_and_lt
                ::safer_ffi::dyn_traits::ReprCTrait
            for
                dyn #Trait
            #trait_where_clause
            {
                type VTable = #VTableName #fwd_trait_generics_and_lt;

                unsafe
                fn drop_ptr (
                    ptr: #ඞ::ptr::NonNullOwned<#ErasedTy>,
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
                fn dyn_clone (this: &#VirtualPtr<Self>)
                  -> #VirtualPtr<Self>
                {
                    let (ptr, vtable) = (this.__ptr(), this.__vtable());
                    unsafe {
                        let ptr = (vtable.retain_vptr)(ptr.into());
                        #VirtualPtr::from_raw_parts(
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
                #VirtualPtr<dyn #Trait>
            #trait_where_clause
            {
                #(#each_method_def)*
            }

            #constructors
        ));
    }
    drop(each_vtable_entry_value_f);
    ret = squote!(
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
    squote!(
        ::safer_ffi::ඞ::CLayoutOf<#T>
    )
}
