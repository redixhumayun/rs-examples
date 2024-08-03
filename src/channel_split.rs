use std::{
    cell::UnsafeCell,
    marker::PhantomData,
    mem::MaybeUninit,
    rc::Rc,
    sync::atomic::AtomicBool,
    thread::{self, Thread},
};

pub struct Channel<T> {
    ready: AtomicBool,
    message: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T> Sync for Channel<T> where T: Send {}

impl<T> Channel<T> {
    fn new() -> Self {
        Channel {
            ready: AtomicBool::new(false),
            message: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    fn split<'a>(&'a mut self) -> (Sender<'a, T>, Receiver<'a, T>) {
        *self = Self::new();
        let sender = Sender {
            channel: self,
            recv_thread: thread::current(),
        };
        let receiver = Receiver {
            channel: self,
            _no_send: PhantomData,
        };
        (sender, receiver)
    }
}

impl<T> Drop for Channel<T> {
    fn drop(&mut self) {
        if self.ready.load(std::sync::atomic::Ordering::Acquire) {
            unsafe { self.message.get_mut().assume_init_drop() };
        }
    }
}

pub struct Sender<'a, T> {
    channel: &'a Channel<T>,
    recv_thread: Thread,
}

impl<T> Sender<'_, T> {
    fn send(self, message: T) {
        unsafe { (*self.channel.message.get()).write(message) };
        self.channel
            .ready
            .store(true, std::sync::atomic::Ordering::Release);
        self.recv_thread.unpark();
    }
}

pub struct Receiver<'a, T> {
    channel: &'a Channel<T>,
    _no_send: PhantomData<Rc<()>>,
}

impl<T> Receiver<'_, T> {
    fn is_ready(&self) -> bool {
        self.channel
            .ready
            .load(std::sync::atomic::Ordering::Acquire)
    }

    fn receive(self) -> T {
        while !self
            .channel
            .ready
            .swap(false, std::sync::atomic::Ordering::Acquire)
        {
            thread::park();
        }
        unsafe { (*self.channel.message.get()).assume_init_read() }
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::*;

    #[test]
    fn test_channel() {
        let mut channel = Channel::new();
        let (sender, receiver) = channel.split();
        sender.send(42);
        assert_eq!(receiver.receive(), 42);
    }

    #[test]
    fn test_channel_thread() {
        let mut channel = Channel::new();
        let (sender, receiver) = channel.split();
        thread::scope(|s| {
            s.spawn(|| {
                sender.send(42);
            });
            assert_eq!(receiver.receive(), 42);
        });
    }
}
