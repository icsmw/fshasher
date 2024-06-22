use glob::PatternError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("Fail to parse pattern {0}: {1}")]
    PatternError(String, PatternError),
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
    #[error("Channel \"{0}\" isn't available")]
    ChannelErr(String),
}

impl From<(String, PatternError)> for E {
    fn from(err: (String, PatternError)) -> Self {
        E::PatternError(err.0, err.1)
    }
}
