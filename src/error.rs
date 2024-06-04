use crate::{collector, walker};
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("{0}")]
    Walker(walker::E),
    #[error("{0}")]
    Collector(collector::E),
    #[error("IO: {0}")]
    IO(#[from] io::Error),
}

impl From<walker::E> for E {
    fn from(err: walker::E) -> Self {
        E::Walker(err)
    }
}

impl From<collector::E> for E {
    fn from(err: collector::E) -> Self {
        E::Collector(err)
    }
}
