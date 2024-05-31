use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("IO Error: {0}")]
    IO(#[from] io::Error),
    #[error("No available workers")]
    NoAvailableWorkers,
    #[error("Fail delivery result because issue of channel")]
    ChannelIssue,
    #[error("Fail to get optimal threads number")]
    OptimalThreadsNumber,
}
