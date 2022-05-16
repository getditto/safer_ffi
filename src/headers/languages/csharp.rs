use super::*;

pub
struct CSharp;

impl HeaderLanguage for CSharp {
    fn emit_docs (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        indent: &'_ Indentation,
    ) -> io::Result<()>
    {
        mk_out!(indent, "{indent}", ctx.out());

        if docs.is_empty() {
            // out!(("/// <summary> No documentation available> </summary>"));
            return Ok(());
        }

        out!(("/// <summary>"));
        for mut line in docs.iter().copied().map(str::trim) {
            let mut storage = None;
            if line.contains('`') {
                let s = storage.get_or_insert_with(rust::String::new);
                let mut parity = 0..;
                line.chars().for_each(|c| match c {
                    | '`' => s.push_str(["<c>", "</c>"][parity.next().unwrap() % 2]),
                    | _ => s.push(c),
                });
                line = s;
            }
            let sep = if line.is_empty() { "" } else { " " };
            out!(("///{sep}{line}"));
        }
        out!(("/// </summary>"));

        Ok(())
    }

    fn emit_simple_enum (
        self: &'_ CSharp,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        enum_name: &'_ str,
        size: Option<(bool, u8)>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, "{indent}", ctx.out());

        let ref IntN =
            size.map(|(signed, bitwidth)| if bitwidth != 8 {
                format!(
                    "{}Int{bitwidth}", if signed { "" } else { "U" },
                )
            } else {
                format!(
                    "{}byte", if signed { "s" } else { "" },
                )
            })
        ;

        self.emit_docs(ctx, docs, indent)?;

        out!(
            ("public enum {enum_name}_t {super} {{"),
            super = if let Some(IntN) = IntN {
                format!(": {IntN}")
            } else {
                "".into()
            },
        );

        if let _ = indent.scope() {
            for v in variants {
                self.emit_docs(ctx, v.docs, indent)?;
                let variant_name = v.name /* ctx.adjust_variant_name(
                    Language::CSharp,
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

        out!(("}}"));

        out!("\n");
        Ok(())
    }

    fn emit_struct (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
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
        docs: Docs<'_>,
        fname: &'_ str,
        arg_names: &'_ [FunctionArg<'_>],
        ret_ty: &'_ str,
    ) -> io::Result<()>
    {
        todo!()
    }
}
