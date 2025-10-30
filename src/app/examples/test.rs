use crossbeam::channel;
use std::thread;
use std::time::Duration;

fn main() {
    // 创建一个有界通道，缓冲区大小为3
    let (sender, receiver) = channel::bounded(3);

    // 创建多个生产者
    for producer_id in 0..3 {
        let s = sender.clone();
        thread::spawn(move || {
            for i in 0..5 {
                // try_send 不会阻塞，若缓冲区满则返回错误
                // 使用 send 则会阻塞直到有空位
                if let Err(e) = s.try_send(format!("[生产者{}] 批次 {}", producer_id, i)) {
                    println!("发送失败（缓冲区可能已满）: {}", e);
                } else {
                    println!("[生产者{}] 已发送消息 {}", producer_id, i);
                }
                thread::sleep(Duration::from_millis(200));
            }
            println!("[生产者{}] 任务完成。", producer_id);
        });
    }

    // 创建多个消费者
    for consumer_id in 0..2 {
        let r = receiver.clone(); // crossbeam的接收端也可以克隆，实现多消费者！
        thread::spawn(move || {
            loop {
                // 使用 recv 阻塞接收，也可以使用 try_recv 非阻塞接收
                match r.recv() {
                    Ok(msg) => println!("[消费者{}] 收到: {}", consumer_id, msg),
                    Err(_) => {
                        println!("[消费者{}] 检测到所有发送端已关闭，退出。", consumer_id);
                        break; // 所有发送端关闭，消费者线程退出循环
                    }
                }
            }
        });
    }

    // 等待生产者完成，然后drop原始的发送端
    thread::sleep(Duration::from_secs(3));
    drop(sender); // 丢弃所有发送端后，接收端的recv会返回错误

    // 等待消费者线程自然退出
    thread::sleep(Duration::from_secs(1));
    println!("主程序结束。");
}