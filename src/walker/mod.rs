mod collector;
mod entry;
mod error;
mod options;
mod progress;

use crate::{hasher::HasherWrapper, reader::ReaderWrapper, Breaker, Hasher, Reader};
use collector::Collector;
pub use entry::{Entry, Filter};
pub use error::E;
use log::debug;
pub use options::{Options, Tolerance};
pub use progress::{Progress, Tick};
use std::{io::Read, mem, path::PathBuf, sync::mpsc::Receiver, time::Instant};

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
    progress: Option<Progress>,
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
        let mut opt = self.opt.take().ok_or(E::AlreadyInited)?;
        let mut collector = Collector::new(
            opt.tolerance.clone(),
            &self.breaker,
            mem::take(&mut opt.entries),
        );
        collector.collect()?;
        self.paths = mem::take(&mut collector.collected);
        self.invalid = mem::take(&mut collector.invalid);
        Ok(())
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
        self.progress.as_mut().and_then(|progress| progress.take())
    }

    pub fn hash(&mut self) -> Result<&[u8], E> {
        let now = Instant::now();
        let total = self.total();
        while let Some(path) = self.next() {
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
            if self.breaker.is_aborded() {
                return Err(E::Aborted);
            }
            hasher.finish()?;
            self.hasher.absorb(hasher.hash()?)?;
            if let Some(tracker) = self.progress.as_mut() {
                tracker.notify(self.cursor, total);
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
        entry.entry("/tmp").unwrap();
        let mut walker = Options::new()
            .entry(entry)
            .unwrap()
            .progress()
            .walker(hasher::blake::Blake::new(), reader::direct::Direct::new())
            .unwrap();
        let progress = walker.progress().unwrap();
        let hashing = thread::spawn(move || {
            walker.init().unwrap();
            let hash = walker.hash().unwrap();
            println!("{hash:?}");
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
        hashing.join();
        tracking.join();
    }
}
