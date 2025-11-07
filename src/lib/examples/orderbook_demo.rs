/// 订单簿演示程序
///
/// 本示例演示高性能订单簿实现和匹配引擎

use lib::orderbook::{OrderBook, Side, TraderId};

fn main() {
    println!("=== 高性能订单簿演示 ===\n");

    // 示例 1: 基础订单匹配
    basic_matching_demo();

    println!("\n{}\n", "=".repeat(60));

    // 示例 2: 部分成交
    partial_fill_demo();

    println!("\n{}\n", "=".repeat(60));

    // 示例 3: 价格改善
    price_improvement_demo();

    println!("\n{}\n", "=".repeat(60));

    // 示例 4: 订单取消
    cancellation_demo();

    println!("\n{}\n", "=".repeat(60));

    // 示例 5: 市场深度
    market_depth_demo();
}

fn basic_matching_demo() {
    println!("1. 基础订单匹配");
    println!("   创建新订单簿并匹配订单...\n");

    let mut book = OrderBook::new();

    // 放置卖单
    let seller = TraderId::from_str("ALICE");
    println!("   ALICE 放置卖单: 100 @ $100.00");
    book.limit_order(seller, Side::Sell, 10000, 100);

    println!("   最佳卖价: ${:.2}", book.best_ask().unwrap() as f64 / 100.0);

    // 放置匹配的买单
    let buyer = TraderId::from_str("BOB");
    println!("\n   BOB 放置买单: 100 @ $100.00");
    let (_order_id, trades) = book.limit_order(buyer, Side::Buy, 10000, 100);

    println!("\n   ✅ 交易成功执行:");
    for trade in &trades {
        println!("      {}", trade);
    }

    println!("\n   订单簿现已清空");
    println!("   最佳买价: {:?}", book.best_bid());
    println!("   最佳卖价: {:?}", book.best_ask());
}

fn partial_fill_demo() {
    println!("2. 部分成交场景");
    println!("   演示订单的部分成交...\n");

    let mut book = OrderBook::new();

    // 放置大额卖单
    let seller = TraderId::from_str("CAROL");
    println!("   CAROL 放置卖单: 500 @ $99.50");
    book.limit_order(seller, Side::Sell, 9950, 500);

    // 放置较小的买单
    let buyer = TraderId::from_str("DAVE");
    println!("   DAVE 放置买单: 200 @ $99.50\n");
    let (_order_id, trades) = book.limit_order(buyer, Side::Buy, 9950, 200);

    println!("   ✅ 部分成交:");
    for trade in &trades {
        println!("      {}", trade);
    }

    println!(
        "\n   订单簿剩余: 300 @ ${:.2}",
        book.best_ask().unwrap() as f64 / 100.0
    );
}

fn price_improvement_demo() {
    println!("3. 价格改善");
    println!("   订单以最优可用价格成交...\n");

    let mut book = OrderBook::new();

    // 在$100放置卖单
    let seller = TraderId::from_str("EVE");
    println!("   EVE 放置卖单: 100 @ $100.00");
    book.limit_order(seller, Side::Sell, 10000, 100);

    // 以更高价格放置买单
    let buyer = TraderId::from_str("FRANK");
    println!("   FRANK 放置买单: 100 @ $101.00 (愿意支付更多)\n");
    let (_order_id, trades) = book.limit_order(buyer, Side::Buy, 10100, 100);

    println!("   ✅ 价格改善成交:");
    for trade in &trades {
        println!(
            "      成交价 ${:.2} (节省 ${:.2})",
            trade.price as f64 / 100.0,
            (10100 - trade.price) as f64 / 100.0
        );
    }
}

fn cancellation_demo() {
    println!("4. 订单取消");
    println!("   使用单次内存写入快速取消订单...\n");

    let mut book = OrderBook::new();

    let trader = TraderId::from_str("GRACE");

    // 放置多个订单
    println!("   GRACE 放置 3 个买单:");
    let (id1, _) = book.limit_order(trader, Side::Buy, 9900, 100);
    println!("      订单 #{}: 100 @ $99.00", id1);

    let (id2, _) = book.limit_order(trader, Side::Buy, 9950, 200);
    println!("      订单 #{}: 200 @ $99.50", id2);

    let (id3, _) = book.limit_order(trader, Side::Buy, 10000, 150);
    println!("      订单 #{}: 150 @ $100.00", id3);

    // 取消中间订单
    println!("\n   取消订单 #{}...", id2);
    let cancelled = book.cancel_order(id2);
    println!("   ✅ 已取消: {}", cancelled);

    // 尝试再次取消
    let cancelled_again = book.cancel_order(id2);
    println!("   再次取消: {} (已经取消)", cancelled_again);
}

fn market_depth_demo() {
    println!("5. 市场深度和价差");
    println!("   分析订单簿深度...\n");

    let mut book = OrderBook::new();

    // 构建买方深度
    println!("   构建买单深度:");
    book.limit_order(TraderId::from_str("B1"), Side::Buy, 9900, 100);
    println!("      100 @ $99.00");
    book.limit_order(TraderId::from_str("B2"), Side::Buy, 9950, 200);
    println!("      200 @ $99.50");
    book.limit_order(TraderId::from_str("B3"), Side::Buy, 9980, 150);
    println!("      150 @ $99.80");

    // 构建卖方深度
    println!("\n   构建卖单深度:");
    book.limit_order(TraderId::from_str("S1"), Side::Sell, 10020, 120);
    println!("      120 @ $100.20");
    book.limit_order(TraderId::from_str("S2"), Side::Sell, 10050, 180);
    println!("      180 @ $100.50");
    book.limit_order(TraderId::from_str("S3"), Side::Sell, 10100, 250);
    println!("      250 @ $101.00");

    // 显示市场统计
    println!("\n   📊 市场统计:");
    if let Some(bid) = book.best_bid() {
        println!("      最佳买价:  ${:.2}", bid as f64 / 100.0);
    }
    if let Some(ask) = book.best_ask() {
        println!("      最佳卖价:  ${:.2}", ask as f64 / 100.0);
    }
    if let Some(spread) = book.spread() {
        println!("      价差:      ${:.2}", spread as f64 / 100.0);
    }
    if let Some(mid) = book.mid_price() {
        println!("      中间价:    ${:.2}", mid as f64 / 100.0);
    }
}
