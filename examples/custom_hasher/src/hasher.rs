use crate::error::E;
use fshasher::Hasher;
use sha2::{Digest, Sha256};

pub struct CustomHasher {
    hasher: Sha256,
    hash: Option<Vec<u8>>,
}

impl Default for CustomHasher {
    fn default() -> Self {
        CustomHasher {
            hasher: Sha256::new(),
            hash: None,
        }
    }
}

impl CustomHasher {
    pub fn new() -> Self {
        CustomHasher {
            hasher: Sha256::new(),
            hash: None,
        }
    }
}

impl Hasher for CustomHasher {
    type Error = E;

    fn new() -> Self
    where
        Self: Sized,
    {
        Self::new()
    }

    fn hash(&self) -> Result<&[u8], E> {
        Ok(self.hash.as_ref().ok_or(E::NotFinished)?)
    }

    fn absorb(&mut self, data: &[u8]) -> Result<(), E> {
        self.hasher.update(data);
        Ok(())
    }

    fn finish(&mut self) -> Result<(), E> {
        self.hash = Some(self.hasher.clone().finalize().to_vec());
        Ok(())
    }

    fn clone(&self) -> Self {
        Self::new()
    }
}
