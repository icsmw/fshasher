use crate::{collector, entry};
use glob::PatternError;
use std::{io, path::PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("Fail to parse pattern {0}: {1}")]
    PatternError(String, PatternError),
    #[error("Fail to parse path {0}: {1}")]
    IOError(PathBuf, io::Error),
    #[error("Path {0} cannot be included into a list of targets")]
    InvalidEntity(PathBuf),
    #[error("Path {0} cannot be included into a list of targets. Only files and folders can be included")]
    OnlyFileOrFolder(PathBuf),
    #[error("Relative path {0} cannot be used as entry. ")]
    RelativePathAsEntry(PathBuf),
    #[error("Absolute path {0} cannot be used as filter (included/excluded).")]
    AbsolutePathAsFilter(String),
    #[error("Path {0} cannot be used as cwd because it isn't folder")]
    OnlyFolderAsCwd(PathBuf),
    #[error("Path {0} cannot be used because it isn't absolute")]
    AbsolutePathRequired(PathBuf),
    #[error("Operation has been aborted")]
    Aborted,
    #[error("Invalid number of threads for collecting and hashing")]
    InvalidNumberOfThreads,
    #[error(
        "Not optimal number of threads for collecting and hashing. Twice more than cores number"
    )]
    NotOptimalNumberOfThreads,
    #[error("File doesn't exist: {0}")]
    FileDoesNotExists(PathBuf),
    #[error("Walker isn't inited")]
    IsNotInited,
    #[error("Reader error: {0}")]
    Reader(String),
    #[error("Hasher error: {0}")]
    Hasher(String),
    #[error("Reading IO error: {0}")]
    ReadingIOError(io::Error),
    #[error("Fail to get access to data between threads: {0}")]
    PoisonError(String),
    #[error("Channel error: {0}")]
    ChannelError(String),
    #[error("Collector error: {0}")]
    CollectorError(collector::E),
    #[error("Entry error: {0}")]
    EntryError(entry::E),
    #[error("Fail to get optimal threads number")]
    OptimalThreadsNumber,
    #[error("No available workers")]
    NoAvailableWorkers,
    #[error("Error hashing file {0}: {1}")]
    Bound(PathBuf, Box<Self>),
    #[error("Fail get feedback from main hashing thread: {0}")]
    JoinError(String),
    #[error("Ranges for reading strategy \"scenario\" doesn't cover file size: {0}")]
    NoRangeForScenarioStrategy(u64),
    #[error(
        "Break between ranges for reading strategy \"scenario\"; no scenario for size from: {0}"
    )]
    InvalidRangesForScenarioStrategy(u64),
    #[error("Nested ReadingStrategy::Scenario isn't allowed")]
    NestedScenarioStrategy,
}

impl E {
    pub fn reader<Er: std::error::Error>(err: Er) -> E {
        E::Reader(err.to_string())
    }
    pub fn hasher<Er: std::error::Error>(err: Er) -> E {
        E::Hasher(err.to_string())
    }
}

impl From<(String, PatternError)> for E {
    fn from(err: (String, PatternError)) -> Self {
        E::PatternError(err.0, err.1)
    }
}

impl From<collector::E> for E {
    fn from(err: collector::E) -> Self {
        if matches!(err, collector::E::Aborted) {
            E::Aborted
        } else {
            E::CollectorError(err)
        }
    }
}

impl From<entry::E> for E {
    fn from(err: entry::E) -> Self {
        E::EntryError(err)
    }
}

impl From<io::Error> for E {
    fn from(err: io::Error) -> Self {
        E::ReadingIOError(err)
    }
}

impl From<(PathBuf, io::Error)> for E {
    fn from(err: (PathBuf, io::Error)) -> Self {
        E::IOError(err.0, err.1)
    }
}
