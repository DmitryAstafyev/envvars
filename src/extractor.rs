use crate::EXTRACTOR;
use std::{
    collections::HashMap,
    env::temp_dir,
    fs::{File, OpenOptions},
    io::{Error, ErrorKind, Write},
    path::{Path, PathBuf},
    process::{Command, Output},
    str::from_utf8,
};
use uuid::Uuid;

#[cfg(not(windows))]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(not(windows))]
static EXECUTOR_BIN: &[u8] = include_bytes!("../extractor/target/debug/extractor");

#[cfg(windows)]
static EXECUTOR_BIN: &[u8] = include_bytes!("../extractor/target/debug/extractor.exe");

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
            #[cfg(not(windows))]
            location: temp_dir().join(Path::new(&Uuid::new_v4().to_string())),
            #[cfg(windows)]
            location: temp_dir().join(Path::new(&format!("{}.exe", Uuid::new_v4()))),
        }
    }

    #[cfg(not(windows))]
    fn create_file(&self) -> Result<File, Error> {
        OpenOptions::new()
            .mode(0o777)
            .read(true)
            .write(true)
            .create(true)
            .open(&self.location)
    }
    #[cfg(windows)]
    fn create_file(&self) -> Result<File, Error> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.location)
    }

    fn delivery(&self) -> Result<(), Error> {
        if self.location.exists() {
            log::warn!("Extractor {:?} already exists", self.location);
            return Ok(());
        }
        let mut file = self.create_file()?;
        file.write_all(EXECUTOR_BIN)?;
        file.flush()?;
        log::debug!("File is written in: {:?}", self.location);
        Ok(())
    }

    fn output(&self, shell: Option<&PathBuf>, args: &Vec<String>) -> Result<Output, Error> {
        if cfg!(windows) {
            if let Some(shell) = shell {
                let location_str = self
                    .location
                    .to_string_lossy()
                    .to_string()
                    .replace("\\", "\\\\");
                Command::new(shell)
                    .args(args.iter())
                    .arg(
                        &self
                            .location
                            .to_string_lossy()
                            .to_string()
                            .replace("\\", "\\\\"),
                    )
                    .output()
            } else {
                Command::new(&self.location).output()
            }
        } else if cfg!(unix) {
            if let Some(shell) = shell {
                Command::new(shell).arg("-c").arg(&self.location).output()
            } else {
                Command::new(&self.location).output()
            }
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Current platform isn't supported",
            ))
        }
    }

    pub fn get(
        &self,
        shell: Option<&PathBuf>,
        args: &Vec<String>,
    ) -> Result<HashMap<String, String>, Error> {
        self.delivery()?;
        let output = self.output(shell, args)?;
        let stdout = from_utf8(&output.stdout).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Fail to parse stdout to UTF8: {e}"),
            )
        })?;
        let stderr = from_utf8(&output.stderr).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("Fail to parse stderr to UTF8: {e}"),
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
    use crate::profiles::get as get_profiles;

    #[test]
    fn test() {
        let mut profiles = get_profiles().unwrap();
        profiles.iter_mut().for_each(|p| {
            if let Err(err) = p.load() {
                println!(
                    "{}: {:?}; fail to get envvars: {err}",
                    p.name,
                    p.path,
                );
            }
            println!(
                "{}: {:?}; (envvars: {})",
                p.name,
                p.path,
                if let Some(envvars) = p.envvars.as_ref() {
                    envvars.len()
                } else {
                    0
                }
            );
        });
    }
}
