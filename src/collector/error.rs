use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("IO Error: {0}")]
    IO(#[from] io::Error),
    #[error("No available workers")]
    NoAvailableWorkers,
    #[error("Fail get feedback from main paths collector thread: {0}")]
    JoinError(String),
    #[error("Fail to get optimal threads number")]
    OptimalThreadsNumber,
    #[error("Operation has been aborted")]
    Aborted,
}
