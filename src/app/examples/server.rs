#[cfg(target_os = "linux")]
use async_rdma::{Rdma, RdmaListener};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("RDMA is only supported on Linux systems");
        eprintln!("This example requires Linux with libibverbs-dev installed");
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    run_server().await
}

#[cfg(target_os = "linux")]
async fn run_server() -> Result<(), Box<dyn std::error::Error>> {
    // 服务器在指定地址端口监听连接
    let addr = "192.168.0.1:8080".parse().unwrap();
    let rdma_listener = RdmaListener::bind(addr).await?;

    // 接受客户端的RDMA连接
    let rdma: Rdma = rdma_listener.accept().await?;

    // 接收客户端发送过来的内存区域(MR)元数据
    let local_mr = rdma.receive_local_mr().await?;

    // 从接收到的内存区域中读取数据
    let received_data = local_mr.as_slice();
    println!("从客户端接收到的数据: {:?}", received_data); // 例如，输出可能是 [1, 2, 3, 4]

    Ok(())
}
