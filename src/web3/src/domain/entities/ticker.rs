use super::{price::{Price, Quantity}, symbol::Symbol};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Ticker represents real-time price update for a symbol
/// This is the core domain entity for US-001
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticker {
    /// Trading pair symbol
    pub symbol: Symbol,
    /// Current price
    pub price: Price,
    /// Best bid price
    pub bid_price: Option<Price>,
    /// Best bid quantity
    pub bid_qty: Option<Quantity>,
    /// Best ask price
    pub ask_price: Option<Price>,
    /// Best ask quantity
    pub ask_qty: Option<Quantity>,
    /// Timestamp in milliseconds
    pub timestamp: u64,
}

impl Ticker {
    /// Create a new ticker
    pub fn new(
        symbol: Symbol,
        price: Price,
        bid_price: Option<Price>,
        bid_qty: Option<Quantity>,
        ask_price: Option<Price>,
        ask_qty: Option<Quantity>,
        timestamp: u64,
    ) -> Self {
        Self {
            symbol,
            price,
            bid_price,
            bid_qty,
            ask_price,
            ask_qty,
            timestamp,
        }
    }

    /// Calculate the spread between bid and ask prices
    #[inline]
    pub fn spread(&self) -> Option<f64> {
        match (self.bid_price, self.ask_price) {
            (Some(bid), Some(ask)) => Some(ask.value() - bid.value()),
            _ => None,
        }
    }

    /// Calculate the mid price
    #[inline]
    pub fn mid_price(&self) -> Option<f64> {
        match (self.bid_price, self.ask_price) {
            (Some(bid), Some(ask)) => Some((bid.value() + ask.value()) / 2.0),
            _ => None,
        }
    }
}

impl Display for Ticker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} | Price: {} | Bid: {} @ {} | Ask: {} @ {}",
            self.symbol,
            self.price,
            self.bid_price
                .map(|p| format!("{}", p))
                .unwrap_or_else(|| "N/A".to_string()),
            self.bid_qty
                .map(|q| format!("{}", q))
                .unwrap_or_else(|| "N/A".to_string()),
            self.ask_price
                .map(|p| format!("{}", p))
                .unwrap_or_else(|| "N/A".to_string()),
            self.ask_qty
                .map(|q| format!("{}", q))
                .unwrap_or_else(|| "N/A".to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticker_spread() {
        let ticker = Ticker::new(
            Symbol::new("BTCUSDT"),
            Price::new(50000.0),
            Some(Price::new(49999.0)),
            Some(Quantity::new(1.0)),
            Some(Price::new(50001.0)),
            Some(Quantity::new(1.5)),
            1234567890,
        );

        assert_eq!(ticker.spread(), Some(2.0));
    }

    #[test]
    fn test_ticker_mid_price() {
        let ticker = Ticker::new(
            Symbol::new("BTCUSDT"),
            Price::new(50000.0),
            Some(Price::new(49999.0)),
            Some(Quantity::new(1.0)),
            Some(Price::new(50001.0)),
            Some(Quantity::new(1.5)),
            1234567890,
        );

        assert_eq!(ticker.mid_price(), Some(50000.0));
    }
}
