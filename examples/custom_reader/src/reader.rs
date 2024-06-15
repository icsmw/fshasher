use crate::error::E;
use fshasher::Reader;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct CustomReader {
    path: PathBuf,
    file: Option<File>,
}

impl Reader for CustomReader {
    type Error = E;

    fn new<P: AsRef<Path>>(path: P) -> Self
    where
        Self: Sized,
    {
        Self {
            file: None,
            path: path.as_ref().to_path_buf(),
        }
    }

    fn mmap(&mut self) -> Result<&[u8], E> {
        Err(E::MemoryMappingNotSupported)
    }
}

impl Read for CustomReader {
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
