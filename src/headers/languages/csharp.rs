use super::*;

pub struct CSharp;

impl HeaderLanguage for CSharp {
    fn emit_docs(
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        indent: &'_ Indentation,
    ) -> io::Result<()> {
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
                let mut iter = line.chars().peekable();
                while let Some(c) = iter.next() {
                    match (c, iter.peek()) {
                        | ('`', Some('`')) => {
                            s.push(c);
                            s.push(iter.next().unwrap());
                            iter.next().map(|c| s.push(c));
                        },
                        | ('`', _) => {
                            s.push_str(["<c>", "</c>"][parity.next().unwrap() % 2]);
                        },
                        | _ => s.push(c),
                    }
                }
                line = s;
            }
            let sep = if line.is_empty() { "" } else { " " };
            out!(("///{sep}{line}"));
        }
        out!(("/// </summary>"));

        Ok(())
    }

    fn emit_simple_enum(
        self: &'_ CSharp,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        backing_integer: Option<&dyn PhantomCType>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        let ref IntN = backing_integer.map(|it| it.name(self));

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

    fn emit_struct(
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        fields: &'_ [StructField<'_>],
    ) -> io::Result<()> {
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

    fn emit_opaque_type(
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        let full_ty_name = self_ty.name(self);

        self.emit_docs(ctx, docs, indent)?;
        out!(("public struct {full_ty_name} {{"));
        if let _ = indent.scope() {
            out!((
                "#pragma warning disable 0169"
                "private byte OPAQUE;"
                "#pragma warning restore 0169"
            ))
        }
        out!(("}}"));

        out!("\n");
        Ok(())
    }

    fn emit_function(
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        fname: &'_ str,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        out!((
            "public unsafe partial class Ffi {{"
        ));

        if let _ = indent.scope() {
            self.emit_docs(ctx, docs, indent)?;

            if let Some(marshaler) = ret_ty.csharp_marshaler() {
                out!((
                    "[return: MarshalAs({marshaler})]"
                ));
            }

            out!((
                "[DllImport(RustLib, ExactSpelling = true)] public static unsafe extern"
            ));

            let ret_ty = ret_ty.name(self);
            out!("{}{ret_ty} {fname} (", indent);
            let mut first = true;
            if let _ = indent.scope() {
                for FunctionArg { name: arg_name, ty } in args {
                    if mem::take(&mut first).not() {
                        out!(",");
                    }
                    out!("\n");
                    if let Some(marshaler) = ty.csharp_marshaler() {
                        out!((
                            "[MarshalAs({marshaler})]"
                        ));
                    }
                    let arg_ty = ty.name(self);
                    out!("{}{arg_ty} {arg_name}", indent)
                }
            }
            out!(");\n");
        }
        out!(("}}"));

        out!("\n");
        Ok(())
    }

    fn emit_constant(
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        name: &'_ str,
        ty: &'_ dyn PhantomCType,
        skip_type: bool,
        value: &'_ dyn ::core::fmt::Debug,
    ) -> io::Result<()> {
        if skip_type {
            // Skip the whole const for now.
            // TODO: properly support constants in C#
            return Ok(());
        }
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        out!(("public unsafe partial class Ffi {{"));
        if let _ = indent.scope() {
            self.emit_docs(ctx, docs, indent)?;
            let ty = ty.name(self);
            out!((
                "public const {ty} {name} = {value:?};"
            ));
        }
        out!(("}}"));

        out!("\n");
        Ok(())
    }
}
