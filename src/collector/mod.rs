mod error;
mod pool;
mod worker;

use crate::{
    breaker::Breaker,
    entry::Entry,
    walker::{JobType, Progress, Tolerance},
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

pub enum Action {
    Deligate(PathBuf),
    Processed(Vec<PathBuf>),
    Error(PathBuf, E),
}

pub type CollectingResult = Result<(Vec<PathBuf>, Vec<PathBuf>), E>;

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
        .send(Action::Deligate(entry.entry.clone()))
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
        if breaker.is_aborded() {
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
            if breaker.is_aborded() {
                break 'listener Err(E::Aborted);
            }
            match next {
                Action::Deligate(next) => {
                    queue += 1;
                    let Some(worker) = workers.get() else {
                        break 'listener Err(E::NoAvailableWorkers);
                    };
                    worker.deligate(next);
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
                Action::Error(path, err) => {
                    println!(">>>>>>>>>>>>>>>>>>>>> err: {err}");
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
        if breaker.is_aborded() {
            Err(E::Aborted)
        } else {
            result
        }
    });
    let (collected, ignored) = handle
        .join()
        .map_err(|e| E::JoinError(format!("{e:?}")))??;
    debug!(
        "collected {} files (ignored: {}) in {}µs / {}ms / {}s; source: {}",
        collected.len(),
        ignored.len(),
        now.elapsed().as_micros(),
        now.elapsed().as_millis(),
        now.elapsed().as_secs(),
        entry.entry.display()
    );
    Ok((collected, ignored))
}
