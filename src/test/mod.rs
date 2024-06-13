mod collector;
mod prankster;
pub(crate) mod usecase;
pub(crate) mod utils;
mod walker;

use ctor::ctor;

#[ctor]
fn logs() {
    env_logger::init();
}

const STRESS_TEST_ITERATIONS_LIMIT: usize = 500;
const STRESS_TEST_ITERATIONS_LIMIT_ENVVAR: &str = "FSHASHER_STRESS_TEST_ITER_LIM";

/// Returns the number of repeated tests (stress test). For CI having a huge number of tests might cause errors
/// related to resource limitations. To limit it can be used FSHASHER_STRESS_TEST_ITER_LIM variable.
pub fn get_stress_iterations_count() -> usize {
    std::env::var(STRESS_TEST_ITERATIONS_LIMIT_ENVVAR)
        .map(|v| v.parse::<usize>().unwrap_or(STRESS_TEST_ITERATIONS_LIMIT))
        .unwrap_or(STRESS_TEST_ITERATIONS_LIMIT)
}
