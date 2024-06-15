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
    fn new() -> Self
    where
        Self: Sized,
    {
        Self::new()
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
