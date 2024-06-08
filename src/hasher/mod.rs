pub mod blake;
use crate::walker;
use std::error;

pub trait Hasher: Send + Sync {
    type Error: error::Error;

    fn setup(&self) -> Result<Self, Self::Error>
    where
        Self: Sized;
    fn absorb(&mut self, data: &[u8]) -> Result<(), Self::Error>;
    fn finish(&mut self) -> Result<(), Self::Error>;
    fn hash(&self) -> Result<&[u8], Self::Error>;
    fn reset(&mut self) -> Result<(), Self::Error>;
    fn clone(&self) -> Self;
}

#[derive(Debug)]
pub struct HasherWrapper<T: Hasher> {
    inner: T,
}

impl<T: Hasher> HasherWrapper<T> {
    pub fn new(inner: T) -> Self {
        HasherWrapper { inner }
    }
    pub fn setup(&self) -> Result<Self, walker::E>
    where
        Self: Sized,
    {
        Ok(HasherWrapper {
            inner: self.inner.setup().map_err(walker::E::hasher)?,
        })
    }
    pub fn absorb(&mut self, data: &[u8]) -> Result<(), walker::E> {
        self.inner.absorb(data).map_err(walker::E::hasher)
    }
    pub fn finish(&mut self) -> Result<(), walker::E> {
        self.inner.finish().map_err(walker::E::hasher)
    }
    pub fn hash(&self) -> Result<&[u8], walker::E> {
        self.inner.hash().map_err(walker::E::hasher)
    }
    pub fn reset(&mut self) -> Result<(), walker::E> {
        self.inner.reset().map_err(walker::E::hasher)
    }
}

impl<T: Hasher> Clone for HasherWrapper<T> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}
