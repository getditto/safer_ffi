use super::*;

pub struct CSharp;

#[derive(Default)]
pub struct CSharpMarshaler(pub &'static str);

impl CSharpMarshaler {
    fn pretty_print(
        &self,
        prefix: &str,
        suffix: &str,
    ) -> String {
        if self.0.is_empty().not() {
            format!("{prefix}{}{suffix}", self.0)
        } else {
            "".into()
        }
    }
}

impl HeaderLanguage for CSharp {
    fn emit_primitive_ty(
        self: &'_ Self,
        out: &mut dyn io::Write,
        primitive: Primitive,
    ) -> io::Result<()> {
        let this = self;
        match primitive {
            | Primitive::Bool => {
                write!(out, "bool")?;
            },
            | Primitive::CChar => {
                write!(out, "byte")?;
            },
            | Primitive::Integer { signed, bitwidth } => match bitwidth {
                | IntBitWidth::PointerSized => {
                    let sign_prefix = if signed { "" } else { "U" };
                    write!(out, "{sign_prefix}IntPtr")?;
                },
                | IntBitWidth::CInt => {
                    this.emit_primitive_ty(out, Primitive::Integer {
                        signed,
                        bitwidth: IntBitWidth::Fixed(FixedIntBitWidth::_32),
                    })?;
                },
                | IntBitWidth::Fixed(FixedIntBitWidth::_8) => {
                    let sign_prefix = if signed { "s" } else { "" };
                    write!(out, "{sign_prefix}byte")?;
                },
                | IntBitWidth::Fixed(numbits) => {
                    let sign_prefix = if signed { "" } else { "U" };
                    let numbits = numbits as u8;
                    write!(out, "{sign_prefix}Int{numbits}")?;
                },
            },
            | Primitive::Float { bitwidth } => write!(out, "{}", match bitwidth {
                | FloatBitWidth::_32 => "float",
                | FloatBitWidth::_64 => "double",
            })?,
        }
        Ok(())
    }

    fn emit_pointer_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        out: &mut dyn io::Write,
        pointee_is_immutable: bool,
        pointee: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        let maybe_const = if pointee_is_immutable {
            "/*const*/ "
        } else {
            ""
        };
        write!(
            out,
            "{pointee} {maybe_const}*",
            pointee = F(|out| pointee.render(out, this)),
        )
    }

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

    fn declare_simple_enum(
        self: &'_ CSharp,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
        backing_integer: Option<&dyn PhantomCType>,
        variants: &'_ [EnumVariant<'_>],
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        let ref IntN = backing_integer.map(|it| it.name(this));

        let ref full_ty_name = self_ty.name(this);

        this.emit_docs(ctx, docs, indent)?;

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
                this.emit_docs(ctx, v.docs, indent)?;
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

    fn declare_struct(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
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

        let ref name = self_ty.name(this);

        this.emit_docs(ctx, docs, indent)?;
        out!((
            "[StructLayout(LayoutKind.Sequential, Size = {size})]"
            "public unsafe struct {name} {{"
        ));
        if let _ = indent.scope() {
            let ref mut first = true;
            for &StructField {
                docs,
                name,
                ty: field_ty,
            } in fields
            {
                // Skip ZSTs
                if field_ty.size() == 0 {
                    if field_ty.align() > 1 {
                        panic!("Zero-sized fields must have an alignment of `1`");
                    } else {
                        continue;
                    }
                }
                if mem::take(first).not() {
                    out!("\n");
                }
                this.emit_docs(ctx, docs, indent)?;
                if let Some(CSharpMarshaler(csharp_marshaler)) = field_ty.metadata().dyn_request() {
                    out!((
                        "[MarshalAs({csharp_marshaler})]"
                    ));
                }
                out!(
                    ("public {};"),
                    F(|out| field_ty.render_wrapping_var(out, this, name)),
                );
            }
        }
        out!(("}}"));

        out!("\n");
        Ok(())
    }

    fn declare_opaque_type(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        docs: Docs<'_>,
        self_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        let ref indent = Indentation::new(4 /* ctx.indent_width() */);
        mk_out!(indent, ctx.out());

        let full_ty_name = self_ty.name(this);

        this.emit_docs(ctx, docs, indent)?;
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

    fn declare_function(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
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
            this.emit_docs(ctx, docs, indent)?;

            if let Some(CSharpMarshaler(marshaler)) =
                ret_ty.metadata().dyn_request::<CSharpMarshaler>()
            {
                out!((
                    "[return: MarshalAs({marshaler})]"
                ));
            }

            out!((
                "[DllImport(RustLib, ExactSpelling = true)] public static unsafe extern"
            ));

            let ret_ty = ret_ty.name(this);
            out!("{}{ret_ty} {fname} (", indent);
            let mut first = true;
            if let _ = indent.scope() {
                for FunctionArg {
                    name: arg_name,
                    ty: arg_ty,
                } in args
                {
                    if mem::take(&mut first).not() {
                        out!(",");
                    }
                    out!("\n");

                    if let Some(CSharpMarshaler(marshaler)) =
                        arg_ty.metadata().dyn_request::<CSharpMarshaler>()
                    {
                        out!((
                            "[MarshalAs({marshaler})]"
                        ));
                    }
                    let arg_ty = arg_ty.name(this);
                    out!("{}{arg_ty} {arg_name}", indent)
                }
            }
            out!(");\n");
        }
        out!(("}}"));

        out!("\n");
        Ok(())
    }

    fn declare_constant(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
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
            this.emit_docs(ctx, docs, indent)?;
            let ty = ty.name(this);
            out!((
                "public const {ty} {name} = {value:?};"
            ));
        }
        out!(("}}"));

        out!("\n");
        Ok(())
    }

    fn define_function_ptr_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        ctx: &'_ mut dyn Definer,
        self_ty: &'_ dyn PhantomCType,
        args: &'_ [FunctionArg<'_>],
        ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        let out = ctx.out();
        write!(
            out,
            concat!(
                // IIUC,
                //   - For 32-bits / x86, Rust's extern "C" is the same as C#'s (default) Winapi:
                //     "cdecl" for Linux, and "stdcall" for Windows.
                //
                //   - For everything else, this is param is ignored. I guess because both OSes
                //     agree on the calling convention?
                "[UnmanagedFunctionPointer(CallingConvention.Winapi)]\n",
                "{ret_ty_marshaler}",
                "public unsafe /* static */ delegate\n",
                "    {Ret}\n",
                "    {me} (",
                "{args}",
                ");",
                "\n",
                "\n",
            ),
            ret_ty_marshaler = ret_ty
                .metadata()
                .dyn_request::<CSharpMarshaler>()
                .unwrap_or_default()
                .pretty_print("[return: MarshalAs(", ")]\n"),
            Ret = F(|out| ret_ty.render(out, this)),
            me = self_ty.name(this),
            args = F(|out| {
                let first = &mut true;
                for (arg_idx, arg_ty) in (0..).zip(args) {
                    write!(
                        out,
                        concat! {
                            "{intro_sep}",
                            "{arg_ty_marshaler}",
                            "{arg_ty}",
                        },
                        intro_sep = if mem::take(first) {
                            "\n        "
                        } else {
                            ",\n        "
                        },
                        arg_ty_marshaler = arg_ty
                            .ty
                            .metadata()
                            .dyn_request::<CSharpMarshaler>()
                            .unwrap_or_default()
                            .pretty_print("[MarshalAs(", ")]\n        "),
                        arg_ty = arg_ty.ty.name_wrapping_var(this, &format!("_{arg_idx}"),),
                    )?;
                }
                Ok(())
            }),
        )?;
        Ok(())
    }

    fn emit_function_ptr_ty(
        self: &'_ Self,
        _this: &dyn HeaderLanguage,
        out: &mut dyn io::Write,
        newtype_name: &str,
        _name: &'_ str,
        _args: &'_ [FunctionArg<'_>],
        _ret_ty: &'_ dyn PhantomCType,
    ) -> io::Result<()> {
        write!(out, "{}", newtype_name)
    }

    fn define_array_ty(
        self: &'_ Self,
        this: &dyn HeaderLanguage,
        definer: &'_ mut dyn Definer,
        self_ty: &'_ dyn PhantomCType,
        elem_ty: &'_ dyn PhantomCType,
        array_len: usize,
    ) -> io::Result<()> {
        let me = self_ty.name(this);
        let array_items = F(|out| {
            let elem_ty_name = elem_ty.name(this);
            #[rustfmt::skip]
            const FIXED_ARRAY_COMPATIBLE_TYPE_NAMES: &[&str] = &[
                "bool",
                "byte", "UInt8", "UInt16", "UInt32", "UInt64", "UIntPtr",
                "sbyte", "Int8", "Int16", "Int32", "Int64", "IntPtr",
                "float", "double",
            ];
            // Poorman's specialization to use `fixed` arrays.
            if FIXED_ARRAY_COMPATIBLE_TYPE_NAMES.contains(&&elem_ty_name[..]) {
                write!(
                    out,
                    "    public fixed {elem_ty_name} arr[{array_len}];\n",
                    /* no need for a marshaler here */
                )?;
            } else {
                // Sadly for the general case fixed arrays are
                // not supported.
                for i in 0..array_len {
                    write!(
                        out,
                        "    {marshaler}public {elem_ty_name} _{i};\n",
                        marshaler = elem_ty
                            .metadata()
                            .dyn_request::<CSharpMarshaler>()
                            .unwrap_or_default()
                            .pretty_print("[MarshalAs(", ")]\n    "),
                    )?;
                }
            }
            Ok(())
        });
        writeln!(
            definer.out(),
            concat!(
                "[StructLayout(LayoutKind.Sequential, Size = {size})]\n",
                "public unsafe struct {me} {{\n",
                "{array_items}",
                "}}\n",
            ),
            me = me,
            array_items = array_items,
            size = self_ty.size(),
        )?;
        Ok(())
    }

    fn emit_array_ty(
        self: &'_ Self,
        _this: &dyn HeaderLanguage,
        out: &mut dyn io::Write,
        _var_name: &'_ str,
        newtype_name: &'_ str,
        _elem_ty: &'_ dyn PhantomCType,
        _array_len: usize,
    ) -> io::Result<()> {
        write!(out, "{newtype_name}")
    }

    fn emit_void_output_type(
        self: &'_ Self,
        out: &mut dyn io::Write,
    ) -> io::Result<()> {
        // TODO: remove default impl
        write!(out, "void")?;
        Ok(())
    }
}
