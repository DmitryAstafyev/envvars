#[path = "../src/checksum.rs"]
mod checksum;

use super::paths;
use checksum::checksum;
use std::{
    fs::{File, OpenOptions},
    io,
    io::Write,
};
use uuid::Uuid;

pub fn write_extractor_file_name() -> Result<(), io::Error> {
    let mut file: File = OpenOptions::new()
        .write(true)
        .create(true)
        .open(paths::output_dir()?.join("filename.asset"))?;
    let filename = Uuid::new_v4();
    file.write_all(filename.to_string().as_bytes())?;
    file.flush()
}

pub fn write_extractor_checksum() -> Result<(), io::Error> {
    let mut file: File = OpenOptions::new()
        .write(true)
        .create(true)
        .open(paths::output_dir()?.join("checksum.asset"))?;
    let checksum = checksum(&paths::extractor_executable()?)?;
    file.write_all(checksum.as_bytes())?;
    file.flush()
}
