#![allow(dead_code)]

use std::sync::{Arc, Mutex};
use std_semaphore::Semaphore;

const CAPACITY: usize = 5;

struct BoundedQueue<T> {
    buffer: [T; CAPACITY],
    producer: usize,
    consumer: usize,
}

impl<T> BoundedQueue<T>
where
    T: Copy + Default,
{
    pub fn new() -> Self {
        Self {
            buffer: [T::default(); CAPACITY],
            producer: 0,
            consumer: 0,
        }
    }

    pub fn put(&mut self, value: T) {
        self.buffer[self.producer] = value;
        self.producer = (self.producer + 1) % CAPACITY;
    }

    pub fn get(&mut self) -> T {
        let value = self.buffer[self.consumer];
        self.consumer = (self.consumer + 1) % CAPACITY;
        value
    }
}

struct SharedQueue<T> {
    queue: Mutex<BoundedQueue<T>>,
    producer: Semaphore,
    consumer: Semaphore,
}

impl<T> SharedQueue<T>
where
    T: Copy + Default,
{
    fn new() -> Self {
        Self {
            queue: Mutex::new(BoundedQueue::new()),
            producer: Semaphore::new(CAPACITY as isize),
            consumer: Semaphore::new(0),
        }
    }
}

fn producer(shared_queue: Arc<SharedQueue<i32>>, loops: usize) {
    for i in 0..loops {
        shared_queue.producer.acquire();
        shared_queue.queue.lock().unwrap().put(i as i32);
        shared_queue.consumer.release();
        println!("Produced: {}", i);
    }
}

fn consumer(shared_queue: Arc<SharedQueue<i32>>, loops: usize) {
    for _ in 0..loops {
        shared_queue.consumer.acquire();
        let value = shared_queue.queue.lock().unwrap().get();
        shared_queue.producer.release();
        println!("Consumed: {}", value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounded_queue() {
        let shared_queue = Arc::new(SharedQueue::<i32>::new());
        let producer_queue_1 = shared_queue.clone();
        let producer_queue_2 = shared_queue.clone();
        let consumer_queue = shared_queue.clone();

        let loops = 100;

        let producer_handle_1 = std::thread::spawn(move || {
            producer(producer_queue_1, loops);
        });
        let producer_handle_2 = std::thread::spawn(move || {
            producer(producer_queue_2, loops);
        });
        let consumer_handle = std::thread::spawn(move || {
            consumer(consumer_queue, loops * 2);
        });

        producer_handle_1.join().unwrap();
        producer_handle_2.join().unwrap();
        consumer_handle.join().unwrap();
    }
}
