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

    /// Recalculate the total based on line items, discounts, and tax.
    fn recalculate_total(&mut self) {
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
    // TODO: Add dependencies
    _private: (),
}

impl InvoiceGenerator {
    /// Create a new invoice generator.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Generate an invoice for an organization's usage in a billing period.
    pub async fn generate(
        &self,
        organization_id: OrganizationId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<Invoice, creto_common::CretoError> {
        // TODO: Implement invoice generation
        // 1. Fetch aggregated usage for the period
        // 2. Apply pricing models
        // 3. Generate line items
        // 4. Apply discounts
        // 5. Calculate tax

        let _ = (organization_id, period_start, period_end);

        todo!("Invoice generation not yet implemented")
    }
}

impl Default for InvoiceGenerator {
    fn default() -> Self {
        Self::new()
    }
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
}
