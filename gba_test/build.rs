use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = &PathBuf::from(env::var("OUT_DIR").unwrap());
    // fs::write(
    //     out_dir.join("gba.ld"),
    //     include_bytes!("linker_scripts/gba.ld").as_slice(),
    // )
    // .unwrap();
    fs::write(
        out_dir.join("mb.ld"),
        include_bytes!("linker_scripts/mb.ld").as_slice(),
    )
    .unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());
}
