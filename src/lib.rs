#![doc = include_str!("../README.md")]

mod breaker;
pub mod collector;
pub(crate) mod entry;
pub mod hasher;
pub mod reader;
#[cfg(test)]
pub(crate) mod test;
pub mod walker;

pub use breaker::Breaker;
pub use collector::{collect, Tolerance};
pub use entry::{ContextFile, Entry, Filter};
pub use hasher::Hasher;
pub use reader::Reader;
pub use walker::{JobType, Options, Progress, ReadingStrategy, Tick, Walker, E};
