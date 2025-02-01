#![allow(dead_code)]

use std::sync::{Condvar, Mutex};

struct Semaphore {
    value: Mutex<usize>,
    cond_var: Condvar,
}

impl Semaphore {
    fn new(value: usize) -> Self {
        Self {
            value: Mutex::new(value),
            cond_var: Condvar::new(),
        }
    }

    fn acquire(&self) {
        let mut guard = self.value.lock().unwrap();
        while *guard <= 0 {
            guard = self.cond_var.wait(guard).unwrap();
        }
        *guard -= 1;
    }

    fn release(&self) {
        *self.value.lock().unwrap() += 1;
        self.cond_var.notify_all();
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, thread, time::Duration};

    use super::*;

    #[test]
    fn test_basic_sempahore() {
        let capacity = 5;
        let sem = Arc::new(Semaphore::new(capacity));
        let mut handles = vec![];

        for i in 0..capacity {
            let sem_clone = Arc::clone(&sem);
            handles.push(thread::spawn(move || {
                sem_clone.acquire();
                println!("Thread {} acquired", i);
                thread::sleep(Duration::from_millis(100));
                println!("Thread {} released", i);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
