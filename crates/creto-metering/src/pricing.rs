//! Pricing models for usage-based billing.
//!
//! Supports multiple pricing strategies following Lago patterns.

use creto_common::types::Money;
use serde::{Deserialize, Serialize};

/// A pricing model that determines cost based on usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingModel {
    /// Unique identifier for this pricing model.
    pub id: String,

    /// Human-readable name.
    pub name: String,

    /// Billable metric code this pricing applies to.
    pub metric_code: String,

    /// Pricing strategy.
    pub strategy: PricingStrategy,
}

/// Strategy for calculating price from usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PricingStrategy {
    /// Fixed fee per billing period (regardless of usage).
    FlatFee {
        /// Amount in cents.
        amount_cents: i64,
    },

    /// Fixed price per unit consumed.
    PerUnit {
        /// Price per unit in cents.
        unit_price_cents: i64,
    },

    /// Graduated tiers (each tier has its own price).
    ///
    /// Example: First 100 units at $0.01, next 900 at $0.005, etc.
    GraduatedTiered {
        /// Ordered list of tiers.
        tiers: Vec<PricingTier>,
    },

    /// Volume tiers (all units priced at the tier reached).
    ///
    /// Example: If usage is 150, all 150 units are at the 100-500 tier price.
    VolumeTiered {
        /// Ordered list of tiers.
        tiers: Vec<PricingTier>,
    },

    /// Package pricing (buy in bulk).
    ///
    /// Example: $10 per 1000 tokens (partial packages rounded up).
    Package {
        /// Package size.
        package_size: i64,
        /// Price per package in cents.
        package_price_cents: i64,
    },

    /// Percentage of a base amount.
    ///
    /// Example: 2.5% transaction fee.
    Percentage {
        /// Percentage rate (e.g., 2.5 for 2.5%).
        rate: f64,
        /// Optional fixed component in cents.
        fixed_amount_cents: Option<i64>,
    },
}

/// A tier in graduated or volume pricing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    /// Minimum units for this tier (inclusive).
    pub from_units: i64,

    /// Maximum units for this tier (exclusive, None = unlimited).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_units: Option<i64>,

    /// Price per unit in cents.
    pub unit_price_cents: i64,

    /// Optional flat fee for entering this tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flat_fee_cents: Option<i64>,
}

impl PricingModel {
    /// Calculate the effective unit price for a given usage amount.
    /// For tiered pricing, this is the average price per unit.
    pub fn calculate_unit_price(&self, usage: i64) -> Money {
        if usage == 0 {
            return Money::usd(0);
        }
        let total = self.calculate(usage);
        Money::usd(total.amount / usage)
    }

    /// Calculate the total cost for a given usage amount.
    pub fn calculate(&self, usage: i64) -> Money {
        let cents = match &self.strategy {
            PricingStrategy::FlatFee { amount_cents } => *amount_cents,

            PricingStrategy::PerUnit { unit_price_cents } => usage * unit_price_cents,

            PricingStrategy::GraduatedTiered { tiers } => {
                Self::calculate_graduated(usage, tiers)
            }

            PricingStrategy::VolumeTiered { tiers } => {
                Self::calculate_volume(usage, tiers)
            }

            PricingStrategy::Package {
                package_size,
                package_price_cents,
            } => {
                let packages = (usage + package_size - 1) / package_size; // Round up
                packages * package_price_cents
            }

            PricingStrategy::Percentage {
                rate,
                fixed_amount_cents,
            } => {
                let percentage_amount = (usage as f64 * rate / 100.0) as i64;
                percentage_amount + fixed_amount_cents.unwrap_or(0)
            }
        };

        Money::usd(cents)
    }

    /// Calculate graduated tiered pricing.
    fn calculate_graduated(usage: i64, tiers: &[PricingTier]) -> i64 {
        let mut total = 0i64;
        let mut remaining = usage;

        for tier in tiers {
            if remaining <= 0 {
                break;
            }

            // Add flat fee if entering this tier
            if let Some(flat_fee) = tier.flat_fee_cents {
                if remaining > 0 {
                    total += flat_fee;
                }
            }

            let tier_size = tier
                .to_units
                .map(|to| to - tier.from_units)
                .unwrap_or(i64::MAX);

            let units_in_tier = remaining.min(tier_size);
            total += units_in_tier * tier.unit_price_cents;
            remaining -= units_in_tier;
        }

        total
    }

    /// Calculate volume tiered pricing.
    fn calculate_volume(usage: i64, tiers: &[PricingTier]) -> i64 {
        // Find the applicable tier
        let tier = tiers
            .iter()
            .find(|t| {
                usage >= t.from_units && t.to_units.map(|to| usage < to).unwrap_or(true)
            })
            .unwrap_or_else(|| tiers.last().expect("At least one tier required"));

        let flat_fee = tier.flat_fee_cents.unwrap_or(0);
        usage * tier.unit_price_cents + flat_fee
    }
}

/// Engine for calculating prices.
pub struct PricingEngine {
    // TODO: Add pricing model storage, caching
    _private: (),
}

impl PricingEngine {
    /// Create a new pricing engine.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Calculate the total cost for usage across multiple metrics.
    pub fn calculate_total(
        &self,
        usage: &[(String, i64)], // (metric_code, quantity)
        models: &[PricingModel],
    ) -> Money {
        let mut total_cents = 0i64;

        for (metric_code, quantity) in usage {
            if let Some(model) = models.iter().find(|m| &m.metric_code == metric_code) {
                let cost = model.calculate(*quantity);
                total_cents += cost.amount;
            }
        }

        Money::usd(total_cents)
    }
}

impl Default for PricingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_per_unit_pricing() {
        let model = PricingModel {
            id: "tokens".to_string(),
            name: "Token Pricing".to_string(),
            metric_code: "input_tokens".to_string(),
            strategy: PricingStrategy::PerUnit {
                unit_price_cents: 1, // $0.01 per token
            },
        };

        let cost = model.calculate(1000);
        assert_eq!(cost.amount, 1000); // $10.00
    }

    #[test]
    fn test_graduated_tiered_pricing() {
        let model = PricingModel {
            id: "api_calls".to_string(),
            name: "API Tiered".to_string(),
            metric_code: "api_calls".to_string(),
            strategy: PricingStrategy::GraduatedTiered {
                tiers: vec![
                    PricingTier {
                        from_units: 0,
                        to_units: Some(100),
                        unit_price_cents: 10, // $0.10 per call
                        flat_fee_cents: None,
                    },
                    PricingTier {
                        from_units: 100,
                        to_units: Some(1000),
                        unit_price_cents: 5, // $0.05 per call
                        flat_fee_cents: None,
                    },
                    PricingTier {
                        from_units: 1000,
                        to_units: None,
                        unit_price_cents: 1, // $0.01 per call
                        flat_fee_cents: None,
                    },
                ],
            },
        };

        // 150 calls: first 100 at $0.10, next 50 at $0.05
        let cost = model.calculate(150);
        assert_eq!(cost.amount, 100 * 10 + 50 * 5); // $12.50
    }

    #[test]
    fn test_package_pricing() {
        let model = PricingModel {
            id: "tokens_package".to_string(),
            name: "Token Packages".to_string(),
            metric_code: "total_tokens".to_string(),
            strategy: PricingStrategy::Package {
                package_size: 1000,
                package_price_cents: 100, // $1.00 per 1000 tokens
            },
        };

        // 2500 tokens = 3 packages (rounded up)
        let cost = model.calculate(2500);
        assert_eq!(cost.amount, 300); // $3.00
    }
}
