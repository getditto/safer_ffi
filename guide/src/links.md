[comments]: <> (
    Try to keep all links and references within this file, to make it
    easier to maintain.
    Given how the rendered `book/` becomes the root of the published site,
    absolute paths can be used, _e.g._, when referring to other chapters:
    the `src/` directory can be seen as `/`.
)

[comments]: <> (
    EXTERNAL TOOLS
)
[`cbindgen`]: https://github.com/eqrion/cbindgen
[cargo-edit]: https://github.com/killercup/cargo-edit
[wasm_bindgen]: https://github.com/rustwasm/wasm-bindgen

[comments]: <> (
    EXTERNAL REFERENCES
)
[`niche-layout`]: https://rust-lang.github.io/unsafe-code-guidelines/glossary.html#niche
[rust-reference-fieldless-enums]: https://doc.rust-lang.org/stable/reference/items/enumerations.html#custom-discriminant-values-for-fieldless-enumerations
[parse-dont-validate]: https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/


[comments]: <> (
    DITTO LINKS
)
[Ditto]: https://www.ditto.live/about/company
[Ditto-logo]: /assets/ditto-logo-with-title-small.png

[comments]: <> (
    INTERNAL CHAPTERS OR SECTIONS
)
[usage]: /safer_ffi/usage/_.md
[ffi_export]: /safer_ffi/ffi-export/_.md
[derive_ReprC]: /safer_ffi/derive-reprc/_.md
[how-does-safer_ffi-work]: /safer_ffi/appendix/how-does-safer_ffi-work.md
[cargo-toml]: /safer_ffi/usage/cargo-toml.md
[header-generation]: /safer_ffi/usage/lib-rs.md#header-generation
[c-compilation]: /safer_ffi/appendix/c-compilation.md
[repr-c-forall]: /safer_ffi/motivation/repr-c-forall.md

[comments]: <> (
    RUST DOCUMENTATION
)
[Rust documentation]: /safer_ffi/rustdoc/safer_ffi/
[`ReprC`]: /safer_ffi/rustdoc/safer_ffi/layout/trait.ReprC.html
[`CType`]: /safer_ffi/rustdoc/safer_ffi/layout/trait.CType.html
[`repr_c::Box`]: /safer_ffi/rustdoc/safer_ffi/boxed/struct.Box.html
[`c_slice::Box`]: /safer_ffi/rustdoc/safer_ffi/slice/struct.slice_boxed.html
[`c_slice::Ref`]: /safer_ffi/rustdoc/safer_ffi/slice/struct.slice_ref.html
[`c_slice::Mut`]: /safer_ffi/rustdoc/safer_ffi/slice/struct.slice_mut.html
[`repr_c::Vec`]: /safer_ffi/rustdoc/safer_ffi/vec/struct.Vec.html
[`repr_c::String`]: /safer_ffi/rustdoc/safer_ffi/string/struct.String.html
[`str::Box`]: /safer_ffi/rustdoc/safer_ffi/string/struct.str_boxed.html
[`str::Ref`]: /safer_ffi/rustdoc/safer_ffi/string/struct.str_ref.html
[`char_p::Box`]: /safer_ffi/rustdoc/safer_ffi/char_p/struct.char_p_boxed.html
[`char_p::Ref`]: /safer_ffi/rustdoc/safer_ffi/char_p/struct.char_p_ref.html
[`safer_ffi::headers::builder()`]: /safer_ffi/rustdoc/safer_ffi/headers/struct.Builder.html
