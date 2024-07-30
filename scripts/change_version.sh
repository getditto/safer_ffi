#!/usr/bin/env bash

# Script to bump the `safer-ffi` version of the repo (both in `.toml` and `.lock` files).

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

usage() {
    cat >&2 <<-EOF
	Usage:
	    ${0} <new version number>
	Current version:
	    ${0} $(sed -nE 's/^version = "(.*)"\s*# Keep in sync.*/\1/p' src/proc_macro/Cargo.toml)
	EOF
    false
}

new_version="${1:?$(usage)}"

PACKAGES=(
    Cargo.toml
    src/proc_macro/Cargo.toml
)

for package in "${PACKAGES[@]}"; do
    (
        set -x
        sed -i.bak -E 's/^version = "(=?)(.*)"(\s*# Keep in sync)/version = "\1'"${new_version}"'"\3/' "${package}"
    )
    rm "${package}".bak
done

find . -type f -name 'Cargo.lock' -print0 \
| while IFS= read -r -d '' lockfile; do
    dir="$(dirname ${lockfile})"
    (
        set -x
        cd "${dir}" && cargo update -vw
    )
done
