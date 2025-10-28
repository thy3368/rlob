/// UDP组播接收器实现
///
/// 高性能UDP组播接收，用于市场数据接收

use crate::multicase::domain::multicast::*;
use async_trait::async_trait;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// UDP组播接收器
pub struct UdpMulticastSubscriber {
    socket: Arc<UdpSocket>,
    stats: Arc<SubscriberStatsImpl>,
    last_sequence: Arc<AtomicU64>,
}

struct SubscriberStatsImpl {
    messages_received: AtomicU64,
    bytes_received: AtomicU64,
    packets_lost: AtomicU64,
    parse_errors: AtomicU64,
}

impl Default for SubscriberStatsImpl {
    fn default() -> Self {
        Self {
            messages_received: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            packets_lost: AtomicU64::new(0),
            parse_errors: AtomicU64::new(0),
        }
    }
}

impl UdpMulticastSubscriber {
    /// 创建新的UDP组播接收器
    pub fn new(config: MulticastConfig) -> Result<Self, MulticastError> {
        // 绑定到组播端口
        let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), config.port);
        let socket = UdpSocket::bind(bind_addr)
            .map_err(|e| MulticastError::Socket(format!("Failed to bind socket: {}", e)))?;

        // 加入组播组
        match config.multicast_addr {
            IpAddr::V4(multicast_ipv4) => {
                let interface = match config.interface {
                    Some(IpAddr::V4(ipv4)) => ipv4,
                    _ => Ipv4Addr::UNSPECIFIED,
                };

                socket
                    .join_multicast_v4(&multicast_ipv4, &interface)
                    .map_err(|e| {
                        MulticastError::Socket(format!("Failed to join multicast group: {}", e))
                    })?;
            }
            IpAddr::V6(_) => {
                return Err(MulticastError::Config(
                    "IPv6 multicast not yet supported".to_string(),
                ));
            }
        }

        // 设置为非阻塞模式
        socket
            .set_nonblocking(true)
            .map_err(|e| MulticastError::Socket(format!("Failed to set non-blocking: {}", e)))?;

        Ok(Self {
            socket: Arc::new(socket),
            stats: Arc::new(SubscriberStatsImpl::default()),
            last_sequence: Arc::new(AtomicU64::new(0)),
        })
    }

    /// 反序列化消息
    ///
    /// 消息格式:
    /// - 8字节: 序列号 (little-endian u64)
    /// - 8字节: 时间戳 (little-endian u64)
    /// - 1字节: 消息类型
    /// - 4字节: 载荷长度 (little-endian u32)
    /// - N字节: 载荷数据
    fn deserialize_message(&self, data: &[u8]) -> Result<MulticastMessage, MulticastError> {
        if data.len() < 21 {
            // 最小消息大小: 8+8+1+4 = 21字节
            return Err(MulticastError::Deserialization(
                "Message too short".to_string(),
            ));
        }

        // 解析序列号
        let sequence = u64::from_le_bytes(
            data[0..8]
                .try_into()
                .map_err(|_| MulticastError::Deserialization("Invalid sequence".to_string()))?,
        );

        // 解析时间戳
        let timestamp_ns = u64::from_le_bytes(
            data[8..16]
                .try_into()
                .map_err(|_| MulticastError::Deserialization("Invalid timestamp".to_string()))?,
        );

        // 解析消息类型
        let msg_type_byte = data[16];
        let msg_type = MessageType::from_u8(msg_type_byte)
            .ok_or_else(|| MulticastError::InvalidMessageType(msg_type_byte))?;

        // 解析载荷长度
        let payload_len = u32::from_le_bytes(
            data[17..21]
                .try_into()
                .map_err(|_| MulticastError::Deserialization("Invalid payload length".to_string()))?,
        ) as usize;

        // 验证载荷长度
        if data.len() < 21 + payload_len {
            return Err(MulticastError::Deserialization(
                "Incomplete payload".to_string(),
            ));
        }

        // 提取载荷
        let payload = data[21..21 + payload_len].to_vec();

        Ok(MulticastMessage {
            sequence,
            timestamp_ns,
            msg_type,
            payload,
        })
    }

    /// 检测丢包
    fn check_packet_loss(&self, sequence: u64) {
        let last_seq = self.last_sequence.load(Ordering::Relaxed);

        if last_seq > 0 && sequence > last_seq + 1 {
            // 检测到丢包
            let lost = sequence - last_seq - 1;
            self.stats.packets_lost.fetch_add(lost, Ordering::Relaxed);
        }

        self.last_sequence.store(sequence, Ordering::Relaxed);
    }
}

#[async_trait]
impl MulticastSubscriber for UdpMulticastSubscriber {
    async fn subscribe<F>(&self, callback: F) -> Result<(), MulticastError>
    where
        F: Fn(MulticastMessage) + Send + Sync + 'static,
    {
        let socket = self.socket.clone();
        let stats = self.stats.clone();
        let last_sequence = self.last_sequence.clone();

        let callback = Arc::new(callback);

        tokio::task::spawn(async move {
            let buffer_template = vec![0u8; 65536]; // 64KB缓冲区模板

            loop {
                // 使用spawn_blocking避免阻塞异步运行时
                let socket_clone = socket.clone();
                let mut buf = buffer_template.clone();

                match tokio::task::spawn_blocking(move || {
                    let result = socket_clone.recv_from(&mut buf);
                    (result, buf)  // 返回结果和buffer
                })
                .await
                {
                    Ok((Ok((size, _addr)), buf)) => {
                        stats.bytes_received.fetch_add(size as u64, Ordering::Relaxed);

                        // 反序列化消息
                        match Self::deserialize_message_static(&buf[..size]) {
                            Ok(message) => {
                                // 检测丢包
                                Self::check_packet_loss_static(&last_sequence, &stats, message.sequence);

                                stats.messages_received.fetch_add(1, Ordering::Relaxed);

                                // 调用回调
                                callback(message);
                            }
                            Err(e) => {
                                stats.parse_errors.fetch_add(1, Ordering::Relaxed);
                                eprintln!("Failed to parse message: {}", e);
                            }
                        }
                    }
                    Ok((Err(e), _)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // 非阻塞模式下没有数据，短暂休眠
                        tokio::time::sleep(tokio::time::Duration::from_micros(100)).await;
                    }
                    Ok((Err(e), _)) => {
                        eprintln!("Socket error: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        eprintln!("Task error: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    fn stats(&self) -> SubscriberStats {
        SubscriberStats {
            messages_received: self.stats.messages_received.load(Ordering::Relaxed),
            bytes_received: self.stats.bytes_received.load(Ordering::Relaxed),
            packets_lost: self.stats.packets_lost.load(Ordering::Relaxed),
            parse_errors: self.stats.parse_errors.load(Ordering::Relaxed),
        }
    }
}

impl UdpMulticastSubscriber {
    // 静态辅助方法，用于spawn_blocking中调用
    fn deserialize_message_static(data: &[u8]) -> Result<MulticastMessage, MulticastError> {
        if data.len() < 21 {
            return Err(MulticastError::Deserialization(
                "Message too short".to_string(),
            ));
        }

        let sequence = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let timestamp_ns = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let msg_type_byte = data[16];
        let msg_type = MessageType::from_u8(msg_type_byte)
            .ok_or_else(|| MulticastError::InvalidMessageType(msg_type_byte))?;
        let payload_len = u32::from_le_bytes(data[17..21].try_into().unwrap()) as usize;

        if data.len() < 21 + payload_len {
            return Err(MulticastError::Deserialization(
                "Incomplete payload".to_string(),
            ));
        }

        let payload = data[21..21 + payload_len].to_vec();

        Ok(MulticastMessage {
            sequence,
            timestamp_ns,
            msg_type,
            payload,
        })
    }

    fn check_packet_loss_static(
        last_sequence: &Arc<AtomicU64>,
        stats: &Arc<SubscriberStatsImpl>,
        sequence: u64,
    ) {
        let last_seq = last_sequence.load(Ordering::Relaxed);

        if last_seq > 0 && sequence > last_seq + 1 {
            let lost = sequence - last_seq - 1;
            stats.packets_lost.fetch_add(lost, Ordering::Relaxed);
        }

        last_sequence.store(sequence, Ordering::Relaxed);
    }
}
