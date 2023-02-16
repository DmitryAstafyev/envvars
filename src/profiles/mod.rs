use crate::extractor::{self, Extractor};
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
    pub envvars: HashMap<String, String>,
}

impl Profile {
    pub fn new(shell: &PathBuf, extractor: Option<&Extractor>) -> Result<Self, Error> {
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
            envvars: if let Some(extractor) = extractor {
                extractor.get(shell)?
            } else {
                Extractor::new().get(shell)?
            },
        })
    }
}

pub fn get(extractor: &Extractor) -> Result<Vec<Profile>, Error> {
    if cfg!(windows) {
        Err(Error::new(ErrorKind::Other, "Not implemented"))
    } else if cfg!(unix) {
        unix::get(extractor)
    } else {
        Err(Error::new(
            ErrorKind::Other,
            "Current platform isn't supported",
        ))
    }
}
