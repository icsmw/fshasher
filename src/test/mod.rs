use std::path::PathBuf;

use ctor::ctor;

pub(crate) mod usecase;
mod usecases;

pub fn paths_to_cmp_string(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<String>>()
        .join(",")
}

#[ctor]
fn logs() {
    env_logger::init();
}
