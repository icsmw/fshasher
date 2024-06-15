use super::{Action, ReadingStrategy, Worker};
use crate::{breaker::Breaker, walker, Hasher, Reader, Tolerance};
use std::{slice::Iter, sync::mpsc::Sender};

/// Created by the `Walker` function to manage available workers. Each worker takes a vector of file paths and manages the calculation of their hashes.
/// To calculate a hash, the worker reads the file with the given reader and provides the file's content to the hasher, which returns the hash of the file.
/// As soon as the worker completes all given paths, it returns a vector of file paths with hashes to the `Walker`.
pub struct Pool {
    workers: Vec<Worker>,
    shutdowning: bool,
}

impl Pool {
    /// Creates a new `Pool` with the specified number of workers.
    ///
    /// # Parameters
    ///
    /// - `count`: The number of workers to create.
    /// - `tx_queue`: The sender channel for sending actions to the workers.
    /// - `reading_strategy`: The strategy for reading files.
    ///   - `ReadingStrategy::Buffer` - Each file will be read in the "classic" way using a limited size buffer, chunk by
    ///     chunk until the end. The hasher will receive small chunks of data to calculate the hash of the file. This strategy
    ///     doesn't load the CPU much, but it entails many IO operations.
    ///   - `ReadingStrategy::Complete` - With this strategy, the file will be read first and the complete file's content will
    ///     be passed into the hasher to calculate the hash. This strategy makes fewer IO operations, but it loads the CPU more.
    ///   - `ReadingStrategy::MemoryMapped` - Instead of reading the file, the reader tries to map the file into memory and give
    ///     the full content of the file to the hasher.
    ///   - `ReadingStrategy::Scenario(..)` - The scenario strategy can be used to combine different strategies according to the
    ///     file's size.
    /// - `breaker`: The breaker to handle interruptions.
    ///
    /// # Returns
    ///
    /// - A new `Pool` instance.
    pub fn new<H: Hasher + 'static, R: Reader + 'static>(
        count: usize,
        tx_queue: Sender<Action>,
        reading_strategy: &ReadingStrategy,
        tolerance: &Tolerance,
        breaker: &Breaker,
    ) -> Self
    where
        walker::E: From<<R as Reader>::Error> + From<<H as Hasher>::Error>,
    {
        let mut workers: Vec<Worker> = Vec::new();
        for id in 0..count {
            workers.push(Worker::run::<H, R>(
                tx_queue.clone(),
                reading_strategy.clone(),
                tolerance.clone(),
                breaker.clone(),
                id as u16,
            ));
        }
        Self {
            workers,
            shutdowning: false,
        }
    }

    /// Returns an iterator over the workers.
    ///
    /// # Returns
    ///
    /// - `Iter<Worker>`: An iterator over the workers.
    pub fn iter(&self) -> Iter<Worker> {
        self.workers.iter()
    }

    pub fn workers(&self) -> Vec<&Worker> {
        self.iter().collect()
    }

    /// Checks if all workers are shut down.
    ///
    /// # Returns
    ///
    /// - `bool`: `true` if all workers are done, `false` otherwise.
    pub fn is_all_down(&self) -> bool {
        self.workers
            .iter()
            .map(|w| if w.is_available() { 1 } else { 0 })
            .sum::<usize>()
            == 0
    }

    /// Sends a signal to each worker to shut down. This method doesn't wait for the workers to shut down;
    /// it only sends a shutdown signal to the workers.
    ///
    /// # Returns
    ///
    /// - `&mut Self`: The modified `Pool` instance.
    pub fn shutdown(&mut self) -> &mut Self {
        if self.shutdowning {
            return self;
        }
        self.shutdowning = true;
        for worker in self.workers.iter() {
            worker.shutdown();
        }
        self
    }

    /// Waits for all workers to shut down.
    pub fn wait(&mut self) {
        for worker in self.workers.iter_mut() {
            worker.wait();
        }
    }
}
