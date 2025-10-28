/// UDP组播接收器测试程序
///
/// 演示如何使用UDP组播接收市场数据

use lib::domain::multicast::*;
use lib::outbound::udp_subscriber::UdpMulticastSubscriber;
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(70));
    println!("UDP组播接收器测试");
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
    println!();

    // 创建接收器
    println!("创建UDP组播接收器...");
    let subscriber = UdpMulticastSubscriber::new(config)?;
    println!("✓ 接收器创建成功");
    println!();

    println!("开始接收消息...");
    println!("按 Ctrl+C 停止");
    println!();

    // 订阅消息
    let subscriber_clone = std::sync::Arc::new(subscriber);
    let subscriber_stats = subscriber_clone.clone();

    subscriber_clone
        .subscribe(move |message| {
            // 解析载荷
            let payload_str = String::from_utf8_lossy(&message.payload);

            // 计算延迟
            let now_ns = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;

            let latency_ns = now_ns.saturating_sub(message.timestamp_ns);
            let latency_us = latency_ns / 1000;

            // 显示消息
            match message.msg_type {
                MessageType::Ticker => {
                    println!(
                        "📊 [Seq: {}] Ticker: {} (延迟: {} μs)",
                        message.sequence, payload_str, latency_us
                    );
                }
                MessageType::Heartbeat => {
                    println!(
                        "💓 [Seq: {}] Heartbeat: {} (延迟: {} μs)",
                        message.sequence, payload_str, latency_us
                    );
                }
                MessageType::OrderBook => {
                    println!(
                        "📖 [Seq: {}] OrderBook: {} (延迟: {} μs)",
                        message.sequence, payload_str, latency_us
                    );
                }
                MessageType::Trade => {
                    println!(
                        "💱 [Seq: {}] Trade: {} (延迟: {} μs)",
                        message.sequence, payload_str, latency_us
                    );
                }
            }
        })
        .await?;

    // 每10秒显示统计信息
    let mut interval = time::interval(Duration::from_secs(10));

    loop {
        interval.tick().await;

        let stats = subscriber_stats.stats();

        println!();
        println!("{}", "-".repeat(70));
        println!("统计信息:");
        println!("  接收消息数: {}", stats.messages_received);
        println!("  接收字节数: {}", stats.bytes_received);
        println!("  丢包数: {}", stats.packets_lost);
        println!("  解析错误数: {}", stats.parse_errors);

        if stats.messages_received > 0 {
            let loss_rate =
                (stats.packets_lost as f64 / stats.messages_received as f64) * 100.0;
            println!("  丢包率: {:.2}%", loss_rate);
        }

        println!("{}", "-".repeat(70));
        println!();
    }
}
