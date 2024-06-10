use super::{Action, HasherWrapper, ReaderWrapper, ReadingStrategy, E};
use crate::{breaker::Breaker, Hasher, Reader};
use log::{debug, error};
use std::{
    io::Read,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
};

const BUFFER_SIZE: usize = 1024 * 32;

pub type Job<H, R> = (PathBuf, HasherWrapper<H>, ReaderWrapper<R>);

/// Represents tasks for the `Worker` to perform.
enum Task<H: Hasher, R: Reader> {
    /// Task to hash a vector of jobs (file path, hasher, reader).
    Hash(Vec<Job<H, R>>),
    /// Task to shut down the worker.
    Shutdown,
}

type TaskChannel<H, R> = (Sender<Task<H, R>>, Receiver<Task<H, R>>);

/// `Worker` is used by `Walker` to read files and calculate hashes. `Worker` creates one thread and
/// listens for incoming messages (`Task`). As a task, `Worker` receives a vector of files and assigns
/// an instance of hasher and reader to each file.
pub struct Worker<H: Hasher, R: Reader> {
    tx_task: Sender<Task<H, R>>,
    queue: Arc<RwLock<usize>>,
    available: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl<H: Hasher + 'static, R: Reader + 'static> Worker<H, R> {
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

    pub fn run(
        tx_queue: Sender<Action<H>>,
        reading_strategy: ReadingStrategy,
        breaker: Breaker,
    ) -> Self {
        let (tx_task, rx_task): TaskChannel<H, R> = channel();
        let queue = Arc::new(RwLock::new(0));
        let available: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
        let available_inner = available.clone();
        let queue_inner = queue.clone();
        let handle = thread::spawn(move || {
            let response = |action: Action<H>| {
                *queue_inner.write().unwrap() -= 1;
                tx_queue.send(action).map_err(|err| {
                    error!(
                        "Worker cannot communicate with pool. Channel error. Worker will be closed"
                    );
                    err
                })
            };
            'outer: while let Ok(task) = rx_task.recv() {
                let jobs = match task {
                    Task::Hash(jobs) => jobs,
                    Task::Shutdown => {
                        break;
                    }
                };
                let mut collected = Vec::new();
                for (path, hasher, reader) in jobs.into_iter() {
                    if breaker.is_aborted() {
                        break 'outer;
                    }
                    match hash_file(&path, hasher, reader, &reading_strategy, &breaker) {
                        Ok(hasher) => collected.push((path, hasher)),
                        Err(err) => {
                            if response(Action::Error(path, err)).is_err() {
                                break 'outer;
                            }
                        }
                    };
                }
                if response(Action::Processed(collected)).is_err() {
                    break 'outer;
                }
            }
            available_inner.store(false, Ordering::Relaxed);
            if tx_queue.send(Action::WorkerShutdownNotification).is_err() {
                error!("Worker cannot communicate with pool. Channel error. Worker will be closed");
            }
            debug!("Hasher worker has been shut down");
        });
        Self {
            tx_task,
            queue,
            available,
            handle: Some(handle),
        }
    }

    /// Checks if the worker is free (i.e., has no tasks in the queue).
    ///
    /// # Returns
    ///
    /// - `bool`: `true` if the worker is free, `false` otherwise.
    pub fn is_free(&self) -> bool {
        *self.queue.read().unwrap() == 0
    }

    /// Checks if the worker is available to take new tasks.
    ///
    /// # Returns
    ///
    /// - `bool`: `true` if the worker is available, `false` otherwise.
    pub fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }

    /// Delegates a task to read a vector of file paths to the worker.
    ///
    /// # Parameters
    ///
    /// - `jobs`: The vector of jobs (file paths, hasher, reader) to be processed by the worker.
    pub fn delegate(&self, jobs: Vec<Job<H, R>>) {
        *self.queue.write().unwrap() += 1;
        let _ = self.tx_task.send(Task::Hash(jobs));
    }

    /// Sends a shutdown signal to the worker.
    pub fn shutdown(&self) {
        if self.is_available() {
            let _ = self.tx_task.send(Task::Shutdown);
        }
    }

    /// Waits for the worker to shut down.
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
/// - `hasher`: The hasher instance to use for hashing the file.
/// - `reader`: The reader instance to use for reading the file.
/// - `reading_strategy`: The strategy to use for reading the file.
/// - `breaker`: The breaker to handle interruptions.
///
/// # Returns
///
/// - `Result<HasherWrapper<H>, E>`: The resulting hasher instance with the file's hash or an error if the operation fails.
///
/// # Errors
///
/// This function will return an error if the operation is interrupted or if there is an issue with reading the file.
fn hash_file<H: Hasher, R: Reader>(
    path: &Path,
    mut hasher: HasherWrapper<H>,
    mut reader: ReaderWrapper<R>,
    reading_strategy: &ReadingStrategy,
    breaker: &Breaker,
) -> Result<HasherWrapper<H>, E> {
    if breaker.is_aborted() {
        return Err(E::Aborted);
    }
    if !path.exists() {
        return Err(E::FileDoesNotExists(path.to_path_buf()));
    }
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
    Ok(hasher)
}
