mod context;
mod error;
mod filter;
mod pattern;

pub use context::{ContextFile, ContextFileAccepted};
pub use error::E;
pub use filter::Filter;
pub(crate) use filter::FilterAccepted;
pub use pattern::PatternFilter;
pub(crate) use pattern::PatternFilterAccepted;
#[cfg(feature = "tracking")]
use std::fmt;
use std::path::{Path, PathBuf};

/// Represents an entry with filtering options for file and directory paths. `Entry` provides a powerful
/// way to configure the filtering of file paths that should be collected and hashed.
///
/// Ways to use `Entry`:
///
/// - No filters; take all files from the destination folder. `Entry` can be used without any filtering.
///   It is sufficient to assign `Entry` with a path to an existing folder. `Walker` will take this path
///   and collect and hash (recursively) all files without filtering.
///
/// - Filtering by target's nature. Depending on what you want to filter, file or folder, you can add
///   filters to include or exclude items using the methods `include(..)` and `exclude(..)`. Both methods
///   take a `Filter` enum as an argument, which can be used to assign a filter based on the target's nature.
///
///   - `Filter::Files("*key_word_in_file_name*")` will be applied to each found file name.
///   - `Filter::Folders("*key_word_in_folder_name*")` will be applied to each found folder name.
///   - `Filter::Common("*key_word*")` will apply the filter to any target to full path.
///
/// Excluding filters have higher priority than including filters. If a target matches an excluding filter, the target
/// will be ignored even if it matches an including filter.
///
/// The argument of the filter is a glob pattern.
///
/// - Direct filtering by glob pattern. Using this method, you can define positive and negative glob patterns
///   with `PatternFilter`.
///
///   - `PatternFilter::Ignore("**/*/*.ts")` - ignore all paths (both folders and files) if they match
///      the glob pattern.
///   - `PatternFilter::Accept("**/*/*.ts")` - include all paths (both folders and files) if they match
///     the glob pattern.
///   - `PatternFilter::Cmb(vec![PatternFilter::Ignore("**/*/*.ts"), PatternFilter::Ignore("**/*/*.tjs")])` -
///     allows creating a combination of patterns. `PatternFilter::Cmb(..)` doesn't support nested combinations;
///     attempting to nest another `PatternFilter::Cmb(..)` inside will cause an error.
///
///
/// The difference between "Filtering by target's nature" and "Direct filtering by glob pattern" is: in the
/// first case, the glob pattern can be applied to file names only or folder names only, while in the
/// second case, the glob pattern will always be applied to the full path.
///
/// `PatternFilter` filters have higher priority than `Filter`. If at least one `PatternFilter` is defined,
/// any `Filter` will be ignored.
#[derive(Default, Debug, Clone)]
pub struct Entry {
    /// The path of the entry.
    pub entry: PathBuf,
    /// A list of filters for including paths.
    pub include: Vec<FilterAccepted>,
    /// A list of filters for excluding paths.
    pub exclude: Vec<FilterAccepted>,
    /// A list of patterns for filtering paths.
    pub patterns: Vec<PatternFilterAccepted>,
    pub context: Vec<ContextFileAccepted>,
}

#[cfg(feature = "tracking")]
impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{};({});({});({});({});",
            self.entry.display(),
            self.include
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(";"),
            self.exclude
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(";"),
            self.patterns
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(";"),
            self.context
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<String>>()
                .join(";")
        )
    }
}

impl Entry {
    /// Creates a new `Entry` instance with default values.
    ///
    /// # Returns
    ///
    /// - A new instance of `Entry`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `Entry` instance from a given path. It's simplest way to initialize `Walker`
    /// to collect and hash files from some folder.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to initialize the `Entry` with.
    ///
    /// # Returns
    ///
    /// - `Result<Self, E>`: A new instance of `Entry` or an error if the path is invalid.
    ///
    /// # Examples
    /// ```
    ///     use fshasher::{hasher, reader, Entry, Options};
    ///     use std::{
    ///         env::temp_dir,
    ///         fs::{create_dir, remove_dir},
    ///     };
    ///     use uuid::Uuid;
    ///
    ///     let entry = temp_dir().join(Uuid::new_v4().to_string());
    ///     let _ = create_dir(&entry);
    ///
    ///     let mut walker = Options::new()
    ///         .entry(Entry::from(&entry).unwrap())
    ///         .unwrap()
    ///         .walker();
    ///     // ... some work here
    ///     let _ = remove_dir(&entry);
    /// ```
    pub fn from<T: AsRef<Path>>(path: T) -> Result<Self, E> {
        Self::new().entry(path)
    }

    /// Sets the entry path. The entry path is the folder from which files will be collected and hashed.
    /// You can define multiple instances of `Entry` with unique entry folders. In this case, `Walker` will
    /// collect file paths from all entry folders (recursively) and calculate a common hash for all of
    /// them.
    ///
    /// # Examples
    ///
    /// In the example below, a `Walker` is created to calculate the hash for `/few/A`, `/few/B`, and `/few/C`:
    /// ```
    /// use fshasher::{hasher, reader, Entry, Options};
    /// use std::{
    ///     env::temp_dir,
    ///     fs::{create_dir, remove_dir},
    ///     path::PathBuf,
    /// };
    /// use uuid::Uuid;
    ///
    /// let entries: Vec<PathBuf> = (0..3)
    ///     .map(|_| temp_dir().join(Uuid::new_v4().to_string()))
    ///     .collect();
    /// let mut opt = Options::new();
    /// for p in entries.iter() {
    ///     let _ = create_dir(p);
    ///     opt = opt.entry(Entry::from(p).unwrap()).unwrap();
    /// }
    ///
    /// let mut walker = opt.walker();
    /// // ... some work here
    /// entries.iter().for_each(|p| {
    ///     let _ = remove_dir(p);
    /// });
    /// ```
    ///
    /// # Parameters
    ///
    /// - `path`: The path to set as the entry.
    ///
    /// # Returns
    ///
    /// - `Result<&mut Self, E>`: A modified `Entry` instance or an error if
    ///   the path is invalid.
    ///
    /// # Errors
    ///
    /// - The entry path will be accepted only if it is a path to an existing folder; in all other cases, it will
    ///   cause an error.
    pub fn entry<T: AsRef<Path>>(mut self, path: T) -> Result<Self, E> {
        let path = path.as_ref().to_path_buf();
        if !path.is_absolute() {
            return Err(E::RelativePathAsEntry(path));
        } else if !path.is_dir() {
            return Err(E::OnlyFolderAsCwd(path));
        }
        self.entry = path;
        Ok(self)
    }

    /// Adds a context file (like `.gitignore`) which will be used to obtain patterns for filtering content.
    /// Nested context files are also considered.
    ///
    /// # Arguments
    ///
    /// * `context` - A `ContextFile` name of context file.
    ///
    ///   * `ContextFile::Ignore` - All rules in the file will be used as ignore rules. If the path matches,
    ///     it will be ignored. Ignore rules are used regularly. This means the rule will be applied to the
    ///     full path: both to check folder paths and file paths.
    ///   * `ContextFile::Accept` - All rules in the file will be used as accept rules. If the path matches,
    ///     it will be accepted. If this rule from the file doesn't match, the file will be ignored. Accept
    ///     rules are used in a non-regular way. This means the rule will be applied only to file paths;
    ///     folder path checks will be skipped.
    ///
    /// # Returns
    ///
    /// - The modified instance of the struct with the new context file added to the list.
    ///
    /// # Example
    ///
    /// ```
    /// use fshasher::{Entry, Options, ContextFile};
    /// use std::{
    ///     env::temp_dir,
    /// };
    ///
    /// let mut opt = Options::new();
    /// let mut walker = Options::new().entry(
    ///     Entry::new()
    ///         .entry(temp_dir())
    ///         .unwrap()
    ///         .context(
    ///             ContextFile::Ignore(".gitignore")
    ///         )
    ///     ).unwrap().walker().unwrap();
    /// let _ = walker.collect().unwrap();
    /// ```
    pub fn context<T: AsRef<str>>(mut self, context: ContextFile<T>) -> Self {
        let accepted: ContextFileAccepted = context.into();
        if !self.context.contains(&accepted) {
            self.context.push(accepted);
        }
        self
    }

    /// Adds an include filter to the entry.
    ///
    /// # Parameters
    ///
    /// - `filter`: The filter to add.
    ///
    /// # Returns
    ///
    /// - `Result<&mut Self, E>`: A modified `Entry` instance or an error if the filter is invalid.
    pub fn include<T: AsRef<str>>(mut self, filter: Filter<T>) -> Result<Self, E> {
        let accepted: FilterAccepted = filter.try_into()?;
        if !self.include.contains(&accepted) && !self.exclude.contains(&accepted) {
            self.include.push(accepted);
        }
        Ok(self)
    }

    /// Adds an exclude filter to the entry.
    ///
    /// # Parameters
    ///
    /// - `filter`: The filter to add.
    ///
    /// # Returns
    ///
    /// - `Result<&mut Self, E>`: A modified `Entry` instance or an error if the filter is invalid.
    pub fn exclude<T: AsRef<str>>(mut self, filter: Filter<T>) -> Result<Self, E> {
        let accepted: FilterAccepted = filter.try_into()?;
        if !self.include.contains(&accepted) && !self.exclude.contains(&accepted) {
            self.exclude.push(accepted);
        }
        Ok(self)
    }

    /// Adds a pattern filter to the entry.
    ///
    /// # Parameters
    ///
    /// - `pattern`: The pattern filter to add.
    ///
    /// # Returns
    ///
    /// - `Result<&mut Self, E>`: A modified `Entry` instance or an error if the pattern is invalid.
    pub fn pattern<T: AsRef<str>>(mut self, pattern: PatternFilter<T>) -> Result<Self, E> {
        let accepted: PatternFilterAccepted = pattern.try_into()?;
        if !self.patterns.contains(&accepted) {
            self.patterns.push(accepted);
        }
        Ok(self)
    }

    /// Filters a given path based on the entry's include, exclude, and pattern filters.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to filter.
    ///
    /// # Returns
    ///
    /// - `bool`: `true` if the path is accepted, `false` otherwise.
    pub fn filtered<P: AsRef<Path>>(&self, path: P) -> bool {
        if !self.patterns.is_empty() {
            return self.patterns.iter().any(|pattern| pattern.filtered(&path));
        } else if self
            .exclude
            .iter()
            .any(|filter| filter.filtered(&path).unwrap_or_default())
        {
            return false;
        }
        if self.include.is_empty() {
            true
        } else {
            self.include
                .iter()
                .any(|filter| filter.filtered(&path).unwrap_or(true))
        }
    }
}
