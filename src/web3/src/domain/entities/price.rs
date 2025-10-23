use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Price represents a decimal price value
/// Using f64 for low-latency performance (cache-aligned)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Price(f64);

impl Price {
    /// Create a new price
    #[inline]
    pub fn new(value: f64) -> Self {
        Price(value)
    }

    /// Get the price value
    #[inline]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if price is positive
    #[inline]
    pub fn is_positive(&self) -> bool {
        self.0 > 0.0
    }
}

impl Display for Price {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.8}", self.0)
    }
}

impl From<f64> for Price {
    fn from(value: f64) -> Self {
        Price::new(value)
    }
}

/// Quantity represents a decimal quantity value
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quantity(f64);

impl Quantity {
    /// Create a new quantity
    #[inline]
    pub fn new(value: f64) -> Self {
        Quantity(value)
    }

    /// Get the quantity value
    #[inline]
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Check if quantity is positive
    #[inline]
    pub fn is_positive(&self) -> bool {
        self.0 > 0.0
    }
}

impl Display for Quantity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.8}", self.0)
    }
}

impl From<f64> for Quantity {
    fn from(value: f64) -> Self {
        Quantity::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_creation() {
        let price = Price::new(50000.12345678);
        assert_eq!(price.value(), 50000.12345678);
    }

    #[test]
    fn test_price_display() {
        let price = Price::new(50000.12345678);
        assert_eq!(format!("{}", price), "50000.12345678");
    }

    #[test]
    fn test_quantity_positive() {
        let qty = Quantity::new(1.5);
        assert!(qty.is_positive());
    }
}
