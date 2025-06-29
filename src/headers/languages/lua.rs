use super::*;

pub struct Lua;

impl HeaderLanguage for Lua {
    fn emit_docs(
        self: &'_ Self,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        indent: &'_ Indentation,
    ) -> io::Result<()> {
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

        let ref intn_t = backing_integer.map(|it| it.name(this));

        this.emit_docs(ctx, docs, indent)?;

        let ref short_name = self_ty.short_name();
        let ref full_ty_name = self_ty.name(this);

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
            out!(("}}; typedef {intn_t} {full_ty_name};"));
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
        C.declare_struct(this, ctx, docs, self_ty, fields)
    }

    fn declare_opaque_type(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        C.declare_opaque_type(this, ctx, docs, self_ty)
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
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        name: &'_ str,
        ty: &'_ dyn PhantomCType,
        _skip_type: bool,
        value: &'_ dyn ::core::fmt::Debug,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4);
        mk_out!(indent, ctx.out());

        this.emit_docs(ctx, docs, indent)?;
        let ty = ty.name(this);
        match ty.as_str() {
            | "int32_t" | "uint32_t" | "int16_t" | "uint16_t" | "int8_t" | "uint8_t" => {
                out!(("static const {ty} {name} = {value:?};"));
            },
            | "Opaque__str_t" => {
                out!(("extern const char* {name};"));
            },
            // Based on https://luajit.org/ext_ffi_semantics.html
            // "static const declarations only work for integer types up to 32 bits."
            | _ => panic!("Lua does not support this const type: {}", ty),
        }

        out!("\n");
        Ok(())
    }

    fn emit_function_ptr_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        out: &mut dyn io::Write,
        newtype_name: &'_ str,
        name: Option<&dyn ::core::fmt::Display>,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        C.emit_function_ptr_ty(this, out, newtype_name, name, args, ret_ty)
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

    fn emit_array_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        out: &mut dyn io::Write,
        var_name: Option<&dyn ::core::fmt::Display>,
        _newtype_name: &'_ str,
        elem_ty: &'_ dyn PhantomCType,
        array_len: usize,
    ) -> io::Result<()> {
        // In these instances, it is important to be mindful of ordering.
        // The ordering for arrays (or other such nested types) in C-like header languages goes from
        // outer to inner, since this is how the "unwrapping" operations would be written on a
        // variable of that type.
        //
        // To illustrate, given a `arr: [[c_int; 3]; 4]`, you'd have to index `..4` first, and then
        // to `..3`. Papering over off-by-one errors, this would be:
        // ```C
        // arr[4][3] // of type int.
        // ```
        //
        // Thus, the type definition is to become `int {var_name}[4][3]`.
        //
        // If we did `"{}[array_len]", elem_ty.name_wrapping_var(this)` in this instance, where
        // `elem_ty` refers to `[c_int; 3]`, and `array_len`, to `4`, we'd incorrectly end up with
        // `"{}[array_len]", "int {var_name}[3]"`, i.e., `int {var_name}[3][4]`!
        //
        // Hence the handling *first* this current outermost layer, and only then calling into
        // `elem_ty.…_wrapping_var()` logic on this transformated output.
        elem_ty.render_wrapping_var(
            out,
            this,
            Some(&format_args!(
                "{var_name}[{array_len}]",
                var_name = var_name.or_empty()
            )),
        )?;
        Ok(())
    }

    fn emit_void_output_type(
        self: &'_ Self,
        out: &mut dyn io::Write,
    ) -> io::Result<()> {
        C.emit_void_output_type(out)
    }
}
