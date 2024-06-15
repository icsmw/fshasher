mod error;
mod reader;

use fshasher::{hasher::blake::Blake, Options};
use reader::CustomReader;
use std::env::temp_dir;

fn main() {
    let mut walker = Options::new()
        .tolerance(fshasher::Tolerance::LogErrors)
        .path(temp_dir())
        .expect("System tmp folder exist")
        .walker()
        .expect("Walker is created");
    let hash = walker
        .collect()
        .expect("Files are collected from tmp")
        .hash::<Blake, CustomReader>()
        .expect("Hash calculated")
        .to_vec();
    println!("Hash of {hash:?} files: {}", walker.count());
}
