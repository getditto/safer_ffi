use super::*;

const _: () = { macro_rules! impl_CTypes {
    () => (
        impl_CTypes! { @pointers }
        impl_CTypes! { @zsts }
        impl_CTypes! { @integers
            unsafe // Safety: trivial integer equivalence.
            u8 => "uint8",

            unsafe // Safety: trivial integer equivalence.
            u16 => "uint16",

            unsafe // Safety: trivial integer equivalence.
            u32 => "uint32",

            unsafe // Safety: trivial integer equivalence.
            u64 => "uint64",

            // unsafe u128 => "uint128",

            unsafe // Safety: Contrary to what most people think,
                   // `usize` is not a `size_t` but an `uintptr_t`,
                   // since it has a guaranteed non-`unsafe` transmute (`as`)
                   // with pointers.
            usize => "uintptr",


            unsafe // Safety: trivial integer equivalence.
            i8 => "int8",

            unsafe // Safety: trivial integer equivalence.
            i16 => "int16",

            unsafe // Safety: trivial integer equivalence.
            i32 => "int32",

            unsafe // Safety: trivial integer equivalence.
            i64 => "int64",

            // unsafe i128 => "int128",

            unsafe // Safety: Contrary to what most people think,
                   // `isize` is not a `ssize_t` but an `intptr_t`,
                   // since it has a guaranteed non-`unsafe` transmute (`as`)
                   // with pointers.
            isize => "intptr",
        }
        #[cfg(docs)] impl_CTypes! { @fns (A1) } #[cfg(not(docs))]
        impl_CTypes! { @fns
            (A6, A5, A4, A3, A2, A1)
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
                // item_t_N_array
                write!(fmt,
                    concat!("{}_", stringify!($N), "_array"),
                    Item::c_short_name(),
                )
            }

            fn c_define_self (definer: &'_ mut dyn Definer)
              -> io::Result<()>
            {
                let mut buf = &mut [0_u8; 256][..];
                {
                    use ::std::io::Write;
                    write!(buf, "{}", Self::c_short_name())
                        .expect("`short_name()` was too long")
                }
                if let Some(n) = buf.iter().position(|&b| b == b'\0') {
                    buf = &mut buf[.. n];
                }
                let short_name = ::core::str::from_utf8(buf).unwrap();
                definer.define_once(
                    short_name,
                    &mut |definer| {
                        Item::c_define_self(definer)?;
                        write!(definer.out(),
                            "typedef struct {{ {}; }} {}_t;\n\n",
                            Item::c_var(concat!(
                                "idx[", stringify!($N), "]",
                            )),
                            short_name,
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
        }}

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
        /// **up to 6 function parameters**.
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
                // ret_t_arg1_t_arg2_t_fptr
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
                write!(fmt, "(*{})(", var_name)?; $(
                write!(fmt, "{}", $An::c_var(""))?; $(
                write!(fmt, ", {}", $Ai::c_var(""))?; )*)?
                fmt.write_str(")")
            }
        }}

        /// Simplified for lighter documentation, but the actual impls include
        /// **up to 6 function parameters**.
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
        /// **up to 6 function parameters**.
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
        /// **up to 6 function parameters**.
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
        /// **up to 6 function parameters**.
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
    );

    (@integers
        $(
            $unsafe:tt
            $RustInt:ident => $CInt:literal,
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
                    "<stdint.h>",
                    &mut |definer| write!(definer.out(),
                        "\n#include <stdint.h>\n\n",
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
        }}
        from_CType_impl_ReprC! { $RustInt }
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
        }}
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
        }}
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

impl_ReprC_for! { unsafe {
    bool
        => |ref byte: u8| (*byte & !0b1) == 0
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

    @for[T : ReprC]
    Option<ptr::NonNull<T>>
        => |ref it: *const T::CLayout| {
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for[T : ReprC]
    Option< ptr::NonNullRef<T> >
        => |ref it: *const T::CLayout| {
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for[T : ReprC]
    Option< ptr::NonNullMut<T> >
        => |ref it: *mut T::CLayout| {
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
    @for[T : ReprC]
    Option< ptr::NonNullOwned<T> >
        => |ref it: *mut T::CLayout| {
            (*it as usize) % ::core::mem::align_of::<T>() == 0
        }
    ,
}}
