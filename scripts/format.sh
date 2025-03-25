#!/usr/bin/env bash

set -euo pipefail

set -x
cd "$({ set +x; } 2>/dev/null; realpath "$(dirname "${0}")/formatting")"
{ set +x; } 2>/dev/null

unset RUSTUP_TOOLCHAIN ||:

# sed -i.bk -E 's/^(fmt =)/# \1/' ../../.cargo/config.toml
# trap 'mv ../../.cargo/config.toml{.bk,}' EXIT

(set -x; cargo fmt -- -V)

MANIFEST_PATHS=(
    ./Cargo.toml
    ./src/proc_macro/Cargo.toml
    ./run-sh/Cargo.toml
    ./js_tests/Cargo.toml
    ./ffi_tests/Cargo.toml
    ./safer-ffi-build/Cargo.toml
    ./napi-dispatcher/Cargo.toml
    ./napi-dispatcher/nodejs-derive/Cargo.toml
    ./napi-dispatcher/nodejs-derive/src/proc_macros/Cargo.toml
    ./napi-dispatcher/wasm/Cargo.toml
    ./napi-dispatcher/wasm/src/proc_macros/Cargo.toml
    ./examples/point/Cargo.toml
)
for manifest_path in "${MANIFEST_PATHS[@]}"; do
    (set -x; cargo fmt --manifest-path "../../${manifest_path}") &
done
wait
