mod error;

use super::Hasher;
use blake3::{Hash, Hasher as BlakeHasher};
use error::E;
/// Hasher based on `blake3` crate.
pub struct Blake {
    hasher: BlakeHasher,
    hash: Option<Hash>,
}

impl Default for Blake {
    /// Creates a default instance of `Blake` hasher.
    fn default() -> Self {
        Blake {
            hasher: BlakeHasher::new(),
            hash: None,
        }
    }
}

impl Blake {
    /// Creates a new instance of `Blake` hasher.
    pub fn new() -> Self {
        Blake {
            hasher: BlakeHasher::new(),
            hash: None,
        }
    }
}

impl Hasher for Blake {
    type Error = E;

    /// Creates a new instance of `Blake` hasher.
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::new()
    }

    /// Returns the computed hash.
    ///
    /// # Returns
    ///
    /// - `Ok(&[u8])` containing the hash bytes if hashing is finished.
    /// - `Err(E)` if the hash is not yet finalized.
    fn hash(&self) -> Result<&[u8], E> {
        Ok(self.hash.as_ref().ok_or(E::NotFinished)?.as_bytes())
    }

    /// Absorbs input data into the hasher.
    ///
    /// # Parameters
    ///
    /// - `data`: A slice of bytes to be hashed.
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success.
    /// - `Err(E)` on error.
    fn absorb(&mut self, data: &[u8]) -> Result<(), E> {
        self.hasher.update(data);
        Ok(())
    }

    /// Finalizes the hash computation and stores the result.
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success.
    /// - `Err(E)` on error.
    fn finish(&mut self) -> Result<(), E> {
        self.hash = Some(self.hasher.finalize());
        Ok(())
    }

    /// Creates a new instance of `Blake` hasher, effectively cloning it.
    ///
    /// # Returns
    ///
    /// - A new instance of `Blake`.
    fn clone(&self) -> Self {
        Self::new()
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
    fn correction_buffering() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::compare_same_dest::<hasher::blake::Blake, reader::buffering::Buffering>(
            &usecase, None,
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn changes_buffering() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::check_for_changes::<hasher::blake::Blake, reader::buffering::Buffering>(
            &usecase, None,
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn correction_complete() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::compare_same_dest::<hasher::blake::Blake, reader::buffering::Buffering>(
            &usecase,
            Some(ReadingStrategy::Complete),
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn changes_complete() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::check_for_changes::<hasher::blake::Blake, reader::buffering::Buffering>(
            &usecase,
            Some(ReadingStrategy::Complete),
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn correction_mapped() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::compare_same_dest::<hasher::blake::Blake, reader::mapping::Mapping>(
            &usecase,
            Some(ReadingStrategy::MemoryMapped),
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn changes_mapped() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::check_for_changes::<hasher::blake::Blake, reader::mapping::Mapping>(
            &usecase,
            Some(ReadingStrategy::MemoryMapped),
        )?;
        usecase.clean()?;
        Ok(())
    }
}
