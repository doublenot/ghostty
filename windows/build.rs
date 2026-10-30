//! Links against libghostty built by `zig build` at the repository root
//! and copies its DLL next to the executable so it is found at runtime.

use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let lib_dir = env::var_os("GHOSTTY_LIB_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| manifest_dir.join("..").join("zig-out").join("lib"));
    let dll = lib_dir.join("ghostty-internal.dll");

    println!("cargo:rerun-if-env-changed=GHOSTTY_LIB_DIR");
    println!("cargo:rerun-if-changed={}", dll.display());
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=dylib=ghostty-internal");

    // libghostty runs on our main thread and expects the 16 MiB stack Zig
    // gives its own executables, not the 1 MiB Windows default.
    println!("cargo:rustc-link-arg-bins=/STACK:16777216");

    if !dll.exists() {
        println!(
            "cargo:warning=ghostty-internal.dll not found in {}; run `zig build` at the repository root first",
            lib_dir.display()
        );
        return;
    }

    // OUT_DIR is target/<profile>/build/<package>-<hash>/out, and the
    // executable lands in target/<profile>.
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let profile_dir = out_dir.ancestors().nth(3).unwrap();
    fs::copy(&dll, profile_dir.join("ghostty-internal.dll"))
        .expect("failed to copy ghostty-internal.dll");
}
