use super::*;

const_assert! {
    ::core::mem::size_of::<crate::libc::uintptr_t>()
    ==
    ::core::mem::size_of::<crate::libc::size_t>()
}

const _: () = { macro_rules! impl_CTypes {
    () => (
        impl_CTypes! { @pointers }
        impl_CTypes! { @zsts }
        impl_CTypes! { @floats
            unsafe
            f32 => "float",

            unsafe
            f64 => "double",
        }
        impl_CTypes! { @integers
            // C# safety: equivalence based onhttps://docs.microsoft.com/en-us/dotnet/csharp/language-reference/builtin-types/built-in-types

            unsafe // Safety: trivial integer equivalence.
            u8 => "uint8" "byte",

            unsafe // Safety: trivial integer equivalence.
            u16 => "uint16" "UInt16",

            unsafe // Safety: trivial integer equivalence.
            u32 => "uint32" "UInt32",

            unsafe // Safety: trivial integer equivalence.
            u64 => "uint64" "UInt64",

            // unsafe u128 => "uint128",

            unsafe // Safety: Contrary to what most people think,
                   // `usize` is not a `size_t` but an `uintptr_t`,
                   // since it has a guaranteed non-`unsafe` transmute (`as`)
                   // with pointers.
                   //
                   // That being said, many places where Rust uses `usize`
                   // C would expect a `size_t` instead, so there is definitely
                   // a confusion going on with Rust in that regard.
                   //
                   // In practice, it looks like Rust will therefore never
                   // support a platform where `size_t != uintptr_t`.
                   //
                   // Given that, and given how `size_t` for, for instance,
                   // slice lengths, feels far more natural and idiomatic,
                   // this crate makes the opinionated choice not to support
                   // such a platform, so as to use `size_t` instead.
                   //
                   // To ensure soundness in case Rust were to support such as
                   // platform, a compile-time assertion is added, that
                   // ensure the crate will not compile on such platforms.
                   // (search for `size_of` in this file).
            usize => "size" "UIntPtr",


            unsafe // Safety: trivial integer equivalence.
            i8 => "int8" "sbyte",

            unsafe // Safety: trivial integer equivalence.
            i16 => "int16" "Int16",

            unsafe // Safety: trivial integer equivalence.
            i32 => "int32" "Int32",

            unsafe // Safety: trivial integer equivalence.
            i64 => "int64" "Int32",

            // unsafe i128 => "int128",

            unsafe // Safety: See `usize`'s
            isize => "ssize" "IntPtr",
        }
        #[cfg(docs)] impl_CTypes! { @fns (A1) } #[cfg(not(docs))]
        impl_CTypes! { @fns
            (A9, A8, A7, A6, A5, A4, A3, A2, A1)
        }
        #[cfg(docs)] impl_CTypes! { @arrays 1 2 } #[cfg(not(docs))]
        impl_CTypes! { @arrays
            // 0
            1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20 21 22 23 24 25
            26 27 28 29 30 31 32 40 48 50 60 64 70 75 80 90 96 100 125 128 192
            200 250 256 300 400 500 512 600 700 750 800 900 1000 1024
        }
    );

    (
        @arrays
        $($N:tt)*
    ) => ($(
        // CType
        /// Simplified for lighter documentation, but the actual impls
        /// range **from `1` up to `32`, plus a bunch of significant
        /// lengths up to `1024`**.
        unsafe // Safety: Rust arrays _are_ `#[repr(C)]`
        impl<Item : CType> CType
            for [Item; $N]
        { __cfg_headers__! {
            fn c_short_name_fmt (fmt: &'_ mut fmt::Formatter<'_>)
              -> fmt::Result
            {
                // item_N_array
                write!(fmt,
                    concat!("{}_", stringify!($N), "_array"),
                    Item::c_short_name(),
                )
            }

            fn c_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            {
                let ref me = Self::c_var("").to_string();
                definer.define_once(
                    me,
                    &mut |definer| {
                        Item::c_define_self(definer)?;
                        writeln!(definer.out(),
                            concat!(
                                "typedef struct {{\n",
                                "    {inline_array};\n",
                                "}} {me};\n",
                            ),
                            inline_array = Item::c_var(concat!(
                                "idx[", stringify!($N), "]",
                            )),
                            me = me,
                        )
                    }
                )
            }

            fn c_var_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                // _e.g._, item_N_array_t
                write!(fmt,
                    "{}_t{sep}{}",
                    Self::c_short_name(),
                    var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            }

            __cfg_csharp__! {
                fn csharp_define_self (definer: &'_ mut dyn Definer)
                  -> io::Result<()>
                {
                    let ref me = Self::csharp_ty();
                    Item::csharp_define_self(definer)?;
                    definer.define_once(me, &mut |definer| {
                        let array_items = {
                            // Poor man's specialization to use `fixed` arrays.
                            if  [
                                    "bool",
                                    "u8", "u16", "u32", "u64", "usize",
                                    "i8", "i16", "i32", "i64", "isize",
                                    "float", "double",
                                ].contains(&::core::any::type_name::<Item>())
                            {
                                format!(
                                    "    public fixed {ItemTy} arr[{N}];\n",
                                    ItemTy = Item::csharp_ty(),
                                    N = $N,
                                    // no need for a marshaler here
                                )
                            } else {
                                // Sadly for the general case fixed arrays are
                                // not supported.
                                (0 .. $N)
                                    .map(|i| format!(
                                        "    \
                                        {marshaler}\
                                        public {ItemTy} _{i};\n",
                                        ItemTy = Item::csharp_ty(),
                                        i = i,
                                        marshaler =
                                            Item::csharp_marshaler()
                                                .map(|m| format!("[MarshalAs({})]\n    ", m))
                                                .as_deref()
                                                .unwrap_or("")
                                        ,
                                    ))
                                    .collect::<rust::String>()
                            }
                        };
                        writeln!(definer.out(),
                            concat!(
                                "[StructLayout(LayoutKind.Sequential, Size = {size})]\n",
                                "public unsafe struct {me} {{\n",
                                "{array_items}",
                                "}}\n",
                            ),
                            me = me,
                            array_items = array_items,
                            size = mem::size_of::<Self>(),
                        )
                    })
                }
            }
        } type OPAQUE_KIND = OpaqueKind::Concrete; }

        // ReprC
        /// Simplified for lighter documentation, but the actual impls
        /// range **from `1` up to `32`, plus a bunch of significant
        /// lengths up to `1024`**.
        unsafe
        impl<Item : ReprC> ReprC
            for [Item; $N]
        {
            type CLayout = [Item::CLayout; $N];

            #[inline]
            fn is_valid (it: &'_ Self::CLayout)
              -> bool
            {
                it.iter().all(Item::is_valid)
            }
        }
    )*);

    (@fns
        (
            $(
                $An:ident $(,
                $Ai:ident)* $(,)?
            )?
        )
    ) => (
        // recurse
        $(
            impl_CTypes! {
                @fns
                ($($Ai ,)*)
            }
        )?

        // CType
        /// Simplified for lighter documentation, but the actual impls include
        /// **up to 9 function parameters**.
        unsafe // Safety: this is the "blessed" type recommended across Rust
               // literature. Still the alignment of function pointers is not
               // as well-defined, as one would wish.
        impl<
            Ret : CType, $(
            $An : CType, $(
            $Ai : CType,
        )*)?> CType
            for Option<unsafe extern "C" fn ($($An, $($Ai ,)*)?) -> Ret>
        { __cfg_headers__! {
            fn c_short_name_fmt (fmt: &'_ mut fmt::Formatter<'_>)
              -> fmt::Result
            {
                // ret_arg1_arg2_fptr
                Ret::c_short_name_fmt(fmt)?; $(
                write!(fmt, "_{}", $An::c_short_name())?; $(
                write!(fmt, "_{}", $Ai::c_short_name())?; )*)?
                fmt.write_str("_fptr")
            }

            fn c_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            {
                Ret::c_define_self(definer)?; $(
                $An::c_define_self(definer)?; $(
                $Ai::c_define_self(definer)?; )*)?
                Ok(())
            }

            fn c_var_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                write!(fmt, "{} ", Ret::c_var(""))?;
                write!(fmt, "(*{})(", var_name)?;
                let _empty = true; $(
                let _empty = false;
                write!(fmt, "{}", $An::c_var(""))?; $(
                write!(fmt, ", {}", $Ai::c_var(""))?; )*)?
                if _empty {
                    fmt.write_str("void")?;
                }
                fmt.write_str(")")
            }

            __cfg_csharp__! {
                fn csharp_define_self (definer: &'_ mut dyn Definer)
                  -> io::Result<()>
                {
                    Ret::csharp_define_self(definer)?; $(
                    $An::csharp_define_self(definer)?; $(
                    $Ai::csharp_define_self(definer)?; )*)?
                    let ref me = Self::csharp_ty();
                    let ref mut _arg = {
                        let mut iter = (0 ..).map(|c| format!("_{}", c));
                        move || iter.next().unwrap()
                    };
                    definer.define_once(me, &mut |definer| writeln!(definer.out(),
                        concat!(
                            // IIUC,
                            //   - For 32-bits / x86,
                            //     Rust's extern "C" is the same as C#'s (default) Winapi:
                            //     "cdecl" for Linux, and "stdcall" for Windows.
                            //
                            //   - For everything else, this is param is ignored.
                            //     I guess because both OSes agree on the calling convention?
                            "[UnmanagedFunctionPointer(CallingConvention.Winapi)]\n",

                            "{ret_marshaler}public unsafe /* static */ delegate\n",
                            "    {Ret}\n",
                            "    {me} (", $("\n",
                            "        {}{", stringify!($An), "}", $(",\n",
                            "        {}{", stringify!($Ai), "}", )*)?
                            ");\n"
                        ),$(
                        $An::csharp_marshaler()
                            .map(|m| format!("[MarshalAs({})]\n        ", m))
                            .as_deref()
                            .unwrap_or("")
                        , $(
                        $Ai::csharp_marshaler()
                            .map(|m| format!("[MarshalAs({})]\n        ", m))
                            .as_deref()
                            .unwrap_or("")
                        , )*)?
                        me = me,
                        ret_marshaler =
                            Ret::csharp_marshaler()
                                .map(|m| format!("[return: MarshalAs({})]\n", m))
                                .as_deref()
                                .unwrap_or("")
                        ,
                        Ret = Ret::csharp_ty(), $(
                        $An = $An::csharp_var(&_arg()), $(
                        $Ai = $Ai::csharp_var(&_arg()), )*)?
                    ))
                }

                fn csharp_marshaler ()
                  -> Option<rust::String>
                {
                    // This assumes the calling convention from the above
                    // `UnmanagedFunctionPointer` attribute.
                    Some("UnmanagedType.FunctionPtr".into())
                }
            }
        } type OPAQUE_KIND = OpaqueKind::Concrete; }

        /// Simplified for lighter documentation, but the actual impls include
        /// **up to 9 function parameters**.
        unsafe // Safety: byte-wise the layout is the same, but the safety
               // invariants will still have to be checked at each site.
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> ReprC
            for Option<unsafe extern "C" fn ($($An, $($Ai ,)*)?) -> Ret>
        {
            type CLayout = Option<
                unsafe extern "C"
                fn ($($An::CLayout, $($Ai::CLayout ,)*)?) -> Ret::CLayout
            >;

            #[inline]
            fn is_valid (_: &'_ Self::CLayout)
              -> bool
            {
                true
            }
        }

        /// Simplified for lighter documentation, but the actual impls include
        /// **up to 9 function parameters**.
        unsafe // Safety: byte-wise the layout is the same, but the safety
               // invariants will still have to be checked at each site.
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> ReprC
            for Option</*unsafe*/ extern "C" fn ($($An, $($Ai ,)*)?) -> Ret>
        {
            type CLayout = Option<
                unsafe extern "C"
                fn ($($An::CLayout, $($Ai::CLayout ,)*)?) -> Ret::CLayout
            >;

            #[inline]
            fn is_valid (_: &'_ Self::CLayout)
              -> bool
            {
                true
            }
        }

        /* == ReprC for Option-less == */

        /// Simplified for lighter documentation, but the actual impls include
        /// **up to 9 function parameters**.
        unsafe // Safety: byte-wise the layout is the same, but the safety
               // invariants will still have to be checked at each site.
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> ReprC
            for unsafe extern "C" fn ($($An, $($Ai ,)*)?) -> Ret
        {
            type CLayout = Option<
                unsafe extern "C"
                fn ($($An::CLayout, $($Ai::CLayout ,)*)?) -> Ret::CLayout
            >;

            #[inline]
            fn is_valid (c_layout: &'_ Self::CLayout)
              -> bool
            {
                c_layout.is_some()
            }
        }

        /// Simplified for lighter documentation, but the actual impls include
        /// **up to 9 function parameters**.
        unsafe // Safety: byte-wise the layout is the same, but the safety
               // invariants will still have to be checked at each site.
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> ReprC
            for /*unsafe*/ extern "C" fn ($($An, $($Ai ,)*)?) -> Ret
        {
            type CLayout = Option<
                unsafe extern "C"
                fn ($($An::CLayout, $($Ai::CLayout ,)*)?) -> Ret::CLayout
            >;

            #[inline]
            fn is_valid (c_layout: &'_ Self::CLayout)
              -> bool
            {
                c_layout.is_some()
            }
        }

        // Improve the error message when encountering a non-`extern "C"` fn
        // wrapped in an `Option` (otherwise `rustc` tunnelvisions _w.r.t_
        // the lack of Niche).
        unsafe // Safety: `Self : ReprC` is not met so this impl never happens
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> crate::layout::__HasNiche__
            for /*unsafe*/ /*extern "C"*/ fn ($($An, $($Ai ,)*)?) -> Ret
        where
            Self : ReprC, // bound not met
        {
            #[inline]
            fn is_niche (_: &'_ Self::CLayout)
              -> bool
            {
                unreachable!()
            }
        }
        unsafe // Safety: `Self : ReprC` is not met so this impl never happens
        impl<
            Ret : ReprC, $(
            $An : ReprC, $(
            $Ai : ReprC,
        )*)?> crate::layout::__HasNiche__
            for unsafe /*extern "C"*/ fn ($($An, $($Ai ,)*)?) -> Ret
        where
            Self : ReprC, // bound not met
        {
            #[inline]
            fn is_niche (_: &'_ Self::CLayout)
            -> bool
            {
                unreachable!()
            }
        }
    );

    (@integers
        $(
            $unsafe:tt
            $RustInt:ident => $CInt:literal $CSharpInt:literal,
        )*
    ) => ($(
        $unsafe // Safety: guaranteed by the caller of the macro
        impl CType
            for $RustInt
        { __cfg_headers__! {
            fn c_short_name_fmt (fmt: &'_ mut fmt::Formatter<'_>)
              -> fmt::Result
            {
                fmt.write_str($CInt)
            }

            fn c_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            {
                definer.define_once(
                    "__int_headers__",
                    &mut |definer| write!(definer.out(),
                        concat!(
                            "\n",
                            "#include <stddef.h>\n",
                            "#include <stdint.h>\n",
                            "\n",
                        ),
                    ),
                )
            }

            fn c_var_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                write!(fmt,
                    concat!($CInt, "_t{sep}{}"),
                    var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            }

            __cfg_csharp__! {
                fn csharp_ty ()
                  -> rust::String
                {
                    $CSharpInt.into()
                }
            }
        } type OPAQUE_KIND = OpaqueKind::Concrete; }
        from_CType_impl_ReprC! { $RustInt }
    )*);

    (@floats
        $(
            $unsafe:tt
            $fN:ident => $Cty:literal,
        )*
    ) => ($(
        $unsafe // Safety: guaranteed by the caller of the macro
        impl CType
            for $fN
        { __cfg_headers__! {
            fn c_short_name_fmt (fmt: &'_ mut fmt::Formatter<'_>)
              -> fmt::Result
            {
                fmt.write_str($Cty)
            }

            fn c_var_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                write!(fmt,
                    concat!($Cty, "{sep}{}"),
                    var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            }

            __cfg_csharp__! {
                fn csharp_ty ()
                  -> rust::String
                {
                    $Cty.into()
                }
            }
        } type OPAQUE_KIND = OpaqueKind::Concrete; }
        from_CType_impl_ReprC! { $fN }
    )*);

    (
        @pointers
    ) => (
        unsafe
        impl<T : CType> CType
            for *const T
        { __cfg_headers__! {
            fn c_short_name_fmt (fmt: &'_ mut fmt::Formatter<'_>)
              -> fmt::Result
            {
                write!(fmt, "{}_const_ptr", T::c_short_name())
            }

            fn c_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            {
                T::c_define_self(definer)
            }

            fn c_var_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                write!(fmt,
                    "{} const *{sep}{}",
                    T::c_var(""),
                    var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            }

            __cfg_csharp__! {
                fn csharp_define_self (definer: &'_ mut dyn $crate::headers::Definer)
                  -> $crate::std::io::Result<()>
                {
                    T::csharp_define_self(definer)?;
                    // definer.define_once("Const", &mut |definer| {
                    //     definer.out().write_all(concat!(
                    //         "[StructLayout(LayoutKind.Sequential)]\n",
                    //         "public readonly struct Const<T> {\n",
                    //         "    public readonly T value;\n",
                    //         "}\n\n",
                    //     ).as_bytes())
                    // })?
                    Ok(())
                }

                fn csharp_ty ()
                  -> rust::String
                {
                    format!("{} /*const*/ *", T::csharp_ty())
                }
            }
        } type OPAQUE_KIND = OpaqueKind::Concrete; }

        unsafe
        impl<T : ReprC> ReprC
        for *const T
        {
            type CLayout = *const T::CLayout;

            #[inline]
            fn is_valid (_: &'_ Self::CLayout)
              -> bool
            {
                true
            }
        }

        unsafe
        impl<T : CType> CType
            for *mut T
        { __cfg_headers__! {
            fn c_short_name_fmt (fmt: &'_ mut fmt::Formatter<'_>)
              -> fmt::Result
            {
                write!(fmt, "{}_ptr", T::c_short_name())
            }

            fn c_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            {
                T::c_define_self(definer)
            }

            fn c_var_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                write!(fmt,
                    "{} *{sep}{}",
                    T::c_var(""),
                    var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            }

            __cfg_csharp__! {
                fn csharp_define_self (definer: &'_ mut dyn $crate::headers::Definer)
                  -> $crate::std::io::Result<()>
                {
                    T::csharp_define_self(definer)
                }

                fn csharp_ty ()
                  -> rust::String
                {
                    format!("{} *", T::csharp_ty())
                }
            }
        } type OPAQUE_KIND = OpaqueKind::Concrete; }
        unsafe
        impl<T : ReprC> ReprC
            for *mut T
        {
            type CLayout = *mut T::CLayout;

            #[inline]
            fn is_valid (_: &'_ Self::CLayout)
              -> bool
            {
                true
            }
        }
    );

    (
        @zsts
    ) => (
        // needed for compatibility with functions returning `()`
        // FIXME: Use special impls in `@fns` for `-> ()` instead.
        unsafe
        impl ReprC
            for ()
        {
            type CLayout = CVoid;

            #[inline]
            fn is_valid (_: &'_ CVoid)
              -> bool
            {
                panic!("It is a logic error to try and get a ZST from C");
            }
        }
        // Needed for structs containing a `PhantomData` field.
        unsafe
        impl<T : ?Sized> ReprC
            for PhantomData<T>
        {
            type CLayout = CVoid;

            #[inline]
            fn is_valid (_: &'_ CVoid)
              -> bool
            {
                panic!("It is a logic error to try and get a ZST from C");
            }
        }
    );
} impl_CTypes! {} };

macro_rules! impl_ReprC_for {(
    $unsafe:tt {
        $(
            $(@for [$($generics:tt)+])? $T:ty
                => |ref $it:tt : $Layout:ty| $expr:expr
        ),* $(,)?
    }
) => (
    $(
        $unsafe
        impl $(<$($generics)+>)? ReprC
            for $T
        {
            type CLayout = $Layout;

            #[inline]
            fn is_valid (it: &'_ $Layout)
              -> bool
            {
                let $it = it;
                if $expr {
                    true
                } else {
                    #[cfg(feature = "log")]
                    ::log::error!(
                        "{:#x?} is not a _valid_ bit pattern for the type `{}`",
                        unsafe {
                            ::core::slice::from_raw_parts(
                                <*const _>::cast::<u8>(it),
                                ::core::mem::size_of_val(it),
                            )
                        },
                        ::core::any::type_name::<Self>(),
                    );
                    false
                }
            }
        }
    )*
)}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(missing_debug_implementations)]
pub
struct Bool(u8);

unsafe
    impl CType
        for Bool
    {
        __cfg_headers__! {
            fn c_short_name_fmt (fmt: &'_ mut fmt::Formatter<'_>)
              -> fmt::Result
            {
                fmt.write_str("bool")
            }

            fn c_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            {
                definer.define_once(
                    "bool",
                    &mut |definer| {
                        definer.out().write_all(
                            b"\n#include <stdbool.h>\n\n"
                        )
                    },
                )
            }

            fn c_var_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                write!(fmt,
                    "bool{sep}{}",
                    var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            }

            __cfg_csharp__! {
                fn csharp_marshaler ()
                  -> Option<rust::String>
                {
                    Some("UnmanagedType.U1".into())
                }

                fn csharp_ty ()
                  -> rust::String
                {
                    "bool".into()
                }
            }
        }

        type OPAQUE_KIND = OpaqueKind::Concrete;
    }

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
#[allow(missing_debug_implementations)]
pub
struct c_int(pub crate::libc::c_int);

unsafe
    impl CType
        for c_int
    {
        __cfg_headers__! {
            fn c_short_name_fmt (fmt: &'_ mut fmt::Formatter<'_>)
                -> fmt::Result
            {
                fmt.write_str("int")
            }

            fn c_var_fmt (
                fmt: &'_ mut fmt::Formatter<'_>,
                var_name: &'_ str,
            ) -> fmt::Result
            {
                write!(fmt,
                    "int{sep}{}",
                    var_name,
                    sep = if var_name.is_empty() { "" } else { " " },
                )
            }

            __cfg_csharp__! {
                fn csharp_ty ()
                  -> rust::String
                {
                    "int".into()
                }

                fn csharp_marshaler ()
                  -> Option<rust::String>
                {
                    Some("UnmanagedType.SysInt".into())
                }
            }
        }

        type OPAQUE_KIND = OpaqueKind::Concrete;
    }

impl_ReprC_for! { unsafe {
    bool
        => |ref byte: Bool| (byte.0 & !0b1) == 0
    ,

    @for[T : ReprC]
    ptr::NonNull<T>
        => |ref it: *mut T::CLayout| {
            it.is_null().not() &&
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for[T : ReprC]
    ptr::NonNullRef<T>
        => |ref it: *const T::CLayout| {
            it.is_null().not() &&
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for[T : ReprC]
    ptr::NonNullMut<T>
        => |ref it: *mut T::CLayout| {
            it.is_null().not() &&
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for[T : ReprC]
    ptr::NonNullOwned<T>
        => |ref it: *mut T::CLayout| {
            it.is_null().not() &&
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for['a, T : 'a + ReprC]
    &'a T
        => |ref it: *const T::CLayout| {
            it.is_null().not() &&
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for['a, T : 'a + ReprC]
    &'a mut T
        => |ref it: *mut T::CLayout| {
            it.is_null().not() &&
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
}}

/* `HasNiche` from `niche.rs` impls `ReprC` for `Option<ptr>` types. */

#[cfg(feature = "out-refs")]
impl_ReprC_for! { unsafe {
    @for['out, T : 'out + Sized + ReprC]
    Out<'out, T>
        => |ref it: *mut T::CLayout| {
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        },
}}
