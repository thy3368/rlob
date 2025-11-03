fn main() {
    let mut count = 0;
    'outer_loop: loop {
        // 为外层循环设置标签 'outer_loop
        println!("外层循环计数: {}", count);
        let mut inner_counter = 5;

        while inner_counter > 0 {
            println!("  内层循环值: {}", inner_counter);
            if inner_counter == 3 && count == 1 {
                break 'outer_loop; // 直接跳出标签指向的外层循环
            }
            inner_counter -= 1;
        }

        count += 1;
        if count >= 3 {
            break; // 普通 break，只跳出当前层循环
        }
    }
    println!("已跳出所有循环。");
}
