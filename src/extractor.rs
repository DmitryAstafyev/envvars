use crate::Error;
use std::{
    collections::HashMap,
    env::temp_dir,
    fs::{File, OpenOptions},
    io,
    io::Write,
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

    fn delivery(&self) -> Result<(), io::Error> {
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
        &self,
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
