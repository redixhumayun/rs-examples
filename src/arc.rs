#![allow(dead_code)]

use std::{
    mem::ManuallyDrop,
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
    usize,
};

struct ArcData<T> {
    strong: AtomicUsize,
    weak: AtomicUsize,
    data: ManuallyDrop<T>,
}

struct Weak<T> {
    ptr: NonNull<ArcData<T>>,
}

impl<T> Weak<T> {
    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    pub fn upgrade(&self) -> Option<SafeArc<T>> {
        let mut strong_count = self.data().strong.load(Ordering::Relaxed);
        loop {
            if strong_count == 0 {
                return None;
            }
            if let Err(e) = self.data().strong.compare_exchange_weak(
                strong_count,
                strong_count + 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                strong_count = e;
                continue;
            };
            return Some(SafeArc { ptr: self.ptr });
        }
    }
}

impl<T> Clone for Weak<T> {
    fn clone(&self) -> Self {
        self.data().weak.fetch_add(1, Ordering::Relaxed);
        Self {
            ptr: self.ptr.clone(),
        }
    }
}

impl<T> Drop for Weak<T> {
    fn drop(&mut self) {
        if self.data().weak.fetch_sub(1, Ordering::AcqRel) == 1 {
            let boxed = unsafe { Box::from_raw(self.ptr.as_ptr()) };
            drop(boxed);
        }
    }
}

struct SafeArc<T> {
    ptr: NonNull<ArcData<T>>,
}

impl<T> SafeArc<T> {
    fn new(data: T) -> SafeArc<T> {
        SafeArc {
            ptr: NonNull::new(Box::into_raw(Box::new(ArcData {
                strong: AtomicUsize::new(1),
                weak: AtomicUsize::new(1),
                data: ManuallyDrop::new(data),
            })))
            .expect("cannot be null"),
        }
    }

    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if let Err(_) =
            arc.data()
                .weak
                .compare_exchange(1, usize::MAX, Ordering::AcqRel, Ordering::Relaxed)
        {
            return None;
        }
        let is_unique = arc.data().strong.load(Ordering::Relaxed) == 1;
        arc.data().weak.store(1, Ordering::Release);
        if is_unique {
            unsafe { return Some(&mut arc.ptr.as_mut().data) }
        } else {
            return None;
        }
    }

    pub fn downgrade(arc: &mut Self) -> Weak<T> {
        let mut n = arc.data().weak.load(Ordering::Acquire);
        loop {
            if n == usize::MAX {
                std::hint::spin_loop();
                n = arc.data().weak.load(Ordering::Acquire);
                continue;
            }
            if let Err(e) =
                arc.data()
                    .weak
                    .compare_exchange(n, n + 1, Ordering::Relaxed, Ordering::Relaxed)
            {
                n = e;
                continue;
            }
            return Weak { ptr: arc.ptr };
        }
    }
}

impl<T> Deref for SafeArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data().data
    }
}

impl<T> Clone for SafeArc<T> {
    fn clone(&self) -> Self {
        if self.data().strong.load(Ordering::Relaxed) > usize::MAX / 2 {
            std::process::abort();
        }
        self.data().strong.fetch_add(1, Ordering::Relaxed);
        Self { ptr: self.ptr }
    }
}

impl<T> Drop for SafeArc<T> {
    fn drop(&mut self) {
        if self.data().strong.fetch_sub(1, Ordering::AcqRel) == 1 {
            unsafe {
                ManuallyDrop::drop(&mut self.ptr.as_mut().data);
            }
            drop(Weak { ptr: self.ptr });
        }
    }
}

unsafe impl<T: Send> Send for SafeArc<T> {}
unsafe impl<T: Sync> Sync for SafeArc<T> {}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::arc::SafeArc;

    #[test]
    fn arc_basic_test() {
        let arc_1 = SafeArc::new(42);
        let arc_2 = SafeArc::clone(&arc_1);
        drop(arc_1);
        assert_eq!(*arc_2, 42);
    }

    #[test]
    fn arc_send_test() {
        let arc_1 = SafeArc::new(42);
        let thread = thread::spawn(move || {
            assert_eq!(*arc_1, 42);
        });
        thread.join().unwrap();
    }

    #[test]
    fn arc_sync_test() {
        let safe_arc = SafeArc::new(42);

        std::thread::scope(|s| {
            let mut handles = Vec::new();
            for _ in 0..4 {
                let handle = s.spawn(|| {
                    assert_eq!(*safe_arc, 42);
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }
        });
    }

    #[test]
    fn arc_full_test() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        static DROP_COUNTER: AtomicUsize = AtomicUsize::new(0);

        struct DropCounter;

        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROP_COUNTER.fetch_add(1, Ordering::SeqCst);
            }
        }

        DROP_COUNTER.store(0, Ordering::SeqCst);

        let arc1 = SafeArc::new(DropCounter);
        let arc2 = arc1.clone();
        let arc3 = arc1.clone();
        let arc4 = arc2.clone();

        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 0);

        drop(arc1);
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 0);

        drop(arc2);
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 0);

        drop(arc3);
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 0);

        drop(arc4);
        assert_eq!(DROP_COUNTER.load(Ordering::SeqCst), 1);
    }
}
