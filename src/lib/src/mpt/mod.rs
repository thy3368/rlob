/// Ethereum Merkle Patricia Trie implementation
///
/// This is a custom implementation without third-party dependencies.
///
/// References:
/// - Ethereum Yellow Paper: https://ethereum.github.io/yellowpaper/paper.pdf
/// - Ethereum Wiki: https://eth.wiki/fundamentals/patricia-tree

pub mod node;
pub mod trie;
pub mod nibbles;
pub mod hash;
pub mod proof;

pub use trie::MerklePatriciaTrie;
pub use node::{Node, NodeType};
pub use proof::MerkleProof;
