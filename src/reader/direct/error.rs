use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("Fail to read: {0}")]
    IOError(io::Error),
    #[error("Setup step is missed")]
    SetupIsMissed,
}

impl From<io::Error> for E {
    fn from(err: io::Error) -> Self {
        E::IOError(err)
    }
}
