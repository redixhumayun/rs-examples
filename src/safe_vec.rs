#![allow(dead_code)]

use std::{
    alloc::{alloc, dealloc, Layout},
    mem::MaybeUninit,
};

struct SafeVec<T> {
    ptr: *mut MaybeUninit<T>,
    len: usize,
    capacity: usize,
}

impl<T> SafeVec<T> {
    fn new(capacity: usize) -> Self {
        let layout = Layout::array::<MaybeUninit<T>>(capacity).expect(&format!("invalid layout"));
        let ptr = unsafe { alloc(layout) as *mut MaybeUninit<T> };
        if ptr.is_null() {
            panic!("unable to allocate {capacity} for SafeVec")
        }
        SafeVec {
            ptr,
            len: 0,
            capacity,
        }
    }

    fn reallocate(&mut self) {
        let old_ptr = self.ptr;
        let old_capacity = self.capacity;
        let old_layout = Layout::array::<MaybeUninit<T>>(old_capacity).expect("invalid layout");

        self.capacity = self.capacity.saturating_mul(2);
        let layout = Layout::array::<MaybeUninit<T>>(self.capacity).expect("invalid layout");
        let ptr = unsafe { alloc(layout) as *mut MaybeUninit<T> };
        if ptr.is_null() {
            panic!("unable to allocate {0} for SafeVec", self.capacity);
        }

        for i in 0..self.len {
            unsafe {
                let value = self.ptr.add(i).read();
                ptr.add(i).write(value);
            }
        }

        self.ptr = ptr;
        unsafe {
            dealloc(old_ptr as *mut u8, old_layout);
        };
    }

    fn push(&mut self, elem: T) {
        if self.len == self.capacity {
            self.reallocate();
        }
        unsafe {
            self.ptr.add(self.len).write(MaybeUninit::new(elem));
        }
        self.len += 1;
    }

    fn pop(&mut self) -> T {
        if self.len == 0 {
            panic!("cannot pop from an empty vector");
        }
        self.len -= 1;
        unsafe { self.ptr.add(self.len).read().assume_init() }
    }

    fn get(&self, index: usize) -> &T {
        if self.len == 0 {
            panic!("attempt to read from an empty vector");
        }
        if index >= self.len {
            panic!("attempt to read beyond the length of the vector");
        }
        unsafe { (&*self.ptr.add(index)).assume_init_ref() }
    }
}

impl<T> Drop for SafeVec<T> {
    fn drop(&mut self) {
        for i in 0..self.len {
            unsafe { (&mut *self.ptr.add(i)).assume_init_drop() };
        }
        let layout = Layout::array::<MaybeUninit<T>>(self.capacity).expect("invalid layout");
        unsafe {
            dealloc(self.ptr as *mut u8, layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::safe_vec::SafeVec;

    #[test]
    fn test_basic_safe_vec() {
        let mut vec: SafeVec<usize> = SafeVec::new(10);
        vec.push(1);
        vec.push(2);
        vec.push(3);
        let elem = vec.get(0);
        assert_eq!(*elem, 1);
        assert_eq!(vec.len, 3);

        let elem = vec.pop();
        assert_eq!(elem, 3);
        assert_eq!(vec.len, 2);
    }

    #[test]
    fn test_reallocation_preserves_elements() {
        let mut vec: SafeVec<usize> = SafeVec::new(10);
        for i in 0..25 {
            vec.push(i);
        }
        assert_eq!(vec.len, 25);
        for i in 0..25 {
            assert_eq!(*vec.get(i), i);
        }
        for expected in (0..25).rev() {
            assert_eq!(vec.pop(), expected);
        }
        assert_eq!(vec.len, 0);
    }

    #[test]
    fn test_memory_leak() {
        let mut vec: SafeVec<String> = SafeVec::new(2);
        vec.push("hello".to_string());
        vec.push("world".to_string());
        vec.push("leak".to_string()); // This should trigger reallocation
    }
}
