#![allow(dead_code)]

use std::{
    alloc::{alloc, Layout},
    ptr::NonNull,
};

// Example: A simple buffer type that owns some memory
struct Buffer {
    ptr: NonNull<u8>,
    len: usize,
    capacity: usize,
}

impl Buffer {
    // UNSAFE FUNCTION EXAMPLE
    // This is marked unsafe because the CALLER must guarantee something
    pub unsafe fn write_byte_at_unchecked(&mut self, index: usize, byte: u8) {
        // Even in unsafe fn, we need unsafe blocks for unsafe operations
        // But calling this function requires the caller to verify index < self.capacity
        unsafe {
            *self.ptr.as_ptr().add(index) = byte;
        }
        self.len = std::cmp::max(self.len, index + 1);
    }

    // SAFE FUNCTION WITH UNSAFE BLOCK EXAMPLE
    // This is safe to call - no preconditions for the caller
    pub fn write_byte_at(&mut self, index: usize, byte: u8) -> Result<(), &'static str> {
        if index >= self.capacity {
            return Err("Index out of bounds");
        }

        // WE (the function author) take responsibility for safety here
        unsafe {
            // We guarantee this is safe because we checked bounds above
            *self.ptr.as_ptr().add(index) = byte;
        }
        self.len = std::cmp::max(self.len, index + 1);
        Ok(())
    }

    // Another UNSAFE FUNCTION - caller must guarantee buffer outlives returned slice
    pub unsafe fn as_slice_unchecked(&self) -> &[u8] {
        // Again, function body has safe operations, but calling requires care
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    // SAFE FUNCTION that encapsulates the unsafe operation
    pub fn as_slice(&self) -> &[u8] {
        // WE guarantee this is safe because we own the buffer and it's valid
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

// Usage examples showing the difference:

fn demonstrate_difference() {
    let mut buffer = Buffer::new(10); // assume this exists and creates valid buffer

    // === USING UNSAFE FUNCTIONS ===
    // The CALLER must verify safety at each call site

    unsafe {
        // I (the caller) promise that index 5 < buffer.capacity
        buffer.write_byte_at_unchecked(5, 42);

        // I (the caller) promise this buffer will outlive the returned slice
        let slice = buffer.as_slice_unchecked();
        println!("Byte written: {}", slice[5]);
    } // slice goes out of scope here, so our promise is kept

    // === USING SAFE FUNCTIONS ===
    // No promises needed from caller - the function handles safety internally

    match buffer.write_byte_at(5, 42) {
        Ok(()) => println!("Write successful"),
        Err(e) => println!("Write failed: {}", e),
    }

    let slice = buffer.as_slice(); // No unsafe block needed!
    println!("Byte written: {}", slice[5]);
}

// BAD USAGE - This would be undefined behavior!
fn bad_usage_example() {
    let mut buffer = Buffer::new(10);

    unsafe {
        // WRONG: Caller failed to uphold the safety contract!
        // We're writing at index 15 but buffer capacity is only 10
        buffer.write_byte_at_unchecked(15, 42); // 💥 Undefined behavior!
    }

    // The safe version would catch this error:
    match buffer.write_byte_at(15, 42) {
        Ok(()) => println!("Write successful"),
        Err(e) => println!("Caught error: {}", e), // "Index out of bounds"
    }
}

impl Buffer {
    fn new(capacity: usize) -> Self {
        if capacity == 0 {
            panic!("Cannot allocate for capacity of 0");
        }
        let layout = Layout::array::<u8>(capacity).expect("invalid layout");
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            panic!("Failed to allocate memory");
        }
        let non_null_ptr = unsafe { NonNull::new_unchecked(ptr) };
        Buffer {
            ptr: non_null_ptr,
            len: 0,
            capacity,
        }
    }
}

fn main() {
    demonstrate_difference();
}
