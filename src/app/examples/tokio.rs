// Cargo.toml 依赖: tokio = { version = "1.0", features = ["full"] }

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main] // 此宏将主函数转换为Tokio异步运行时入口
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 绑定监听地址
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("服务器运行在 127.0.0.1:8080");

    loop {
        // 异步接受新连接。await点不会阻塞线程，而是让出CPU给其他任务
        let (mut socket, addr) = listener.accept().await?;
        println!("接收到新连接: {}", addr);

        // 为每个连接生成一个独立的异步任务（轻量级，非OS线程）
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                // 异步读取数据
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        // 返回0字节表示连接已关闭
                        println!("连接 {} 已关闭", addr);
                        return;
                    }
                    Ok(n) => {
                        // 将接收到的数据原样写回（回显）
                        if let Err(e) = socket.write_all(&buf[..n]).await {
                            eprintln!("向 {} 写入数据失败: {}", addr, e);
                            return;
                        }
                    }
                    Err(e) => {
                        eprintln!("从 {} 读取数据失败: {}", addr, e);
                        return;
                    }
                }
            }
        });
    }
}
