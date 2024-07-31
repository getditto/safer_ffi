#!/usr/bin/env bash

# Script to cargo publish a new version of safer-ffi

set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

(set -x
    git status >&2
)
if [ -n "$(git status --porcelain)" ]; then
    echo >&2 "❌ Uncommitted changes detected."
    false
fi

current_version="$(sed -nE 's/^version = "(.*)"\s*# Keep in sync.*/\1/p' src/proc_macro/Cargo.toml)"
(set -x
    current_version="${current_version}"
)

echo -n "Version desired? [${current_version}] "
read -r desired_version

set -x
desired_version="${desired_version:-$current_version}"
{ set +x; } 2>/dev/null

(set -x
    ./scripts/change_version.sh "${desired_version}"
)

if [ -n "$(git status --porcelain)" ]; then
    git add -u
    (set -x
        git commit -m "Version \`${desired_version}\` release"
    )
fi
(set -x
    git push
)

PACKAGES=(
    src/proc_macro/
    ./
)
for package in "${PACKAGES[@]}"; do
    (set -x
        cd "${package}"
        cargo publish
    )
done
