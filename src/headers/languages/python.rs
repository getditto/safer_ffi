use super::*;

pub struct Python;

/// Create a file whose content will be set in `ffi.cdef` of the Python module `cffi`
impl HeaderLanguage for Python {
    fn emit_docs(
        self: &'_ Self,
        _ctx: &'_ mut dyn Definer,
        _docs: Docs<'_>,
        _indent: &'_ Indentation,
    ) -> io::Result<()> {
        // No documentation
        return Ok(());
    }

    fn declare_simple_enum(
        self: &'_ Self,
        _this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        _docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        _backing_integer: Option<&dyn PhantomCType>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        let ref short_name = self_ty.short_name();
        let ref full_ty_name = self_ty.name(self);

        out!(("typedef enum {short_name} {{"));

        if let _ = indent.scope() {
            for v in variants {
                self.emit_docs(ctx, v.docs, indent)?;
                let variant_name = crate::utils::screaming_case(short_name, v.name) /* ctx.adjust_variant_name(
                    Language::C,
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

    fn declare_struct(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        fields: &'_ [StructField<'_>],
    ) -> io::Result<()> {
        C.declare_struct(this, ctx, docs, self_ty, fields)
    }

    fn declare_opaque_type(
        self: &'_ Self,
        _this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        _docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());
        let full_ty_name = self_ty.name(self);
        out!(("typedef ... {full_ty_name};"));
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
        C.declare_function(this, ctx, docs, fname, args, ret_ty)
    }

    fn declare_constant(
        self: &'_ Self,
        _this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        _docs: Docs<'_>,
        name: &'_ str,
        _ty: &'_ dyn PhantomCType,
        _skip_type: bool,
        _value: &'_ dyn ::core::fmt::Debug,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        out!((
            "#define {name} ..."
        ));

        out!("\n");
        Ok(())
    }

    fn emit_primitive_ty(
        self: &'_ Self,
        out: &mut dyn io::Write,
        primitive: Primitive,
    ) -> io::Result<()> {
        C.emit_primitive_ty(out, primitive)
    }

    fn emit_pointer_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        out: &mut dyn io::Write,
        pointee_is_immutable: bool,
        pointee: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        C.emit_pointer_ty(this, out, pointee_is_immutable, pointee)
    }

    fn define_function_ptr_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        self_ty: &'_ dyn PhantomCType,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        C.define_function_ptr_ty(this, ctx, self_ty, args, ret_ty)
    }

    fn emit_function_ptr_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        out: &mut dyn io::Write,
        newtype_name: &'_ str,
        name: &'_ str,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        C.emit_function_ptr_ty(this, out, newtype_name, name, args, ret_ty)
    }

    fn define_array_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        self_ty: &'_ dyn PhantomCType,
        elem_ty: &'_ dyn PhantomCType,
        array_len: usize,
    ) -> io::Result<()> {
        C.define_array_ty(this, ctx, self_ty, elem_ty, array_len)
    }

    fn emit_array_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        out: &mut dyn io::Write,
        var_name: &'_ str,
        newtype_name: &'_ str,
        elem_ty: &'_ dyn PhantomCType,
        array_len: usize,
    ) -> io::Result<()> {
        C.emit_array_ty(this, out, var_name, newtype_name, elem_ty, array_len)
    }

    fn emit_void_output_type(
        self: &'_ Self,
        out: &mut dyn io::Write,
    ) -> io::Result<()> {
        C.emit_void_output_type(out)
    }
}
