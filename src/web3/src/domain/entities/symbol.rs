use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Symbol represents a trading pair (e.g., "BTCUSDT")
/// This is a value object in DDD terminology
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(String);

impl Symbol {
    /// Create a new symbol, converting to uppercase
    pub fn new(symbol: impl Into<String>) -> Self {
        Symbol(symbol.into().to_uppercase())
    }

    /// Get the symbol as a string slice
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Symbol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Symbol::new(s)
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Symbol::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol::new("btcusdt");
        assert_eq!(symbol.as_str(), "BTCUSDT");
    }

    #[test]
    fn test_symbol_from_str() {
        let symbol: Symbol = "ethusdt".into();
        assert_eq!(symbol.as_str(), "ETHUSDT");
    }
}
