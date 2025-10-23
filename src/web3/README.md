# US-001: Binance BTC/USDT 实时价格数据

## ✅ 功能状态：已完成并验证

从币安交易所实时获取 BTC/USDT 价格数据，支持自动重连和多端点故障转移。

## 🎯 验收标准

- ✅ 能够连接到币安 WebSocket API
- ✅ 实时接收 BTC/USDT 价格更新数据
- ✅ 连接断开时能够自动重连

## 🏗️ 架构设计

本项目遵循 **Clean Architecture** 原则，实现清晰的分层架构：

```
src/web3/
├── domain/                         # 领域层（核心业务逻辑）
│   ├── entities/                  # 领域实体
│   │   ├── symbol.rs              # 交易对符号
│   │   ├── price.rs               # 价格和数量值对象
│   │   └── ticker.rs              # Ticker 实体
│   └── gateways/                  # 网关接口定义
│       └── market_data.rs         # 市场数据网关接口
│
├── infrastructure/                # 基础设施层（外部实现）
│   └── exchanges/                 # 交易所实现
│       └── binance/               # Binance 具体实现
│           ├── market_data.rs     # WebSocket 实现
│           └── types.rs           # Binance 特定类型
│
└── main.rs                        # 应用层（CLI入口）
```

### 分层职责

#### Domain Layer (领域层)
- **零外部依赖**：纯 Rust 代码，不依赖任何框架
- **业务规则**：定义核心实体和业务逻辑
- **接口定义**：定义网关接口，由基础设施层实现

#### Infrastructure Layer (基础设施层)
- **实现细节**：WebSocket 连接、消息解析
- **依赖倒置**：实现领域层定义的接口
- **技术选型**：tokio-tungstenite, serde_json

#### Application Layer (应用层)
- **用例编排**：组合领域层和基础设施层
- **依赖注入**：通过构造函数注入具体实现
- **用户界面**：CLI 交互

## 🚀 快速开始

### 编译

```bash
cd src/web3
cargo build --release
```

### 运行

```bash
cargo run --release
```

### 预期输出

```
╔══════════════════════════════════════════════════════╗
║  🚀 Binance BTC/USDT Real-time Price Monitor       ║
║  US-001: 实时获取币安交易所价格数据                  ║
╚══════════════════════════════════════════════════════╝

📡 Subscribing to BTCUSDT ticker updates...

⏳ Attempting to connect to: wss://stream.binance.com:9443/ws/btcusdt@ticker
❌ Failed to connect to wss://stream.binance.com:9443/ws: HTTP error: 451
⏳ Attempting to connect to: wss://stream.binance.us:9443/ws/btcusdt@ticker
✅ Successfully connected to Binance WebSocket
🔗 Connection status: ✅ Connected

Press Ctrl+C to stop...

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📊 BTCUSDT | Price: 109772.77 | Bid: 109773.80 @ 0.21191 | Ask: 110156.10 @ 2.18840
   💰 Spread: 382.30
   🎯 Mid Price: 109964.95
```

## 🔧 关键技术特性

### 1. 多端点故障转移
```rust
const BINANCE_WS_URLS: &[&str] = &[
    "wss://stream.binance.com:9443/ws",
    "wss://stream.binance.com:443/ws",
    "wss://stream.binance.us:9443/ws",
    "wss://fstream.binance.com",
];
```

系统会依次尝试每个端点，直到成功连接。

### 2. 自动重连机制
- 最大重试次数：10次
- 重连延迟：3秒
- 断线自动恢复

### 3. 低延迟设计

遵循 `CLAUDE.md` 中的低延迟标准：

- **零拷贝解析**：直接从 JSON 解析到 Ticker 实体
- **无锁操作**：使用 `AtomicBool` 管理连接状态
- **异步非阻塞**：基于 tokio 的异步 I/O
- **编译优化**：
  ```toml
  [profile.release]
  opt-level = 3
  lto = "fat"
  codegen-units = 1
  panic = "abort"
  ```

### 4. 线程安全

使用 `Arc` 和 `Mutex` 保证多线程安全：
```rust
ws_stream: Arc<Mutex<Option<WsStream>>>
connected: Arc<AtomicBool>
reconnect_count: Arc<AtomicU32>
```

## 📊 数据格式

### Ticker 实体

```rust
pub struct Ticker {
    pub symbol: Symbol,           // 交易对符号（如 "BTCUSDT"）
    pub price: Price,             // 当前价格
    pub bid_price: Option<Price>, // 最优买价
    pub bid_qty: Option<Quantity>,// 最优买量
    pub ask_price: Option<Price>, // 最优卖价
    pub ask_qty: Option<Quantity>,// 最优卖量
    pub timestamp: u64,           // 时间戳（毫秒）
}
```

### 计算指标

- **Spread（价差）**：`ask_price - bid_price`
- **Mid Price（中间价）**：`(bid_price + ask_price) / 2`

## 🧪 测试

### 运行单元测试

```bash
cargo test
```

### 测试覆盖

- ✅ Symbol 创建和转换
- ✅ Price 显示格式化
- ✅ Ticker spread 计算
- ✅ Ticker mid_price 计算

## 🔍 故障排查

### 问题：无法连接到 Binance（HTTP 451）

**原因**：地理位置限制

**解决方案**：
1. 系统会自动尝试备用端点（包括 Binance.US）
2. 如所有端点都失败，考虑使用 VPN
3. 或实现其他交易所数据源（US-002, US-003, US-004）

### 问题：连接频繁断开

**原因**：网络不稳定

**解决方案**：
1. 调整 `MAX_RECONNECT_ATTEMPTS` 增加重试次数
2. 调整 `RECONNECT_DELAY_MS` 增加重连延迟
3. 检查本地网络连接

## 📝 依赖项

```toml
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = { version = "0.24", features = ["native-tls"] }
futures-util = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
async-trait = "0.1"
```

## 🔜 后续开发

根据用户故事规划，后续可以实现：

- **US-002**：Binance 订单簿深度数据
- **US-002-1**：Binance 订单动作流（需要 API 密钥）
- **US-003**：Bitget 交易所数据
- **US-003-1**：Hyperliquid 交易所数据
- **US-004**：OKX 交易所数据

## 📚 参考文档

- [Binance WebSocket API](https://binance-docs.github.io/apidocs/spot/en/#websocket-market-streams)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [Tokio 异步运行时](https://tokio.rs/)

## 🎉 完成情况

✅ **US-001 已完成并验证**

- 成功连接到 Binance WebSocket API
- 实时接收 BTC/USDT 价格数据
- 自动重连机制正常工作
- 代码结构符合 Clean Architecture
- 性能优化符合低延迟标准
