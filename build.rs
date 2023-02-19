use std::{
    env,
    fs::{remove_file, File, OpenOptions},
    io,
    io::Write,
    path::Path,
};
use uuid::Uuid;

#[path = "src/checksum.rs"]
mod checksum;

use checksum::checksum;

const ASSET_FILENAME: &str = "./assets/filename.asset";
const ASSET_CHECKSUM: &str = "./assets/checksum.asset";

#[cfg(not(windows))]
const EXTRACTOR_PATH: &str = "extractor/target/release/extractor";
#[cfg(windows)]
const EXTRACTOR_PATH: &str = "extractor/target/release/extractor.exe";

fn write_extractor_file_name(location: &Path) -> Result<(), io::Error> {
    if location.exists() {
        return Ok(());
    }
    let mut file: File = OpenOptions::new().write(true).create(true).open(location)?;
    let filename = Uuid::new_v4();
    file.write_all(filename.to_string().as_bytes())?;
    file.flush()
}

fn write_extractor_checksum(location: &Path) -> Result<(), io::Error> {
    if location.exists() {
        remove_file(location)?;
    }
    let mut file: File = OpenOptions::new().write(true).create(true).open(location)?;
    let checksum = checksum(&Path::new(EXTRACTOR_PATH).to_path_buf())?;
    file.write_all(checksum.as_bytes())?;
    file.flush()
}

fn main() -> Result<(), io::Error> {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR should be present in EnvVars");
    write_extractor_file_name(&Path::new(&manifest_dir).join(ASSET_FILENAME))?;
    write_extractor_checksum(&Path::new(&manifest_dir).join(ASSET_CHECKSUM))?;
    Ok(())
}
