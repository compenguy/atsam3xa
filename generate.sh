#!/usr/bin/env bash

# Bash script to (re)generate Platform Access Crates (PACs) with svd2rust.
# For PAC to be generated, the following items are required:
#
# - It needs to be registered below in the `PACS` array.
# - The directory needs to contain an SVD file with the name matching
#   the crate name.

set -euo pipefail
#set -o xtrace

# everything is relative to the generate script
cd "$(dirname "$0")"


die() {
    echo "$1"
    exit 1
}

process_template() {
    local template_file="$1"
    local output_file="$2"
    mkdir -p $(dirname "${output_file}")
    cat "${template_file}" | envsubst > "${output_file}"
}

make_pac() {
    # canonicalize the path becase we'll want to refer to it after
    # changing cwd
    local svd_path="$(${canon_cmd} $1)"
    local chip=$(basename "${svd_path}" .svd)
    local chip_lower=$(echo "${chip}" | tr '[:upper:]' '[:lower:]')
    local pac_path="./pac/${chip_lower}"
    echo "Creating pac for ${chip}..."
    [ -d "${pac_path}" ] && rm -rf "${pac_path}"
    echo "	Creating pac directory ${pac_path}"
    mkdir -p "${pac_path}"
    echo "	Generating pac Cargo.toml and README.md from template..."
    crate=${chip_lower} mcu=${chip} process_template pac/templates/Cargo.toml.template pac/${chip_lower}/Cargo.toml
    crate=${chip_lower} mcu=${chip} process_template pac/templates/README.md.template pac/${chip_lower}/README.md
    echo "	Adding pac license(s)..."
    cp pac/licenses/* pac/${chip_lower}/
    echo "	Running pac codegen..."
    (cd "${pac_path}" && \
        svd2rust -i "${svd_path}" 2> svd2rust-${chip_lower}-warnings.log && \
        RUST_LOG=form=warn form -i lib.rs -o src >> svd2rust-${chip_lower}-warnings.log && \
        rm lib.rs)
    echo "	Formatting generated code..."
    (cd "${pac_path}" && cargo fmt && rustfmt build.rs)
    echo ""
}

command -v envsubst >/dev/null 2>&1 || die "Missing command 'envsubst'. Install the 'gettext' or 'gettext-base' package."
command -v readlink >/dev/null 2>&1 && canon_cmd="readlink -f"
command -v realpath >/dev/null 2>&1 && canon_cmd="realpath"
[ -n "${canon_cmd}" ] || die "Failed to locate a shell program for canonicalizing paths (e.g. readlink/realpath)"

command -v cargo-fmt >/dev/null 2>&1 || die "Missing command 'cargo-fmt'"
command -v svd2rust >/dev/null 2>&1 || cargo install --force \
        --git https://github.com/rust-embedded/svd2rust.git \
        --branch master svd2rust || die "Failed to install 'svd2rust'"
command -v form >/dev/null 2>&1 || cargo install --force \
        --version 0.7.0 form || die "Failed to install 'form'"

### Main

for svd in svd/*.svd; do
    make_pac "${svd}"
done
