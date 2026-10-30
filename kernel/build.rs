use std::env;
use std::path::PathBuf;

fn main() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("none") {
        let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR").map(PathBuf::from) else {
            return;
        };
        let linker_script = manifest_dir.join("linker.ld");
        println!(
            "cargo:rustc-link-arg-bin=kernel=-T{}",
            linker_script.display()
        );
        println!("cargo:rustc-link-arg-bin=kernel=-no-pie");
        println!("cargo:rerun-if-changed={}", linker_script.display());
    }
}
