// sender.rs
use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct MessageRepo {
    send_socket: UdpSocket,
    send_addr: String,
}

impl MessageRepo {
    pub fn new() -> MessageRepo {
        let send_socket = UdpSocket::bind("0.0.0.0:0");
        MessageRepo {
            send_socket: send_socket.expect("REASON"),
            send_addr: "234.2.2.2:8888".to_string(),
        }
    }

    pub fn send(&self, message: String) {
        let handle = thread::spawn(|| {
            for i in 1..=5 {
                println!("子线程打印: {}", i);
                thread::sleep(Duration::from_millis(100));
            }
        });

        // 等待子线程结束
        handle.join().unwrap();
        println!("所有线程执行完毕。");

        self.send_socket
            .send_to(message.as_bytes(), &self.send_addr);
        println!("已发送: {}", message);
    }
}
fn main() -> std::io::Result<()> {
    let repo = MessageRepo::new();

    for i in 1..=5 {
        let message = format!("你好！这是第 {} 条组播消息", i);
        repo.send(message.clone());

        // 间隔1秒
        thread::sleep(Duration::from_secs(1));
    }

    Ok(())
}
