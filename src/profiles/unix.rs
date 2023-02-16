use crate::{ profiles::Profile};
use std::{
    fs::read_to_string,
    io::{Error, ErrorKind},
    path::Path,
};

const SHELLS_FILE_PATH: &str = "/etc/shells";

pub(crate) fn get() -> Result<Vec<Profile>, Error> {
    let shells_file_path = Path::new(SHELLS_FILE_PATH);
    if !shells_file_path.exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("Fail to find shells list: {SHELLS_FILE_PATH}. File doesn't exist"),
        ));
    }
    let mut profiles: Vec<Profile> = vec![];
    for shell in read_to_string(shells_file_path)?.split('\n') {
        let path = Path::new(shell);
        let profile = match Profile::new(&path.to_path_buf(), vec!["-c"]) {
            Ok(profile) => profile,
            Err(err) => {
                log::warn!("Cannot get envvars for {shell}: {err}");
                continue;
            }
        };
        profiles.push(profile);
    }
    Ok(profiles)
}

