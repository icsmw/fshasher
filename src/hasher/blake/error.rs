use thiserror::Error;

#[derive(Error, Debug)]
pub enum E {
    #[error("Hashing not finished")]
    NotFinished,
}
