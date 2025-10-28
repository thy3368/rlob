/// TCP客户端实现
///
/// 实现低延迟、高可靠的TCP单播客户端
/// 关键特性:
/// - 自动重连机制
/// - 指数退避重连策略
/// - TCP_NODELAY降低延迟
/// - 连接状态跟踪

use async_trait::async_trait;
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, timeout, Duration};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use parking_lot::RwLock;
use crate::unicase::domain::unicase::{ClientStats, ConnectionState, MessageType, TcpClient, TcpConfig, UnicastError, UnicastMessage};

/// TCP客户端实现
pub struct TcpUnicastClient {
    /// 配置
    config: TcpConfig,
    /// TCP连接（使用Tokio的Mutex以支持async）
    stream: Arc<Mutex<Option<TcpStream>>>,
    /// 连接状态
    state: Arc<RwLock<ConnectionState>>,
    /// 统计信息
    stats: Arc<ClientStatsInternal>,
    /// 是否正在运行
    running: Arc<AtomicBool>,
}

/// 内部统计信息（使用原子操作）
struct ClientStatsInternal {
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    connect_count: AtomicU64,
    reconnect_count: AtomicU64,
    send_errors: AtomicU64,
    receive_errors: AtomicU64,
}

impl Default for ClientStatsInternal {
    fn default() -> Self {
        Self {
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            connect_count: AtomicU64::new(0),
            reconnect_count: AtomicU64::new(0),
            send_errors: AtomicU64::new(0),
            receive_errors: AtomicU64::new(0),
        }
    }
}

impl TcpUnicastClient {
    /// 创建新的TCP客户端
    pub fn new(config: TcpConfig) -> Self {
        Self {
            config,
            stream: Arc::new(Mutex::new(None)),
            state: Arc::new(RwLock::new(ConnectionState::Disconnected)),
            stats: Arc::new(ClientStatsInternal::default()),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// 内部连接实现
    async fn connect_internal(&mut self) -> Result<(), UnicastError> {
        // 设置连接中状态
        *self.state.write() = ConnectionState::Connecting;

        // 尝试连接
        let stream = match timeout(
            self.config.connect_timeout,
            TcpStream::connect(self.config.server_addr)
        ).await {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                *self.state.write() = ConnectionState::Disconnected;
                return Err(UnicastError::Connection(format!("Failed to connect: {}", e)));
            }
            Err(_) => {
                *self.state.write() = ConnectionState::Disconnected;
                return Err(UnicastError::Timeout);
            }
        };

        // 配置TCP选项
        if self.config.nodelay {
            stream.set_nodelay(true)?;
        }

        // 更新状态
        *self.stream.lock().await = Some(stream);
        *self.state.write() = ConnectionState::Connected;
        self.stats.connect_count.fetch_add(1, Ordering::Relaxed);
        self.running.store(true, Ordering::Relaxed);

        Ok(())
    }

    /// 重连逻辑（带指数退避）
    async fn reconnect_with_backoff(&mut self) -> Result<(), UnicastError> {
        if !self.config.reconnect.enabled {
            return Err(UnicastError::Connection("Reconnect disabled".to_string()));
        }

        *self.state.write() = ConnectionState::Reconnecting;

        let mut attempt = 0u32;
        let mut delay = self.config.reconnect.initial_delay;

        loop {
            // 检查最大重连次数
            if let Some(max) = self.config.reconnect.max_attempts {
                if attempt >= max {
                    *self.state.write() = ConnectionState::Disconnected;
                    return Err(UnicastError::MaxReconnectAttemptsReached);
                }
            }

            attempt += 1;
            self.stats.reconnect_count.fetch_add(1, Ordering::Relaxed);

            eprintln!("Reconnect attempt {} after {:?}", attempt, delay);
            sleep(delay).await;

            // 尝试连接
            match self.connect_internal().await {
                Ok(_) => {
                    eprintln!("Reconnected successfully");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Reconnect failed: {}", e);
                }
            }

            // 指数退避
            delay = std::cmp::min(
                Duration::from_secs_f64(delay.as_secs_f64() * self.config.reconnect.backoff_multiplier),
                self.config.reconnect.max_delay
            );
        }
    }

    /// 序列化消息
    fn serialize_message(message: &UnicastMessage) -> Vec<u8> {
        let mut buf = Vec::new();

        // 消息格式: [长度(4字节)][消息ID(8字节)][时间戳(8字节)][类型(1字节)][载荷]
        let payload_len = message.payload.len();
        let total_len = 4 + 8 + 8 + 1 + payload_len;

        buf.extend_from_slice(&(total_len as u32).to_be_bytes());
        buf.extend_from_slice(&message.message_id.to_be_bytes());
        buf.extend_from_slice(&message.timestamp_ns.to_be_bytes());
        buf.push(message.msg_type.to_u8());
        buf.extend_from_slice(&message.payload);

        buf
    }

    /// 反序列化消息
    fn deserialize_message(data: &[u8]) -> Result<UnicastMessage, UnicastError> {
        if data.len() < 21 {
            return Err(UnicastError::Deserialization("Message too short".to_string()));
        }

        let message_id = u64::from_be_bytes(data[4..12].try_into().unwrap());
        let timestamp_ns = u64::from_be_bytes(data[12..20].try_into().unwrap());
        let msg_type = MessageType::from_u8(data[20])
            .ok_or(UnicastError::InvalidMessageType(data[20]))?;
        let payload = data[21..].to_vec();

        Ok(UnicastMessage {
            message_id,
            timestamp_ns,
            msg_type,
            payload,
        })
    }
}

#[async_trait]
impl TcpClient for TcpUnicastClient {
    async fn connect(&mut self) -> Result<(), UnicastError> {
        self.connect_internal().await
    }

    async fn disconnect(&mut self) -> Result<(), UnicastError> {
        self.running.store(false, Ordering::Relaxed);

        if let Some(mut stream) = self.stream.lock().await.take() {
            stream.shutdown().await?;
        }

        *self.state.write() = ConnectionState::Disconnected;
        Ok(())
    }

    async fn send(&mut self, message: &UnicastMessage) -> Result<(), UnicastError> {
        let data = Self::serialize_message(message);
        self.send_raw(&data).await
    }

    async fn send_raw(&mut self, data: &[u8]) -> Result<(), UnicastError> {
        loop {
            // 获取锁并尝试发送
            let mut stream_guard = self.stream.lock().await;

            if let Some(stream) = stream_guard.as_mut() {
                // 尝试发送
                let result = timeout(
                    self.config.write_timeout.unwrap_or(Duration::from_secs(10)),
                    stream.write_all(data)
                ).await;

                match result {
                    Ok(Ok(_)) => {
                        self.stats.bytes_sent.fetch_add(data.len() as u64, Ordering::Relaxed);
                        self.stats.messages_sent.fetch_add(1, Ordering::Relaxed);
                        return Ok(());
                    }
                    Ok(Err(_)) | Err(_) => {
                        self.stats.send_errors.fetch_add(1, Ordering::Relaxed);
                        *stream_guard = None;
                        drop(stream_guard);

                        // 尝试重连
                        self.reconnect_with_backoff().await?;
                        continue;
                    }
                }
            } else {
                drop(stream_guard);
                // 连接已断开,尝试重连
                self.reconnect_with_backoff().await?;
            }
        }
    }

    async fn receive(&mut self) -> Result<UnicastMessage, UnicastError> {
        // 先读取消息长度(4字节)
        let mut len_buf = [0u8; 4];
        self.receive_raw(&mut len_buf).await?;
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        // 读取完整消息
        let mut msg_buf = vec![0u8; msg_len];
        msg_buf[0..4].copy_from_slice(&len_buf);
        self.receive_raw(&mut msg_buf[4..]).await?;

        // 反序列化
        Self::deserialize_message(&msg_buf)
    }

    async fn receive_raw(&mut self, buffer: &mut [u8]) -> Result<usize, UnicastError> {
        loop {
            // 获取锁并尝试接收
            let mut stream_guard = self.stream.lock().await;

            if let Some(stream) = stream_guard.as_mut() {
                // 尝试接收
                let result = timeout(
                    self.config.read_timeout.unwrap_or(Duration::from_secs(30)),
                    stream.read_exact(buffer)
                ).await;

                match result {
                    Ok(Ok(_)) => {
                        let bytes_read = buffer.len();
                        self.stats.bytes_received.fetch_add(bytes_read as u64, Ordering::Relaxed);
                        self.stats.messages_received.fetch_add(1, Ordering::Relaxed);
                        return Ok(bytes_read);
                    }
                    Ok(Err(_)) | Err(_) => {
                        self.stats.receive_errors.fetch_add(1, Ordering::Relaxed);
                        *stream_guard = None;
                        drop(stream_guard);

                        // 尝试重连
                        self.reconnect_with_backoff().await?;
                        continue;
                    }
                }
            } else {
                drop(stream_guard);
                // 连接已断开,尝试重连
                self.reconnect_with_backoff().await?;
            }
        }
    }

    fn is_connected(&self) -> bool {
        *self.state.read() == ConnectionState::Connected
    }

    fn stats(&self) -> ClientStats {
        ClientStats {
            messages_sent: self.stats.messages_sent.load(Ordering::Relaxed),
            messages_received: self.stats.messages_received.load(Ordering::Relaxed),
            bytes_sent: self.stats.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.stats.bytes_received.load(Ordering::Relaxed),
            connect_count: self.stats.connect_count.load(Ordering::Relaxed),
            reconnect_count: self.stats.reconnect_count.load(Ordering::Relaxed),
            send_errors: self.stats.send_errors.load(Ordering::Relaxed),
            receive_errors: self.stats.receive_errors.load(Ordering::Relaxed),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize() {
        let message = UnicastMessage {
            message_id: 12345,
            timestamp_ns: 67890,
            msg_type: MessageType::OrderCommand,
            payload: vec![1, 2, 3, 4, 5],
        };

        let serialized = TcpUnicastClient::serialize_message(&message);
        let deserialized = TcpUnicastClient::deserialize_message(&serialized).unwrap();

        assert_eq!(deserialized.message_id, message.message_id);
        assert_eq!(deserialized.timestamp_ns, message.timestamp_ns);
        assert_eq!(deserialized.msg_type, message.msg_type);
        assert_eq!(deserialized.payload, message.payload);
    }
}
