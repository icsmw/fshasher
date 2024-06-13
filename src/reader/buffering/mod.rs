mod error;

use super::Reader;
use error::E;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Buffering {
    path: PathBuf,
    file: Option<File>,
}

impl Reader for Buffering {
    type Error = E;
    fn bind<P: AsRef<Path>>(&self, path: P) -> Self
    where
        Self: Sized,
    {
        Self {
            file: None,
            path: path.as_ref().to_path_buf(),
        }
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
        if self.file.is_none() {
            self.file = Some(File::open(&self.path)?);
        }
        if let Some(file) = self.file.as_mut() {
            file.read(buffer)
        } else {
            Ok(0)
        }
    }
}
