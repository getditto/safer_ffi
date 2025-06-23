use super::*;

pub struct C;

impl HeaderLanguage for C {
    fn emit_docs(
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        indent: &'_ Indentation,
    ) -> io::Result<()> {
        mk_out!(indent, ctx.out());

        if docs.is_empty() {
            out!(("/** <No documentation available> */"));
            return Ok(());
        }

        out!(("/** \\brief"));
        for line in docs.iter().copied().map(str::trim) {
            let sep = if line.is_empty() { "" } else { "  " };
            out!((" *{sep}{line}"));
        }
        out!((" */"));

        Ok(())
    }

    fn supports_type_aliases(self: &'_ C) -> Option<&'_ dyn HeaderLanguageSupportingTypeAliases> {
        return Some(self);
        // where
        #[expect(non_local_definitions)]
        impl HeaderLanguageSupportingTypeAliases for C {
            fn declare_type_alias(
                self: &'_ Self,
                ctx: &'_ mut dyn Definer,
                docs: Docs<'_>,
                self_ty: &'_ dyn PhantomCType,
                inner_ty: &'_ dyn PhantomCType,
            ) -> io::Result<()> {
                // No `this` in this design yet; let's stick to `this` nonetheless
                // for the syntactical search for the `self` antipattern.
                let this = self;
                let ref indent = Indentation::new(4 /* ctx.indent_width() */);
                mk_out!(indent, ctx.out());
                this.emit_docs(ctx, docs, indent)?;
                let ref aliaser = self_ty.name(this);
                let ref aliasee = inner_ty.name(this);
                out!((
                    "typedef {aliasee} {aliaser};"
                ));

                out!("\n");
                Ok(())
            }
        }
    }

    fn declare_simple_enum(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        backing_integer: Option<&dyn PhantomCType>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        let ref intn_t = backing_integer.map(|it| it.name(this));

        this.emit_docs(ctx, docs, indent)?;

        let ref short_name = self_ty.short_name();
        let ref full_ty_name = self_ty.name(this);

        if let Some(intn_t) = intn_t {
            out!((
                "/** \\remark Has the same ABI as `{intn_t}` **/"
                "#ifdef DOXYGEN"
                "typedef"
                "#endif"
                "enum {short_name} {{"
            ));
        } else {
            out!(("typedef enum {short_name} {{"));
        }

        if let _ = indent.scope() {
            for v in variants {
                this.emit_docs(ctx, v.docs, indent)?;
                let variant_name = crate::utils::screaming_case(short_name, v.name) /* ctx.adjust_variant_name(
                    Language::C,
                    enum_name,
                    v.name,
                ) */;
                if let Some(value) = v.discriminant {
                    out!(("{variant_name} = {value:?},"));
                } else {
                    out!(("{variant_name},"));
                }
            }
        }

        if let Some(intn_t) = intn_t {
            out!((
                "}}"
                "#ifndef DOXYGEN"
                "; typedef {intn_t}"
                "#endif"
                "{full_ty_name};"
            ));
        } else {
            out!(("}} {full_ty_name};"));
        }

        out!("\n");
        Ok(())
    }

    fn declare_struct(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        fields: &'_ [StructField<'_>],
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());
        let short_name = self_ty.short_name();
        let full_ty_name = self_ty.name(this);

        if self_ty.size() == 0 {
            panic!("C does not support zero-sized structs!")
        }

        this.emit_docs(ctx, docs, indent)?;
        out!(("typedef struct {short_name} {{"));
        if let _ = indent.scope() {
            let ref mut first = true;
            for &StructField { docs, name, ty } in fields {
                // Skip ZSTs
                if ty.size() == 0 {
                    if ty.align() > 1 {
                        panic!("Zero-sized fields must have an alignment of `1`");
                    } else {
                        continue;
                    }
                }
                if mem::take(first).not() {
                    out!("\n");
                }
                this.emit_docs(ctx, docs, indent)?;
                out!(
                    ("{};"),
                    ty.name_wrapping_var(this, Some(&name))
                );
            }
        }
        out!(("}} {full_ty_name};"));

        out!("\n");
        Ok(())
    }

    fn declare_opaque_type(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());
        let short_name = self_ty.short_name();
        let full_ty_name = self_ty.name(this);

        this.emit_docs(ctx, docs, indent)?;
        out!(("typedef struct {short_name} {full_ty_name};"));

        out!("\n");
        Ok(())
    }

    fn declare_function(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        fname: &'_ str,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);

        this.emit_docs(ctx, docs, indent)?;

        let ref fn_sig_but_for_ret_type: String = {
            let mut buf = Vec::<u8>::new();
            mk_out!(indent, buf);

            out!(
                "\n{indent}{fn}{fname} (",
                fn = if cfg!(feature = "c-headers-with-fn-style") {
                    "/* fn */ "
                } else {
                    ""
                },
            );
            let mut first = true;
            if let _ = indent.scope() {
                for arg in args {
                    if mem::take(&mut first).not() {
                        out!(",");
                    }
                    out!("\n{indent}{}", arg.ty.name_wrapping_var(this, Some(&arg.name)))
                }
                if first {
                    out!("void");
                }
            }
            out!(")");
            String::from_utf8(buf).unwrap()
        };

        mk_out!(indent, ctx.out());
        out!(
            ("{};"), ret_ty.name_wrapping_var(this, Some(&fn_sig_but_for_ret_type))
        );

        out!("\n");
        Ok(())
    }

    fn declare_constant(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        name: &'_ str,
        ty: &'_ dyn PhantomCType,
        skip_type: bool,
        value: &'_ dyn ::core::fmt::Debug,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        this.emit_docs(ctx, docs, indent)?;
        if skip_type {
            out!((
                "#define {name} {value:?}"
            ));
        } else {
            let ty = ty.name(this);
            out!((
                "#define {name} (({ty}) {value:?})"
            ));
        }

        out!("\n");
        Ok(())
    }

    fn emit_function_ptr_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        out: &'_ mut dyn io::Write,
        _newtype_name: &'_ str,
        name: Option<&dyn ::core::fmt::Display>,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        write!(
            out,
            "{ret_ty} (*{name})({args})",
            ret_ty = F(|out| ret_ty.render(out, this)),
            args = F(|out| {
                if args.is_empty() {
                    return write!(out, "void");
                }
                let first = &mut true;
                for arg in args {
                    if mem::take(first).not() {
                        write!(out, ", ")?;
                    }
                    arg.ty.render(out, this)?;
                }
                Ok(())
            }),
            name = name.or_empty(),
        )
    }

    fn emit_primitive_ty(
        self: &'_ Self,
        out: &mut dyn io::Write,
        primitive: Primitive,
    ) -> io::Result<()> {
        match primitive {
            | Primitive::Bool => {
                write!(out, "bool")?;
            },
            | Primitive::CChar => {
                write!(out, "char")?;
            },
            | Primitive::Integer { signed, bitwidth } => match bitwidth {
                | IntBitWidth::PointerSized => {
                    let sign_prefix = if signed { "s" } else { "" };
                    write!(out, "{sign_prefix}size_t")?;
                },
                | IntBitWidth::Fixed(num_bits) => {
                    let sign_prefix = if signed { "" } else { "u" };
                    let num_bits = num_bits as u8;
                    write!(out, "{sign_prefix}int{num_bits}_t")?;
                },
                | IntBitWidth::CInt => {
                    let sign_prefix = if signed { "" } else { "u" };
                    write!(out, "{sign_prefix}int")?;
                },
            },
            | Primitive::Float { bitwidth } => match bitwidth {
                | FloatBitWidth::_32 => write!(out, "float")?,
                | FloatBitWidth::_64 => write!(out, "double")?,
            },
        }
        Ok(())
    }

    fn emit_pointer_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        out: &mut dyn io::Write,
        pointee_is_immutable: bool,
        pointee: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        let maybe_const = if pointee_is_immutable { "const " } else { "" };
        write!(
            out,
            "{pointee} {maybe_const}*",
            pointee = F(|out| pointee.render(out, this)),
        )
    }

    fn define_array_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        definer: &'_ mut dyn Definer,
        self_ty: &'_ dyn PhantomCType,
        elem_ty: &'_ dyn PhantomCType,
        array_len: usize,
    ) -> io::Result<()> {
        let me = &F(|out| self_ty.render(out, this)).to_string();
        write!(
            definer.out(),
            concat!(
                "typedef struct {{\n",
                "    {inline_array};\n",
                "}} {me};\n",
                "\n",
            ),
            inline_array = F(
                |out| {
                    elem_ty.render_wrapping_var(
                        out,
                        this,
                        Some(&format_args!("idx[{}]", array_len)),
                    )
                }
            ),
            me = me,
        )?;
        Ok(())
    }

    fn emit_array_ty(
        self: &'_ Self,
        _this: &dyn HeaderLanguage,
        out: &mut dyn io::Write,
        var_name: Option<&dyn ::core::fmt::Display>,
        newtype_name: &'_ str,
        _elem_ty: &'_ dyn PhantomCType,
        _array_len: usize,
    ) -> io::Result<()> {
        write!(out, "{newtype_name}{sep}{var_name}", sep = var_name.sep(), var_name = var_name.or_empty())
    }

    fn define_primitive_ty(
        self: &'_ Self,
        _this: &dyn HeaderLanguage,
        definer: &'_ mut dyn Definer,
        primitive: Primitive,
    ) -> io::Result<()> {
        match primitive {
            | Primitive::Integer {
                signed: _,
                bitwidth,
            } => match bitwidth {
                | primitives::IntBitWidth::CInt => {},
                | _ => {
                    definer.define_once("__int_headers__", &mut |definer| {
                        write!(definer.out(), concat! {
                            "\n",
                            "#include <stddef.h>\n",
                            "#include <stdint.h>\n",
                            "\n",
                        },)
                    })?;
                },
            },
            | Primitive::Bool => {
                definer.define_once("bool", &mut |definer| {
                    write!(definer.out(), concat! {
                        "\n",
                        "#include <stdbool.h>\n",
                        "\n",
                    },)
                })?;
            },
            | _ => {},
        }
        Ok(())
    }

    fn emit_void_output_type(
        self: &'_ Self,
        out: &mut dyn io::Write,
    ) -> io::Result<()> {
        write!(out, "void")?;
        Ok(())
    }
}
