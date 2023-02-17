use std::{env, path::Path, process::Command};

fn build_extractor() {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be present in EnvVars");
    Command::new("cargo")
        .current_dir(Path::new(&manifest_dir).join(Path::new("extractor")))
        .args(["--release"])
        .status()
        .unwrap();
}

fn main() {
    build_extractor();
}
