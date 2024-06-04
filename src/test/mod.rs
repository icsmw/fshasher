mod collector;
pub(crate) mod usecase;
pub(crate) mod utils;
mod walker;

use ctor::ctor;

#[ctor]
fn logs() {
    env_logger::init();
}
