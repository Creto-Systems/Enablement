//! Metering integration for usage tracking.
//!
//! This module provides integration with creto-metering to emit usage events
//! when oversight requests are created and resolved.

#[cfg(feature = "metering")]
use creto_common::{AgentId, OrganizationId};
#[cfg(feature = "metering")]
use creto_metering::{UsageEvent, UsageEventType};
#[cfg(feature = "metering")]
use uuid::Uuid;

/// Create a usage event for an oversight request creation.
#[cfg(feature = "metering")]
pub fn oversight_request_event(
    org_id: OrganizationId,
    agent_id: AgentId,
    request_id: Uuid,
    delegation_depth: u8,
) -> UsageEvent {
    let mut properties = serde_json::Map::new();
    properties.insert("request_id".to_string(), serde_json::json!(request_id.to_string()));

    UsageEvent {
        transaction_id: Uuid::now_v7().to_string(),
        organization_id: org_id,
        agent_id,
        external_subscription_id: None,
        event_type: UsageEventType::OversightRequest,
        code: "oversight_request".to_string(),
        quantity: 1,
        timestamp: chrono::Utc::now(),
        properties: serde_json::Value::Object(properties),
        delegation_depth,
    }
}

/// Metering event types for oversight actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OversightMeteringEvent {
    /// A new oversight request was created.
    RequestCreated,
    /// An oversight request was approved.
    RequestApproved,
    /// An oversight request was rejected.
    RequestRejected,
    /// An oversight request timed out.
    RequestTimedOut,
}

impl OversightMeteringEvent {
    /// Get the event code for metering.
    pub fn code(&self) -> &'static str {
        match self {
            OversightMeteringEvent::RequestCreated => "oversight_request_created",
            OversightMeteringEvent::RequestApproved => "oversight_request_approved",
            OversightMeteringEvent::RequestRejected => "oversight_request_rejected",
            OversightMeteringEvent::RequestTimedOut => "oversight_request_timed_out",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metering_event_codes() {
        assert_eq!(OversightMeteringEvent::RequestCreated.code(), "oversight_request_created");
        assert_eq!(OversightMeteringEvent::RequestApproved.code(), "oversight_request_approved");
    }

    #[cfg(feature = "metering")]
    #[test]
    fn test_oversight_request_event() {
        let org_id = OrganizationId::new();
        let agent_id = AgentId::new();
        let request_id = Uuid::now_v7();

        let event = oversight_request_event(org_id, agent_id, request_id, 1);

        assert_eq!(event.event_type, UsageEventType::OversightRequest);
        assert_eq!(event.code, "oversight_request");
        assert_eq!(event.quantity, 1);
    }
}
