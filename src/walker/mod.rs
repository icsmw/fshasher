mod collector;
mod entry;
mod error;
mod options;

use crate::{hasher::HasherWrapper, reader::ReaderWrapper, Breaker, Hasher, Reader};
use collector::Collector;
pub use entry::{Entry, Filter};
pub use error::E;
use log::debug;
pub use options::{Options, Tolerance};
use std::{io::Read, mem, path::PathBuf, time::Instant};

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
}

impl<H: Hasher, R: Reader> Walker<H, R> {
    pub fn new(opt: Options, hasher: H, reader: R) -> Result<Self, E> {
        Ok(Self {
            opt: Some(opt),
            breaker: Breaker::new(),
            paths: Vec::new(),
            invalid: Vec::new(),
            cursor: 0,
            hasher: HasherWrapper::new(hasher),
            reader: ReaderWrapper::new(reader),
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

    pub fn hash(&mut self) -> Result<&[u8], E> {
        let now = Instant::now();
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
        }
        self.hasher.finish()?;
        debug!(
            "hashing of {} paths in {}µs / {}ms / {}s",
            self.total(),
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
}
