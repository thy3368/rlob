/// UDP组播发送器测试程序
///
/// 演示如何使用UDP组播发送市场数据

use lib::domain::multicast::*;
use lib::outbound::udp_publisher::UdpMulticastPublisher;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(70));
    println!("UDP组播发送器测试");
    println!("{}", "=".repeat(70));
    println!();

    // 配置组播参数
    let config = MulticastConfig {
        multicast_addr: "239.255.0.1".parse().unwrap(),
        port: 9000,
        interface: None,
        ttl: 1,
        loopback: true,
    };

    println!("配置:");
    println!("  组播地址: {}", config.multicast_addr);
    println!("  端口: {}", config.port);
    println!("  TTL: {}", config.ttl);
    println!("  环回: {}", config.loopback);
    println!();

    // 创建发送器
    println!("创建UDP组播发送器...");
    let publisher = UdpMulticastPublisher::new(config)?;
    println!("✓ 发送器创建成功");
    println!();

    println!("开始发送测试消息...");
    println!("按 Ctrl+C 停止");
    println!();

    let mut counter = 0u32;

    loop {
        counter += 1;

        // 发送Ticker消息
        let ticker_data = format!("BTCUSDT Price: {:.2}", 95000.0 + (counter as f64 * 0.1));
        publisher
            .send(MessageType::Ticker, ticker_data.as_bytes().to_vec())
            .await?;

        println!("[{}] 发送Ticker: {}", counter, ticker_data);

        // 每5条消息发送一次心跳
        if counter % 5 == 0 {
            let heartbeat_data = format!("Heartbeat #{}", counter / 5);
            publisher
                .send(MessageType::Heartbeat, heartbeat_data.as_bytes().to_vec())
                .await?;

            println!("[{}] 发送心跳: {}", counter, heartbeat_data);
        }

        // 显示统计
        if counter % 10 == 0 {
            let stats = publisher.stats();
            println!();
            println!("统计信息:");
            println!("  发送消息数: {}", stats.messages_sent);
            println!("  发送字节数: {}", stats.bytes_sent);
            println!("  错误数: {}", stats.errors);
            println!();
        }

        // 休眠1秒
        time::sleep(Duration::from_secs(1)).await;
    }
}
