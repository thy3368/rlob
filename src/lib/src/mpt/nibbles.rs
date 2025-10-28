/// Nibbles (半字节) 处理工具
///
/// MPT使用半字节（4位）作为路径的基本单位
/// 例如: 字节 0xAB 转换为 nibbles [0xA, 0xB]

/// Convert bytes to nibbles (半字节)
///
/// # Example
/// ```
/// let bytes = vec![0xAB, 0xCD];
/// let nibbles = bytes_to_nibbles(&bytes);
/// assert_eq!(nibbles, vec![0xA, 0xB, 0xC, 0xD]);
/// ```
pub fn bytes_to_nibbles(bytes: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        nibbles.push(byte >> 4);      // 高4位
        nibbles.push(byte & 0x0F);    // 低4位
    }
    nibbles
}

/// Convert nibbles back to bytes
///
/// # Example
/// ```
/// let nibbles = vec![0xA, 0xB, 0xC, 0xD];
/// let bytes = nibbles_to_bytes(&nibbles);
/// assert_eq!(bytes, vec![0xAB, 0xCD]);
/// ```
pub fn nibbles_to_bytes(nibbles: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity((nibbles.len() + 1) / 2);

    for chunk in nibbles.chunks(2) {
        if chunk.len() == 2 {
            bytes.push((chunk[0] << 4) | chunk[1]);
        } else {
            // Odd number of nibbles, pad with 0
            bytes.push(chunk[0] << 4);
        }
    }

    bytes
}

/// Compact encoding for hex-prefix
///
/// Ethereum uses HP (Hex-Prefix) encoding:
/// - Flag for terminator (leaf vs extension)
/// - Flag for odd/even length
///
/// HP Encoding:
/// - If terminator flag is true (leaf node):
///   - odd length: [0x3, nibble0, byte(nibble1, nibble2), ...]
///   - even length: [0x20 + nibble0, byte(nibble1, nibble2), ...]
/// - If terminator flag is false (extension node):
///   - odd length: [0x1, nibble0, byte(nibble1, nibble2), ...]
///   - even length: [0x00, byte(nibble0, nibble1), ...]
pub fn compact_encode(nibbles: &[u8], is_leaf: bool) -> Vec<u8> {
    let mut encoded = Vec::new();
    let terminator = if is_leaf { 0x20 } else { 0x00 };

    if nibbles.len() % 2 == 0 {
        // Even length
        encoded.push(terminator);
        encoded.extend_from_slice(&nibbles_to_bytes(nibbles));
    } else {
        // Odd length
        let flag = terminator + 0x10 + nibbles[0];
        encoded.push(flag);
        encoded.extend_from_slice(&nibbles_to_bytes(&nibbles[1..]));
    }

    encoded
}

/// Decode compact (HP) encoding
///
/// Returns (nibbles, is_leaf)
pub fn compact_decode(encoded: &[u8]) -> (Vec<u8>, bool) {
    if encoded.is_empty() {
        return (Vec::new(), false);
    }

    let first = encoded[0];
    let is_leaf = (first & 0x20) != 0;
    let is_odd = (first & 0x10) != 0;

    let mut nibbles = Vec::new();

    if is_odd {
        // Odd length: first nibble is in lower 4 bits of first byte
        nibbles.push(first & 0x0F);
        nibbles.extend_from_slice(&bytes_to_nibbles(&encoded[1..]));
    } else {
        // Even length: skip first byte
        nibbles.extend_from_slice(&bytes_to_nibbles(&encoded[1..]));
    }

    (nibbles, is_leaf)
}

/// Find common prefix between two nibble slices
pub fn common_prefix(a: &[u8], b: &[u8]) -> usize {
    let len = a.len().min(b.len());
    for i in 0..len {
        if a[i] != b[i] {
            return i;
        }
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bytes_to_nibbles() {
        assert_eq!(bytes_to_nibbles(&[0xAB]), vec![0xA, 0xB]);
        assert_eq!(bytes_to_nibbles(&[0xAB, 0xCD]), vec![0xA, 0xB, 0xC, 0xD]);
        assert_eq!(bytes_to_nibbles(&[0x12, 0x34, 0x56]), vec![0x1, 0x2, 0x3, 0x4, 0x5, 0x6]);
    }

    #[test]
    fn test_nibbles_to_bytes() {
        assert_eq!(nibbles_to_bytes(&[0xA, 0xB]), vec![0xAB]);
        assert_eq!(nibbles_to_bytes(&[0xA, 0xB, 0xC, 0xD]), vec![0xAB, 0xCD]);
        assert_eq!(nibbles_to_bytes(&[0x1, 0x2, 0x3, 0x4, 0x5, 0x6]), vec![0x12, 0x34, 0x56]);
    }

    #[test]
    fn test_compact_encode_leaf_even() {
        let nibbles = vec![0x1, 0x2, 0x3, 0x4];
        let encoded = compact_encode(&nibbles, true);
        assert_eq!(encoded, vec![0x20, 0x12, 0x34]);
    }

    #[test]
    fn test_compact_encode_leaf_odd() {
        let nibbles = vec![0x1, 0x2, 0x3];
        let encoded = compact_encode(&nibbles, true);
        assert_eq!(encoded, vec![0x31, 0x23]);
    }

    #[test]
    fn test_compact_encode_extension_even() {
        let nibbles = vec![0x1, 0x2, 0x3, 0x4];
        let encoded = compact_encode(&nibbles, false);
        assert_eq!(encoded, vec![0x00, 0x12, 0x34]);
    }

    #[test]
    fn test_compact_encode_extension_odd() {
        let nibbles = vec![0x1, 0x2, 0x3];
        let encoded = compact_encode(&nibbles, false);
        assert_eq!(encoded, vec![0x11, 0x23]);
    }

    #[test]
    fn test_compact_decode() {
        // Leaf, even length
        let (nibbles, is_leaf) = compact_decode(&[0x20, 0x12, 0x34]);
        assert_eq!(nibbles, vec![0x1, 0x2, 0x3, 0x4]);
        assert!(is_leaf);

        // Leaf, odd length
        let (nibbles, is_leaf) = compact_decode(&[0x31, 0x23]);
        assert_eq!(nibbles, vec![0x1, 0x2, 0x3]);
        assert!(is_leaf);

        // Extension, even length
        let (nibbles, is_leaf) = compact_decode(&[0x00, 0x12, 0x34]);
        assert_eq!(nibbles, vec![0x1, 0x2, 0x3, 0x4]);
        assert!(!is_leaf);

        // Extension, odd length
        let (nibbles, is_leaf) = compact_decode(&[0x11, 0x23]);
        assert_eq!(nibbles, vec![0x1, 0x2, 0x3]);
        assert!(!is_leaf);
    }

    #[test]
    fn test_common_prefix() {
        assert_eq!(common_prefix(&[1, 2, 3], &[1, 2, 4]), 2);
        assert_eq!(common_prefix(&[1, 2, 3], &[1, 2, 3]), 3);
        assert_eq!(common_prefix(&[1, 2, 3], &[4, 5, 6]), 0);
        assert_eq!(common_prefix(&[1, 2], &[1, 2, 3, 4]), 2);
    }
}
