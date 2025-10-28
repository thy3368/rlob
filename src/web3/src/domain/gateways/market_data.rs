use async_trait::async_trait;
use thiserror::Error;

use crate::domain::entities::{OrderBook, Symbol, Ticker};

/// Errors that can occur during market data operations
#[derive(Debug, Error)]
pub enum MarketDataError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Invalid message format: {0}")]
    InvalidMessage(String),

    #[error("WebSocket error: {0}")]
    WebSocketError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Reconnection failed after {0} attempts")]
    ReconnectionFailed(u32),

    #[error("Subscription error: {0}")]
    SubscriptionError(String),
}

/// Gateway interface for receiving real-time market data
///
/// This follows Clean Architecture principles:
/// - The domain defines the interface
/// - The infrastructure layer provides the implementation
/// - Business logic depends on abstractions, not concrete implementations
#[async_trait]
pub trait MarketDataGateway: Send + Sync {
    /// Subscribe to ticker updates for a symbol
    /// The callback is invoked for each ticker update received
    async fn subscribe_ticker(
        &self,
        symbol: Symbol,
        callback: Box<dyn Fn(Ticker) + Send + Sync>,
    ) -> Result<(), MarketDataError>;

    /// Get the order book depth for a specified symbol
    ///
    /// # Arguments
    /// * `symbol` - The trading pair symbol
    /// * `depth` - Number of levels to retrieve (default: 100, max: 100)
    ///
    /// # Returns
    /// Returns an OrderBook with up to `depth` levels on both bid and ask sides
    ///
    /// # Example
    /// ```
    /// let orderbook = gateway.get_orderbook(Symbol::new("BTCUSDT"), 100).await?;
    /// println!("Best bid: {:?}", orderbook.best_bid());
    /// println!("Best ask: {:?}", orderbook.best_ask());
    /// ```
    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: usize,
    ) -> Result<OrderBook, MarketDataError>;

    /// Check if the gateway is currently connected
    fn is_connected(&self) -> bool;

    /// Manually trigger a reconnection
    async fn reconnect(&self) -> Result<(), MarketDataError>;

    /// Close the connection gracefully
    async fn close(&self) -> Result<(), MarketDataError>;
}
