use crate::{profiles::Profile, EXTRACTOR};
use home::home_dir;
use std::{
    collections::HashMap,
    env,
    fs::read_to_string,
    io::{Error, ErrorKind},
    path::{Path, PathBuf},
    str::FromStr,
};

const WINDIR: &str = "windir";
const PROCESSOR_ARCHITEW6432: &str = "PROCESSOR_ARCHITEW6432";
const HOMEDRIVE: &str = "HOMEDRIVE";
const LOCALAPPDATA: &str = "LOCALAPPDATA";

fn get_envvars() -> HashMap<String, String> {
    let envvars = match EXTRACTOR.get(None, &Vec::new()) {
        Ok(vars) => vars,
        Err(err) => {
            log::warn!("Fail to get envvars with extractor: {err}");
            HashMap::new()
        }
    };
    let mut proc_envvars: HashMap<String, String> = HashMap::new();
    for (key, value) in env::vars() {
        proc_envvars.insert(key, value);
    }
    if proc_envvars.len() > envvars.len() {
        proc_envvars
    } else {
        envvars
    }
}

fn get_path_buf(str_path: &str) -> Result<PathBuf, Error> {
    PathBuf::from_str(str_path).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("Fail to convert string \"{str_path}\" to path: {e}"),
        )
    })
}

fn add_profile(
    envvars: &HashMap<String, String>,
    list: &mut Vec<Profile>,
    name: String,
    path: PathBuf,
    args: Vec<&str>,
) {
    if !path.exists() {
        return;
    }
    if let Ok(profile) = Profile::new(&path, args) {
        list.push(profile);
    }
}

pub(crate) fn get() -> Result<Vec<Profile>, Error> {
    let envvars = get_envvars();
    let windir = envvars.get(WINDIR).ok_or(Error::new(
        ErrorKind::NotFound,
        format!("Cannot find var \"{WINDIR}\""),
    ))?;
    let homedrive = envvars.get(HOMEDRIVE).ok_or(Error::new(
        ErrorKind::NotFound,
        format!("Cannot find var \"{WINDIR}\""),
    ))?;
    let system_32_path = if envvars.contains_key(PROCESSOR_ARCHITEW6432) {
        format!("{windir}\\Sysnative")
    } else {
        format!("{windir}\\System32")
    };
    let mut profiles: Vec<Profile> = vec![];
    // Windows PowerShell
    add_profile(
        &envvars,
        &mut profiles,
        String::from("Windows PowerShell"),
        get_path_buf(&format!(
            "{system_32_path}\\WindowsPowerShell\\v1.0\\powershell.exe"
        ))?,
        vec![],
    );
    if let Some(home) = home_dir() {
        // .NET Core PowerShell Global Tool
        add_profile(
            &envvars,
            &mut profiles,
            String::from(".NET Core PowerShell Global Tool"),
            get_path_buf(&format!(
                "{}\\.dotnet\\tools\\pwsh.exe",
                home.to_string_lossy()
            ))?,
            vec![],
        );
    }
    // Command Prompt
    add_profile(
        &envvars,
        &mut profiles,
        String::from("Command Prompt"),
        get_path_buf(&format!("{system_32_path}\\cmd.exe"))?,
        vec![],
    );
    // Cygwin
    add_profile(
        &envvars,
        &mut profiles,
        String::from("Cygwin x64"),
        get_path_buf(&format!("{homedrive}\\cygwin64\\bin\\bash.exe"))?,
        vec!["--login", "-c"],
    );
    add_profile(
        &envvars,
        &mut profiles,
        String::from("Cygwin"),
        get_path_buf(&format!("{homedrive}\\cygwin\\bin\\bash.exe"))?,
        vec!["--login", "-c"],
    );
    // bash (MSYS2)
    add_profile(
        &envvars,
        &mut profiles,
        String::from("bash (MSYS2)"),
        get_path_buf(&format!("{homedrive}\\msys64\\usr\\bin\\bash.exe"))?,
        vec!["--login", "-i", "-c"],
    );
    // GitBash
    for key in vec!["ProgramW6432", "ProgramFiles", "ProgramFiles(X86)"] {
        if let Some(v) = envvars.get(key) {
            add_profile(
                &envvars,
                &mut profiles,
                String::from("GitBash"),
                get_path_buf(&format!("{v}\\Git\\bin\\bash.exe"))?,
                vec!["--login", "-i", "-c"],
            );
            add_profile(
                &envvars,
                &mut profiles,
                String::from("GitBash"),
                get_path_buf(&format!("{v}\\Git\\usr\\bin\\bash.exe"))?,
                vec!["--login", "-i", "-c"],
            );
        }
    }
    if let Some(v) = envvars.get("LocalAppData") {
        add_profile(
            &envvars,
            &mut profiles,
            String::from("GitBash"),
            get_path_buf(&format!("{v}\\Programs\\Git\\bin\\bash.exe"))?,
            vec!["--login", "-i", "-c"],
        );
    }
    if let Some(v) = envvars.get("UserProfile") {
        add_profile(
            &envvars,
            &mut profiles,
            String::from("GitBash"),
            get_path_buf(&format!(
                "{v}\\scoop\\apps\\git-with-openssh\\current\\bin\\bash.exe"
            ))?,
            vec!["--login", "-i", "-c"],
        );
    }
    Ok(profiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut profiles = get().unwrap();
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
