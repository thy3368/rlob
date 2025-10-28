/// UDP组播发送器实现
///
/// 高性能UDP组播发送，用于市场数据分发

use crate::multicase::domain::multicast::*;
use async_trait::async_trait;
use std::net::{IpAddr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// UDP组播发送器
pub struct UdpMulticastPublisher {
    socket: Arc<UdpSocket>,
    target_addr: SocketAddr,
    sequence: Arc<AtomicU64>,
    stats: Arc<PublisherStatsImpl>,
}

struct PublisherStatsImpl {
    messages_sent: AtomicU64,
    bytes_sent: AtomicU64,
    errors: AtomicU64,
}

impl Default for PublisherStatsImpl {
    fn default() -> Self {
        Self {
            messages_sent: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            errors: AtomicU64::new(0),
        }
    }
}

impl UdpMulticastPublisher {
    /// 创建新的UDP组播发送器
    pub fn new(config: MulticastConfig) -> Result<Self, MulticastError> {
        // 创建UDP socket
        let socket = UdpSocket::bind("0.0.0.0:0")
            .map_err(|e| MulticastError::Socket(format!("Failed to bind socket: {}", e)))?;

        // 设置组播TTL
        socket
            .set_multicast_ttl_v4(config.ttl)
            .map_err(|e| MulticastError::Socket(format!("Failed to set TTL: {}", e)))?;

        // 设置组播环回
        socket
            .set_multicast_loop_v4(config.loopback)
            .map_err(|e| MulticastError::Socket(format!("Failed to set loopback: {}", e)))?;

        // 注意: std::net::UdpSocket不支持set_multicast_if_v4
        // 如果需要指定接口，需要使用tokio::net::UdpSocket或socket2 crate
        // 这里我们跳过接口设置
        if config.interface.is_some() {
            eprintln!("Warning: Interface setting not implemented for std::net::UdpSocket");
        }

        // 设置为非阻塞模式
        socket
            .set_nonblocking(true)
            .map_err(|e| MulticastError::Socket(format!("Failed to set non-blocking: {}", e)))?;

        let target_addr = SocketAddr::new(config.multicast_addr, config.port);

        Ok(Self {
            socket: Arc::new(socket),
            target_addr,
            sequence: Arc::new(AtomicU64::new(0)),
            stats: Arc::new(PublisherStatsImpl::default()),
        })
    }

    /// 序列化消息为二进制格式
    ///
    /// 消息格式:
    /// - 8字节: 序列号 (little-endian u64)
    /// - 8字节: 时间戳 (little-endian u64)
    /// - 1字节: 消息类型
    /// - 4字节: 载荷长度 (little-endian u32)
    /// - N字节: 载荷数据
    fn serialize_message(&self, message: &MulticastMessage) -> Vec<u8> {
        let payload_len = message.payload.len() as u32;
        let total_len = 8 + 8 + 1 + 4 + payload_len as usize;

        let mut buffer = Vec::with_capacity(total_len);

        // 序列号
        buffer.extend_from_slice(&message.sequence.to_le_bytes());

        // 时间戳
        buffer.extend_from_slice(&message.timestamp_ns.to_le_bytes());

        // 消息类型
        buffer.push(message.msg_type.to_u8());

        // 载荷长度
        buffer.extend_from_slice(&payload_len.to_le_bytes());

        // 载荷
        buffer.extend_from_slice(&message.payload);

        buffer
    }

    /// 获取当前纳秒时间戳
    fn get_timestamp_ns() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

#[async_trait]
impl MulticastPublisher for UdpMulticastPublisher {
    async fn publish(&self, message: &MulticastMessage) -> Result<(), MulticastError> {
        let data = self.serialize_message(message);
        self.publish_raw(&data).await
    }

    async fn publish_raw(&self, data: &[u8]) -> Result<(), MulticastError> {
        // 克隆数据以满足'static生命周期要求
        let data = data.to_vec();
        let socket = self.socket.clone();
        let target = self.target_addr;
        let stats = self.stats.clone();

        tokio::task::spawn_blocking(move || {
            match socket.send_to(&data, target) {
                Ok(sent) => {
                    stats.messages_sent.fetch_add(1, Ordering::Relaxed);
                    stats.bytes_sent.fetch_add(sent as u64, Ordering::Relaxed);
                    Ok(())
                }
                Err(e) => {
                    stats.errors.fetch_add(1, Ordering::Relaxed);
                    Err(MulticastError::Io(e))
                }
            }
        })
        .await
        .map_err(|e| MulticastError::Socket(format!("Task join error: {}", e)))?
    }

    fn stats(&self) -> PublisherStats {
        PublisherStats {
            messages_sent: self.stats.messages_sent.load(Ordering::Relaxed),
            bytes_sent: self.stats.bytes_sent.load(Ordering::Relaxed),
            errors: self.stats.errors.load(Ordering::Relaxed),
        }
    }
}

impl UdpMulticastPublisher {
    /// 便捷方法：创建并发送消息
    pub async fn send(
        &self,
        msg_type: MessageType,
        payload: Vec<u8>,
    ) -> Result<(), MulticastError> {
        let sequence = self.sequence.fetch_add(1, Ordering::SeqCst);
        let timestamp_ns = Self::get_timestamp_ns();

        let message = MulticastMessage {
            sequence,
            timestamp_ns,
            msg_type,
            payload,
        };

        self.publish(&message).await
    }
}
