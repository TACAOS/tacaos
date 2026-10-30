//! Architecture-independent synchronization primitives.

use core::cell::UnsafeCell;
use core::hint::spin_loop;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

const ONCE_EMPTY: u8 = 0;
const ONCE_INITIALIZING: u8 = 1;
const ONCE_READY: u8 = 2;

/// A value that is initialized at most once and can be read by all threads.
pub struct OnceCell<T> {
    state: AtomicU8,
    value: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send> Send for OnceCell<T> {}
unsafe impl<T: Send + Sync> Sync for OnceCell<T> {}

impl<T> OnceCell<T> {
    /// Creates an empty cell.
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(ONCE_EMPTY),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Returns the initialized value, or `None` if initialization has not completed.
    pub fn get(&self) -> Option<&T> {
        if self.state.load(Ordering::Acquire) == ONCE_READY {
            Some(unsafe { (&*self.value.get()).assume_init_ref() })
        } else {
            None
        }
    }

    /// Initializes the cell once and returns the shared value.
    pub fn get_or_init(&self, initialize: impl FnOnce() -> T) -> &T {
        let mut initialize = Some(initialize);

        loop {
            match self.state.load(Ordering::Acquire) {
                ONCE_READY => return unsafe { (&*self.value.get()).assume_init_ref() },
                ONCE_EMPTY => {
                    if self
                        .state
                        .compare_exchange(
                            ONCE_EMPTY,
                            ONCE_INITIALIZING,
                            Ordering::Acquire,
                            Ordering::Relaxed,
                        )
                        .is_ok()
                    {
                        let mut reset = OnceInitGuard {
                            state: &self.state,
                            completed: false,
                        };
                        let Some(initialize) = initialize.take() else {
                            continue;
                        };
                        unsafe { (*self.value.get()).write(initialize()) };
                        self.state.store(ONCE_READY, Ordering::Release);
                        reset.completed = true;
                        return unsafe { (&*self.value.get()).assume_init_ref() };
                    }
                }
                ONCE_INITIALIZING => spin_loop(),
                _ => spin_loop(),
            }
        }
    }
}

impl<T> Drop for OnceCell<T> {
    fn drop(&mut self) {
        if self.state.load(Ordering::Relaxed) == ONCE_READY {
            // The ready state is published only after the value is initialized.
            unsafe { self.value.get_mut().assume_init_drop() };
        }
    }
}

struct OnceInitGuard<'a> {
    state: &'a AtomicU8,
    completed: bool,
}

impl Drop for OnceInitGuard<'_> {
    fn drop(&mut self) {
        if !self.completed {
            self.state.store(ONCE_EMPTY, Ordering::Release);
        }
    }
}

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
    use super::{OnceCell, SpinLock};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn once_cell_initializes_only_once_across_threads() {
        let cell = Arc::new(OnceCell::new());
        let initializations = Arc::new(AtomicUsize::new(0));
        let mut threads = Vec::new();

        for _ in 0..8 {
            let cell = Arc::clone(&cell);
            let initializations = Arc::clone(&initializations);
            threads.push(std::thread::spawn(move || {
                *cell.get_or_init(|| {
                    initializations.fetch_add(1, Ordering::Relaxed);
                    42
                })
            }));
        }

        for thread in threads {
            assert_eq!(thread.join().expect("worker thread should complete"), 42);
        }
        assert_eq!(initializations.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn once_cell_can_retry_after_initializer_panics() {
        let cell = OnceCell::<u8>::new();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            cell.get_or_init(|| panic!("test initializer failure"));
        }));

        assert!(result.is_err());
        assert_eq!(cell.get(), None);
        assert_eq!(*cell.get_or_init(|| 7), 7);
    }

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
