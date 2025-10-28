use serde::Deserialize;
use crate::domain::{
    entities::{OrderBook, OrderBookLevel, Price, Quantity, Symbol, Ticker},
    gateways::MarketDataError,
};

/// Binance WebSocket ticker response format
/// Based on Binance 24hr Ticker Stream
/// Reference: https://binance-docs.github.io/apidocs/spot/en/#individual-symbol-ticker-streams
#[derive(Debug, Deserialize)]
pub struct BinanceTickerResponse {
    /// Event type
    #[serde(rename = "e")]
    pub event_type: String,

    /// Event time
    #[serde(rename = "E")]
    pub event_time: u64,

    /// Symbol
    #[serde(rename = "s")]
    pub symbol: String,

    /// Current day's close price (last price)
    #[serde(rename = "c")]
    pub current_price: String,

    /// Best bid price
    #[serde(rename = "b")]
    pub bid_price: String,

    /// Best bid quantity
    #[serde(rename = "B")]
    pub bid_qty: String,

    /// Best ask price
    #[serde(rename = "a")]
    pub ask_price: String,

    /// Best ask quantity
    #[serde(rename = "A")]
    pub ask_qty: String,
}

impl BinanceTickerResponse {
    /// Convert Binance response to domain Ticker entity
    pub fn to_ticker(&self) -> Result<Ticker, MarketDataError> {
        let symbol = Symbol::new(&self.symbol);

        let price = self
            .current_price
            .parse::<f64>()
            .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid price: {}", e)))?;

        let bid_price = self
            .bid_price
            .parse::<f64>()
            .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid bid price: {}", e)))?;

        let bid_qty = self
            .bid_qty
            .parse::<f64>()
            .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid bid qty: {}", e)))?;

        let ask_price = self
            .ask_price
            .parse::<f64>()
            .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid ask price: {}", e)))?;

        let ask_qty = self
            .ask_qty
            .parse::<f64>()
            .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid ask qty: {}", e)))?;

        Ok(Ticker::new(
            symbol,
            Price::new(price),
            Some(Price::new(bid_price)),
            Some(Quantity::new(bid_qty)),
            Some(Price::new(ask_price)),
            Some(Quantity::new(ask_qty)),
            self.event_time,
        ))
    }
}

/// Binance REST API order book depth response
/// Reference: https://binance-docs.github.io/apidocs/spot/en/#order-book
#[derive(Debug, Deserialize)]
pub struct BinanceOrderBookResponse {
    /// Last update ID
    #[serde(rename = "lastUpdateId")]
    pub last_update_id: u64,

    /// Bids: [[price, quantity], ...]
    pub bids: Vec<(String, String)>,

    /// Asks: [[price, quantity], ...]
    pub asks: Vec<(String, String)>,
}

impl BinanceOrderBookResponse {
    /// Convert Binance response to domain OrderBook entity
    pub fn to_orderbook(&self, symbol: Symbol) -> Result<OrderBook, MarketDataError> {
        let bids: Result<Vec<OrderBookLevel>, MarketDataError> = self
            .bids
            .iter()
            .map(|(price_str, qty_str)| {
                let price = price_str
                    .parse::<f64>()
                    .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid bid price: {}", e)))?;
                let quantity = qty_str
                    .parse::<f64>()
                    .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid bid quantity: {}", e)))?;
                Ok(OrderBookLevel::new(Price::new(price), Quantity::new(quantity)))
            })
            .collect();

        let asks: Result<Vec<OrderBookLevel>, MarketDataError> = self
            .asks
            .iter()
            .map(|(price_str, qty_str)| {
                let price = price_str
                    .parse::<f64>()
                    .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid ask price: {}", e)))?;
                let quantity = qty_str
                    .parse::<f64>()
                    .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid ask quantity: {}", e)))?;
                Ok(OrderBookLevel::new(Price::new(price), Quantity::new(quantity)))
            })
            .collect();

        // Use current timestamp in milliseconds
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Ok(OrderBook::new(symbol, bids?, asks?, timestamp))
    }
}
