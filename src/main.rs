use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
    sync::atomic::AtomicBool,
};

struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T> Send for SpinLock<T> where T: Send {}
unsafe impl<T> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    fn lock(&self) -> LockGuard<T> {
        while self
            .locked
            .compare_exchange(
                false,
                true,
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Relaxed,
            )
            .is_err()
        {
            std::hint::spin_loop();
        }
        LockGuard { lock: &self }
    }
}

struct LockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Deref for LockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for LockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for LockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock
            .locked
            .store(false, std::sync::atomic::Ordering::Release);
    }
}

fn main() {
    let spin_lock = SpinLock::new(0);
    std::thread::scope(|s| {
        s.spawn(|| {
            let mut guard = spin_lock.lock();
            *guard = 2;
            println!("thread 1 acquired the spin lock");
            println!("the value is {}", *guard);
        });
        s.spawn(|| {
            let guard = spin_lock.lock();
            println!("thread 2 acquired the spin lock");
            println!("the value is {}", *guard);
        });
    });
}
