use std::io;
use thiserror::Error;

use crate::walker;

#[derive(Error, Debug)]
pub enum E {
    #[error("Fail to read: {0}")]
    IOError(io::Error),
    #[error("Buffering reader doesn't support mapping file into memory")]
    MemoryMappingNotSupported,
}

impl From<io::Error> for E {
    fn from(err: io::Error) -> Self {
        E::IOError(err)
    }
}

impl From<E> for walker::E {
    fn from(val: E) -> Self {
        walker::E::Reader(val.to_string())
    }
}
