#![cfg_attr(rustfmt, rustfmt::skip)]

use super::*;


pub struct Python;


/// Create a file whose content will be set in `ffi.cdef` of the Python module `cffi`
impl HeaderLanguage for Python {
    fn emit_docs (
        self: &'_ Self,
        _ctx: &'_ mut dyn Definer,
        _lang_config: &'_ &LanguageConfig,
        _docs: Docs<'_>,
        _indent: &'_ Indentation,
    ) -> io::Result<()>
    {
        // No documentation
        return Ok(())
    }

    fn emit_simple_enum (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        lang_config: &'_ &LanguageConfig,
        _docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        _backing_integer: Option<&dyn PhantomCType>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        let ref short_name = self_ty.short_name();
        let ref full_ty_name = self_ty.name(self);

        out!(("typedef enum {short_name} {{"));

        if let _ = indent.scope() {
            for v in variants {
                self.emit_docs(ctx, v.docs, indent)?;
                let variant_name = crate::utils::screaming_case(short_name, v.name) /* ctx.adjust_variant_name(
                    LanguageConfig::C,
                    enum_name,
                    v.name,
                ) */;
                out!(("{variant_name},"));
            }
        }

        out!(("}} {full_ty_name};"));
        out!("\n");
        Ok(())
    }

    fn emit_struct (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        lang_config: &'_ &LanguageConfig,
        _docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        fields: &'_ [StructField<'_>]
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());
        let short_name = self_ty.short_name();
        let full_ty_name = self_ty.name(self);

        if self_ty.size() == 0 {
            panic!("C does not support zero-sized structs!")
        }

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
        lang_config: &'_ &LanguageConfig,
        _docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());
        let full_ty_name = self_ty.name(self);
        out!(("typedef ... {full_ty_name};"));
        out!("\n");
        Ok(())
    }

    fn emit_function (
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        lang_config: &'_ &LanguageConfig,
        _docs: Docs<'_>,
        fname: &'_ str,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);

        let ref fn_sig_but_for_ret_type: String = {
            let mut buf = Vec::<u8>::new();
            mk_out!(indent, buf);

            out!("\n{indent}{fname} (");
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
        lang_config: &'_ &LanguageConfig,
        _docs: Docs<'_>,
        name: &'_ str,
        _ty: &'_ dyn PhantomCType,
        _value: &'_ dyn ::core::fmt::Debug,
    ) -> io::Result<()>
    {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        out!((
            "#define {name} ..."
        ));

        out!("\n");
        Ok(())
    }
}

#[derive(
    Debug, Default,
    Copy, Clone,
    PartialEq, Eq,
)]
pub
struct PythonLanguageConfig {

}