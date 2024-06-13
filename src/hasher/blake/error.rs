use thiserror::Error;

use crate::walker;

#[derive(Error, Debug)]
pub enum E {
    #[error("Hashing not finished")]
    NotFinished,
}

impl From<E> for walker::E {
    fn from(val: E) -> Self {
        walker::E::Hasher(val.to_string())
    }
}
