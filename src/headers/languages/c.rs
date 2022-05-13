use super::*;

pub
struct C;

impl HeaderLanguage for C {
    fn emit_docs (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        indent: &'_ Indentation,
    ) -> io::Result<()>
    {
        mk_out!(indent, "{indent}", ctx.out());

        if docs.is_empty() {
            out!(("/** <No documentation available> */"));
            return Ok(());
        }

        out!(("/** \\brief"));
        if let _ = indent.scope() {
            for line in docs {
                out!((" * {line}\n"));
            }
        }
        out!(("*/"));

        Ok(())
    }

    fn emit_simple_enum (
        self: &'_ C,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        enum_name: &'_ str,
        size: Option<(bool, u8)>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, "{indent}", ctx.out());

        let ref intn_t =
            size.map(|(signed, bitwidth)| format!(
                "{}int{bitwidth}_t", if signed { "" } else { "u" },
            ))
        ;

        self.emit_docs(ctx, docs, indent)?;

        if let Some(intn_t) = intn_t {
            out!((
                "/** \\remark Has the same ABI as `{intn_t}` **/"
                "#ifdef DOXYGEN"
                "typedef enum {enum_name}"
                "#else"
                "typedef {intn_t} {enum_name}_t; enum"
                "#endif"
                "{{"
            ));
        } else {
            out!(("typedef enum {enum_name} {{"));
        }

        if let _ = indent.scope() {
            for v in variants {
                self.emit_docs(ctx, v.docs, indent)?;
                let variant_name = crate::utils::screaming_case(enum_name, v.name) /* ctx.adjust_variant_name(
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

        if intn_t.is_some() {
            out!((
                "}}"
                "#ifdef DOXYGEN"
                "{enum_name}_t"
                "#endif"
                ";"
            ));
        } else {
            out!(("}} {enum_name}_t;"));
        }
        out!("\n");

        Ok(())
    }

    fn emit_struct (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        name: &'_ str,
        size: usize,
        fields: &'_ [StructField<'_>]
    ) -> io::Result<()>
    {
        todo!()
    }

    fn emit_function (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        fname: &'_ str,
        arg_names: &'_ [FunctionArg<'_>],
        ret_ty: &'_ str,
    ) -> io::Result<()>
    {
        todo!()
    }
}
