/*! \file */
/*******************************************
 *                                         *
 *  File auto-generated by `::safer_ffi`.  *
 *                                         *
 *  Do not manually edit this file.        *
 *                                         *
 *******************************************/

#pragma warning disable IDE0044, IDE0049, IDE0055, IDE1006,
#pragma warning disable SA1004, SA1008, SA1023, SA1028,
#pragma warning disable SA1121, SA1134,
#pragma warning disable SA1201,
#pragma warning disable SA1300, SA1306, SA1307, SA1310, SA1313,
#pragma warning disable SA1500, SA1505, SA1507,
#pragma warning disable SA1600, SA1601, SA1604, SA1605, SA1611, SA1615, SA1649,

namespace FfiTests {
using System;
using System.Runtime.InteropServices;

public unsafe partial class Ffi {
    private const string RustLib = "ffi_tests";
}
public enum Triforce_t : byte {
    Din = 3,
    Farore = 1,
    Naryu,
}

public enum Wow_t : byte {
    Leroy,
    Jenkins,
}

public unsafe partial class Ffi {
    [DllImport(RustLib, ExactSpelling = true)] public static unsafe extern
    Int32 async_get_ft ();
}

public enum SomeReprCEnum_t {
    SomeVariant,
}

public unsafe partial class Ffi {
    [DllImport(RustLib, ExactSpelling = true)] public static unsafe extern
    void check_SomeReprCEnum (
        SomeReprCEnum_t _baz);
}

public enum Bar_t : byte {
    A,
}

public unsafe partial class Ffi {
    [DllImport(RustLib, ExactSpelling = true)] public static unsafe extern
    void check_bar (
        Bar_t _bar);
}

/** \brief
 *  Concatenate the two input strings into a new one.
 *
 *  The returned string must be freed using `free_char_p`.
 */
public unsafe partial class Ffi {
    [DllImport(RustLib, ExactSpelling = true)] public static unsafe extern
    byte * concat (
        byte /*const*/ * fst,
        byte /*const*/ * snd);
}

/** \brief
 *  Frees a string created by `concat`.
 */
public unsafe partial class Ffi {
    [DllImport(RustLib, ExactSpelling = true)] public static unsafe extern
    void free_char_p (
        byte * _string);
}

public struct foo_t {
   #pragma warning disable 0169
   private byte OPAQUE;
   #pragma warning restore 0169
}

public unsafe partial class Ffi {
    [DllImport(RustLib, ExactSpelling = true)] public static unsafe extern
    void free_foo (
        foo_t * foo);
}

[StructLayout(LayoutKind.Sequential, Size = 16)]
public unsafe struct slice_ref_int32_t {
    public Int32 /*const*/ * ptr;
    public UIntPtr len;
}

/** \brief
 *  Returns a pointer to the maximum integer of the input slice, or `NULL` if
 *  it is empty.
 */
public unsafe partial class Ffi {
    [DllImport(RustLib, ExactSpelling = true)] public static unsafe extern
    Int32 /*const*/ * max (
        slice_ref_int32_t xs);
}

public unsafe partial class Ffi {
    [DllImport(RustLib, ExactSpelling = true)] public static unsafe extern
    foo_t * new_foo ();
}

public unsafe partial class Ffi {
    [DllImport(RustLib, ExactSpelling = true)] public static unsafe extern
    Int32 read_foo (
        foo_t /*const*/ * foo);
}

[UnmanagedFunctionPointer(CallingConvention.Winapi)]
public unsafe /* static */ delegate
    void
    void_void_ptr_char_const_ptr_fptr (
        void * _0,
        byte /*const*/ * _1);

[StructLayout(LayoutKind.Sequential, Size = 16)]
public unsafe struct RefDynFnMut1_void_char_const_ptr_t {
    public void * env_ptr;
    [MarshalAs(UnmanagedType.FunctionPtr)]
    public void_void_ptr_char_const_ptr_fptr call;
}

/** \brief
 *  Same as `concat`, but with a callback-based API to auto-free the created
 *  string.
 */
public unsafe partial class Ffi {
    [DllImport(RustLib, ExactSpelling = true)] public static unsafe extern
    void with_concat (
        byte /*const*/ * fst,
        byte /*const*/ * snd,
        RefDynFnMut1_void_char_const_ptr_t cb);
}

[UnmanagedFunctionPointer(CallingConvention.Winapi)]
public unsafe /* static */ delegate
    void
    void_foo_ptr_fptr (
        foo_t * _0);

public unsafe partial class Ffi {
    [DllImport(RustLib, ExactSpelling = true)] public static unsafe extern
    void with_foo (
        [MarshalAs(UnmanagedType.FunctionPtr)]
        void_foo_ptr_fptr cb);
}


} /* FfiTests */
