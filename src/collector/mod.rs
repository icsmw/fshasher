mod error;
mod pool;
mod worker;

use crate::{
    breaker::Breaker,
    walker::{Entry, JobType, Progress, Tolerance},
};
pub use error::E;
use log::{debug, error, warn};
pub use pool::Pool;
use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
};
pub use worker::Worker;

pub enum Action {
    Read(PathBuf),
    Finished(Vec<PathBuf>),
    Error(PathBuf, E),
}

pub type CollectingResult = Result<(Vec<PathBuf>, Vec<PathBuf>), E>;

pub fn collect(
    progress: &Option<Progress>,
    entry: Entry,
    breaker: &Breaker,
    tolerance: &Tolerance,
    threads: &Option<usize>,
) -> CollectingResult {
    let (tx_queue, rx_queue): (Sender<Action>, Receiver<Action>) = channel();
    tx_queue.send(Action::Read(entry.entry.clone())).unwrap();
    let progress = progress.clone();
    let breaker = breaker.clone();
    let tolerance = tolerance.clone();
    let threads = threads
        .or_else(|| thread::available_parallelism().ok().map(|n| n.get()))
        .ok_or(E::OptimalThreadsNumber)?;
    let handle: JoinHandle<CollectingResult> = thread::spawn(move || {
        let mut collected: Vec<PathBuf> = Vec::new();
        let mut invalid: Vec<PathBuf> = Vec::new();
        let mut workers = Pool::new(threads, entry.clone(), tx_queue.clone(), &breaker);
        debug!("Created pool with {threads} workers for paths collecting");
        let mut pending: Option<Action> = None;
        let result = 'listener: loop {
            let next = if let Some(next) = pending.take() {
                next
            } else if let Ok(next) = rx_queue.recv() {
                next
            } else {
                break 'listener Ok((collected, invalid));
            };
            match next {
                Action::Read(next) => {
                    let Some(worker) = workers.get() else {
                        break 'listener Err(E::NoAvailableWorkers);
                    };
                    worker.deligate(next);
                }
                Action::Finished(mut paths) => {
                    collected.append(&mut paths);
                }
                Action::Error(path, err) => {
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
            if let Ok(next) = rx_queue.try_recv() {
                pending = Some(next);
                continue;
            }
            if let Some(ref progress) = progress {
                let count = collected.len();
                progress.notify(JobType::Collecting, count, count);
            }
            if workers.is_done() {
                break 'listener Ok((collected, invalid));
            }
        };
        workers.shutdown();
        result
    });
    handle.join().map_err(|e| E::JoinError(format!("{e:?}")))?
}
