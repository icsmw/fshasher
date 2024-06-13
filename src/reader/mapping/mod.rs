mod error;

use super::Reader;
use error::E;
use memmap2::{Mmap, MmapOptions};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Mapping {
    path: PathBuf,
    file: Option<File>,
    md: Option<fs::Metadata>,
    mmap: Option<Mmap>,
}

impl Reader for Mapping {
    type Error = E;
    fn bind<P: AsRef<Path>>(&self, path: P) -> Self
    where
        Self: Sized,
    {
        Self {
            path: path.as_ref().to_path_buf(),
            file: None,
            md: None,
            mmap: None,
        }
    }
    fn clone(&self) -> Self {
        Self::default()
    }
    fn mmap(&mut self) -> Result<&[u8], E> {
        if self.file.is_none() {
            let file = File::open(&self.path)?;
            self.md = Some(file.metadata()?);
            self.file = Some(file);
        }
        if self.md.is_none() {
            self.file = Some(File::open(&self.path)?);
        }
        let md = self.md.as_ref().ok_or(E::SetupIsMissed)?;
        let file = self.file.as_ref().ok_or(E::SetupIsMissed)?;
        if md.len() as usize > usize::MAX {
            Err(E::FileIsTooBig)
        } else {
            self.mmap = Some(unsafe { MmapOptions::new().len(md.len() as usize).map(file) }?);
            if let Some(mmap) = self.mmap.as_ref() {
                Ok(&mmap[..])
            } else {
                unreachable!("File has been mapped into memory");
            }
        }
    }
}

impl Read for Mapping {
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
