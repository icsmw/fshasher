mod error;

use super::Reader;
use error::E;
use memmap2::{Mmap, MmapOptions};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

#[derive(Default)]
pub struct Moving {
    file: Option<File>,
    md: Option<fs::Metadata>,
}

impl Reader for Moving {
    type Error = E;
    fn setup<P: AsRef<Path>>(&self, path: P) -> Result<Self, E>
    where
        Self: Sized,
    {
        Ok(Self {
            file: Some(File::open(path.as_ref())?),
            md: Some(path.as_ref().metadata()?),
        })
    }
    fn clone(&self) -> Self {
        Self::default()
    }
    fn mmap(&self) -> Option<Mmap> {
        if self.md.as_ref()?.len() as usize > usize::MAX {
            None
        } else if let Ok(mmap) = unsafe {
            MmapOptions::new()
                .len(self.md.as_ref()?.len() as usize)
                .map(self.file.as_ref()?)
        } {
            Some(mmap)
        } else {
            None
        }
    }
}

impl Read for Moving {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if let Some(file) = self.file.as_mut() {
            file.read(buffer)
        } else {
            Ok(0)
        }
    }
}
