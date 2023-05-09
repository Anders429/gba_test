use std::env;

fn main() {
    let target = env::var("TARGET").ok();

    if let Some(target) = target {
        println!("cargo:rustc-cfg=target=\"{}\"", target);
    }
}
