use super::{Entry, Filter, Walker, E};
use crate::collector::Tolerance;
#[cfg(feature = "tracking")]
use crate::walker::tracking::get_default_path;
#[cfg(feature = "tracking")]
use std::path::PathBuf;
use std::{ops::Range, path::Path, thread};

pub(crate) const MIN_THREADS_COUNT: usize = 1;
pub(crate) const MAX_THREADS_MLT_TO_CORES: usize = 2;

/// Defines the reader's strategy.
#[derive(Debug, Clone, Default)]
pub enum ReadingStrategy {
    /// Each file will be read in the "classic" way using a limited size buffer, chunk by chunk until
    /// the end. The hasher will receive small chunks of data to calculate the hash of the file. This strategy
    /// doesn't load the CPU much, but it entails many IO operations.
    #[default]
    Buffer,
    /// With this strategy, the file will be read first and the complete file's content will be passed into
    /// the hasher to calculate the hash. This strategy makes fewer IO operations, but it loads the CPU more.
    Complete,
    /// Instead of reading the file, the reader tries to map the file into memory and give the full content of
    /// the file to the hasher.
    MemoryMapped,
    /// The scenario strategy can be used to combine different strategies according to the file's size.
    Scenario(Vec<(Range<u64>, Box<ReadingStrategy>)>),
}

/// Configuration options for the `Walker`.
#[derive(Default, Debug, Clone)]
pub struct Options {
    /// Tolerance level for error handling. The level of tolerance is used in the scope of collecting files
    /// and hashing files. In case of collecting, IO errors could be ignored (for example, errors related to
    /// permission issues); in case of hashing, hasher's and reader's errors could be ignored (for example, if
    /// a file doesn't exist anymore).
    pub tolerance: Tolerance,

    /// List of entries (paths) to be processed.
    pub entries: Vec<Entry>,

    /// Global entry settings that apply to all entries.
    pub global: Entry,

    /// Optional capacity for progress tracking. Recommended capacity is 10.
    pub progress: Option<usize>,

    /// Optional number of threads to use for processing. If this setting is not set, the number of threads
    /// will default to the number of available cores.
    pub threads: Option<usize>,

    /// Strategy for reading files.
    pub reading_strategy: ReadingStrategy,

    /// A path to store states of checks. Used with "tracking" feature
    #[cfg(feature = "tracking")]
    pub storage: PathBuf,
}

impl Options {
    /// Creates a new instance of `Options` with default settings.
    ///
    /// # Returns
    ///
    /// - A new instance of `Options`.
    pub fn new() -> Self {
        Self {
            tolerance: Tolerance::LogErrors,
            entries: Vec::new(),
            global: Entry::default(),
            progress: None,
            threads: None,
            reading_strategy: ReadingStrategy::default(),
            #[cfg(feature = "tracking")]
            storage: get_default_path(),
        }
    }

    /// Creates a new instance of `Options` from the given path.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to initialize the `Options` with.
    ///
    /// # Returns
    ///
    /// - `Result<Self, E>`: A new instance of `Options` or an error if the path is invalid.
    pub fn from<P: AsRef<Path>>(path: P) -> Result<Self, E> {
        Ok(Self {
            tolerance: Tolerance::LogErrors,
            entries: vec![Entry::from(path)?],
            global: Entry::default(),
            progress: None,
            threads: None,
            reading_strategy: ReadingStrategy::default(),
            #[cfg(feature = "tracking")]
            storage: get_default_path(),
        })
    }

    /// Sets the reading strategy for the `Walker`.
    ///
    /// # Parameters
    ///
    /// - `reading_strategy`: The reading strategy to use.
    ///
    /// # Returns
    ///
    /// - `Result<Self, E>`: An instance of `Options` an error if the strategy is invalid.
    pub fn reading_strategy(mut self, reading_strategy: ReadingStrategy) -> Result<Self, E> {
        if let ReadingStrategy::Scenario(scenario) = &reading_strategy {
            let mut from = 0;
            for (range, strategy) in scenario.iter() {
                if matches!(**strategy, ReadingStrategy::Scenario(_)) {
                    return Err(E::NestedScenarioStrategy);
                }
                if range.start != from {
                    return Err(E::InvalidRangesForScenarioStrategy(from));
                }
                from = range.end;
            }
        }
        self.reading_strategy = reading_strategy;
        Ok(self)
    }

    /// Sets the number of threads to use for collecting and hashing.
    ///
    /// # Parameters
    ///
    /// - `threads`: The number of threads to use.
    ///
    /// # Returns
    ///
    /// - `Result<Self, E>`: An instance of `Options` or an error.
    pub fn threads(mut self, threads: usize) -> Result<Self, E> {
        if threads < MIN_THREADS_COUNT {
            return Err(E::InvalidNumberOfThreads);
        }
        if let Some(cores) = thread::available_parallelism().ok().map(|n| n.get()) {
            if threads > cores * MAX_THREADS_MLT_TO_CORES {
                return Err(E::NotOptimalNumberOfThreads);
            }
        }
        self.threads = Some(threads);
        Ok(self)
    }

    /// Sets the capacity for progress tracking.
    ///
    /// # Parameters
    ///
    /// - `capacity`: The capacity for the progress tracker.
    ///
    /// # Returns
    ///
    /// - An `Options` instance.
    pub fn progress(mut self, capacity: usize) -> Self {
        self.progress = Some(capacity);
        self
    }

    /// Sets the tolerance level for error handling. Only collecting paths is sensitive to
    /// the tolerance level. Hashing is not sensitive to it.
    ///
    /// # Parameters
    ///
    /// - `tolerance`: The tolerance level to use.
    ///
    /// # Returns
    ///
    /// - An `Options` instance.
    pub fn tolerance(mut self, tolerance: Tolerance) -> Self {
        self.tolerance = tolerance;
        self
    }

    /// Adds a path to the list of entries to be processed.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to add.
    ///
    /// # Returns
    ///
    /// - `Result<Self, E>`: An instance of `Options` or an error if the path is invalid.
    pub fn path<P: AsRef<Path>>(mut self, path: P) -> Result<Self, E> {
        self.entries.push(Entry::from(path)?);
        Ok(self)
    }

    /// Adds an entry to the list of entries to be processed.
    ///
    /// # Parameters
    ///
    /// - `entry`: The entry to add.
    ///
    /// # Returns
    ///
    /// - `Result<Self, E>`: An instance of `Options` or an error if the entry is invalid.
    pub fn entry(mut self, entry: Entry) -> Result<Self, E> {
        if !entry.entry.is_absolute() {
            return Err(E::RelativePathAsEntry(entry.entry));
        }
        self.entries.push(entry);
        Ok(self)
    }

    /// Adds an include filter to the global entry.
    ///
    /// # Parameters
    ///
    /// - `filter`: The filter to add.
    ///
    /// # Returns
    ///
    /// - `Result<Self, E>`: An instance of `Options` or an error if the filter is invalid.
    pub fn include<T: AsRef<str>>(mut self, filter: Filter<T>) -> Result<Self, E> {
        self.global = self.global.include(filter)?;
        Ok(self)
    }

    /// Adds an exclude filter to the global entry.
    ///
    /// # Parameters
    ///
    /// - `filter`: The filter to add.
    ///
    /// # Returns
    ///
    /// - `Result<Self, E>`: An instance of `Options` or an error if the filter is invalid.
    pub fn exclude<T: AsRef<str>>(mut self, filter: Filter<T>) -> Result<Self, E> {
        self.global = self.global.exclude(filter)?;
        Ok(self)
    }

    /// Sets the path to the storage for saving hashes for the "tracking" feature. `fshasher` will use this storage to save hashes and later compare
    /// them with new hashes to determine if changes have occurred.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to the storage folder. If it doesn't exist, it will be created.
    ///
    /// # Returns
    ///
    /// - `Result<Self, E>`: An instance of `Options` or an error if failed to create the storage folder.
    #[cfg(feature = "tracking")]
    pub fn storage<P: AsRef<Path>>(mut self, path: P) -> Result<Self, E> {
        use crate::walker::tracking::get_storage_name;
        use std::fs::create_dir_all;

        let path = path.as_ref().to_path_buf().join(get_storage_name());
        if !path.exists() {
            create_dir_all(&path)?;
        }
        self.storage = path;
        Ok(self)
    }

    /// Returns own hash based on entries
    ///
    /// # Returns
    ///
    /// - `String` - hash calculated based on entries
    #[cfg(feature = "tracking")]
    pub fn hash(&self) -> Vec<u8> {
        format!(
            "{}:{}",
            self.entries
                .iter()
                .map(|en| en.to_string())
                .collect::<Vec<String>>()
                .join(";"),
            self.global
        )
        .as_bytes()
        .to_vec()
    }
    /// Creates a `Walker` with the specified hasher and reader.
    ///
    /// # Parameters
    ///
    /// - `hasher`: The hasher to use.
    /// - `reader`: The reader to use.
    ///
    /// # Returns
    ///
    /// - `Result<Walker<H, R>, E>`: A new `Walker` instance or an error if the creation fails.
    pub fn walker(self) -> Result<Walker, E> {
        Ok(Walker::new(self))
    }
}
