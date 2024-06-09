mod error;

use super::Reader;
use error::E;
use std::{fs::File, io::Read, path::Path};

#[derive(Default)]
pub struct Buffering {
    file: Option<File>,
}

impl Reader for Buffering {
    type Error = E;
    fn bind<P: AsRef<Path>>(&self, path: P) -> Result<Self, E>
    where
        Self: Sized,
    {
        Ok(Self {
            file: Some(File::open(path.as_ref())?),
        })
    }
    fn clone(&self) -> Self {
        Self::default()
    }
    fn mmap(&mut self) -> Result<&[u8], E> {
        Err(E::MemoryMappingNotSupported)
    }
}

impl Read for Buffering {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if let Some(file) = self.file.as_mut() {
            file.read(buffer)
        } else {
            Ok(0)
        }
    }
}
