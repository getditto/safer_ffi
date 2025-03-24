use super::*;

pub struct Lua;

impl HeaderLanguage for Lua {
    fn emit_docs (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        indent: &'_ Indentation,
    ) -> io::Result<()>
    {
        mk_out!(indent, ctx.out());

        if docs.is_empty() {
            out!(("// <No documentation available>"));
            return Ok(());
        }

        for line in docs.iter().copied().map(str::trim) {
            let sep = if line.is_empty() { "" } else { " " };
            out!(("//{sep}{line}"));
        }

        Ok(())
    }

    fn emit_simple_enum (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        backing_integer: Option<&dyn PhantomCType>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4);
        mk_out!(indent, ctx.out());

        let ref intn_t =
            backing_integer.map(|it| it.name(self))
            ;

        self.emit_docs(ctx, docs, indent)?;

        let ref short_name = self_ty.short_name();
        let ref full_ty_name = self_ty.name(self);

        if let Some(intn_t) = intn_t {
            out!((
                "// enum has the same ABI as `{intn_t}`"
                "typedef enum {short_name} {{"
            ));
        } else {
            out!(("typedef enum {short_name} {{"));
        }

        if let _ = indent.scope() {
            for v in variants {
                self.emit_docs(ctx, v.docs, indent)?;
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
            out!(("}}; typedef {intn_t} {full_ty_name};"));
        } else {
            out!(("}} {full_ty_name};"));
        }

        out!("\n");

        Ok(())
    }

    fn emit_struct (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        fields: &'_ [StructField<'_>]
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4);
        mk_out!(indent, ctx.out());
        let short_name = self_ty.short_name();
        let full_ty_name = self_ty.name(self);

        if self_ty.size() == 0 {
            panic!("C does not support zero-sized structs!")
        }

        self.emit_docs(ctx, docs, indent)?;
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
                self.emit_docs(ctx, docs, indent)?;
                out!(
                    ("{};"),
                    ty.name_wrapping_var(self, name)
                );
            }
        }
        out!(("}} {full_ty_name};"));

        out!("\n");
        Ok(())
    }

    fn emit_opaque_type (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4);
        mk_out!(indent, ctx.out());
        let short_name = self_ty.short_name();
        let full_ty_name = self_ty.name(self);

        self.emit_docs(ctx, docs, indent)?;
        out!(("typedef struct {short_name} {full_ty_name};"));

        out!("\n");
        Ok(())
    }

    fn emit_function (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        fname: &'_ str,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4);

        self.emit_docs(ctx, docs, indent)?;

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
                    out!("\n{indent}{}", arg.ty.name_wrapping_var(self, arg.name))
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
            ("{};"), ret_ty.name_wrapping_var(self, fn_sig_but_for_ret_type)
        );

        out!("\n");
        Ok(())
    }

    fn emit_constant (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        name: &'_ str,
        ty: &'_ dyn PhantomCType,
        _skip_type: bool,
        value: &'_ dyn ::core::fmt::Debug,
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4);
        mk_out!(indent, ctx.out());

        self.emit_docs(ctx, docs, indent)?;
        let ty = ty.name(self);
        match ty.as_str() {
            "int32_t" | "uint32_t" | "int16_t" | "uint16_t" | "int8_t" | "uint8_t" => {
                out!(("static const {ty} {name} = {value:?};"));
            },
            "Opaque__str_t" => {
                out!(("extern const char* {name};"));
            },
            // Based on https://luajit.org/ext_ffi_semantics.html
            // "static const declarations only work for integer types up to 32 bits."
            _ => panic!("Lua doesn't support this const type: {}", ty),
        }

        out!("\n");
        Ok(())
    }
}
