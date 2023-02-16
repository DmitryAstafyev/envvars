use std::{
    collections::HashMap,
    env::temp_dir,
    fs::OpenOptions,
    io::{Error, ErrorKind, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    process::{Command, Output},
    str::from_utf8,
};
use uuid::Uuid;

static EXECUTOR_BIN: &[u8] = include_bytes!("../extractor/target/debug/extractor");

pub struct Extractor {
    location: PathBuf,
}

impl Drop for Extractor {
    fn drop(&mut self) {
        if self.location.exists() {
            if let Err(err) = std::fs::remove_file(&self.location) {
                log::error!(
                    "Fail to remove tmp file {}; error: {err}",
                    self.location.to_string_lossy()
                );
            }
        }
    }
}

impl Extractor {
    pub fn new() -> Self {
        Extractor {
            location: temp_dir().join(Path::new(&Uuid::new_v4().to_string())),
        }
    }
    fn delivery(&self) -> Result<(), Error> {
        if self.location.exists() {
            log::warn!("Extractor {:?} already exists", self.location);
            return Ok(());
        }
        let mut file = OpenOptions::new()
            .mode(0o777)
            .read(true)
            .write(true)
            .create(true)
            .open(&self.location)?;
        file.write_all(EXECUTOR_BIN)?;
        file.flush()?;
        log::debug!("File is written in: {:?}", self.location);
        Ok(())
    }

    fn output(&self, shell: &PathBuf) -> Result<Output, Error> {
        if cfg!(windows) {
            Err(Error::new(ErrorKind::Other, "Not implemented"))
        } else if cfg!(unix) {
            Command::new(shell).arg("-c").arg(&self.location).output()
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Current platform isn't supported",
            ))
        }
    }

    pub fn get(&self, shell: &PathBuf) -> Result<HashMap<String, String>, Error> {
        self.delivery()?;
        let output = self.output(shell)?;
        let stdout = from_utf8(&output.stdout).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Fail to parse stdout to UTF8: {e}"),
            )
        })?;
        serde_json::from_str::<HashMap<String, String>>(stdout).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Fail convert stdout into JSON: {e}"),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profiles::get as get_profiles;

    #[test]
    fn test_it() {
        let extractor = Extractor::new();
        let profiles = get_profiles(&extractor).unwrap();
        profiles.iter().for_each(|profile| {
            println!(
                "Getting envvars from {} ({:?}): {}",
                profile.name,
                profile.path,
                profile.envvars.len()
            );
        });
    }
}
