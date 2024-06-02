use crate::walker;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("{0}")]
    Walker(walker::E),
}

impl From<walker::E> for E {
    fn from(err: walker::E) -> Self {
        E::Walker(err)
    }
}
