# `#[derive_ReprC(dyn, …)]` usage

## Summary

Given a _simple_ `Trait` definition with **signatures involving `ReprC` types _exclusively_**, and using `Traits` as syntax for `'usability + Trait $(+ Send)? $(+ Sync)?`, then:

 1. annotating it with `#[derive_ReprC(dyn)]`

    (or `#[derive_ReprC(dyn, Clone)])` when "`Clone`-annotating"),

    ```rust
    #[derive_ReprC(dyn, /* Clone */)]
    trait Trait /* : Send + Sync */ {
    # }
    ```

 1. makes `dyn Traits : ReprCTrait`, which, in turn,

 1. makes `VirtualPtr<dyn Traits>` become a legal/nameable type, so that:

      - `VirtualPtr<dyn Traits> : Traits`,

      - `VirtualPtr<dyn Traits> : ReprC` and thus, [FFI-compatible],

      - `VirtualPtr<dyn Traits> : From<Box<impl Traits>>` [and so on for the other most pervasive Rust "smart" pointer types][construction],

      - In the case of `#[derive_ReprC(dyn, Clone)]`, we'll also have:

          - `VirtualPtr<dyn Traits> : Clone`,

          - `From<{A,}Rc<impl Traits>>`,

        But at the cost of losing the `From` impls for `&mut` and `Box<impl !Clone>`.

      - `From<&impl Traits>` (and `From<{A,}Rc<impl Traits>>`) will only be available when there are no `&mut self` methods in the trait definition.

[FFI-compatible]: virtual_ptr.md#its-ffi-layout-constructing-and-using-virtualptr-from-the-ffi

[construction]: virtual_ptr.md#constructing-a-virtualptr-from-rust

### A _simple_ `Trait` definition?

A `Trait` definition is deemed _simple_ if:

  - it only has _methods_ in it (`fn method(self: …, …)`);
  - is `dyn`-safe (no generics, no `where Self : Sized`);
  - with only `&Self` and `&mut Self` receiver types (the owned case is not supported yet).
      - `Pin<>`-wrapping them is, however, accepted (thereby making the `From` impls require it).
