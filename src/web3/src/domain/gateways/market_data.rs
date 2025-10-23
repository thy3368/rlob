use async_trait::async_trait;
use thiserror::Error;

use crate::domain::entities::{Symbol, Ticker};

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

    /// Check if the gateway is currently connected
    fn is_connected(&self) -> bool;

    /// Manually trigger a reconnection
    async fn reconnect(&self) -> Result<(), MarketDataError>;

    /// Close the connection gracefully
    async fn close(&self) -> Result<(), MarketDataError>;
}
