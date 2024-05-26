use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

#[derive(Default, Debug)]
pub struct Breaker {
    state: Arc<AtomicBool>,
}

impl Breaker {
    pub fn new() -> Self {
        Self {
            state: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn breaker(&self) -> impl Fn() {
        let signal = self.state.clone();
        move || signal.store(true, Ordering::Relaxed)
    }

    pub fn is_aborded(&self) -> bool {
        self.state.load(Ordering::Relaxed)
    }

    pub fn abort(&self) {
        self.state.store(true, Ordering::Relaxed)
    }
}
