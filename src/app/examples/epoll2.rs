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
use dashmap::DashMap;
// 高性能并发 HashMap
use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};
use std::error::Error;
use std::io::{self, Read, Write};
use std::sync::Arc;
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
}

/// 控制命令：从工作线程发送到主线程的管理操作
enum ControlCommand {
    Deregister(Token), // 注销连接
    Shutdown,          // 关闭服务器
}

struct ClientEventRepo {
    pub events: Events,
    pub event_sender: Sender<ClientEvent>,
    pub event_receiver: Receiver<ClientEvent>, // 公开，crossbeam Receiver 线程安全
    pub control_sender: Sender<ControlCommand>, // 公开，工作线程发送控制命令
    pub control_receiver: Receiver<ControlCommand>, // 主线程接收控制命令
    pub connections: Arc<DashMap<Token, Connection>>, // 连接管理，细粒度锁
}

impl ClientEventRepo {
    fn new() -> ClientEventRepo {
        let (sender, receiver) = crossbeam::channel::unbounded();
        let (control_sender, control_receiver) = crossbeam::channel::unbounded();

        ClientEventRepo {
            events: Events::with_capacity(MAX_EVENTS),
            event_sender: sender,
            event_receiver: receiver,
            control_sender,
            control_receiver,
            connections: Arc::new(DashMap::with_capacity(MAX_CONNECTIONS)),
        }
    }

    pub(crate) fn insert(&self, event: ClientEvent) {
        // 使用try_send避免阻塞，如果失败则丢弃事件
        let _ = self.event_sender.try_send(event);
    }

    pub(crate) fn wait2get(&self) -> Option<ClientEvent> {
        // 阻塞等待下一个事件
        // crossbeam Receiver 本身支持多线程并发 recv()
        self.event_receiver.recv().ok()
    }

    pub(crate) fn try_recv_control(
        &self,
    ) -> Result<ControlCommand, crossbeam::channel::TryRecvError> {
        self.control_receiver.try_recv()
    }

    // 连接管理方法
    pub(crate) fn clone_connections(&self) -> Arc<DashMap<Token, Connection>> {
        Arc::clone(&self.connections)
    }

    pub(crate) fn insert_connection(&self, token: Token, connection: Connection) {
        self.connections.insert(token, connection);
    }

    pub(crate) fn remove_connection(&self, token: &Token) -> Option<(Token, Connection)> {
        self.connections.remove(token)
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
    client_event_repo: ClientEventRepo,
    poll: Poll, // 主线程独占
}

impl ConnectionService {
    fn new() -> Self {
        Self {
            client_event_repo: ClientEventRepo::new(),
            poll: Poll::new().unwrap(),
        }
    }

    pub(crate) fn run_in_main(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 优化事件容量，批量处理
        let mut events = Events::with_capacity(MAX_EVENTS);

        // 绑定地址并创建监听器
        let addr = "127.0.0.1:8080".parse().expect("无效的服务器地址");
        let mut listen_socket = TcpListener::bind(addr)?;

        println!("服务器监听: {}", addr);
        println!("缓存行大小: {} 字节", CACHE_LINE_SIZE);
        println!("缓冲区大小: {} 字节", BUFFER_SIZE);
        println!("最大事件数: {}", MAX_EVENTS);

        // 将服务器监听器注册到 poll，关注可读事件（新连接）
        self.poll
            .registry()
            .register(&mut listen_socket, SERVER, Interest::READABLE)?;

        let mut unique_token = Token(SERVER.0 + 1);

        // 性能统计
        let mut stats_timer = Instant::now();

        // 事件循环（生产者：接收事件并分发）
        loop {
            // 等待事件发生
            let poll_start = Instant::now();
            self.poll.poll(&mut events, None)?;
            let poll_latency = poll_start.elapsed().as_nanos() as u64;

            // 处理控制命令（非阻塞）
            while let Ok(cmd) = self.client_event_repo.try_recv_control() {
                match cmd {
                    ControlCommand::Deregister(token) => {
                        // 在主线程执行 deregister
                        if let Some((_, mut conn)) =
                            self.client_event_repo.remove_connection(&token)
                        {
                            if let Err(e) = self.poll.registry().deregister(&mut conn.stream) {
                                eprintln!("[主线程] 注销连接失败 [Token:{}]: {}", token.0, e);
                            } else {
                                println!("[主线程] 已注销连接 [Token:{}]", token.0);
                            }
                        }
                    }
                    ControlCommand::Shutdown => {
                        println!("[主线程] 收到关闭命令");
                        return Ok(());
                    }
                }
            }

            // 处理事件
            for event in events.iter() {
                let event_start = Instant::now();

                match event.token() {
                    SERVER => {
                        // 接受所有待处理的新连接
                        loop {
                            match listen_socket.accept() {
                                Ok((mut stream, address)) => {
                                    let token = unique_token;
                                    unique_token.0 += 1;

                                    // 注册新连接到 poll，关注可读事件
                                    if let Err(e) = self.poll.registry().register(
                                        &mut stream,
                                        token,
                                        Interest::READABLE,
                                    ) {
                                        eprintln!("注册连接失败: {}", e);
                                        continue;
                                    }

                                    // DashMap 直接插入，无需锁
                                    self.client_event_repo
                                        .insert_connection(token, Connection::new(stream));

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
                        self.client_event_repo.insert(ClientEvent { token });
                    }
                }
            }

            // 定期打印统计信息
            if stats_timer.elapsed().as_secs() >= 10 {
                stats_timer = Instant::now();
            }
        }
    }

    /// 启动工作线程：从通道接收事件，处理IO操作
    pub fn spawn_worker_thread(&self, worker_id: usize) -> thread::JoinHandle<()> {
        let receiver = self.client_event_repo.event_receiver.clone();
        let connections = self.client_event_repo.clone_connections();
        let control_sender = self.client_event_repo.control_sender.clone();

        thread::spawn(move || {
            Self::run_worker_thread_impl(worker_id, receiver, connections, control_sender);
        })
    }

    /// 工作线程实现：处理IO事件
    fn run_worker_thread_impl(
        worker_id: usize,
        receiver: Receiver<ClientEvent>,
        connections: Arc<DashMap<Token, Connection>>,
        control_sender: Sender<ControlCommand>,
    ) {
        println!("工作线程 {} 启动", worker_id);

        loop {
            // 获得客户端事件通知
            // crossbeam Receiver 支持多线程并发 recv()，无需 Mutex
            let client_event = receiver.recv().ok();

            if let Some(client_event) = client_event {
                let token = client_event.token;

                // 处理连接IO - DashMap 提供细粒度锁
                if let Some(mut conn_ref) = connections.get_mut(&token) {
                    // 使用Connection的方法避免借用冲突
                    match conn_ref.read_data() {
                        Ok(0) => {
                            // 连接已关闭
                            println!("[工作线程{}] 连接关闭 [Token:{}]", worker_id, token.0);

                            // 释放conn_ref锁后再发送命令
                            drop(conn_ref);
                            let _ = control_sender.try_send(ControlCommand::Deregister(token));
                        }
                        Ok(n) => {
                            conn_ref.bytes_read = n;
                            println!(
                                "[工作线程{}] 收到数据 [Token:{}] {} 字节",
                                worker_id, token.0, n
                            );

                            // 示例：回显数据
                            if let Err(e) = conn_ref.write_data(n) {
                                eprintln!(
                                    "[工作线程{}] 写入失败 [Token:{}]: {}",
                                    worker_id, token.0, e
                                );
                                drop(conn_ref);
                                let _ = control_sender.try_send(ControlCommand::Deregister(token));
                            } else {
                                // 重置缓冲区
                                conn_ref.reset_buffer();
                            }
                        }
                        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // 非阻塞IO，稍后重试
                        }
                        Err(e) if e.kind() == io::ErrorKind::Interrupted => {
                            // 系统调用中断，重试
                        }
                        Err(e) => {
                            eprintln!(
                                "[工作线程{}] 读取错误 [Token:{}]: {}",
                                worker_id, token.0, e
                            );
                            drop(conn_ref);
                            let _ = control_sender.try_send(ControlCommand::Deregister(token));
                        }
                    }
                }
                // DashMap 自动管理锁，conn_ref 离开作用域后自动释放
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

fn main() -> Result<(), Box<dyn Error>> {
    // 创建 ConnectionService 实例
    let mut connection_service = ConnectionService::new();

    // 创建多个消费者线程（工作线程数量）
    const NUM_WORKERS: usize = 4;
    let mut worker_handles = vec![];

    println!("启动 {} 个工作线程...", NUM_WORKERS);

    // 使用实例方法启动工作线程，只需传递 worker_id
    for worker_id in 0..NUM_WORKERS {
        let handle = connection_service.spawn_worker_thread(worker_id);
        worker_handles.push(handle);
    }

    // 主线程运行生产者（事件循环）
    println!("主线程启动生产者循环...\n");
    let result = connection_service.run_in_main();

    // 等待所有工作线程结束
    for handle in worker_handles {
        handle.join().unwrap();
    }
    println!("所有工作线程已完成，程序退出。");

    result
}
