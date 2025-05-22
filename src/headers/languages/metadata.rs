#![cfg_attr(rustfmt, rustfmt::skip)]

use crate::layout;
use super::*;

pub struct Metadata;

impl HeaderLanguage for Metadata {

    fn emit_docs (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        indent: &'_ Indentation,
    ) -> io::Result<()> {
        mk_out!(indent, ctx.out());

        if docs.is_empty() {
            return Ok(());
        }

        out!(("\"comment\": {{"));

        if let _ = indent.scope() {
            out!(("\"lines\": ["));

            if let _ = indent.scope() {
                let docs_lines = docs.iter().copied()
                    .map(str::trim)
                    .map(|line| line.replace("\\", "\\\\"))
                    .map(|line| line.replace("\"", "\\\""));

                for (index, line) in docs_lines.enumerate() {
                    if index + 1 < docs.len() {
                        out!(("\"{line}\","));
                    } else {
                        out!(("\"{line}\""));
                    }
                }
            }

            out!(("]"));
        }

        out!(("}},"));

        Ok(())
    }

    fn emit_simple_enum (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        backing_integer: Option<&dyn PhantomCType>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4);
        mk_out!(indent, ctx.out());

        if let _ = indent.scope() {
            out!((",{{"));

            if let _ = indent.scope() {
                out!(("\"kind\": \"Enum\","));

                self.emit_docs(ctx, docs, indent)?;

                let ref short_name = self_ty.short_name();

                out!(("\"name\": \"{short_name}\","));

                out!(("\"cases\": ["));

                if let _ = indent.scope() {
                    for (index, variant) in variants.iter().enumerate() {
                        out!(("{{"));

                        if let _ = indent.scope() {
                            self.emit_docs(ctx, variant.docs, indent)?;

                            if let Some(discriminant) = variant.discriminant {
                                let formatted_discriminant = format!("{:?}", discriminant);

                                out!(("\"value\": \"{formatted_discriminant}\","));
                            }

                            let variant_short_name = variant.name;
                            let variant_name = crate::utils::screaming_case(short_name, variant.name);

                            out!(("\"rustName\": \"{variant_short_name}\","));
                            out!(("\"cName\": \"{variant_name}\""));
                        }

                        if index + 1 < variants.len() {
                            out!(("}},"));
                        } else {
                            out!(("}}"));
                        }
                    }
                }

                if let Some(backing_integer) = backing_integer {
                    out!(("],"));
                    self.emit_type_usage(ctx, indent, "backingType", backing_integer)?;
                } else {
                    out!(("]"));
                }
            }

            out!(("}}"));
        }

        Ok(())
    }

    fn emit_struct (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        fields: &'_ [StructField<'_>]
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4);
        mk_out!(indent, ctx.out());

        if let _ = indent.scope() {
            out!((",{{"));

            if let _ = indent.scope() {
                out!(("\"kind\": \"Struct\","));

                self.emit_docs(ctx, docs, indent)?;

                let ref short_name = self_ty.short_name();

                out!(("\"name\": \"{short_name}\","));

                out!(("\"fields\": ["));

                if let _ = indent.scope() {
                    let non_empty_fields: Vec<&StructField<'_>> = fields.iter()
                        .filter(|f| f.ty.size() != 0)
                        .collect();

                    for (index, field) in non_empty_fields.iter().enumerate() {
                        out!(("{{"));

                        if let _ = indent.scope() {
                            self.emit_docs(ctx, field.docs, indent)?;

                            let field_name = field.name;

                            out!(("\"name\": \"{field_name}\","));

                            self.emit_type_usage(ctx, indent, "type", field.ty)?;
                        }

                        if index + 1 < non_empty_fields.len() {
                            out!(("}},"));
                        } else {
                            out!(("}}"));
                        }
                    }
                }

                out!(("]"));
            }

            out!(("}}"));
        }

        Ok(())
    }

    fn emit_opaque_type (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4);
        mk_out!(indent, ctx.out());

        if let _ = indent.scope() {
            out!((",{{"));

            if let _ = indent.scope() {
                out!(("\"kind\": \"Opaque\","));

                self.emit_docs(ctx, docs, indent)?;

                let short_name = self_ty.short_name();

                out!(("\"name\": \"{short_name}\""));
            }

            out!(("}}"));
        }

        Ok(())
    }

    fn emit_function (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        fname: &'_ str,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4);
        mk_out!(indent, ctx.out());

        if let _ = indent.scope() {
            out!((",{{"));

            if let _ = indent.scope() {
                out!(("\"kind\": \"Function\","));

                self.emit_docs(ctx, docs, indent)?;

                out!(("\"name\": \"{fname}\","));

                out!(("\"valueParameters\": ["));

                if let _ = indent.scope() {
                    for (index, arg) in args.iter().enumerate() {
                        out!(("{{"));

                        if let _ = indent.scope() {
                            let arg_name = arg.name;

                            out!(("\"name\": \"{arg_name}\","));

                            self.emit_type_usage(ctx, indent, "type", arg.ty)?;
                        }

                        if index + 1 < args.len() {
                            out!(("}},"));
                        } else {
                            out!(("}}"));
                        }
                    }
                }

                out!(("],"));

                self.emit_type_usage(ctx, indent, "returnType", ret_ty)?;
            }

            out!(("}}"));
        }

        Ok(())
    }

    fn emit_constant (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        name: &'_ str,
        ty: &'_ dyn PhantomCType,
        skip_type: bool,
        value: &'_ dyn fmt::Debug,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4);
        mk_out!(indent, ctx.out());

        if let _ = indent.scope() {
            out!((",{{"));

            if let _ = indent.scope() {
                out!(("\"kind\": \"Constant\","));

                self.emit_docs(ctx, docs, indent)?;

                out!(("\"name\": \"{name}\","));

                let constant_type = if skip_type {
                    let formatted_constant = format!("{:?}", value);

                    if formatted_constant.starts_with("\"") {
                        &PhantomData::<<char_p::Ref<'_> as layout::ReprC>::CLayout> as &dyn PhantomCType
                    } else if formatted_constant.starts_with("\'") {
                        &PhantomData::<<c_char as layout::ReprC>::CLayout> as &dyn PhantomCType
                    } else if formatted_constant.contains(".") {
                        &PhantomData::<<f64 as layout::ReprC>::CLayout> as &dyn PhantomCType
                    } else {
                        &PhantomData::<<libc::c_int as layout::ReprC>::CLayout> as &dyn PhantomCType
                    }
                } else {
                    ty
                };

                self.emit_type_usage(ctx, indent, "type", constant_type)?;
            }

            out!(("}}"));
        }

        Ok(())
    }

    fn must_declare_built_in_types(self: &'_ Self) -> bool {
        false
    }
}

impl Metadata {

    fn emit_type_usage (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        indent: &'_ Indentation,
        field_name: &'_ str,
        ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        mk_out!(indent, ctx.out());

        out!(("\"{field_name}\": {{"));

        if let _ = indent.scope() {
            let type_usage = ty.metadata_type_usage();

            for line in type_usage.lines() {
                out!(("{line}"));
            }
        }

        out!(("}}"));

        Ok(())
    }
}
