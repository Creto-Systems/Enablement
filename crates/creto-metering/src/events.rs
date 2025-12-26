//! Usage event types and ingestion.
//!
//! Events are the atomic unit of metering. Each event represents a single
//! billable action performed by an agent.

use chrono::{DateTime, Utc};
use creto_common::{AgentId, OrganizationId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A usage event representing a billable action.
///
/// Follows the Lago event schema with Creto extensions for NHI and delegation.
///
/// # Idempotency
///
/// The `transaction_id` field ensures idempotent ingestion. Duplicate events
/// with the same transaction ID are silently ignored.
///
/// # Example
///
/// ```
/// use creto_metering::{UsageEvent, UsageEventType};
///
/// let event = UsageEvent::builder()
///     .event_type(UsageEventType::ApiCall)
///     .quantity(1)
///     .build();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    /// Unique transaction ID for idempotency.
    pub transaction_id: String,

    /// Organization (tenant) this event belongs to.
    pub organization_id: OrganizationId,

    /// Agent that generated this event.
    pub agent_id: AgentId,

    /// External subscription ID for billing system integration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_subscription_id: Option<String>,

    /// Type of usage event.
    pub event_type: UsageEventType,

    /// Billable metric code (e.g., "api_calls", "tokens_used").
    pub code: String,

    /// Quantity of the billable unit.
    pub quantity: i64,

    /// When the event occurred.
    pub timestamp: DateTime<Utc>,

    /// Additional properties for filtering and attribution.
    #[serde(default)]
    pub properties: serde_json::Value,

    /// Delegation depth when event was generated.
    #[serde(default)]
    pub delegation_depth: u8,
}

impl UsageEvent {
    /// Create a new usage event builder.
    pub fn builder() -> UsageEventBuilder {
        UsageEventBuilder::default()
    }

    /// Generate a unique transaction ID.
    pub fn generate_transaction_id() -> String {
        Uuid::now_v7().to_string()
    }
}

/// Builder for constructing usage events.
#[derive(Default)]
pub struct UsageEventBuilder {
    transaction_id: Option<String>,
    organization_id: Option<OrganizationId>,
    agent_id: Option<AgentId>,
    external_subscription_id: Option<String>,
    event_type: Option<UsageEventType>,
    code: Option<String>,
    quantity: Option<i64>,
    timestamp: Option<DateTime<Utc>>,
    properties: serde_json::Value,
    delegation_depth: u8,
}

impl UsageEventBuilder {
    /// Set the transaction ID (auto-generated if not provided).
    pub fn transaction_id(mut self, id: impl Into<String>) -> Self {
        self.transaction_id = Some(id.into());
        self
    }

    /// Set the organization ID.
    pub fn organization_id(mut self, id: OrganizationId) -> Self {
        self.organization_id = Some(id);
        self
    }

    /// Set the agent ID.
    pub fn agent_id(mut self, id: AgentId) -> Self {
        self.agent_id = Some(id);
        self
    }

    /// Set the external subscription ID.
    pub fn external_subscription_id(mut self, id: impl Into<String>) -> Self {
        self.external_subscription_id = Some(id.into());
        self
    }

    /// Set the event type.
    pub fn event_type(mut self, event_type: UsageEventType) -> Self {
        self.event_type = Some(event_type);
        self
    }

    /// Set the billable metric code.
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Set the quantity.
    pub fn quantity(mut self, quantity: i64) -> Self {
        self.quantity = Some(quantity);
        self
    }

    /// Set the timestamp.
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Set additional properties.
    pub fn properties(mut self, properties: serde_json::Value) -> Self {
        self.properties = properties;
        self
    }

    /// Set the delegation depth.
    pub fn delegation_depth(mut self, depth: u8) -> Self {
        self.delegation_depth = depth;
        self
    }

    /// Build the usage event.
    ///
    /// # Panics
    ///
    /// Panics if required fields are not set.
    pub fn build(self) -> UsageEvent {
        let event_type = self.event_type.expect("event_type is required");

        UsageEvent {
            transaction_id: self.transaction_id
                .unwrap_or_else(UsageEvent::generate_transaction_id),
            organization_id: self.organization_id
                .unwrap_or_default(),
            agent_id: self.agent_id
                .unwrap_or_default(),
            external_subscription_id: self.external_subscription_id,
            event_type,
            code: self.code.unwrap_or_else(|| event_type.default_code().to_string()),
            quantity: self.quantity.unwrap_or(1),
            timestamp: self.timestamp.unwrap_or_else(Utc::now),
            properties: self.properties,
            delegation_depth: self.delegation_depth,
        }
    }
}

/// Types of usage events that can be metered.
///
/// Based on the 13 event types defined in the SDD.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UsageEventType {
    // ─────────────────────────────────────────────────────────────────────────
    // API Usage
    // ─────────────────────────────────────────────────────────────────────────
    /// Generic API call.
    ApiCall,
    /// LLM inference request.
    LlmInference,
    /// Embedding generation.
    EmbeddingGeneration,

    // ─────────────────────────────────────────────────────────────────────────
    // Token Usage
    // ─────────────────────────────────────────────────────────────────────────
    /// Input tokens to LLM.
    InputTokens,
    /// Output tokens from LLM.
    OutputTokens,
    /// Total tokens (input + output).
    TotalTokens,

    // ─────────────────────────────────────────────────────────────────────────
    // Compute Usage
    // ─────────────────────────────────────────────────────────────────────────
    /// CPU time in milliseconds.
    CpuMilliseconds,
    /// Memory usage in megabyte-seconds.
    MemoryMbSeconds,
    /// GPU time in milliseconds.
    GpuMilliseconds,

    // ─────────────────────────────────────────────────────────────────────────
    // Storage Usage
    // ─────────────────────────────────────────────────────────────────────────
    /// Storage in bytes.
    StorageBytes,
    /// Network egress in bytes.
    NetworkEgressBytes,

    // ─────────────────────────────────────────────────────────────────────────
    // Enablement-Specific
    // ─────────────────────────────────────────────────────────────────────────
    /// Oversight approval request.
    OversightRequest,
    /// Sandbox execution.
    SandboxExecution,
    /// Encrypted message sent.
    MessageSent,
}

impl UsageEventType {
    /// Get the default billable metric code for this event type.
    pub fn default_code(&self) -> &'static str {
        match self {
            UsageEventType::ApiCall => "api_calls",
            UsageEventType::LlmInference => "llm_inferences",
            UsageEventType::EmbeddingGeneration => "embeddings",
            UsageEventType::InputTokens => "input_tokens",
            UsageEventType::OutputTokens => "output_tokens",
            UsageEventType::TotalTokens => "total_tokens",
            UsageEventType::CpuMilliseconds => "cpu_ms",
            UsageEventType::MemoryMbSeconds => "memory_mb_s",
            UsageEventType::GpuMilliseconds => "gpu_ms",
            UsageEventType::StorageBytes => "storage_bytes",
            UsageEventType::NetworkEgressBytes => "network_egress_bytes",
            UsageEventType::OversightRequest => "oversight_requests",
            UsageEventType::SandboxExecution => "sandbox_executions",
            UsageEventType::MessageSent => "messages_sent",
        }
    }

    /// Get the unit name for display.
    pub fn unit_name(&self) -> &'static str {
        match self {
            UsageEventType::ApiCall => "calls",
            UsageEventType::LlmInference => "inferences",
            UsageEventType::EmbeddingGeneration => "embeddings",
            UsageEventType::InputTokens => "tokens",
            UsageEventType::OutputTokens => "tokens",
            UsageEventType::TotalTokens => "tokens",
            UsageEventType::CpuMilliseconds => "ms",
            UsageEventType::MemoryMbSeconds => "MB-s",
            UsageEventType::GpuMilliseconds => "ms",
            UsageEventType::StorageBytes => "bytes",
            UsageEventType::NetworkEgressBytes => "bytes",
            UsageEventType::OversightRequest => "requests",
            UsageEventType::SandboxExecution => "executions",
            UsageEventType::MessageSent => "messages",
        }
    }
}

/// Trait for ingesting usage events.
///
/// Implementations handle persistence and deduplication.
#[trait_variant::make(EventIngestion: Send)]
pub trait LocalEventIngestion {
    /// Ingest a single usage event.
    ///
    /// Returns `Ok(())` for successful ingestion or duplicate detection.
    /// Returns `Err` only for actual failures (DB errors, validation failures).
    async fn ingest(&self, event: UsageEvent) -> Result<(), creto_common::CretoError>;

    /// Ingest multiple events in a batch.
    ///
    /// More efficient than individual ingestion for bulk imports.
    async fn ingest_batch(&self, events: Vec<UsageEvent>) -> Result<usize, creto_common::CretoError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_builder() {
        let event = UsageEvent::builder()
            .event_type(UsageEventType::ApiCall)
            .quantity(5)
            .build();

        assert_eq!(event.quantity, 5);
        assert_eq!(event.code, "api_calls");
        assert!(!event.transaction_id.is_empty());
    }

    #[test]
    fn test_event_type_codes() {
        assert_eq!(UsageEventType::InputTokens.default_code(), "input_tokens");
        assert_eq!(UsageEventType::SandboxExecution.default_code(), "sandbox_executions");
    }
}
