#[cfg(target_os = "linux")]
use ibverbs::{Context, MemoryRegion, ProtectionDomain, QueuePair};

// 示例性代码，展示关键步骤
fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("RDMA is only supported on Linux systems");
        eprintln!("This example requires Linux with libibverbs-dev installed");
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    run_client()
}

#[cfg(target_os = "linux")]
fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    let context = Context::new()?;
    let pd = ProtectionDomain::new(&context)?;

    // 注册一块内存区域来存放待发送的数据
    let mut my_data = vec![5u8, 6, 7, 8];
    let local_mr = MemoryRegion::new(pd, &mut my_data, ibverbs::Access::LOCAL_WRITE)?;

    // 创建队列对
    let qp = QueuePair::new(&pd, /* 属性配置 */)?;

    // 连接到服务器...
    // 获取服务器的远程内存区域信息...

    // 使用 RDMA Write 操作，将本地数据直接写入服务器的内存
    // 这是一个单边操作，服务器CPU不参与
    // qp.post_send(/* 包含本地和远程内存描述符的发送请求 */);

    println!("数据已通过RDMA发送！");
    Ok(())
}