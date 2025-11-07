// 示例性代码，展示关键步骤
#[cfg(target_os = "linux")]
use ibverbs::{Context, ProtectionDomain, QueuePair, MemoryRegion};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("RDMA is only supported on Linux systems");
        eprintln!("This example requires Linux with libibverbs-dev installed");
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    run_server()
}

#[cfg(target_os = "linux")]
fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    let context = Context::new()?;
    let pd = ProtectionDomain::new(&context)?;

    // 准备一块内存区域并注册，使其可供 RDMA 访问
    let mut data_to_send = vec![1u8, 2, 3, 4];
    let local_mr = MemoryRegion::new(pd, &mut data_to_send, ibverbs::Access::LOCAL_WRITE)?;

    // 创建队列对 (Queue Pair)，这是 RDMA 中数据传输的通道
    let qp = QueuePair::new(&pd, /* 属性配置 */)?;

    // 监听连接...
    // 交换连接信息（如地址句柄）...
    // 等待接收数据...
    Ok(())
}