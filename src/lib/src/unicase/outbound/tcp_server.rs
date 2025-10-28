/// TCP服务器实现
///
/// 实现低延迟、高并发的TCP单播服务器
/// 关键特性:
/// - 支持多客户端连接
/// - 每个连接独立的异步任务
/// - 广播和单播支持
/// - 连接管理和统计

use async_trait::async_trait;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use parking_lot::RwLock;
use crate::unicase::domain::unicase::{ServerStats, TcpServer, UnicastError, UnicastMessage};

/// 客户端连接信息
struct ClientConnection {
    /// 客户端ID
    id: u64,
    /// 客户端地址
    addr: SocketAddr,
    /// 发送消息通道
    tx: mpsc::UnboundedSender<Vec<u8>>,
}

/// TCP服务器实现
pub struct TcpUnicastServer {
    /// 监听地址
    listen_addr: SocketAddr,
    /// 客户端连接映射
    clients: Arc<RwLock<HashMap<u64, ClientConnection>>>,
    /// 下一个客户端ID
    next_client_id: Arc<AtomicU64>,
    /// 是否正在运行
    running: Arc<AtomicBool>,
    /// 统计信息
    stats: Arc<ServerStatsInternal>,
}

/// 内部统计信息
struct ServerStatsInternal {
    active_connections: AtomicU64,
    total_connections: AtomicU64,
    messages_sent: AtomicU64,
    messages_received: AtomicU64,
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
}

impl Default for ServerStatsInternal {
    fn default() -> Self {
        Self {
            active_connections: AtomicU64::new(0),
            total_connections: AtomicU64::new(0),
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
        }
    }
}

impl TcpUnicastServer {
    /// 创建新的TCP服务器
    pub fn new(listen_addr: SocketAddr) -> Self {
        Self {
            listen_addr,
            clients: Arc::new(RwLock::new(HashMap::new())),
            next_client_id: Arc::new(AtomicU64::new(1)),
            running: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(ServerStatsInternal::default()),
        }
    }

    /// 处理单个客户端连接
    async fn handle_client(
        client_id: u64,
        mut stream: TcpStream,
        addr: SocketAddr,
        mut rx: mpsc::UnboundedReceiver<Vec<u8>>,
        clients: Arc<RwLock<HashMap<u64, ClientConnection>>>,
        stats: Arc<ServerStatsInternal>,
    ) {
        eprintln!("Client {} ({}) connected", client_id, addr);

        // 配置TCP选项
        let _ = stream.set_nodelay(true);

        // 分离读写流
        let (mut reader, mut writer) = stream.into_split();

        // 克隆stats给两个任务使用
        let stats_send = stats.clone();
        let stats_recv = stats.clone();

        // 发送任务
        let send_task = tokio::spawn(async move {
            while let Some(data) = rx.recv().await {
                if let Err(e) = writer.write_all(&data).await {
                    eprintln!("Failed to send to client {}: {}", client_id, e);
                    break;
                }
                stats_send.bytes_sent.fetch_add(data.len() as u64, Ordering::Relaxed);
                stats_send.messages_sent.fetch_add(1, Ordering::Relaxed);
            }
        });

        // 接收任务
        let recv_task = tokio::spawn(async move {
            let mut len_buf = [0u8; 4];

            loop {
                // 读取消息长度
                if let Err(e) = reader.read_exact(&mut len_buf).await {
                    eprintln!("Failed to read from client {}: {}", client_id, e);
                    break;
                }

                let msg_len = u32::from_be_bytes(len_buf) as usize;

                // 读取完整消息
                let mut msg_buf = vec![0u8; msg_len];
                msg_buf[0..4].copy_from_slice(&len_buf);

                if let Err(e) = reader.read_exact(&mut msg_buf[4..]).await {
                    eprintln!("Failed to read message from client {}: {}", client_id, e);
                    break;
                }

                stats_recv.bytes_received.fetch_add(msg_buf.len() as u64, Ordering::Relaxed);
                stats_recv.messages_received.fetch_add(1, Ordering::Relaxed);

                // 这里可以添加消息处理逻辑
                // 例如: 解析消息并触发回调
            }
        });

        // 等待任务完成
        tokio::select! {
            _ = send_task => {},
            _ = recv_task => {},
        }

        // 清理客户端连接
        clients.write().remove(&client_id);
        stats.active_connections.fetch_sub(1, Ordering::Relaxed);

        eprintln!("Client {} ({}) disconnected", client_id, addr);
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
}

#[async_trait]
impl TcpServer for TcpUnicastServer {
    async fn start(&mut self) -> Result<(), UnicastError> {
        if self.running.load(Ordering::Relaxed) {
            return Err(UnicastError::Config("Server already running".to_string()));
        }

        let listener = TcpListener::bind(self.listen_addr).await?;
        self.running.store(true, Ordering::Relaxed);

        eprintln!("TCP server listening on {}", self.listen_addr);

        let clients = self.clients.clone();
        let next_client_id = self.next_client_id.clone();
        let running = self.running.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            while running.load(Ordering::Relaxed) {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        // 生成客户端ID
                        let client_id = next_client_id.fetch_add(1, Ordering::Relaxed);

                        // 创建消息通道
                        let (tx, rx) = mpsc::unbounded_channel();

                        // 保存客户端连接
                        let connection = ClientConnection {
                            id: client_id,
                            addr,
                            tx,
                        };
                        clients.write().insert(client_id, connection);

                        // 更新统计
                        stats.active_connections.fetch_add(1, Ordering::Relaxed);
                        stats.total_connections.fetch_add(1, Ordering::Relaxed);

                        // 启动客户端处理任务
                        let clients_clone = clients.clone();
                        let stats_clone = stats.clone();
                        tokio::spawn(Self::handle_client(
                            client_id,
                            stream,
                            addr,
                            rx,
                            clients_clone,
                            stats_clone,
                        ));
                    }
                    Err(e) => {
                        eprintln!("Failed to accept connection: {}", e);
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<(), UnicastError> {
        self.running.store(false, Ordering::Relaxed);

        // 清理所有客户端连接
        self.clients.write().clear();

        Ok(())
    }

    async fn broadcast(&self, message: &UnicastMessage) -> Result<(), UnicastError> {
        let data = Self::serialize_message(message);
        let clients = self.clients.read();

        for (client_id, client) in clients.iter() {
            if let Err(e) = client.tx.send(data.clone()) {
                eprintln!("Failed to send to client {}: {}", client_id, e);
            }
        }

        Ok(())
    }

    async fn send_to(&self, client_id: u64, message: &UnicastMessage) -> Result<(), UnicastError> {
        let data = Self::serialize_message(message);
        let clients = self.clients.read();

        if let Some(client) = clients.get(&client_id) {
            client.tx.send(data)
                .map_err(|e| UnicastError::Connection(format!("Failed to send: {}", e)))?;
            Ok(())
        } else {
            Err(UnicastError::Connection(format!("Client {} not found", client_id)))
        }
    }

    fn stats(&self) -> ServerStats {
        ServerStats {
            active_connections: self.stats.active_connections.load(Ordering::Relaxed),
            total_connections: self.stats.total_connections.load(Ordering::Relaxed),
            messages_sent: self.stats.messages_sent.load(Ordering::Relaxed),
            messages_received: self.stats.messages_received.load(Ordering::Relaxed),
            bytes_sent: self.stats.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.stats.bytes_received.load(Ordering::Relaxed),
        }
    }
}
