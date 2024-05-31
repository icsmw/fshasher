mod entry;
mod error;
mod options;
mod progress;

use crate::{
    collector::collect, hasher::HasherWrapper, reader::ReaderWrapper, Breaker, Hasher, Reader,
};
pub use entry::{Entry, Filter};
pub use error::E;
use log::debug;
pub use options::{Options, Tolerance};
pub use progress::{JobType, Progress, ProgressChannel, Tick};
use rayon::prelude::*;
use std::{
    collections::HashMap,
    io::Read,
    mem,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc::Receiver,
        Arc, RwLock,
    },
    time::Instant,
};

const BUFFER_SIZE: usize = 1024 * 8;

#[derive(Debug)]
pub struct Walker<H: Hasher, R: Reader> {
    opt: Option<Options>,
    breaker: Breaker,
    paths: Vec<PathBuf>,
    invalid: Vec<PathBuf>,
    cursor: usize,
    hasher: HasherWrapper<H>,
    reader: ReaderWrapper<R>,
    progress: Option<ProgressChannel>,
}

impl<H: Hasher, R: Reader> Walker<H, R> {
    pub fn new(mut opt: Options, hasher: H, reader: R) -> Result<Self, E> {
        let progress = opt.progress.take();
        Ok(Self {
            opt: Some(opt),
            breaker: Breaker::new(),
            paths: Vec::new(),
            invalid: Vec::new(),
            cursor: 0,
            hasher: HasherWrapper::new(hasher),
            reader: ReaderWrapper::new(reader),
            progress,
        })
    }

    pub fn init(&mut self) -> Result<(), E> {
        let now = Instant::now();
        let mut opt = self.opt.take().ok_or(E::AlreadyInited)?;
        let progress = self.progress.as_ref().map(|(progress, _)| progress.clone());
        for entry in mem::take(&mut opt.entries) {
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
        Ok(())
    }

    pub fn breaker(&self) -> Breaker {
        self.breaker.clone()
    }

    pub fn invalid(&self) -> &[PathBuf] {
        &self.invalid
    }

    pub fn total(&self) -> usize {
        self.paths.len()
    }

    pub fn pos(&self) -> usize {
        self.cursor
    }

    pub fn reset(&mut self) {
        self.cursor = 0;
    }

    pub fn progress(&mut self) -> Option<Receiver<Tick>> {
        self.progress.as_mut().and_then(|(_, rx)| rx.take())
    }

    pub fn hash(&mut self) -> Result<&[u8], E> {
        let now = Instant::now();
        let total = self.total();
        let breaker = self.breaker();
        let hashes: Arc<RwLock<HashMap<usize, Vec<u8>>>> = Arc::new(RwLock::new(HashMap::new()));
        let hasher = self.hasher.clone();
        let reader = self.reader.clone();
        let done: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
        let progress = self.progress.as_ref().map(|(progress, _)| progress.clone());
        self.paths
            .par_iter()
            .enumerate()
            .try_for_each(|(i, path)| {
                if breaker.is_aborded() {
                    return Err(E::Aborted);
                }
                let hash = Walker::<H, R>::hash_file(path, &hasher, &reader, &breaker)?;
                hashes
                    .write()
                    .map_err(|e| E::PoisonError(e.to_string()))?
                    .insert(i, hash.hash()?.to_vec());
                let current = done.load(Ordering::Relaxed);
                done.store(current + 1, Ordering::Relaxed);
                if let Some(progress) = progress.as_ref() {
                    progress.notify(JobType::Hashing, current + 1, total);
                }
                Ok(())
            })?;
        let hashes = hashes.read().map_err(|e| E::PoisonError(e.to_string()))?;
        for i in 0..self.paths.len() {
            if let Some(data) = hashes.get(&i) {
                self.hasher.absorb(data)?;
            }
        }
        self.hasher.finish()?;
        debug!(
            "hashing of {} paths in {}µs / {}ms / {}s",
            total,
            now.elapsed().as_micros(),
            now.elapsed().as_millis(),
            now.elapsed().as_secs()
        );
        self.hasher.hash()
    }

    pub fn next_hash(&mut self) -> Result<Option<(PathBuf, Vec<u8>)>, E> {
        if let Some(path) = self.next() {
            if self.breaker.is_aborded() {
                return Err(E::Aborted);
            }
            let mut reader = self.reader.setup(&path)?;
            let mut hasher = self.hasher.setup()?;
            let mut buffer = [0u8; BUFFER_SIZE];
            loop {
                if self.breaker.is_aborded() {
                    return Err(E::Aborted);
                }
                let bytes_read = reader.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.absorb(&buffer[..bytes_read])?;
            }
            hasher.finish()?;
            Ok(Some((path, hasher.hash()?.to_vec())))
        } else {
            Ok(None)
        }
    }

    fn hash_file<P: AsRef<Path>>(
        path: P,
        hasher: &HasherWrapper<H>,
        reader: &ReaderWrapper<R>,
        breaker: &Breaker,
    ) -> Result<HasherWrapper<H>, E> {
        if breaker.is_aborded() {
            return Err(E::Aborted);
        }
        let mut reader = reader.setup(&path)?;
        let mut hasher = hasher.setup()?;
        let mut buffer = Vec::new();
        // Try read full first
        if reader.read_to_end(&mut buffer).is_ok() {
            hasher.absorb(&buffer)?;
        } else {
            // If cannot read full file, read part by part
            let mut buffer = [0u8; BUFFER_SIZE];
            loop {
                if breaker.is_aborded() {
                    return Err(E::Aborted);
                }
                let bytes_read = reader.read(&mut buffer)?;
                if bytes_read == 0 {
                    break;
                }
                hasher.absorb(&buffer[..bytes_read])?;
            }
        }
        hasher.finish()?;
        Ok(hasher)
    }
}

impl<H: Hasher, R: Reader> Iterator for Walker<H, R> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        if self.breaker.is_aborded() {
            None
        } else {
            let next = self.paths.get(self.cursor).map(|p| p.to_owned());
            self.cursor += 1;
            next
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::*;
    use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
    use std::thread;

    #[test]
    fn walk() {
        env_logger::init();
        let mut entry = Entry::new();
        entry.entry("/tmp").unwrap();
        let mut walker = Options::new()
            .entry(entry)
            .unwrap()
            .walker(hasher::blake::Blake::new(), reader::direct::Direct::new())
            .unwrap();
        walker.init().unwrap();
        let hash = walker.hash().unwrap();
        println!("{hash:?}");
    }

    #[test]
    fn progress() {
        env_logger::init();
        let mut entry = Entry::new();
        entry.entry("/storage/projects/private").unwrap();
        let mut walker = Options::new()
            .entry(entry)
            .unwrap()
            .progress(10)
            .walker(hasher::blake::Blake::new(), reader::direct::Direct::new())
            .unwrap();
        let progress = walker.progress().unwrap();
        let hashing = thread::spawn(move || {
            walker.init().unwrap();
            // let hash = walker.hash().unwrap();
            // println!("{hash:?}");
        });
        let tracking = thread::spawn(move || {
            let mp = MultiProgress::new();
            let spinner_style =
                ProgressStyle::with_template("{spinner} {prefix:.bold.dim} {wide_msg}")
                    .unwrap()
                    .tick_chars("▂▃▅▆▇▆▅▃▂ ");
            let bar = mp.add(ProgressBar::new(u64::MAX));
            bar.set_style(spinner_style.clone());
            while let Ok(tick) = progress.recv() {
                bar.set_message(tick.to_string());
                bar.set_length(tick.total as u64);
                bar.set_position(tick.done as u64);
            }
        });
        hashing.join().unwrap();
        tracking.join().unwrap();
    }

    #[test]
    fn aborting() {
        env_logger::init();
        let mut entry = Entry::new();
        entry.entry("/tmp").unwrap();
        let mut walker = Options::new()
            .entry(entry)
            .unwrap()
            .progress(10)
            .walker(hasher::blake::Blake::new(), reader::direct::Direct::new())
            .unwrap();
        let progress = walker.progress().unwrap();
        let breaker = walker.breaker();
        let hashing = thread::spawn(move || {
            walker.init().unwrap();
            match walker.hash() {
                Err(E::Aborted) => {
                    println!("hashing has been aborted");
                }
                Err(e) => panic!("{e}"),
                Ok(_) => panic!("hashing isn't aborted"),
            }
        });
        let tracking = thread::spawn(move || {
            let mp = MultiProgress::new();
            let spinner_style =
                ProgressStyle::with_template("{spinner} {prefix:.bold.dim} {wide_msg}")
                    .unwrap()
                    .tick_chars("▂▃▅▆▇▆▅▃▂ ");
            let bar = mp.add(ProgressBar::new(u64::MAX));
            bar.set_style(spinner_style.clone());
            while let Ok(tick) = progress.recv() {
                bar.set_message(tick.to_string());
                bar.set_length(tick.total as u64);
                bar.set_position(tick.done as u64);
                if tick.total as f64 / tick.done as f64 <= 2.0 && !breaker.is_aborded() {
                    println!("Aborting on: {tick}");
                    breaker.abort();
                    break;
                }
            }
        });
        hashing.join().unwrap();
        tracking.join().unwrap();
    }
}
