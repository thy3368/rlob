// 1. 添加依赖到 Cargo.toml
// [dependencies]
// io-uring = "0.6"

// io_uring is Linux-only, this example will not compile on macOS
#[cfg(target_os = "linux")]
use io_uring::{opcode, IoUring};
#[cfg(target_os = "linux")]
use std::fs::File;
#[cfg(target_os = "linux")]
use std::os::unix::io::AsRawFd;

#[cfg(target_os = "linux")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 2. 创建 io_uring 实例，设置队列深度为 8
    let mut ring = IoUring::new(8)?;
    let file = File::open("test.txt")?;
    let fd = file.as_raw_fd();
    let mut buffer = vec![0u8; 1024];

    // 3. 获取提交队列条目并准备读操作
    let read_e = opcode::Read::new(io_uring::types::Fd(fd), buffer.as_mut_ptr(), buffer.len())
        .offset(0) // 从文件开头读取
        .build()
        .user_data(0x42); // 用于标识此请求的自定义数据

    // 4. 将任务推入提交队列
    unsafe {
        ring.submission().push(&read_e)?;
    }

    // 5. 提交请求给内核并等待至少1个任务完成
    ring.submit_and_wait(1)?;

    // 6. 从完成队列获取结果
    let cqe = ring.completion().next().ok_or("No completion events")?;
    if cqe.result() >= 0 {
        println!("Read {} bytes", cqe.result());
        // 安全地使用已读取的数据
        let bytes_read = cqe.result() as usize;
        println!("Content: {:?}", &buffer[..bytes_read]);
    } else {
        eprintln!("Read error: {}", cqe.result());
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("io_uring is only available on Linux systems.");
    eprintln!("This example cannot run on macOS or other non-Linux platforms.");
    std::process::exit(1);
}
