// Cargo.toml 依赖: mio = "1.1.0"
// 注意: 此示例使用 mio 跨平台 I/O 库，可在 Linux、macOS 等平台运行

use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self, Read};

const SERVER: Token = Token(0);

fn main() -> io::Result<()> {
    // 创建事件轮询器 (Poll)
    let mut poll = Poll::new()?;
    // 创建事件存储容器
    let mut events = Events::with_capacity(128);

    // 绑定地址并创建监听器
    let addr = "127.0.0.1:8080".parse().unwrap();
    let mut server = TcpListener::bind(addr)?;

    // 将服务器监听器注册到 poll，关注可读事件（新连接）
    poll.registry()
        .register(&mut server, SERVER, Interest::READABLE)?;

    // 用于保存已建立的连接
    let mut connections = HashMap::new();
    let mut unique_token = Token(SERVER.0 + 1); // 从下一个 Token 开始分配

    // 事件循环
    loop {
        // 等待事件发生
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    // 接受新连接
                    loop {
                        match server.accept() {
                            Ok((mut stream, address)) => {
                                println!("新客户端: {}", address);
                                let token = unique_token;
                                unique_token.0 += 1;

                                // 注册新连接到 poll，关注可读事件
                                poll.registry()
                                    .register(&mut stream, token, Interest::READABLE)?;
                                connections.insert(token, stream);
                            }
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                            Err(e) => return Err(e),
                        }
                    }
                }
                token => {
                    // 处理客户端发送的数据
                    if let Some(stream) = connections.get_mut(&token) {
                        // 这里应读取数据。示例中简单打印并关闭连接。
                        let mut buf = [0; 512];
                        match stream.read(&mut buf) {
                            Ok(0) => {
                                // 连接已关闭
                                println!("客户端关闭连接");
                                connections.remove(&token);
                            }
                            Ok(n) => {
                                println!("收到数据: {} 字节", n);
                                // 示例：原样回显数据
                                // stream.write_all(&buf[..n]).unwrap();
                            }
                            Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
        }
    }
}
