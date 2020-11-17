struct StripLifetimeParams;
impl ::syn::visit_mut::VisitMut for StripLifetimeParams {
    fn visit_lifetime_mut (self: &'_ mut Self, lifetime: &'_ mut Lifetime)
    {
        *lifetime = Lifetime::new("'static", Span2::call_site());
    }

    fn visit_type_mut (self: &'_ mut Self, ty: &'_ mut Type)
    {
        ::syn::visit_mut::visit_type_mut(self, ty);
        match *ty {
            | Type::Reference(TypeReference { ref mut lifetime, .. }) => {
                *lifetime = Some(Lifetime::new("'static", Span2::call_site()));
            },
            | _ => {},
        }
    }
}

fn try_handle_fptr (input: &'_ DeriveInput)
  -> Option<TokenStream>
{
    let span = Span2::call_site();

    macro_rules! error {(
        $msg:expr $(=> $span:expr)? $(,)?
    ) => ({
        $(
            let span = $span;
        )?
        return Some(Error::new(span, &*$msg).to_compile_error().into());
    })}
    macro_rules! bail {() => ({
        dbg!();
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
        let cb_ty = match fields.iter().next().unwrap().ty {
            | Type::BareFn(ref it) => it,
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

        let mut ret = input.to_token_stream();

        let ReprC = quote!( ::safer_ffi::layout::ReprC );
        let CType = quote!( ::safer_ffi::layout::CType );
        let Definer = quote!( ::safer_ffi::headers::Definer );
        let Concrete = quote!( ::safer_ffi::layout::OpaqueKind::Concrete );
        let __cfg_headers__ = quote!( ::safer_ffi::__cfg_headers__ );
        let __cfg_csharp__ = quote!( ::safer_ffi::__cfg_csharp__ );
        let bool = quote!( ::safer_ffi::bool );
        let core = quote!( ::safer_ffi::core );
        let std = quote!( ::safer_ffi::std );

        let fmt = quote!( #core::fmt );
        let write = quote!( #core::write );

        let EachArg = cb_ty.inputs.iter().map(|arg| {
            let mut ty = arg.ty.clone();
            StripLifetimeParams.visit_type_mut(&mut ty);
            ty
        }).collect::<Vec<_>>();
        let ref Ret = match cb_ty.output {
            | ReturnType::Default => parse_quote!( () ),
            | ReturnType::Type(_, ref ty) => {
                let mut ty = (&**ty).clone();
                StripLifetimeParams.visit_type_mut(&mut ty);
                ty
            },
        };

        let mut layout_def = input.clone();
        let lifetimes =
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
                    let is_ReprC: WherePredicate = parse_quote!(
                        for<#(#lifetimes),*>
                            #ty : #ReprC
                    );
                    let ty_no_lt = {
                        let mut it = ty.clone();
                        StripLifetimeParams.visit_type_mut(&mut it);
                        it
                    };
                    // let its_CType_is_concrete = parse_quote!(
                    //     // for<#(#lifetimes),*> < #ty
                    //     <#ty_no_lt as #ReprC>::CLayout
                    //     :
                    //     #CType<OPAQUE_KIND = #Concrete>
                    // );
                    // Iterator::chain(
                        ::core::iter::once(is_ReprC)
                    //     , ::core::iter::once(its_CType_is_concrete)
                    // )
                })
                .collect()
        ;
        layout_def
            .generics
            .make_where_clause()
            .predicates
            .extend(repr_c_clauses.iter().cloned())
        ;
        layout_def.ident = format_ident!(
            "{}_Layout", StructName,
        );
        let ref StructName_Layout = layout_def.ident;
        let layout_def_data = match layout_def.data {
            | Data::Struct(ref mut it) => it,
            | _ => unreachable!(),
        };
        *layout_def_data.fields.iter_mut().next().unwrap() = {
            ::syn::parse::Parser::parse2(
                Field::parse_unnamed,
                quote!(
                    pub
                    #core::option::Option<
                        unsafe
                        extern "C"
                        fn (#(<#EachArg as #ReprC>::CLayout),*)
                          -> <#Ret as #ReprC>::CLayout
                    >
                ),
            ).unwrap()
        };
        /// Add a PhantomData field to account for unused lifetime params.
        if generics.lifetimes().count() > 0 {
            let fields_mut = match layout_def_data.fields {
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
            EachArg
                .iter()
                .map(|_| "\n        {}{},")
                .collect::<String>()
        ;
        let _ = c_sharp_format_args.pop();
        let (intro, fwd, where_) = layout_def.generics.split_for_impl();
        ret.extend(quote!(
            unsafe
            impl#intro
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
            #layout_def

            impl#intro
                #core::clone::Clone
            for
                #StructName_Layout #fwd
            #where_
            {
                fn clone (self: &'_ Self)
                  -> Self
                {
                    impl#intro
                        #core::marker::Copy
                    for
                        #StructName_Layout #fwd
                    {}

                    *self
                }
            }

            unsafe
            impl#intro
                #CType
            for
                #StructName_Layout #fwd
            #where_
            { #__cfg_headers__! {
                fn c_short_name_fmt (fmt: &'_ mut #fmt::Formatter<'_>)
                   -> #fmt::Result
                {
                    // fmt.write_str(stringify!(#StructName))
                    // ret_arg1_arg2_fptr
                    <<#Ret as #ReprC>::CLayout as #CType>::c_short_name_fmt(fmt)?; #(
                    #write!(fmt, "_{}", <<#EachArg as #ReprC>::CLayout as #CType>::c_short_name())?; )*
                    fmt.write_str("_fptr")
                }

                fn c_define_self (definer: &'_ mut dyn #Definer)
                  -> #std::io::Result<()>
                {
                    <<#Ret as #ReprC>::CLayout as #CType>::c_define_self(definer)?; #(
                    <<#EachArg as #ReprC>::CLayout as #CType>::c_define_self(definer)?; )*
                    Ok(())
                }

                fn c_var_fmt (
                    fmt: &'_ mut #fmt::Formatter<'_>,
                    var_name: &'_ str,
                ) -> #fmt::Result
                {
                    #write!(fmt, "{} ", <<#Ret as #ReprC>::CLayout as #CType>::c_var(""))?;
                    #write!(fmt, "(*{})(", var_name)?;
                    let _empty = true;
                    #(
                        #write!(fmt,
                            "{comma}{arg}",
                            arg = <<#EachArg as #ReprC>::CLayout as #CType>::c_var(""),
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
                        <<#Ret as #ReprC>::CLayout as #CType>::csharp_define_self(definer)?; #(
                        <<#EachArg as #ReprC>::CLayout as #CType>::csharp_define_self(definer)?; )*
                        let ref me = Self::csharp_ty();
                        let ref mut _arg = {
                            let mut iter = (0 ..).map(|c| format!("_{}", c));
                            move || iter.next().unwrap()
                        };
                        definer.define_once(me, &mut |definer| writeln!(definer.out(),
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
                                <<#EachArg as #ReprC>::CLayout as #CType>::csharp_marshaler()
                                    .map(|m| format!("[MarshalAs({})]\n        ", m))
                                    .as_deref()
                                    .unwrap_or("")
                                ,
                                <<#EachArg as #ReprC>::CLayout as #CType>::csharp_var(&_arg()),
                            )*
                            me = me,
                            ret_marshaler =
                                <<#Ret as #ReprC>::CLayout as #CType>::csharp_marshaler()
                                    .map(|m| format!("[return: MarshalAs({})]\n", m))
                                    .as_deref()
                                    .unwrap_or("")
                            ,
                            Ret = <<#Ret as #ReprC>::CLayout as #CType>::csharp_ty(),
                        ))
                    }

                    fn csharp_ty ()
                      -> #std::string::String
                    {
                        Self::c_short_name().to_string()
                    }

                    fn csharp_marshaler ()
                      -> Option<#std::string::String>
                    {
                        // This assumes the calling convention from the above
                        // `UnmanagedFunctionPointer` attribute.
                        Some("UnmanagedType.FunctionPtr".into())
                    }
                }
            } type OPAQUE_KIND = #Concrete; }
        ));

        let (intro_, fwd, where_) = input.generics.split_for_impl();
        let inputs = &cb_ty.inputs;
        let output = &cb_ty.output;
        ret.extend(quote!(
            impl #intro
                #core::ops::Deref
            for
                #StructName #fwd
            #where_
            {
                type Target = #cb_ty;

                #[inline]
                fn deref (self: &'_ #StructName #fwd)
                  -> &'_ #cb_ty
                {
                    &self.0
                }
            }
        ));
        let ret = TokenStream::from(ret);
        pretty_print_tokenstream(&ret, "");
        Some(ret.into())
    } else {
        None
    }
}
