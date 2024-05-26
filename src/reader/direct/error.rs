use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("Fail to parse path {0}: {1}")]
    IOError(PathBuf, io::Error),
}

impl From<(PathBuf, io::Error)> for E {
    fn from(err: (PathBuf, io::Error)) -> Self {
        E::IOError(err.0, err.1)
    }
}
