pub mod breaker;
mod error;
pub mod hasher;
pub mod reader;
pub mod walker;

pub use breaker::Breaker;
use error::E;
pub use hasher::Hasher;
pub use reader::Reader;
pub use walker::{Options, Walker};

// pub fn hash(mut opt: Options) -> Result<(), E> {
//     let walker = opt.walker()?;
//     Ok(())
// }

// trait MyTrait {
//     fn my_method(&self) -> u8;
// }

// struct A {}

// impl MyTrait for A {
//     fn my_method(&self) -> u8 {
//         0u8
//     }
// }

// struct Holder<T: MyTrait> {
//     field: T,
// }

// impl<T: MyTrait> Holder<T> {
//     pub fn new() -> Self {
//         Self { field: A {} }
//     }
// }
