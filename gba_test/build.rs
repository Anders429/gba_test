use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = &PathBuf::from(env::var("OUT_DIR").unwrap());
    let linker_script = if env::var("CARGO_CFG_GBA_TEST_MB").is_ok() {
        println!("cargo:warning=using multiboot linker script");
        include_bytes!("linker_scripts/mb.ld").as_slice()
    } else {
        println!("cargo:warning=using monoboot linker script");
        include_bytes!("linker_scripts/gba.ld").as_slice()
    };
    fs::write(out_dir.join("gba.ld"), linker_script).unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
}
