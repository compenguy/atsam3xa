//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

/*
#[cfg(any(feature = "atsam3a4c", feature = "atsam3x4c", feature = "atsam3x4e"))]
const LINKER_SCRIPT: &str = "memory_sam3_4.x";
#[cfg(any(feature = "atsam3a8c", feature = "atsam3x8c", feature = "atsam3x8e", feature = "atsam3x8h"))]
*/
const LINKER_SCRIPT: &str = "memory_sam3_8.x";

fn main() {
    // Put the linker script somewhere the linker can find it
    let src_dir = std::path::PathBuf::from(
        std::env::var_os("CARGO_MANIFEST_DIR").expect("Failed to locate project root directory"),
    );
    let link_dir = std::path::PathBuf::from(
        std::env::var_os("OUT_DIR").expect("Failed to locate project build directory"),
    );
    std::fs::copy(src_dir.join(LINKER_SCRIPT), link_dir.join("memory.x"))
        .expect("Failed copying linker script from project root to build directory");
    println!("cargo:rustc-link-search={}", link_dir.to_string_lossy());

    // Only re-run the build script when memory.x is changed,
    // instead of when any part of the source code changes.
    println!("{}", format!("cargo:rerun-if-changed={}", LINKER_SCRIPT));
}
