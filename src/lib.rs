#[macro_use]
extern crate lazy_static;
use std::{collections::HashMap, io::Error};
mod extractor;
mod profiles;

use extractor::Extractor;
pub use profiles::{get as get_profiles, Profile};

lazy_static! {
    pub static ref EXTRACTOR: Extractor = Extractor::new();
}

pub fn get_context_envvars() -> Result<HashMap<String, String>, Error> {
    EXTRACTOR.get(None, &Vec::new())
}
