/// Merkle证明相关数据结构和验证逻辑
///
/// Merkle证明允许在不访问整个trie的情况下验证某个键值对是否存在

use super::node::Node;
use super::nibbles::bytes_to_nibbles;
use super::hash::keccak256;

/// Merkle证明
#[derive(Debug, Clone, PartialEq)]
pub struct MerkleProof {
    /// 被证明的键
    pub key: Vec<u8>,
    /// 被证明的值（如果存在）
    pub value: Option<Vec<u8>>,
    /// 证明路径上的节点列表（从根到叶）
    pub proof_nodes: Vec<Node>,
}

impl MerkleProof {
    /// 创建新的Merkle证明
    pub fn new(key: Vec<u8>, value: Option<Vec<u8>>, proof_nodes: Vec<Node>) -> Self {
        Self {
            key,
            value,
            proof_nodes,
        }
    }

    /// 验证Merkle证明
    ///
    /// # 参数
    /// - `root_hash`: 已知的根哈希
    ///
    /// # 返回
    /// - `true`: 证明有效
    /// - `false`: 证明无效
    pub fn verify(&self, root_hash: &[u8]) -> bool {
        if self.proof_nodes.is_empty() {
            return false;
        }

        let nibbles = bytes_to_nibbles(&self.key);
        self.verify_at(&self.proof_nodes[0], &nibbles, 0, root_hash)
    }

    /// 递归验证节点
    fn verify_at(&self, node: &Node, path: &[u8], node_index: usize, expected_hash: &[u8]) -> bool {
        // 验证当前节点的哈希
        let node_hash = self.hash_node(node);
        if node_hash != expected_hash {
            return false;
        }

        match node {
            Node::Empty => {
                // 空节点：值应该不存在
                self.value.is_none()
            }

            Node::Leaf { path: leaf_path, value: leaf_value } => {
                // 叶节点：路径和值都应该匹配
                if path != leaf_path.as_slice() {
                    // 路径不匹配：这证明了键不存在
                    return self.value.is_none();
                }
                match (&self.value, leaf_value) {
                    (Some(expected), actual) => expected == actual,
                    (None, _) => false,
                }
            }

            Node::Extension { path: ext_path, child_hash } => {
                // 扩展节点：路径应该匹配前缀，继续验证子节点
                if !path.starts_with(ext_path) {
                    // 路径不匹配：这证明了键不存在
                    return self.value.is_none();
                }

                let remaining = &path[ext_path.len()..];
                let next_index = node_index + 1;

                if next_index >= self.proof_nodes.len() {
                    return false;
                }

                self.verify_at(&self.proof_nodes[next_index], remaining, next_index, child_hash)
            }

            Node::Branch { children, value: branch_value } => {
                if path.is_empty() {
                    // 路径到达分支节点：验证值
                    match (&self.value, branch_value) {
                        (Some(expected), Some(actual)) => expected == actual,
                        (None, None) => true,
                        _ => false,
                    }
                } else {
                    // 继续沿路径前进
                    let nibble = path[0] as usize;
                    let remaining = &path[1..];

                    match &children[nibble] {
                        Some(child_hash) => {
                            let next_index = node_index + 1;
                            if next_index >= self.proof_nodes.len() {
                                return false;
                            }
                            self.verify_at(&self.proof_nodes[next_index], remaining, next_index, child_hash)
                        }
                        None => self.value.is_none(), // 子节点不存在，值应该不存在
                    }
                }
            }
        }
    }

    /// 计算节点哈希（与trie中的实现相同）
    fn hash_node(&self, node: &Node) -> Vec<u8> {
        match node {
            Node::Empty => vec![],
            Node::Leaf { path, value } => {
                let encoded_path = super::nibbles::compact_encode(path, true);
                let mut data = encoded_path;
                data.extend_from_slice(value);
                keccak256(&data).to_vec()
            }
            Node::Extension { path, child_hash } => {
                let encoded_path = super::nibbles::compact_encode(path, false);
                let mut data = encoded_path;
                data.extend_from_slice(child_hash);
                keccak256(&data).to_vec()
            }
            Node::Branch { children, value } => {
                let mut data = Vec::new();
                for child in children.iter() {
                    if let Some(hash) = child {
                        data.extend_from_slice(hash);
                    }
                }
                if let Some(v) = value {
                    data.extend_from_slice(v);
                }
                keccak256(&data).to_vec()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_proof_creation() {
        let proof = MerkleProof::new(
            b"test".to_vec(),
            Some(b"value".to_vec()),
            vec![Node::leaf(vec![1, 2, 3], b"value".to_vec())],
        );

        assert_eq!(proof.key, b"test");
        assert_eq!(proof.value, Some(b"value".to_vec()));
        assert_eq!(proof.proof_nodes.len(), 1);
    }

    #[test]
    fn test_simple_leaf_proof() {
        // 创建一个简单的叶节点证明
        let key = b"test";
        let value = b"value";
        let nibbles = bytes_to_nibbles(key);

        let leaf = Node::leaf(nibbles.clone(), value.to_vec());
        let proof = MerkleProof::new(
            key.to_vec(),
            Some(value.to_vec()),
            vec![leaf.clone()],
        );

        // 计算根哈希
        let root_hash = proof.hash_node(&leaf);

        // 验证证明
        assert!(proof.verify(&root_hash));
    }

    #[test]
    fn test_invalid_proof() {
        let key = b"test";
        let value = b"value";
        let nibbles = bytes_to_nibbles(key);

        let leaf = Node::leaf(nibbles.clone(), value.to_vec());
        let proof = MerkleProof::new(
            key.to_vec(),
            Some(b"wrong_value".to_vec()), // 错误的值
            vec![leaf.clone()],
        );

        let root_hash = proof.hash_node(&leaf);

        // 验证应该失败
        assert!(!proof.verify(&root_hash));
    }
}
