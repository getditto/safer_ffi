{{#include ../links.md}}

# Up until now: Rust→C FFI using `cbindgen`

It is true that Rust→C FFI was already doable with `#[no_mangle]` and [`cbindgen`]:

  - Instead of:

    ```rust,noplaypen
    use ::repr_c::prelude::*;

    #[derive(ReprC)]
    #[repr(C)]
    pub
    struct Point {
        x: i32,
        y: i32,
    }

    #[ffi_export]
    fn origin () -> Point
    {
        Point { x: 0, y: 0 }
    }
    ```

  - one could already just write:

    ```rust,noplaypen
    #[repr(C)]
    pub
    struct Point {
        x: i32,
        y: i32,
    }

    #[no_mangle] pub extern "C"
    fn origin () -> Point
    {
        Point { x: 0, y: 0 }
    }
    ```

    and then use [`cbindgen`] to generate the bindings.

But it turns out that [`cbindgen`] doesn't really make the cut.

  - at least that has been the case in the company I work for,
    [![Ditto-Logo]][Ditto]

    See [the dedicated chapter](../ditto/_.md) for more info about it.

Indeed, as of this writing (`cbindgen 0.14.2`):

  - [`cbindgen`] does not support more complex types and their layout / ABI
    semantics (_e.g._, `Option<ptr::NonNull<i32>>` _vs._ `*const i32`);

      - For instance,

        ```rust,noplaypen
        #[no_mangle] pub extern "C"
        fn my_free (ptr: Box<i32>)
        {
            drop(ptr)
        }
        ```

        generates:

        ```C
        typedef struct Box_i32 Box_i32;

        void my_free(Box_i32 ptr);
        ```

        instead of the expected:

        ```C
        void my_free(int32_t * ptr);
        ```

      - this means that the moment [you want to use types to express properties
        and invariants](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/
        ), you quickly stumble upon this limitation, meaning
        that all the Rust→C FFI code out there is always using "flat" raw
        pointers; which in turn also results in the `unsafe` implementations
        being more error-prone (_e.g._, how can we be sure that all the pointer
        dereferences have been guarded against the `NULL` case, if not by
        ensuring that all the pointer types are `Option<_>`-wrapped?)

  - [`cbindgen`] currently operates at a syntactic level:
    the moment the signatures get more involved (_e.g._, (procedural) macros expanding to `#[no_mangle]` functions or type definitions), it starts missing stuff.

      - For instance, the following code:

        ```rust,noplaypen
        macro_rules! adders {(
            $(
                $T:ty => $add_T:ident,
            )*
        ) => (
            $(
                #[no_mangle] pub extern "C"
                fn $add_T (x: $T, y: $T) -> $T
                {
                    x.wrapping_add(y)
                }
            )*
        )}

        adders! {
            u8  => add_uint8,
            i8  => add_int8,
            u16 => add_uint16,
            i16 => add_int16,
            u32 => add_uint32,
            i32 => add_int32,
            u64 => add_uint64,
            i64 => add_int64,
        }
        ```

        does not output anything with [`cbindgen`].

      - Worse, this gets really **dangerous** the moment `cbindgen` syntactic
        heuristics fail to see what Rust is referring to:

        ```rust,noplaypen
        /// Let's imagine that for whatever reason we want to have our own
        /// `Option` type, one that will always have a defined C layout, in
        /// exchange of opting out of niche layout optimization.
        /// (https://rust-lang.github.io/unsafe-code-guidelines/glossary.html#niche)
        #[repr(C)]
        pub
        struct Option<T> {
            is_some: bool,
            value: ::core::mem::MaybeUninit<T>,
        }

        mod ffi_functions {
            use super::*; // <- This is what `cbindgen` currently struggles with

            #[no_mangle] pub extern "C"
            fn with_my_option (my_opt: Option<&'_ i32>) -> i8
            {
                if my_opt.is_some {
                    let value: &'_ i32 = unsafe { my_opt.value.assume_init() };
                    println!("{}", *value);
                    0
                } else {
                    -1
                }
            }
        }
        ```

        generates (with no warnings whatsoever):

        ```C
        void with_my_option(const int32_t *_it);
        ```

        that is, ⚠️ a straight up incorrect function signature ⚠️
