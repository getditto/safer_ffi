#!/bin/bash

set -euxo pipefail

BASE_DIR="$(git rev-parse --show-toplevel)"

cd $BASE_DIR
cargo +nightly doc --all-features
(cd guide
    (cd src && sed -e "s#{ROOT_PATH}#${1-/}#g" links.md.template > links.md)
    mdbook build
    mkdir -p book/{assets,rustdoc}
    cp -r assets/* book/assets/
    cp -r ../target/doc/* book/rustdoc/
)
