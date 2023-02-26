use crate::{profiles::Profile, Error, EXTRACTOR};
use home::home_dir;
use std::{collections::HashMap, env, path::PathBuf, str::FromStr};

const WINDIR: &str = "windir";
const SYSTEM_ROOT: &str = "systemroot";
const PROCESSOR_ARCHITEW6432: &str = "processor_architew6432";
const HOMEDRIVE: &str = "homedrive";

fn get_envvars() -> Result<HashMap<String, String>, Error> {
    let envvars = match EXTRACTOR
        .lock()
        .map_err(|e| Error::PoisonError(e.to_string()))?
        .get(None, &Vec::new())
    {
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
    Ok(if proc_envvars.len() > envvars.len() {
        proc_envvars
    } else {
        envvars
    })
}

fn keys_to_lower_case(map: &HashMap<String, String>) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    map.iter().for_each(|(k, v)| {
        result.insert(k.to_lowercase(), v.clone());
    });
    result
}

fn get_path_buf(str_path: &str) -> Result<PathBuf, Error> {
    PathBuf::from_str(str_path).map_err(Error::Infallible)
}

fn add_profile(list: &mut Vec<Profile>, name: &str, path: PathBuf, args: Vec<&str>) {
    if !path.exists() {
        return;
    }
    if let Ok(profile) = Profile::new(&path, args, Some(name)) {
        list.push(profile);
    }
}

pub(crate) fn get() -> Result<Vec<Profile>, Error> {
    let envvars = get_envvars()?;
    let envvars_lower_case = keys_to_lower_case(&get_envvars()?);
    let windir = envvars_lower_case
        .get(WINDIR)
        .ok_or(Error::NotFoundEnvVar(WINDIR.to_string()))?;
    let homedrive = envvars_lower_case
        .get(HOMEDRIVE)
        .ok_or(Error::NotFoundEnvVar(HOMEDRIVE.to_string()))?;
    let system_32_path = if envvars_lower_case.contains_key(PROCESSOR_ARCHITEW6432) {
        format!("{windir}\\Sysnative")
    } else {
        format!("{windir}\\System32")
    };
    let mut profiles: Vec<Profile> = vec![];
    if let Some(sys_root) = envvars_lower_case.get(SYSTEM_ROOT) {
        let system_path = if envvars_lower_case.contains_key(PROCESSOR_ARCHITEW6432) {
            format!("{sys_root}\\Sysnative")
        } else {
            format!("{sys_root}\\System32")
        };
        // WSL (build > 16299)
        add_profile(
            &mut profiles,
            "WSL",
            get_path_buf(&format!("{system_path}\\wsl.exe"))?,
            vec!["-c"],
        );
        // WSL Bash (build < 16299)
        add_profile(
            &mut profiles,
            "WSL (bash)",
            get_path_buf(&format!("{system_path}\\bash.exe"))?,
            vec!["-c"],
        );
    }
    // Windows PowerShell
    add_profile(
        &mut profiles,
        "Windows PowerShell",
        get_path_buf(&format!(
            "{system_32_path}\\WindowsPowerShell\\v1.0\\powershell.exe"
        ))?,
        vec![],
    );
    if let Some(home) = home_dir() {
        // .NET Core PowerShell Global Tool
        add_profile(
            &mut profiles,
            ".NET Core PowerShell Global Tool",
            get_path_buf(&format!(
                "{}\\.dotnet\\tools\\pwsh.exe",
                home.to_string_lossy()
            ))?,
            vec![],
        );
    }
    // Command Prompt
    add_profile(
        &mut profiles,
        "Command Prompt",
        get_path_buf(&format!("{system_32_path}\\cmd.exe"))?,
        vec![],
    );
    // Cygwin
    add_profile(
        &mut profiles,
        "Cygwin x64",
        get_path_buf(&format!("{homedrive}\\cygwin64\\bin\\bash.exe"))?,
        vec!["--login", "-c"],
    );
    add_profile(
        &mut profiles,
        "Cygwin",
        get_path_buf(&format!("{homedrive}\\cygwin\\bin\\bash.exe"))?,
        vec!["--login", "-c"],
    );
    // bash (MSYS2)
    add_profile(
        &mut profiles,
        "bash (MSYS2)",
        get_path_buf(&format!("{homedrive}\\msys64\\usr\\bin\\bash.exe"))?,
        vec!["--login", "-i", "-c"],
    );
    // GitBash
    for key in ["ProgramW6432", "ProgramFiles", "ProgramFiles(X86)"] {
        if let Some(v) = envvars.get(key) {
            add_profile(
                &mut profiles,
                "GitBash",
                get_path_buf(&format!("{v}\\Git\\bin\\bash.exe"))?,
                vec!["--login", "-i", "-c"],
            );
            add_profile(
                &mut profiles,
                "GitBash",
                get_path_buf(&format!("{v}\\Git\\usr\\bin\\bash.exe"))?,
                vec!["--login", "-i", "-c"],
            );
        }
    }
    if let Some(v) = envvars.get("LocalAppData") {
        add_profile(
            &mut profiles,
            "GitBash",
            get_path_buf(&format!("{v}\\Programs\\Git\\bin\\bash.exe"))?,
            vec!["--login", "-i", "-c"],
        );
    }
    if let Some(v) = envvars.get("UserProfile") {
        add_profile(
            &mut profiles,
            "GitBash",
            get_path_buf(&format!(
                "{v}\\scoop\\apps\\git-with-openssh\\current\\bin\\bash.exe"
            ))?,
            vec!["--login", "-i", "-c"],
        );
    }
    Ok(profiles)
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut profiles = get().unwrap();
        profiles.iter_mut().for_each(|p| {
            if let Err(err) = p.load() {
                println!("{}: {:?}; fail to get envvars: {err}", p.name, p.path,);
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
