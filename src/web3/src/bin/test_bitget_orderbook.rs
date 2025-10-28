/// Test Bitget orderbook API
use web3::domain::{
    entities::Symbol,
    gateways::MarketDataGateway,
};
use web3::infrastructure::exchanges::bitget::BitgetMarketDataGateway;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Bitget OrderBook API\n");
    println!("{}", "=".repeat(60));

    let bitget = BitgetMarketDataGateway::new();

    // Test different symbols
    let symbols = vec![
        "BTCUSDT",
        "ETHUSDT",
        "BTCUSDT_SPBL",  // Bitget spot trading pair format
    ];

    for symbol_str in symbols {
        println!("\n📊 Testing symbol: {}", symbol_str);
        println!("{}", "-".repeat(60));

        let symbol = Symbol::new(symbol_str);

        match bitget.get_orderbook(symbol.clone(), 20).await {
            Ok(orderbook) => {
                println!("✅ Successfully fetched orderbook for {}!", symbol_str);
                println!("\n📈 Statistics:");
                println!("  - Best Bid: {:?}", orderbook.best_bid());
                println!("  - Best Ask: {:?}", orderbook.best_ask());
                println!("  - Spread: {:?}", orderbook.spread());
                println!("  - Bid Depth: {} levels", orderbook.bid_depth());
                println!("  - Ask Depth: {} levels", orderbook.ask_depth());
                println!("  - Timestamp: {}", orderbook.timestamp);

                // Show first 5 levels
                println!("\n📖 Top 5 Bid Levels:");
                for (i, level) in orderbook.bids.iter().take(5).enumerate() {
                    println!("  [{}] {}", i + 1, level);
                }

                println!("\n📖 Top 5 Ask Levels:");
                for (i, level) in orderbook.asks.iter().take(5).enumerate() {
                    println!("  [{}] {}", i + 1, level);
                }

                // Validation
                if orderbook.bid_depth() > 0 && orderbook.ask_depth() > 0 {
                    println!("\n✅ Depth check: PASSED");
                } else {
                    println!("\n❌ Depth check: FAILED (empty orderbook)");
                }

                if let (Some(bid), Some(ask)) = (orderbook.best_bid(), orderbook.best_ask()) {
                    if bid.value() < ask.value() {
                        println!("✅ Price check: PASSED (bid < ask)");
                    } else {
                        println!("❌ Price check: FAILED (crossed market)");
                    }
                }

                break;  // Success, stop testing
            }
            Err(e) => {
                println!("❌ Failed to fetch orderbook: {}", e);
                println!("   Trying next symbol...");
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Test different depths
    println!("\n\n📊 Testing different depth levels");
    println!("{}", "=".repeat(60));

    let symbol = Symbol::new("BTCUSDT");
    for depth in [5, 15, 50, 100] {
        println!("\n🔍 Testing depth: {}", depth);
        match bitget.get_orderbook(symbol.clone(), depth).await {
            Ok(orderbook) => {
                println!("  ✅ Success: {} bids, {} asks",
                    orderbook.bid_depth(), orderbook.ask_depth());
            }
            Err(e) => {
                println!("  ❌ Failed: {}", e);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    println!("\n{}", "=".repeat(60));
    println!("✅ Bitget test completed!");
    println!("{}", "=".repeat(60));

    Ok(())
}
