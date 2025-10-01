#![allow(dead_code)]

use std::{
    ops::Deref,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

struct ArcData<T> {
    ref_count: AtomicUsize,
    data: T,
}

struct SafeArc<T> {
    ptr: NonNull<ArcData<T>>,
}

impl<T> SafeArc<T> {
    fn new(data: T) -> SafeArc<T> {
        let data = ArcData {
            ref_count: AtomicUsize::new(1),
            data,
        };
        let boxed_data = Box::new(data);
        let raw_ptr = Box::into_raw(boxed_data);
        let ptr = NonNull::new(raw_ptr).expect("cannot be null");
        SafeArc { ptr }
    }

    fn data(&self) -> &ArcData<T> {
        unsafe { self.ptr.as_ref() }
    }

    fn get_mut(arc: &mut Self) -> Option<&mut T> {
        if arc
            .data()
            .ref_count
            .load(std::sync::atomic::Ordering::Relaxed)
            == 1
        {
            unsafe {
                return Some(&mut arc.ptr.as_mut().data);
            }
        } else {
            None
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
        if self.data().ref_count.load(Ordering::Relaxed) == usize::MAX {
            std::process::abort();
        }
        self.data().ref_count.fetch_add(1, Ordering::Relaxed);
        SafeArc { ptr: self.ptr }
    }
}

impl<T> Drop for SafeArc<T> {
    fn drop(&mut self) {
        if self.data().ref_count.fetch_sub(1, Ordering::AcqRel) == 1 {
            let boxed = unsafe { Box::from_raw(self.ptr.as_ptr()) };
            drop(boxed);
        }
    }
}

unsafe impl<T: Send> Send for SafeArc<T> {}

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
    fn arc_multithreaded_test() {
        let arc_1 = SafeArc::new(42);
        let thread = thread::spawn(move || {
            assert_eq!(*arc_1, 42);
        });
        thread.join().unwrap();
    }
}
