pub mod buffering;
pub mod mapping;

use std::{error, io::Read, path::Path};

use crate::walker;

/// A trait that extends the standard `Read` trait with additional capabilities for reading data.
/// Implementers of this trait must also implement `Send` and `Sync`.
///
/// `Walker` takes one instance of a reader during the creation of a new `Walker` instance.
/// This instance will be used by `Walker` as follows for each file that needs to be read and hashed:
/// - Clone the instance of `Reader`.
/// - Bind the instance to the target file.
/// - Read the file using the instance of `Reader`.
/// - Drop the instance of `Reader`.
pub trait Reader: Read + Send + Sync {
    /// The type of error that can occur during operations.
    type Error: error::Error + Into<walker::E>;

    /// Binds the reader to the specified file path.
    ///
    /// # Parameters
    ///
    /// - `path`: A reference to a path that the reader will be bound to.
    ///
    /// # Returns
    ///
    /// - `Result<Self, Self::Error>`: On success, returns an instance of the reader. On failure,
    ///   returns an error of type `Self::Error`.
    fn bind<P: AsRef<Path>>(&self, path: P) -> Self
    where
        Self: Sized;

    /// Creates a clone of the reader.
    ///
    /// # Returns
    ///
    /// - `Self`: A cloned instance of the reader.
    fn clone(&self) -> Self;

    /// Memory-maps the file for reading. This method must be implemented only if the reader supports
    /// mapping the file into memory. This method will be called only if `Walker` is used with the
    /// `ReadingStrategy::MemoryMapped`.
    ///
    /// If the implementation of `Reader` doesn't support memory mapping, it should return an error.
    ///
    /// # Returns
    ///
    /// - `Result<&[u8], Self::Error>`: On success, returns a reference to the memory-mapped data.
    ///   On failure, returns an error of type `Self::Error`.
    fn mmap(&mut self) -> Result<&[u8], Self::Error>;
}
