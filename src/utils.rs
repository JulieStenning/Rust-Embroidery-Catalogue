// Utility functions

#[cfg(test)]
pub(crate) mod test_support {
    use std::sync::{Mutex, MutexGuard, OnceLock};

    /// Process-wide lock for tests that mutate environment variables, the
    /// current working directory, or any `.env` file.
    ///
    /// Cargo runs tests in parallel threads within a single process, so
    /// mutating global process state from more than one module at a time is
    /// a data race that only shows up intermittently (for example under
    /// `llvm-cov`, which changes test timing). Every test that touches such
    /// state must acquire this shared mutex, regardless of which module it
    /// lives in.
    pub(crate) fn env_mutex() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    /// Lock the shared env mutex, recovering from poisoning so a single
    /// panicking test cannot cascade `PoisonError` failures into every other
    /// test that holds the same lock.
    pub(crate) fn lock_env() -> MutexGuard<'static, ()> {
        env_mutex()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}