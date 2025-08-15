#![cfg_attr(rustfmt, rustfmt::skip)]

use core::fmt::Display;
use std::io::{BufRead, Write};
use libc::write;
use crate::layout;
use crate::utils::DisplayFromFn;
use super::*;

pub struct Metadata;

impl<F: Fn(&mut dyn ::std::io::Write) -> ::std::io::Result<()>> DisplayFromFn<F> {
    fn indented_lines(&self) -> String {
        self.to_string().lines().map(|line| format!("    {line}\n")).collect()
    }
}

impl HeaderLanguage for Metadata {
    fn declare_simple_enum(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
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
                    self.emit_type_usage(this, ctx, indent, "backingType", backing_integer)?;
                } else {
                    out!(("]"));
                }
            }

            out!(("}}"));
        }

        Ok(())
    }

    fn declare_struct (
        self: &'_ Self,
        this: &dyn HeaderLanguage,
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

                            self.emit_type_usage(this, ctx, indent, "type", field.ty)?;
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

    fn declare_opaque_type (
        self: &'_ Self,
        this: &dyn HeaderLanguage,
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

    fn declare_function (
        self: &'_ Self,
        this: &dyn HeaderLanguage,
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

                            self.emit_type_usage(this, ctx, indent, "type", arg.ty)?;
                        }

                        if index + 1 < args.len() {
                            out!(("}},"));
                        } else {
                            out!(("}}"));
                        }
                    }
                }

                out!(("],"));

                self.emit_type_usage(this, ctx, indent, "returnType", ret_ty)?;
            }

            out!(("}}"));
        }

        Ok(())
    }

    fn emit_primitive_ty(
        self: &'_ Self,
        out: &mut dyn Write,
        primitive: Primitive
    ) -> io::Result<()> {
        write!(
            out,
            r#""kind": "{kind}""#,
            kind = match primitive {
                Primitive::Bool => "bool".into(),
                Primitive::CChar => "char".into(),
                Primitive::Integer { signed, bitwidth } => match bitwidth {
                    IntBitWidth::PointerSized => {
                        let sign_prefix = if signed { "s" } else { "" };
                        format!("{sign_prefix}size_t")
                    },
                    IntBitWidth::CInt => {
                        let sign_prefix = if signed { "" } else { "u" };
                        format!("{sign_prefix}int")
                    },
                    IntBitWidth::Fixed(num_bits) => {
                        let prefix = if signed { "i" } else { "u" };
                        let num_bits = num_bits as u8;
                        format!("{prefix}{num_bits}")
                    },
                },
                Primitive::Float { bitwidth } => match bitwidth {
                    | FloatBitWidth::_32 => "f32",
                    | FloatBitWidth::_64 => "f64",
                }.into(),
            }
        )
    }

    fn emit_pointer_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        out: &mut dyn Write,
        pointee_is_immutable: bool,
        pointee: &'_ dyn PhantomCType
    ) -> io::Result<()> {
        writeln!(out, r#""kind": "Pointer","#)?;
        writeln!(
            out,
            r#""isMutable": {pointee_is_mutable},"#,
            pointee_is_mutable = !pointee_is_immutable
        )?;
        writeln!(out, r#""type": {{"#)?;
        write!(
            out,
            "{}",
            F(|out| pointee.render(out, this)).indented_lines(),
        )?;
        writeln!(out, "}}")
    }

    fn emit_void_output_type(self: &'_ Self, out: &mut dyn Write) -> io::Result<()> {
        write!(out, r#""kind": "void""#)
    }

    fn emit_function_ptr_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        out: &mut dyn Write,
        _newtype_name: &str,
        _name: Option<&dyn Display>,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType
    ) -> io::Result<()> {
        write!(
            out,
            r#"
            "kind": "Function",
            "valueParameters": [
            {value_parameters}
            ],
            "returnType": {{
            {return_type}
            }}
            "#,
            value_parameters = F(|out| {
                let first = &mut true;
                for arg in args {
                    if mem::take(first).not() {
                        write!(out, ", ")?;
                    }

                    write!(
                        out,
                        r#"{{
                        {argument}
                        }}"#,
                        argument = F(|out| arg.ty.render(out, this)).indented_lines(),
                    )?;
                }
                Ok(())
            }).indented_lines(),
            return_type = F(|out| ret_ty.render(out, this))
        )
    }

    fn emit_array_ty(
        self: &'_ Self, this: &dyn HeaderLanguage,
        out: &mut dyn Write,
        var_name: Option<&dyn Display>,
        newtype_name: &'_ str,
        elem_ty: &'_ dyn PhantomCType,
        array_len: usize
    ) -> io::Result<()> {
        writeln!(out, r#""kind": "StaticArray","#)?;
        writeln!(out, r#""backingTypeName": "{newtype_name}","#)?;
        writeln!(out, r#""size": {array_len},"#)?;
        writeln!(out, r#""type": {{"#)?;
        write!(
            out,
            "{}",
            F(|out| elem_ty.render(out, this)).indented_lines(),
        )?;
        writeln!(out, r#"}}"#)
    }

    fn declare_constant (
        self: &'_ Self,
        this: &dyn HeaderLanguage,
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
                out!((r#""kind": "Constant","#));

                self.emit_docs(ctx, docs, indent)?;

                out!((r#""name": "{name}","#));

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

                self.emit_type_usage(this, ctx, indent, "type", constant_type)?;
            }

            out!(("}}"));
        }

        Ok(())
    }

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

        out!((r#""comment": {{"#));

        if let _ = indent.scope() {
            out!((r#""lines": ["#));

            if let _ = indent.scope() {
                let docs_lines = docs.iter().copied()
                    .map(str::trim)
                    .map(|line| line.replace("\\", "\\\\"))
                    .map(|line| line.replace("\"", "\\\""));

                for (index, line) in docs_lines.enumerate() {
                    if index + 1 < docs.len() {
                        out!((r#""{line}","#));
                    } else {
                        out!((r#""{line}""#));
                    }
                }
            }

            out!(("]"));
        }

        out!(("}},"));

        Ok(())
    }

    fn must_declare_built_in_types(self: &'_ Self) -> bool {
        false
    }
}

impl Metadata {

    fn emit_type_usage(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        indent: &'_ Indentation,
        field_name: &'_ str,
        ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        mk_out!(indent, ctx.out());

        out!((r#""{field_name}": {{"#));

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
