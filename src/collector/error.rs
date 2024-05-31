use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("IO Error: {0}")]
    IO(#[from] io::Error),
    #[error("Error related to \"{0}\": {1}")]
    IOBound(PathBuf, io::Error),
}
