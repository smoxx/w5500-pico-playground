use std::{env, fs, path::PathBuf};

fn main() {
    let output_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());

    fs::copy("memory.x", output_dir.join("memory.x")).unwrap();

    println!("cargo:rustc-link-search={}", output_dir.display());
    println!("cargo:rerun-if-changed=memory.x");
}
