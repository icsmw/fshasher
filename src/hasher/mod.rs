pub mod blake;
use crate::walker;
use std::error;

/// A trait that defines the behavior of a hasher, which is used to process and compute hashes.
/// Implementers of this trait must also implement `Send` and `Sync`.
///
/// `Walker` takes one instance of a hasher during the creation of a new `Walker` instance.
/// This instance will be used by `Walker` as follows for each file that needs to be hashed:
/// - Clone the instance of `Hasher` (with method `clone()`).
/// - Setup/initialize the instance (with method `setup()`).
/// - Add file's content during reading (with method `absorb(..)`).
/// - Finalize hash calculation for the file (with method `finish()`).
/// - Request file's hash (with method `hash()`).
/// - Drop the instance of `Hasher`.
pub trait Hasher: Send + Sync {
    /// The type of error that can occur during operations.
    type Error: error::Error;

    /// Sets up the hasher. This method may perform any necessary initialization. This method will
    /// be called only once per each file. For each new file, a new instance of `Hasher` will be
    /// cloned.
    ///
    /// # Returns
    ///
    /// - `Result<Self, Self::Error>`: On success, returns an instance of the hasher. On failure,
    ///   returns an error of type `Self::Error`.
    fn setup(&self) -> Result<Self, Self::Error>
    where
        Self: Sized;

    /// Absorbs data into the hasher. This method processes the input data and updates the hasher
    /// state. This method might be called multiple times during the reading of a file.
    ///
    /// # Parameters
    ///
    /// - `data`: A reference to a slice of bytes to be absorbed by the hasher.
    ///
    /// # Returns
    ///
    /// - `Result<(), Self::Error>`: On success, returns `Ok(())`. On failure,
    ///   returns an error of type `Self::Error`.
    fn absorb(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Finalizes the hashing process. This method should be called after all data has been absorbed.
    /// This method will be called only once for each file.
    ///
    /// # Returns
    ///
    /// - `Result<(), Self::Error>`: On success, returns `Ok(())`. On failure,
    ///   returns an error of type `Self::Error`.
    fn finish(&mut self) -> Result<(), Self::Error>;

    /// Retrieves the computed hash. This method should be called after `finish` to get the resulting hash.
    ///
    /// # Returns
    ///
    /// - `Result<&[u8], Self::Error>`: On success, returns a reference to the computed hash. On failure,
    ///   returns an error of type `Self::Error`.
    fn hash(&self) -> Result<&[u8], Self::Error>;

    /// Creates a clone of the hasher.
    ///
    /// # Returns
    ///
    /// - `Self`: A cloned instance of the hasher.
    fn clone(&self) -> Self;
}

/// A wrapper for the `Hasher` trait that provides additional functionality.
/// `Hasher` isn't used directly in `Walker`. Instead, `Walker` uses `HasherWrapper`,
/// which helps better manage error handling.
#[derive(Debug)]
pub struct HasherWrapper<T: Hasher> {
    inner: T,
}

impl<T: Hasher> HasherWrapper<T> {
    /// Creates a new `HasherWrapper` instance.
    ///
    /// # Parameters
    ///
    /// - `inner`: The inner hasher instance.
    ///
    /// # Returns
    ///
    /// - `HasherWrapper<T>`: A new `HasherWrapper` instance.
    pub fn new(inner: T) -> Self {
        HasherWrapper { inner }
    }

    /// Sets up the inner hasher. This method may perform any necessary initialization.
    ///
    /// # Returns
    ///
    /// - `Result<Self, walker::E>`: On success, returns a new `HasherWrapper` instance with the setup hasher.
    ///   On failure, returns an error of type `walker::E`.
    pub fn setup(&self) -> Result<Self, walker::E>
    where
        Self: Sized,
    {
        Ok(HasherWrapper {
            inner: self.inner.setup().map_err(walker::E::hasher)?,
        })
    }

    /// Absorbs data into the inner hasher. This method processes the input data and updates the hasher state.
    ///
    /// # Parameters
    ///
    /// - `data`: A reference to a slice of bytes to be absorbed by the hasher.
    ///
    /// # Returns
    ///
    /// - `Result<(), walker::E>`: On success, returns `Ok(())`. On failure, returns an error of type `walker::E`.
    pub fn absorb(&mut self, data: &[u8]) -> Result<(), walker::E> {
        self.inner.absorb(data).map_err(walker::E::hasher)
    }

    /// Finalizes the hashing process of the inner hasher. This method should be called after all data has been absorbed.
    ///
    /// # Returns
    ///
    /// - `Result<(), walker::E>`: On success, returns `Ok(())`. On failure, returns an error of type `walker::E`.
    pub fn finish(&mut self) -> Result<(), walker::E> {
        self.inner.finish().map_err(walker::E::hasher)
    }

    /// Retrieves the computed hash from the inner hasher. This method should be called after `finish` to get the resulting hash.
    ///
    /// # Returns
    ///
    /// - `Result<&[u8], walker::E>`: On success, returns a reference to the computed hash. On failure, returns an error of type `walker::E`.
    pub fn hash(&self) -> Result<&[u8], walker::E> {
        self.inner.hash().map_err(walker::E::hasher)
    }
}

impl<T: Hasher> Clone for HasherWrapper<T> {
    fn clone(&self) -> Self {
        Self::new(self.inner.clone())
    }
}
