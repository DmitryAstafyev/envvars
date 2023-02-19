use crate::{checksum::checksum, Error};
use std::{
    collections::HashMap,
    env::temp_dir,
    fs::{remove_file, File, OpenOptions},
    io,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Output},
    str::from_utf8,
};

#[cfg(not(windows))]
use std::os::unix::fs::OpenOptionsExt;
#[cfg(not(windows))]
static EXECUTOR_BIN: &[u8] = include_bytes!("../extractor/target/release/extractor");
#[cfg(windows)]
static EXECUTOR_BIN: &[u8] = include_bytes!("../extractor/target/release/extractor.exe");

static ASSET_CHECKSUM: &str = include_str!("../assets/checksum.asset");
static ASSET_FILENAME: &str = include_str!("../assets/filename.asset");

pub struct Extractor {
    location: PathBuf,
    checked: bool,
}

impl Extractor {
    pub fn new() -> Self {
        Extractor {
            #[cfg(not(windows))]
            location: temp_dir().join(Path::new(ASSET_FILENAME)),
            #[cfg(windows)]
            location: temp_dir().join(Path::new(ASSET_FILENAME)),
            checked: false,
        }
    }

    #[cfg(not(windows))]
    fn create_file(&self) -> Result<File, io::Error> {
        OpenOptions::new()
            .mode(0o777)
            .read(true)
            .write(true)
            .create(true)
            .open(&self.location)
    }
    #[cfg(windows)]
    fn create_file(&self) -> Result<File, io::Error> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.location)
    }

    fn delivery(&mut self) -> Result<(), io::Error> {
        if self.location.exists() && self.checked {
            log::warn!("Extractor {:?} already exists", self.location);
            return Ok(());
        }
        if self.location.exists() && !self.checked {
            if !match checksum(&self.location) {
                Ok(checksum) => checksum == ASSET_CHECKSUM,
                Err(err) => {
                    log::warn!(
                        "Fail to get checksum of extractor {:?}: {err}",
                        self.location
                    );
                    false
                }
            } {
                remove_file(&self.location)?;
            } else {
                self.checked = true;
                return Ok(());
            }
        }
        let mut file = self.create_file()?;
        file.write_all(EXECUTOR_BIN)?;
        file.flush()?;
        self.checked = true;
        log::debug!("File is written in: {:?}", self.location);
        Ok(())
    }

    fn output(&self, shell: Option<&PathBuf>, args: &[String]) -> Result<Output, io::Error> {
        if cfg!(windows) {
            if let Some(shell) = shell {
                Command::new(shell)
                    .args(args.iter())
                    .arg(
                        &self
                            .location
                            .to_string_lossy()
                            .to_string()
                            .replace('\\', "\\\\"),
                    )
                    .output()
            } else {
                Command::new(&self.location).output()
            }
        } else if cfg!(unix) {
            if let Some(shell) = shell {
                Command::new(shell)
                    .args(args.iter())
                    .arg(&self.location)
                    .output()
            } else {
                Command::new(&self.location).output()
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::Other,
                "Current platform isn't supported",
            ))
        }
    }

    pub fn get(
        &mut self,
        shell: Option<&PathBuf>,
        args: &[String],
    ) -> Result<HashMap<String, String>, Error> {
        self.delivery().map_err(Error::Create)?;
        let output = self.output(shell, args).map_err(Error::Executing)?;
        let stdout = from_utf8(&output.stdout).map_err(Error::Decoding)?;
        let stderr = from_utf8(&output.stderr).map_err(Error::Decoding)?;
        serde_json::from_str::<HashMap<String, String>>(stdout)
            .map_err(|e| Error::Parsing(e, stdout.to_owned(), stderr.to_owned()))
    }
}

impl Default for Extractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{profiles::get as get_profiles, Profile};

    #[test]
    fn test() {
        let mut profiles = get_profiles().unwrap();
        let mut failed: Vec<(Profile, Error)> = vec![];
        println!("{}", "=".repeat(50));
        println!("Found shells with detected envvars:");
        println!("{}", "=".repeat(50));
        profiles.iter_mut().for_each(|p| {
            if let Err(err) = p.load() {
                failed.push((p.clone(), err));
            } else {
                println!(
                    "{}: {:?}; (detected variables: {})",
                    p.name,
                    p.path,
                    if let Some(envvars) = p.envvars.as_ref() {
                        envvars.len()
                    } else {
                        0
                    }
                );
            }
        });
        println!("{}", "=".repeat(50));
        println!("Found shells with failed detection of envvars:");
        println!("{}", "=".repeat(50));
        failed.iter().for_each(|(p, err)| match err {
            Error::Parsing(_err, _stdout, stderr) => {
                println!("{}: {:?}; error:\n{stderr}", p.name, p.path,);
            }
            _ => {
                println!("{}: {:?}; fail to get envvars: {err}", p.name, p.path,);
            }
        });
    }
}
