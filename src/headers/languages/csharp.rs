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
        mk_out!(indent, ctx.out());

        if docs.is_empty() {
            // out!(("/// <summary> No documentation available </summary>"));
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
        self_ty: &'_ dyn PhantomCType,
        backing_integer: Option<&dyn PhantomCType>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        let ref IntN =
            backing_integer.map(|it| it.name(self))
        ;

        let ref full_ty_name = self_ty.name(self);

        self.emit_docs(ctx, docs, indent)?;

        out!(
            ("public enum {full_ty_name} {super} {{"),
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
        self_ty: &'_ dyn PhantomCType,
        fields: &'_ [StructField<'_>]
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        let size = self_ty.size();
        if size == 0 {
            panic!("C# does not support zero-sized structs!")
        }

        let ref name = self_ty.name(self);

        self.emit_docs(ctx, docs, indent)?;
        out!((
            "[StructLayout(LayoutKind.Sequential, Size = {size})]"
            "public unsafe struct {name} {{"
        ));
        if let _ = indent.scope() {
            let ref mut first = true;
            for &StructField { docs, name, ty } in fields {
                if ty.size() == 0 && ty.align() > 1 {
                    panic!("Zero-sized fields must have an alignment of `1`");
                }
                if mem::take(first).not() {
                    out!("\n");
                }
                self.emit_docs(ctx, docs, indent)?;
                if let Some(csharp_marshaler) = ty.csharp_marshaler() {
                    out!((
                        "[MarshalAs({csharp_marshaler})]"
                    ));
                }
                out!(
                    ("public {} {name};"),
                    ty.name(self), // _wrapping_var(self, name)
                );
            }
        }
        out!(("}}"));

        out!("\n");
        Ok(())
    }

    fn emit_function (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        fname: &'_ str,
        arg_names: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()>
    {
        todo!()
    }
}
