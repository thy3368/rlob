使用压测工具验证背压机制：

# 1. 启动服务器
cargo run --example epoll2

# 2. 使用wrk压测（调整连接数触发背压）
wrk -t4 -c2000 -d30s http://127.0.0.1:8080/

# 3. 观察日志输出
# 应该看到：
# ⏸️ [背压] 队列长度 820 >= 高水位 819, 暂停accept
# ▶️ [背压恢复] 队列长度 200 <= 低水位 205, 恢复accept

  ---
🔧 调优参数

根据实际场景调整 ServerConfig:

// 高吞吐场景（牺牲延迟换吞吐）
channel_capacity: 4096,
high_water_mark_pct: 90,
low_water_mark_pct: 30,

// 低延迟场景（严格控制队列长度）
channel_capacity: 512,
high_water_mark_pct: 70,
low_water_mark_pct: 10,
