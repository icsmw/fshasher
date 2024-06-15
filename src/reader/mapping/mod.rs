mod error;

use super::Reader;
use error::E;
use memmap2::{Mmap, MmapOptions};
use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

/// This reader supports all strategies: `ReadingStrategy::MemoryMapped`, `ReadingStrategy::Buffer` and
/// `ReadingStrategy::Complete`.
///
/// If `ReadingStrategy::MemoryMapped` is used, it maps the file into memory and gives the `hasher`
/// access to the full content of the file.
///
/// The reader should be used carefully because the `hasher` might not be optimized for large amounts of data.
/// The recommended way to use this reader is with `ReadingStrategy::Scenario`. With this strategy, you will
/// be able to define a file size limit for using this reader.
#[derive(Default)]
pub struct Mapping {
    path: PathBuf,
    file: Option<File>,
    md: Option<fs::Metadata>,
    mmap: Option<Mmap>,
}

impl Reader for Mapping {
    type Error = E;

    /// Creates a `Mapping` reader bound to the specified path.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to the file to be read.
    ///
    /// # Returns
    ///
    /// - A new instance of `Mapping` reader bound to the specified path.
    fn new<P: AsRef<Path>>(path: P) -> Self
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

    /// Maps the file into memory and returns a reference to its content.
    ///
    /// # Returns
    ///
    /// - `Ok(&[u8])` with the file content if mapping is successful.
    /// - `Err(E)` if an error occurs or if memory mapping is not supported.
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
    /// Reads a chunk of data into the provided buffer.
    ///
    /// # Parameters
    ///
    /// - `buffer`: A mutable slice of bytes where the read data will be stored.
    ///
    /// # Returns
    ///
    /// - `Ok(usize)`: The number of bytes read.
    /// - `Err(std::io::Error)`: An error occurred during reading.
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

#[cfg(test)]
mod test {
    use crate::{
        hasher, reader,
        test::{usecase::*, utils},
        ReadingStrategy, E,
    };

    #[test]
    fn correction() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::compare_same_dest::<hasher::blake::Blake, reader::mapping::Mapping>(
            &usecase,
            Some(ReadingStrategy::MemoryMapped),
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn changes() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::check_for_changes::<hasher::blake::Blake, reader::mapping::Mapping>(
            &usecase,
            Some(ReadingStrategy::MemoryMapped),
        )?;
        usecase.clean()?;
        Ok(())
    }
}
