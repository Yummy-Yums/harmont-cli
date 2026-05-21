//! Cancellation atomic.
//!
//! Ctrl-C handlers and orchestrator failure paths flip the atomic;
//! the `hm_should_cancel` host fn reports its state to plugins;
//! plugins poll between long-running operations and unwind quickly.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    inner: Arc<AtomicBool>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.inner.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_not_cancelled() {
        assert!(!CancellationToken::new().is_cancelled());
    }

    #[test]
    fn cancel_persists() {
        let t = CancellationToken::new();
        t.cancel();
        assert!(t.is_cancelled());
    }

    #[test]
    fn cancel_is_clone_shared() {
        let t = CancellationToken::new();
        let u = t.clone();
        t.cancel();
        assert!(u.is_cancelled());
    }
}
