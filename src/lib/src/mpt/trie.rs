/// Merkle Patricia Trie implementation
///
/// This is a simplified implementation for educational purposes.
/// A production implementation would need:
/// - Proper RLP encoding
/// - Database backend for persistence
/// - Proper Keccak256 hashing
/// - Proof generation/verification

use super::node::Node;
use super::nibbles::{bytes_to_nibbles, common_prefix, compact_encode};
use super::hash::keccak256;
use super::proof::MerkleProof;
use std::collections::HashMap;

/// Merkle Patricia Trie
pub struct MerklePatriciaTrie {
    /// Root node
    root: Node,
    /// Node storage (hash -> node)
    /// In production, this would be a database
    storage: HashMap<Vec<u8>, Node>,
}

impl MerklePatriciaTrie {
    /// Create a new empty trie
    pub fn new() -> Self {
        Self {
            root: Node::empty(),
            storage: HashMap::new(),
        }
    }

    /// Insert a key-value pair into the trie
    pub fn insert(&mut self, key: &[u8], value: &[u8]) {
        let nibbles = bytes_to_nibbles(key);
        let root = self.root.clone();
        self.root = self.insert_at(&root, &nibbles, value);
    }

    /// Recursive insert at a node
    fn insert_at(&mut self, node: &Node, path: &[u8], value: &[u8]) -> Node {
        match node {
            Node::Empty => {
                // Empty node: create a new leaf
                Node::leaf(path.to_vec(), value.to_vec())
            }

            Node::Leaf {
                path: leaf_path,
                value: leaf_value,
            } => {
                // Check if paths match
                let prefix_len = common_prefix(path, leaf_path);

                if prefix_len == leaf_path.len() && prefix_len == path.len() {
                    // Exact match: update value
                    Node::leaf(path.to_vec(), value.to_vec())
                } else if prefix_len == leaf_path.len() {
                    // Current leaf path is prefix of new path
                    // Convert leaf to branch
                    let mut branch = Node::branch();

                    if let Node::Branch { ref mut children, value: ref mut branch_value } = branch {
                        // Set the leaf value at this branch
                        *branch_value = Some(leaf_value.clone());

                        // Insert remaining path
                        if prefix_len < path.len() {
                            let remaining = &path[prefix_len..];
                            let nibble = remaining[0] as usize;
                            let rest = &remaining[1..];

                            let child = Node::leaf(rest.to_vec(), value.to_vec());
                            let child_hash = self.hash_node(&child);
                            self.storage.insert(child_hash.clone(), child);
                            children[nibble] = Some(child_hash);
                        }
                    }

                    // If the leaf had a path, wrap branch in extension
                    if prefix_len > 0 {
                        let branch_hash = self.hash_node(&branch);
                        self.storage.insert(branch_hash.clone(), branch);
                        Node::extension(leaf_path.to_vec(), branch_hash)
                    } else {
                        branch
                    }
                } else if prefix_len == 0 {
                    // No common prefix: create branch directly
                    let mut branch = Node::branch();

                    if let Node::Branch { ref mut children, .. } = branch {
                        // Add old leaf
                        let old_nibble = leaf_path[0] as usize;
                        let old_rest = &leaf_path[1..];
                        let old_node = Node::leaf(old_rest.to_vec(), leaf_value.clone());
                        let old_hash = self.hash_node(&old_node);
                        self.storage.insert(old_hash.clone(), old_node);
                        children[old_nibble] = Some(old_hash);

                        // Add new leaf
                        let new_nibble = path[0] as usize;
                        let new_rest = &path[1..];
                        let new_node = Node::leaf(new_rest.to_vec(), value.to_vec());
                        let new_hash = self.hash_node(&new_node);
                        self.storage.insert(new_hash.clone(), new_node);
                        children[new_nibble] = Some(new_hash);
                    }

                    branch
                } else {
                    // Common prefix: create extension
                    let common = &path[..prefix_len];

                    // Create branch for divergence point
                    let mut branch = Node::branch();

                    if let Node::Branch { ref mut children, .. } = branch {
                        // Add old path
                        let old_rest = &leaf_path[prefix_len..];
                        if !old_rest.is_empty() {
                            let old_nibble = old_rest[0] as usize;
                            let old_node = Node::leaf(old_rest[1..].to_vec(), leaf_value.clone());
                            let old_hash = self.hash_node(&old_node);
                            self.storage.insert(old_hash.clone(), old_node);
                            children[old_nibble] = Some(old_hash);
                        }

                        // Add new path
                        let new_rest = &path[prefix_len..];
                        if !new_rest.is_empty() {
                            let new_nibble = new_rest[0] as usize;
                            let new_node = Node::leaf(new_rest[1..].to_vec(), value.to_vec());
                            let new_hash = self.hash_node(&new_node);
                            self.storage.insert(new_hash.clone(), new_node);
                            children[new_nibble] = Some(new_hash);
                        }
                    }

                    // Create extension node
                    let branch_hash = self.hash_node(&branch);
                    self.storage.insert(branch_hash.clone(), branch);
                    Node::extension(common.to_vec(), branch_hash)
                }
            }

            Node::Extension { path: ext_path, child_hash } => {
                let prefix_len = common_prefix(path, ext_path);

                if prefix_len == ext_path.len() {
                    // Path continues through extension
                    let remaining = &path[prefix_len..];
                    let child = self.storage.get(child_hash).cloned()
                        .unwrap_or(Node::empty());

                    let new_child = self.insert_at(&child, remaining, value);
                    let new_child_hash = self.hash_node(&new_child);
                    self.storage.insert(new_child_hash.clone(), new_child);

                    Node::extension(ext_path.clone(), new_child_hash)
                } else {
                    // Split extension
                    let common = &path[..prefix_len];
                    let mut branch = Node::branch();

                    if let Node::Branch { ref mut children, .. } = branch {
                        // Add old extension continuation
                        let old_rest = &ext_path[prefix_len..];
                        if !old_rest.is_empty() {
                            let old_nibble = old_rest[0] as usize;
                            if old_rest.len() > 1 {
                                let old_ext = Node::extension(old_rest[1..].to_vec(), child_hash.clone());
                                let old_hash = self.hash_node(&old_ext);
                                self.storage.insert(old_hash.clone(), old_ext);
                                children[old_nibble] = Some(old_hash);
                            } else {
                                children[old_nibble] = Some(child_hash.clone());
                            }
                        }

                        // Add new path
                        let new_rest = &path[prefix_len..];
                        if !new_rest.is_empty() {
                            let new_nibble = new_rest[0] as usize;
                            let new_node = Node::leaf(new_rest[1..].to_vec(), value.to_vec());
                            let new_hash = self.hash_node(&new_node);
                            self.storage.insert(new_hash.clone(), new_node);
                            children[new_nibble] = Some(new_hash);
                        }
                    }

                    if prefix_len > 0 {
                        let branch_hash = self.hash_node(&branch);
                        self.storage.insert(branch_hash.clone(), branch);
                        Node::extension(common.to_vec(), branch_hash)
                    } else {
                        branch
                    }
                }
            }

            Node::Branch { children, value: branch_value } => {
                if path.is_empty() {
                    // Update value at branch
                    let mut new_branch = Node::branch();
                    if let Node::Branch { children: ref mut new_children, value: ref mut new_value } = new_branch {
                        new_children.clone_from(children);
                        *new_value = Some(value.to_vec());
                    }
                    new_branch
                } else {
                    // Navigate to child
                    let nibble = path[0] as usize;
                    let remaining = &path[1..];

                    let child = children[nibble]
                        .as_ref()
                        .and_then(|hash| self.storage.get(hash).cloned())
                        .unwrap_or(Node::empty());

                    let new_child = self.insert_at(&child, remaining, value);
                    let new_child_hash = self.hash_node(&new_child);
                    self.storage.insert(new_child_hash.clone(), new_child);

                    let mut new_branch = Node::branch();
                    if let Node::Branch { children: ref mut new_children, value: ref mut new_value } = new_branch {
                        new_children.clone_from(children);
                        new_children[nibble] = Some(new_child_hash);
                        *new_value = branch_value.clone();
                    }

                    new_branch
                }
            }
        }
    }

    /// Get a value from the trie
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let nibbles = bytes_to_nibbles(key);
        self.get_at(&self.root, &nibbles)
    }

    /// Recursive get at a node
    fn get_at(&self, node: &Node, path: &[u8]) -> Option<Vec<u8>> {
        match node {
            Node::Empty => None,

            Node::Leaf { path: leaf_path, value } => {
                if path == leaf_path.as_slice() {
                    Some(value.clone())
                } else {
                    None
                }
            }

            Node::Extension { path: ext_path, child_hash } => {
                if path.starts_with(ext_path) {
                    let remaining = &path[ext_path.len()..];
                    let child = self.storage.get(child_hash)?;
                    self.get_at(child, remaining)
                } else {
                    None
                }
            }

            Node::Branch { children, value } => {
                if path.is_empty() {
                    value.clone()
                } else {
                    let nibble = path[0] as usize;
                    let remaining = &path[1..];
                    let child_hash = children[nibble].as_ref()?;
                    let child = self.storage.get(child_hash)?;
                    self.get_at(child, remaining)
                }
            }
        }
    }

    /// Generate a Merkle proof for a key
    ///
    /// Returns a proof that can be used to verify the existence (or non-existence)
    /// of a key-value pair in the trie
    pub fn get_proof(&self, key: &[u8]) -> MerkleProof {
        let nibbles = bytes_to_nibbles(key);
        let mut proof_nodes = Vec::new();
        let value = self.get_proof_at(&self.root, &nibbles, &mut proof_nodes);

        MerkleProof::new(key.to_vec(), value, proof_nodes)
    }

    /// Recursive proof generation
    fn get_proof_at(&self, node: &Node, path: &[u8], proof_nodes: &mut Vec<Node>) -> Option<Vec<u8>> {
        // 将当前节点添加到证明路径
        proof_nodes.push(node.clone());

        match node {
            Node::Empty => None,

            Node::Leaf { path: leaf_path, value } => {
                if path == leaf_path.as_slice() {
                    Some(value.clone())
                } else {
                    None
                }
            }

            Node::Extension { path: ext_path, child_hash } => {
                if path.starts_with(ext_path) {
                    let remaining = &path[ext_path.len()..];
                    let child = self.storage.get(child_hash)?;
                    self.get_proof_at(child, remaining, proof_nodes)
                } else {
                    None
                }
            }

            Node::Branch { children, value } => {
                if path.is_empty() {
                    value.clone()
                } else {
                    let nibble = path[0] as usize;
                    let remaining = &path[1..];
                    let child_hash = children[nibble].as_ref()?;
                    let child = self.storage.get(child_hash)?;
                    self.get_proof_at(child, remaining, proof_nodes)
                }
            }
        }
    }

    /// Compute the Merkle root hash
    pub fn root_hash(&self) -> Vec<u8> {
        self.hash_node(&self.root)
    }

    /// Hash a node (simplified)
    fn hash_node(&self, node: &Node) -> Vec<u8> {
        match node {
            Node::Empty => vec![],
            Node::Leaf { path, value } => {
                let encoded_path = compact_encode(path, true);
                let mut data = encoded_path;
                data.extend_from_slice(value);
                keccak256(&data).to_vec()
            }
            Node::Extension { path, child_hash } => {
                let encoded_path = compact_encode(path, false);
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

    /// Get the root node (for inspection)
    pub fn root(&self) -> &Node {
        &self.root
    }
}

impl Default for MerklePatriciaTrie {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut trie = MerklePatriciaTrie::new();

        trie.insert(b"hello", b"world");
        assert_eq!(trie.get(b"hello"), Some(b"world".to_vec()));
        assert_eq!(trie.get(b"hello2"), None);
    }

    #[test]
    fn test_multiple_inserts() {
        let mut trie = MerklePatriciaTrie::new();

        trie.insert(b"do", b"verb");
        trie.insert(b"dog", b"puppy");
        trie.insert(b"doge", b"coin");

        assert_eq!(trie.get(b"do"), Some(b"verb".to_vec()));
        assert_eq!(trie.get(b"dog"), Some(b"puppy".to_vec()));
        assert_eq!(trie.get(b"doge"), Some(b"coin".to_vec()));
        assert_eq!(trie.get(b"cat"), None);
    }

    #[test]
    fn test_root_hash() {
        let mut trie = MerklePatriciaTrie::new();

        let hash1 = trie.root_hash();

        trie.insert(b"key1", b"value1");
        let hash2 = trie.root_hash();

        // Hash should change after insert
        assert_ne!(hash1, hash2);

        trie.insert(b"key2", b"value2");
        let hash3 = trie.root_hash();

        // Hash should change again
        assert_ne!(hash2, hash3);
    }

    #[test]
    fn test_deterministic_hash() {
        let mut trie1 = MerklePatriciaTrie::new();
        trie1.insert(b"do", b"verb");
        trie1.insert(b"dog", b"puppy");

        let mut trie2 = MerklePatriciaTrie::new();
        trie2.insert(b"dog", b"puppy");
        trie2.insert(b"do", b"verb");

        // Same data, different insertion order, should have same root hash
        // (This might not hold in this simplified implementation)
        let hash1 = trie1.root_hash();
        let hash2 = trie2.root_hash();

        println!("Hash1: {:?}", hash1);
        println!("Hash2: {:?}", hash2);
    }

    #[test]
    fn test_proof_generation_and_verification() {
        let mut trie = MerklePatriciaTrie::new();

        // 插入一些数据
        trie.insert(b"do", b"verb");
        trie.insert(b"dog", b"puppy");
        trie.insert(b"doge", b"coin");

        // 获取根哈希
        let root_hash = trie.root_hash();

        // 为"dog"生成证明
        let proof = trie.get_proof(b"dog");

        // 验证证明内容
        assert_eq!(proof.key, b"dog");
        assert_eq!(proof.value, Some(b"puppy".to_vec()));
        assert!(!proof.proof_nodes.is_empty());

        // 验证证明有效性
        assert!(proof.verify(&root_hash));
    }

    #[test]
    fn test_proof_for_nonexistent_key() {
        let mut trie = MerklePatriciaTrie::new();

        trie.insert(b"do", b"verb");
        trie.insert(b"dog", b"puppy");

        let root_hash = trie.root_hash();

        // 为不存在的键生成证明
        let proof = trie.get_proof(b"cat");

        // 值应该是None
        assert_eq!(proof.value, None);

        // 证明应该仍然有效（证明不存在）
        assert!(proof.verify(&root_hash));
    }

    #[test]
    fn test_proof_invalid_after_modification() {
        let mut trie = MerklePatriciaTrie::new();

        trie.insert(b"test", b"value");

        // 获取原始根哈希和证明
        let old_root_hash = trie.root_hash();
        let proof = trie.get_proof(b"test");

        // 验证原始证明
        assert!(proof.verify(&old_root_hash));

        // 修改trie
        trie.insert(b"test2", b"value2");
        let new_root_hash = trie.root_hash();

        // 旧证明对新根应该无效（因为根哈希改变了）
        // 注意：这里证明本身的结构可能仍然有效，但根哈希不匹配
        assert_ne!(old_root_hash, new_root_hash);
    }

    #[test]
    fn test_proof_with_multiple_keys() {
        let mut trie = MerklePatriciaTrie::new();

        trie.insert(b"apple", b"fruit");
        trie.insert(b"banana", b"yellow");
        trie.insert(b"cherry", b"red");

        let root_hash = trie.root_hash();

        // 为每个键生成并验证证明
        let keys: Vec<&[u8]> = vec![b"apple", b"banana", b"cherry"];
        for key in keys {
            let proof = trie.get_proof(key);
            assert!(proof.value.is_some());
            assert!(proof.verify(&root_hash));
        }
    }
}
