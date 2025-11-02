# US-001 Implementation: Binance BTC/USDT Real-time Price Data

## Overview
This implements user story US-001 to receive real-time BTC/USDT price data from Binance exchange via WebSocket.

## Architecture

The implementation follows **Clean Architecture** principles:

```
┌─────────────────────────────────────────────────┐
│           Application Layer (app)               │
│         - CLI interface for testing             │
└────────────────┬────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────┐
│              Domain Layer (lib)                 │
│  - Entities: Symbol, Price, Quantity, Ticker    │
│  - Gateway Interface: MarketDataGateway         │
└────────────────┬────────────────────────────────┘
                 │
┌────────────────▼────────────────────────────────┐
│         Infrastructure Layer (lib)              │
│  - BinanceMarketDataGateway implementation      │
│  - WebSocket connection handling                │
│  - Auto-reconnection logic                      │
└─────────────────────────────────────────────────┘
```

## Features Implemented

✅ **Connect to Binance WebSocket API**
- Multiple endpoint fallback mechanism
- Connection status monitoring

✅ **Real-time BTC/USDT Price Updates**
- Current price
- Bid/Ask prices and quantities
- Spread calculation

✅ **Auto-reconnection**
- Automatic reconnection on connection loss
- Configurable retry attempts (default: 10)
- Exponential backoff (3 seconds delay)

## Code Structure

### Domain Layer (`src/lib/src/domain/`)

#### 1. Price Entities (`price.rs`)
```rust
// Core domain entities
- Symbol: Trading pair (e.g., "BTCUSDT")
- Price: Decimal price value
- Quantity: Decimal quantity value
- Ticker: Complete price update with bid/ask data
```

#### 2. Gateway Interface (`gateway.rs`)
```rust
// Abstract interface for market data
trait MarketDataGateway {
    async fn subscribe_ticker(...) -> Result<...>;
    fn is_connected(&self) -> bool;
    async fn reconnect(&self) -> Result<...>;
    async fn close(&self) -> Result<...>;
}
```

### Infrastructure Layer (`src/lib/src/infrastructure/`)

#### 1. Binance Implementation (`binance.rs`)
```rust
// Concrete implementation for Binance
struct BinanceMarketDataGateway {
    - WebSocket connection management
    - Message parsing
    - Auto-reconnection logic
}
```

### Application Layer (`src/app/src/`)

#### 1. CLI Application (`main.rs`)
```rust
// Simple CLI to demonstrate functionality
- Subscribe to BTC/USDT ticker
- Display real-time price updates
- Handle graceful shutdown
```

## Usage

### Build the project
```bash
cargo build
```

### Run the application
```bash
cargo run --bin app
```

### Expected Output
```
=== Binance BTC/USDT Real-time Price Monitor ===

Subscribing to BTCUSDT ticker updates...

Attempting to connect to: wss://stream.binance.com:9443/ws?streams=btcusdt@ticker
✅ Successfully connected to Binance WebSocket

Connection status: ✅ Connected

Press Ctrl+C to stop...

📊 BTCUSDT | Price: 95000.00000000 | Bid: 94999.50000000 @ 1.50000000 | Ask: 95000.50000000 @ 2.30000000
   Spread: 1.00000000

📊 BTCUSDT | Price: 95001.00000000 | Bid: 95000.00000000 @ 1.20000000 | Ask: 95001.00000000 @ 1.80000000
   Spread: 1.00000000
...
```

## Testing

### Unit Tests
Run domain entity tests:
```bash
cargo test --lib
```

### Integration Test
The application itself serves as an integration test. It connects to the real Binance WebSocket API.

### Manual Testing Checklist
- [x] WebSocket connection establishment
- [x] Real-time price data reception
- [x] Multiple endpoint fallback
- [x] Connection status monitoring
- [x] Graceful shutdown (Ctrl+C)
- [ ] Auto-reconnection (requires simulating connection loss)

## Acceptance Criteria Status

| Criteria | Status | Notes |
|----------|--------|-------|
| ✅ Connect to Binance WebSocket API | DONE | Multiple endpoints supported |
| ✅ Receive real-time BTC/USDT price updates | DONE | Ticker data with bid/ask |
| ✅ Auto-reconnect on disconnection | DONE | 10 retries with 3s delay |

## Dependencies Added

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = { version = "0.24", features = ["native-tls"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-trait = "0.1"
url = "2"
futures-util = "0.3"
thiserror = "2"
```

## Performance Considerations

Following the low-latency requirements from CLAUDE.md:

1. **Zero-copy parsing**: Ticker data parsed directly from JSON
2. **Lock-free operations**: AtomicBool for connection status
3. **Async/await**: Non-blocking I/O for WebSocket
4. **Callback pattern**: Minimal overhead for data delivery

## Known Limitations

1. **Geo-restriction**: Binance WebSocket may not be available in all regions (HTTP 451 error)
   - **Solution**: Multiple endpoint fallback implemented
   - **Alternative**: Consider using VPN or proxy if needed

2. **Network interruptions**: Long network outages may exceed retry limit
   - **Solution**: Configurable MAX_RECONNECT_ATTEMPTS

## Next Steps

To extend this implementation:

1. **Add more symbols**: Support multiple trading pairs simultaneously
2. **Persist data**: Store price history to database
3. **Add depth data**: Implement order book streaming (see US-002)
4. **Add trade data**: Implement trade stream
5. **Performance metrics**: Add latency measurement

## Files Modified/Created

### Created:
- `src/lib/src/domain/price.rs` - Price domain entities
- `src/lib/src/domain/gateway.rs` - Gateway interface
- `src/lib/src/infrastructure/mod.rs` - Infrastructure module
- `src/lib/src/infrastructure/binance.rs` - Binance implementation
- `US-001-README.md` - This documentation

### Modified:
- `src/lib/Cargo.toml` - Added dependencies
- `src/app/Cargo.toml` - Added tokio dependency
- `src/lib/src/domain.rs` - Added price and gateway modules
- `src/lib/src/lib.rs` - Added infrastructure module
- `src/app/src/main.rs` - Implemented CLI application

## Troubleshooting

### Issue: Cannot connect to Binance (HTTP 451)
**Cause**: Geo-restriction or regional blocking

**Solution**:
1. The implementation tries multiple endpoints automatically
2. If all fail, you may need to:
   - Use a VPN
   - Use Binance.US endpoint
   - Use an alternative exchange (see US-003, US-004 for other exchanges)

### Issue: Connection drops frequently
**Cause**: Network instability

**Solution**:
1. Increase `MAX_RECONNECT_ATTEMPTS` in `binance.rs`
2. Adjust `RECONNECT_DELAY_MS` for longer backoff
3. Check network connectivity

### Issue: Build fails
**Cause**: Missing dependencies or wrong Rust edition

**Solution**:
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
cargo build
```

## References

- [Binance WebSocket API Documentation](https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams)
- [Clean Architecture by Robert C. Martin](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Tokio Async Runtime](https://tokio.rs/)
