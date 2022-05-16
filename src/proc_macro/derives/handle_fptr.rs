use {
    ::syn::{
        visit_mut::VisitMut,
    },
    super::*,
};

pub(in super)
fn try_handle_fptr (
    input: &'_ DeriveInput,
) -> Option<TokenStream2>
{
    let span = Span::call_site();

    macro_rules! error {(
        $msg:expr $(=> $span:expr)? $(,)?
    ) => ({
        $( let span = $span; )?
        return Some(Error::new(span, &*$msg).to_compile_error().into());
    })}
    macro_rules! bail {() => ({
        // dbg!();
        return None;
    })}

    if let &DeriveInput {
        ref attrs,
        ref vis,
        ident: ref StructName,
        ref generics,
        // Check that it is a unit struct with one field
        data: Data::Struct(DataStruct {
            ref struct_token,
            fields: Fields::Unnamed(FieldsUnnamed {
                unnamed: ref fields, ..
            }),
            ..
        }),
    } = input
    {
        if fields.len() != 1 { bail!(); }
        if matches!(vis, Visibility::Public(_)).not() {
            error!("Missing `pub`" => struct_token.span());
        }
        mod kw {
            ::syn::custom_keyword!(transparent);
        }
        match attrs.iter().find(|attr| attr.path.is_ident("repr")) {
            | Some(attr) => match attr.parse_args::<kw::transparent>() {
                | Ok(_) => {},
                | Err(_) => bail!(),
            },
            | None => error!("Missing `#[repr(…)]` annotation"), // or bail!() and let the parent handle the error
        }
        // Check that the given ty is an `fn` pointer type.
        let cb_ty = match fields.iter().next().unwrap() {
            | Field { ty: Type::BareFn(ref cb_ty), ref vis, .. } => {
                if matches!(vis, Visibility::Public(_)).not() {
                    error!("Missing `pub`" => cb_ty.span());
                }
                cb_ty
            },
            | _ => bail!(),
        };
        if let Some(ref v) = cb_ty.variadic {
            error!("`safer-ffi` does not support variadics" => v.span());
        }
        // Check that it is `extern "C"`.
        match *cb_ty {
            | TypeBareFn {
                abi: Some(Abi { name: Some(ref abi), .. }),
                ..
            }
                if abi.value() != "C"
            => {
                error!("Expected `\"C\"`" => abi.span());
            },

            | TypeBareFn {
                abi: Some(_),
                ..
            }
            => {}

            | _ => error!(
                "Missing `extern \"C\"`" => cb_ty.fn_token.span()
            ),
        }

        /* == VALIDATION PASSED, TIME TO EXPAND == */
        // Fully-qualified paths to be robust to an weird/antagonistic
        // namespace (except for `::safer_ffi`; that's our path-resolution
        // keystone.
        #[apply(let_quote!)]
        use ::safer_ffi::{
            layout::{
                CType,
                CType,
                LegacyCType,
                OpaqueKind,
                ReprC,
                __HasNiche__,
            },
            headers::Definer,
            __cfg_headers__,
            __cfg_csharp__,
            ඞ::{
                bool,
                core,
                fmt,
                Option,
                std,
                str,
                write,
            },
        };

        let (intro, fwd, where_) = input.generics.split_for_impl();
        let mut ret = quote!(
            #[allow(nonstandard_style)]
            #input

            /// Convenience deref impl, so as to have the wrapper type
            /// behave as if it were a function pointer directly :)
            impl #intro
                #core::ops::Deref
            for
                #StructName #fwd
            #where_
            {
                type Target = #cb_ty;

                #[inline]
                fn deref (self: &'_ #StructName #fwd)
                  -> &'_ Self::Target
                {
                    &self.0
                }
            }
        );

        let EachArgCType: Vec<Type> =
            cb_ty
                .inputs
                .iter()
                .map(|arg| {
                    let mut ty = arg.ty.clone();
                    StripLifetimeParams.visit_type_mut(&mut ty);
                    ty = parse_quote!(
                        <#ty as #ReprC>::CLayout
                    );
                    ty
                })
                .collect()
        ;
        let ref mut RetCType = match cb_ty.output {
            | ReturnType::Default => parse_quote!( () ),
            | ReturnType::Type(_, ref ty) => {
                let mut ty = (&**ty).clone();
                StripLifetimeParams.visit_type_mut(&mut ty);
                ty
            },
        };
        *RetCType = parse_quote!(
            <#RetCType as #ReprC>::CLayout
        );


        let mut input_Layout = DeriveInput {
            attrs: vec![],
            vis: vis.clone(),
            ident: format_ident!("{}_Layout", StructName),
            generics: generics.clone(),
            data: input.data.clone(),
        };
        let ref StructName_Layout = input_Layout.ident;
        let ref lifetimes =
            cb_ty
                .lifetimes
                .as_ref()
                .map(|it| it
                    .lifetimes
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                )
                .unwrap_or_default()
        ;
        let ref repr_c_clauses: Vec<WherePredicate> =
            cb_ty
                .inputs
                .iter()
                .map(|input| &input.ty)
                .chain(match cb_ty.output {
                    | ReturnType::Default => None,
                    | ReturnType::Type(_, ref ty) => Some(&**ty),
                })
                .flat_map(|ty| {
                    let ref mut ty = ty.clone();
                    let ref mut lifetimes =
                        ::std::borrow::Cow::<'_, Vec<_>>::Borrowed(lifetimes)
                    ;
                    UnelideLifetimes {
                        lifetime_params: lifetimes,
                        counter: (0 ..),
                    }.visit_type_mut(ty);
                    let is_ReprC: WherePredicate = parse_quote!(
                        for<#(#lifetimes),*>
                            #ty : #ReprC
                    );
                    // FIXME assert that the types involved are concrete.
                    // let ty_no_lt = {
                    //     let mut it = ty.clone();
                    //     StripLifetimeParams.visit_type_mut(&mut it);
                    //     it
                    // };
                    // let its_CType_is_concrete = parse_quote!(
                    //     // for<#(#lifetimes),*> < #ty
                    //     <#ty_no_lt as #ReprC>::CLayout
                    //     :
                    //     #LegacyCType<OPAQUE_KIND = #OpaqueKind::Concrete>
                    // );
                    // Iterator::chain(
                        ::core::iter::once(is_ReprC)
                    //     , ::core::iter::once(its_CType_is_concrete)
                    // )
                })
                .collect()
        ;
        input_Layout
            .generics
            .make_where_clause()
            .predicates
            .extend(repr_c_clauses.iter().cloned())
        ;
        let input_Layout_data = match input_Layout.data {
            | Data::Struct(ref mut it) => it,
            | _ => unreachable!(),
        };
        *input_Layout_data.fields.iter_mut().next().unwrap() = {
            ::syn::parse::Parser::parse2(
                Field::parse_unnamed,
                quote!(
                    pub
                    #core::option::Option<
                        unsafe
                        extern "C"
                        fn (#(#EachArgCType),*) -> #RetCType
                    >
                ),
            ).unwrap()
        };
        // Add a PhantomData field to account for unused lifetime params.
        // (given that we've had to strip them to become `LegacyCType`)
        if generics.lifetimes().next().is_some() { // non-empty.
            let fields_mut = match input_Layout_data.fields {
                | Fields::Unnamed(FieldsUnnamed {
                    unnamed: ref mut it,
                    ..
                }) => it,
                | _ => unreachable!(),
            };
            fields_mut.extend(Some({
                let phantom_tys = // Iterator::chain(
                    generics
                        .lifetimes()
                        .map(|&LifetimeDef { lifetime: ref lt, .. }| -> Type {
                            parse_quote!( *mut (&#lt ()) )
                        })
                //     ,
                //     generics
                //         .type_params
                //         .map(|&TypeParam { ident: ref T, .. }| -> Type {
                //             parse_quote!( *mut #T )
                //         })
                //     ,
                // )
                ;
                ::syn::parse::Parser::parse2(
                    Field::parse_unnamed,
                    quote!(
                        /// Conservatively invariant.
                        #[allow(unused_parens)]
                        #core::marker::PhantomData<(#(#phantom_tys),*)>
                    ),
                ).unwrap()
            }));
        }
        let ref mut c_sharp_format_args =
            EachArgCType
                .iter()
                .map(|_| "\n        {}{},")
                .collect::<String>()
        ;
        let _trailing_comma = c_sharp_format_args.pop();
        let (intro, fwd, where_) = input_Layout.generics.split_for_impl();
        ret.extend(quote!(
            unsafe
            impl #intro
                #ReprC
            for
                #StructName #fwd
            #where_
            {
                type CLayout = #StructName_Layout #fwd;

                fn is_valid (it: &'_ Self::CLayout)
                  -> #bool
                {
                    it.0.is_some()
                }
            }

            #[allow(nonstandard_style)]
            #input_Layout

            impl #intro
                #core::clone::Clone
            for
                #StructName_Layout #fwd
            #where_
            {
                fn clone (self: &'_ Self)
                  -> Self
                {
                    impl #intro
                        #core::marker::Copy
                    for
                        #StructName_Layout #fwd
                    {}

                    *self
                }
            }

            unsafe
            impl #intro
                #LegacyCType
            for
                #StructName_Layout #fwd
            #where_
            { #__cfg_headers__! {
                fn c_short_name_fmt (fmt: &'_ mut #fmt::Formatter<'_>)
                  -> #fmt::Result
                {
                    // fmt.write_str(stringify!(#StructName))
                    // ret_arg1_arg2_fptr
                    fmt.write_str(&<#RetCType as #CType>::short_name())?; #(
                    #write!(fmt, "_{}", <#EachArgCType as #CType>::short_name())?; )*
                    fmt.write_str("_fptr")
                }

                fn c_define_self (definer: &'_ mut dyn #Definer)
                  -> #std::io::Result<()>
                {
                    <#RetCType as #CType>::define_self(&::safer_ffi::headers::languages::C, definer)?; #(
                    <#EachArgCType as #CType>::define_self(&::safer_ffi::headers::languages::C, definer)?; )*
                    Ok(())
                }

                fn c_var_fmt (
                    fmt: &'_ mut #fmt::Formatter<'_>,
                    var_name: &'_ #str,
                ) -> #fmt::Result
                {
                    #write!(fmt, "{} ", <#RetCType as #CType>::name_wrapping_var(&::safer_ffi::headers::languages::C, ""))?;
                    #write!(fmt, "(*{})(", var_name)?;
                    let _empty = true;
                    #(
                        #write!(fmt,
                            "{comma}{arg}",
                            arg = <#EachArgCType as #CType>::name_wrapping_var(&::safer_ffi::headers::languages::C, ""),
                            comma = if _empty { "" } else { ", " },
                        )?;
                        let _empty = false;
                    )*
                    if _empty {
                        fmt.write_str("void")?;
                    }
                    fmt.write_str(")")
                }

                #__cfg_csharp__! {
                    fn csharp_define_self (definer: &'_ mut dyn #Definer)
                      -> #std::io::Result<()>
                    {
                        <#RetCType as #CType>::define_self(&::safer_ffi::headers::languages::CSharp, definer)?; #(
                        <#EachArgCType as #CType>::define_self(&::safer_ffi::headers::languages::CSharp, definer)?; )*
                        let ref me = <Self as #CType>::name(&::safer_ffi::headers::languages::CSharp).to_string();
                        let ref mut _forge_arg_name = {
                            let mut iter = (0 ..).map(|c| #std::format!("_{}", c));
                            move || iter.next().unwrap()
                        };
                        definer.define_once(me, &mut |definer| #core::writeln!(definer.out(),
                            concat!(
                                // IIUC,
                                //   - For 32-bits / x86,
                                //     Rust's extern "C" is the same as C#'s (default) Winapi:
                                //     "cdecl" for Linux, and "stdcall" for Windows.
                                //
                                //   - For everything else, this is param is ignored.
                                //     I guess because both OSes agree on the calling convention?
                                "[UnmanagedFunctionPointer(CallingConvention.Winapi)]\n",

                                "{ret_marshaler}public unsafe /* static */ delegate\n",
                                "    {Ret}\n",
                                "    {me} (",
                                    #c_sharp_format_args,
                                ");\n",
                            ),
                            #(
                                <#EachArgCType as #CType>::csharp_marshaler()
                                    .map(|m| format!("[MarshalAs({})]\n        ", m))
                                    .as_deref()
                                    .unwrap_or("")
                                ,
                                <#EachArgCType as #CType>::name_wrapping_var(&::safer_ffi::headers::languages::CSharp, &_forge_arg_name()),
                            )*
                            me = me,
                            ret_marshaler =
                                <#RetCType as #CType>::csharp_marshaler()
                                    .map(|m| format!("[return: MarshalAs({})]\n", m))
                                    .as_deref()
                                    .unwrap_or("")
                            ,
                            Ret = <#RetCType as #CType>::name(&::safer_ffi::headers::languages::CSharp),
                        ))
                    }

                    fn csharp_ty ()
                      -> #std::string::String
                    {
                        Self::c_short_name().to_string()
                    }

                    fn legacy_csharp_marshaler ()
                      -> #Option<#std::string::String>
                    {
                        // This assumes the calling convention from the above
                        // `UnmanagedFunctionPointer` attribute.
                        #Option::Some("UnmanagedType.FunctionPtr".into())
                    }
                }
            } type OPAQUE_KIND = #OpaqueKind::Concrete; }

            unsafe
            impl #intro
                #__HasNiche__
            for
                #StructName #fwd
            #where_
            {
                #[inline]
                fn is_niche (it: &'_ <Self as #ReprC>::CLayout)
                  -> bool
                {
                    it.0.is_none()
                }
            }
        ));
        let ret = TokenStream::from(ret);
        // pretty_print_tokenstream(&ret, "");
        Some(ret.into())
    } else {
        None
    }
}

/// Convert, for instance, `fn(&i32)` into `fn(&'__elided_0 i32)`, yielding
/// `'__elided_0`
struct UnelideLifetimes<'__, 'vec> {
    lifetime_params: &'__ mut ::std::borrow::Cow<'vec, Vec<LifetimeDef>>,
    counter: ::core::ops::RangeFrom<usize>,
}

const _: () = {
    macro_rules! ELIDED_LIFETIME_TEMPLATE {() => (
        "__elided_{}"
    )}
    impl ::syn::visit_mut::VisitMut for UnelideLifetimes<'_, '_> {
        fn visit_lifetime_mut (
            self: &'_ mut Self,
            lifetime: &'_ mut Lifetime,
        )
        {
            let Self { lifetime_params, counter } = self;
            if lifetime.ident == "_" {
                lifetime.ident = format_ident!(
                    ELIDED_LIFETIME_TEMPLATE!(),
                    counter.next().unwrap(),
                );
                lifetime_params.to_mut().push(parse_quote!( #lifetime ));
            }
        }

        fn visit_type_mut (
            self: &'_ mut Self,
            ty: &'_ mut Type,
        )
        {
            ::syn::visit_mut::visit_type_mut(self, ty);
            let Self { lifetime_params, counter } = self;
            match *ty {
                | Type::Reference(TypeReference {
                    lifetime: ref mut implicitly_elided_lifetime @ None,
                    ..
                }) => {
                    let unelided_lifetime =
                        implicitly_elided_lifetime
                            .get_or_insert(Lifetime::new(
                                &format!(
                                    concat!(
                                        "'",
                                        ELIDED_LIFETIME_TEMPLATE!(),
                                    ),
                                    counter.next().unwrap(),
                                ),
                                Span::call_site(),
                            ))
                    ;
                    lifetime_params.to_mut().push(parse_quote!(
                        #unelided_lifetime
                    ));
                },
                | _ => {},
            }
        }
    }
};

/// Pretty self-explanatory: since we can't do `<for<'lt> Ty as Tr>::Assoc`
/// (technically the result value could depend on `'lt`, but not in our case,
/// since `LegacyCType` is `'static`)
struct StripLifetimeParams;

impl VisitMut for StripLifetimeParams {
    fn visit_lifetime_mut (
        self: &'_ mut Self,
        lifetime: &'_ mut Lifetime,
    )
    {
        *lifetime = Lifetime::new("'static", Span::call_site());
    }

    fn visit_type_mut (
        self: &'_ mut Self,
        ty: &'_ mut Type,
    )
    {
        ::syn::visit_mut::visit_type_mut(self, ty);
        match *ty {
            | Type::Reference(TypeReference {
                lifetime: ref mut implicitly_elided_lifetime @ None,
                ..
            }) => {
                *implicitly_elided_lifetime = Some(
                    Lifetime::new("'static", Span::call_site())
                );
            },
            | _ => {},
        }
    }
}

macro_rules! quote_use {
    (
        $(
            use
                $(:: $(@$leading:tt)?)?
                $($path:ident)::+
            ;
        )*
    ) => (
        $(
            quote_use! {
                @single = $($path)+,
                $(:: $($leading)?)?
                $($path)::+
            }
        )*
    );

    (
        @single = $not_last:ident $($others:ident)+,
        $($rest:tt)*
    ) => (
        quote_use! {
            @single = $($others)+,
            $($rest)*
        }
    );

    (
        @single = $last:ident,
        $(:: $(@$leading:tt)?)?
        $($path:ident)::+
    ) => (
        let $last = quote!( $(:: $($leading)?)? $($path)::+ );
    );
} use quote_use;
