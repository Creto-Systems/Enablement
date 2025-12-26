//! Main metering service facade.

use creto_common::{AgentId, CretoResult, OrganizationId};

use crate::{
    aggregation::AggregationEngine,
    events::UsageEvent,
    invoice::InvoiceGenerator,
    pricing::PricingEngine,
    quota::QuotaEnforcer,
};

/// Main entry point for the metering system.
///
/// Coordinates event ingestion, quota enforcement, aggregation, and billing.
pub struct MeteringService {
    /// Quota enforcement.
    pub quota_enforcer: QuotaEnforcer,

    /// Usage aggregation.
    pub aggregation_engine: AggregationEngine,

    /// Pricing calculation.
    pub pricing_engine: PricingEngine,

    /// Invoice generation.
    pub invoice_generator: InvoiceGenerator,
}

impl MeteringService {
    /// Create a new metering service.
    pub fn new() -> Self {
        Self {
            quota_enforcer: QuotaEnforcer::new(),
            aggregation_engine: AggregationEngine::new(),
            pricing_engine: PricingEngine::new(),
            invoice_generator: InvoiceGenerator::new(),
        }
    }

    /// Check if an operation is allowed and record usage.
    ///
    /// This is the main hot path called before every billable operation.
    ///
    /// # Performance
    ///
    /// Target: <10µs for the quota check portion.
    pub async fn check_and_record(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        event: UsageEvent,
    ) -> CretoResult<()> {
        // 1. Check quota (fast path)
        self.quota_enforcer
            .check(organization_id, agent_id, &event.code, event.quantity)
            .await?;

        // 2. Record usage (async, non-blocking)
        self.quota_enforcer
            .record(organization_id, agent_id, &event.code, event.quantity)
            .await?;

        // 3. Ingest event for aggregation (background)
        // TODO: Send to event queue

        Ok(())
    }

    /// Get quota status for an agent.
    pub async fn get_quota_status(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        metric_code: &str,
    ) -> CretoResult<crate::quota::QuotaStatus> {
        self.quota_enforcer
            .get_status(organization_id, agent_id, metric_code)
            .await
    }
}

impl Default for MeteringService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metering_service_creation() {
        let service = MeteringService::new();

        // Service should be creatable without errors
        let status = service
            .get_quota_status(OrganizationId::new(), AgentId::new(), "api_calls")
            .await;

        assert!(status.is_ok());
    }
}
