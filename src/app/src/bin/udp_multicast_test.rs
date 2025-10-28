/// UDP组播集成测试
///
/// 在同一进程中运行发送端和接收端，验证端到端功能

use lib::domain::multicast::*;
use lib::outbound::udp_publisher::UdpMulticastPublisher;
use lib::outbound::udp_subscriber::UdpMulticastSubscriber;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "=".repeat(70));
    println!("UDP组播集成测试");
    println!("{}", "=".repeat(70));
    println!();

    // 配置组播参数
    let config = MulticastConfig {
        multicast_addr: "239.255.0.1".parse().unwrap(),
        port: 9001,  // 使用不同端口避免冲突
        interface: None,
        ttl: 1,
        loopback: true,
    };

    println!("测试配置:");
    println!("  组播地址: {}", config.multicast_addr);
    println!("  端口: {}", config.port);
    println!();

    // 创建接收器
    println!("1. 创建UDP组播接收器...");
    let subscriber = Arc::new(UdpMulticastSubscriber::new(config.clone())?);
    println!("   ✓ 接收器创建成功");

    // 用于统计接收的消息
    let received_count = Arc::new(AtomicU64::new(0));
    let received_count_clone = received_count.clone();

    // 订阅消息
    println!("2. 订阅消息...");
    subscriber
        .subscribe(move |message| {
            let count = received_count_clone.fetch_add(1, Ordering::SeqCst) + 1;
            let payload = String::from_utf8_lossy(&message.payload);

            println!("   [✓] 接收消息 #{}: [Seq: {}] {} - {}",
                count,
                message.sequence,
                match message.msg_type {
                    MessageType::Ticker => "Ticker",
                    MessageType::OrderBook => "OrderBook",
                    MessageType::Trade => "Trade",
                    MessageType::Heartbeat => "Heartbeat",
                },
                payload
            );
        })
        .await?;
    println!("   ✓ 订阅成功");
    println!();

    // 短暂延迟，确保接收器准备好
    time::sleep(Duration::from_millis(100)).await;

    // 创建发送器
    println!("3. 创建UDP组播发送器...");
    let publisher = UdpMulticastPublisher::new(config)?;
    println!("   ✓ 发送器创建成功");
    println!();

    // 发送测试消息
    println!("4. 发送测试消息...");

    // 发送5条Ticker消息
    for i in 1..=5 {
        let ticker_data = format!("BTCUSDT Price: {:.2}", 95000.0 + (i as f64 * 0.1));
        publisher
            .send(MessageType::Ticker, ticker_data.as_bytes().to_vec())
            .await?;
        println!("   [→] 发送Ticker #{}: {}", i, ticker_data);
        time::sleep(Duration::from_millis(50)).await;
    }

    // 发送1条心跳
    let heartbeat_data = "Heartbeat #1".to_string();
    publisher
        .send(MessageType::Heartbeat, heartbeat_data.as_bytes().to_vec())
        .await?;
    println!("   [→] 发送Heartbeat: {}", heartbeat_data);
    time::sleep(Duration::from_millis(50)).await;

    // 发送2条OrderBook消息
    for i in 1..=2 {
        let orderbook_data = format!("OrderBook Update #{}", i);
        publisher
            .send(MessageType::OrderBook, orderbook_data.as_bytes().to_vec())
            .await?;
        println!("   [→] 发送OrderBook: {}", orderbook_data);
        time::sleep(Duration::from_millis(50)).await;
    }

    // 发送2条Trade消息
    for i in 1..=2 {
        let trade_data = format!("Trade #{}: BTCUSDT @ 95000", i);
        publisher
            .send(MessageType::Trade, trade_data.as_bytes().to_vec())
            .await?;
        println!("   [→] 发送Trade: {}", trade_data);
        time::sleep(Duration::from_millis(50)).await;
    }

    println!();

    // 等待接收完成
    println!("5. 等待接收完成...");
    time::sleep(Duration::from_millis(500)).await;
    println!();

    // 显示统计信息
    println!("{}", "-".repeat(70));
    println!("测试结果:");
    println!();

    let pub_stats = publisher.stats();
    println!("发送端统计:");
    println!("  发送消息数: {}", pub_stats.messages_sent);
    println!("  发送字节数: {}", pub_stats.bytes_sent);
    println!("  错误数: {}", pub_stats.errors);
    println!();

    let sub_stats = subscriber.stats();
    println!("接收端统计:");
    println!("  接收消息数: {}", sub_stats.messages_received);
    println!("  接收字节数: {}", sub_stats.bytes_received);
    println!("  丢包数: {}", sub_stats.packets_lost);
    println!("  解析错误数: {}", sub_stats.parse_errors);
    println!();

    // 验证结果
    let expected_messages = 10;  // 5 Ticker + 1 Heartbeat + 2 OrderBook + 2 Trade
    let received = received_count.load(Ordering::SeqCst);

    println!("{}", "-".repeat(70));
    println!("验收结果:");
    println!();

    if received == expected_messages {
        println!("✅ 测试通过!");
        println!("   发送: {} 消息", expected_messages);
        println!("   接收: {} 消息", received);
        println!("   丢包: 0");
    } else {
        println!("⚠️  测试异常:");
        println!("   期望接收: {} 消息", expected_messages);
        println!("   实际接收: {} 消息", received);
        println!("   丢失: {} 消息", expected_messages - received);
    }

    println!();

    if sub_stats.parse_errors == 0 {
        println!("✅ 无解析错误");
    } else {
        println!("⚠️  解析错误: {}", sub_stats.parse_errors);
    }

    println!();
    println!("{}", "=".repeat(70));
    println!("测试完成!");
    println!("{}", "=".repeat(70));

    Ok(())
}
