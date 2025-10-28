/// TCP单播领域定义
///
/// TCP单播用于点对点的可靠消息传输
/// 适用场景:
/// - 交易指令下单
/// - 账户查询
/// - 配置同步
/// - 需要确认的关键消息

use async_trait::async_trait;
use thiserror::Error;
use std::net::SocketAddr;
use std::time::Duration;

/// 单播消息
#[derive(Debug, Clone)]
pub struct UnicastMessage {
    /// 消息ID（用于追踪和确认）
    pub message_id: u64,
    /// 时间戳（纳秒）
    pub timestamp_ns: u64,
    /// 消息类型
    pub msg_type: MessageType,
    /// 消息载荷
    pub payload: Vec<u8>,
}

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    /// 交易指令
    OrderCommand = 1,
    /// 查询请求
    QueryRequest = 2,
    /// 查询响应
    QueryResponse = 3,
    /// 配置同步
    ConfigSync = 4,
    /// 心跳
    Heartbeat = 5,
    /// 确认消息
    Ack = 6,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::OrderCommand),
            2 => Some(Self::QueryRequest),
            3 => Some(Self::QueryResponse),
            4 => Some(Self::ConfigSync),
            5 => Some(Self::Heartbeat),
            6 => Some(Self::Ack),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// TCP连接配置
#[derive(Debug, Clone)]
pub struct TcpConfig {
    /// 服务器地址
    pub server_addr: SocketAddr,
    /// 连接超时
    pub connect_timeout: Duration,
    /// 读超时
    pub read_timeout: Option<Duration>,
    /// 写超时
    pub write_timeout: Option<Duration>,
    /// 是否启用Nagle算法（默认禁用以降低延迟）
    pub nodelay: bool,
    /// 接收缓冲区大小
    pub recv_buffer_size: Option<usize>,
    /// 发送缓冲区大小
    pub send_buffer_size: Option<usize>,
    /// 保活配置
    pub keepalive: Option<Duration>,
    /// 自动重连配置
    pub reconnect: ReconnectConfig,
}

impl Default for TcpConfig {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:8080".parse().unwrap(),
            connect_timeout: Duration::from_secs(5),
            read_timeout: Some(Duration::from_secs(30)),
            write_timeout: Some(Duration::from_secs(10)),
            nodelay: true, // 禁用Nagle算法以降低延迟
            recv_buffer_size: Some(64 * 1024),
            send_buffer_size: Some(64 * 1024),
            keepalive: Some(Duration::from_secs(60)),
            reconnect: ReconnectConfig::default(),
        }
    }
}

/// 重连配置
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// 是否启用自动重连
    pub enabled: bool,
    /// 最大重连次数（None表示无限重连）
    pub max_attempts: Option<u32>,
    /// 初始重连延迟
    pub initial_delay: Duration,
    /// 最大重连延迟
    pub max_delay: Duration,
    /// 退避倍数
    pub backoff_multiplier: f64,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: None, // 无限重连
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// TCP客户端接口
#[async_trait]
pub trait TcpClient: Send + Sync {
    /// 连接服务器
    async fn connect(&mut self) -> Result<(), UnicastError>;

    /// 断开连接
    async fn disconnect(&mut self) -> Result<(), UnicastError>;

    /// 发送消息
    async fn send(&mut self, message: &UnicastMessage) -> Result<(), UnicastError>;

    /// 发送原始数据
    async fn send_raw(&mut self, data: &[u8]) -> Result<(), UnicastError>;

    /// 接收消息
    async fn receive(&mut self) -> Result<UnicastMessage, UnicastError>;

    /// 接收原始数据
    async fn receive_raw(&mut self, buffer: &mut [u8]) -> Result<usize, UnicastError>;

    /// 检查连接状态
    fn is_connected(&self) -> bool;

    /// 获取统计信息
    fn stats(&self) -> ClientStats;
}

/// TCP服务器接口
#[async_trait]
pub trait TcpServer: Send + Sync {
    /// 启动服务器
    async fn start(&mut self) -> Result<(), UnicastError>;

    /// 停止服务器
    async fn stop(&mut self) -> Result<(), UnicastError>;

    /// 广播消息到所有连接
    async fn broadcast(&self, message: &UnicastMessage) -> Result<(), UnicastError>;

    /// 发送消息到指定客户端
    async fn send_to(&self, client_id: u64, message: &UnicastMessage) -> Result<(), UnicastError>;

    /// 获取统计信息
    fn stats(&self) -> ServerStats;
}

/// 客户端统计
#[derive(Debug, Clone, Default)]
pub struct ClientStats {
    /// 发送的消息数
    pub messages_sent: u64,
    /// 接收的消息数
    pub messages_received: u64,
    /// 发送的字节数
    pub bytes_sent: u64,
    /// 接收的字节数
    pub bytes_received: u64,
    /// 连接次数
    pub connect_count: u64,
    /// 重连次数
    pub reconnect_count: u64,
    /// 发送错误数
    pub send_errors: u64,
    /// 接收错误数
    pub receive_errors: u64,
}

/// 服务器统计
#[derive(Debug, Clone, Default)]
pub struct ServerStats {
    /// 当前连接数
    pub active_connections: u64,
    /// 累计连接数
    pub total_connections: u64,
    /// 发送的消息数
    pub messages_sent: u64,
    /// 接收的消息数
    pub messages_received: u64,
    /// 发送的字节数
    pub bytes_sent: u64,
    /// 接收的字节数
    pub bytes_received: u64,
}

/// 单播错误
#[derive(Error, Debug)]
pub enum UnicastError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Disconnected")]
    Disconnected,

    #[error("Timeout")]
    Timeout,

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Invalid message type: {0}")]
    InvalidMessageType(u8),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Max reconnect attempts reached")]
    MaxReconnectAttemptsReached,
}

/// 连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// 未连接
    Disconnected,
    /// 正在连接
    Connecting,
    /// 已连接
    Connected,
    /// 正在重连
    Reconnecting,
    /// 已关闭
    Closed,
}
