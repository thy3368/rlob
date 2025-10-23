mod domain;
mod infrastructure;

use domain::{entities::Symbol, gateways::MarketDataGateway};
use infrastructure::exchanges::binance::BinanceMarketDataGateway;
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║  🚀 Binance BTC/USDT Real-time Price Monitor       ║");
    println!("║  US-001: 实时获取币安交易所价格数据                  ║");
    println!("╚══════════════════════════════════════════════════════╝\n");

    // Create Binance gateway (Infrastructure Layer)
    let gateway = Arc::new(BinanceMarketDataGateway::new());

    // Define the trading pair symbol (Domain Layer)
    let symbol = Symbol::new("BTCUSDT");

    println!("📡 Subscribing to {} ticker updates...\n", symbol);

    // Define callback to handle ticker updates
    let callback: Box<dyn Fn(domain::entities::Ticker) + Send + Sync> =
        Box::new(move |ticker| {
            println!("📊 {}", ticker);

            // Display spread if available
            if let Some(spread) = ticker.spread() {
                println!("   💰 Spread: {:.8}", spread);
            }

            // Display mid price if available
            if let Some(mid) = ticker.mid_price() {
                println!("   🎯 Mid Price: {:.8}", mid);
            }

            println!();
        });

    // Subscribe to ticker updates (Use Case Layer)
    gateway.subscribe_ticker(symbol, callback).await?;

    // Display connection status
    println!(
        "🔗 Connection status: {}\n",
        if gateway.is_connected() {
            "✅ Connected"
        } else {
            "❌ Disconnected"
        }
    );

    println!("Press Ctrl+C to stop...\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Wait for Ctrl+C signal
    signal::ctrl_c().await?;

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🛑 Shutting down gracefully...");

    // Close connection gracefully
    gateway.close().await?;

    println!("✅ Connection closed. Goodbye!");

    Ok(())
}
