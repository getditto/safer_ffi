using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Runtime.CompilerServices;
using System.Runtime.InteropServices;
using System.Text;
using FfiTests;

static class Tests
{
    public unsafe delegate R WithUTF8Continuation<R>(byte * _);

    public static R WithUTF8<R>(this string s, WithUTF8Continuation<R> f)
    {
        unsafe {
            int len;
            fixed (char * cp = s) {
                len = Encoding.UTF8.GetByteCount(cp, s.Length);
            }
            IntPtr p = Marshal.StringToCoTaskMemUTF8(s);
            var buf = (byte *)p;
            for (uint i = 0; i + 1 < len; ++i) {
                if (buf[i] == 0) {
                    throw new InvalidOperationException(
                        $"`WithUTF8()` does not support strings with inner null bytes: `{s}`"
                    );
                }
            }
            var ret = f((byte *)p);
            Marshal.FreeCoTaskMem(p);
            return ret;
        }
    }

    public unsafe delegate R WithSliceRefContinuation<R>(slice_ref_int32_t _);

    public static R WithSliceRef<R>(this Int32[] arr, WithSliceRefContinuation<R> f)
    {
        unsafe {
            fixed (Int32 * p = arr) {
                return f(new slice_ref_int32_t {
                    len = (UIntPtr)arr.Length,
                    ptr =
                        arr.Length > 0
                            ? (Int32 *)p
                            : (Int32 *)0xbad00
                });
            }
        }
    }

    static void Main(string[] _)
    {
        var s1 = "Hello, ";
        var s2 = "World!";
        // test concat
        unsafe {
            var s = s1.WithUTF8(p1 => s2.WithUTF8(p2 => {
                var p = Ffi.concat(p1, p2);
                var ret = Marshal.PtrToStringUTF8((IntPtr)p);
                Ffi.free_char_p(p);
                return ret;
            }));
            Trace.Assert(s == s1 + s2);
        }

        // test with_concat
        unsafe {
            var s = new List<string>();
            var handle = GCHandle.Alloc(s);

            [UnmanagedCallersOnly()]
            static unsafe void cb(void * ctx, byte /*const*/ * p) {
                GCHandle handle = GCHandle.FromIntPtr((IntPtr) ctx);
                var s = (List<string>)handle.Target;
                s.Add(Marshal.PtrToStringUTF8((IntPtr)p));
            }

            s1.WithUTF8(p1 => s2.WithUTF8(p2 => {
                Ffi.with_concat(
                    p1,
                    p2,
                    new RefDynFnMut1_void_char_const_ptr_t {
                        env_ptr = (void *)(IntPtr)handle,
                        call = &cb,
                    }
                );
                return 0;
            }));
            handle.Free();
            Trace.Assert(s[0] == s1 + s2);
        }

        // test max
        unsafe {
            Int32[] arr = { -27, -42, 9, -8 };
            arr.WithSliceRef(slice_ref => {
                Int32 * p = Ffi.max(slice_ref);
                Trace.Assert(p != null);
                Trace.Assert(*p == 9);
                return 0;
            });
        }

        // test max
        unsafe {
            int[] arr = {};
            arr.WithSliceRef(slice_ref => {
                Int32 * p = Ffi.max(slice_ref);
                Trace.Assert(p == null);
                return 0;
            });
        }

        // test foo
        unsafe {
            foo_t * foo = Ffi.new_foo();
            Trace.Assert(
                Ffi.read_foo(foo)
                ==
                42
            );
            Ffi.free_foo(foo);
            Ffi.free_foo(null);

            [UnmanagedCallersOnly()]
            static unsafe void cb(foo_t * foo) {
                Trace.Assert(
                    Ffi.read_foo(foo)
                    ==
                    42
                );
            }

            Ffi.with_foo(&cb);
        }

        // test constant
        unsafe {
            Trace.Assert(Ffi.FOO == 42);
        }

        // test the currified thing
        unsafe {
            Trace.Assert(Ffi.returns_a_fn_ptr()(0x42) == 0x4200);
        }

        unsafe {
            Trace.Assert(Ffi.my_renamed_ptr_api() == (void *)0xbad000);
        }

        Console.WriteLine("C#: [ok]");
    }
}
