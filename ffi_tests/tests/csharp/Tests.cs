using System;
using System.Runtime.InteropServices;
using System.Diagnostics;
using System.Text;

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

    public unsafe delegate R WithSliceRefContinuation<R>(FfiTests.slice_ref_int32_t _);

    public static R WithSliceRef<R>(this Int32[] arr, WithSliceRefContinuation<R> f)
    {
        unsafe {
            fixed (Int32 * p = arr) {
                return f(new FfiTests.slice_ref_int32_t {
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
                var p = FfiTests.concat(p1, p2);
                var ret = Marshal.PtrToStringUTF8((IntPtr)p);
                FfiTests.free_char_p(p);
                return ret;
            }));
            Trace.Assert(s == s1 + s2);
        }

        // test with_concat
        unsafe {
            string s = null;
            s1.WithUTF8(p1 => s2.WithUTF8(p2 => {
                FfiTests.with_concat(
                    p1,
                    p2,
                    new FfiTests.RefDynFnMut1_void_char_const_ptr_t { env_ptr = (void *) 0xbad00, call = (void * _,
                    byte * p) => {
                        s = Marshal.PtrToStringUTF8((IntPtr)p);
                    },
                });
                return 0;
            }));
            Trace.Assert(s == s1 + s2);
        }

        // test max
        unsafe {
            Int32[] arr = { -27, -42, 9, -8 };
            arr.WithSliceRef(slice_ref => {
                Int32 * p = FfiTests.max(slice_ref);
                Trace.Assert(p != null);
                Trace.Assert(*p == 9);
                return 0;
            });
        }

        // test max
        unsafe {
            int[] arr = {};
            arr.WithSliceRef(slice_ref => {
                Int32 * p = FfiTests.max(slice_ref);
                Trace.Assert(p == null);
                return 0;
            });
        }

        // test foo
        unsafe {
            FfiTests.foo_t * foo = FfiTests.new_foo();
            Trace.Assert(
                FfiTests.read_foo(foo)
                ==
                42
            );
            FfiTests.free_foo(foo);
            FfiTests.free_foo(null);
        }

        Console.WriteLine("[ok]");
    }
}
