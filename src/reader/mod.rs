pub(crate) mod direct;
use std::{error, io::Read, path::Path};

use crate::walker;

pub trait Reader: Read {
    type Error: error::Error;

    fn setup<P: AsRef<Path>>(&self, path: P) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct ReaderWrapper<T: Reader> {
    inner: T,
}

impl<T: Reader> ReaderWrapper<T> {
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
}

impl<T: Reader> Read for ReaderWrapper<T> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}
