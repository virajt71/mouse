use std::sync::{Mutex, MutexGuard};

pub trait MutexExt<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T>;
}

impl<T> MutexExt<T> for Mutex<T> {
    fn lock_safe(&self) -> MutexGuard<'_, T> {
        match self.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                log::warn!("Recovering from poisoned Mutex");
                poisoned.into_inner()
            }
        }
    }
}
