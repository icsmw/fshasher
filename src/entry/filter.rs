pub use super::E;
use glob::Pattern;
#[cfg(feature = "tracking")]
use std::fmt;
use std::path::Path;

/// With `Filter`, a glob pattern can be applied to a file's name or a folder's name only,
/// whereas a regular glob pattern is applied to the full path. This allows for more accurate filtering.
#[derive(Debug)]
pub enum Filter<T: AsRef<str>> {
    /// A glob pattern that will be applied to a folder's name only.
    Folders(T),
    /// A glob pattern that will be applied to a file's name only.
    Files(T),
    /// A glob pattern that will be applied to the full path (regular usage of glob patterns).
    Common(T),
}

impl<T: AsRef<str>> Filter<T> {
    /// Returns the glob pattern as a string slice.
    ///
    /// # Returns
    ///
    /// - `&str`: The glob pattern as a string slice.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Files(s) => s.as_ref(),
            Self::Folders(s) => s.as_ref(),
            Self::Common(s) => s.as_ref(),
        }
    }
}

impl<T: AsRef<str>> TryInto<FilterAccepted> for Filter<T> {
    type Error = E;

    /// Converts the `Filter` into a `FilterAccepted`.
    ///
    /// # Returns
    ///
    /// - `Result<FilterAccepted, Self::Error>`: A `FilterAccepted` instance or an error if the pattern is invalid.
    fn try_into(self) -> Result<FilterAccepted, Self::Error> {
        let pattern =
            Pattern::new(self.as_str()).map_err(|err| (self.as_str().to_string(), err))?;
        Ok(match self {
            Self::Files(..) => FilterAccepted::Files(pattern),
            Self::Folders(..) => FilterAccepted::Folders(pattern),
            Self::Common(..) => FilterAccepted::Common(pattern),
        })
    }
}

/// Represents an accepted filter with a compiled glob pattern.
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum FilterAccepted {
    /// A compiled glob pattern applied to folder names.
    Folders(Pattern),
    /// A compiled glob pattern applied to file names.
    Files(Pattern),
    /// A compiled glob pattern applied to the full path.
    Common(Pattern),
}

#[cfg(feature = "tracking")]
impl fmt::Display for FilterAccepted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Folders(p) => p.as_str(),
                Self::Files(p) => p.as_str(),
                Self::Common(p) => p.as_str(),
            }
        )
    }
}

impl FilterAccepted {
    /// Filters a given path based on the compiled glob pattern.
    ///
    /// # Parameters
    ///
    /// - `full_path`: The path to be filtered.
    ///
    /// # Returns
    ///
    /// - `Option<bool>`: `Some(true)` if the path matches the pattern, `Some(false)` if it doesn't,
    ///   or `None` if the path type does not match the filter (e.g., a file filter applied to a directory).
    pub fn filtered<P: AsRef<Path>>(&self, full_path: P) -> Option<bool> {
        let path = match self {
            Self::Files(..) => Path::new(full_path.as_ref().file_name()?),
            Self::Folders(..) => full_path.as_ref(),
            Self::Common(..) => full_path.as_ref(),
        };
        Some(
            match self {
                Self::Files(p) => {
                    if full_path.as_ref().is_file() {
                        p
                    } else {
                        return None;
                    }
                }
                Self::Folders(p) => {
                    if full_path.as_ref().is_dir() {
                        p
                    } else {
                        return None;
                    }
                }
                Self::Common(p) => p,
            }
            .matches_path(path),
        )
    }
}
