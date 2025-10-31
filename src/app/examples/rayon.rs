use rayon::prelude::*;
use std::time::Instant;

fn main() {
    let v: Vec<i64> = (1..100_000_000).collect();

    // 顺序计算
    let start = Instant::now();
    let sum_seq: i64 = v.iter().sum();
    let seq_duration = start.elapsed();

    // 并行计算
    let start = Instant::now();
    let sum_par: i64 = v.par_iter().sum(); // 关键改动：`iter()` -> `par_iter()`
    let par_duration = start.elapsed();

    println!("顺序求和结果: {}, 耗时: {:?}", sum_seq, seq_duration);
    println!("并行求和结果: {}, 耗时: {:?}", sum_par, par_duration);
    // 典型输出可能是：
    // 顺序求和结果: 4999999950000000, 耗时: 67.672ms
    // 并行求和结果: 4999999950000000, 耗时: 29.368ms
}