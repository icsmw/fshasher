mod error;

use super::Reader;
use error::E;
use std::io;
use std::time::UNIX_EPOCH;
use std::{
    io::Read,
    path::{Path, PathBuf},
};

/// Actually fake reader. It doesn't mean it does nothing, but instead of reading the file, it reads
/// the metadata of the file and returns as bytes to the `hasher` the date of the last modification
/// of the file and its size.
///
/// Obviously, this reader will give very fast results, but it should be used only if you are sure
/// checking the metadata would be enough to make the right conclusion.
#[derive(Default)]
pub struct Md {
    path: PathBuf,
    done: bool,
}

impl Reader for Md {
    type Error = E;

    /// Creates a `Md` reader bound to the specified path.
    ///
    /// # Parameters
    ///
    /// - `path`: The path to the file to be read.
    ///
    /// # Returns
    ///
    /// - A new instance of `Md` reader bound to the specified path.
    fn new<P: AsRef<Path>>(path: P) -> Self
    where
        Self: Sized,
    {
        Self {
            path: path.as_ref().to_path_buf(),
            done: false,
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

impl Read for Md {
    /// Reads the metadata of the file and returns it as bytes.
    ///
    /// # Parameters
    ///
    /// - `buffer`: A mutable slice of bytes where the read data will be stored.
    ///
    /// # Returns
    ///
    /// - `Ok(usize)`: The number of bytes read.
    /// - `Err(std::io::Error)`: An error occurred during reading.
    ///
    /// This method will read the metadata only once. Subsequent reads will return 0.
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.done {
            Ok(0)
        } else {
            self.done = true;
            let md = self.path.metadata()?;
            let modified = md
                .modified()?
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .map_err(|err| {
                    io::Error::new(io::ErrorKind::Other, format!("SystemTimeError: {err}"))
                })?;
            let as_bytes = [
                modified.to_be_bytes().as_ref(),
                md.len().to_be_bytes().as_ref(),
            ]
            .concat();
            if as_bytes.len() > buffer.len() {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    String::from("Md reader needs at least 255 bytes buffer"),
                ))
            } else {
                buffer[..as_bytes.len()].copy_from_slice(&as_bytes);
                Ok(as_bytes.len())
            }
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
    fn correction_chunked() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::compare_same_dest::<hasher::blake::Blake, reader::md::Md>(&usecase, None)?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn changes_chunked() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::check_for_changes::<hasher::blake::Blake, reader::md::Md>(&usecase, None)?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn correction_complete() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::compare_same_dest::<hasher::blake::Blake, reader::md::Md>(
            &usecase,
            Some(ReadingStrategy::Complete),
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn changes_complete() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::check_for_changes::<hasher::blake::Blake, reader::md::Md>(
            &usecase,
            Some(ReadingStrategy::Complete),
        )?;
        usecase.clean()?;
        Ok(())
    }
}
