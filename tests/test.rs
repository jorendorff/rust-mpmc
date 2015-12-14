extern crate mpmc;
use mpmc::queue::Queue;
use std::thread;

#[test]
fn test_single_thread_use() {
    let q = Queue::with_capacity(10);

    for i in 0..10 {
        q.put(i);
    }

    for j in 10..20 {
        assert_eq!(q.take(), j - 10);
        q.put(j);
    }

    for k in 10..20 {
        assert_eq!(q.take(), k);
    }
}

#[test]
fn test_single_consumer() {
    let q = Queue::with_capacity(10);
    let count = 100;
    
    let reader_thread = {
        let qr = q.clone();
        thread::spawn(move || {
            let mut total = 0;
            for _ in 0..count {
                total += qr.take();
            }
            total
        })
    };

    let mut expected = 0;
    for i in 0..count {
        q.put(i);
        expected += i;
    }

    assert_eq!(reader_thread.join().unwrap(), expected);
}

// to test:
// *   single producer, multiple consumers
// *   multiple producers, single consumer
// *   multiple producers, multiple consumers
// *   queue usually empty, consumers waiting
// *   queue usually full, producers waiting
// *   multiple queues: for scatter-gather
// *   multiple queues: running concurrently
