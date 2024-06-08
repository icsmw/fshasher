mod error;

use super::Hasher;
use blake3::{Hash, Hasher as BlakeHasher};
use error::E;
pub struct Blake {
    hasher: BlakeHasher,
    hash: Option<Hash>,
}

impl Default for Blake {
    fn default() -> Self {
        Blake {
            hasher: BlakeHasher::new(),
            hash: None,
        }
    }
}

impl Blake {
    pub fn new() -> Self {
        Blake {
            hasher: BlakeHasher::new(),
            hash: None,
        }
    }
}

impl Hasher for Blake {
    type Error = E;
    fn setup(&self) -> Result<Self, Self::Error>
    where
        Self: Sized,
    {
        Ok(Self::new())
    }
    fn hash(&self) -> Result<&[u8], E> {
        Ok(self.hash.as_ref().ok_or(E::NotFinished)?.as_bytes())
    }
    fn absorb(&mut self, data: &[u8]) -> Result<(), E> {
        self.hasher.update(data);
        Ok(())
    }
    fn finish(&mut self) -> Result<(), E> {
        self.hash = Some(self.hasher.finalize());
        Ok(())
    }
    fn reset(&mut self) -> Result<(), E> {
        self.hasher.reset();
        Ok(())
    }
    fn clone(&self) -> Self {
        Self::new()
    }
}
