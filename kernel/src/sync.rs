//! Architecture-independent synchronization primitives.

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// A non-poisoning mutual exclusion lock that spins while it is contended.
///
/// Do not acquire this lock in an interrupt handler if interrupted code may hold it.
/// Interrupt-safe locking requires architecture-specific interrupt control.
pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    /// Creates an unlocked spin lock containing `value`.
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Acquires the lock, spinning until it becomes available.
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                spin_loop();
            }
        }

        SpinLockGuard { lock: self }
    }

    /// Acquires the lock if it is available without waiting.
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .ok()
            .map(|_| SpinLockGuard { lock: self })
    }

    /// Returns mutable access to the value when the lock itself is exclusively borrowed.
    pub fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }
}

/// An acquired [`SpinLock`] guard.
pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::SpinLock;
    use std::sync::Arc;

    #[test]
    fn try_lock_reports_contention_and_releases_on_drop() {
        let lock = SpinLock::new(7);
        let guard = lock.try_lock().expect("new lock should be available");

        assert_eq!(*guard, 7);
        assert!(lock.try_lock().is_none());
        drop(guard);
        assert!(lock.try_lock().is_some());
    }

    #[test]
    fn lock_serializes_concurrent_updates() {
        let value = Arc::new(SpinLock::new(0_u64));
        let workers = 4;
        let iterations = 10_000;
        let mut threads = Vec::new();

        for _ in 0..workers {
            let value = Arc::clone(&value);
            threads.push(std::thread::spawn(move || {
                for _ in 0..iterations {
                    *value.lock() += 1;
                }
            }));
        }

        for thread in threads {
            thread.join().expect("worker thread should complete");
        }

        assert_eq!(*value.lock(), workers * iterations);
    }
}
