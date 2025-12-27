//! Common value types used across the Enablement Layer.

use serde::{Deserialize, Serialize};

/// Monetary value with currency code.
///
/// Uses minor units (cents) to avoid floating-point precision issues.
///
/// # Example
/// ```
/// use creto_common::Money;
///
/// let price = Money::usd(1999); // $19.99
/// assert_eq!(price.to_major_units(), 19.99);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// Amount in minor units (e.g., cents for USD).
    pub amount: i64,
    /// ISO 4217 currency code.
    pub currency: Currency,
}

impl Money {
    /// Create a new Money value.
    pub fn new(amount: i64, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Create USD money (amount in cents).
    pub fn usd(cents: i64) -> Self {
        Self::new(cents, Currency::USD)
    }

    /// Convert to major units (e.g., dollars).
    pub fn to_major_units(&self) -> f64 {
        self.amount as f64 / self.currency.minor_unit_factor() as f64
    }

    /// Add two money values (must be same currency).
    pub fn add(&self, other: &Money) -> Result<Money, &'static str> {
        if self.currency != other.currency {
            return Err("Currency mismatch");
        }
        Ok(Money::new(self.amount + other.amount, self.currency))
    }

    /// Check if this amount is zero.
    pub fn is_zero(&self) -> bool {
        self.amount == 0
    }

    /// Check if this amount is negative.
    pub fn is_negative(&self) -> bool {
        self.amount < 0
    }
}

impl std::fmt::Display for Money {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:.2}", self.currency, self.to_major_units())
    }
}

/// ISO 4217 currency codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CAD,
    AUD,
}

impl Currency {
    /// Get the minor unit factor (e.g., 100 for USD = 100 cents per dollar).
    pub fn minor_unit_factor(&self) -> i64 {
        match self {
            Currency::JPY => 1, // Yen has no minor unit
            _ => 100,
        }
    }
}

impl std::fmt::Display for Currency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Currency::USD => write!(f, "$"),
            Currency::EUR => write!(f, "EUR"),
            Currency::GBP => write!(f, "GBP"),
            Currency::JPY => write!(f, "JPY"),
            Currency::CAD => write!(f, "CAD"),
            Currency::AUD => write!(f, "AUD"),
        }
    }
}

/// Unix timestamp in milliseconds.
///
/// Wrapper around chrono::DateTime for consistent serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Create a timestamp from the current time.
    pub fn now() -> Self {
        Self(chrono::Utc::now().timestamp_millis())
    }

    /// Create a timestamp from milliseconds since Unix epoch.
    pub fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    /// Get milliseconds since Unix epoch.
    pub fn as_millis(&self) -> i64 {
        self.0
    }

    /// Convert to chrono DateTime.
    pub fn to_datetime(&self) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::from_timestamp_millis(self.0).unwrap_or_else(chrono::Utc::now)
    }

    /// Check if this timestamp is before another.
    pub fn is_before(&self, other: &Timestamp) -> bool {
        self.0 < other.0
    }

    /// Calculate duration since another timestamp.
    pub fn duration_since(&self, other: &Timestamp) -> std::time::Duration {
        let diff = (self.0 - other.0).max(0) as u64;
        std::time::Duration::from_millis(diff)
    }
}

impl From<chrono::DateTime<chrono::Utc>> for Timestamp {
    fn from(dt: chrono::DateTime<chrono::Utc>) -> Self {
        Self(dt.timestamp_millis())
    }
}

impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_datetime().format("%Y-%m-%dT%H:%M:%S%.3fZ"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_display() {
        let money = Money::usd(1999);
        assert_eq!(money.to_string(), "$ 19.99");
    }

    #[test]
    fn test_money_add_same_currency() {
        let a = Money::usd(1000);
        let b = Money::usd(500);
        let sum = a.add(&b).unwrap();
        assert_eq!(sum.amount, 1500);
    }

    #[test]
    fn test_money_add_different_currency() {
        let a = Money::usd(1000);
        let b = Money::new(500, Currency::EUR);
        assert!(a.add(&b).is_err());
    }

    #[test]
    fn test_timestamp_ordering() {
        let t1 = Timestamp::from_millis(1000);
        let t2 = Timestamp::from_millis(2000);
        assert!(t1.is_before(&t2));
        assert!(!t2.is_before(&t1));
    }
}
