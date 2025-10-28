/// Merkle Patricia Trie 证明演示程序
///
/// 演示如何生成和验证Merkle证明

use lib::mpt::{MerklePatriciaTrie, MerkleProof};

fn print_separator() {
    println!("{}", "=".repeat(70));
}

fn print_subseparator() {
    println!("{}", "-".repeat(70));
}

fn print_proof_info(proof: &MerkleProof, label: &str) {
    println!("\n{} 证明信息:", label);
    println!("  键: {}", String::from_utf8_lossy(&proof.key));
    match &proof.value {
        Some(v) => println!("  值: {}", String::from_utf8_lossy(v)),
        None => println!("  值: (不存在)"),
    }
    println!("  证明路径节点数: {}", proof.proof_nodes.len());
    println!("  节点类型: {:?}",
        proof.proof_nodes.iter()
            .map(|n| n.node_type())
            .collect::<Vec<_>>()
    );
}

fn main() {
    print_separator();
    println!("Merkle Patricia Trie 证明系统演示");
    print_separator();
    println!();

    // 场景1: 基本证明生成和验证
    println!("场景1: 基本证明生成和验证");
    print_subseparator();

    let mut trie = MerklePatriciaTrie::new();

    println!("\n插入数据:");
    let data: Vec<(&[u8], &[u8])> = vec![
        (b"apple" as &[u8], b"A red fruit" as &[u8]),
        (b"banana", b"A yellow fruit"),
        (b"cherry", b"A red berry"),
        (b"date", b"A sweet fruit"),
    ];

    for (key, value) in &data {
        trie.insert(key, value);
        println!("  {} -> {}",
            String::from_utf8_lossy(key),
            String::from_utf8_lossy(value)
        );
    }

    let root_hash = trie.root_hash();
    let root_hex = root_hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    println!("\nMerkle根哈希: {}", &root_hex[..16]);
    println!("(简化显示前16个字符)");

    // 为"banana"生成证明
    println!("\n为键 'banana' 生成证明...");
    let proof = trie.get_proof(b"banana");
    print_proof_info(&proof, "banana");

    // 验证证明
    println!("\n验证证明...");
    let is_valid = proof.verify(&root_hash);
    if is_valid {
        println!("  ✓ 证明有效！值确实是: {}",
            String::from_utf8_lossy(proof.value.as_ref().unwrap())
        );
    } else {
        println!("  ✗ 证明无效");
    }

    println!();

    // 场景2: 证明键不存在
    println!("场景2: 证明键不存在");
    print_subseparator();

    println!("\n为不存在的键 'grape' 生成证明...");
    let nonexistent_proof = trie.get_proof(b"grape");
    print_proof_info(&nonexistent_proof, "grape");

    println!("\n验证不存在证明...");
    let is_valid = nonexistent_proof.verify(&root_hash);
    if is_valid {
        println!("  ✓ 证明有效！键确实不存在");
    } else {
        println!("  ✗ 证明无效");
    }

    println!();

    // 场景3: 修改后证明失效
    println!("场景3: 修改后证明失效");
    print_subseparator();

    println!("\n为键 'apple' 生成原始证明...");
    let original_proof = trie.get_proof(b"apple");
    let original_root = trie.root_hash();

    println!("验证原始证明...");
    assert!(original_proof.verify(&original_root));
    println!("  ✓ 原始证明有效");

    println!("\n修改trie: 插入新键 'elderberry'...");
    trie.insert(b"elderberry", b"A dark berry");
    let new_root = trie.root_hash();

    let new_root_hex = new_root.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    println!("  新根哈希: {}", &new_root_hex[..16]);

    println!("\n用新根验证旧证明...");
    let is_valid = original_proof.verify(&new_root);
    if !is_valid {
        println!("  ✓ 如预期，旧证明对新根无效");
    } else {
        println!("  ✗ 意外：旧证明对新根仍然有效");
    }

    println!("\n生成新的证明并验证...");
    let new_proof = trie.get_proof(b"apple");
    let is_valid = new_proof.verify(&new_root);
    if is_valid {
        println!("  ✓ 新证明对新根有效");
    }

    println!();

    // 场景4: 批量验证
    println!("场景4: 批量证明验证");
    print_subseparator();

    let test_keys: Vec<&[u8]> = vec![b"apple", b"banana", b"cherry", b"date"];
    println!("\n为所有键生成并验证证明:");

    let mut all_valid = true;
    for key in test_keys {
        let proof = trie.get_proof(key);
        let is_valid = proof.verify(&new_root);

        let status = if is_valid { "✓" } else { "✗" };
        println!("  {} {}: {}",
            status,
            String::from_utf8_lossy(key),
            if is_valid { "有效" } else { "无效" }
        );

        all_valid = all_valid && is_valid;
    }

    if all_valid {
        println!("\n✓ 所有证明都有效！");
    }

    println!();

    // 场景5: 轻客户端验证示例
    println!("场景5: 轻客户端验证场景");
    print_subseparator();
    println!("\n模拟场景：");
    println!("  - 全节点拥有完整的trie");
    println!("  - 轻客户端只知道根哈希");
    println!("  - 轻客户端想验证 'date' 的值");

    // 全节点：生成证明
    println!("\n全节点: 为 'date' 生成证明...");
    let light_proof = trie.get_proof(b"date");

    // 轻客户端：只有根哈希
    let light_client_root = new_root.clone();

    println!("轻客户端: 使用根哈希验证证明...");
    println!("  已知根哈希: {}", &new_root_hex[..16]);

    // 验证
    let is_valid = light_proof.verify(&light_client_root);

    if is_valid {
        println!("  ✓ 验证成功！");
        println!("  确认值: {}",
            String::from_utf8_lossy(light_proof.value.as_ref().unwrap())
        );
        println!("\n  轻客户端无需下载整个trie，");
        println!("  仅通过 {} 个节点的证明即可验证！",
            light_proof.proof_nodes.len()
        );
    } else {
        println!("  ✗ 验证失败");
    }

    println!();

    // 场景6: 篡改检测
    println!("场景6: 篡改检测");
    print_subseparator();

    println!("\n生成合法证明...");
    let valid_proof = trie.get_proof(b"cherry");
    println!("  原始值: {}",
        String::from_utf8_lossy(valid_proof.value.as_ref().unwrap())
    );

    // 创建一个篡改的证明
    println!("\n尝试篡改证明（修改值）...");
    let tampered_proof = MerkleProof::new(
        b"cherry".to_vec(),
        Some(b"TAMPERED VALUE".to_vec()),  // 篡改的值
        valid_proof.proof_nodes.clone(),
    );

    println!("验证篡改的证明...");
    let is_valid = tampered_proof.verify(&new_root);
    if !is_valid {
        println!("  ✓ 检测到篡改！证明无效");
    } else {
        println!("  ✗ 未能检测到篡改（不应该发生）");
    }

    println!();
    print_separator();
    println!("演示完成！");
    println!();

    println!("总结:");
    println!("  ✓ Merkle证明可以高效验证键值对的存在性");
    println!("  ✓ 可以证明键的不存在");
    println!("  ✓ 任何篡改都会导致验证失败");
    println!("  ✓ 轻客户端无需完整数据即可验证");
    println!("  ✓ 证明大小与trie大小对数相关（高效）");

    print_separator();
}
