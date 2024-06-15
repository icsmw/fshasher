mod error;

use super::Reader;
use error::E;
use std::io;
use std::{
    io::Read,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Md {
    path: PathBuf,
    done: bool,
}

impl Reader for Md {
    type Error = E;
    fn unbound() -> Self {
        Self::default()
    }
    fn bound<P: AsRef<Path>>(path: P) -> Self
    where
        Self: Sized,
    {
        Self {
            path: path.as_ref().to_path_buf(),
            done: false,
        }
    }
    fn mmap(&mut self) -> Result<&[u8], E> {
        Err(E::MemoryMappingNotSupported)
    }
}

impl Read for Md {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if self.done {
            Ok(0)
        } else {
            let md = self.path.metadata()?;
            let like_hash = format!("{};{}", md.ctime(), md.size());
            let as_bytes = like_hash.as_bytes();
            if as_bytes.len() < buffer.len() {
                Err(io::Error::new(
                    io::ErrorKind::Other,
                    String::from("Md reader needs at least 255 bytes buffer"),
                ))
            } else {
                buffer[..as_bytes.len()].copy_from_slice(as_bytes);
                Ok(as_bytes.len())
            }
        }
    }
}
