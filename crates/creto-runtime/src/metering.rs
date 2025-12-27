//! Metering integration for sandbox execution tracking.
//!
//! This module provides integration with creto-metering to emit usage events
//! when sandboxes are created and executions complete.

#[cfg(feature = "metering")]
use creto_common::{AgentId, OrganizationId};
#[cfg(feature = "metering")]
use creto_metering::{UsageEvent, UsageEventType};
#[cfg(feature = "metering")]
use uuid::Uuid;

/// Create a usage event for sandbox execution.
#[cfg(feature = "metering")]
pub fn sandbox_execution_event(
    org_id: OrganizationId,
    agent_id: AgentId,
    sandbox_id: Uuid,
    duration_ms: u64,
    delegation_depth: u8,
) -> UsageEvent {
    let mut properties = serde_json::Map::new();
    properties.insert(
        "sandbox_id".to_string(),
        serde_json::json!(sandbox_id.to_string()),
    );
    properties.insert("duration_ms".to_string(), serde_json::json!(duration_ms));

    UsageEvent {
        transaction_id: Uuid::now_v7().to_string(),
        organization_id: org_id,
        agent_id,
        external_subscription_id: None,
        event_type: UsageEventType::SandboxExecution,
        code: "sandbox_execution".to_string(),
        quantity: 1,
        timestamp: chrono::Utc::now(),
        properties: serde_json::Value::Object(properties),
        delegation_depth,
    }
}

/// Create CPU usage event.
#[cfg(feature = "metering")]
pub fn cpu_usage_event(
    org_id: OrganizationId,
    agent_id: AgentId,
    sandbox_id: Uuid,
    cpu_ms: u64,
    delegation_depth: u8,
) -> UsageEvent {
    let mut properties = serde_json::Map::new();
    properties.insert(
        "sandbox_id".to_string(),
        serde_json::json!(sandbox_id.to_string()),
    );

    UsageEvent {
        transaction_id: Uuid::now_v7().to_string(),
        organization_id: org_id,
        agent_id,
        external_subscription_id: None,
        event_type: UsageEventType::CpuMilliseconds,
        code: "cpu_milliseconds".to_string(),
        quantity: cpu_ms as i64,
        timestamp: chrono::Utc::now(),
        properties: serde_json::Value::Object(properties),
        delegation_depth,
    }
}

/// Metering event types for sandbox operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMeteringEvent {
    /// A sandbox was created.
    SandboxCreated,
    /// A code execution completed.
    ExecutionCompleted,
    /// CPU time consumed.
    CpuTime,
    /// Memory used.
    MemoryUsage,
}

impl RuntimeMeteringEvent {
    /// Get the event code for metering.
    pub fn code(&self) -> &'static str {
        match self {
            RuntimeMeteringEvent::SandboxCreated => "sandbox_created",
            RuntimeMeteringEvent::ExecutionCompleted => "execution_completed",
            RuntimeMeteringEvent::CpuTime => "cpu_milliseconds",
            RuntimeMeteringEvent::MemoryUsage => "memory_mb_seconds",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metering_event_codes() {
        assert_eq!(
            RuntimeMeteringEvent::SandboxCreated.code(),
            "sandbox_created"
        );
        assert_eq!(RuntimeMeteringEvent::CpuTime.code(), "cpu_milliseconds");
    }
}
