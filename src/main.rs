mod bounded_queue;
mod buffer;
mod channel;
mod channel_split;
mod drop_no_drop;
mod mutex;
mod safe_vec;
mod semaphore;

use mutex::SpinLock;

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
    run_mutex_example();
}
