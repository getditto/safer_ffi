#![cfg_attr(rustfmt, rustfmt::skip)]

use {
    ::core::{
        mem, slice,
    },
    super::*,
    self::{
        vtable_entry::{
            VTableEntry,
            vtable_entries,
        },
    }
};

enum Input {
    Trait(ItemTrait),
    TokenStream(TokenStream2),
}

mod vtable_entry;

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
    attrs: &'_ TokenStream2,
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
    let VTableName @ _ = format_ident!("{}Vtable", TraitName);
    let impl_Trait = format_ident!("__impl_{}", TraitName);

    // Original generics but for introducing the `'usability` lifetime param.
    let mut trait_generics_and_lt = generics.clone();
    let lifetime = quote_spanned!(Span::mixed_site()=>
        '__usability
    );
    trait_generics_and_lt.params.insert(0, parse_quote!(
        #lifetime
    ));
    let ref trait_generics_and_lt = trait_generics_and_lt;
    let (intro_trait_generics_and_lt, fwd_trait_generics_and_lt, _) =
        trait_generics_and_lt.split_for_impl()
    ;

    let EachToBeInvariantParam @ _ =
        Iterator::chain(
            trait_generics_and_lt.lifetimes().map(|LifetimeDef { lifetime, .. }| quote!(
                &#lifetime ()
            )),
            trait_generics_and_lt.type_params().map(|ty| {
                ty.ident.to_token_stream()
            })
        )
    ;

    // Emit the vtable type definition
    ret.extend(quote!(
        // #[cfg_eval]
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
            retain_vptr: ::core::option::Option<
                unsafe
                extern "C"
                fn (
                    _: ::safer_ffi::ptr::NonNullRef< #ErasedTy >,
                )
            >,
        #(
            #(#each_vtable_entry_attrs)*
            #each_vtable_entry_name: #EachVTableEntryType,
        )*
            // __type_name__debug: ::core::option::Option<
            //     extern "C"
            //     fn() -> #ඞ::string::str_ref<'static>
            // >,
            _invariant:
                ::core::marker::PhantomData<
                // &#lifetime mut Self,
                // &#lifetime mut #VTableName #fwd_trait_generics_and_lt,
                    fn(&()) -> &mut (#(
                        #EachToBeInvariantParam,
                    )*)
                >
            ,
        }
    ));

    let Send @ _ = &[quote!(::core::marker::Send)][..];
    let Sync @ _ = &[quote!(::core::marker::Sync)][..];
    let ref mut must_emit_generic_vtable_reference = true;
    for &(is_send, is_sync) in &[
        (false, false),
        (true, true),
        (true, false),
        (false, true),
    ]
    {
        let MbSend @ _ = if is_send { Send } else { &[] };
        let MbSync @ _ = if is_sync { Sync } else { &[] };
        let Trait @ _ = quote!(
            #lifetime #(+ #MbSend)* #(+ #MbSync)* + #TraitName #fwd_trait_generics
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

        let QSelf @ _ = quote!(
            <#impl_Trait as #TraitName #fwd_trait_generics>
        );

        let EACH_VTABLE_ENTRY_VALUE @ _ =
            each_vtable_entry_value_f.iter().map(|f| f(&QSelf, &all_generics))
        ;
        if mem::take(must_emit_generic_vtable_reference) {
            ret.extend(quote_spanned!(Span::mixed_site()=>
                struct __GenericConst #intro_all_generics (
                    *mut Self,
                );

                impl #intro_all_generics
                //     ::safer_ffi::dyn_traits::__AssocConst<
                //         &#lifetime <
                //             dyn #Trait
                //             as
                //             ::safer_ffi::dyn_traits::ReprCTrait
                //         >::VTable
                //     >
                // for
                    __GenericConst #fwd_all_generics
                #where_clause
                {
                    #[allow(unused_parens)]
                    const VALUE
                    : (
                        &#lifetime <
                            dyn #Trait
                            as
                            ::safer_ffi::dyn_traits::ReprCTrait
                        >::VTable
                    ) = (
                        &#VTableName {
                            // __type_name__debug: Some({
                            //     extern "C"
                            //     fn __type_name__debug<#impl_Trait> ()
                            //       -> #ඞ::string::str_ref<'static>
                            //     {
                            //         let s: &'static ::core::primitive::str =
                            //             ::core::any::type_name::<#impl_Trait>()
                            //         ;
                            //         let ptr = s.as_bytes().as_ptr() as *mut u8;
                            //         let len = s.len();
                            //         unsafe {
                            //             ::core::mem::transmute::<
                            //                 #ඞ::slice::slice_raw_Layout<u8>,
                            //                 #ඞ::string::str_ref<'static>,
                            //             >(
                            //                 safer_ffi::slice::slice_raw_Layout {
                            //                     ptr,
                            //                     len,
                            //                 }
                            //             )
                            //         }
                            //     }
                            //     __type_name__debug::<#impl_Trait>
                            // }),
                            release_vptr: {
                                unsafe extern "C"
                                fn release_vptr<#impl_Trait : #TraitName #fwd_trait_generics> (
                                    ptr: ::safer_ffi::ptr::NonNullOwned< #ErasedTy >,
                                )
                                {
                                    let ptr = ptr.cast::<#impl_Trait>();
                                    ::core::mem::drop(
                                        #Box::from_raw(
                                            { ptr }.as_mut_ptr()
                                        )
                                    )
                                }
                                release_vptr::<#impl_Trait> // as …
                            },
                            retain_vptr: None,
                        #(
                            #(#each_vtable_entry_attrs)*
                            #each_vtable_entry_name: #EACH_VTABLE_ENTRY_VALUE,
                        )*
                            _invariant: ::core::marker::PhantomData,
                        }
                    );
                }
            ));
        }
        ret.extend(quote_spanned!(Span::mixed_site()=>
            impl #intro_trait_generics_and_lt
                ::safer_ffi::dyn_traits::ReprCTrait
            for
                dyn #Trait
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

                // fn type_name (
                //     &Self::VTable { __type_name__debug, .. }: &'_ Self::VTable,
                // ) -> &'static ::core::primitive::str
                // {
                //     if let ::core::option::Option::Some(f) = __type_name__debug {
                //         f().as_str()
                //     } else {
                //         "<no debug type name available>"
                //     }
                // }
            }

            impl #intro_trait_generics_and_lt
                #TraitName #fwd_trait_generics
            for
                ::safer_ffi::dyn_traits::VirtualPtr<dyn #Trait>
            {
                #(#each_method_def)*
            }

            impl #intro_all_generics
                ::safer_ffi::dyn_traits::VirtualPtrFromBox<#impl_Trait>
            for
                dyn #Trait
            {
                fn boxed_into_virtual_ptr (
                    boxed: #Box<#impl_Trait>
                ) -> ::safer_ffi::dyn_traits::VirtualPtr<dyn #Trait>
                {
                    unsafe {
                        ::safer_ffi::dyn_traits::VirtualPtr::<dyn #Trait>::
                        from_raw_parts(
                            ::core::mem::transmute(boxed),
                            ::core::convert::Into::into(
                                <
                                    __GenericConst #fwd_all_generics
                                    // as
                                    // ::safer_ffi::dyn_traits::__AssocConst<
                                    //     &#lifetime <
                                    //         dyn #Trait
                                    //         as
                                    //         ::safer_ffi::dyn_traits::ReprCTrait
                                    //     >::VTable
                                    // >
                                >::VALUE
                            ),
                        )
                    }
                }
            }
        ));
    }
    let _: parse::Nothing = parse2(attrs.clone())?;
    drop(each_vtable_entry_value_f);
    ret = quote!(
        #trait_

        #[allow(warnings, clippy::all)]
        const _: () = { #ret };
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
