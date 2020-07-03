using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Text;

namespace RustMarshaller {
    class UTF8 : ICustomMarshaler
    {
        private int direction;

        private UTF8(int direction) // : direction(direction_)
        {
            this.direction = direction;
        }

		public static ICustomMarshaler GetInstance (string cookie)
		{
            if (String.IsNullOrEmpty(cookie)) {
                return null;
            }
            return new UTF8(cookie == "in" ? 1 : 0);
		}

		public int GetNativeDataSize ()
		{
			return IntPtr.Size;
		}

		public IntPtr MarshalManagedToNative (object obj)
		{
            Console.WriteLine($"MarshalManagedToNative ({obj})");
			string s = obj as string;
			if (s == null) {
                return IntPtr.Zero;
            }
            unsafe {
                fixed (char * p = s) {
                    var len = Encoding.UTF8.GetByteCount(p, s.Length);
                    Console.WriteLine($"    len = {len}");
                    var c = (byte *) Marshal.AllocHGlobal(len + 1).ToPointer();
                    Console.WriteLine("    c = 0x{0}", ((IntPtr) c).ToString("x"));
                    var bytesWritten = Encoding.UTF8.GetBytes(p, s.Length, c, len);
                    Trace.Assert(len == bytesWritten);
                    for (uint i = 0; i + 1 < len; ++i) {
                        if (c[i] == 0) {
                            Marshal.FreeHGlobal((IntPtr) c);
                            return IntPtr.Zero;
                        }
                    }
                    c[len] = 0;
                    return (IntPtr) c;
                }
            }
		}

		public void CleanUpNativeData (IntPtr pNativeData)
		{
            if (direction != 1) { return; }
            Console.WriteLine("CleanUpNativeData(0x{0})", pNativeData.ToString("x"));
            if (pNativeData != IntPtr.Zero) {
                Marshal.FreeHGlobal(pNativeData);
                pNativeData = IntPtr.Zero;
            }
		}

        [DllImport(foo.Ffi.Name, EntryPoint = "__safer_ffi_helper_free_string")]
        private unsafe extern static void rust_free_string (byte * p);

		public object MarshalNativeToManaged (IntPtr pNativeData)
		{ unsafe {
            Console.WriteLine("MarshalNativeToManaged(0x{0})", pNativeData.ToString("x"));
            var p = (byte *) pNativeData;
            int byteCount = 0; while(p[byteCount++] != 0);
            Console.WriteLine($"    byteCount = {byteCount}");
            var ret = Encoding.UTF8.GetString(p, byteCount);
            if (direction != 1) {
                rust_free_string((byte *) pNativeData);
            }
            return ret;
		}}

		public void CleanUpManagedData (object _)
		{}
    }
}

class Ptr {
    public
    IntPtr untyped;

    public
    Ptr(IntPtr untyped) {
        this.untyped = untyped;
    }

    public
    class Byte : Ptr {
        public Byte(IntPtr untyped) : base(untyped) {}
    }

    public
    class Marshaler : ICustomMarshaler {
        private static Marshaler instance = new Marshaler();
        private Marshaler () {}

        public static ICustomMarshaler GetInstance (string _)
        {
            return instance;
        }

		public int GetNativeDataSize ()
		{
			return IntPtr.Size;
		}

        public IntPtr MarshalManagedToNative (object obj)
        {
            var ptr = obj as Ptr;
            if (ptr == null) {
                throw new MarshalDirectiveException(
                    "`PtrMarshaler` requires must be used on a Ptr type"
                );
            }
            return ptr.untyped;
        }

		public void CleanUpNativeData (IntPtr pNativeData)
		{}

		public object MarshalNativeToManaged (IntPtr pNativeData)
		{
            return new Ptr(pNativeData);
		}

		public void CleanUpManagedData (object _)
		{}
    }
}

namespace foo
{
    static partial class Ffi
    {
        public const string Name = "rust_lib";

        [DllImport(Name)]
        public extern static Int32 add(Int32 x, Int32 y);

        [StructLayout(LayoutKind.Sequential)]
        public struct Point
        {
            public float x;
            public float y;
        }


        [DllImport(Name)]
        [return: MarshalAs(UnmanagedType.U1)]
        public extern static Boolean new_Point(out Point p);

        public unsafe static string MarshalNativeUtf8ToManagedString (byte * c_str)
        {
            var byteCount = 0; while (c_str[byteCount++] != 0);
            return Encoding.UTF8.GetString(c_str, byteCount);
        }

        // // C# raw header translation of the C def:
        [DllImport(Name)]
        public unsafe extern static byte * concat(
            byte * s1,
            byte * s2);

        public unsafe delegate R WithUTF8Continuation<R>(byte * _);

        public static R WithUTF8<R> (this string s, WithUTF8Continuation<R> f)
        {
            var len = 0;
            unsafe {
                fixed (char * cp = s) {
                    len = Encoding.UTF8.GetByteCount(cp, s.Length);
                }
            }
            unsafe {
                IntPtr p = Marshal.StringToCoTaskMemUTF8(s);
                var buf = (byte *) p;
                for (uint i = 0; i + 1 < len; ++i) {
                    if (buf[i] == 0) {
                        throw new InvalidOperationException(
                            $"`WithUTF8()` does not support strings with inner null bytes: `{s}`"
                        );
                    }
                }
                var ret = f((byte *) p);
                Marshal.FreeCoTaskMem(p);
                return ret;
            }
        }

        // Idiomatic wrapper
        public static string concat(
            string s1,
            string s2)
        {
            unsafe {
                return s1.WithUTF8(p1 => s2.WithUTF8(p2 => {
                    byte * s = concat(p1, p2);
                    return Marshal.PtrToStringUTF8((IntPtr) s);
                }));
            }
        }

        // [DllImport(Name)]
        // [return : MarshalAs(UnmanagedType.CustomMarshaler
        //     , MarshalTypeRef = typeof(RustMarshaller.UTF8)
        //     , MarshalCookie = "out"
        // )]
        // public unsafe extern static byte * concat(
        //     [MarshalAs(UnmanagedType.CustomMarshaler
        //         , MarshalTypeRef = typeof(RustMarshaller.UTF8)
        //         , MarshalCookie = "in"
        //     )]
        //     string s1,
        //     [MarshalAs(UnmanagedType.CustomMarshaler
        //         , MarshalTypeRef = typeof(RustMarshaller.UTF8)
        //         , MarshalCookie = "in"
        //     )]
        //     string s2);

        [DllImport(Name)]
        public unsafe extern static void free_string(byte * s);

        // public static string concat (string s1, string s2)
        // { unsafe {
        //     byte * c1;
        //     fixed (char * p1 = s1) {
        //         var len = Encoding.UTF8.GetByteCount(p1, s1.Length);
        //         c1 = (byte *) Marshal.AllocHGlobal(len + 1).ToPointer();
        //         var bytesWritten = Encoding.UTF8.GetBytes(p1, s1.Length, c1, len);
        //         Trace.Assert(len == bytesWritten);
        //         c1[len] = 0;
        //     }
        //     byte * c2;
        //     fixed (char * p2 = s2) {
        //         var len = Encoding.UTF8.GetByteCount(p2, s2.Length);
        //         c2 = (byte *) Marshal.AllocHGlobal(len + 1).ToPointer();
        //         var bytesWritten = Encoding.UTF8.GetBytes(p2, s2.Length, c2, len);
        //         Trace.Assert(len == bytesWritten);
        //         c2[len] = 0;
        //     }
        //     // var c_ret = concat(c1, c2);
        //     // Marshal.FreeHGlobal((IntPtr)c1);
        //     // Marshal.FreeHGlobal((IntPtr)c2);
        //     // var ret = MarshalNativeUtf8ToManagedString(c_ret);
        //     // free_string(c_ret);
        //     // OR:
        //     string ret = null;
        //     RefDynFnMut1_void_char_ptr cb;
        //     cb.ctx = (IntPtr) 0xbad00;
        //     cb.call = (IntPtr _, byte * c_ret) => {
        //         Marshal.FreeHGlobal((IntPtr)c1);
        //         Marshal.FreeHGlobal((IntPtr)c2);
        //         ret = MarshalNativeUtf8ToManagedString(c_ret);
        //     };
        //     Console.WriteLine(Marshal.OffsetOf<RefDynFnMut1_void_char_ptr>("call"));
        //     Console.WriteLine(Marshal.SizeOf<RefDynFnMut1_void_char_ptr>());
        //     with_concat(c1, c2, cb);
        //     return ret;
        // }}

        private unsafe delegate void WithConcatByteP(IntPtr ctx, byte * concat);

        [StructLayout(LayoutKind.Sequential)]
        private struct RefDynFnMut1_void_char_ptr {
            public IntPtr ctx;

            [MarshalAs(UnmanagedType.FunctionPtr)]
            public WithConcatByteP call;

        }

        [DllImport(Name)]
        private unsafe extern static void with_concat(
            byte * s1,
            byte * s2,
            RefDynFnMut1_void_char_ptr cb);
    }

    class Program
    {

        static void Main(string[] args)
        {
            Int32 x = 42;
            Int32 y = 27;
            Int32 S = Ffi.add(x, y);
            Console.WriteLine($"{x} + {y} = {S}");

            Ffi.Point p;
            Boolean b = Ffi.new_Point(out p);
            Debug.Assert(b);
            Console.WriteLine($"Point {{ x: {p.x}, y: {p.y} }}");

            string s = Ffi.concat("Hello, ", "World!");
            // Ffi.concat("Hell\0, ", "World!");
            Console.WriteLine(s);

            foreach (var Name in Ffi.Names())
            {
                Console.WriteLine($"'{Name}'");
            }
        }
    }

    partial class Ffi
    {
        [StructLayout(LayoutKind.Sequential)]
        private unsafe struct RustString
        {
            public byte * ptr;
            public IntPtr len;
            public IntPtr capacity;
        }

        [StructLayout(LayoutKind.Sequential)]
        private unsafe struct Vec_RustString
        {
            public RustString * ptr;
            public IntPtr len;
            public IntPtr capacity;
        }

        [DllImport(Name, EntryPoint = "names")]
        private extern static Vec_RustString rust_names ();

        [DllImport(Name)]
        private extern static void free_names (Vec_RustString vec);

        public static string[] Names ()
        { unsafe {
            var vec = rust_names();
            var arr = new string[(int) vec.len];
            for (int i = 0; i < arr.Length; ++i) {
                arr[i] = Encoding.UTF8.GetString(vec.ptr[i].ptr, (int) vec.ptr[i].len);
            }
            free_names(vec);
            return arr;
        }}
    }
}
