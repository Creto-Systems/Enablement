//! Main metering service facade.
//!
//! Provides a unified API for the complete billing pipeline:
//! - Event ingestion with quota enforcement
//! - Usage aggregation
//! - Invoice generation with credits application
//! - Pricing management

use chrono::{DateTime, Utc};
use creto_common::{AgentId, CretoResult, OrganizationId};

use crate::{
    aggregation::AggregationEngine,
    credits::{CreditApplication, CreditManager},
    events::UsageEvent,
    invoice::{Invoice, InvoiceGenerator, UsageAggregation},
    pricing::{PricingEngine, PricingModel},
    quota::{Quota, QuotaCheckResult, QuotaEnforcer, QuotaPeriod},
};

/// Main entry point for the metering system.
///
/// Coordinates event ingestion, quota enforcement, aggregation, and billing.
///
/// # Architecture
///
/// ```text
/// ┌─────────────────────────────────────────────────────────────────┐
/// │                      MeteringService                            │
/// │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
/// │  │    Quota     │  │   Credits    │  │   Invoice    │          │
/// │  │   Enforcer   │  │   Manager    │  │  Generator   │          │
/// │  └──────────────┘  └──────────────┘  └──────────────┘          │
/// │         ↓                  ↓                  ↓                 │
/// │  ┌──────────────────────────────────────────────────┐          │
/// │  │         Aggregation Engine + Pricing Engine      │          │
/// │  └──────────────────────────────────────────────────┘          │
/// └─────────────────────────────────────────────────────────────────┘
/// ```
pub struct MeteringService {
    /// Quota enforcement with bloom filter and reservations.
    pub quota_enforcer: QuotaEnforcer,

    /// Usage aggregation.
    pub aggregation_engine: AggregationEngine,

    /// Pricing calculation.
    pub pricing_engine: PricingEngine,

    /// Invoice generation.
    pub invoice_generator: InvoiceGenerator,

    /// Credit/wallet management.
    pub credit_manager: CreditManager,

    /// In-memory usage storage for aggregation (production: database).
    usage_records: std::sync::RwLock<Vec<UsageRecord>>,
}

/// Internal usage record for aggregation.
#[derive(Debug, Clone)]
struct UsageRecord {
    organization_id: OrganizationId,
    agent_id: AgentId,
    metric_code: String,
    quantity: i64,
    timestamp: DateTime<Utc>,
}

impl MeteringService {
    /// Create a new metering service with default configuration.
    pub fn new() -> Self {
        Self {
            quota_enforcer: QuotaEnforcer::new(),
            aggregation_engine: AggregationEngine::new(),
            pricing_engine: PricingEngine::new(),
            invoice_generator: InvoiceGenerator::new(),
            credit_manager: CreditManager::new(),
            usage_records: std::sync::RwLock::new(Vec::new()),
        }
    }

    /// Create with custom invoice configuration.
    pub fn with_invoice_config(due_days: i64, tax_rate: f64) -> Self {
        Self {
            quota_enforcer: QuotaEnforcer::new(),
            aggregation_engine: AggregationEngine::new(),
            pricing_engine: PricingEngine::new(),
            invoice_generator: InvoiceGenerator::with_config(due_days, tax_rate),
            credit_manager: CreditManager::new(),
            usage_records: std::sync::RwLock::new(Vec::new()),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Quota Management
    // ─────────────────────────────────────────────────────────────────────────

    /// Register a quota for enforcement.
    pub fn register_quota(&self, quota: &Quota) {
        self.quota_enforcer.register_quota(quota);
    }

    /// Create and register a simple quota.
    pub fn create_quota(
        &self,
        organization_id: OrganizationId,
        metric_code: &str,
        limit: i64,
        period: QuotaPeriod,
    ) -> Quota {
        let quota = Quota::new(organization_id, metric_code, limit, period);
        self.quota_enforcer.register_quota(&quota);
        quota
    }

    /// Get quota status for an organization/agent.
    pub fn get_quota_status(
        &self,
        organization_id: &OrganizationId,
        agent_id: &AgentId,
        metric_code: &str,
    ) -> CretoResult<QuotaCheckResult> {
        self.quota_enforcer
            .check(organization_id, agent_id, metric_code, 0)
            .map_err(|_e| creto_common::CretoError::QuotaExceeded {
                resource: metric_code.to_string(),
                used: 0,
                limit: 0,
            })
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Event Ingestion
    // ─────────────────────────────────────────────────────────────────────────

    /// Check if an operation is allowed and record usage.
    ///
    /// This is the main hot path called before every billable operation.
    ///
    /// # Performance
    ///
    /// Target: <10µs for the quota check portion.
    pub fn check_and_record(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        event: UsageEvent,
    ) -> CretoResult<()> {
        // 1. Check quota (fast path - sync, <10µs target)
        let result = self
            .quota_enforcer
            .check(&organization_id, &agent_id, &event.code, event.quantity)
            .map_err(|_e| creto_common::CretoError::QuotaExceeded {
                resource: event.code.clone(),
                used: 0,
                limit: 0,
            })?;

        if !result.allowed {
            return Err(creto_common::CretoError::QuotaExceeded {
                resource: event.code.clone(),
                used: result.current_usage as u64,
                limit: result.limit as u64,
            });
        }

        // 2. Record usage in quota enforcer (sync)
        self.quota_enforcer
            .record_usage(&organization_id, &agent_id, &event.code, event.quantity)
            .map_err(|_e| creto_common::CretoError::QuotaExceeded {
                resource: event.code.clone(),
                used: 0,
                limit: 0,
            })?;

        // 3. Store for aggregation
        let record = UsageRecord {
            organization_id,
            agent_id,
            metric_code: event.code,
            quantity: event.quantity,
            timestamp: event.timestamp,
        };
        self.usage_records.write().unwrap().push(record);

        Ok(())
    }

    /// Record usage without quota check (for pre-authorized operations).
    pub fn record_usage(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        event: UsageEvent,
    ) {
        let record = UsageRecord {
            organization_id,
            agent_id,
            metric_code: event.code,
            quantity: event.quantity,
            timestamp: event.timestamp,
        };
        self.usage_records.write().unwrap().push(record);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Pricing Management
    // ─────────────────────────────────────────────────────────────────────────

    /// Register a pricing model.
    pub fn register_pricing_model(&mut self, model: PricingModel) {
        self.invoice_generator.register_pricing_model(model);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Credit Management
    // ─────────────────────────────────────────────────────────────────────────

    /// Grant credits to an organization.
    pub fn grant_credits(
        &self,
        organization_id: OrganizationId,
        amount_cents: i64,
        description: Option<&str>,
    ) -> CretoResult<()> {
        self.credit_manager
            .grant_credits(organization_id, amount_cents, description)?;
        Ok(())
    }

    /// Get credit balance for an organization.
    pub fn get_credit_balance(&self, organization_id: &OrganizationId) -> i64 {
        self.credit_manager.get_balance(organization_id)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Billing & Invoicing
    // ─────────────────────────────────────────────────────────────────────────

    /// Aggregate usage for a billing period.
    pub fn aggregate_usage(
        &self,
        organization_id: &OrganizationId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Vec<UsageAggregation> {
        let records = self.usage_records.read().unwrap();

        // Group by metric code
        let mut aggregations: std::collections::HashMap<String, i64> =
            std::collections::HashMap::new();

        for record in records.iter() {
            if &record.organization_id == organization_id
                && record.timestamp >= period_start
                && record.timestamp <= period_end
            {
                *aggregations.entry(record.metric_code.clone()).or_insert(0) += record.quantity;
            }
        }

        aggregations
            .into_iter()
            .map(|(metric_code, quantity)| UsageAggregation {
                metric_code: metric_code.clone(),
                description: format!("{} usage", metric_code),
                quantity,
                unit: "units".to_string(),
            })
            .collect()
    }

    /// Generate an invoice for a billing period.
    pub fn generate_invoice(
        &self,
        organization_id: OrganizationId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Invoice {
        let aggregations = self.aggregate_usage(&organization_id, period_start, period_end);

        self.invoice_generator.generate_from_aggregations(
            organization_id,
            period_start,
            period_end,
            &aggregations,
        )
    }

    /// Generate invoice and apply available credits.
    pub fn generate_invoice_with_credits(
        &self,
        organization_id: OrganizationId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> (Invoice, CreditApplication) {
        let mut invoice = self.generate_invoice(organization_id.clone(), period_start, period_end);

        // Apply credits
        let application = self
            .credit_manager
            .apply_credits_to_invoice(
                &organization_id,
                invoice.total.amount,
                &invoice.id.to_string(),
            )
            .unwrap_or(CreditApplication {
                credits_applied: 0,
                remaining_to_invoice: invoice.total.amount,
            });

        // Update invoice total if credits were applied
        if application.credits_applied > 0 {
            let credits_discount = crate::invoice::Discount {
                code: "CREDITS".to_string(),
                discount_type: crate::invoice::DiscountType::FixedAmount {
                    amount_cents: application.credits_applied,
                },
            };
            invoice.apply_discount(credits_discount);
        }

        (invoice, application)
    }

    /// Complete billing workflow: aggregate, price, apply credits, issue invoice.
    pub fn run_billing_cycle(
        &self,
        organization_id: OrganizationId,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> BillingResult {
        // 1. Aggregate usage
        let aggregations = self.aggregate_usage(&organization_id, period_start, period_end);

        // 2. Generate invoice
        let mut invoice = self.invoice_generator.generate_from_aggregations(
            organization_id.clone(),
            period_start,
            period_end,
            &aggregations,
        );

        let subtotal = invoice.subtotal.amount;

        // 3. Apply credits
        let credit_application = self
            .credit_manager
            .apply_credits_to_invoice(
                &organization_id,
                invoice.total.amount,
                &invoice.id.to_string(),
            )
            .unwrap_or(CreditApplication {
                credits_applied: 0,
                remaining_to_invoice: invoice.total.amount,
            });

        if credit_application.credits_applied > 0 {
            let credits_discount = crate::invoice::Discount {
                code: "CREDITS_APPLIED".to_string(),
                discount_type: crate::invoice::DiscountType::FixedAmount {
                    amount_cents: credit_application.credits_applied,
                },
            };
            invoice.apply_discount(credits_discount);
        }

        // 4. Issue invoice
        invoice.issue(30); // Default 30-day payment terms

        BillingResult {
            invoice,
            usage_count: aggregations.len(),
            subtotal_cents: subtotal,
            credits_applied: credit_application.credits_applied,
            amount_due: credit_application.remaining_to_invoice,
        }
    }
}

impl Default for MeteringService {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a billing cycle.
#[derive(Debug)]
pub struct BillingResult {
    /// Generated invoice.
    pub invoice: Invoice,
    /// Number of usage metrics aggregated.
    pub usage_count: usize,
    /// Subtotal before credits.
    pub subtotal_cents: i64,
    /// Credits applied.
    pub credits_applied: i64,
    /// Final amount due.
    pub amount_due: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pricing::{PricingModel, PricingStrategy};

    #[test]
    fn test_metering_service_creation() {
        let service = MeteringService::new();

        // Service should be creatable without errors
        let status =
            service.get_quota_status(&OrganizationId::new(), &AgentId::new(), "api_calls");

        // Just verify it doesn't panic
        let _ = status;
    }

    #[test]
    fn test_full_billing_workflow() {
        let mut service = MeteringService::new();
        let org_id = OrganizationId::new();
        let agent_id = AgentId::new();

        // 1. Setup pricing
        service.register_pricing_model(PricingModel {
            id: "api_calls".to_string(),
            name: "API Call Pricing".to_string(),
            metric_code: "api_calls".to_string(),
            strategy: PricingStrategy::PerUnit {
                unit_price_cents: 1, // $0.01 per call
            },
        });

        // 2. Setup quota
        service.create_quota(org_id.clone(), "api_calls", 10000, QuotaPeriod::Monthly);

        // 3. Record some usage - use fixed base time to avoid timing issues
        let base_time = Utc::now();
        let period_start = base_time - chrono::Duration::days(30);
        let period_end = base_time;

        for i in 0..100 {
            let event = UsageEvent {
                transaction_id: format!("tx_{}", i),
                organization_id: org_id.clone(),
                agent_id: agent_id.clone(),
                event_type: crate::events::UsageEventType::ApiCall,
                code: "api_calls".to_string(),
                quantity: 10,
                timestamp: base_time - chrono::Duration::hours(i),
                properties: Default::default(),
                delegation_depth: 0,
                external_subscription_id: None,
            };
            service.record_usage(org_id.clone(), agent_id.clone(), event);
        }

        // 4. Generate invoice
        let invoice = service.generate_invoice(org_id.clone(), period_start, period_end);

        assert_eq!(invoice.line_items.len(), 1);
        assert_eq!(invoice.subtotal.amount, 1000); // 100 * 10 = 1000 units * $0.01 = $10.00
    }

    #[test]
    fn test_billing_with_credits() {
        let mut service = MeteringService::new();
        let org_id = OrganizationId::new();
        let agent_id = AgentId::new();

        // Setup pricing
        service.register_pricing_model(PricingModel {
            id: "tokens".to_string(),
            name: "Token Pricing".to_string(),
            metric_code: "tokens".to_string(),
            strategy: PricingStrategy::PerUnit {
                unit_price_cents: 1,
            },
        });

        // Grant $50 in credits
        service
            .grant_credits(org_id.clone(), 5000, Some("Welcome bonus"))
            .unwrap();

        // Record $100 worth of usage
        let period_start = Utc::now() - chrono::Duration::days(30);

        for i in 0..100 {
            let event = UsageEvent {
                transaction_id: format!("tx_{}", i),
                organization_id: org_id.clone(),
                agent_id: agent_id.clone(),
                event_type: crate::events::UsageEventType::TotalTokens,
                code: "tokens".to_string(),
                quantity: 100, // 100 tokens per event
                timestamp: Utc::now() - chrono::Duration::hours(i),
                properties: Default::default(),
                delegation_depth: 0,
                external_subscription_id: None,
            };
            service.record_usage(org_id.clone(), agent_id.clone(), event);
        }

        // Run billing cycle
        let result = service.run_billing_cycle(org_id.clone(), period_start, Utc::now());

        assert_eq!(result.subtotal_cents, 10000); // $100.00
        assert_eq!(result.credits_applied, 5000); // $50.00 credits
        assert_eq!(result.amount_due, 5000); // $50.00 remaining
        assert_eq!(service.get_credit_balance(&org_id), 0); // Credits depleted
    }

    #[test]
    fn test_quota_enforcement_in_workflow() {
        let service = MeteringService::new();
        let org_id = OrganizationId::new();
        let agent_id = AgentId::new();

        // Create a small quota
        service.create_quota(org_id.clone(), "limited_calls", 100, QuotaPeriod::Daily);

        // Should succeed initially
        let event = UsageEvent {
            transaction_id: "tx_1".to_string(),
            organization_id: org_id.clone(),
            agent_id: agent_id.clone(),
            event_type: crate::events::UsageEventType::ApiCall,
            code: "limited_calls".to_string(),
            quantity: 50,
            timestamp: Utc::now(),
            properties: Default::default(),
            delegation_depth: 0,
            external_subscription_id: None,
        };

        let result = service.check_and_record(org_id.clone(), agent_id.clone(), event);
        assert!(result.is_ok());

        // Should fail when exceeding quota
        let event2 = UsageEvent {
            transaction_id: "tx_2".to_string(),
            organization_id: org_id.clone(),
            agent_id: agent_id.clone(),
            event_type: crate::events::UsageEventType::ApiCall,
            code: "limited_calls".to_string(),
            quantity: 100, // Would exceed limit
            timestamp: Utc::now(),
            properties: Default::default(),
            delegation_depth: 0,
            external_subscription_id: None,
        };

        let result2 = service.check_and_record(org_id, agent_id, event2);
        assert!(result2.is_err());
    }
}
