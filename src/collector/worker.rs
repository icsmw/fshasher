use super::Action;

use crate::{breaker::Breaker, entry::Entry};
use log::error;
use std::{
    fs::{read_dir, read_link},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Receiver, Sender},
        Arc, RwLock,
    },
    thread::{self, JoinHandle},
};

/// Enum represents a list of messages for communication between the `collect()` function and `Worker`.
enum Task {
    /// Task to read a folder and collect paths to files.
    Read(PathBuf),
    /// Breaking the listening loop of the `Worker` to free resources and close opened channels. Once `Shutdown`
    /// has been called, the `Worker` cannot be reused.
    Shutdown,
}

/// `Worker` is used by the `collect()` function for collecting paths to files. `Worker` creates one thread and
/// listens for incoming messages (tasks). `Worker` does not read nested folders; as soon as `Worker` encounters a
/// folder, it sends the path back to `collect()` for further assignment to another worker in the queue.
///
/// Error handling: `Worker` doesn't stop the listener loop on IO errors. This is the responsibility of the `collect()`
/// function. Any IO errors will be reported to the `collect()` function, which will make a decision based on the level
/// of tolerance.
pub struct Worker {
    tx_task: Sender<Task>,
    queue: Arc<RwLock<usize>>,
    available: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    /// Runs a new `Worker` instance.
    ///
    /// # Parameters
    ///
    /// - `entry`: The entry point for collecting file paths.
    /// - `tx_queue`: The sender channel for sending actions to `collect()` function.
    /// - `breaker`: The breaker to handle interruptions.
    ///
    /// # Returns
    ///
    /// - A new `Worker` instance.
    pub fn run(entry: Entry, tx_queue: Sender<Action>, breaker: Breaker) -> Self {
        let (tx_task, rx_task): (Sender<Task>, Receiver<Task>) = channel();
        let queue = Arc::new(RwLock::new(0));
        let available: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
        let available_inner = available.clone();
        let queue_inner = queue.clone();
        let handle = thread::spawn(move || {
            let send = |action: Action| {
                tx_queue.send(action).inspect_err(|_err| {
                    error!(
                        "Worker cannot communicate with pool. Channel error. Worker will be closed"
                    )
                })
            };
            let response = |action: Action| {
                let _ = queue_inner.write().map(|mut v| *v -= 1);
                send(action)
            };
            let check = |path: PathBuf, collected: &mut Vec<PathBuf>| {
                if path.is_file() {
                    collected.push(path);
                    Ok(())
                } else if path.is_dir() {
                    send(Action::Delegate(path))
                } else {
                    // This situation is possible in some timing. After folder is read, file can be removed.
                    // Actualy nothing todo here.
                    Ok(())
                }
            };
            'outer: while let Ok(task) = rx_task.recv() {
                let path = match task {
                    Task::Read(path) => path,
                    Task::Shutdown => break 'outer,
                };
                let els = match read_dir(&path) {
                    Ok(els) => els,
                    Err(err) => {
                        if response(Action::Processed(Err((path, err.into())))).is_err() {
                            break 'outer;
                        } else {
                            continue;
                        }
                    }
                };
                let mut collected: Vec<PathBuf> = Vec::new();
                for el in els.into_iter() {
                    if breaker.is_aborted() {
                        let _ = response(Action::Processed(Ok(collected)));
                        break 'outer;
                    }
                    let path = match el.map(|el| el.path()) {
                        Ok(p) => p,
                        Err(err) => {
                            if response(Action::Error(
                                path.join("err__fail_parse_DirEntry__"),
                                err.into(),
                            ))
                            .is_err()
                            {
                                break 'outer;
                            } else {
                                continue;
                            }
                        }
                    };
                    if !path.exists() {
                        // It might be after folder read, file already doesn't exist
                        continue;
                    }
                    if !entry.filtered(&path) {
                        continue;
                    }
                    if path.is_file() || path.is_dir() {
                        if check(path, &mut collected).is_err() {
                            break 'outer;
                        }
                    } else if path.is_symlink() {
                        let path = match read_link(&path) {
                            Ok(path) => path,
                            Err(err) => {
                                if response(Action::Error(path, err.into())).is_err() {
                                    break 'outer;
                                } else {
                                    continue;
                                }
                            }
                        };
                        if path.exists() && (path.is_file() || path.is_dir()) {
                            if check(path, &mut collected).is_err() {
                                break 'outer;
                            }
                        } else {
                            continue;
                        }
                    }
                }
                if response(Action::Processed(Ok(collected))).is_err() {
                    break 'outer;
                }
            }
            available_inner.store(false, Ordering::SeqCst);
        });
        Self {
            tx_task,
            queue,
            available,
            handle: Some(handle),
        }
    }

    /// Returns the number of tasks in the worker's queue.
    ///
    /// # Returns
    ///
    /// - `usize`: The number of tasks in the queue.
    pub fn count(&self) -> usize {
        *self.queue.read().expect("Worker's queue index available")
    }

    /// Checks if the worker is available to take new tasks.
    ///
    /// # Returns
    ///
    /// - `bool`: `true` if the worker is available, `false` otherwise.
    pub fn is_available(&self) -> bool {
        self.available.load(Ordering::SeqCst)
    }

    /// Delegates a task to read a path to the worker.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to be read by the worker.
    pub fn delegate(&self, path: PathBuf) {
        let _ = self.queue.write().map(|mut v| *v += 1);
        let _ = self.tx_task.send(Task::Read(path));
    }

    /// Send command to `Worker` to exit from a listener loop as soon as possible
    pub fn shutdown(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = self.tx_task.send(Task::Shutdown);
            let _ = handle.join();
        }
    }
}
