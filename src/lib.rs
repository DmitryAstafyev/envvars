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
