#![doc = include_str!("../README.md")]

mod breaker;
mod collector;
pub(crate) mod entry;
mod error;
pub mod hasher;
pub mod reader;
#[cfg(test)]
pub(crate) mod test;
pub mod walker;

pub use breaker::Breaker;
pub use entry::{Entry, Filter};
pub use hasher::Hasher;
pub use reader::Reader;
pub use walker::{JobType, Options, ReadingStrategy, Tick, Tolerance, Walker};

// TODO:
// - usecase when during collecting or hashing files are removed or created isn't covered
// - cover by tests iterator too
