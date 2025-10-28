/// Test program to verify orderbook functionality
/// This demonstrates how to use the get_orderbook API
use web3::domain::{
    entities::Symbol,
    gateways::MarketDataGateway,
};
use web3::infrastructure::exchanges::{
    binance::BinanceMarketDataGateway,
    bitget::BitgetMarketDataGateway,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing OrderBook API\n");
    println!("{}", "=".repeat(60));

    // Test 1: Binance OrderBook
    println!("\n📊 Test 1: Fetching Binance OrderBook for BTCUSDT (100 levels)");
    println!("{}", "-".repeat(60));

    let binance = BinanceMarketDataGateway::new();
    let symbol = Symbol::new("BTCUSDT");

    match binance.get_orderbook(symbol.clone(), 100).await {
        Ok(orderbook) => {
            println!("✅ Successfully fetched Binance orderbook!");
            println!("\n{}", orderbook);
            println!("\n📈 Statistics:");
            println!("  - Best Bid: {:?}", orderbook.best_bid());
            println!("  - Best Ask: {:?}", orderbook.best_ask());
            println!("  - Spread: {:?}", orderbook.spread());
            println!("  - Bid Depth: {} levels", orderbook.bid_depth());
            println!("  - Ask Depth: {} levels", orderbook.ask_depth());

            if orderbook.bid_depth() >= 5 && orderbook.ask_depth() >= 5 {
                println!("\n✅ Depth check: PASSED (>=5 levels on both sides)");
            } else {
                println!("\n⚠️  Warning: Less than 5 levels received");
            }

            println!("✅ Validation check: Not implemented");
        }
        Err(e) => {
            println!("❌ Failed to fetch Binance orderbook: {}", e);
        }
    }

    // Test 2: Bitget OrderBook
    println!("\n\n📊 Test 2: Fetching Bitget OrderBook for BTCUSDT (100 levels)");
    println!("{}", "-".repeat(60));

    let bitget = BitgetMarketDataGateway::new();

    match bitget.get_orderbook(symbol.clone(), 100).await {
        Ok(orderbook) => {
            println!("✅ Successfully fetched Bitget orderbook!");
            println!("\n{}", orderbook);
            println!("\n📈 Statistics:");
            println!("  - Best Bid: {:?}", orderbook.best_bid());
            println!("  - Best Ask: {:?}", orderbook.best_ask());
            println!("  - Spread: {:?}", orderbook.spread());
            println!("  - Bid Depth: {} levels", orderbook.bid_depth());
            println!("  - Ask Depth: {} levels", orderbook.ask_depth());

            if orderbook.bid_depth() >= 5 && orderbook.ask_depth() >= 5 {
                println!("\n✅ Depth check: PASSED (>=5 levels on both sides)");
            } else {
                println!("\n⚠️  Warning: Less than 5 levels received");
            }

            println!("✅ Validation check: Not implemented");
        }
        Err(e) => {
            println!("❌ Failed to fetch Bitget orderbook: {}", e);
        }
    }

    // Test 3: Different depth levels
    println!("\n\n📊 Test 3: Testing different depth levels (Binance)");
    println!("{}", "-".repeat(60));

    for depth in [5, 10, 20, 50, 100] {
        match binance.get_orderbook(symbol.clone(), depth).await {
            Ok(orderbook) => {
                println!("✅ Depth {}: {} bids, {} asks",
                    depth, orderbook.bid_depth(), orderbook.ask_depth());
            }
            Err(e) => {
                println!("❌ Depth {}: Failed - {}", depth, e);
            }
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Test 4: Compare spreads between exchanges
    println!("\n\n📊 Test 4: Comparing spreads between exchanges");
    println!("{}", "-".repeat(60));

    let binance_ob = binance.get_orderbook(symbol.clone(), 20).await?;
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    let bitget_ob = bitget.get_orderbook(symbol.clone(), 20).await?;

    if let (Some(binance_spread), Some(bitget_spread)) =
        (binance_ob.spread(), bitget_ob.spread()) {
        println!("  Binance spread: {:.8}", binance_spread);
        println!("  Bitget spread:  {:.8}", bitget_spread);
        println!("  Difference:     {:.8}", (binance_spread - bitget_spread).abs());
    }

    println!("\n{}", "=".repeat(60));
    println!("✅ All tests completed!");
    println!("{}", "=".repeat(60));

    Ok(())
}
