mod error;

use super::Reader;
use error::E;
use std::{fs::File, io::Read, path::Path};

pub struct Direct {
    file: Option<File>,
}

impl Direct {
    pub fn new() -> Self {
        Self { file: None }
    }
}

impl Reader for Direct {
    type Error = E;
    fn setup<P: AsRef<Path>>(&self, path: P) -> Result<Self, E>
    where
        Self: Sized,
    {
        Ok(Self {
            file: Some(File::open(path.as_ref())?),
        })
    }
    fn clone(&self) -> Self {
        Self::new()
    }
}

impl Read for Direct {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if let Some(file) = self.file.as_mut() {
            file.read(buffer)
        } else {
            Ok(0)
        }
    }
}
