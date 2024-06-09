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

enum Task<H: Hasher, R: Reader> {
    Hash(Vec<Job<H, R>>),
    Shutdown,
}

type TaskChannel<H, R> = (Sender<Task<H, R>>, Receiver<Task<H, R>>);

pub struct Worker<H: Hasher, R: Reader> {
    tx_task: Sender<Task<H, R>>,
    queue: Arc<RwLock<usize>>,
    available: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl<H: Hasher + 'static, R: Reader + 'static> Worker<H, R> {
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
                        "Worker cannot comunicate with pool. Channel error. Worker will be closed"
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
                    if breaker.is_aborded() {
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
                error!("Worker cannot comunicate with pool. Channel error. Worker will be closed");
            }
            debug!("Hasher worker has been shutdown");
        });
        Self {
            tx_task,
            queue,
            available,
            handle: Some(handle),
        }
    }

    pub fn is_free(&self) -> bool {
        *self.queue.read().unwrap() == 0
    }

    pub fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }

    pub fn deligate(&self, jobs: Vec<Job<H, R>>) {
        *self.queue.write().unwrap() += 1;
        let _ = self.tx_task.send(Task::Hash(jobs));
    }

    pub fn shutdown(&self) {
        if self.is_available() {
            let _ = self.tx_task.send(Task::Shutdown);
        }
    }

    pub fn wait(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn hash_file<H: Hasher, R: Reader>(
    path: &Path,
    mut hasher: HasherWrapper<H>,
    mut reader: ReaderWrapper<R>,
    reading_strategy: &ReadingStrategy,
    breaker: &Breaker,
) -> Result<HasherWrapper<H>, E> {
    if breaker.is_aborded() {
        return Err(E::Aborted);
    }
    let mut apply = |reading_strategy: &ReadingStrategy| {
        match reading_strategy {
            ReadingStrategy::Buffer => {
                let mut buffer = [0u8; BUFFER_SIZE];
                loop {
                    if breaker.is_aborded() {
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
