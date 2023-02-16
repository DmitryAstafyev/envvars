use crate::EXTRACTOR;
use std::{
    collections::HashMap,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
};

pub mod unix;
pub mod windows;

#[derive(Debug, Clone)]
pub struct Profile {
    pub name: String,
    pub path: PathBuf,
    pub envvars: Option<HashMap<String, String>>,
    args: Vec<String>,
}

impl Profile {
    pub fn new(shell: &PathBuf, args: Vec<&str>) -> Result<Self, Error> {
        let path = Path::new(shell);
        if !path.exists() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("File {shell:?} doesn't exist"),
            ));
        }
        let name = path
            .file_name()
            .ok_or(Error::new(
                ErrorKind::Other,
                format!("Found {shell:?}, but cannot convert path"),
            ))?
            .to_string_lossy()
            .to_string();
        Ok(Profile {
            name,
            path: shell.clone(),
            envvars: None,
            args: args.into_iter().map(|s| s.to_owned()).collect::<Vec<String>>(),
        })
    }

    pub fn load(&mut self) -> Result<(), Error> {
        self.envvars = Some(EXTRACTOR.get(Some(&self.path), &self.args)?);
        Ok(())
    }
}


pub fn get() -> Result<Vec<Profile>, Error> {
    if cfg!(windows) {
        windows::get()
    } else if cfg!(unix) {
        unix::get()
    } else {
        Err(Error::new(
            ErrorKind::Other,
            "Current platform isn't supported",
        ))
    }
}
