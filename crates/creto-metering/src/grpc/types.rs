//! gRPC message types for the metering service.
//!
//! These types mirror the proto definitions but are defined in Rust for simplicity.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::events::{UsageEvent, UsageEventType};
use creto_common::{AgentId, OrganizationId};

/// Request to ingest a single event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestEventRequest {
    pub event: GrpcUsageEvent,
}

/// Response from ingesting a single event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestEventResponse {
    pub success: bool,
    pub status: IngestStatus,
    pub error_message: Option<String>,
}

/// Request to ingest a batch of events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestEventBatchRequest {
    pub events: Vec<GrpcUsageEvent>,
    pub continue_on_error: bool,
}

/// Response from batch ingestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestEventBatchResponse {
    pub accepted_count: u32,
    pub duplicate_count: u32,
    pub failed_count: u32,
    pub results: Vec<EventResult>,
}

/// Status of an individual event ingestion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[derive(Default)]
pub enum IngestStatus {
    #[default]
    Unspecified,
    Accepted,
    Duplicate,
    ValidationError,
    QuotaExceeded,
    InternalError,
}


/// Result for a specific event in a batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventResult {
    pub index: u32,
    pub status: IngestStatus,
    pub error_message: Option<String>,
}

/// gRPC representation of a usage event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcUsageEvent {
    pub transaction_id: String,
    pub organization_id: String,
    pub agent_id: String,
    pub external_subscription_id: Option<String>,
    pub event_type: GrpcUsageEventType,
    pub code: String,
    pub quantity: i64,
    pub timestamp: Option<DateTime<Utc>>,
    pub properties: Option<serde_json::Value>,
    pub delegation_depth: u32,
}

impl GrpcUsageEvent {
    /// Convert to internal UsageEvent type.
    pub fn to_usage_event(&self) -> Result<UsageEvent, String> {
        let org_id = uuid::Uuid::parse_str(&self.organization_id)
            .map_err(|e| format!("Invalid organization_id: {}", e))?;
        let agent_id = uuid::Uuid::parse_str(&self.agent_id)
            .map_err(|e| format!("Invalid agent_id: {}", e))?;

        Ok(UsageEvent {
            transaction_id: self.transaction_id.clone(),
            organization_id: OrganizationId::from_uuid(org_id),
            agent_id: AgentId::from_uuid(agent_id),
            external_subscription_id: self.external_subscription_id.clone(),
            event_type: self.event_type.to_internal(),
            code: self.code.clone(),
            quantity: self.quantity,
            timestamp: self.timestamp.unwrap_or_else(Utc::now),
            properties: self.properties.clone().unwrap_or(serde_json::json!({})),
            delegation_depth: self.delegation_depth as u8,
        })
    }
}

impl From<UsageEvent> for GrpcUsageEvent {
    fn from(event: UsageEvent) -> Self {
        Self {
            transaction_id: event.transaction_id,
            organization_id: event.organization_id.as_uuid().to_string(),
            agent_id: event.agent_id.as_uuid().to_string(),
            external_subscription_id: event.external_subscription_id,
            event_type: GrpcUsageEventType::from_internal(event.event_type),
            code: event.code,
            quantity: event.quantity,
            timestamp: Some(event.timestamp),
            properties: Some(event.properties),
            delegation_depth: event.delegation_depth as u32,
        }
    }
}

/// gRPC event type enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
#[derive(Default)]
pub enum GrpcUsageEventType {
    #[default]
    Unspecified = 0,
    ApiCall = 1,
    LlmInference = 2,
    EmbeddingGeneration = 3,
    InputTokens = 10,
    OutputTokens = 11,
    TotalTokens = 12,
    CpuMilliseconds = 20,
    MemoryMbSeconds = 21,
    GpuMilliseconds = 22,
    StorageBytes = 30,
    NetworkEgressBytes = 31,
    OversightRequest = 40,
    SandboxExecution = 41,
    MessageSent = 42,
}


impl GrpcUsageEventType {
    pub fn to_internal(self) -> UsageEventType {
        match self {
            Self::Unspecified => UsageEventType::ApiCall, // Default fallback
            Self::ApiCall => UsageEventType::ApiCall,
            Self::LlmInference => UsageEventType::LlmInference,
            Self::EmbeddingGeneration => UsageEventType::EmbeddingGeneration,
            Self::InputTokens => UsageEventType::InputTokens,
            Self::OutputTokens => UsageEventType::OutputTokens,
            Self::TotalTokens => UsageEventType::TotalTokens,
            Self::CpuMilliseconds => UsageEventType::CpuMilliseconds,
            Self::MemoryMbSeconds => UsageEventType::MemoryMbSeconds,
            Self::GpuMilliseconds => UsageEventType::GpuMilliseconds,
            Self::StorageBytes => UsageEventType::StorageBytes,
            Self::NetworkEgressBytes => UsageEventType::NetworkEgressBytes,
            Self::OversightRequest => UsageEventType::OversightRequest,
            Self::SandboxExecution => UsageEventType::SandboxExecution,
            Self::MessageSent => UsageEventType::MessageSent,
        }
    }

    pub fn from_internal(event_type: UsageEventType) -> Self {
        match event_type {
            UsageEventType::ApiCall => Self::ApiCall,
            UsageEventType::LlmInference => Self::LlmInference,
            UsageEventType::EmbeddingGeneration => Self::EmbeddingGeneration,
            UsageEventType::InputTokens => Self::InputTokens,
            UsageEventType::OutputTokens => Self::OutputTokens,
            UsageEventType::TotalTokens => Self::TotalTokens,
            UsageEventType::CpuMilliseconds => Self::CpuMilliseconds,
            UsageEventType::MemoryMbSeconds => Self::MemoryMbSeconds,
            UsageEventType::GpuMilliseconds => Self::GpuMilliseconds,
            UsageEventType::StorageBytes => Self::StorageBytes,
            UsageEventType::NetworkEgressBytes => Self::NetworkEgressBytes,
            UsageEventType::OversightRequest => Self::OversightRequest,
            UsageEventType::SandboxExecution => Self::SandboxExecution,
            UsageEventType::MessageSent => Self::MessageSent,
        }
    }
}

/// Request to check quota.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckQuotaRequest {
    pub organization_id: String,
    pub agent_id: Option<String>,
    pub metric_code: String,
    pub quantity: i64,
}

/// Response from quota check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckQuotaResponse {
    pub allowed: bool,
    pub current_usage: i64,
    pub limit: i64,
    pub remaining: i64,
    pub denial_reason: Option<String>,
}

/// Request to get quota status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetQuotaStatusRequest {
    pub organization_id: String,
    pub agent_id: Option<String>,
    pub metric_code: String,
}

/// Response with quota status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetQuotaStatusResponse {
    pub metric_code: String,
    pub limit: i64,
    pub current_usage: i64,
    pub remaining: i64,
    pub usage_percentage: f64,
    pub period: GrpcQuotaPeriod,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}

/// gRPC quota period enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
#[derive(Default)]
pub enum GrpcQuotaPeriod {
    #[default]
    Unspecified = 0,
    Hourly = 1,
    Daily = 2,
    Weekly = 3,
    Monthly = 4,
    Lifetime = 5,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_type_roundtrip() {
        for event_type in [
            UsageEventType::ApiCall,
            UsageEventType::LlmInference,
            UsageEventType::InputTokens,
            UsageEventType::SandboxExecution,
        ] {
            let grpc = GrpcUsageEventType::from_internal(event_type);
            let back = grpc.to_internal();
            assert_eq!(event_type, back);
        }
    }

    #[test]
    fn test_grpc_event_conversion() {
        let grpc_event = GrpcUsageEvent {
            transaction_id: "txn_123".to_string(),
            organization_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            agent_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            external_subscription_id: None,
            event_type: GrpcUsageEventType::ApiCall,
            code: "api_calls".to_string(),
            quantity: 5,
            timestamp: None,
            properties: None,
            delegation_depth: 0,
        };

        let usage_event = grpc_event.to_usage_event().unwrap();
        assert_eq!(usage_event.transaction_id, "txn_123");
        assert_eq!(usage_event.quantity, 5);
        assert_eq!(usage_event.event_type, UsageEventType::ApiCall);
    }

    #[test]
    fn test_invalid_uuid_conversion() {
        let grpc_event = GrpcUsageEvent {
            transaction_id: "txn_123".to_string(),
            organization_id: "not-a-uuid".to_string(),
            agent_id: "550e8400-e29b-41d4-a716-446655440001".to_string(),
            external_subscription_id: None,
            event_type: GrpcUsageEventType::ApiCall,
            code: "api_calls".to_string(),
            quantity: 5,
            timestamp: None,
            properties: None,
            delegation_depth: 0,
        };

        let result = grpc_event.to_usage_event();
        assert!(result.is_err());
    }
}
