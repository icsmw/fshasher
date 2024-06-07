#![doc = include_str!("../README.md")]

mod breaker;
mod collector;
pub(crate) mod entry;
mod error;
mod hasher;
mod reader;
#[cfg(test)]
pub(crate) mod test;
pub(crate) mod walker;

pub use breaker::Breaker;
pub use entry::{Entry, Filter};
pub use hasher::Hasher;
pub use reader::Reader;
pub use walker::{JobType, Options, ReadingStrategy, Tick, Tolerance, Walker};
