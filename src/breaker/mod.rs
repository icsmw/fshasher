use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// `Breaker` is used for aborting collecting or hashing operations. Take into account, in the scope
/// of usage with `Walker`, the method `collect()` resets the state of `Breaker` to its initial state.
///
/// Cloning: An instance of `Breaker` can be cloned; the cloned instance will be bound with the parent
/// instance. `Breaker` is safe to be shared between threads.
#[derive(Default, Debug, Clone)]
pub struct Breaker {
    state: Arc<AtomicBool>,
}

impl Breaker {
    /// Creates a new instance of `Breaker`.
    ///
    /// # Returns
    ///
    /// - A new `Breaker` instance.
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Resets the instance of `Breaker` to its initial state.
    ///
    /// This method is typically used internally within the `Walker`.
    pub(crate) fn reset(&mut self) {
        self.state.store(false, Ordering::SeqCst)
    }

    /// Returns a closure that, when called, will abort the operation.
    ///
    /// # Returns
    ///
    /// - A closure that sets the internal state to `true`, indicating that an abort has been requested.
    pub fn breaker(&self) -> impl Fn() {
        let signal = self.state.clone();
        move || signal.store(true, Ordering::SeqCst)
    }

    /// Checks if the operation has been aborted.
    ///
    /// # Returns
    ///
    /// - `true` if the operation has been aborted, `false` otherwise.
    pub fn is_aborted(&self) -> bool {
        self.state.load(Ordering::SeqCst)
    }

    /// Aborts the operation by setting the internal state to `true`.
    pub fn abort(&self) {
        self.state.store(true, Ordering::SeqCst)
    }
}
