//! Invoice generation and management.

use chrono::{DateTime, Utc};
use creto_common::{types::Money, OrganizationId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An invoice for a billing period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// Unique invoice ID.
    pub id: Uuid,

    /// Invoice number (human-readable, sequential).
    pub number: String,

    /// Organization being billed.
    pub organization_id: OrganizationId,

    /// Start of billing period.
    pub period_start: DateTime<Utc>,

    /// End of billing period.
    pub period_end: DateTime<Utc>,

    /// Invoice status.
    pub status: InvoiceStatus,

    /// Line items.
    pub line_items: Vec<LineItem>,

    /// Subtotal before discounts/taxes.
    pub subtotal: Money,

    /// Applied discounts.
    #[serde(default)]
    pub discounts: Vec<Discount>,

    /// Tax amount.
    pub tax: Money,

    /// Total amount due.
    pub total: Money,

    /// When the invoice was issued.
    pub issued_at: Option<DateTime<Utc>>,

    /// Payment due date.
    pub due_at: Option<DateTime<Utc>>,

    /// When payment was received.
    pub paid_at: Option<DateTime<Utc>>,
}

impl Invoice {
    /// Create a new draft invoice.
    pub fn new(
        organization_id: OrganizationId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            number: Self::generate_invoice_number(),
            organization_id,
            period_start,
            period_end,
            status: InvoiceStatus::Draft,
            line_items: Vec::new(),
            subtotal: Money::usd(0),
            discounts: Vec::new(),
            tax: Money::usd(0),
            total: Money::usd(0),
            issued_at: None,
            due_at: None,
            paid_at: None,
        }
    }

    /// Generate a sequential invoice number.
    fn generate_invoice_number() -> String {
        // TODO: Use database sequence for real sequential numbers
        let timestamp = Utc::now().format("%Y%m");
        let random = uuid::Uuid::new_v4().as_u128() % 10000;
        format!("INV-{}-{:04}", timestamp, random)
    }

    /// Add a line item to the invoice.
    pub fn add_line_item(&mut self, item: LineItem) {
        self.subtotal = self
            .subtotal
            .add(&item.amount)
            .expect("Same currency");
        self.line_items.push(item);
        self.recalculate_total();
    }

    /// Apply a discount.
    pub fn apply_discount(&mut self, discount: Discount) {
        self.discounts.push(discount);
        self.recalculate_total();
    }

    /// Set the tax amount and recalculate total.
    pub fn set_tax(&mut self, tax: Money) {
        self.tax = tax;
        self.recalculate_total_with_tax();
    }

    /// Recalculate the total based on line items, discounts, and existing tax.
    fn recalculate_total_with_tax(&mut self) {
        let discount_amount: i64 = self
            .discounts
            .iter()
            .map(|d| d.calculate(self.subtotal.amount))
            .sum();

        let after_discount = self.subtotal.amount - discount_amount;
        self.total = Money::usd(after_discount + self.tax.amount);
    }

    /// Recalculate the total based on line items, discounts, and tax.
    pub fn recalculate_total(&mut self) {
        let discount_amount: i64 = self
            .discounts
            .iter()
            .map(|d| d.calculate(self.subtotal.amount))
            .sum();

        let after_discount = self.subtotal.amount - discount_amount;

        // TODO: Calculate tax based on jurisdiction
        self.tax = Money::usd(0);

        self.total = Money::usd(after_discount + self.tax.amount);
    }

    /// Finalize and issue the invoice.
    pub fn issue(&mut self, due_days: i64) {
        let now = Utc::now();
        self.status = InvoiceStatus::Issued;
        self.issued_at = Some(now);
        self.due_at = Some(now + chrono::Duration::days(due_days));
    }

    /// Mark the invoice as paid.
    pub fn mark_paid(&mut self) {
        self.status = InvoiceStatus::Paid;
        self.paid_at = Some(Utc::now());
    }

    /// Check if the invoice is overdue.
    pub fn is_overdue(&self) -> bool {
        matches!(self.status, InvoiceStatus::Issued)
            && self.due_at.map(|due| Utc::now() > due).unwrap_or(false)
    }
}

/// Status of an invoice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    /// Being prepared, not yet sent.
    Draft,
    /// Sent to customer.
    Issued,
    /// Payment received.
    Paid,
    /// Payment failed or refused.
    Failed,
    /// Voided (cancelled).
    Voided,
}

/// A line item on an invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    /// Unique line item ID.
    pub id: Uuid,

    /// Description of the charge.
    pub description: String,

    /// Billable metric code.
    pub metric_code: String,

    /// Quantity consumed.
    pub quantity: i64,

    /// Unit of measurement.
    pub unit: String,

    /// Unit price.
    pub unit_price: Money,

    /// Total amount (quantity * unit_price, may include adjustments).
    pub amount: Money,
}

impl LineItem {
    /// Create a new line item.
    pub fn new(
        description: impl Into<String>,
        metric_code: impl Into<String>,
        quantity: i64,
        unit: impl Into<String>,
        unit_price: Money,
    ) -> Self {
        let amount = Money::usd(quantity * unit_price.amount);

        Self {
            id: Uuid::now_v7(),
            description: description.into(),
            metric_code: metric_code.into(),
            quantity,
            unit: unit.into(),
            unit_price,
            amount,
        }
    }
}

/// A discount applied to an invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discount {
    /// Discount code or description.
    pub code: String,

    /// Type of discount.
    pub discount_type: DiscountType,
}

/// Type of discount.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DiscountType {
    /// Fixed amount discount.
    FixedAmount {
        /// Amount in cents.
        amount_cents: i64,
    },
    /// Percentage discount.
    Percentage {
        /// Percentage (e.g., 10.0 for 10%).
        rate: f64,
    },
}

impl Discount {
    /// Calculate the discount amount for a given subtotal.
    pub fn calculate(&self, subtotal_cents: i64) -> i64 {
        match &self.discount_type {
            DiscountType::FixedAmount { amount_cents } => {
                (*amount_cents).min(subtotal_cents) // Can't discount more than subtotal
            }
            DiscountType::Percentage { rate } => {
                (subtotal_cents as f64 * rate / 100.0) as i64
            }
        }
    }
}

/// Generator for creating invoices from usage data.
pub struct InvoiceGenerator {
    /// Pricing models by metric code.
    pricing_models: std::collections::HashMap<String, crate::pricing::PricingModel>,
    /// Default due days for issued invoices.
    due_days: i64,
    /// Tax rate (percentage).
    tax_rate: f64,
}

impl InvoiceGenerator {
    /// Create a new invoice generator.
    pub fn new() -> Self {
        Self {
            pricing_models: std::collections::HashMap::new(),
            due_days: 30,
            tax_rate: 0.0, // No tax by default
        }
    }

    /// Create with specific configuration.
    pub fn with_config(due_days: i64, tax_rate: f64) -> Self {
        Self {
            pricing_models: std::collections::HashMap::new(),
            due_days,
            tax_rate,
        }
    }

    /// Register a pricing model.
    pub fn register_pricing_model(&mut self, model: crate::pricing::PricingModel) {
        self.pricing_models.insert(model.metric_code.clone(), model);
    }

    /// Generate an invoice from aggregated usage data.
    ///
    /// This is the synchronous version that takes pre-computed aggregations.
    pub fn generate_from_aggregations(
        &self,
        organization_id: OrganizationId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        aggregations: &[UsageAggregation],
    ) -> Invoice {
        let mut invoice = Invoice::new(organization_id, period_start, period_end);

        for agg in aggregations {
            // Find pricing model for this metric
            if let Some(model) = self.pricing_models.get(&agg.metric_code) {
                let cost = model.calculate(agg.quantity);

                let line_item = LineItem::new(
                    &agg.description,
                    &agg.metric_code,
                    agg.quantity,
                    &agg.unit,
                    model.calculate_unit_price(agg.quantity),
                );

                invoice.add_line_item(LineItem {
                    amount: cost,
                    ..line_item
                });
            } else {
                // No pricing model - use quantity as cents (1:1 mapping)
                let line_item = LineItem::new(
                    &agg.description,
                    &agg.metric_code,
                    agg.quantity,
                    &agg.unit,
                    Money::usd(1), // Default $0.01 per unit
                );
                invoice.add_line_item(line_item);
            }
        }

        // Apply tax if configured
        if self.tax_rate > 0.0 {
            let tax_cents = (invoice.subtotal.amount as f64 * self.tax_rate / 100.0) as i64;
            invoice.set_tax(Money::usd(tax_cents));
        }

        invoice
    }

    /// Generate and issue an invoice.
    pub fn generate_and_issue(
        &self,
        organization_id: OrganizationId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        aggregations: &[UsageAggregation],
    ) -> Invoice {
        let mut invoice = self.generate_from_aggregations(
            organization_id,
            period_start,
            period_end,
            aggregations,
        );

        invoice.issue(self.due_days);
        invoice
    }

    /// Generate an invoice for an organization's usage in a billing period.
    ///
    /// This async version fetches aggregations from the database.
    pub async fn generate(
        &self,
        organization_id: OrganizationId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Invoice, creto_common::CretoError> {
        // In production, this would fetch from the aggregation engine
        // For now, return an empty invoice
        let invoice = Invoice::new(organization_id, period_start, period_end);
        Ok(invoice)
    }

    /// Get registered pricing models.
    pub fn pricing_models(&self) -> &std::collections::HashMap<String, crate::pricing::PricingModel> {
        &self.pricing_models
    }
}

impl Default for InvoiceGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Aggregated usage data for invoice generation.
#[derive(Debug, Clone)]
pub struct UsageAggregation {
    /// Metric code.
    pub metric_code: String,
    /// Description for the line item.
    pub description: String,
    /// Total quantity consumed.
    pub quantity: i64,
    /// Unit of measurement.
    pub unit: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invoice_line_item_calculation() {
        let mut invoice = Invoice::new(
            OrganizationId::new(),
            Utc::now() - chrono::Duration::days(30),
            Utc::now(),
        );

        let item = LineItem::new(
            "API Calls",
            "api_calls",
            1000,
            "calls",
            Money::usd(1), // $0.01 per call
        );

        invoice.add_line_item(item);

        assert_eq!(invoice.subtotal.amount, 1000); // $10.00
        assert_eq!(invoice.total.amount, 1000);
    }

    #[test]
    fn test_percentage_discount() {
        let mut invoice = Invoice::new(
            OrganizationId::new(),
            Utc::now() - chrono::Duration::days(30),
            Utc::now(),
        );

        invoice.add_line_item(LineItem::new(
            "Tokens",
            "tokens",
            10000,
            "tokens",
            Money::usd(1),
        ));

        invoice.apply_discount(Discount {
            code: "WELCOME10".to_string(),
            discount_type: DiscountType::Percentage { rate: 10.0 },
        });

        assert_eq!(invoice.subtotal.amount, 10000); // $100.00
        assert_eq!(invoice.total.amount, 9000); // $90.00 after 10% discount
    }

    #[test]
    fn test_invoice_status_transitions() {
        let mut invoice = Invoice::new(
            OrganizationId::new(),
            Utc::now() - chrono::Duration::days(30),
            Utc::now(),
        );

        assert_eq!(invoice.status, InvoiceStatus::Draft);

        invoice.issue(30);
        assert_eq!(invoice.status, InvoiceStatus::Issued);
        assert!(invoice.issued_at.is_some());
        assert!(invoice.due_at.is_some());

        invoice.mark_paid();
        assert_eq!(invoice.status, InvoiceStatus::Paid);
        assert!(invoice.paid_at.is_some());
    }

    #[test]
    fn test_invoice_generator_with_pricing() {
        use crate::pricing::{PricingModel, PricingStrategy};

        let mut generator = InvoiceGenerator::new();

        // Register per-unit pricing for tokens
        generator.register_pricing_model(PricingModel {
            id: "tokens".to_string(),
            name: "Token Pricing".to_string(),
            metric_code: "tokens".to_string(),
            strategy: PricingStrategy::PerUnit {
                unit_price_cents: 1, // $0.01 per token
            },
        });

        // Register package pricing for API calls
        generator.register_pricing_model(PricingModel {
            id: "api_calls".to_string(),
            name: "API Call Packages".to_string(),
            metric_code: "api_calls".to_string(),
            strategy: PricingStrategy::Package {
                package_size: 1000,
                package_price_cents: 500, // $5.00 per 1000 calls
            },
        });

        let org_id = OrganizationId::new();
        let period_start = Utc::now() - chrono::Duration::days(30);
        let period_end = Utc::now();

        let aggregations = vec![
            UsageAggregation {
                metric_code: "tokens".to_string(),
                description: "Input Tokens".to_string(),
                quantity: 5000,
                unit: "tokens".to_string(),
            },
            UsageAggregation {
                metric_code: "api_calls".to_string(),
                description: "API Calls".to_string(),
                quantity: 2500, // 3 packages
                unit: "calls".to_string(),
            },
        ];

        let invoice = generator.generate_from_aggregations(
            org_id,
            period_start,
            period_end,
            &aggregations,
        );

        assert_eq!(invoice.line_items.len(), 2);
        // Tokens: 5000 * $0.01 = $50.00
        // API Calls: 3 packages * $5.00 = $15.00
        // Total: $65.00
        assert_eq!(invoice.subtotal.amount, 5000 + 1500);
        assert_eq!(invoice.total.amount, 6500);
    }

    #[test]
    fn test_invoice_generator_with_tax() {
        let generator = InvoiceGenerator::with_config(30, 10.0); // 10% tax

        let org_id = OrganizationId::new();
        let period_start = Utc::now() - chrono::Duration::days(30);
        let period_end = Utc::now();

        let aggregations = vec![UsageAggregation {
            metric_code: "compute".to_string(),
            description: "Compute Hours".to_string(),
            quantity: 10000, // $100.00 at default rate
            unit: "hours".to_string(),
        }];

        let invoice = generator.generate_from_aggregations(
            org_id,
            period_start,
            period_end,
            &aggregations,
        );

        assert_eq!(invoice.subtotal.amount, 10000); // $100.00
        assert_eq!(invoice.tax.amount, 1000); // $10.00 tax
        assert_eq!(invoice.total.amount, 11000); // $110.00 total
    }

    #[test]
    fn test_invoice_generator_and_issue() {
        let generator = InvoiceGenerator::with_config(14, 0.0); // 14 day due, no tax

        let org_id = OrganizationId::new();
        let period_start = Utc::now() - chrono::Duration::days(30);
        let period_end = Utc::now();

        let aggregations = vec![UsageAggregation {
            metric_code: "storage".to_string(),
            description: "Storage GB".to_string(),
            quantity: 1000,
            unit: "GB".to_string(),
        }];

        let invoice = generator.generate_and_issue(
            org_id,
            period_start,
            period_end,
            &aggregations,
        );

        assert_eq!(invoice.status, InvoiceStatus::Issued);
        assert!(invoice.issued_at.is_some());
        assert!(invoice.due_at.is_some());

        // Due date should be ~14 days from issue
        let due_diff = invoice.due_at.unwrap() - invoice.issued_at.unwrap();
        assert_eq!(due_diff.num_days(), 14);
    }
}
