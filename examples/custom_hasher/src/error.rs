use thiserror::Error;

use fshasher::walker::E as FsHasherError;

#[derive(Error, Debug)]
pub enum E {
    #[error("Hashing not finished")]
    NotFinished,
}

impl From<E> for FsHasherError {
    fn from(val: E) -> Self {
        FsHasherError::Hasher(val.to_string())
    }
}
