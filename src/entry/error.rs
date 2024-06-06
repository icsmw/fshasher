use glob::PatternError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("Fail to parse pattern {0}: {1}")]
    PatternError(String, PatternError),
    #[error("Path {0} cannot be included into a list of targets. Only files and folders can be included")]
    OnlyFileOrFolder(PathBuf),
    #[error("Relative path {0} cannot be used as entry. ")]
    RelativePathAsEntry(PathBuf),
    #[error("Absolute path {0} cannot be used as filter (included/excluded).")]
    AbsolutePathAsFilter(String),
    #[error("Path {0} cannot be used as cwd because it isn't folder")]
    OnlyFolderAsCwd(PathBuf),
}

impl From<(String, PatternError)> for E {
    fn from(err: (String, PatternError)) -> Self {
        E::PatternError(err.0, err.1)
    }
}
