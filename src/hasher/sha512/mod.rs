mod error;

use super::Hasher;
use error::E;
use sha2::{Digest, Sha512 as Origin};

/// Hasher based on `sha2` crate.
pub struct Sha512 {
    hasher: Option<Origin>,
    hash: Option<Vec<u8>>,
}

impl Default for Sha512 {
    /// Creates a default instance of `Sha512` hasher.
    fn default() -> Self {
        Sha512 {
            hasher: Some(Origin::new()),
            hash: None,
        }
    }
}

impl Sha512 {
    /// Creates a new instance of `Sha512` hasher.
    pub fn new() -> Self {
        Sha512 {
            hasher: Some(Origin::new()),
            hash: None,
        }
    }
}

impl Hasher for Sha512 {
    type Error = E;

    /// Creates a new instance of `Sha512` hasher.
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
        Ok(self.hash.as_ref().ok_or(E::NotFinished)?)
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
        if let Some(h) = self.hasher.as_mut() {
            h.update(data)
        }
        Ok(())
    }

    /// Finalizes the hash computation and stores the result.
    ///
    /// # Returns
    ///
    /// - `Ok(())` on success.
    /// - `Err(E)` on error.
    fn finish(&mut self) -> Result<(), E> {
        let Some(hasher) = self.hasher.take() else {
            return Err(E::AlreadyFinished);
        };
        self.hash = Some(hasher.finalize().to_vec());
        Ok(())
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
        utils::compare_same_dest::<hasher::sha512::Sha512, reader::buffering::Buffering>(
            &usecase, None,
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn changes_buffering() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::check_for_changes::<hasher::sha512::Sha512, reader::buffering::Buffering>(
            &usecase, None,
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn correction_complete() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::compare_same_dest::<hasher::sha512::Sha512, reader::buffering::Buffering>(
            &usecase,
            Some(ReadingStrategy::Complete),
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn changes_complete() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::check_for_changes::<hasher::sha512::Sha512, reader::buffering::Buffering>(
            &usecase,
            Some(ReadingStrategy::Complete),
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn correction_mapped() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::compare_same_dest::<hasher::sha512::Sha512, reader::mapping::Mapping>(
            &usecase,
            Some(ReadingStrategy::MemoryMapped),
        )?;
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn changes_mapped() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        utils::check_for_changes::<hasher::sha512::Sha512, reader::mapping::Mapping>(
            &usecase,
            Some(ReadingStrategy::MemoryMapped),
        )?;
        usecase.clean()?;
        Ok(())
    }
}
