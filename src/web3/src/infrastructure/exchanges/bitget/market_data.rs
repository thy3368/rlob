use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration, interval};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::domain::{
    entities::{Symbol, Ticker},
    gateways::{MarketDataError, MarketDataGateway},
};

use super::types::{BitgetSubscription, BitgetTickerResponse};

/// Bitget WebSocket endpoints
const BITGET_WS_URLS: &[&str] = &[
    "wss://ws.bitget.com/v2/ws/public",
    "wss://ws.bitget.com/spot/v1/stream",
];

const MAX_RECONNECT_ATTEMPTS: u32 = 10;
const RECONNECT_DELAY_MS: u64 = 3000;
const PING_INTERVAL_SECS: u64 = 25; // Bitget requires ping every 30s

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Bitget implementation of MarketDataGateway
///
/// Features:
/// - Multiple endpoint fallback
/// - Automatic reconnection
/// - Ping/pong heartbeat mechanism
/// - Low-latency message processing
pub struct BitgetMarketDataGateway {
    ws_stream: Arc<Mutex<Option<WsStream>>>,
    connected: Arc<AtomicBool>,
    reconnect_count: Arc<AtomicU32>,
    symbol: Arc<Mutex<Option<Symbol>>>,
}

impl BitgetMarketDataGateway {
    /// Create a new Bitget gateway instance
    pub fn new() -> Self {
        Self {
            ws_stream: Arc::new(Mutex::new(None)),
            connected: Arc::new(AtomicBool::new(false)),
            reconnect_count: Arc::new(AtomicU32::new(0)),
            symbol: Arc::new(Mutex::new(None)),
        }
    }

    /// Attempt to connect to Bitget WebSocket
    async fn connect_ws(&self, symbol: &Symbol) -> Result<WsStream, MarketDataError> {
        let mut last_error = None;

        for base_url in BITGET_WS_URLS {
            println!("⏳ [Bitget] Attempting to connect to: {}", base_url);

            match connect_async(*base_url).await {
                Ok((mut ws_stream, _)) => {
                    println!("✅ [Bitget] Successfully connected to WebSocket");

                    // Send subscription message
                    let subscription = BitgetSubscription::ticker(symbol.as_str());
                    let sub_msg = serde_json::to_string(&subscription)
                        .map_err(|e| MarketDataError::InvalidMessage(e.to_string()))?;

                    ws_stream
                        .send(Message::Text(sub_msg))
                        .await
                        .map_err(|e| MarketDataError::WebSocketError(e.to_string()))?;

                    println!("📡 [Bitget] Subscribed to {} ticker", symbol);

                    self.connected.store(true, Ordering::SeqCst);
                    self.reconnect_count.store(0, Ordering::SeqCst);

                    return Ok(ws_stream);
                }
                Err(e) => {
                    println!("❌ [Bitget] Failed to connect to {}: {}", base_url, e);
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(MarketDataError::ConnectionError(format!(
            "Failed to connect to all Bitget endpoints. Last error: {}",
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
            "🔄 [Bitget] Attempting to reconnect... (attempt {}/{})",
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

impl Default for BitgetMarketDataGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MarketDataGateway for BitgetMarketDataGateway {
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

        // Clone Arc references for spawned tasks
        let ws_stream_arc = Arc::clone(&self.ws_stream);
        let connected_arc = Arc::clone(&self.connected);
        let reconnect_count_arc = Arc::clone(&self.reconnect_count);
        let symbol_arc = Arc::clone(&self.symbol);

        // Spawn ping task for heartbeat
        let ws_stream_ping = Arc::clone(&self.ws_stream);
        let connected_ping = Arc::clone(&self.connected);
        tokio::spawn(async move {
            let mut ping_interval = interval(Duration::from_secs(PING_INTERVAL_SECS));
            loop {
                ping_interval.tick().await;

                if !connected_ping.load(Ordering::SeqCst) {
                    break;
                }

                let mut stream_lock = ws_stream_ping.lock().await;
                if let Some(stream) = stream_lock.as_mut() {
                    if let Err(e) = stream.send(Message::Text("ping".to_string())).await {
                        eprintln!("⚠️  [Bitget] Failed to send ping: {}", e);
                        break;
                    }
                }
            }
        });

        // Spawn message handling task
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
                        // Handle pong response
                        if text == "pong" {
                            continue;
                        }

                        // Parse ticker message
                        match serde_json::from_str::<BitgetTickerResponse>(&text) {
                            Ok(ticker_response) => {
                                for ticker_data in ticker_response.data {
                                    match ticker_data.to_ticker() {
                                        Ok(ticker) => {
                                            callback(ticker);
                                        }
                                        Err(e) => {
                                            eprintln!("⚠️  [Bitget] Error converting ticker: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                // Ignore subscription confirmation and other non-ticker messages
                                if !text.contains("\"event\":\"subscribe\"") {
                                    eprintln!("⚠️  [Bitget] Error parsing ticker response: {}", e);
                                    eprintln!("⚠️  [Bitget] Raw message: {}", text);
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        println!("🔌 [Bitget] WebSocket connection closed by server");
                        connected_arc.store(false, Ordering::SeqCst);

                        // Attempt reconnection
                        let gateway = BitgetMarketDataGateway {
                            ws_stream: Arc::clone(&ws_stream_arc),
                            connected: Arc::clone(&connected_arc),
                            reconnect_count: Arc::clone(&reconnect_count_arc),
                            symbol: Arc::clone(&symbol_arc),
                        };

                        if let Err(e) = gateway.handle_reconnect().await {
                            eprintln!("❌ [Bitget] Failed to reconnect: {}", e);
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("⚠️  [Bitget] WebSocket error: {}", e);
                        connected_arc.store(false, Ordering::SeqCst);

                        // Attempt reconnection
                        let gateway = BitgetMarketDataGateway {
                            ws_stream: Arc::clone(&ws_stream_arc),
                            connected: Arc::clone(&connected_arc),
                            reconnect_count: Arc::clone(&reconnect_count_arc),
                            symbol: Arc::clone(&symbol_arc),
                        };

                        if let Err(e) = gateway.handle_reconnect().await {
                            eprintln!("❌ [Bitget] Failed to reconnect: {}", e);
                            break;
                        }
                    }
                    None => {
                        println!("🔌 [Bitget] WebSocket stream ended");
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
}
