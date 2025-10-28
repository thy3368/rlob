use serde::{Deserialize, Serialize};
use crate::domain::{
    entities::{OrderBook, OrderBookLevel, Price, Quantity, Symbol, Ticker},
    gateways::MarketDataError,
};

/// Bitget WebSocket subscription message
#[derive(Debug, Serialize)]
pub struct BitgetSubscription {
    pub op: String,
    pub args: Vec<BitgetSubscriptionArg>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BitgetSubscriptionArg {
    pub inst_type: String,
    pub channel: String,
    pub inst_id: String,
}

impl BitgetSubscription {
    /// Create a ticker subscription for a symbol
    pub fn ticker(symbol: &str) -> Self {
        Self {
            op: "subscribe".to_string(),
            args: vec![BitgetSubscriptionArg {
                inst_type: "SPOT".to_string(),
                channel: "ticker".to_string(),
                inst_id: symbol.to_uppercase(),
            }],
        }
    }
}

/// Bitget WebSocket ticker response
/// Based on: https://www.bitget.com/api-doc/spot/websocket/public/Tickers-Channel
#[derive(Debug, Deserialize)]
pub struct BitgetTickerResponse {
    /// Action type
    pub action: String,

    /// Arguments
    pub arg: BitgetResponseArg,

    /// Ticker data
    pub data: Vec<BitgetTickerData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitgetResponseArg {
    pub inst_type: String,
    pub channel: String,
    pub inst_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BitgetTickerData {
    /// Instrument ID (e.g., "BTCUSDT")
    pub inst_id: String,

    /// Last price
    #[serde(rename = "lastPr")]
    pub last_price: String,

    /// Best bid price
    #[serde(rename = "bidPr")]
    pub bid_price: String,

    /// Best ask price
    #[serde(rename = "askPr")]
    pub ask_price: String,

    /// Best bid size
    #[serde(rename = "bidSz")]
    pub bid_size: String,

    /// Best ask size
    #[serde(rename = "askSz")]
    pub ask_size: String,

    /// 24h open price
    #[serde(rename = "open24h")]
    pub open_24h: String,

    /// 24h high price
    #[serde(rename = "high24h")]
    pub high_24h: String,

    /// 24h low price
    #[serde(rename = "low24h")]
    pub low_24h: String,

    /// 24h change
    #[serde(rename = "change24h")]
    pub change_24h: String,

    /// Timestamp (milliseconds)
    pub ts: String,
}

impl BitgetTickerData {
    /// Convert Bitget ticker data to domain Ticker entity
    pub fn to_ticker(&self) -> Result<Ticker, MarketDataError> {
        let symbol = Symbol::new(&self.inst_id);

        let price = self
            .last_price
            .parse::<f64>()
            .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid price: {}", e)))?;

        let bid_price = self
            .bid_price
            .parse::<f64>()
            .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid bid price: {}", e)))?;

        let bid_qty = self
            .bid_size
            .parse::<f64>()
            .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid bid size: {}", e)))?;

        let ask_price = self
            .ask_price
            .parse::<f64>()
            .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid ask price: {}", e)))?;

        let ask_qty = self
            .ask_size
            .parse::<f64>()
            .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid ask size: {}", e)))?;

        let timestamp = self
            .ts
            .parse::<u64>()
            .map_err(|e| MarketDataError::InvalidMessage(format!("Invalid timestamp: {}", e)))?;

        Ok(Ticker::new(
            symbol,
            Price::new(price),
            Some(Price::new(bid_price)),
            Some(Quantity::new(bid_qty)),
            Some(Price::new(ask_price)),
            Some(Quantity::new(ask_qty)),
            timestamp,
        ))
    }
}

/// Bitget REST API order book depth response
/// Reference: https://www.bitget.com/api-doc/spot/market/Get-Orderbook
#[derive(Debug, Deserialize)]
pub struct BitgetOrderBookResponse {
    pub code: String,
    pub msg: String,
    pub data: BitgetOrderBookData,
}

#[derive(Debug, Deserialize)]
pub struct BitgetOrderBookData {
    /// Bids: [[price, quantity], ...]
    pub bids: Vec<(String, String)>,

    /// Asks: [[price, quantity], ...]
    pub asks: Vec<(String, String)>,

    /// Timestamp
    pub ts: String,
}

impl BitgetOrderBookResponse {
    /// Convert Bitget response to domain OrderBook entity
    pub fn to_orderbook(&self, symbol: Symbol) -> Result<OrderBook, MarketDataError> {
        // Check response code
        if self.code != "00000" {
            return Err(MarketDataError::InvalidMessage(format!(
                "Bitget API error: {} - {}",
                self.code, self.msg
            )));
        }

        let bids: Result<Vec<OrderBookLevel>, MarketDataError> = self
            .data
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
            .data
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

        let timestamp = self
            .data
            .ts
            .parse::<u64>()
            .unwrap_or_else(|_| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64
            });

        Ok(OrderBook::new(symbol, bids?, asks?, timestamp))
    }
}
