/// Unit tests for OrderBook functionality
use web3::domain::entities::{OrderBook, OrderBookLevel, Price, Quantity, Symbol};

#[test]
fn test_orderbook_creation() {
    let symbol = Symbol::new("BTCUSDT");
    let bids = vec![
        OrderBookLevel::new(Price::new(50000.0), Quantity::new(1.0)),
        OrderBookLevel::new(Price::new(49999.0), Quantity::new(2.0)),
        OrderBookLevel::new(Price::new(49998.0), Quantity::new(1.5)),
    ];
    let asks = vec![
        OrderBookLevel::new(Price::new(50001.0), Quantity::new(1.5)),
        OrderBookLevel::new(Price::new(50002.0), Quantity::new(2.0)),
        OrderBookLevel::new(Price::new(50003.0), Quantity::new(1.0)),
    ];

    let orderbook = OrderBook::new(symbol.clone(), bids.clone(), asks.clone(), 1234567890);

    assert_eq!(orderbook.symbol, symbol);
    assert_eq!(orderbook.bid_depth(), 3);
    assert_eq!(orderbook.ask_depth(), 3);
    assert_eq!(orderbook.timestamp, 1234567890);
}

#[test]
fn test_best_bid_ask() {
    let symbol = Symbol::new("BTCUSDT");
    let bids = vec![
        OrderBookLevel::new(Price::new(50000.0), Quantity::new(1.0)),
        OrderBookLevel::new(Price::new(49999.0), Quantity::new(2.0)),
    ];
    let asks = vec![
        OrderBookLevel::new(Price::new(50001.0), Quantity::new(1.5)),
        OrderBookLevel::new(Price::new(50002.0), Quantity::new(2.0)),
    ];

    let orderbook = OrderBook::new(symbol, bids, asks, 1234567890);

    assert_eq!(orderbook.best_bid(), Some(Price::new(50000.0)));
    assert_eq!(orderbook.best_ask(), Some(Price::new(50001.0)));
}

#[test]
fn test_spread_calculation() {
    let symbol = Symbol::new("BTCUSDT");
    let bids = vec![OrderBookLevel::new(Price::new(50000.0), Quantity::new(1.0))];
    let asks = vec![OrderBookLevel::new(Price::new(50001.0), Quantity::new(1.5))];

    let orderbook = OrderBook::new(symbol, bids, asks, 1234567890);

    assert_eq!(orderbook.spread(), Some(1.0));
}

#[test]
fn test_spread_with_empty_orderbook() {
    let symbol = Symbol::new("BTCUSDT");
    let bids = vec![];
    let asks = vec![];

    let orderbook = OrderBook::new(symbol, bids, asks, 1234567890);

    assert_eq!(orderbook.spread(), None);
    assert_eq!(orderbook.best_bid(), None);
    assert_eq!(orderbook.best_ask(), None);
}

#[test]
fn test_mid_price() {
    let symbol = Symbol::new("BTCUSDT");
    let bids = vec![OrderBookLevel::new(Price::new(50000.0), Quantity::new(1.0))];
    let asks = vec![OrderBookLevel::new(Price::new(50002.0), Quantity::new(1.5))];

    let orderbook = OrderBook::new(symbol, bids, asks, 1234567890);

    // Calculate mid price manually
    if let (Some(bid), Some(ask)) = (orderbook.best_bid(), orderbook.best_ask()) {
        let mid = (bid.value() + ask.value()) / 2.0;
        assert_eq!(mid, 50001.0);
    }
}

#[test]
fn test_100_levels() {
    let symbol = Symbol::new("BTCUSDT");

    // Create 100 bid levels
    let mut bids = Vec::new();
    for i in 0..100 {
        bids.push(OrderBookLevel::new(
            Price::new(50000.0 - i as f64),
            Quantity::new(1.0 + i as f64 * 0.1),
        ));
    }

    // Create 100 ask levels
    let mut asks = Vec::new();
    for i in 0..100 {
        asks.push(OrderBookLevel::new(
            Price::new(50001.0 + i as f64),
            Quantity::new(1.5 + i as f64 * 0.1),
        ));
    }

    let orderbook = OrderBook::new(symbol, bids, asks, 1234567890);

    assert_eq!(orderbook.bid_depth(), 100);
    assert_eq!(orderbook.ask_depth(), 100);
    assert_eq!(orderbook.best_bid(), Some(Price::new(50000.0)));
    assert_eq!(orderbook.best_ask(), Some(Price::new(50001.0)));
    assert_eq!(orderbook.spread(), Some(1.0));
}

// Note: Response parsing tests require access to internal types module
// These tests would verify JSON deserialization but are omitted here
// to maintain encapsulation. The actual parsing is tested implicitly
// when calling get_orderbook() in integration tests.

#[test]
fn test_orderbook_display() {
    let symbol = Symbol::new("BTCUSDT");
    let bids = vec![
        OrderBookLevel::new(Price::new(50000.0), Quantity::new(1.0)),
        OrderBookLevel::new(Price::new(49999.0), Quantity::new(2.0)),
    ];
    let asks = vec![
        OrderBookLevel::new(Price::new(50001.0), Quantity::new(1.5)),
        OrderBookLevel::new(Price::new(50002.0), Quantity::new(2.0)),
    ];

    let orderbook = OrderBook::new(symbol, bids, asks, 1234567890);

    // Test Display trait
    let display_str = format!("{}", orderbook);
    assert!(display_str.contains("BTCUSDT"));
    assert!(display_str.contains("Bids:"));
    assert!(display_str.contains("Asks:"));
}
