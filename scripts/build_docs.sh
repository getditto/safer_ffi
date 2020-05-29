#!/bin/bash

set -euxo pipefail

BASE_DIR="$(git rev-parse --show-toplevel)"

cd $BASE_DIR
cargo +nightly doc --all-features
(cd guide
    mdbook build
    mkdir -p book/{assets,rustdoc}
    cp -r assets/* book/assets/
    cp -r ../target/doc/* book/rustdoc/
)

