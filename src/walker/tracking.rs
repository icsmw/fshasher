use crate::{walker::E, Hasher, Reader, Walker};
use blake3::Hasher as BlakeHasher;
use bstorage::Storage;
use serde::{Deserialize, Serialize};
use std::{env::temp_dir, path::PathBuf};

pub(crate) fn get_default_path() -> PathBuf {
    dirs::home_dir()
        .or_else(dirs::data_local_dir)
        .unwrap_or(temp_dir())
        .join(".fshasher")
        .join(get_storage_name())
}

pub(crate) fn get_storage_name() -> String {
    "tracking".to_owned()
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PreviousHash {
    hash: Vec<u8>,
}

/// Available with feature "tracking". Provides the `is_same()` method.
/// `fshasher` will create storage to save information about recently calculated hashes. Using the `is_same()` method
/// it will be possible to detect if any changes have occurred or not.
///
/// Since the data is saved permanently on the disk, the `is_same()` method will provide accurate information between
/// application's runs.
pub trait Tracking {
    /// Returns information whether the hash of the destination has changed (since the last check) or not.
    ///
    /// # Returns
    ///
    /// - `bool` - `true` if the content is the same; `false` if something has changed.
    ///
    /// # Errors
    ///
    /// Returns an error if the hashing or reading operation fails.
    ///
    /// # Example
    ///
    /// ```
    /// use fshasher::{hasher, reader, Entry, Options, Tolerance, Tracking};
    /// use std::env::temp_dir;
    ///
    /// let mut walker = Options::new()
    ///     .entry(Entry::from(temp_dir()).unwrap())
    ///     .unwrap()
    ///     .tolerance(Tolerance::LogErrors)
    ///     .walker()
    ///     .unwrap();
    /// // false - because never checked before
    /// println!(
    ///     "First check: {}",
    ///     walker
    ///         .is_same::<hasher::blake::Blake, reader::buffering::Buffering>()
    ///         .unwrap()
    /// );
    /// // true - because checked before
    /// println!(
    ///     "Second check: {}",
    ///     walker
    ///         .is_same::<hasher::blake::Blake, reader::buffering::Buffering>()
    ///         .unwrap()
    /// );
    /// ```
    fn is_same<H: Hasher + 'static, R: Reader + 'static>(&mut self) -> Result<bool, E>
    where
        E: From<<H as Hasher>::Error> + From<<R as Reader>::Error>;
}

impl Tracking for Walker {
    fn is_same<H: Hasher + 'static, R: Reader + 'static>(&mut self) -> Result<bool, E>
    where
        E: From<<H as Hasher>::Error> + From<<R as Reader>::Error>,
    {
        let opt = self.opt.as_ref().ok_or(E::IsNotInited)?;
        let storage = opt.storage.clone();
        let alias = BlakeHasher::new()
            .update(&opt.hash())
            .finalize()
            .to_string();
        let nhash = self.collect()?.hash::<H, R>()?.to_vec();
        let mut storage = Storage::create(storage)?;
        let mut phash: PreviousHash = storage.get_or_default(&alias)?;
        let changed = phash.hash == nhash;
        phash.hash = nhash;
        storage.set(alias, &phash)?;
        Ok(changed)
    }
}

#[cfg(test)]
mod test {
    use std::{env::temp_dir, fs::remove_dir_all};

    use crate::{hasher, reader, test::usecase::*, Entry, Options, Tolerance, Tracking, E};

    #[test]
    fn tracking() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        let mut walker = Options::new()
            .entry(Entry::from(&usecase.root)?)?
            .tolerance(Tolerance::LogErrors)
            .walker()?;
        let mut states: Vec<bool> = Vec::new();
        states.push(walker.is_same::<hasher::blake::Blake, reader::buffering::Buffering>()?); // false
        states.push(walker.is_same::<hasher::blake::Blake, reader::buffering::Buffering>()?); // true
        states.push(walker.is_same::<hasher::blake::Blake, reader::buffering::Buffering>()?); // true
        usecase.change(1)?;
        states.push(walker.is_same::<hasher::blake::Blake, reader::buffering::Buffering>()?); // false
        states.push(walker.is_same::<hasher::blake::Blake, reader::buffering::Buffering>()?); // true
        assert!(!states[0]);
        assert!(states[1]);
        assert!(states[2]);
        assert!(!states[3]);
        assert!(states[4]);
        usecase.clean()?;
        Ok(())
    }

    #[test]
    fn tracking_custom_storage_path() -> Result<(), E> {
        let usecase = UseCase::unnamed(2, 2, 2, &[])?;
        let custom_path = temp_dir().join("custom_path");
        let mut walker = Options::new()
            .entry(Entry::from(&usecase.root)?)?
            .tolerance(Tolerance::LogErrors)
            .storage(&custom_path)?
            .walker()?;
        let mut states: Vec<bool> = Vec::new();
        states.push(walker.is_same::<hasher::blake::Blake, reader::buffering::Buffering>()?); // false
        states.push(walker.is_same::<hasher::blake::Blake, reader::buffering::Buffering>()?); // true
        states.push(walker.is_same::<hasher::blake::Blake, reader::buffering::Buffering>()?); // true
        usecase.change(1)?;
        states.push(walker.is_same::<hasher::blake::Blake, reader::buffering::Buffering>()?); // false
        states.push(walker.is_same::<hasher::blake::Blake, reader::buffering::Buffering>()?); // true
        assert!(!states[0]);
        assert!(states[1]);
        assert!(states[2]);
        assert!(!states[3]);
        assert!(states[4]);
        usecase.clean()?;
        remove_dir_all(custom_path)?;
        Ok(())
    }
}
