pub use super::E;
use glob::Pattern;
use std::path::Path;

const MAX_DEPTH: usize = 1;

/// Allows applying a glob pattern in a regular way. With `PatternFilter`, a glob pattern will be applied to
/// the full path in contrast to `Filter`.
#[derive(Debug)]
pub enum PatternFilter<T: AsRef<str>> {
    /// If the given glob pattern matches, the path will be ignored.
    Ignore(T),
    /// If the given glob pattern matches, the path will be included.
    Accept(T),
    /// Allows defining a combination of `PatternFilter`. `PatternFilter::Cmb(..)` doesn't support nested
    /// combinations; attempting to nest another `PatternFilter::Cmb(..)` inside will cause an error.
    Cmb(Vec<PatternFilter<T>>),
}

impl<T: AsRef<str>> TryInto<PatternFilterAccepted> for PatternFilter<T> {
    type Error = E;

    /// Converts the `PatternFilter` into a `PatternFilterAccepted`.
    ///
    /// # Returns
    ///
    /// - `Result<PatternFilterAccepted, Self::Error>`: A `PatternFilterAccepted` instance or an error if the pattern is invalid.
    fn try_into(self) -> Result<PatternFilterAccepted, Self::Error> {
        Ok(match self {
            Self::Ignore(s) => PatternFilterAccepted::Ignore(
                Pattern::new(s.as_ref())
                    .map_err(|err: glob::PatternError| (s.as_ref().to_string(), err))?,
            ),
            Self::Accept(s) => PatternFilterAccepted::Accept(
                Pattern::new(s.as_ref())
                    .map_err(|err: glob::PatternError| (s.as_ref().to_string(), err))?,
            ),
            Self::Cmb(filters) => {
                let mut patterns: Vec<PatternFilterAccepted> = Vec::new();
                for filter in filters {
                    patterns.push(filter.try_into()?);
                }
                PatternFilterAccepted::Cmb(patterns)
            }
        })
    }
}

/// Represents an accepted pattern filter with a compiled glob pattern.
#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum PatternFilterAccepted {
    /// A compiled glob pattern that ignores paths matching the pattern.
    Ignore(Pattern),
    /// A compiled glob pattern that accepts paths matching the pattern.
    Accept(Pattern),
    /// A combination of multiple `PatternFilterAccepted` patterns.
    Cmb(Vec<PatternFilterAccepted>),
}

impl PatternFilterAccepted {
    /// Filters a given path based on the compiled glob pattern.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to be filtered.
    ///
    /// # Returns
    ///
    /// - `bool`: `true` if the path matches the pattern criteria, `false` otherwise.
    pub fn filtered<P: AsRef<Path>>(&self, path: P) -> bool {
        self.filtered_with_depth(path.as_ref(), 0)
    }

    /// Filters a given path with a specified depth to prevent nested combinations.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to be filtered.
    /// - `depth`: The current depth of nested patterns.
    ///
    /// # Panics
    ///
    /// This method will panic if the depth exceeds `MAX_DEPTH` (default value 1).
    ///
    /// # Returns
    ///
    /// - `bool`: `true` if the path matches the pattern criteria, `false` otherwise.
    fn filtered_with_depth(&self, path: &Path, depth: usize) -> bool {
        if depth > MAX_DEPTH {
            panic!(
                "PatternFilterAccepted::Cmb cannot have nested PatternFilterAccepted::Cmb elements"
            );
        }
        match self {
            Self::Ignore(p) => !p.matches_path(path),
            Self::Accept(p) => p.matches_path(path),
            Self::Cmb(patterns) => !patterns
                .iter()
                .any(|p| !p.filtered_with_depth(path, depth + 1)),
        }
    }
}
// !*.ts
