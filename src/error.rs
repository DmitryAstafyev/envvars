use std::{io, path::PathBuf};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO Error: {0:?}")]
    Io(io::Error),
    #[error("Parsing error: {0:?}")]
    Parsing(serde_json::Error, String, String),
    #[error("Fail to execute extractor: {0:?}")]
    Executing(io::Error),
    #[error("Fail to create extractor: {0:?}")]
    Create(io::Error),
    #[error("Fail to decode stdout/stderr: {0:?}")]
    Decoding(std::str::Utf8Error),
    #[error("Shell executor isn't found: {0:?}")]
    NotFound(PathBuf),
    #[error("Other: {0}")]
    Other(String),
    #[error("Platform isn't supported")]
    NotSupportedPlatform,
    #[error("Infallible: {0}")]
    Infallible(std::convert::Infallible),
    #[error("Fail to find envvar: {0}")]
    NotFoundEnvVar(String),
}
