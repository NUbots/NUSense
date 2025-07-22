use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    // Create an empty memory.x file to satisfy the include in the main linker script
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("memory.x");
    File::create(&dest_path).unwrap();

    // Add the output directory to the linker search path so `link.x` can find the empty `memory.x`
    println!("cargo:rustc-link-search={}", out_dir);

    // Set linker flags for all binaries
    println!("cargo:rustc-link-arg-bins=--nmagic");

    // Use the appropriate memory file based on the binary being built
    println!("cargo:rustc-link-arg-bin=boot=-Tmemory-bootloader.x");
    println!("cargo:rerun-if-changed=memory-bootloader.x");
    println!("cargo:rustc-link-arg-bin=nusense-rs=-Tmemory-app.x");
    println!("cargo:rerun-if-changed=memory-app.x");

    // Main linker script, uses the memory layout from above
    println!("cargo:rustc-link-arg-bins=-Tlink.x");

    // Only link defmt if the feature is enabled
    #[cfg(feature = "defmt")]
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");
}
