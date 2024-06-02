use super::{Action, ReadingStrategy, Worker};
use crate::{breaker::Breaker, Hasher, Reader};
use std::{slice::Iter, sync::mpsc::Sender};

pub struct Pool<H: Hasher, R: Reader> {
    workers: Vec<Worker<H, R>>,
}

impl<H: Hasher + 'static, R: Reader + 'static> Pool<H, R> {
    pub fn new(
        count: usize,
        tx_queue: Sender<Action<H>>,
        reading_strategy: &ReadingStrategy,
        breaker: &Breaker,
    ) -> Self {
        let mut workers: Vec<Worker<H, R>> = Vec::new();
        for _ in 0..count {
            workers.push(Worker::run(
                tx_queue.clone(),
                reading_strategy.clone(),
                breaker.clone(),
            ));
        }
        Self { workers }
    }
    pub fn iter(&self) -> Iter<Worker<H, R>> {
        self.workers.iter()
    }
    pub fn is_all_down(&self) -> bool {
        self.workers
            .iter()
            .map(|w| if w.is_available() { 1 } else { 0 })
            .sum::<usize>()
            == 0
    }
    pub fn shutdown(&mut self) -> &mut Self {
        for worker in self.workers.iter() {
            worker.shutdown();
        }
        self
    }

    pub fn wait(&mut self) {
        for worker in self.workers.iter_mut() {
            worker.wait();
        }
    }
}
