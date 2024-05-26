use log::warn;
use std::{
    fmt,
    sync::mpsc::{channel, Receiver, Sender},
};

#[derive(Debug, Default)]
pub struct Tick {
    pub done: usize,
    pub total: usize,
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "done {}% ({} / {})",
            ((self.done as f64 / self.total as f64) * 100f64) as usize,
            self.done,
            self.total
        )
    }
}

#[derive(Debug)]
pub struct Progress {
    pub tx: Option<Sender<Tick>>,
    pub rx: Option<Receiver<Tick>>,
}

impl Progress {
    pub(crate) fn new() -> Self {
        let (tx, rx): (Sender<Tick>, Receiver<Tick>) = channel();
        Progress {
            tx: Some(tx),
            rx: Some(rx),
        }
    }
    pub fn take(&mut self) -> Option<Receiver<Tick>> {
        self.rx.take()
    }
    pub fn notify(&mut self, done: usize, total: usize) {
        if let Some(tx) = self.tx.as_ref() {
            if let Err(_err) = tx.send(Tick { done, total }) {
                warn!("Fail to send progress data because channel problems. Progress tracking is stopped.");
                self.tx = None;
            }
        }
    }
}
