use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("Fail to read: {0}")]
    IOError(io::Error),
    #[error("Direct reader doesn't support mapping file into memory")]
    MemoryMappingNotSupported,
}

impl From<io::Error> for E {
    fn from(err: io::Error) -> Self {
        E::IOError(err)
    }
}
