use super::{Action, Worker};
use crate::{breaker::Breaker, entry::Entry};
use std::sync::mpsc::Sender;

/// Created by the `collect()` function to manage available workers. Each worker takes a path to an existing folder
/// and collects paths to files that satisfy the given filters. If a worker encounters a nested folder, it delegates
/// the path back to `collect()` for further assignment to another worker in the queue.
pub struct Pool {
    workers: Vec<Worker>,
}

impl Pool {
    /// Creates a new `Pool` with the specified number of workers.
    ///
    /// # Parameters
    ///
    /// - `count`: The number of workers to create.
    /// - `entry`: The entry point for collecting file paths.
    /// - `tx_queue`: The sender channel for sending actions to the workers.
    /// - `breaker`: The breaker to handle interruptions.
    ///
    /// # Returns
    ///
    /// - A new `Pool` instance.
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

    /// Gets an available worker with the least amount of work.
    ///
    /// # Returns
    ///
    /// - `Option<&Worker>`: An available worker or `None` if no workers are available.
    pub fn get(&self) -> Option<&Worker> {
        self.workers
            .iter()
            .filter(|w| w.is_available())
            .min_by_key(|w| w.count())
    }

    /// Checks if all workers have completed their tasks.
    ///
    /// # Returns
    ///
    /// - `bool`: `true` if all workers are done, `false` otherwise.
    pub fn is_all_done(&self) -> bool {
        self.workers.iter().map(|w| w.count()).sum::<usize>() == 0
    }

    /// Shuts down all workers.
    pub fn shutdown(&mut self) {
        for worker in self.workers.iter_mut() {
            worker.shutdown();
        }
    }
}
