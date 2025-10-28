/// UDP组播领域定义
///
/// UDP组播用于低延迟的市场数据分发
/// 适用场景:
/// - 实时行情广播
/// - 订单簿更新
/// - 成交数据分发

use std::net::IpAddr;
use async_trait::async_trait;
use thiserror::Error;

/// 组播消息
#[derive(Debug, Clone)]
pub struct MulticastMessage {
    /// 序列号（用于检测丢包）
    pub sequence: u64,
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
    /// Ticker更新
    Ticker = 1,
    /// 订单簿更新
    OrderBook = 2,
    /// 成交数据
    Trade = 3,
    /// 心跳
    Heartbeat = 4,
}

impl MessageType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            1 => Some(Self::Ticker),
            2 => Some(Self::OrderBook),
            3 => Some(Self::Trade),
            4 => Some(Self::Heartbeat),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

/// 组播配置
#[derive(Debug, Clone)]
pub struct MulticastConfig {
    /// 组播地址
    pub multicast_addr: IpAddr,
    /// 组播端口
    pub port: u16,
    /// 本地接口地址（可选，用于多网卡环境）
    pub interface: Option<IpAddr>,
    /// TTL（Time To Live）
    pub ttl: u32,
    /// 是否启用环回
    pub loopback: bool,
}

impl Default for MulticastConfig {
    fn default() -> Self {
        Self {
            multicast_addr: "239.255.0.1".parse().unwrap(),
            port: 9000,
            interface: None,
            ttl: 1,
            loopback: true,
        }
    }
}

/// 组播发送器接口
#[async_trait]
pub trait MulticastPublisher: Send + Sync {
    /// 发送消息
    async fn publish(&self, message: &MulticastMessage) -> Result<(), MulticastError>;

    /// 发送原始数据
    async fn publish_raw(&self, data: &[u8]) -> Result<(), MulticastError>;

    /// 获取发送统计
    fn stats(&self) -> PublisherStats;
}

/// 组播接收器接口
#[async_trait]
pub trait MulticastSubscriber: Send + Sync {
    /// 订阅消息
    async fn subscribe<F>(&self, callback: F) -> Result<(), MulticastError>
    where
        F: Fn(MulticastMessage) + Send + Sync + 'static;

    /// 获取接收统计
    fn stats(&self) -> SubscriberStats;
}

/// 发送统计
#[derive(Debug, Clone, Default)]
pub struct PublisherStats {
    /// 发送的消息数
    pub messages_sent: u64,
    /// 发送的字节数
    pub bytes_sent: u64,
    /// 发送错误数
    pub errors: u64,
}

/// 接收统计
#[derive(Debug, Clone, Default)]
pub struct SubscriberStats {
    /// 接收的消息数
    pub messages_received: u64,
    /// 接收的字节数
    pub bytes_received: u64,
    /// 丢包数（基于序列号检测）
    pub packets_lost: u64,
    /// 解析错误数
    pub parse_errors: u64,
}

/// 组播错误
#[derive(Error, Debug)]
pub enum MulticastError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Deserialization error: {0}")]
    Deserialization(String),

    #[error("Invalid message type: {0}")]
    InvalidMessageType(u8),

    #[error("Socket error: {0}")]
    Socket(String),

    #[error("Configuration error: {0}")]
    Config(String),
}
