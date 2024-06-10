#![doc = include_str!("../README.md")]

mod breaker;
pub mod collector;
pub(crate) mod entry;
mod error;
pub mod hasher;
pub mod reader;
#[cfg(test)]
pub(crate) mod test;
pub mod walker;

pub use breaker::Breaker;
pub use collector::{collect, Tolerance};
pub use entry::{Entry, Filter};
pub use hasher::Hasher;
pub use reader::Reader;
pub use walker::{JobType, Options, Progress, ReadingStrategy, Tick, Walker};

// TODO:
// - [x] usecase when during collecting or hashing files are removed or created isn't covered
// - [x] cover by tests iterator too
// - get rid of all unwrap() in code
// - [x] tests for different number of threads
// - [x] test for 0 threads
