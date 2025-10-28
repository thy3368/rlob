/// TCP单播回显示例
///
/// 演示如何使用TCP客户端和服务器进行消息收发
/// 服务器会将接收到的消息回显给客户端


use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::time::sleep;
use lib::unicase::domain::unicase::{MessageType, ReconnectConfig, TcpClient, TcpConfig, TcpServer, UnicastMessage};
use lib::unicase::outbound::tcp_client::TcpUnicastClient;
use lib::unicase::outbound::tcp_server::TcpUnicastServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TCP单播回显示例 ===\n");

    // 启动服务器
    let server_addr = "127.0.0.1:9090".parse().unwrap();
    let mut server = TcpUnicastServer::new(server_addr);

    println!("1. 启动TCP服务器: {}", server_addr);
    server.start().await?;

    // 等待服务器启动
    sleep(Duration::from_millis(100)).await;

    // 创建客户端配置
    let config = TcpConfig {
        server_addr,
        connect_timeout: Duration::from_secs(5),
        read_timeout: Some(Duration::from_secs(10)),
        write_timeout: Some(Duration::from_secs(10)),
        nodelay: true,
        recv_buffer_size: Some(64 * 1024),
        send_buffer_size: Some(64 * 1024),
        keepalive: Some(Duration::from_secs(60)),
        reconnect: ReconnectConfig {
            enabled: true,
            max_attempts: Some(3),
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
        },
    };

    // 创建并连接客户端
    let mut client = TcpUnicastClient::new(config);
    println!("\n2. 创建TCP客户端");
    println!("3. 连接到服务器...");

    client.connect().await?;
    println!("   ✓ 连接成功!");

    // 发送测试消息
    println!("\n4. 发送测试消息:");

    for i in 1..=5 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let payload = format!("Hello from client! Message #{}", i).into_bytes();

        let message = UnicastMessage {
            message_id: i,
            timestamp_ns: timestamp,
            msg_type: MessageType::OrderCommand,
            payload,
        };

        println!("   发送消息 #{}: {:?}", i, String::from_utf8_lossy(&message.payload));
        client.send(&message).await?;

        sleep(Duration::from_millis(500)).await;
    }

    // 显示统计信息
    println!("\n5. 客户端统计:");
    let stats = client.stats();
    println!("   - 发送消息数: {}", stats.messages_sent);
    println!("   - 接收消息数: {}", stats.messages_received);
    println!("   - 发送字节数: {}", stats.bytes_sent);
    println!("   - 接收字节数: {}", stats.bytes_received);
    println!("   - 连接次数: {}", stats.connect_count);
    println!("   - 重连次数: {}", stats.reconnect_count);

    println!("\n6. 服务器统计:");
    let server_stats = server.stats();
    println!("   - 活跃连接数: {}", server_stats.active_connections);
    println!("   - 累计连接数: {}", server_stats.total_connections);
    println!("   - 发送消息数: {}", server_stats.messages_sent);
    println!("   - 接收消息数: {}", server_stats.messages_received);

    // 断开连接
    println!("\n7. 断开客户端连接");
    client.disconnect().await?;

    // 停止服务器
    println!("8. 停止服务器");
    server.stop().await?;

    println!("\n=== 示例完成 ===");

    Ok(())
}
