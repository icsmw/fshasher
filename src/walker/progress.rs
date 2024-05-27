use log::warn;
use std::{
    fmt,
    sync::mpsc::{channel, Receiver, Sender},
};

#[derive(Debug)]
pub enum JobType {
    Collecting,
    Hashing,
}

impl Default for JobType {
    fn default() -> Self {
        JobType::Collecting
    }
}

impl fmt::Display for JobType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Collecting => "collecting",
                Self::Hashing => "hashing",
            },
        )
    }
}

#[derive(Debug, Default)]
pub struct Tick {
    pub done: usize,
    pub total: usize,
    pub job: JobType,
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} done {}% ({} / {})",
            self.job,
            ((self.done as f64 / self.total as f64) * 100f64) as usize,
            self.done,
            self.total
        )
    }
}

pub type ProgressChannel = (Progress, Option<Receiver<Tick>>);

#[derive(Debug, Clone)]
pub struct Progress {
    pub tx: Sender<Tick>,
}

impl Progress {
    pub(crate) fn channel() -> ProgressChannel {
        let (tx, rx): (Sender<Tick>, Receiver<Tick>) = channel();
        (Progress { tx }, Some(rx))
    }

    pub fn notify(&self, job: JobType, done: usize, total: usize) {
        if let Err(_err) = self.tx.send(Tick { done, total, job }) {
            warn!("Fail to send progress data because channel problems. Progress tracking is stopped.");
        }
    }
}
