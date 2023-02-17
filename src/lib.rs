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

pub fn get_context_envvars() -> Result<HashMap<String, String>, Error> {
    EXTRACTOR.get(None, &Vec::new())
}
