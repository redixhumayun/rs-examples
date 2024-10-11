mod mutex;
use std::pin::Pin;

use mutex::SpinLock;

mod channel;
mod channel_split;

fn run_mutex_example() {
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

fn main() {
    // run_mutex_example();
    #[derive(Debug)]
    struct MyData {
        value: String,
    }

    let my_data = Pin::new(Box::new(MyData {
        value: String::from("hello"),
    }));
    // let my_data_ptr: *const MyData = &*my_data; // Pointer to data

    // Moving the box
    let moved_data = Box::new(MyData {
        value: my_data.value,
    });
    println!("Moved data value: {}", moved_data.value);
}
