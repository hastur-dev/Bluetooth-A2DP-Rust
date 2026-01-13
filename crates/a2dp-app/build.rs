//! Build script to set up linker scripts for RP2350.

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("../../memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // Re-run when memory.x changes.
    println!("cargo:rerun-if-changed=../../memory.x");

    // Linker arguments for embedded
    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
