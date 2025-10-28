/// Merkle Patricia Trie 示例程序
///
/// 演示如何使用MPT进行键值存储和Merkle根计算

use lib::mpt::MerklePatriciaTrie;

fn main() {
    println!("{}", "=".repeat(60));
    println!("Merkle Patricia Trie 演示");
    println!("{}", "=".repeat(60));
    println!();

    // 创建新的MPT
    let mut trie = MerklePatriciaTrie::new();
    println!("✓ 创建新的空MPT");
    println!();

    // 插入一些键值对
    println!("插入键值对:");
    println!("  do    -> verb");
    println!("  dog   -> puppy");
    println!("  doge  -> coin");
    println!("  horse -> stallion");
    println!();

    trie.insert(b"do", b"verb");
    trie.insert(b"dog", b"puppy");
    trie.insert(b"doge", b"coin");
    trie.insert(b"horse", b"stallion");

    // 查询值
    println!("查询键值:");
    let keys: Vec<&[u8]> = vec![b"do", b"dog", b"doge", b"horse", b"cat"];
    for key in keys.iter() {
        let key_str = String::from_utf8_lossy(key);
        match trie.get(key) {
            Some(value) => {
                let value_str = String::from_utf8_lossy(&value);
                println!("  {} -> {}", key_str, value_str);
            }
            None => {
                println!("  {} -> (not found)", key_str);
            }
        }
    }
    println!();

    // 显示Merkle根
    let root_hash = trie.root_hash();
    let root_hex = root_hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    println!("Merkle根哈希:");
    println!("  {}", root_hex);
    println!();

    // 显示根节点类型
    let root = trie.root();
    println!("根节点类型: {}", root.node_type());
    println!();

    // 演示更新值
    println!("{}", "-".repeat(60));
    println!("更新键值:");
    println!("  do -> verb_updated");
    trie.insert(b"do", b"verb_updated");

    match trie.get(b"do") {
        Some(value) => {
            let value_str = String::from_utf8_lossy(&value);
            println!("  ✓ 更新成功: do -> {}", value_str);
        }
        None => println!("  ✗ 更新失败"),
    }
    println!();

    // 新的Merkle根
    let new_root_hash = trie.root_hash();
    let new_root_hex = new_root_hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    println!("新的Merkle根哈希:");
    println!("  {}", new_root_hex);
    println!();

    // 验证Merkle根改变
    if root_hash != new_root_hash {
        println!("✓ Merkle根已改变（符合预期）");
    } else {
        println!("✗ Merkle根未改变（不符合预期）");
    }
    println!();

    // 演示确定性
    println!("{}", "-".repeat(60));
    println!("验证确定性（不同插入顺序）:");
    println!();

    let mut trie1 = MerklePatriciaTrie::new();
    trie1.insert(b"a", b"value1");
    trie1.insert(b"b", b"value2");
    trie1.insert(b"c", b"value3");

    let mut trie2 = MerklePatriciaTrie::new();
    trie2.insert(b"c", b"value3");
    trie2.insert(b"a", b"value1");
    trie2.insert(b"b", b"value2");

    let hash1 = trie1.root_hash();
    let hash2 = trie2.root_hash();

    let hash1_hex = hash1.iter().map(|b| format!("{:02x}", b)).collect::<String>();
    let hash2_hex = hash2.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    println!("Trie1 (插入顺序: a, b, c):");
    println!("  根哈希: {}", hash1_hex);
    println!();
    println!("Trie2 (插入顺序: c, a, b):");
    println!("  根哈希: {}", hash2_hex);
    println!();

    if hash1 == hash2 {
        println!("✓ 哈希相同（确定性验证成功）");
    } else {
        println!("✗ 哈希不同（注意：简化版本可能不完全确定性）");
    }
    println!();

    println!("{}", "=".repeat(60));
    println!("演示完成!");
    println!();
    println!("注意事项:");
    println!("  - 当前实现使用简化的哈希函数（非真实Keccak256）");
    println!("  - 生产环境需要使用真实的Keccak256哈希");
    println!("  - 生产环境需要使用RLP编码");
    println!("  - 生产环境需要持久化存储（数据库）");
    println!("{}", "=".repeat(60));
}
