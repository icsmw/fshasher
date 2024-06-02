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
    fn mmap(&self) -> Result<Mmap, E> {
        let md = self.md.as_ref().ok_or(E::SetupIsMissed)?;
        let file = self.file.as_ref().ok_or(E::SetupIsMissed)?;
        if md.len() as usize > usize::MAX {
            Err(E::FileIsTooBig)
        } else {
            Ok(unsafe { MmapOptions::new().len(md.len() as usize).map(file) }?)
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
