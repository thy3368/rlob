use std::sync::Arc;
use std::thread;

fn main() {
    // 创建 Arc
    let shared_data = Arc::new(100);
    println!(
        "主线程: 初始强引用计数 = {}",
        Arc::strong_count(&shared_data)
    );

    let mut handles = vec![];

    for i in 0..3 {
        // 为每个线程克隆 Arc
        let data_clone = Arc::clone(&shared_data);
        let handle = thread::spawn(move || {
            // 在每个线程中打印当前的引用计数
            println!(
                "线程 {}: 强引用计数 = {}",
                i,
                Arc::strong_count(&data_clone)
            );
        });
        handles.push(handle);
    }

    // 等待所有线程结束
    for handle in handles {
        handle.join().unwrap();
    }

    println!(
        "所有线程结束后，强引用计数 = {}",
        Arc::strong_count(&shared_data)
    );
}
