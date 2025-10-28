/// TCP自动重连演示
///
/// 演示TCP客户端的自动重连功能
/// 当服务器断开连接后,客户端会自动尝试重连

use std::time::{SystemTime, UNIX_EPOCH, Duration};
use tokio::time::sleep;
use lib::unicase::domain::unicase::{MessageType, ReconnectConfig, TcpClient, TcpConfig, TcpServer, UnicastMessage};
use lib::unicase::outbound::tcp_client::TcpUnicastClient;
use lib::unicase::outbound::tcp_server::TcpUnicastServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TCP自动重连演示 ===\n");

    // 服务器地址
    let server_addr = "127.0.0.1:9091".parse().unwrap();

    // 配置客户端（启用自动重连）
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
            max_attempts: Some(10), // 最多重连10次
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(5),
            backoff_multiplier: 1.5,
        },
    };

    println!("1. 创建TCP客户端（启用自动重连）");
    let mut client = TcpUnicastClient::new(config);

    // 启动服务器
    println!("2. 启动TCP服务器: {}", server_addr);
    let mut server = TcpUnicastServer::new(server_addr);
    server.start().await?;

    sleep(Duration::from_millis(100)).await;

    // 连接到服务器
    println!("3. 客户端连接到服务器");
    client.connect().await?;
    println!("   ✓ 连接成功!");

    // 发送第一条消息
    println!("\n4. 发送第一条消息");
    let message1 = create_message(1, "Before disconnect");
    client.send(&message1).await?;
    println!("   ✓ 消息已发送");

    // 模拟服务器断开
    println!("\n5. 模拟服务器断开连接");
    server.stop().await?;
    println!("   ✓ 服务器已停止");

    sleep(Duration::from_secs(2)).await;

    // 重启服务器
    println!("\n6. 重启服务器");
    let mut server = TcpUnicastServer::new(server_addr);
    server.start().await?;
    println!("   ✓ 服务器已重启");

    sleep(Duration::from_millis(500)).await;

    // 客户端会自动重连并发送消息
    println!("\n7. 客户端自动重连后发送消息");
    let message2 = create_message(2, "After reconnect");

    match client.send(&message2).await {
        Ok(_) => println!("   ✓ 消息已发送（客户端已自动重连）"),
        Err(e) => println!("   ✗ 发送失败: {}", e),
    }

    // 显示统计信息
    println!("\n8. 客户端统计:");
    let stats = client.stats();
    println!("   - 连接次数: {}", stats.connect_count);
    println!("   - 重连次数: {}", stats.reconnect_count);
    println!("   - 发送消息数: {}", stats.messages_sent);
    println!("   - 发送字节数: {}", stats.bytes_sent);
    println!("   - 发送错误数: {}", stats.send_errors);

    // 清理
    println!("\n9. 清理资源");
    client.disconnect().await?;
    server.stop().await?;

    println!("\n=== 演示完成 ===");
    println!("\n关键特性:");
    println!("  ✓ 自动检测连接断开");
    println!("  ✓ 指数退避重连策略");
    println!("  ✓ 可配置的最大重连次数");
    println!("  ✓ 对应用层透明的重连过程");

    Ok(())
}

fn create_message(id: u64, text: &str) -> UnicastMessage {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;

    UnicastMessage {
        message_id: id,
        timestamp_ns: timestamp,
        msg_type: MessageType::OrderCommand,
        payload: text.as_bytes().to_vec(),
    }
}
