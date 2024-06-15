use std::io;
use thiserror::Error;

use fshasher::walker::E as FsHasherError;

#[derive(Error, Debug)]
pub enum E {
    #[error("Fail to read: {0}")]
    IOError(io::Error),
    #[error("Custom reader doesn't support mapping file into memory")]
    MemoryMappingNotSupported,
}

impl From<io::Error> for E {
    fn from(err: io::Error) -> Self {
        E::IOError(err)
    }
}

impl From<E> for FsHasherError {
    fn from(val: E) -> Self {
        FsHasherError::Reader(val.to_string())
    }
}
