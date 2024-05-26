use crate::walker;
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("Fail to parse path {0}: {1}")]
    IOError(PathBuf, io::Error),
    #[error("{0}")]
    Walker(walker::E),
}

impl From<walker::E> for E {
    fn from(err: walker::E) -> Self {
        E::Walker(err)
    }
}
