use crossbeam_queue::SegQueue;
use std::sync::Arc;
use std::thread;

fn main() {
    let queue = Arc::new(SegQueue::new());
    let mut handles = vec![];

    // 生产者
    for i in 0..2 {
        let q = Arc::clone(&queue);
        handles.push(thread::spawn(move || {
            for j in 0..5 {
                q.push(i * 5 + j);
                println!("生产者 {} 推送了 {}", i, i * 5 + j);
            }
        }));
    }

    // 消费者
    let q_clone = Arc::clone(&queue);
    handles.push(thread::spawn(move || {
        for _ in 0..10 {
            if let Some(value) = q_clone.pop() {
                println!("消费者处理: {}", value);
            }
        }
    }));

    for handle in handles {
        handle.join().unwrap();
    }
}