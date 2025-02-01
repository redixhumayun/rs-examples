#![allow(dead_code)]

use std::{cell::UnsafeCell, mem::MaybeUninit, sync::atomic::AtomicBool};

pub struct Channel<T> {
    ready: AtomicBool,
    message: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for Channel<T> {}

impl<T> Channel<T> {
    fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
            message: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    fn send(&self, message: T) {
        //  if there is already a message in the channel, panic
        if self.ready.swap(true, std::sync::atomic::Ordering::Acquire) {
            panic!("cannot send more than one message in a channel");
        }
        //  store the message in the channel and set the flag
        unsafe { (*self.message.get()).write(message) };
    }

    fn receive(&self) -> T {
        //  if the ready flag is not set, panic
        if !self.ready.swap(false, std::sync::atomic::Ordering::Acquire) {
            panic!("there is either no message stored in the channel or the message has already been read");
        }
        unsafe { (*self.message.get()).assume_init_read() }
    }

    fn is_ready(&self) -> bool {
        self.ready.load(std::sync::atomic::Ordering::Acquire)
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::*;

    #[test]
    fn test_channel() {
        let channel = Channel::new();
        channel.send(42);
        assert_eq!(channel.receive(), 42);
    }

    #[test]
    fn test_channel_threads() {
        let channel = Channel::new();
        let thread = thread::current();
        thread::scope(|s| {
            thread::Builder::new()
                .name("SenderThread".to_string())
                .spawn_scoped(s, || {
                    channel.send(42);
                    thread.unpark();
                })
                .unwrap();
            while !channel.is_ready() {
                thread::park();
            }
            assert_eq!(channel.receive(), 42);
        });
    }
}
