//! # envvars
//!
//! `envvars` helps to detect available a list of available shells and related
//! environment variables. If works in two steps:
//!
//! - detecting a list of available shells and creating `Profile` for each found
//! shell
//! - loading a list of environment variables for selected or each shell
//!
//! Under the hood, `envvars` takes each shell, and executes it with a command,
//! which posts a list of environment variables to `stdout`. As soon as executing
//! is done, `envvars` reads `stdout` and parse environment variables into
//! `HashMap<String, String>`.
//!
//! As soon as extracting process could take sensitive some time (~1sec on windows
//! and ~10ms on Unix-based OS), `envvars` doesn't extract environment variables
//! during detecting the shell's profiles. That's the developer's decision when it
//! should be done for the selected or each profile.
//!
//! `envvars` creates a small executable application in the system's temporary folder.
//! This application is used to "drop" list of environment variables into `stdout`
//! of the parent process and does nothing else. As soon as `envvars` instance is
//! dropped, the application would be removed from the disk.
//!  
//! ## Unix specific
//! `envvars` reads `/etc/shells` and analyze each shell from a list
//!
//! ## Windows specific
//! `envvars` checks for availability next shells:
//! - Command Prompt
//! - Windows PowerShell
//! - .NET Core PowerShell Global Tool
//! - Cygwin x64
//! - Cygwin
//! - bash (MSYS2)
//! - GitBash
//!
//! ## Guaranteed results
//! Because `envvars` tries to initialize each shell and "drop" a list of environment
//! variables to `stdout`, the shell should support the possibility to get a command as
//! an argument, for example: `/bin/bash -c path_to_command`. Obviously not many, but
//! still some shells don't support it (like windows command prompt). In this case, you
//! still can use `get_context_envvars()` to get a list of environment variables without
//! the shell's context.
//!
//! ## Diffrence from `std::env::vars`
//! `envvars` actually executes each found `shell` it means: all settings of the target
//! shell will be inited before a list of environment variables will be requested. That's
//! very sensitive if the configuration of some shell includes some initialization script,
//! which affects environment variables. That means in some cases `std::env::vars` and
//! `envvars` could give different results.

#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
mod error;
mod extractor;
mod profiles;

pub use error::Error;
use extractor::Extractor;
pub use profiles::{get as get_profiles, Profile};

lazy_static! {
    #[doc(hidden)]
    pub static ref EXTRACTOR: Extractor = Extractor::new();
}

/// Extract environment variables without shell context.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use envvars::get_context_envvars;
///
/// let vars: HashMap<String, String> = get_context_envvars().unwrap();
///
/// assert!(vars.contains_key("PATH"));
/// ```
pub fn get_context_envvars() -> Result<HashMap<String, String>, Error> {
    EXTRACTOR.get(None, &Vec::new())
}
