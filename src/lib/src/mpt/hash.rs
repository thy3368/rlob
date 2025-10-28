/// Keccak256 hash function for Ethereum
///
/// For now, we'll use a simplified version for demonstration.
/// In production, use a proper crypto library.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Compute Keccak256 hash (simplified version for MVP)
///
/// Note: This is NOT the real Keccak256! For production use,
/// integrate with a proper crypto library like tiny-keccak.
///
/// For demonstration purposes, we use a simple hash function.
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    // WARNING: This is a placeholder!
    // Real implementation should use proper Keccak256
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    let hash_val = hasher.finish();

    let mut result = [0u8; 32];
    result[0..8].copy_from_slice(&hash_val.to_le_bytes());

    // Fill rest with deterministic pattern
    for i in 8..32 {
        result[i] = ((hash_val >> ((i - 8) % 8)) & 0xFF) as u8;
    }

    result
}

/// Convert hash to hex string
pub fn hash_to_hex(hash: &[u8]) -> String {
    hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keccak256() {
        let data = b"hello world";
        let hash1 = keccak256(data);
        let hash2 = keccak256(data);

        // Same input should produce same output
        assert_eq!(hash1, hash2);

        // Different input should produce different output
        let hash3 = keccak256(b"hello world!");
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_to_hex() {
        let hash = [0x12, 0x34, 0x56, 0x78];
        let hex = hash_to_hex(&hash);
        assert_eq!(hex, "12345678");
    }
}
