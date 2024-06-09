use log::warn;
use std::{
    fmt,
    sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender, TrySendError},
};

/// `JobType` gives information about the current work.
#[derive(Debug, Default)]
pub enum JobType {
    /// Progress tick related to the collecting paths stage.
    #[default]
    Collecting,
    /// Progress tick related to reading files and hashing.
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

/// `Tick` - progress information.
#[derive(Debug, Default)]
pub struct Tick {
    /// Number of done jobs:
    /// - `JobType::Collecting` - number of collected paths.
    /// - `JobType::Hashing` - number of calculated hashes.
    pub done: usize,
    /// Total number of tasks:
    /// - `JobType::Collecting` - total number of collected paths. In this case, `done` always equals `total`.
    /// - `JobType::Hashing` - number of files that should be read and hashed.
    pub total: usize,
    /// Type of current job.
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

/// `Tx` is a holder of the sender for `Progress`.
#[derive(Debug, Clone)]
pub enum Tx {
    Unbound(Sender<Tick>),
    Bound(SyncSender<Tick>),
}

impl Tx {
    /// Sends a `Tick` message through the channel.
    ///
    /// # Parameters
    ///
    /// - `tick`: The progress information to send.
    ///
    /// # Returns
    ///
    /// - `bool`: `true` if sending the message failed, `false` otherwise.
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

/// `Progress` keeps a channel for reporting progress.
#[derive(Debug, Clone)]
pub struct Progress {
    pub tx: Tx,
}

impl Progress {
    /// Creates a new progress channel with the specified capacity.
    ///
    /// # Parameters
    ///
    /// - `capacity`: The capacity of the channel. If `0`, an unbounded channel is created.
    ///     The usage of an unbounded channel provides more detailed (not frequent) updates of progress.
    ///     In other words, a `Tick` will be sent for any update of the `Walker` and `collect()` state.
    ///     However, in the case of many files (more than 1K), this becomes impractical. At some point,
    ///     the size of the channel queue can reach very large values. The common recommendation is to
    ///     use a limited channel with a capacity between 10 and 100. This range is sufficient to
    ///     maintain a good frequency of messages while keeping the channel's queue manageable.
    ///
    /// Note: Regardless of whether a bounded or unbounded channel is created, `Walker` and
    /// `collect()` send a `Tick` with each update of their state. This can result in many messages
    /// per time unit. However, when a bounded channel is used, `Walker` and `collect()` check the queue
    /// before sending a notification and send notifications as soon as the queue is free.
    ///
    /// # Returns
    ///
    /// - `ProgressChannel`: A tuple containing the `Progress` instance and an optional receiver.
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

    /// Sends a progress notification.
    ///
    /// # Parameters
    ///
    /// - `job`: The type of job being tracked.
    /// - `done`: The number of completed tasks.
    /// - `total`: The total number of tasks.
    pub fn notify(&self, job: JobType, done: usize, total: usize) {
        if self.tx.send(Tick { done, total, job }) {
            warn!("Failed to send progress data due to channel problems. Progress tracking is stopped.");
        }
    }
}
