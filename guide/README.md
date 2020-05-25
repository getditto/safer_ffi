# Editing, locally testing the guide

The guide is using [mdbook](https://github.com/rust-lang/mdBook).

### Local Development

1. Install mdbook with `cargo install mdbook`
2. Go to the guide directory: `cd guide`
3. Run `mdbook serve` and go to http://localhost:3000/ and edit the __.md__ files in guide/src
4. The browser will refresh with any changes

### Building Publishable HTML

1. `cd guide` and run `mdbook build`
2. The final HTML, CSS etc... will be in the guide/book directory
