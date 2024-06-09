use log::warn;
use std::{
    fmt,
    sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender, TrySendError},
};

#[derive(Debug, Default)]
pub enum JobType {
    #[default]
    Collecting,
    Hashing,
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
pub enum Tx {
    Unbound(Sender<Tick>),
    Bound(SyncSender<Tick>),
}

impl Tx {
    pub fn send(&self, tick: Tick) -> bool {
        match self {
            Self::Unbound(tx) => tx.send(tick).is_err(),
            Self::Bound(tx) => match tx.try_send(tick) {
                Ok(_) => false,
                Err(TrySendError::Full(_)) => false,
                Err(_err) => true,
            },
        }
    }
}
#[derive(Debug, Clone)]
pub struct Progress {
    pub tx: Tx,
}

impl Progress {
    pub fn channel(capacity: usize) -> ProgressChannel {
        let (tx, rx): (Tx, Receiver<Tick>) = if capacity == 0 {
            let (tx, rx) = channel();
            (Tx::Unbound(tx), rx)
        } else {
            let (tx, rx) = sync_channel(capacity);
            (Tx::Bound(tx), rx)
        };
        (Progress { tx }, Some(rx))
    }

    pub fn notify(&self, job: JobType, done: usize, total: usize) {
        if self.tx.send(Tick { done, total, job }) {
            warn!("Fail to send progress data because channel problems. Progress tracking is stopped.");
        }
    }
}
