mod error;
pub(crate) mod options;
mod pool;
mod progress;
mod worker;

use crate::{
    collector::collect,
    entry::{Entry, Filter},
    Breaker, Hasher, Reader, Tolerance,
};
pub use error::E;
use log::{debug, error, warn};
pub use options::{Options, ReadingStrategy};
use pool::Pool;
pub use progress::{JobType, Progress, ProgressChannel, Tick};
use std::{
    io, mem,
    path::PathBuf,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
    time::Instant,
};
pub use worker::Worker;

/// The default minimum number of paths that will be given to a hash worker to calculate hashes.
const MIN_PATHS_PER_JOB: usize = 2;
/// The default maximum number of paths that will be given to a hash worker to calculate hashes.
const MAX_PATHS_PER_JOB: usize = 500;

enum JobCollecting {
    Err(E),
    NoJobs,
    Success,
}

/// Message for communication between `Walker` and workers during hashing.
pub enum Action {
    /// Used by workers to report the results of hashing files to `Walker`.
    ///
    /// # Parameters
    ///
    /// - `Vec<(PathBuf, H)>`: A vector of tuples where each tuple contains
    ///   a file path and its corresponding hash.
    Processed(Vec<(PathBuf, Vec<u8>)>, Vec<(PathBuf, E)>),

    /// Used by workers to notify `Walker` about the closing of a worker's thread.
    WorkerShutdownNotification,

    /// Used by workers to report an error.
    ///
    /// # Parameters
    ///
    /// - `PathBuf`: The path to the file that caused the error.
    /// - `E`: The error encountered during processing.
    Error(PathBuf, E),
}

type HashItem = (PathBuf, Option<Result<Vec<u8>, E>>);

/// `Walker` collects file paths according to a given pattern, then calculates the hash for each
/// file and provides a combined hash for all files.
///
/// `Walker` collects file paths recursively, traversing all nested folders. If a symlink is
/// encountered, `Walker` reads it and, if it is linked to a file, includes it. If the symlink
/// leads to a folder, `Walker` reads this folder as nested. Other entities (except for files,
/// folders, and symlinks) are ignored.
///
/// The operations of path collection and hashing are separated. Reading the folder and collecting
/// paths is done using the `collect()` method; hashing is done with the `hash()` method. If
/// `hash()` is called without a prior call to `collect()`, it will not result in an error but
/// will return an empty hash.
///
/// The `progress()` method provides a `Receiver<Tick>` to track the progress of the operation.
/// A repeated call to `hash()` will recreate the channel. Therefore, before calling `hash()`
/// again, you need to get a new `Receiver<Tick>` using the `progress()` method.
///
/// Path collection and subsequent hashing are interruptible operations. To interrupt, you need to
/// get a `Breaker` by calling the `breaker()` method. Interruption is done by calling the `abort()`
/// method. The operation will be interrupted at the earliest possible time but not instantaneously.
///
/// In case of interruption, both the `collect()` and `hash()` methods will return an `E::Aborted`
/// error.
///
/// The built-in interruption function can be used to implement timeout support.
///
/// When an instance of `E::Aborted` is dropped, the background threads are not stopped automatically.
/// To stop all running background threads, you need to use `Breaker` and call `abort()`. Otherwise,
/// there is a risk of resource leakage.
///
/// The most efficient way to create an instance of `Walker` is to use `Options`, which allows
/// flexible configuration of `Walker`.
///
/// # Example
///
/// ```
/// use fshasher::{Options, Entry, Tolerance, hasher, reader};
/// use std::env::temp_dir;
///
/// let mut walker = Options::new()
///     .entry(Entry::from(temp_dir()).unwrap()).unwrap()
///     .tolerance(Tolerance::LogErrors)
///     .walker(
///         hasher::blake::Blake::default(),
///         reader::buffering::Buffering::default(),
///     ).unwrap();
/// println!("Hash of {}: {:?}", temp_dir().display(), walker.collect().unwrap().hash().unwrap())
/// ```
///
#[derive(Debug)]
pub struct Walker {
    /// Settings for the `Walker`.
    opt: Option<Options>,
    /// `Breaker` structure for interrupting the path collection and hashing operations.
    breaker: Breaker,
    /// Paths collected during the recursive traversal of the paths specified in `Options` and
    /// according to the patterns. This field is populated when `collect()` is called.
    ///
    /// # Fields
    ///
    /// * `PathBuf` - full filepath
    /// * `Option<Result<Vec<u8>, E>>` - can have next values:
    ///   * `None` - right after `collect()` has been called
    ///   * `Result<Vec<u8>, E>` - result of hashing; if hashing was failed, keep related error
    pub paths: Vec<HashItem>,
    /// The resulting hash. Set when `hash()` is called.
    hash: Option<Vec<u8>>,
    /// An instance of the channel for tracking the progress of path collection and hashing.
    progress: Option<ProgressChannel>,
}

impl Walker {
    /// Creates a new instance of `Walker`.
    ///
    /// # Parameters
    ///
    /// - `opt`: An instance of `Options` containing the configuration for `Walker`.
    /// - `hasher`: An instance of the hasher that will be used for hashing each found file.
    /// - `reader`: An instance of the reader that will be used for reading each found file.
    ///
    /// # Returns
    ///
    /// - A new instance of `Walker`.
    pub fn new(opt: Options) -> Self {
        let progress = opt.progress.map(Progress::channel);
        Self {
            opt: Some(opt),
            breaker: Breaker::new(),
            paths: Vec::new(),
            hash: None,
            progress,
        }
    }

    /// Collects file paths and saves them in the `paths` field for further hashing.
    ///
    /// # Returns
    ///
    /// - A new instance of `Walker`.
    ///
    /// # Errors
    ///
    /// This method will return an error if the operation is interrupted. By default, `Walker` has
    /// a tolerance level of `Tolerance::LogErrors`, which means that the collection process will
    /// not stop on an IO error; instead, the problematic path will be ignored. To change this strategy,
    /// set the tolerance level to `Tolerance::StopOnErrors`. With `Tolerance::StopOnErrors`, the `collect`
    /// method will return an error for any IO error encountered.
    pub fn collect(&mut self) -> Result<&mut Self, E> {
        let now = Instant::now();
        self.reset();
        let opt = self.opt.as_mut().ok_or(E::IsNotInited)?;
        let progress = self.progress.as_ref().map(|(progress, _)| progress.clone());
        for entry in opt.entries.iter() {
            let (collected, mut invalid) = collect(
                &progress,
                entry,
                &self.breaker,
                &opt.tolerance,
                &opt.threads,
            )?;
            self.paths
                .append(&mut collected.into_iter().map(|p| (p, None)).collect());
        }
        debug!(
            "collected {} paths in {}µs / {}ms / {}s",
            self.paths.len(),
            now.elapsed().as_micros(),
            now.elapsed().as_millis(),
            now.elapsed().as_secs()
        );
        Ok(self)
    }

    /// Returns a `Breaker` which can be used to abort collecting and hashing operations.
    /// Interruption is done by calling the `abort()` method. The operation will be interrupted
    /// at the earliest possible time but not instantaneously.
    ///
    /// In case of interruption, both the `collect()` and `hash()` methods will return an `E::Aborted`
    /// error.
    ///
    /// When an instance of `E::Aborted` is dropped, the background threads are not stopped automatically.
    /// To stop all running background threads, you need to use `Breaker` and call `abort()`. Otherwise,
    /// there is a risk of resource leakage.
    ///
    /// # Returns
    ///
    /// - A new instance of `Breaker`.
    ///
    /// # Example
    ///
    /// ```
    /// use fshasher::{hasher, reader, walker::E, Entry, Options, Tolerance};
    /// use std::{env::temp_dir, thread};
    ///
    /// let mut walker = Options::new()
    ///     .entry(Entry::from(temp_dir()).unwrap())
    ///     .unwrap()
    ///     .tolerance(Tolerance::LogErrors)
    ///     .progress(10)
    ///     .walker(
    ///         hasher::blake::Blake::default(),
    ///         reader::buffering::Buffering::default(),
    ///     )
    ///     .unwrap();
    /// let progress = walker.progress().unwrap();
    /// let breaker = walker.breaker();
    /// thread::spawn(move || {
    ///     let _ = progress.recv();
    ///     // Abort collecting as soon as it's started
    ///     breaker.abort();
    /// });
    /// assert!(matches!(walker.collect().err().unwrap(), E::Aborted));
    /// ```
    pub fn breaker(&self) -> Breaker {
        self.breaker.clone()
    }

    /// This is equal to the number paths, found by `collect()`.
    ///
    /// # Returns
    ///
    /// - The number of calculated hashes.
    pub fn count(&self) -> usize {
        self.paths.len()
    }

    /// Returns a channel for tracking the progress of collecting and hashing. A repeated call to `hash()`
    /// will recreate the channel. Therefore, before calling `hash()` again, you need to get a new
    /// `Receiver<Tick>` using the `progress()` method.
    ///
    /// # Returns
    ///
    /// - `Option<Receiver<Tick>>`: A channel for tracking progress, or `None` if the channel is not available.
    pub fn progress(&mut self) -> Option<Receiver<Tick>> {
        self.progress.as_mut().and_then(|(_, rx)| rx.take())
    }

    /// Calculates a common hash and returns it. `hash()` should always be used in pair with `collect()`,
    /// because `collect()` gathers the paths to files that will be hashed.
    ///
    /// # Returns
    ///
    /// - `Result<&[u8], E>`: A hash calculated based on the paths collected with the given patterns.
    ///
    /// # Errors
    ///
    /// This method, like `collect()`, is sensitive to tolerance settings. By default, `Walker` has
    /// a tolerance level of `Tolerance::LogErrors`, which means that the hashing process will
    /// not stop on an IO error (caused by hasher or reader); instead, the problematic path will be ignored.
    /// To change this strategy, set the tolerance level to `Tolerance::StopOnErrors`. With
    /// `Tolerance::StopOnErrors`, the `hash()` method will return an error for any IO error encountered.
    ///
    /// All ignored paths will be saved into the `invalid` field.

    pub fn hash<H: Hasher + 'static, R: Reader + 'static>(&mut self) -> Result<&[u8], E>
    where
        E: From<<H as Hasher>::Error> + From<<R as Reader>::Error>,
    {
        let now = Instant::now();
        if self.paths.is_empty() {
            return Ok(&[]);
        }
        let opt = self.opt.as_mut().ok_or(E::IsNotInited)?;
        let tolerance = opt.tolerance.clone();
        let (tx_queue, rx_queue): (Sender<Action>, Receiver<Action>) = channel();
        let progress = self.progress.as_ref().map(|(progress, _)| progress.clone());
        let breaker = self.breaker.clone();
        let cores = thread::available_parallelism()
            .ok()
            .map(|n| n.get())
            .ok_or(E::OptimalThreadsNumber)?;
        if let Some(threads) = &opt.threads {
            if cores * options::MAX_THREADS_MLT_TO_CORES < *threads {
                return Err(E::OptimalThreadsNumber);
            }
        }
        let threads = opt.threads.unwrap_or(cores);
        let mut pool: Pool = Pool::new::<H, R>(
            threads,
            tx_queue.clone(),
            &opt.reading_strategy,
            &opt.tolerance,
            &self.breaker,
        );
        debug!("Created pool with {threads} workers for hashing");
        let mut paths = mem::take(&mut self.paths);
        let total = paths.len();
        let paths_per_jobs =
            ((total as f64 * 0.05).ceil() as usize).clamp(MIN_PATHS_PER_JOB, MAX_PATHS_PER_JOB);

        type HashingResult<T> = Result<(T, Vec<HashItem>), E>;

        let handle: JoinHandle<HashingResult<H>> = thread::spawn(move || {
            fn check_err(
                path: PathBuf,
                err: E,
                tolerance: &Tolerance,
                hashes: &mut Vec<HashItem>,
            ) -> Result<(), E> {
                match tolerance {
                    Tolerance::StopOnErrors => {
                        error!("entry: {}; error: {err}", path.display());
                        Err(E::Bound(path, Box::new(err)))
                    }
                    Tolerance::LogErrors => {
                        warn!("entry: {}; error: {err}", path.display());
                        hashes.push((path, Some(Err(err))));
                        Ok(())
                    }
                    Tolerance::DoNotLogErrors => {
                        hashes.push((path, Some(Err(err))));
                        Ok(())
                    }
                }
            }
            fn get_next_job(
                paths: &mut Vec<HashItem>,
                paths_per_jobs: usize,
                tolerance: &Tolerance,
                hashes: &mut Vec<HashItem>,
            ) -> Result<Vec<PathBuf>, E> {
                let mut jobs = Vec::new();
                while jobs.len() < paths_per_jobs && !paths.is_empty() {
                    let (path, _) = paths.remove(0);
                    if !path.exists() {
                        check_err(
                            path,
                            io::Error::new(io::ErrorKind::NotFound, "File not found").into(),
                            tolerance,
                            hashes,
                        )?;
                        continue;
                    }
                    jobs.push(path);
                }
                Ok(jobs)
            }
            fn deligate(
                workers: Vec<&Worker>,
                paths: &mut Vec<HashItem>,
                paths_per_jobs: usize,
                tolerance: &Tolerance,
                hashes: &mut Vec<HashItem>,
            ) -> JobCollecting {
                for (i, worker) in workers.iter().enumerate() {
                    match get_next_job(paths, paths_per_jobs, tolerance, hashes) {
                        Ok(jobs) => {
                            if jobs.is_empty() && i == 0 {
                                // No any worker get a job
                                return JobCollecting::NoJobs;
                            } else if jobs.is_empty() && i != 0 {
                                // At least one worker get a job
                                break;
                            } else {
                                worker.delegate(jobs);
                            }
                        }
                        Err(err) => {
                            return JobCollecting::Err(err);
                        }
                    }
                }
                JobCollecting::Success
            }
            let mut summary = H::new();
            let mut hashes: Vec<HashItem> = Vec::new();
            let initialization = deligate(
                pool.workers(),
                &mut paths,
                paths_per_jobs,
                &tolerance,
                &mut hashes,
            );
            if !matches!(initialization, JobCollecting::Success) {
                pool.shutdown().wait();
                summary.finish()?;
                return if let JobCollecting::Err(err) = initialization {
                    Err(err)
                } else {
                    Ok((summary, hashes))
                };
            }
            let mut pending: Option<Action> = None;
            let outer: Result<(), E> = 'outer: loop {
                let next = if let Some(next) = pending.take() {
                    next
                } else if let Ok(next) = rx_queue.recv() {
                    next
                } else {
                    break 'outer Ok(());
                };
                if breaker.is_aborted() {
                    break 'outer Err(E::Aborted);
                }
                match next {
                    Action::Processed(processed, reports) => {
                        for (path, err) in reports.into_iter() {
                            // If error reported by Worker, it's already not Tolerance::StopOnErrors
                            let _ = check_err(path, err, &tolerance, &mut hashes);
                        }
                        hashes.append(
                            &mut processed
                                .into_iter()
                                .map(|(p, h)| (p, Some(Ok(h))))
                                .collect(),
                        );
                        if let Some(ref progress) = progress {
                            progress.notify(JobType::Hashing, hashes.len(), total)
                        }
                    }
                    Action::WorkerShutdownNotification => {
                        if pool.is_all_down() {
                            if let Ok(next) = rx_queue.try_recv() {
                                pending = Some(next);
                                continue;
                            } else {
                                break 'outer Ok(());
                            }
                        }
                    }
                    Action::Error(path, err) => {
                        if let Err(err) = check_err(path, err, &tolerance, &mut hashes) {
                            break 'outer Err(err);
                        }
                    }
                }
                if pool.is_shutdowning() {
                    continue;
                }
                match deligate(
                    pool.workers(),
                    &mut paths,
                    paths_per_jobs,
                    &tolerance,
                    &mut hashes,
                ) {
                    JobCollecting::Err(err) => {
                        break 'outer Err(err);
                    }
                    JobCollecting::Success => {}
                    JobCollecting::NoJobs => {
                        pool.shutdown();
                    }
                };
            };
            pool.shutdown().wait();
            if let Err(err) = outer {
                Err(err)
            } else {
                hashes.sort_by(|(a, _), (b, _)| a.cmp(b));
                for (_, hash) in hashes.iter() {
                    if let Some(Ok(hash)) = hash {
                        summary.absorb(hash)?;
                    }
                }
                summary.finish()?;
                Ok((summary, hashes))
            }
        });
        self.progress = opt.progress.map(Progress::channel);
        let (summary, mut hashes) = handle
            .join()
            .map_err(|e| E::JoinError(format!("{e:?}")))??;
        self.paths = mem::take(&mut hashes);
        self.hash = Some(summary.hash()?.to_vec());
        self.progress = opt.progress.map(Progress::channel);
        let hash = if let Some(ref hash) = self.hash {
            hash
        } else {
            unreachable!("Hash has been stored");
        };
        debug!(
            "hashing of {} paths in {}µs / {}ms / {}s",
            total,
            now.elapsed().as_micros(),
            now.elapsed().as_millis(),
            now.elapsed().as_secs()
        );
        Ok(hash)
    }

    /// Returns an iterator to iterate over the calculated hashes.
    ///
    /// # Returns
    ///
    /// - `WalkerIter<'_, H, R>`: An iterator to iterate over the calculated hashes.
    pub fn iter(&self) -> WalkerIter<'_> {
        WalkerIter {
            walker: self,
            pos: 0,
        }
    }

    /// This method is used each time before `collect()` is called. It resets the previous state to default.
    fn reset(&mut self) {
        self.paths = Vec::new();
        self.hash = None;
        self.breaker.reset();
    }
}

/// An iterator over the calculated hashes in a `Walker`.
///
/// `WalkerIter` is used to iterate over the `(PathBuf, H)` pairs that
/// represent the paths and their corresponding hashes calculated by the `Walker`.
pub struct WalkerIter<'a> {
    /// A reference to the `Walker` instance.
    walker: &'a Walker,
    /// The current position in the `hashes` vector.
    pos: usize,
}

impl<'a> Iterator for WalkerIter<'a> {
    type Item = &'a (PathBuf, Option<Result<Vec<u8>, E>>);

    /// Advances the iterator and returns the next `(PathBuf, H)` pair.
    ///
    /// # Returns
    ///
    /// - `Some(&(PathBuf, H))` if there is another hash to return.
    /// - `None` if there are no more hashes to return.
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.walker.paths.len() {
            None
        } else {
            self.pos += 1;
            Some(&self.walker.paths[self.pos - 1])
        }
    }
}

impl<'a> IntoIterator for &'a Walker {
    type Item = &'a (PathBuf, Option<Result<Vec<u8>, E>>);
    type IntoIter = WalkerIter<'a>;

    /// Creates an iterator over the calculated hashes in the `Walker`.
    ///
    /// # Returns
    ///
    /// - `WalkerIter<'a, H, R>`: An iterator to iterate over the calculated hashes.
    fn into_iter(self) -> Self::IntoIter {
        WalkerIter {
            walker: self,
            pos: 0,
        }
    }
}
