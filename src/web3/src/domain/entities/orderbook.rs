use super::{price::{Price, Quantity}, symbol::Symbol};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// OrderBookLevel represents a single price level in the order book
/// Optimized for low-latency with inline functions
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OrderBookLevel {
    /// Price at this level
    pub price: Price,
    /// Total quantity at this price level
    pub quantity: Quantity,
}

impl OrderBookLevel {
    /// Create a new order book level
    #[inline]
    pub fn new(price: Price, quantity: Quantity) -> Self {
        Self { price, quantity }
    }
}

impl Display for OrderBookLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}", self.quantity, self.price)
    }
}

/// OrderBook represents the limit order book depth for a trading pair
/// Supports up to 100 levels on each side (bid/ask)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderBook {
    /// Trading pair symbol
    pub symbol: Symbol,
    /// Bid levels (buy orders), sorted from highest to lowest price
    pub bids: Vec<OrderBookLevel>,
    /// Ask levels (sell orders), sorted from lowest to highest price
    pub asks: Vec<OrderBookLevel>,
    /// Timestamp in milliseconds
    pub timestamp: u64,
}

impl OrderBook {
    /// Create a new order book
    pub fn new(
        symbol: Symbol,
        bids: Vec<OrderBookLevel>,
        asks: Vec<OrderBookLevel>,
        timestamp: u64,
    ) -> Self {
        Self {
            symbol,
            bids,
            asks,
            timestamp,
        }
    }

    /// Get the best bid price (highest buy price)
    #[inline]
    pub fn best_bid(&self) -> Option<Price> {
        self.bids.first().map(|level| level.price)
    }

    /// Get the best ask price (lowest sell price)
    #[inline]
    pub fn best_ask(&self) -> Option<Price> {
        self.asks.first().map(|level| level.price)
    }

    /// Calculate the spread between best bid and best ask
    #[inline]
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.value() - bid.value()),
            _ => None,
        }
    }

    /// Get the depth (number of levels) on the bid side
    #[inline]
    pub fn bid_depth(&self) -> usize {
        self.bids.len()
    }

    /// Get the depth (number of levels) on the ask side
    #[inline]
    pub fn ask_depth(&self) -> usize {
        self.asks.len()
    }
}

impl Display for OrderBook {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "OrderBook for {}", self.symbol)?;
        writeln!(f, "Bids: {} levels, Asks: {} levels", self.bid_depth(), self.ask_depth())?;
        if let Some(spread) = self.spread() {
            writeln!(f, "Spread: {:.8}", spread)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orderbook_creation() {
        let ob = OrderBook::new(
            Symbol::new("BTCUSDT"),
            vec![OrderBookLevel::new(Price::new(50000.0), Quantity::new(1.0))],
            vec![OrderBookLevel::new(Price::new(50001.0), Quantity::new(1.5))],
            1234567890,
        );

        assert_eq!(ob.symbol, Symbol::new("BTCUSDT"));
        assert_eq!(ob.bid_depth(), 1);
        assert_eq!(ob.ask_depth(), 1);
    }

    #[test]
    fn test_best_prices() {
        let ob = OrderBook::new(
            Symbol::new("BTCUSDT"),
            vec![
                OrderBookLevel::new(Price::new(50000.0), Quantity::new(1.0)),
                OrderBookLevel::new(Price::new(49999.0), Quantity::new(2.0)),
            ],
            vec![
                OrderBookLevel::new(Price::new(50001.0), Quantity::new(1.5)),
                OrderBookLevel::new(Price::new(50002.0), Quantity::new(2.0)),
            ],
            1234567890,
        );

        assert_eq!(ob.best_bid(), Some(Price::new(50000.0)));
        assert_eq!(ob.best_ask(), Some(Price::new(50001.0)));
        assert_eq!(ob.spread(), Some(1.0));
    }
}
