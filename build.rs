use cargo::{
    core::{compiler::CompileMode, Workspace},
    ops::{compile, CompileOptions},
    util::config::Config,
};
use std::{env, path::Path};

fn build_extractor() {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be present in EnvVars");
    let manifest_path = Path::new(&manifest_dir).join(Path::new("extractor/Cargo.toml"));
    let config = Config::default().expect("Default configuration for compiler should be inited");
    let workspace = Workspace::new(&manifest_path, &config)
        .expect("Default workspace for compiler should be inited");
    let options = CompileOptions::new(&config, CompileMode::Build)
        .expect("Default configuration compile options should be inited");
    compile(&workspace, &options).expect("Comiling extractor should be done");
}

fn main() {
    build_extractor();
}
