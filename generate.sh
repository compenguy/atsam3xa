#!/usr/bin/env bash

# Bash script to (re)generate Platform Access Crates (PACs) with svd2rust.
# For PAC to be generated, the following items are required:
#
# - It needs to be registered below in the `PACS` array.
# - The directory needs to contain an SVD file with the name matching
#   the crate name.

set -euo pipefail

CHIP="ATSAM3X8E"

# everything is relative to the generate script
cd "$(dirname "$0")"


die() {
    echo "$1"
    exit 1
}

command -v cargo-fmt >/dev/null 2>&1 || die "Missing command 'cargo-fmt'"
command -v svd2rust >/dev/null 2>&1 || cargo install --force \
        --git https://github.com/rust-embedded/svd2rust.git \
        --branch master svd2rust || die "Failed to install 'svd2rust'"
command -v form >/dev/null 2>&1 cargo install --force \
        --version 0.7.0 form || die "Failed to install 'form'"

### Main

echo "Running svd2rust..."
[ -d src ] && rm -rf src
svd2rust "$@" -i "${CHIP}.svd" 2> >(tee svd2rust-warnings.log >&2)
RUST_LOG=form=warn form -i lib.rs -o src
[ -f lib.rs ] && rm lib.rs
echo "Formatting generated code..."
cargo fmt
rustfmt build.rs
echo ""

# Patch SVD?
#[ -f ATSAM3X8E.svd.patched ] && rm -f ATSAM3X8E.svd.patched
#svd patch ATSAM3X8E.svd

#rm -rf src/
#svd2rust --nightly -i ATSAME3X8E.svd.patched
#form -i lib.rs -o src/
#rm lib.rs
