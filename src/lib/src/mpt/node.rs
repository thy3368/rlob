/// MPT Node Types
///
/// Ethereum MPT has 4 types of nodes:
/// 1. Branch Node: 17 items (16 hex + 1 value)
/// 2. Extension Node: 2 items [encoded_path, child_hash]
/// 3. Leaf Node: 2 items [encoded_path, value]
/// 4. Empty Node: null

use std::fmt;

/// Node types in Merkle Patricia Trie
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// Empty node (null)
    Empty,

    /// Leaf node: [encoded_path, value]
    /// - encoded_path: nibbles with terminator
    /// - value: actual data stored
    Leaf {
        path: Vec<u8>,  // Nibbles (hex digits)
        value: Vec<u8>,
    },

    /// Extension node: [encoded_path, child_hash]
    /// - encoded_path: common path prefix
    /// - child_hash: hash of child node
    Extension {
        path: Vec<u8>,     // Nibbles (hex digits)
        child_hash: Vec<u8>, // Hash of child node
    },

    /// Branch node: [v0, v1, ..., v15, value]
    /// - v0-v15: hashes of 16 possible children (for hex digits 0-F)
    /// - value: optional value stored at this node
    Branch {
        children: [Option<Vec<u8>>; 16], // 16 children for hex digits
        value: Option<Vec<u8>>,           // Optional value at this branch
    },
}

impl Node {
    /// Create a new empty node
    pub fn empty() -> Self {
        Node::Empty
    }

    /// Create a new leaf node
    pub fn leaf(path: Vec<u8>, value: Vec<u8>) -> Self {
        Node::Leaf { path, value }
    }

    /// Create a new extension node
    pub fn extension(path: Vec<u8>, child_hash: Vec<u8>) -> Self {
        Node::Extension { path, child_hash }
    }

    /// Create a new branch node
    pub fn branch() -> Self {
        Node::Branch {
            children: Default::default(),
            value: None,
        }
    }

    /// Check if node is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, Node::Empty)
    }

    /// Get node type as string
    pub fn node_type(&self) -> &str {
        match self {
            Node::Empty => "Empty",
            Node::Leaf { .. } => "Leaf",
            Node::Extension { .. } => "Extension",
            Node::Branch { .. } => "Branch",
        }
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Node::Empty => write!(f, "Empty"),
            Node::Leaf { path, value } => {
                write!(f, "Leaf(path: {:?}, value: {:?})", path, value)
            }
            Node::Extension { path, child_hash } => {
                write!(f, "Extension(path: {:?}, child: {:?})", path, child_hash)
            }
            Node::Branch { children, value } => {
                let child_count = children.iter().filter(|c| c.is_some()).count();
                write!(f, "Branch(children: {}, value: {:?})", child_count, value)
            }
        }
    }
}

/// Node type enum for pattern matching
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    Empty,
    Leaf,
    Extension,
    Branch,
}

impl From<&Node> for NodeType {
    fn from(node: &Node) -> Self {
        match node {
            Node::Empty => NodeType::Empty,
            Node::Leaf { .. } => NodeType::Leaf,
            Node::Extension { .. } => NodeType::Extension,
            Node::Branch { .. } => NodeType::Branch,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let empty = Node::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.node_type(), "Empty");

        let leaf = Node::leaf(vec![1, 2, 3], vec![4, 5, 6]);
        assert_eq!(leaf.node_type(), "Leaf");

        let ext = Node::extension(vec![1, 2], vec![7, 8, 9]);
        assert_eq!(ext.node_type(), "Extension");

        let branch = Node::branch();
        assert_eq!(branch.node_type(), "Branch");
    }

    #[test]
    fn test_node_type_conversion() {
        let leaf = Node::leaf(vec![1], vec![2]);
        let node_type: NodeType = (&leaf).into();
        assert_eq!(node_type, NodeType::Leaf);
    }
}
