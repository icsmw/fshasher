mod error;

use super::Reader;
use error::E;
use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

/// Regular reader based on reading file chunk by chunk. This reader doesn't support mapping files
/// into memory and will return an error when attempting to use `ReadingStrategy::MemoryMapped`.
#[derive(Default)]
pub struct Buffering {
    path: PathBuf,
    file: Option<File>,
}

impl Reader for Buffering {
    type Error = E;

    /// Creates an unbound `Buffering` reader with default values.
    fn unbound() -> Self {
        Self::default()
    }

    /// Creates a `Buffering` reader bound to the specified path.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to the file to be read.
    ///
    /// # Returns
    ///
    /// - A new instance of `Buffering` reader bound to the specified path.
    fn bound<P: AsRef<Path>>(path: P) -> Self
    where
        Self: Sized,
    {
        Self {
            file: None,
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Returns an error as memory mapping is not supported by this reader.
    ///
    /// # Returns
    ///
    /// - `Err(E::MemoryMappingNotSupported)` always.
    fn mmap(&mut self) -> Result<&[u8], E> {
        Err(E::MemoryMappingNotSupported)
    }
}

impl Read for Buffering {
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
        E,
    };

    #[test]
    fn correction() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::compare_same_dest::<hasher::blake::Blake, reader::buffering::Buffering>(
            &usecase, None,
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn changes() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::check_for_changes::<hasher::blake::Blake, reader::buffering::Buffering>(
            &usecase, None,
        )?;
        usecase.clean()?;
        Ok(())
    }
}
