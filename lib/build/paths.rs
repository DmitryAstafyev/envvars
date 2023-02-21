use std::{
    env,
    fs::create_dir,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

fn if_exist(path: PathBuf) -> Result<PathBuf, Error> {
    if !path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("{path:?} doesn't exist"),
        ));
    }
    Ok(path)
}

pub fn manifest_dir() -> Result<PathBuf, Error> {
    if_exist(
        Path::new(
            &env::var("CARGO_MANIFEST_DIR").map_err(|_e| {
                Error::new(ErrorKind::NotFound, "CARGO_MANIFEST_DIR doesn't exist")
            })?,
        )
        .to_path_buf(),
    )
}

pub fn extractor_src_dir() -> Result<PathBuf, Error> {
    if_exist(assets_dir()?.join("extractor"))
}

pub fn extractor_dest_dir() -> Result<PathBuf, Error> {
    let path = env::temp_dir().join("__envvars_crate_build_folder__");
    if !path.exists() {
        create_dir(&path)?;
    }
    if_exist(path)
}

pub fn assets_dir() -> Result<PathBuf, Error> {
    if_exist(manifest_dir()?.join("assets"))
}

pub fn extractor_executable() -> Result<PathBuf, Error> {
    if_exist(
        extractor_dest_dir()?
            .join("extractor")
            .join("target")
            .join("release")
            .join(executable_file_name()),
    )
}

// pub fn output_dir() -> Result<PathBuf, Error> {
//     let path = extractor_dest_dir()?.join("output");
//     if !path.exists() {
//         create_dir(&path)?;
//     }
//     if_exist(path)
// }

pub fn executable_file_name() -> String {
    String::from(if cfg!(windows) {
        "extractor.exe"
    } else {
        "extractor"
    })
}

pub fn cargo_output_dir() -> Result<PathBuf, Error> {
    let out_dir = env::var_os("OUT_DIR").ok_or(Error::new(
        ErrorKind::NotFound,
        "Variable OUT_DIR doesn't exist".to_string(),
    ))?;
    if_exist(Path::new(&out_dir).to_path_buf())
}
