use crossbeam_queue::ArrayQueue;
use std::sync::Arc;
use std::thread;

fn main() {
    // 创建一个容量为 5 的有界队列
    let queue = Arc::new(ArrayQueue::new(5));
    let mut handles = vec![];

    // 生产者线程
    for i in 0..3 {
        let q = Arc::clone(&queue);
        let handle = thread::spawn(move || {
            for j in 0..10 {
                // 尝试推送数据，如果队列满则返回Err
                if let Err(e) = q.push(i * 10 + j) {
                    println!("队列已满，推送失败: {:?}", e);
                }
            }
        });
        handles.push(handle);
    }

    // 消费者线程
    let consumer = thread::spawn(move || {
        for _ in 0..30 {
            if let Some(value) = queue.pop() {
                println!("消费到: {}", value);
            }
        }
    });

    for handle in handles {
        handle.join().unwrap();
    }
    consumer.join().unwrap();
}