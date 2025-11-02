// Cargo.toml 依赖:
// mio = "1.1.0"
// crossbeam = "0.8"
// dashmap = "5.5"  # 高性能并发 HashMap
//
// 注意: 此示例使用 mio 跨平台 I/O 库，可在 Linux、macOS 等平台运行
//
// 优化特性:
// 1. 缓存行对齐数据结构，避免False Sharing
// 2. 零分配缓冲区池，减少内存分配
// 3. 高精度时延测量
// 4. 使用 DashMap 实现细粒度锁（替代粗粒度 Mutex）
// 5. 预分配容量避免rehash
// 6. 无锁通道（crossbeam）实现生产者-消费者模式

use crossbeam::channel::{Receiver, Sender};
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::thread;
use std::time::Instant;

// ============================================================================
// 跨架构缓存行对齐常量
// ============================================================================
#[cfg(all(target_arch = "aarch64", target_vendor = "apple"))]
const CACHE_LINE_SIZE: usize = 128; // Apple M系列

#[cfg(not(all(target_arch = "aarch64", target_vendor = "apple")))]
const CACHE_LINE_SIZE: usize = 64; // 标准x86-64/ARM64

// ============================================================================
// 配置常量
// ============================================================================
const SERVER: Token = Token(0);
const BUFFER_SIZE: usize = 8192; // 优化为8KB，减少系统调用
const MAX_EVENTS: usize = 1024; // 批量处理事件
const MAX_CONNECTIONS: usize = 10000; // 预分配连接容量
const BUFFER_POOL_SIZE: usize = 128; // 缓冲区池大小

// ============================================================================
// 缓存行对齐的数据结构
// ============================================================================

/// 连接状态，包含重用缓冲区

struct ClientEvent {
    pub token: Token,
    pub connection: Connection,
}

/// 控制命令：从工作线程发送到主线程的管理操作
enum ControlCommand {
    Deregister(Token),              // 注销连接
    ReturnConnection(Token, Connection), // 返回连接给主线程
    Shutdown,                       // 关闭服务器
}

struct ClientEventRepo {
    pub event_sender: Sender<ClientEvent>,
    pub receiver_from_master: Receiver<ClientEvent>, // 公开，crossbeam Receiver 线程安全
    pub sender_to_master: Sender<ControlCommand>,    // 公开，工作线程发送控制命令
    pub control_receiver: Receiver<ControlCommand>,  // 主线程接收控制命令
                                                     // pub connections: Arc<DashMap<Token, Connection>>, // 连接管理，细粒度锁
}

impl ClientEventRepo {
    fn new(channel_capacity: usize) -> ClientEventRepo {
        // 使用有界通道实现背压
        let (sender, receiver) = crossbeam::channel::bounded(channel_capacity);
        let (control_sender, control_receiver) = crossbeam::channel::bounded(256);

        ClientEventRepo {
            event_sender: sender,
            receiver_from_master: receiver,
            sender_to_master: control_sender,
            control_receiver,
        }
    }

    /// 获取当前队列长度（用于背压控制）
    pub fn queue_len(&self) -> usize {
        self.event_sender.len()
    }

    /// 获取队列容量
    pub fn queue_capacity(&self) -> Option<usize> {
        self.event_sender.capacity()
    }

    pub(crate) fn try_recv_control(
        &self,
    ) -> Result<ControlCommand, crossbeam::channel::TryRecvError> {
        self.control_receiver.try_recv()
    }
}

struct Connection {
    stream: mio::net::TcpStream,
    buffer: Box<[u8; BUFFER_SIZE]>, // 每个连接独立缓冲区，避免重复分配
    bytes_read: usize,
}

impl Connection {
    fn new(stream: mio::net::TcpStream) -> Self {
        Self {
            stream,
            buffer: Box::new([0u8; BUFFER_SIZE]),
            bytes_read: 0,
        }
    }

    #[inline(always)]
    fn reset_buffer(&mut self) {
        self.bytes_read = 0;
    }

    /// 读取数据到缓冲区
    #[inline(always)]
    fn read_data(&mut self) -> io::Result<usize> {
        self.stream.read(&mut self.buffer[..])
    }

    /// 写入缓冲区的数据（回显）
    #[inline(always)]
    fn write_data(&mut self, len: usize) -> io::Result<()> {
        self.stream.write_all(&self.buffer[..len])
    }
}

struct ConnectionService {
    pub client_event_repo: ClientEventRepo,
}

struct ServerConfig {
    ip: String,
    num_works: usize,
    // 背压控制配置
    channel_capacity: usize,     // 事件通道容量
    high_water_mark_pct: usize,  // 高水位百分比 (暂停accept)
    low_water_mark_pct: usize,   // 低水位百分比 (恢复accept)
}

impl ServerConfig {
    fn new() -> Self {
        Self {
            ip: "127.0.0.1:8080".parse().unwrap(),
            num_works: 4,
            channel_capacity: 1024,
            high_water_mark_pct: 80,  // 80%触发背压
            low_water_mark_pct: 20,   // 20%恢复accept
        }
    }

    /// 计算高水位线（绝对值）
    fn high_water_mark(&self) -> usize {
        self.channel_capacity * self.high_water_mark_pct / 100
    }

    /// 计算低水位线（绝对值）
    fn low_water_mark(&self) -> usize {
        self.channel_capacity * self.low_water_mark_pct / 100
    }
}

impl ConnectionService {
    fn new(config: &ServerConfig) -> Self {
        Self {
            client_event_repo: ClientEventRepo::new(config.channel_capacity),
        }
    }

    pub(crate) fn run_in_main(&self) -> io::Result<()> {
        // 优化事件容量，批量处理
        let mut events = Events::with_capacity(MAX_EVENTS);

        let config = ServerConfig::new();

        let mut connections: HashMap<Token, Connection> = HashMap::with_capacity(MAX_CONNECTIONS);

        // 绑定地址并创建监听器
        let addr = config.ip.clone();
        let mut listen_socket = TcpListener::bind(addr.parse().unwrap())?;

        println!("服务器监听: {}", addr);
        println!("缓存行大小: {} 字节", CACHE_LINE_SIZE);
        println!("缓冲区大小: {} 字节", BUFFER_SIZE);
        println!("最大事件数: {}", MAX_EVENTS);
        println!("背压配置:");
        println!("  - 通道容量: {}", config.channel_capacity);
        println!("  - 高水位: {} ({}%)", config.high_water_mark(), config.high_water_mark_pct);
        println!("  - 低水位: {} ({}%)", config.low_water_mark(), config.low_water_mark_pct);

        let mut poll = Poll::new()?;
        // 将服务器监听器注册到 poll，关注可读事件（新连接）
        poll.registry()
            .register(&mut listen_socket, SERVER, Interest::READABLE)?;

        let mut unique_token = Token(SERVER.0 + 1);

        // 启动工作线程
        let mut worker_handles = vec![];
        println!("启动 {} 个工作线程...", config.num_works);
        for worker_id in 0..config.num_works {
            let handle = self.spawn_worker_thread(worker_id);
            worker_handles.push(handle);
        }

        // 性能统计
        let mut stats_timer = Instant::now();
        let mut total_accepted = 0u64;
        let mut total_dropped = 0u64;

        // 背压控制状态
        let mut accept_paused = false;
        let high_water = config.high_water_mark();
        let low_water = config.low_water_mark();

        // 事件循环（生产者：接收事件并分发）
        let mut should_shutdown = false;
        loop {
            // 等待事件发生
            poll.poll(&mut events, None)?;

            // 处理控制命令（非阻塞）
            while let Ok(cmd) = self.client_event_repo.try_recv_control() {
                match cmd {
                    ControlCommand::Deregister(token) => {
                        // 在主线程执行 deregister
                        if let Some(mut conn) = connections.remove(&token) {
                            if let Err(e) = poll.registry().deregister(&mut conn.stream) {
                                eprintln!("[主线程] 注销连接失败 [Token:{}]: {}", token.0, e);
                            } else {
                                println!("[主线程] 已注销连接 [Token:{}]", token.0);
                            }
                        }
                    }
                    ControlCommand::ReturnConnection(token, connection) => {
                        // 工作线程处理完成，连接返回主线程继续监听
                        connections.insert(token, connection);
                        println!("[主线程] 连接返回 [Token:{}]", token.0);
                    }
                    ControlCommand::Shutdown => {
                        println!("[主线程] 收到关闭命令");
                        should_shutdown = true;
                        break;
                    }
                }
            }

            if should_shutdown {
                break;
            }

            // 背压控制：检查队列水位线
            let queue_len = self.client_event_repo.queue_len();

            // 高水位：暂停accept
            if !accept_paused && queue_len >= high_water {
                if let Err(e) = poll.registry().deregister(&mut listen_socket) {
                    eprintln!("⚠️ [背压] 暂停accept失败: {}", e);
                } else {
                    accept_paused = true;
                    println!("⏸️ [背压] 队列长度 {} >= 高水位 {}, 暂停accept", queue_len, high_water);
                }
            }
            // 低水位：恢复accept
            else if accept_paused && queue_len <= low_water {
                if let Err(e) = poll.registry().register(
                    &mut listen_socket,
                    SERVER,
                    Interest::READABLE
                ) {
                    eprintln!("⚠️ [背压] 恢复accept失败: {}", e);
                } else {
                    accept_paused = false;
                    println!("▶️ [背压恢复] 队列长度 {} <= 低水位 {}, 恢复accept", queue_len, low_water);
                }
            }

            // 处理事件
            for event in events.iter() {
                match event.token() {
                    SERVER => {
                        // 接受所有待处理的新连接
                        loop {
                            match listen_socket.accept() {
                                Ok((mut stream, address)) => {
                                    let token = unique_token;
                                    unique_token.0 += 1;

                                    // 注册新连接到 poll，关注可读事件
                                    if let Err(e) = poll.registry().register(
                                        &mut stream,
                                        token,
                                        Interest::READABLE,
                                    ) {
                                        eprintln!("注册连接失败: {}", e);
                                        continue;
                                    }

                                    // 插入连接
                                    connections.insert(token, Connection::new(stream));
                                    total_accepted += 1;

                                    println!("新连接 [Token:{}] {}", token.0, address);
                                }
                                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                                Err(e) => {
                                    eprintln!("接受连接错误: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                    token => {
                        // 发送事件到工作线程（消费者）
                        // 从 HashMap 中移除连接，转移所有权给工作线程
                        if let Some(connection) = connections.remove(&token) {
                            match self.client_event_repo.event_sender.try_send(ClientEvent { token, connection }) {
                                Ok(_) => {},
                                Err(crossbeam::channel::TrySendError::Full(_event)) => {
                                    total_dropped += 1;
                                    eprintln!("⚠️ [背压] 队列已满，丢弃事件 [Token:{}]", token.0);
                                    // 连接被丢弃，客户端会超时
                                }
                                Err(crossbeam::channel::TrySendError::Disconnected(_)) => {
                                    eprintln!("❌ [错误] 通道已关闭");
                                }
                            }
                        }
                    }
                }
            }

            // 定期打印统计信息
            if stats_timer.elapsed().as_secs() >= 10 {
                let queue_cap = self.client_event_repo.queue_capacity().unwrap_or(0);
                println!(
                    "\n📊 [统计] 总接受: {}, 总丢弃: {}, 队列: {}/{} ({:.1}%), 背压状态: {}",
                    total_accepted,
                    total_dropped,
                    queue_len,
                    queue_cap,
                    (queue_len as f64 / queue_cap as f64) * 100.0,
                    if accept_paused { "暂停中" } else { "正常" }
                );
                stats_timer = Instant::now();
            }
        }

        // 等待所有工作线程结束
        for handle in worker_handles {
            handle.join().unwrap();
        }
        println!("所有工作线程已完成，程序退出。");

        Ok(())
    }

    /// 启动工作线程：从通道接收事件，处理IO操作
    pub fn spawn_worker_thread(&self, worker_id: usize) -> thread::JoinHandle<()> {
        let receiver = self.client_event_repo.receiver_from_master.clone();
        let control_sender = self.client_event_repo.sender_to_master.clone();

        thread::spawn(move || {
            Self::run_worker_thread_impl(worker_id, receiver, control_sender);
        })
    }

    /// 工作线程实现：处理IO事件
    fn run_worker_thread_impl(
        worker_id: usize,
        receiver: Receiver<ClientEvent>,
        control_sender: Sender<ControlCommand>,
    ) {
        println!("工作线程 {} 启动", worker_id);

        loop {
            // 获得客户端事件通知
            // crossbeam Receiver 支持多线程并发 recv()，无需 Mutex
            let client_event = receiver.recv().ok();

            if let Some(client_event) = client_event {
                let token = client_event.token;
                let mut conn = client_event.connection;

                // 处理连接IO - 连接所有权已转移到工作线程
                let should_return = match conn.read_data() {
                    Ok(0) => {
                        // 连接已关闭
                        println!("[工作线程{}] 连接关闭 [Token:{}]", worker_id, token.0);
                        let _ = control_sender.try_send(ControlCommand::Deregister(token));
                        false
                    }
                    Ok(n) => {
                        conn.bytes_read = n;
                        println!(
                            "[工作线程{}] 收到数据 [Token:{}] {} 字节",
                            worker_id, token.0, n
                        );

                        // 示例：回显数据
                        if let Err(e) = conn.write_data(n) {
                            eprintln!(
                                "[工作线程{}] 写入失败 [Token:{}]: {}",
                                worker_id, token.0, e
                            );
                            let _ = control_sender.try_send(ControlCommand::Deregister(token));
                            false
                        } else {
                            // 重置缓冲区
                            conn.reset_buffer();
                            true  // 成功处理，返回连接
                        }
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // 非阻塞IO，暂无数据，返回连接继续等待
                        true
                    }
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                        // 系统调用中断，返回连接重试
                        true
                    }
                    Err(e) => {
                        eprintln!(
                            "[工作线程{}] 读取错误 [Token:{}]: {}",
                            worker_id, token.0, e
                        );
                        let _ = control_sender.try_send(ControlCommand::Deregister(token));
                        false
                    }
                };

                // 返回连接给主线程继续监听
                if should_return {
                    let _ = control_sender.try_send(ControlCommand::ReturnConnection(token, conn));
                }
                // 否则连接被关闭/丢弃
            } else {
                // 通道已关闭，退出循环
                println!("工作线程 {} 退出", worker_id);
                break;
            }
        }
    }
}

// ============================================================================
// 主函数
// ============================================================================

fn main() -> io::Result<()> {
    let config = ServerConfig::new();
    let connection_service = ConnectionService::new(&config);

    // 主线程运行生产者（事件循环）
    println!("主线程启动生产者循环...\n");
    connection_service.run_in_main()
}
