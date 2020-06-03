{{#include ../links.md}}

# Deriving `ReprC` for custom enums

## C enums

A C enum is a field-less enum, _i.e._, an enum that only has _unit_ variants.

<details><summary>Examples</summary>

```rust,noplaypen
enum Ordering {
    Less    /* = 0 */,
    Equal   /* = 1 */,
    Greater /* = 2 */,
}

/// The following discriminants are all guaranteed to be > 0.
enum ErrorKind {
    NotFound            = 1,
    PermissionDenied /* = 2 */,
    TimedOut         /* = 3 */,
    Interrupted      /* = 4 */,
    Other            /* = 5 */,
    // ...
}
```

</details>

See [the reference for more info about them][rust-reference-fieldless-enums].

### Usage

```rust,noplaypen
use ::safer_ffi::prelude:*;

#[derive_ReprC] // <- `::safer_ffi`'s attribute
#[repr(u8)]     // <- explicit integer `repr` is mandatory!
pub
enum LogLevel {
    Off = 0,    // <- explicit discriminants are supported
    Error,
    Warning,
    Info,
    Debug,
}
```

<details><summary>Generated C header</summary>

```c
typedef uint8_t LogLevel_t; enum {
    LOGLEVEL_OFF = 0,
    LOGLEVEL_ERROR,
    LOGLEVEL_WARNING,
    LOGLEVEL_INFO,
    LOGLEVEL_DEBUG,
};
```

</details>

### Layout of C enums

These enums are generally used to define a _closed_ set of _distinct_ integral
constants in a _type-safe_ fashion.

But when used from C, the type safety is kind of lost, given how loosely C
converts back and forth between `enum`s and integers.

This leads to a very important point:

> What is the integer type of the enum discriminants?

With **no** `#[repr(...)]` annotation whatsoever, Rust reserves the right to
choose whatever it wants: no defined C layout, so **not FFI-safe**.

With `#[repr(Int)]` (where `Int` can be `u8`, `i8`, `u32`,
_etc._) Rust is forced to use that very `Int` type.

With `#[repr(C)]`, Rust will pick what C would pick if it were given an
equivalent definition.

<span class="warning">

`#[repr(C)]` enums can cause UB when used across FFI ⚠️

</span>

<details><summary>Click for more info</summary>
It turns out C itself does not really define a concrete integer layout
for its enums. Indeed, the C standard only states that:

  - the discriminants are `int`s.

  - the enum itself represents an integer type that must fit in an `int`.

      - Very often this is an `int` too.

      - but since there is no explicit guarantee that it must be _exactly_
        an `int` too, [compiler flags such as `-fshort-enums` can lead to
        smaller integer types](https://oroboro.com/short-enum/).

        This means that when you link against a library that was compiled
        with a different set of flags, such as a system-wide shared library
        or a Rust generated `staticlib` / `cdylib`, then such mismatch is
        very likely to cause Undefined Behavior!

In practice, when C defines an `enum` to be used by Rust, there is no other
choice but to use `#[repr(C)]` and pray / ensure that the C library is compiled
with the same semantics that Rust expects (_e.g._, no `-fshort-enums` flag).

But when doing FFI in the other direction, there is no reason whatsoever to use
`#[repr(C)]`: **picking a fixed-size integer is then the most sensible thing to
do for a well-defined and thus robust FFI interface**.

</details>

That's why [`#[derive_ReprC]`][derive_ReprC] makes the opinionated choice of
**refusing to handle an `enum` definition that does not provide an
explicit fixed-size integer representation**.

## More complex enums

<span class="warning">

Are not supported yet.

</span>
