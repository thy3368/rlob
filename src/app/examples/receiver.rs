// receiver.rs
use std::net::{Ipv4Addr, UdpSocket};

fn main() -> std::io::Result<()> {
    // 1. 创建Socket并绑定到所有网络接口的本地端口
    // 注意：端口需要与发送端的目标端口一致
    let socket = UdpSocket::bind("0.0.0.0:8888")?;
    println!("接收端已启动，监听端口 8888...");

    // 2. 定义组播地址和网络接口
    let multicast_addr = Ipv4Addr::new(234, 2, 2, 2); // D类组播地址
    let interface_addr = Ipv4Addr::new(0, 0, 0, 0); // 使用默认网络接口

    // 3. 关键步骤：加入组播组
    socket.join_multicast_v4(&multicast_addr, &interface_addr)?;
    println!("已加入组播组: {}", multicast_addr);

    // 4. 准备缓冲区并循环接收数据
    let mut buf = [0u8; 1024];
    loop {
        // `recv_from` 会阻塞直到收到数据
        let (amt, src) = socket.recv_from(&mut buf)?;
        // 将接收到的字节转换为字符串（UTF-8）
        let received_data = String::from_utf8_lossy(&buf[..amt]);
        println!("收到来自 {} 的消息: {}", src, received_data);
    }

    // 5. (可选) 程序退出前离开组播组
    // socket.leave_multicast_v4(&multicast_addr, &interface_addr)?;
}
