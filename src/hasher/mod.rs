pub mod blake;
#[cfg(feature = "use_sha2")]
pub mod sha256;
#[cfg(feature = "use_sha2")]
pub mod sha512;

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
    type Error: error::Error + Into<walker::E>;

    /// Sets up the hasher. This method may perform any necessary initialization. This method will
    /// be called only once per each file. For each new file, a new instance of `Hasher` will be
    /// cloned.
    ///
    /// # Returns
    ///
    /// - `Result<Self, Self::Error>`: On success, returns an instance of the hasher. On failure,
    ///   returns an error of type `Self::Error`.
    fn new() -> Self
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
}
