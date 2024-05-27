use crate::reader;
use glob::PatternError;
use std::{error, io, path::PathBuf, sync::PoisonError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("Fail to parse pattern {0}: {1}")]
    PatternError(String, PatternError),
    #[error("Fail to parse path {0}: {1}")]
    IOError(PathBuf, io::Error),
    #[error("Path {0} cannot be included into a list of targets")]
    InvalidEntity(PathBuf),
    #[error("Path {0} cannot be included into a list of targets. Only files and folders can be included")]
    OnlyFileOrFolder(PathBuf),
    #[error("Relative path {0} cannot be used as entry. ")]
    RelativePathAsEntry(PathBuf),
    #[error("Absolute path {0} cannot be used as filter (included/excluded).")]
    AbsolutePathAsFilter(String),
    #[error("Path {0} cannot be used as cwd because it isn't folder")]
    OnlyFolderAsCwd(PathBuf),
    #[error("Path {0} cannot be used because it isn't absolute")]
    AbsolutePathRequired(PathBuf),
    #[error("Operation has been aborted")]
    Aborted,
    #[error("Walker can be inited only once")]
    AlreadyInited,
    #[error("Reader error: {0}")]
    Reader(String),
    #[error("Hasher error: {0}")]
    Hasher(String),
    #[error("Reading IO error: {0}")]
    ReadingIOError(io::Error),
    #[error("Fail to get access to data between threads: {0}")]
    PoisonError(String),
}

impl E {
    pub fn reader<Er: std::error::Error>(err: Er) -> E {
        E::Reader(err.to_string())
    }
    pub fn hasher<Er: std::error::Error>(err: Er) -> E {
        E::Hasher(err.to_string())
    }
}

impl From<(String, PatternError)> for E {
    fn from(err: (String, PatternError)) -> Self {
        E::PatternError(err.0, err.1)
    }
}

impl From<io::Error> for E {
    fn from(err: io::Error) -> Self {
        E::ReadingIOError(err)
    }
}

impl From<(PathBuf, io::Error)> for E {
    fn from(err: (PathBuf, io::Error)) -> Self {
        E::IOError(err.0, err.1)
    }
}
