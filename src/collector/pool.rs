use super::{Action, Worker};
use crate::{breaker::Breaker, entry::Entry};
use std::sync::mpsc::Sender;

pub struct Pool {
    workers: Vec<Worker>,
}

impl Pool {
    pub fn new(count: usize, entry: Entry, tx_queue: Sender<Action>, breaker: &Breaker) -> Self {
        let mut workers: Vec<Worker> = Vec::new();
        for _ in 0..count {
            workers.push(Worker::run(
                entry.clone(),
                tx_queue.clone(),
                breaker.clone(),
            ));
        }
        Self { workers }
    }
    pub fn get(&self) -> Option<&Worker> {
        self.workers
            .iter()
            .filter(|w| w.is_available())
            .min_by_key(|w| w.count())
    }
    pub fn is_all_done(&self) -> bool {
        self.workers.iter().map(|w| w.count()).sum::<usize>() == 0
    }
    pub fn shutdown(&mut self) {
        for worker in self.workers.iter_mut() {
            worker.shutdown();
        }
    }
}
