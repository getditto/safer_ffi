#!/usr/bin/env bash

# Whilst running `./scripts/format.sh` does work quite well, and
# can be handy for CI, the reality is that most IDEs and setups
# expect `cargo fmt` to Just Work™.
#
# But the usage of `cargo fmt` on this repository relies on `unstable`
# features, which in turn require *versioned nightly* for it to be
# reliable.
#
# Since even now, in 2025, there is still no `1.85.0-nightly`, we end up using:
#     RUSTC_BOOTSTRAP=1 cargo +{MSRV} …
# for `rustc`-specific unstable features,
#
# But the unstable features of Cargo-specific-tooling such as `cargo fmt` do
# require an actual/genuine `nightly` toolchain.
#
# That's why, instead, we just look up which nightly coincides with
# our current stable the best (e.g., using https://releases.rs), so as
# to pin a `nightly` toolchain matching our stable MSRV one
# (e.g. for `{MSRV}-stable=1.85.0`, we have `{MSRV}-nighly=nightly-2025-01-03`).
#
#   - the MSRV one being the one defined at the top-level `rust-toolchain.toml`
#     file.
#
# Hence the `rust-toolchain.toml` file in this very directory.
#
# But for `cargo fmt` to just work, we do need the `{MSRV}-nightly`
# `rustfmt` to be invoked when `rustup` looks up the one over
# `{MSRV}-stable`.
#
# We can achieve this through a "hack": overwriting the local `rustup` setup of
# `{MSRV}-stable`'s `rustfmt` with that of `{MSRV}-nightly`.

set -euo pipefail

if [[ ${IN_NIX_SHELL+x} != "" ]]; then
    # If nix: OK.
    echo >&2 'Refusing to attempt `rustup` shenanigans when inside a Nix setup.'
    false
fi

SAFER_FFI_DIR="$(realpath "$(dirname "${0}")/../..")"

set -x

cd "${SAFER_FFI_DIR}"

TOOLCHAINS_DIR="$(rustup show home)/toolchains"
HOST_ARCH="$(rustc --print host-tuple)"
({ set +x; } 2>/dev/null; echo)

default_toolchain="$(sed -nE 's/^channel = "(.*)".*/\1/p' ./rust-toolchain.toml)"
(
    (
        { set +x; } 2>/dev/null
        echo "🏗️ Installing appropriate toolchain if needed."
    )
    cd .
    cargo -V # This also installs/updates the default toolchain if needed
    (
        { set +x; } 2>/dev/null
        echo -e "🆗\n"
    )
)
rustfmt_toolchain="$(sed -nE 's/^channel = "(.*)".*/\1/p' ./scripts/formatting/rust-toolchain.toml)"
(
    (
        { set +x; } 2>/dev/null
        echo "🏗️ Installing appropriate toolchain if needed."
    )
    cd scripts/formatting
    cargo -V # Ditto
    (
        { set +x; } 2>/dev/null
        echo -e "🆗\n"
    )
)
{ set +x; } 2>/dev/null

echo -e "> 🛡️ Backing up ${default_toolchain}'s \`rustfmt\` 🛡️ <"
(
    set -x
    cp -n "${TOOLCHAINS_DIR}/${default_toolchain}-${HOST_ARCH}/bin/rustfmt"{,.bkup}
) || true
echo -e "🆗\n"

# Note: as the message indicates, this does overwrite our stable toolchain's rustfmt
# with a nightly one. Since a stable toolchain is only able to use the stable API
# of rustfmt, it means this from-the-future rustfmt will result in the same visible
# behavior (on condition that their stability promise hold).
echo -e "> 🚧 Overriding ${default_toolchain}'s \`rustfmt\` with ${rustfmt_toolchain}'s 🚧 <"
(
    set -x
    ln -sf \
        "${TOOLCHAINS_DIR}/${rustfmt_toolchain}-${HOST_ARCH}/bin/rustfmt" \
        "${TOOLCHAINS_DIR}/${default_toolchain}-${HOST_ARCH}/bin/rustfmt" \
        ;
)
ls -lhF "${TOOLCHAINS_DIR}/${default_toolchain}-${HOST_ARCH}/bin/rustfmt"
echo -e "🆗\n"

cat <<EOF
💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡
Should you wish to revert this simply run the following command:
    ln -sf  \\
        "${TOOLCHAINS_DIR}/${default_toolchain}-${HOST_ARCH}/bin/rustfmt.bkup" \\
        "${TOOLCHAINS_DIR}/${default_toolchain}-${HOST_ARCH}/bin/rustfmt" \\
    ;
💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡💡

EOF

echo -e "> 🕵 Sanity check 🕵 <"
if (
    set -x
    [ "$(cargo fmt -- -V)" != "$(cargo +${rustfmt_toolchain} fmt -- -V)" ]
); then
    echo "❌"
    false
fi

cat <<EOF
✅ All done! ✅

Thenceforth, \`cargo fmt\` will Just Work™ whilst being able to use \
\`nightly\` features! 🥳👌
EOF
