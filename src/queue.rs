use std::sync::{Arc, Mutex, Condvar};
use std::collections::VecDeque;

struct QueueData<T> {
    items: Mutex<VecDeque<T>>,
    nonempty: Condvar,
    nonfull: Condvar
}

impl<T> QueueData<T> {
    fn with_capacity(count: usize) -> QueueData<T> {
        QueueData {
            items: Mutex::new(VecDeque::with_capacity(count)),
            nonempty: Condvar::new(),
            nonfull: Condvar::new()
        }
    }
}

/// A multi-producer, multi-consumer queue, for use in dispatching tasks
/// to/from multiple threads.
///
/// Each `Queue` has a fixed capacity, chosen at creation time. If a queue is
/// full, `queue.put(value)` blocks until space is available.  Unix pipes are
/// like this. With this design, a simple "pushback" mechanism is built in.
///
/// Suppose one thread in a pipeline is a bottleneck, and it needs more CPU
/// time to keep up with upstream and downstream threads. If you use lock-free
/// queues to connect the threads in this pipeline, data will pile up in the queue,
/// and downstream consumers have no efficient way to wait for the next item.
/// With `Queue`, upstream producers simply go to sleep in `put()` and downstream
/// producers go to sleep in `take()`, and a thread is awakened when it can do
/// useful work.
#[derive(Clone)]
pub struct Queue<T> {
    data: Arc<QueueData<T>>
}

impl<T> Queue<T> {
    /// Creates an empty `Queue` with space for at most `count` elements.
    ///
    /// The new `Queue`'s capacity is fixed.  `Queue`s do not grow, because we
    /// want `put()` and `take()` to be infallible.
    pub fn with_capacity(count: usize) -> Queue<T> {
        assert!(count > 0);
        Queue {
            data: Arc::new(QueueData::with_capacity(count))
        }
    }
    
    /// Add an item to the back of the queue.
    ///
    /// If the queue is full, this blocks until another thread calls `take()`,
    /// which will free up enough space to add `value`.
    pub fn put(&self, value: T) {
        let data = &*self.data;
        let mut guard = data.items.lock().unwrap();

        // Wait for the queue to be nonfull.
        while guard.len() == guard.capacity() {
            guard = data.nonfull.wait(guard).unwrap();
        }

        guard.push_back(value);
        data.nonempty.notify_one();
    }

    /// Remove an item from the front of the queue.
    ///
    /// If the queue is empty, this blocks until another thread calls `put()`,
    /// so that there's an item to remove.
    pub fn take(&self) -> T {
        let data = &*self.data;
        let mut guard = data.items.lock().unwrap();

        // Wait for the queue to be nonempty.
        while guard.is_empty() {
            guard = data.nonempty.wait(guard).unwrap();
        }

        data.nonfull.notify_one();
        guard.pop_front().unwrap()
    }
}
