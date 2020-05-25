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
[Ditto-logo]: /assets/ditto-small.png

[comments]: <> (
    INTERNAL CHAPTERS OR SECTIONS
)
[usage]: /usage/_.md
[ffi_export]: /ffi-export/_.md
[derive_ReprC]: /derive-reprc/_.md
[how-does-repr_c-work]: /appendix/how-does-repr_c-work.md
[cargo-toml]: /usage/cargo-toml.md
[header-generation]: /usage/lib-rs.md#header-generation
[c-compilation]: /appendix/c-compilation.md
[repr-c-forall]: /motivation/repr-c-forall.md

[comments]: <> (
    RUST DOCUMENTATION
)
[Rust documentation]: /rustdoc/repr_c/
[`ReprC`]: /rustdoc/repr_c/layout/trait.ReprC.html
[`CType`]: /rustdoc/repr_c/layout/trait.CType.html
[`repr_c::Box`]: /rustdoc/repr_c/boxed/struct.Box.html
[`c_slice::Box`]: /rustdoc/repr_c/slice/struct.slice_boxed.html
[`c_slice::Ref`]: /rustdoc/repr_c/slice/struct.slice_ref.html
[`c_slice::Mut`]: /rustdoc/repr_c/slice/struct.slice_mut.html
[`repr_c::Vec`]: /rustdoc/repr_c/vec/struct.Vec.html
[`repr_c::String`]: /rustdoc/repr_c/string/struct.String.html
[`str::Box`]: /rustdoc/repr_c/string/struct.str_boxed.html
[`str::Ref`]: /rustdoc/repr_c/string/struct.str_ref.html
[`char_p::Box`]: /rustdoc/repr_c/char_p/struct.char_p_boxed.html
[`char_p::Ref`]: /rustdoc/repr_c/char_p/struct.char_p_ref.html
[`repr_c::headers::builder()`]: /rustdoc/repr_c/headers/struct.Builder.html
