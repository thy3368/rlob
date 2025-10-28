use async_trait::async_trait;
use futures_util::StreamExt;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::domain::{
    entities::{OrderBook, Symbol, Ticker},
    gateways::{MarketDataError, MarketDataGateway},
};

use super::types::{BinanceOrderBookResponse, BinanceTickerResponse};

/// Binance WebSocket endpoints (with fallback support)
/// Using single stream format without combined streams wrapper
const BINANCE_WS_URLS: &[&str] = &[
    "wss://stream.binance.com:9443/ws",
    "wss://stream.binance.com:443/ws",
    "wss://stream.binance.us:9443/ws",
    "wss://fstream.binance.com",  // Futures stream
];

/// Binance REST API base URL
const BINANCE_REST_API_URL: &str = "https://api.binance.com";

const MAX_RECONNECT_ATTEMPTS: u32 = 10;
const RECONNECT_DELAY_MS: u64 = 3000;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Binance implementation of MarketDataGateway
///
/// Features:
/// - Multiple endpoint fallback
/// - Automatic reconnection
/// - Low-latency message processing
/// - Thread-safe connection management
pub struct BinanceMarketDataGateway {
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    connected: Arc<AtomicBool>,
    reconnect_count: Arc<AtomicU32>,
    symbol: Arc<Mutex<Option<Symbol>>>,
}

impl BinanceMarketDataGateway {
    /// Create a new Binance gateway instance
    pub fn new() -> Self {
        Self {
            ws_stream: Arc::new(Mutex::new(None)),
            connected: Arc::new(AtomicBool::new(false)),
            reconnect_count: Arc::new(AtomicU32::new(0)),
            symbol: Arc::new(Mutex::new(None)),
        }
    }

    /// Attempt to connect to Binance WebSocket
    async fn connect_ws(&self, symbol: &Symbol) -> Result<WsStream, MarketDataError> {
        let symbol_lower = symbol.as_str().to_lowercase();

        // Try each endpoint until one succeeds
        let mut last_error = None;

        for base_url in BINANCE_WS_URLS {
            // Using single stream format: wss://stream.binance.com:9443/ws/btcusdt@ticker
            let url = format!("{}/{}@ticker", base_url, symbol_lower);
            println!("⏳ Attempting to connect to: {}", url);

            match connect_async(&url).await {
                Ok((ws_stream, _)) => {
                    println!("✅ Successfully connected to Binance WebSocket");
                    self.connected.store(true, Ordering::SeqCst);
                    self.reconnect_count.store(0, Ordering::SeqCst);
                    return Ok(ws_stream);
                }
                Err(e) => {
                    println!("❌ Failed to connect to {}: {}", base_url, e);
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(MarketDataError::ConnectionError(format!(
            "Failed to connect to all endpoints. Last error: {}",
            last_error
                .map(|e| e.to_string())
                .unwrap_or_else(|| "Unknown error".to_string())
        )))
    }

    /// Handle reconnection logic
    async fn handle_reconnect(&self) -> Result<(), MarketDataError> {
        let symbol = {
            let sym_lock = self.symbol.lock().await;
            sym_lock
                .as_ref()
                .ok_or_else(|| MarketDataError::ConnectionError("No symbol set".to_string()))?
                .clone()
        };

        let attempts = self.reconnect_count.fetch_add(1, Ordering::SeqCst);

        if attempts >= MAX_RECONNECT_ATTEMPTS {
            return Err(MarketDataError::ReconnectionFailed(attempts));
        }

        println!(
            "🔄 Attempting to reconnect... (attempt {}/{})",
            attempts + 1,
            MAX_RECONNECT_ATTEMPTS
        );

        sleep(Duration::from_millis(RECONNECT_DELAY_MS)).await;

        let new_stream = self.connect_ws(&symbol).await?;
        let mut stream_lock = self.ws_stream.lock().await;
        *stream_lock = Some(new_stream);

        Ok(())
    }
}

impl Default for BinanceMarketDataGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MarketDataGateway for BinanceMarketDataGateway {
    async fn subscribe_ticker(
        &self,
        symbol: Symbol,
        callback: Box<dyn Fn(Ticker) + Send + Sync>,
    ) -> Result<(), MarketDataError> {
        // Store symbol for reconnection
        {
            let mut sym_lock = self.symbol.lock().await;
            *sym_lock = Some(symbol.clone());
        }

        // Establish WebSocket connection
        let ws_stream = self.connect_ws(&symbol).await?;
        {
            let mut stream_lock = self.ws_stream.lock().await;
            *stream_lock = Some(ws_stream);
        }

        // Clone Arc references for spawned task
        let ws_stream_arc = Arc::clone(&self.ws_stream);
        let connected_arc = Arc::clone(&self.connected);
        let reconnect_count_arc = Arc::clone(&self.reconnect_count);
        let symbol_arc = Arc::clone(&self.symbol);

        // Spawn async task to handle incoming messages
        tokio::spawn(async move {
            loop {
                // Get next message from WebSocket
                let message = {
                    let mut stream_lock = ws_stream_arc.lock().await;
                    if let Some(stream) = stream_lock.as_mut() {
                        stream.next().await
                    } else {
                        None
                    }
                };

                match message {
                    Some(Ok(Message::Text(text))) => {
                        // Parse JSON message directly (single stream format)
                        match serde_json::from_str::<BinanceTickerResponse>(&text) {
                            Ok(ticker_response) => {
                                match ticker_response.to_ticker() {
                                    Ok(ticker) => {
                                        callback(ticker);
                                    }
                                    Err(e) => {
                                        eprintln!("⚠️  Error converting ticker: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("⚠️  Error parsing ticker response: {}", e);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("🔌 WebSocket connection closed by server");
                        connected_arc.store(false, Ordering::SeqCst);

                        // Attempt reconnection
                        let gateway = BinanceMarketDataGateway {
                            ws_stream: Arc::clone(&ws_stream_arc),
                            connected: Arc::clone(&connected_arc),
                            reconnect_count: Arc::clone(&reconnect_count_arc),
                            symbol: Arc::clone(&symbol_arc),
                        };

                        if let Err(e) = gateway.handle_reconnect().await {
                            eprintln!("❌ Failed to reconnect: {}", e);
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("⚠️  WebSocket error: {}", e);
                        connected_arc.store(false, Ordering::SeqCst);

                        // Attempt reconnection
                        let gateway = BinanceMarketDataGateway {
                            ws_stream: Arc::clone(&ws_stream_arc),
                            connected: Arc::clone(&connected_arc),
                            reconnect_count: Arc::clone(&reconnect_count_arc),
                            symbol: Arc::clone(&symbol_arc),
                        };

                        if let Err(e) = gateway.handle_reconnect().await {
                            eprintln!("❌ Failed to reconnect: {}", e);
                            break;
                        }
                    }
                    None => {
                        println!("🔌 WebSocket stream ended");
                        connected_arc.store(false, Ordering::SeqCst);
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }

    async fn reconnect(&self) -> Result<(), MarketDataError> {
        self.handle_reconnect().await
    }

    async fn close(&self) -> Result<(), MarketDataError> {
        let mut stream_lock = self.ws_stream.lock().await;
        if let Some(stream) = stream_lock.as_mut() {
            stream
                .close(None)
                .await
                .map_err(|e| MarketDataError::WebSocketError(format!("Close error: {}", e)))?;
        }
        self.connected.store(false, Ordering::SeqCst);
        *stream_lock = None;
        Ok(())
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: usize,
    ) -> Result<OrderBook, MarketDataError> {
        // Validate depth parameter (Binance supports: 5, 10, 20, 50, 100, 500, 1000, 5000)
        // For our use case, we'll use the closest valid depth
        let valid_depth = match depth {
            0..=5 => 5,
            6..=10 => 10,
            11..=20 => 20,
            21..=50 => 50,
            51..=100 => 100,
            101..=500 => 500,
            501..=1000 => 1000,
            _ => 5000,
        };

        // Construct REST API URL
        let url = format!(
            "{}/api/v3/depth?symbol={}&limit={}",
            BINANCE_REST_API_URL,
            symbol.as_str(),
            valid_depth
        );

        // Make HTTP request
        let response = reqwest::get(&url)
            .await
            .map_err(|e| MarketDataError::NetworkError(format!("HTTP request failed: {}", e)))?;

        // Check if request was successful
        if !response.status().is_success() {
            return Err(MarketDataError::NetworkError(format!(
                "API returned error status: {}",
                response.status()
            )));
        }

        // Parse response
        let orderbook_response: BinanceOrderBookResponse = response
            .json()
            .await
            .map_err(|e| MarketDataError::InvalidMessage(format!("Failed to parse response: {}", e)))?;

        // Convert to domain entity
        orderbook_response.to_orderbook(symbol)
    }
}
