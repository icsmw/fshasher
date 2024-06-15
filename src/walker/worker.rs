use super::{Action, ReadingStrategy, E};
use crate::{breaker::Breaker, Hasher, Reader, Tolerance};
use log::error;
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
};

const BUFFER_SIZE: usize = 1024 * 32;

/// Represents tasks for the `Worker` to perform.
enum Task {
    /// Task to hash a vector of files.
    Hash(Vec<PathBuf>),
    /// Task to shut down the worker.
    Shutdown,
}

type TaskChannel = (Sender<Task>, Receiver<Task>);

/// `Worker` is used by `Walker` to read files and calculate hashes. `Worker` creates one thread and
/// listens for incoming messages (`Task`). As a task, `Worker` receives a vector of files and assigns
/// an instance of hasher and reader to each file.
pub struct Worker {
    tx_task: Sender<Task>,
    available: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
    pub id: u16,
}

impl Worker {
    /// Runs a new `Worker` instance.
    ///
    /// # Parameters
    ///
    /// - `tx_queue`: The sender channel for sending actions to the pool.
    /// - `reading_strategy`: The strategy for reading files.
    /// - `breaker`: The breaker to handle interruptions.
    ///
    /// # Returns
    ///
    /// - A new `Worker` instance.
    ///
    /// Errors handeling
    ///
    /// `Worker` doesn't stop the listener loop on IO errors (from hasher or reader), but reports the error
    /// to `Walker`.

    pub fn run<H: Hasher + 'static, R: Reader + 'static>(
        tx_queue: Sender<Action>,
        reading_strategy: ReadingStrategy,
        tolerance: Tolerance,
        breaker: Breaker,
        id: u16,
    ) -> Self
    where
        E: From<<R as Reader>::Error> + From<<H as Hasher>::Error>,
    {
        let (tx_task, rx_task): TaskChannel = channel();
        let available: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
        let available_inner = available.clone();
        let handle = thread::spawn(move || {
            let response = |action: Action| {
                tx_queue.send(action).map_err(|err| {
                    error!(
                        "Hasher worker cannot communicate with pool. Channel error. Worker will be closed"
                    );
                    err
                })
            };
            let report = |action: Action| {
                tx_queue.send(action).map_err(|err| {
                    error!(
                        "Hasher worker cannot communicate with pool. Channel error. Worker will be closed"
                    );
                    err
                })
            };
            'outer: while let Ok(task) = rx_task.recv() {
                let jobs = match task {
                    Task::Hash(jobs) => jobs,
                    Task::Shutdown => {
                        break 'outer;
                    }
                };
                let mut collected = Vec::new();
                let mut reports: Vec<(PathBuf, E)> = Vec::new();
                for path in jobs.into_iter() {
                    if breaker.is_aborted() {
                        break 'outer;
                    }
                    match hash_file::<H, R>(&path, &reading_strategy, &breaker) {
                        Ok(hasher) => collected.push((path, hasher)),
                        Err(err) => {
                            if matches!(tolerance, Tolerance::StopOnErrors) {
                                let _ = report(Action::Error(path, err));
                                break 'outer;
                            } else {
                                reports.push((path, err));
                            }
                        }
                    };
                }
                if response(Action::Processed(id, collected, reports)).is_err() {
                    break 'outer;
                }
            }
            available_inner.store(false, Ordering::SeqCst);
            if tx_queue.send(Action::WorkerShutdownNotification).is_err() {
                error!("Hasher worker cannot communicate with pool. Channel error. Worker will be closed");
            }
        });
        Self {
            tx_task,
            available,
            handle: Some(handle),
            id,
        }
    }

    /// Checks if the worker is available to take new tasks.
    ///
    /// # Returns
    ///
    /// - `bool`: `true` if the worker is available, `false` otherwise.
    pub fn is_available(&self) -> bool {
        self.available.load(Ordering::SeqCst)
    }

    /// Delegates a task to read a vector of file paths to the worker.
    ///
    /// # Parameters
    ///
    /// - `jobs`: The vector of jobs (paths to files) to be processed by the worker.
    pub fn delegate(&self, jobs: Vec<PathBuf>) -> bool {
        if !self.is_available() {
            return false;
        }
        let _ = self.tx_task.send(Task::Hash(jobs));
        true
    }

    /// Sends a shutdown signal to the worker.
    pub fn shutdown(&self) {
        if self.is_available() {
            let _ = self.tx_task.send(Task::Shutdown);
        }
    }

    /// Waits for the worker to shutdown.
    pub fn wait(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

/// Hashes the content of a file based on the given reading strategy.
///
/// # Parameters
///
/// - `path`: The path of the file to be hashed.
/// - `reading_strategy`: The strategy to use for reading the file.
/// - `breaker`: The breaker to handle interruptions.
///
/// # Returns
///
/// - `Result<H, E>`: The resulting hasher instance with the file's hash or an error if the operation fails.
///
/// # Errors
///
/// This function will return an error if the operation is interrupted or if there is an issue with reading the file.
fn hash_file<H: Hasher, R: Reader>(
    path: &Path,
    reading_strategy: &ReadingStrategy,
    breaker: &Breaker,
) -> Result<Vec<u8>, E>
where
    E: From<<R as Reader>::Error> + From<<H as Hasher>::Error>,
{
    if breaker.is_aborted() {
        return Err(E::Aborted);
    }
    if !path.exists() {
        return Err(E::FileDoesNotExists(path.to_path_buf()));
    }
    let mut hasher = H::new();
    let mut reader = R::bound(path);
    let mut apply = |reading_strategy: &ReadingStrategy| {
        match reading_strategy {
            ReadingStrategy::Buffer => {
                let mut buffer = [0u8; BUFFER_SIZE];
                loop {
                    if breaker.is_aborted() {
                        return Err(E::Aborted);
                    }
                    let bytes_read = reader.read(&mut buffer)?;
                    if bytes_read == 0 {
                        break;
                    }
                    hasher.absorb(&buffer[..bytes_read])?;
                }
            }
            ReadingStrategy::Complete => {
                let mut buffer = Vec::new();
                reader.read_to_end(&mut buffer)?;
                hasher.absorb(&buffer)?
            }
            ReadingStrategy::MemoryMapped => {
                hasher.absorb(reader.mmap()?)?;
            }
            ReadingStrategy::Scenario(..) => {
                return Err(E::NestedScenarioStrategy);
            }
        };
        Ok(())
    };
    match reading_strategy {
        ReadingStrategy::Buffer | ReadingStrategy::Complete | ReadingStrategy::MemoryMapped => {
            apply(reading_strategy)?;
        }
        ReadingStrategy::Scenario(scenario) => {
            let md = path.metadata()?;
            let strategy = scenario
                .iter()
                .find_map(|(range, strategy)| {
                    if range.contains(&md.len()) {
                        Some(strategy)
                    } else {
                        None
                    }
                })
                .ok_or(E::NoRangeForScenarioStrategy(md.len()))?;
            apply(strategy)?;
        }
    };
    hasher.finish()?;
    Ok(hasher.hash()?.to_vec())
}
