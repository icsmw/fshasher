mod error;
mod options;
mod pool;
mod progress;
mod worker;

use crate::{
    collector::collect,
    entry::{Entry, Filter},
    hasher::HasherWrapper,
    reader::ReaderWrapper,
    Breaker, Hasher, Reader,
};
pub use error::E;
use log::debug;
pub use options::{Options, ReadingStrategy, Tolerance};
use pool::Pool;
pub use progress::{JobType, Progress, ProgressChannel, Tick};
use std::{
    mem,
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Instant,
};
pub use worker::{Job, Worker};

const MIN_PATHS_PER_JOB: usize = 2;
const MAX_PATHS_PER_JOB: usize = 500;

pub enum Action<H: Hasher> {
    Processed(Vec<(PathBuf, HasherWrapper<H>)>),
    WorkerShutdownNotification,
    Error(PathBuf, E),
}

type HashingResult<T> = Result<(HasherWrapper<T>, Vec<(PathBuf, HasherWrapper<T>)>), E>;

#[derive(Debug)]
pub struct Walker<H: Hasher, R: Reader> {
    opt: Option<Options>,
    breaker: Breaker,
    paths: Vec<PathBuf>,
    invalid: Vec<PathBuf>,
    hashes: Vec<(PathBuf, HasherWrapper<H>)>,
    hash: Option<HasherWrapper<H>>,
    hasher: HasherWrapper<H>,
    reader: ReaderWrapper<R>,
    progress: Option<ProgressChannel>,
}

impl<H: Hasher + 'static, R: Reader + 'static> Walker<H, R> {
    pub fn new(opt: Options, hasher: H, reader: R) -> Result<Self, E> {
        let progress = opt.progress.map(Progress::channel);
        Ok(Self {
            opt: Some(opt),
            breaker: Breaker::new(),
            paths: Vec::new(),
            invalid: Vec::new(),
            hashes: Vec::new(),
            hash: None,
            hasher: HasherWrapper::new(hasher),
            reader: ReaderWrapper::new(reader),
            progress,
        })
    }

    pub fn init(&mut self) -> Result<&mut Self, E> {
        let now = Instant::now();
        self.reset();
        let opt = self.opt.as_mut().ok_or(E::IsNotInited)?;
        let progress = self.progress.as_ref().map(|(progress, _)| progress.clone());
        for entry in opt.entries.iter() {
            let (mut collected, mut invalid) = collect(
                &progress,
                entry,
                &self.breaker,
                &opt.tolerance,
                &opt.threads,
            )?;
            self.paths.append(&mut collected);
            self.invalid.append(&mut invalid);
        }
        debug!(
            "collected {} paths in {}µs / {}ms / {}s",
            self.paths.len(),
            now.elapsed().as_micros(),
            now.elapsed().as_millis(),
            now.elapsed().as_secs()
        );
        Ok(self)
    }

    pub fn breaker(&self) -> Breaker {
        self.breaker.clone()
    }

    pub fn invalid(&self) -> &[PathBuf] {
        &self.invalid
    }

    pub fn count(&self) -> usize {
        self.hashes.len()
    }

    pub fn progress(&mut self) -> Option<Receiver<Tick>> {
        self.progress.as_mut().and_then(|(_, rx)| rx.take())
    }

    pub fn hash(&mut self) -> Result<&[u8], E> {
        let now = Instant::now();
        if self.paths.is_empty() {
            return Ok(&[]);
        }
        let opt = self.opt.as_mut().ok_or(E::IsNotInited)?;
        let (tx_queue, rx_queue): (Sender<Action<H>>, Receiver<Action<H>>) = channel();
        let progress = self.progress.as_ref().map(|(progress, _)| progress.clone());
        let breaker = self.breaker.clone();
        let threads = opt
            .threads
            .or_else(|| thread::available_parallelism().ok().map(|n| n.get()))
            .ok_or(E::OptimalThreadsNumber)?;
        let mut workers: Pool<H, R> = Pool::new(
            threads,
            tx_queue.clone(),
            &opt.reading_strategy,
            &self.breaker,
        );
        debug!("Created pool with {threads} workers for hashing");
        let mut paths = mem::take(&mut self.paths);
        let hasher = self.hasher.clone();
        let reader = self.reader.clone();
        let total = paths.len();
        let paths_per_jobs =
            ((total as f64 * 0.05).ceil() as usize).clamp(MIN_PATHS_PER_JOB, MAX_PATHS_PER_JOB);
        let handle: JoinHandle<HashingResult<H>> = thread::spawn(move || {
            let mut summary = hasher.setup()?;
            let mut next_job = || -> Result<Vec<Job<H, R>>, E> {
                if paths.is_empty() {
                    return Ok(Vec::new());
                }
                let len = paths.len();
                let end = if len < paths_per_jobs {
                    0
                } else {
                    len - paths_per_jobs
                };
                let mut jobs = Vec::new();
                for p in paths.drain(end..).collect::<Vec<PathBuf>>().into_iter() {
                    let r = reader.setup(&p)?;
                    let h = hasher.setup()?;
                    jobs.push((p, h, r));
                }
                Ok(jobs)
            };
            for worker in workers.iter() {
                let jobs: Vec<(PathBuf, HasherWrapper<H>, ReaderWrapper<R>)> = next_job()?;
                if jobs.is_empty() {
                    break;
                }
                worker.deligate(jobs);
            }
            let mut hashes = Vec::new();
            let mut waiting_for_shutdown = false;
            let mut pending: Option<Action<H>> = None;
            'outer: loop {
                let next = if let Some(next) = pending.take() {
                    next
                } else if let Ok(next) = rx_queue.recv() {
                    next
                } else {
                    break 'outer;
                };
                if breaker.is_aborded() {
                    println!(">>>>>>>>>>>>>>>>>>>>>>> ABORTED!");
                    // TODO: this is wrong, we should down all threads before
                    return Err(E::Aborted);
                }
                match next {
                    Action::Processed(mut processed) => {
                        hashes.append(&mut processed);
                        if let Some(ref progress) = progress {
                            progress.notify(JobType::Hashing, hashes.len(), total)
                        }
                    }
                    Action::WorkerShutdownNotification => {
                        if workers.is_all_down() {
                            if let Ok(next) = rx_queue.try_recv() {
                                pending = Some(next);
                                continue;
                            } else {
                                break 'outer;
                            }
                        }
                    }
                    Action::Error(path, err) => {
                        workers.shutdown().wait();
                        return Err(E::Bound(path, Box::new(err)));
                    }
                }
                if waiting_for_shutdown {
                    continue;
                }
                'deligate: for worker in workers.iter().filter(|w| w.is_free()) {
                    let jobs: Vec<(PathBuf, HasherWrapper<H>, ReaderWrapper<R>)> = next_job()?;
                    if jobs.is_empty() {
                        waiting_for_shutdown = true;
                        workers.shutdown();
                        break 'deligate;
                    }
                    worker.deligate(jobs);
                }
            }
            workers.shutdown().wait();
            hashes.sort_by(|(a, _), (b, _)| a.cmp(b));
            for (_, hash) in hashes.iter() {
                summary.absorb(hash.hash()?)?;
            }
            summary.finish()?;
            Ok((summary, hashes))
        });
        let (summary, mut hashes) = handle
            .join()
            .map_err(|e| E::JoinError(format!("{e:?}")))??;
        self.hashes = mem::take(&mut hashes);
        self.hash = Some(summary);
        self.progress = opt.progress.map(Progress::channel);
        let hash = if let Some(ref hash) = self.hash {
            hash.hash()?
        } else {
            unreachable!("Hash has been stored");
        };
        debug!(
            "hashing of {} paths in {}µs / {}ms / {}s",
            total,
            now.elapsed().as_micros(),
            now.elapsed().as_millis(),
            now.elapsed().as_secs()
        );
        Ok(hash)
    }

    pub fn iter(&self) -> WalkerIter<'_, H, R> {
        WalkerIter {
            walker: self,
            pos: 0,
        }
    }

    fn reset(&mut self) {
        self.paths = Vec::new();
        self.invalid = Vec::new();
        self.hash = None;
        self.hashes = Vec::new();
        self.breaker = Breaker::new();
    }
}

pub struct WalkerIter<'a, H: Hasher, R: Reader> {
    walker: &'a Walker<H, R>,
    pos: usize,
}

impl<'a, H: Hasher, R: Reader> Iterator for WalkerIter<'a, H, R> {
    type Item = &'a (PathBuf, HasherWrapper<H>);
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.walker.hashes.len() {
            None
        } else {
            self.pos += 1;
            Some(&self.walker.hashes[self.pos - 1])
        }
    }
}

impl<'a, H: Hasher, R: Reader> IntoIterator for &'a Walker<H, R> {
    type Item = &'a (PathBuf, HasherWrapper<H>);
    type IntoIter = WalkerIter<'a, H, R>;

    fn into_iter(self) -> Self::IntoIter {
        WalkerIter {
            walker: self,
            pos: 0,
        }
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::*;
//     use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
//     use std::{ops::Range, thread};

//     #[test]
//     fn walk() {
//         env_logger::init();
//         let mut entry = Entry::new();
//         entry.entry("/tmp").unwrap();
//         let mut walker = Options::new()
//             .entry(entry)
//             .unwrap()
//             .walker(
//                 hasher::blake::Blake::new(),
//                 reader::buffering::Buffering::default(),
//             )
//             .unwrap();
//         walker.init().unwrap();
//         let hash = walker.hash().unwrap();
//         println!("{hash:?}");
//     }

//     #[test]
//     fn progress() {
//         env_logger::init();
//         let mut entry = Entry::new();
//         entry.entry("/storage/projects/private").unwrap();
//         let mut walker = Options::new()
//             .entry(entry)
//             .unwrap()
//             .progress(10)
//             .reading_strategy(ReadingStrategy::Scenario(vec![
//                 (0..1024 * 1024, Box::new(ReadingStrategy::Complete)),
//                 (1024 * 1024..u64::MAX, Box::new(ReadingStrategy::Buffer)),
//             ]))
//             .unwrap()
//             .walker(
//                 hasher::blake::Blake::new(),
//                 reader::buffering::Buffering::default(),
//             )
//             .unwrap();
//         let progress = walker.progress().unwrap();
//         let hashing = thread::spawn(move || {
//             walker.init().unwrap();
//             let hash = walker.hash().unwrap();
//             println!("{hash:?}");
//         });
//         let tracking = thread::spawn(move || {
//             let mp = MultiProgress::new();
//             let spinner_style =
//                 ProgressStyle::with_template("{spinner} {prefix:.bold.dim} {wide_msg}")
//                     .unwrap()
//                     .tick_chars("▂▃▅▆▇▆▅▃▂ ");
//             let bar = mp.add(ProgressBar::new(u64::MAX));
//             bar.set_style(spinner_style.clone());
//             while let Ok(tick) = progress.recv() {
//                 bar.set_message(tick.to_string());
//                 bar.set_length(tick.total as u64);
//                 bar.set_position(tick.done as u64);
//             }
//         });
//         hashing.join().unwrap();
//         tracking.join().unwrap();
//     }

//     #[test]
//     fn aborting() {
//         env_logger::init();
//         let mut entry = Entry::new();
//         entry.entry("/tmp").unwrap();
//         let mut walker = Options::new()
//             .entry(entry)
//             .unwrap()
//             .progress(10)
//             .walker(
//                 hasher::blake::Blake::new(),
//                 reader::buffering::Buffering::default(),
//             )
//             .unwrap();
//         let progress = walker.progress().unwrap();
//         let breaker = walker.breaker();
//         let hashing = thread::spawn(move || {
//             walker.init().unwrap();
//             match walker.hash() {
//                 Err(E::Aborted) => {
//                     println!("hashing has been aborted");
//                 }
//                 Err(e) => panic!("{e}"),
//                 Ok(_) => panic!("hashing isn't aborted"),
//             }
//         });
//         let tracking = thread::spawn(move || {
//             let mp = MultiProgress::new();
//             let spinner_style =
//                 ProgressStyle::with_template("{spinner} {prefix:.bold.dim} {wide_msg}")
//                     .unwrap()
//                     .tick_chars("▂▃▅▆▇▆▅▃▂ ");
//             let bar = mp.add(ProgressBar::new(u64::MAX));
//             bar.set_style(spinner_style.clone());
//             while let Ok(tick) = progress.recv() {
//                 bar.set_message(tick.to_string());
//                 bar.set_length(tick.total as u64);
//                 bar.set_position(tick.done as u64);
//                 if tick.total as f64 / tick.done as f64 <= 2.0 && !breaker.is_aborded() {
//                     println!("Aborting on: {tick}");
//                     breaker.abort();
//                     break;
//                 }
//             }
//         });
//         hashing.join().unwrap();
//         tracking.join().unwrap();
//     }
// }
