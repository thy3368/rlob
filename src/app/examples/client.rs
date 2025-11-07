#[cfg(target_os = "linux")]
use async_rdma::{LocalMrWriteAccess, RdmaBuilder};
#[cfg(target_os = "linux")]
use std::net::SocketAddrV4;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(not(target_os = "linux"))]
    {
        eprintln!("RDMA is only supported on Linux systems");
        eprintln!("This example requires Linux with libibverbs-dev installed");
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    run_client().await
}

#[cfg(target_os = "linux")]
async fn run_client() -> Result<(), Box<dyn std::error::Error>> {
    // 客户端连接到服务器地址
    let server_addr: SocketAddrV4 = "192.168.0.1:8080".parse().unwrap();
    let rdma = RdmaBuilder::default().connect(server_addr).await?;

    // 为RDMA操作分配一块本地内存区域(Memory Region)
    let layout = std::alloc::Layout::new::<[u8; 8]>(); // 布局描述：8字节的数组
    let mut local_mr = rdma.alloc_local_mr(layout)?;

    // 将数据写入到这块本地内存区域
    let data_to_send = &[1_u8, 2, 3, 4, 5, 6, 7, 8];
    let _bytes_written = local_mr.as_mut_slice().write(data_to_send)?;

    // 向服务器请求一块远程内存区域 (服务器需要预先准备好)
    let mut remote_mr = rdma.request_remote_mr(layout).await?;

    // 使用RDMA Write操作，将本地内存区域的数据直接写入到远程内存区域
    // 这是一个单边操作，服务器端的CPU不会感知到这个写入过程
    rdma.write(&local_mr, &mut remote_mr).await?;

    // （可选）将远程内存区域的元数据发送给服务器，告知数据已写入
    rdma.send_remote_mr(remote_mr).await?;

    Ok(())
}