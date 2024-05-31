use super::Action;

use crate::{breaker::Breaker, walker::Entry};
use log::{debug, error};
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

enum Task {
    Read(PathBuf),
    Shutdown,
}

pub struct Worker {
    tx_task: Sender<Task>,
    queue: Arc<RwLock<usize>>,
    available: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl Worker {
    pub fn run(entry: Entry, tx_queue: Sender<Action>, breaker: Breaker) -> Self {
        let (tx_task, rx_task): (Sender<Task>, Receiver<Task>) = channel();
        let queue = Arc::new(RwLock::new(0));
        let available: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));
        let available_inner = available.clone();
        let queue_inner = queue.clone();
        let handle = thread::spawn(move || {
            let send = |action: Action| {
                tx_queue.send(action).map_err(|err| {
                    error!(
                        "Worker cannot comunicate with pool. Channel error. Worker will be closed"
                    );
                    err
                })
            };
            let response = |action: Action| {
                *queue_inner.write().unwrap() -= 1;
                send(action)
            };
            let check = |path: PathBuf, collected: &mut Vec<PathBuf>| {
                if path.is_file() {
                    collected.push(path);
                    Ok(())
                } else if path.is_dir() {
                    send(Action::Read(path))
                } else {
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
                        if response(Action::Error(path, err.into())).is_err() {
                            break 'outer;
                        } else {
                            continue;
                        }
                    }
                };
                let mut collected: Vec<PathBuf> = Vec::new();
                for el in els.into_iter() {
                    if breaker.is_aborded() {
                        break 'outer;
                    }
                    let Ok(path) = el.map(|el| el.path()) else {
                        continue;
                    };
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
                        if check(path, &mut collected).is_err() {
                            break 'outer;
                        }
                    }
                }
                if response(Action::Finished(collected)).is_err() || breaker.is_aborded() {
                    break 'outer;
                }
            }
            available_inner.store(false, Ordering::Relaxed);
            debug!("Worker has been shutdown");
        });
        Self {
            tx_task,
            queue,
            available,
            handle: Some(handle),
        }
    }

    pub fn count(&self) -> usize {
        *self.queue.read().unwrap()
    }

    pub fn is_available(&self) -> bool {
        self.available.load(Ordering::Relaxed)
    }

    pub fn deligate(&self, path: PathBuf) {
        *self.queue.write().unwrap() += 1;
        let _ = self.tx_task.send(Task::Read(path));
    }

    pub fn shutdown(&self) {
        let _ = self.tx_task.send(Task::Shutdown);
    }

    pub fn wait(&mut self) {
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}
