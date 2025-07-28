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
        .write_all(include_bytes!("memory.x"))
        .unwrap();

    // Add the output directory to the linker search path so `link.x` can find the empty `memory.x`
    println!("cargo:rustc-link-search={}", out.display());

    // println!("cargo:rustc-link-arg-bins=-Tmemory.x");

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    // Configure defmt log level based on features
    if env::var("CARGO_FEATURE_DEBUG").is_ok() {
        // Debug build - enable all log levels
        println!("cargo:rustc-env=DEFMT_LOG=trace");
    } else {
        // Release build - only warnings and errors to minimize size
        println!("cargo:rustc-env=DEFMT_LOG=off");
    }

    // Set linker flags for all binaries
    println!("cargo:rustc-link-arg-bins=--nmagic");

    // Main linker script, uses the memory layout from above
    println!("cargo:rustc-link-arg-bins=-Tlink.x");

    // Link the defmt linker script
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
