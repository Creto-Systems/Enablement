//! Repository traits and PostgreSQL implementations for metering.
//!
//! This module provides the persistence layer for usage events, quotas,
//! billable metrics, pricing models, and invoices.
//!
//! Uses runtime SQL queries to avoid requiring DATABASE_URL at compile time.

use chrono::{DateTime, Utc};
use creto_common::{AgentId, CretoError, OrganizationId};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::events::{UsageEvent, UsageEventType};
use crate::quota::{Quota, QuotaPeriod};

// ─────────────────────────────────────────────────────────────────────────────
// Enum Serialization Helpers
// ─────────────────────────────────────────────────────────────────────────────

// Convenience method for getting current bounds (wraps calculate_bounds)
impl QuotaPeriod {
    /// Get the current period bounds.
    pub fn current_bounds(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        self.calculate_bounds(Utc::now())
    }
}

impl UsageEventType {
    /// Convert to database string representation.
    pub fn as_db_str(&self) -> &'static str {
        match self {
            UsageEventType::ApiCall => "api_call",
            UsageEventType::LlmInference => "llm_inference",
            UsageEventType::EmbeddingGeneration => "embedding_generation",
            UsageEventType::InputTokens => "input_tokens",
            UsageEventType::OutputTokens => "output_tokens",
            UsageEventType::TotalTokens => "total_tokens",
            UsageEventType::CpuMilliseconds => "cpu_milliseconds",
            UsageEventType::MemoryMbSeconds => "memory_mb_seconds",
            UsageEventType::GpuMilliseconds => "gpu_milliseconds",
            UsageEventType::StorageBytes => "storage_bytes",
            UsageEventType::NetworkEgressBytes => "network_egress_bytes",
            UsageEventType::OversightRequest => "oversight_request",
            UsageEventType::SandboxExecution => "sandbox_execution",
            UsageEventType::MessageSent => "message_sent",
        }
    }

    /// Parse from database string.
    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "api_call" => Some(UsageEventType::ApiCall),
            "llm_inference" => Some(UsageEventType::LlmInference),
            "embedding_generation" => Some(UsageEventType::EmbeddingGeneration),
            "input_tokens" => Some(UsageEventType::InputTokens),
            "output_tokens" => Some(UsageEventType::OutputTokens),
            "total_tokens" => Some(UsageEventType::TotalTokens),
            "cpu_milliseconds" => Some(UsageEventType::CpuMilliseconds),
            "memory_mb_seconds" => Some(UsageEventType::MemoryMbSeconds),
            "gpu_milliseconds" => Some(UsageEventType::GpuMilliseconds),
            "storage_bytes" => Some(UsageEventType::StorageBytes),
            "network_egress_bytes" => Some(UsageEventType::NetworkEgressBytes),
            "oversight_request" => Some(UsageEventType::OversightRequest),
            "sandbox_execution" => Some(UsageEventType::SandboxExecution),
            "message_sent" => Some(UsageEventType::MessageSent),
            _ => None,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Event Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Repository for usage event persistence.
#[trait_variant::make(EventRepository: Send)]
pub trait LocalEventRepository {
    /// Insert a usage event.
    async fn insert_event(&self, event: &UsageEvent) -> Result<(), CretoError>;

    /// Insert multiple events in a batch.
    async fn insert_events_batch(&self, events: &[UsageEvent]) -> Result<usize, CretoError>;

    /// Find events by organization within a time range.
    async fn find_by_org_and_time(
        &self,
        org_id: OrganizationId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<UsageEvent>, CretoError>;

    /// Count events by code within a time range.
    async fn count_by_code(
        &self,
        org_id: OrganizationId,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<i64, CretoError>;

    /// Sum quantities by code within a time range.
    async fn sum_by_code(
        &self,
        org_id: OrganizationId,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<i64, CretoError>;
}

/// PostgreSQL implementation of EventRepository.
pub struct PgEventRepository {
    pool: PgPool,
}

impl PgEventRepository {
    /// Create a new repository with the given connection pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl EventRepository for PgEventRepository {
    async fn insert_event(&self, event: &UsageEvent) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            INSERT INTO usage_events (
                transaction_id, organization_id, agent_id, external_subscription_id,
                event_type, code, quantity, timestamp, properties, delegation_depth
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (transaction_id) DO NOTHING
            "#,
        )
        .bind(&event.transaction_id)
        .bind(event.organization_id.as_uuid())
        .bind(event.agent_id.as_uuid())
        .bind(&event.external_subscription_id)
        .bind(event.event_type.as_db_str())
        .bind(&event.code)
        .bind(event.quantity)
        .bind(event.timestamp)
        .bind(&event.properties)
        .bind(event.delegation_depth as i16)
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }

    async fn insert_events_batch(&self, events: &[UsageEvent]) -> Result<usize, CretoError> {
        let mut tx = self.pool.begin().await
            .map_err(|e| CretoError::Database(e.to_string()))?;
        let mut count = 0;

        for event in events {
            let result = sqlx::query(
                r#"
                INSERT INTO usage_events (
                    transaction_id, organization_id, agent_id, external_subscription_id,
                    event_type, code, quantity, timestamp, properties, delegation_depth
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                ON CONFLICT (transaction_id) DO NOTHING
                "#,
            )
            .bind(&event.transaction_id)
            .bind(event.organization_id.as_uuid())
            .bind(event.agent_id.as_uuid())
            .bind(&event.external_subscription_id)
            .bind(event.event_type.as_db_str())
            .bind(&event.code)
            .bind(event.quantity)
            .bind(event.timestamp)
            .bind(&event.properties)
            .bind(event.delegation_depth as i16)
            .execute(&mut *tx)
            .await
            .map_err(|e| CretoError::Database(e.to_string()))?;

            if result.rows_affected() > 0 {
                count += 1;
            }
        }

        tx.commit().await
            .map_err(|e| CretoError::Database(e.to_string()))?;
        Ok(count)
    }

    async fn find_by_org_and_time(
        &self,
        org_id: OrganizationId,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: i64,
    ) -> Result<Vec<UsageEvent>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT transaction_id, organization_id, agent_id, external_subscription_id,
                   event_type, code, quantity, timestamp, properties, delegation_depth
            FROM usage_events
            WHERE organization_id = $1 AND timestamp >= $2 AND timestamp < $3
            ORDER BY timestamp DESC
            LIMIT $4
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(start)
        .bind(end)
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            let event_type_str: String = row.get("event_type");
            let event_type = UsageEventType::from_db_str(&event_type_str)
                .ok_or_else(|| CretoError::Database(format!("Unknown event type: {}", event_type_str)))?;

            events.push(UsageEvent {
                transaction_id: row.get("transaction_id"),
                organization_id: OrganizationId::from_uuid(row.get::<Uuid, _>("organization_id")),
                agent_id: AgentId::from_uuid(row.get::<Uuid, _>("agent_id")),
                external_subscription_id: row.get("external_subscription_id"),
                event_type,
                code: row.get("code"),
                quantity: row.get("quantity"),
                timestamp: row.get("timestamp"),
                properties: row.get("properties"),
                delegation_depth: row.get::<i16, _>("delegation_depth") as u8,
            });
        }

        Ok(events)
    }

    async fn count_by_code(
        &self,
        org_id: OrganizationId,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<i64, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM usage_events
            WHERE organization_id = $1 AND code = $2 AND timestamp >= $3 AND timestamp < $4
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(code)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("count"))
    }

    async fn sum_by_code(
        &self,
        org_id: OrganizationId,
        code: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<i64, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT COALESCE(SUM(quantity), 0) as total
            FROM usage_events
            WHERE organization_id = $1 AND code = $2 AND timestamp >= $3 AND timestamp < $4
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(code)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("total"))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Quota Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Repository for quota persistence.
#[trait_variant::make(QuotaRepository: Send)]
pub trait LocalQuotaRepository {
    /// Get or create a quota for the given parameters.
    async fn get_or_create(
        &self,
        org_id: OrganizationId,
        agent_id: Option<AgentId>,
        metric_code: &str,
        period: QuotaPeriod,
        limit: i64,
    ) -> Result<Quota, CretoError>;

    /// Update current usage for a quota.
    async fn increment_usage(&self, quota_id: Uuid, delta: i64) -> Result<i64, CretoError>;

    /// Get current quota for a resource.
    async fn get_current(
        &self,
        org_id: OrganizationId,
        agent_id: Option<AgentId>,
        metric_code: &str,
    ) -> Result<Option<Quota>, CretoError>;

    /// List all quotas for an organization.
    async fn list_by_org(&self, org_id: OrganizationId) -> Result<Vec<Quota>, CretoError>;
}

/// PostgreSQL implementation of QuotaRepository.
pub struct PgQuotaRepository {
    pool: PgPool,
}

impl PgQuotaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl QuotaRepository for PgQuotaRepository {
    async fn get_or_create(
        &self,
        org_id: OrganizationId,
        agent_id: Option<AgentId>,
        metric_code: &str,
        period: QuotaPeriod,
        limit: i64,
    ) -> Result<Quota, CretoError> {
        let (period_start, period_end) = period.current_bounds();

        let row = sqlx::query(
            r#"
            INSERT INTO quotas (organization_id, agent_id, resource, limit_value, period, period_start, period_end)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (organization_id, agent_id, resource, period_start)
            DO UPDATE SET updated_at = NOW()
            RETURNING id, current_usage
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(agent_id.map(|a| *a.as_uuid()))
        .bind(metric_code)
        .bind(limit)
        .bind(period.as_str())
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(Quota {
            id: row.get("id"),
            organization_id: org_id,
            agent_id,
            metric_code: metric_code.to_string(),
            limit,
            period,
            current_usage: row.get("current_usage"),
            period_start,
            period_end,
            allow_overage: false,
            budget_cents: None,
        })
    }

    async fn increment_usage(&self, quota_id: Uuid, delta: i64) -> Result<i64, CretoError> {
        let row = sqlx::query(
            r#"
            UPDATE quotas
            SET current_usage = current_usage + $2, updated_at = NOW()
            WHERE id = $1
            RETURNING current_usage
            "#,
        )
        .bind(quota_id)
        .bind(delta)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("current_usage"))
    }

    async fn get_current(
        &self,
        org_id: OrganizationId,
        agent_id: Option<AgentId>,
        metric_code: &str,
    ) -> Result<Option<Quota>, CretoError> {
        let now = Utc::now();

        let row = sqlx::query(
            r#"
            SELECT id, limit_value, current_usage, period, period_start, period_end
            FROM quotas
            WHERE organization_id = $1
              AND (agent_id = $2 OR (agent_id IS NULL AND $2 IS NULL))
              AND resource = $3
              AND period_start <= $4
              AND period_end > $4
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(agent_id.map(|a| *a.as_uuid()))
        .bind(metric_code)
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.map(|r| {
            let period_str: String = r.get("period");
            Quota {
                id: r.get("id"),
                organization_id: org_id,
                agent_id,
                metric_code: metric_code.to_string(),
                limit: r.get("limit_value"),
                current_usage: r.get("current_usage"),
                period: parse_period(&period_str),
                period_start: r.get("period_start"),
                period_end: r.get("period_end"),
                allow_overage: false,
                budget_cents: None,
            }
        }))
    }

    async fn list_by_org(&self, org_id: OrganizationId) -> Result<Vec<Quota>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id, agent_id, resource, limit_value, current_usage, period, period_start, period_end
            FROM quotas
            WHERE organization_id = $1
            ORDER BY resource, period_start DESC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| {
                let period_str: String = r.get("period");
                Quota {
                    id: r.get("id"),
                    organization_id: org_id,
                    agent_id: r.get::<Option<Uuid>, _>("agent_id").map(AgentId::from_uuid),
                    metric_code: r.get("resource"),
                    limit: r.get("limit_value"),
                    current_usage: r.get("current_usage"),
                    period: parse_period(&period_str),
                    period_start: r.get("period_start"),
                    period_end: r.get("period_end"),
                    allow_overage: false,
                    budget_cents: None,
                }
            })
            .collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Invoice Repository (Simplified)
// ─────────────────────────────────────────────────────────────────────────────

/// Simplified invoice repository - invoices are complex and will be fleshed out later.
#[trait_variant::make(InvoiceRepository: Send)]
pub trait LocalInvoiceRepository {
    /// Create a new invoice record.
    async fn create_invoice_record(
        &self,
        org_id: OrganizationId,
        invoice_number: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        total_cents: i64,
    ) -> Result<Uuid, CretoError>;

    /// Get invoice by ID.
    async fn get_invoice(&self, id: Uuid) -> Result<Option<InvoiceRecord>, CretoError>;

    /// List invoices by organization.
    async fn list_by_org(&self, org_id: OrganizationId) -> Result<Vec<InvoiceRecord>, CretoError>;
}

/// Simplified invoice record for database storage.
#[derive(Debug, Clone)]
pub struct InvoiceRecord {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub invoice_number: String,
    pub status: String,
    pub total_cents: i64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// PostgreSQL implementation of InvoiceRepository.
pub struct PgInvoiceRepository {
    pool: PgPool,
}

impl PgInvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl InvoiceRepository for PgInvoiceRepository {
    async fn create_invoice_record(
        &self,
        org_id: OrganizationId,
        invoice_number: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        total_cents: i64,
    ) -> Result<Uuid, CretoError> {
        let row = sqlx::query(
            r#"
            INSERT INTO invoices (
                organization_id, invoice_number, status, currency,
                subtotal_cents, tax_cents, discount_cents, total_cents,
                period_start, period_end
            ) VALUES ($1, $2, 'draft', 'USD', $3, 0, 0, $3, $4, $5)
            RETURNING id
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(invoice_number)
        .bind(total_cents)
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn get_invoice(&self, id: Uuid) -> Result<Option<InvoiceRecord>, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT organization_id, invoice_number, status, total_cents,
                   period_start, period_end, created_at
            FROM invoices
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.map(|r| InvoiceRecord {
            id,
            organization_id: OrganizationId::from_uuid(r.get::<Uuid, _>("organization_id")),
            invoice_number: r.get("invoice_number"),
            status: r.get("status"),
            total_cents: r.get("total_cents"),
            period_start: r.get("period_start"),
            period_end: r.get("period_end"),
            created_at: r.get("created_at"),
        }))
    }

    async fn list_by_org(&self, org_id: OrganizationId) -> Result<Vec<InvoiceRecord>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id, invoice_number, status, total_cents, period_start, period_end, created_at
            FROM invoices
            WHERE organization_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| InvoiceRecord {
                id: r.get("id"),
                organization_id: org_id,
                invoice_number: r.get("invoice_number"),
                status: r.get("status"),
                total_cents: r.get("total_cents"),
                period_start: r.get("period_start"),
                period_end: r.get("period_end"),
                created_at: r.get("created_at"),
            })
            .collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────────────

fn parse_period(s: &str) -> QuotaPeriod {
    match s {
        "hourly" => QuotaPeriod::Hourly,
        "daily" => QuotaPeriod::Daily,
        "weekly" => QuotaPeriod::Weekly,
        "monthly" => QuotaPeriod::Monthly,
        "lifetime" => QuotaPeriod::Lifetime,
        _ => QuotaPeriod::Monthly,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_roundtrip() {
        let event_type = UsageEventType::ApiCall;
        let s = event_type.as_db_str();
        let parsed = UsageEventType::from_db_str(s);
        assert_eq!(parsed, Some(UsageEventType::ApiCall));
    }

    #[test]
    fn test_period_as_str() {
        assert_eq!(QuotaPeriod::Hourly.as_str(), "hourly");
        assert_eq!(QuotaPeriod::Monthly.as_str(), "monthly");
    }

    #[test]
    fn test_parse_period() {
        assert_eq!(parse_period("hourly"), QuotaPeriod::Hourly);
        assert_eq!(parse_period("invalid"), QuotaPeriod::Monthly);
    }
}
