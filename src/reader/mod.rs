pub mod direct;
pub mod moving;

use memmap2::Mmap;
use std::{error, io::Read, path::Path};

use crate::walker;

pub trait Reader: Read + Send + Sync {
    type Error: error::Error;

    fn setup<P: AsRef<Path>>(&self, path: P) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn clone(&self) -> Self;
    fn mmap(&self) -> Option<Mmap> {
        None
    }
}

#[derive(Debug)]
pub struct ReaderWrapper<T: Reader + Send + Sync> {
    inner: T,
}

impl<T: Reader + Send + Sync> ReaderWrapper<T> {
    pub fn new(inner: T) -> Self {
        ReaderWrapper { inner }
    }
    pub fn setup<P: AsRef<Path>>(&self, path: P) -> Result<Self, walker::E>
    where
        Self: Sized,
    {
        Ok(ReaderWrapper {
            inner: self.inner.setup(path).map_err(walker::E::reader)?,
        })
    }
    pub fn mmap(&self) -> Option<Mmap> {
        self.inner.mmap()
    }
}

impl<T: Reader + Send + Sync> Clone for ReaderWrapper<T> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}
impl<T: Reader + Send + Sync> Read for ReaderWrapper<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}
