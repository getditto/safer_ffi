{{#include ../links.md}}

# Idiomatic Rust types in FFI signatures?

That was the main objective when creating and using `::safer_ffi`:

Why go through **the dangerously `unsafe` hassle** of:

  - using `ptr: *const/mut T, len: usize` pairs when wanting to use slices?

  - using `*const c_char` and `*mut c_char` and `CStr / CString / String`
    dances when wanting to use strings?

  - losing all kind of ownership-borrow information with signatures such as:

    ```rust,noplaypen
    // Is this taking ownership of `Foo` or "just" mutating it?
    #[unsafe(no_mangle)] pub unsafe extern "C"
    fn foo_stuff (foo: *mut Foo)
    {
        /* ... */
    }
    ```

> Can't we use our good ol' idiomatic `&/&mut/Box` trinity types? And some
equivalent to `[_]` slices, `Vec`s and `String`s? And _quid_ of closure
types?

To which the answer is _yes!_ All these types can be FFI-compatible,
**provided they have a defined C layout**. And this is precisely what `safer_ffi` does:

> `safer_ffi` defines a bunch of idiomatic Rust types with a defined `#[repr(C)]`
  layout, to get both FFI compatibility and non-`unsafe` ergonomics.

That is, for any type `T` that has a defined C layout, _i.e._, that is
[`ReprC`] (and `Sized`):

  - `&'_ T` and `&'_ mut T` are themselves [`ReprC`]!

    - Same goes for [`repr_c::Box`]`<T>`.

    - They all have the C layout of a (non-nullable) raw pointer.

    - And all three support being `Option`-wrapped (the layout remains that
      of a (now nullable) raw pointer, thanks to the
      [enum layout optimization][`niche-layout`])

  - [`c_slice::Ref`]`<'_, T>` and [`c_slice::Mut`]`<'_, T>` are
    also [`ReprC`] equivalents of `&'_ [T]` and `&'_ mut [T]`.

    - Same goes for [`c_slice::Box`]`<T>` (to represent `Box<[T]>`).

    - They all have the C layout of a `struct` with a `.ptr`, `.len` pair
      of fields (where `.ptr` is non-nullable).

    - And all three support being `Option`-wrapped too.

       1. In that case, the `.ptr` field becomes nullable;

       1. when it is `NULL`, the `.len` field can be uninitialized: ⚠️ it is
          thus then UB to read the `.len` field ⚠️ (type safety and encapsulation ensure this UB cannot be
          triggered from within Rust; only the encapsulation-deprived C side
          can do that).

  - There is [`repr_c::Vec`]`<T>` as well (extra `.capacity` field _w.r.t._
    generalization to [`c_slice::Box`]`<T>`),

  - as well as [`repr_c::String`]!

    - with the slice versions (`.capacity`-stripped) too: [`str::Box`] and
      [`str::Ref`]`<'_>` (`Box<str>` and `&'_ str` respectively).

    - although these definitions are capable of representing any sequence of
      UTF-8 encoded strings (thus supporting NULL bytes), since the C world is
      not really capable of handling those (except as opaque blobs of bytes),
      `char *`-compatible null-terminated UTF-8 string types are available
      as well:

        - [`char_p::Ref`]`<'_>` for `char const *`: a temporary borrow of such
          string (useful as input parameter).

        - [`char_p::Box`] for `char *`: a pointer owning a `Box`-allocated
          such string (useful to _return_ strings).
