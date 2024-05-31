mod error;
mod pool;
mod worker;

use crate::{
    breaker::{self, Breaker},
    walker::{Entry, JobType, Progress, Tolerance},
};
pub use error::E;
use log::{debug, error, warn};
pub use pool::Pool;
use std::{
    path::PathBuf,
    process,
    sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender},
    thread::{self, JoinHandle},
};
pub use worker::Worker;

pub enum Action {
    Read(PathBuf),
    Finished(Vec<PathBuf>),
    Error(E),
}

pub fn collect(progress: &Option<Progress>, entry: Entry, breaker: &Breaker) -> Vec<PathBuf> {
    let (tx_result, rx_result): (SyncSender<Vec<PathBuf>>, Receiver<Vec<PathBuf>>) =
        sync_channel(1);
    let (tx_queue, rx_queue): (Sender<Action>, Receiver<Action>) = channel();
    tx_queue.send(Action::Read(entry.entry.clone())).unwrap();
    let progress = progress.clone();
    let breaker = breaker.clone();
    thread::spawn(move || {
        let mut collected: Vec<PathBuf> = Vec::new();
        let workers = Pool::new(4, entry.clone(), tx_queue.clone(), &breaker);
        let mut pending: Option<Action> = None;
        loop {
            let next = if let Some(next) = pending.take() {
                next
            } else if let Ok(next) = rx_queue.recv() {
                next
            } else {
                break;
            };
            match next {
                Action::Read(next) => {
                    if let Some(worker) = workers.get() {
                        worker.deligate(next);
                    } else {
                        println!(">>>>>>>>>>>> no workers");
                        // have to do something
                    }
                }
                Action::Finished(mut paths) => {
                    collected.append(&mut paths);
                }
                Action::Error(err) => {
                    // make decision
                }
            }
            if let Ok(next) = rx_queue.try_recv() {
                pending = Some(next);
                continue;
            }
            if workers.is_done() {
                let _ = tx_result.send(collected);
                break;
            }
            if let Some(ref progress) = progress {
                let count = collected.len();
                progress.notify(JobType::Collecting, count, count);
            }
        }
    });
    rx_result.recv().unwrap()
}
