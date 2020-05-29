{{#include ../links.md}}

# Appendix: how does `safer_ffi` work

Most of the limitations of traditional FFI are related to the design of
`cbindgen` and its being **implemented as a syntactic tool**: without access to
the semantics of the code, only its representation, `cbindgen` will never truly
be able to overcome these issues.

Instead, a tool for true FFI integration, including header generation, needs to
**have a way to interact with the high-level code and type semantics created by
the compiler**, instead of just the original source code.

There are two ways to achieve this (outside official compiler support, of
course):

  - either through a compiler plugin, such as `clippy`. This requires a very
    advanced knowledge of unstable compiler implementation details and
    internals, thus leading to a high maintenance burden: every new release
    of Rust could break such a compiler plugin (_c.f._ `clippy`-incompatible
    `nightly` Rust releases).

  -  by encoding invariants and reflection within the type system, through a
    complex but stable use of helper traits. **This is the choice made by
    `safer_ffi`**, whereby _two_ traits suffice to express the necssary semantics
    for FFI compatibility and integration:

      - the user-facing [`ReprC`] trait, implemented for types having a defined
        C layout:

          - either directly provided by the `safer_ffi` crate (_c.f._ [its
            dedicated chapter][repr-c-forall]),

          - or implemented for custom types having the
            [`#[derive_ReprC]`][derive_ReprC] attribute.

          - this is the trait that [`[ffi_export]`][ffi_export]
            _directly_ relies on.

      - the more avdanced raw [`CType`] trait, that you can simply dismiss as
        an internal trait.

          - Still, for those interested, know that it defines the necessary
            logic for C reflection, that you can, by the way, `unsafe`-ly
            implement for your custom definitions for an **advanced but doable
            custom extension of the framework**!
