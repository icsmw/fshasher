pub mod error;
mod pool;
mod worker;

use crate::{
    breaker::Breaker,
    entry::Entry,
    walker::{JobType, Progress},
};
pub use error::E;
use log::{debug, error, warn};
pub use pool::Pool;
use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Instant,
};
pub use worker::Worker;

/// Defines tolerance levels for errors during the collection of file paths. In some cases,
/// an attempt to read a file or folder can cause an error (for example, a permissions error). To
/// handle such situations, users can define the behavior of the collector.
#[derive(Debug, Clone)]
pub enum Tolerance {
    /// All errors during collection will be logged but will not stop the collecting
    /// process. A list of paths that caused errors will be returned by `collect()`.
    LogErrors,
    /// All errors during collection will be ignored without logging. The collecting
    /// process will not be stopped. A list of paths that caused errors will be returned
    /// by `collect()`.
    DoNotLogErrors,
    /// The collecting process will be stopped on the first error.
    StopOnErrors,
}

impl Default for Tolerance {
    fn default() -> Self {
        Self::LogErrors
    }
}

/// Message for communication between `collect()` and workers during collecting.
#[derive(Debug)]
pub enum Action {
    /// Called by a worker to delegate reading of a found folder to another worker.
    Delegate(PathBuf),
    /// Called by a worker to report found paths to files.
    Processed(Vec<PathBuf>),
    /// Reported by a worker in case of an error.
    ///
    /// # Parameters
    ///
    /// - `PathBuf`: The path that caused the error.
    /// - `E`: The error encountered during processing.
    /// - `bool`: A flag indicating if the error has finished processing.
    Error(PathBuf, E, bool),
}

/// The result type for the `collect()` function.
pub type CollectingResult = Result<(Vec<PathBuf>, Vec<PathBuf>), E>;

/// Collects file paths based on the provided entry and filters.
///
/// # Parameters
///
/// - `progress`: An optional progress tracker.
/// - `entry`: The entry point for collecting file paths.
/// - `breaker`: A breaker to handle interruptions.
/// - `tolerance`: The tolerance level for error handling.
///   - `Tolerance::LogErrors`: Errors will be logged, but the collecting process will not be stopped.
///   - `Tolerance::DoNotLogErrors`: Errors will be ignored, and the collecting process will not be stopped.
///   - `Tolerance::StopOnErrors`: The collecting process will stop on any IO errors.
/// - `threads`: The optional number of threads to use for processing. If this setting is not set
///   (`None`), the number of threads will default to the number of available cores.
///
/// # Returns
///
/// - `CollectingResult`: A result containing a tuple of vectors with collected paths and ignored
///   paths, or an error if the operation fails.
///   - `Ok((Vec<PathBuf>, Vec<PathBuf>))` includes a list of collected file paths and a list of ignored
///     paths (in case of tolerance: `Tolerance::LogErrors` or `Tolerance::DoNotLogErrors`). In the case of
///     `Tolerance::StopOnErrors`, the list of ignored paths will always be empty.
///
/// # Errors
///
/// This function will return an error if the operation is interrupted or if there is an issue with
/// threading. Returning errors is sensitive to the tolerance level. Only in the case of `Tolerance::StopOnErrors`
/// will `collect()` return an error in case of IO errors.
///
/// # Examples
///
/// Example of tracking the collection of files
/// ```
/// use fshasher::{collect, Breaker, Entry, Progress, Tolerance};
/// use std::{env::temp_dir, thread};
/// let (progress, rx) = Progress::channel(10);
/// let rx = rx.unwrap();
/// thread::spawn(move || {
///     while let Ok(tick) = rx.recv() {
///         println!("{tick}");
///     }
///     println!("Collecting is finished");
/// });
/// let (included, ignored) = collect(
///     &Some(progress),
///     &Entry::from(temp_dir()).unwrap(),
///     &Breaker::new(),
///     &Tolerance::LogErrors,
///     &None,
/// )
/// .unwrap();
/// println!(
///     "Found {} accessible paths to files; {} ignored",
///     included.len(),
///     ignored.len()
/// );
/// ```
///
/// Aborting the collecting operation with the first progress tick
///
/// ```
/// use fshasher::{collect, Breaker, Entry, Progress, Tolerance};
/// use std::{env::temp_dir, thread};
///
/// let (progress, rx) = Progress::channel(10);
/// let rx = rx.unwrap();
/// let breaker = Breaker::new();
/// let breaker_inner = breaker.clone();
/// thread::spawn(move || {
///     let _ = rx.recv();
///     println!("Breaking collecting with the first tick");
///     breaker_inner.abort();
/// });
/// assert!(collect(
///     &Some(progress),
///     &Entry::from(temp_dir()).unwrap(),
///     &breaker,
///     &Tolerance::LogErrors,
///     &None,
/// )
/// .is_err());
/// println!("Collecting operation has been aborted");
/// ```
pub fn collect(
    progress: &Option<Progress>,
    entry: &Entry,
    breaker: &Breaker,
    tolerance: &Tolerance,
    threads: &Option<usize>,
) -> CollectingResult {
    let now = Instant::now();
    let (tx_queue, rx_queue): (Sender<Action>, Receiver<Action>) = channel();
    tx_queue
        .send(Action::Delegate(entry.entry.clone()))
        .unwrap();
    let progress = progress.clone();
    let breaker = breaker.clone();
    let tolerance = tolerance.clone();
    let threads = threads
        .or_else(|| thread::available_parallelism().ok().map(|n| n.get()))
        .ok_or(E::OptimalThreadsNumber)?;
    let entry_inner = entry.clone();
    let handle: JoinHandle<CollectingResult> = thread::spawn(move || {
        let mut collected: Vec<PathBuf> = Vec::new();
        let mut invalid: Vec<PathBuf> = Vec::new();
        let mut workers = Pool::new(threads, entry_inner.clone(), tx_queue.clone(), &breaker);
        debug!("Created pool with {threads} workers for paths collecting");
        let mut pending: Option<Action> = None;
        let mut queue: isize = 0;
        if breaker.is_aborted() {
            return Err(E::Aborted);
        }
        let result = 'listener: loop {
            let next = if let Some(next) = pending.take() {
                next
            } else if let Ok(next) = rx_queue.recv() {
                next
            } else {
                break 'listener Ok((collected, invalid));
            };
            if breaker.is_aborted() {
                break 'listener Err(E::Aborted);
            }
            match next {
                Action::Delegate(next) => {
                    queue += 1;
                    let Some(worker) = workers.get() else {
                        break 'listener Err(E::NoAvailableWorkers);
                    };
                    worker.delegate(next);
                }
                Action::Processed(mut paths) => {
                    queue -= 1;
                    collected.append(&mut paths);
                    if let Some(ref progress) = progress {
                        let count = collected.len();
                        progress.notify(JobType::Collecting, count, count);
                    }
                    if let Ok(next) = rx_queue.try_recv() {
                        pending = Some(next);
                        continue;
                    }
                    if workers.is_all_done() && queue == 0 {
                        break 'listener Ok((collected, invalid));
                    }
                }
                Action::Error(path, err, finished) => {
                    if finished {
                        queue -= 1;
                    }
                    match tolerance {
                        Tolerance::StopOnErrors => {
                            error!("entry: {}; error: {err}", path.display());
                            break 'listener Err(err);
                        }
                        Tolerance::LogErrors => {
                            warn!("entry: {}; error: {err}", path.display());
                            invalid.push(path);
                        }
                        Tolerance::DoNotLogErrors => {
                            invalid.push(path);
                        }
                    };
                }
            }
        };
        workers.shutdown();
        if breaker.is_aborted() {
            Err(E::Aborted)
        } else {
            result
        }
    });
    let (collected, ignored) = handle
        .join()
        .map_err(|e| E::JoinError(format!("{e:?}")))??;
    debug!(
        "Collected {} files (ignored: {}) in {}µs / {}ms / {}s; source: {}",
        collected.len(),
        ignored.len(),
        now.elapsed().as_micros(),
        now.elapsed().as_millis(),
        now.elapsed().as_secs(),
        entry.entry.display()
    );
    Ok((collected, ignored))
}
