// my_app/src/main.rs

use macro_lib::log_duration;

// 应用自定义属性宏
#[log_duration]
fn expensive_operation(n: u64) -> u64 {
    // 模拟一个耗时的计算
    std::thread::sleep(std::time::Duration::from_secs(1));
    n * n
}

#[log_duration]
fn another_function() {
    println!("正在执行另一个任务...");
    std::thread::sleep(std::time::Duration::from_millis(500));
}

fn main() {
    let result = expensive_operation(10);
    println!("计算结果: {}", result);

    another_function();
}
