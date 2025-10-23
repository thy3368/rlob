use web3::domain::{entities::Symbol, gateways::MarketDataGateway};
use web3::infrastructure::exchanges::{binance::BinanceMarketDataGateway, bitget::BitgetMarketDataGateway};
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  🚀 Dual Exchange Real-time Price Monitor                  ║");
    println!("║  US-001-1: Binance + Bitget BTC/USDT 实时价格对比          ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Create gateways for both exchanges
    let binance_gateway = Arc::new(BinanceMarketDataGateway::new());
    let bitget_gateway = Arc::new(BitgetMarketDataGateway::new());

    // Define the trading pair symbol
    let symbol = Symbol::new("BTCUSDT");

    println!("📡 Subscribing to {} ticker on multiple exchanges...\n", symbol);

    // Binance callback
    let binance_callback: Box<dyn Fn(web3::domain::entities::Ticker) + Send + Sync> =
        Box::new(move |ticker| {
            println!("🟡 [Binance] {}", ticker);
            if let Some(spread) = ticker.spread() {
                println!("          💰 Spread: {:.8}", spread);
            }
            if let Some(mid) = ticker.mid_price() {
                println!("          🎯 Mid: {:.8}", mid);
            }
            println!();
        });

    // Bitget callback
    let bitget_callback: Box<dyn Fn(web3::domain::entities::Ticker) + Send + Sync> =
        Box::new(move |ticker| {
            println!("🔵 [Bitget ] {}", ticker);
            if let Some(spread) = ticker.spread() {
                println!("          💰 Spread: {:.8}", spread);
            }
            if let Some(mid) = ticker.mid_price() {
                println!("          🎯 Mid: {:.8}", mid);
            }
            println!();
        });

    // Subscribe to both exchanges
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    binance_gateway.subscribe_ticker(symbol.clone(), binance_callback).await?;
    bitget_gateway.subscribe_ticker(symbol, bitget_callback).await?;

    // Display connection status
    println!(
        "🔗 Binance: {} | Bitget: {}\n",
        if binance_gateway.is_connected() {
            "✅ Connected"
        } else {
            "❌ Disconnected"
        },
        if bitget_gateway.is_connected() {
            "✅ Connected"
        } else {
            "❌ Disconnected"
        }
    );

    println!("Press Ctrl+C to stop...\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Wait for Ctrl+C signal
    signal::ctrl_c().await?;

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🛑 Shutting down gracefully...");

    // Close connections gracefully
    binance_gateway.close().await?;
    bitget_gateway.close().await?;

    println!("✅ All connections closed. Goodbye!");

    Ok(())
}
