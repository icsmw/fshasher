#[cfg(feature = "tracking")]
use std::fmt;
use std::path::{Path, PathBuf};

/// `ContextFile` is used to define a rule's file, similar to `.gitignore`.
/// This file will be used to obtain rules for filtering.
///
/// # Type Parameters
///
/// * `T` - A type that can be referenced as a string slice.
#[derive(Debug)]
pub enum ContextFile<T: AsRef<str>> {
    /// All rules in the file will be used as ignore rules. If the path matches,
    /// it will be ignored.
    ///
    /// # Note
    ///
    /// Ignore rules are used regularly. This means the rule will be applied to the full path:
    /// both to check folder paths and file paths.
    Ignore(T),
    /// All rules in the file will be used as accept rules. If the path matches,
    /// it will be accepted. If this rule from the file doesn't match, the file will be ignored.
    ///
    /// # Note
    ///
    /// Accept rules are used in a non-regular way. This means the rule will be applied only
    /// to file paths; folder path checks will be skipped.
    Accept(T),
}

impl<T: AsRef<str>> From<ContextFile<T>> for ContextFileAccepted {
    /// Converts the `ContextFile` into a `ContextFileAccepted`.
    ///
    /// # Arguments
    ///
    /// * `val` - A `ContextFile` instance to convert.
    ///
    /// # Returns
    ///
    /// - A `ContextFileAccepted` instance.
    fn from(val: ContextFile<T>) -> Self {
        match val {
            ContextFile::Ignore(s) => ContextFileAccepted::Ignore(s.as_ref().to_string()),
            ContextFile::Accept(s) => ContextFileAccepted::Accept(s.as_ref().to_string()),
        }
    }
}

/// `ContextFileAccepted` represents a processed `ContextFile` with resolved rules.
///
/// This enum holds rules as strings for either ignoring or accepting paths.
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum ContextFileAccepted {
    /// Represents an ignore rule.
    Ignore(String),
    /// Represents an accept rule.
    Accept(String),
}

#[cfg(feature = "tracking")]
impl fmt::Display for ContextFileAccepted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ignore(s) => s.as_str(),
                Self::Accept(s) => s.as_str(),
            }
        )
    }
}

impl ContextFileAccepted {
    /// Returns the filename associated with the `ContextFileAccepted`.
    ///
    /// # Returns
    ///
    /// - A string slice representing the filename.
    fn filename(&self) -> &str {
        match self {
            Self::Accept(s) => s,
            Self::Ignore(s) => s,
        }
    }

    /// Constructs the full file path and checks if it exists.
    ///
    /// # Arguments
    ///
    /// * `path` - The base path to which the filename will be appended.
    ///
    /// # Returns
    ///
    /// - An `Option<PathBuf>` which is `Some` if the file exists, or `None` if it does not.
    pub fn filepath<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        let filepath = path.as_ref().join(self.filename());
        if filepath.exists() {
            Some(filepath)
        } else {
            None
        }
    }
}
