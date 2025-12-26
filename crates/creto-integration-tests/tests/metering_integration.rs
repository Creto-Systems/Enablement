//! Integration tests for creto-metering.
//!
//! These tests verify the metering service functionality including
//! event ingestion, quota enforcement, and usage tracking.

use creto_integration_tests::common::TestFixture;
use creto_metering::{UsageEvent, UsageEventType, QuotaPeriod, Quota, QuotaEnforcer};
use creto_common::OrganizationId;

#[test]
fn test_usage_event_builder() {
    let fixture = TestFixture::new();
    let event = UsageEvent::builder()
        .organization_id(fixture.org_id)
        .agent_id(fixture.agent_id)
        .event_type(UsageEventType::ApiCall)
        .quantity(5)
        .build();

    assert_eq!(event.event_type, UsageEventType::ApiCall);
    assert_eq!(event.quantity, 5);
    assert_eq!(event.delegation_depth, 0);
}

#[test]
fn test_usage_event_type_codes() {
    // Test default_code method
    assert_eq!(UsageEventType::ApiCall.default_code(), "api_calls");
    assert_eq!(UsageEventType::InputTokens.default_code(), "input_tokens");
    assert_eq!(UsageEventType::OutputTokens.default_code(), "output_tokens");
    assert_eq!(UsageEventType::SandboxExecution.default_code(), "sandbox_executions");
    assert_eq!(UsageEventType::OversightRequest.default_code(), "oversight_requests");
    assert_eq!(UsageEventType::MessageSent.default_code(), "messages_sent");
}

#[test]
fn test_usage_event_type_units() {
    assert_eq!(UsageEventType::ApiCall.unit_name(), "calls");
    assert_eq!(UsageEventType::InputTokens.unit_name(), "tokens");
    assert_eq!(UsageEventType::StorageBytes.unit_name(), "bytes");
    assert_eq!(UsageEventType::CpuMilliseconds.unit_name(), "ms");
}

#[test]
fn test_quota_period_values() {
    // Test that quota periods are properly defined
    let _ = QuotaPeriod::Hourly;
    let _ = QuotaPeriod::Daily;
    let _ = QuotaPeriod::Weekly;
    let _ = QuotaPeriod::Monthly;
    let _ = QuotaPeriod::Lifetime;
}

#[test]
fn test_event_with_delegation_depth() {
    let fixture = TestFixture::new();
    let event = UsageEvent::builder()
        .organization_id(fixture.org_id)
        .agent_id(fixture.agent_id)
        .event_type(UsageEventType::LlmInference)
        .quantity(100)
        .delegation_depth(3)
        .build();

    assert_eq!(event.delegation_depth, 3);
    assert_eq!(event.quantity, 100);
}

#[test]
fn test_event_with_properties() {
    let fixture = TestFixture::new();
    let event = UsageEvent::builder()
        .organization_id(fixture.org_id)
        .agent_id(fixture.agent_id)
        .event_type(UsageEventType::LlmInference)
        .properties(serde_json::json!({
            "model": "claude-3",
            "provider": "anthropic"
        }))
        .build();

    assert_eq!(event.properties["model"], "claude-3");
    assert_eq!(event.properties["provider"], "anthropic");
}

#[test]
fn test_transaction_id_generation() {
    let id1 = UsageEvent::generate_transaction_id();
    let id2 = UsageEvent::generate_transaction_id();

    // Should be unique
    assert_ne!(id1, id2);
    // Should be valid UUIDs (36 chars with hyphens)
    assert_eq!(id1.len(), 36);
}

#[test]
fn test_all_event_types() {
    // Ensure all event types are accessible
    let types = vec![
        UsageEventType::ApiCall,
        UsageEventType::LlmInference,
        UsageEventType::EmbeddingGeneration,
        UsageEventType::InputTokens,
        UsageEventType::OutputTokens,
        UsageEventType::TotalTokens,
        UsageEventType::CpuMilliseconds,
        UsageEventType::MemoryMbSeconds,
        UsageEventType::GpuMilliseconds,
        UsageEventType::StorageBytes,
        UsageEventType::NetworkEgressBytes,
        UsageEventType::OversightRequest,
        UsageEventType::SandboxExecution,
        UsageEventType::MessageSent,
    ];

    // All should have default codes
    for t in types {
        assert!(!t.default_code().is_empty());
        assert!(!t.unit_name().is_empty());
    }
}

#[test]
fn test_quota_creation() {
    let org_id = OrganizationId::new();
    let quota = Quota::new(org_id, "api_calls", 1000, QuotaPeriod::Daily);

    assert_eq!(quota.organization_id, org_id);
    assert_eq!(quota.metric_code, "api_calls");
    assert_eq!(quota.limit, 1000);
    assert_eq!(quota.period, QuotaPeriod::Daily);
    assert_eq!(quota.current_usage, 0);
}

#[test]
fn test_quota_would_exceed() {
    let org_id = OrganizationId::new();
    let mut quota = Quota::new(org_id, "api_calls", 100, QuotaPeriod::Daily);

    assert!(!quota.would_exceed(50));
    quota.current_usage = 90;
    assert!(!quota.would_exceed(10));
    assert!(quota.would_exceed(11));
}

#[test]
fn test_quota_remaining() {
    let org_id = OrganizationId::new();
    let mut quota = Quota::new(org_id, "tokens", 1000, QuotaPeriod::Monthly);

    assert_eq!(quota.remaining(), 1000);
    quota.current_usage = 400;
    assert_eq!(quota.remaining(), 600);
    quota.current_usage = 1200; // Over limit
    assert_eq!(quota.remaining(), 0);
}

#[test]
fn test_quota_usage_percentage() {
    let org_id = OrganizationId::new();
    let mut quota = Quota::new(org_id, "tokens", 1000, QuotaPeriod::Monthly);

    assert_eq!(quota.usage_percentage(), 0.0);
    quota.current_usage = 500;
    assert_eq!(quota.usage_percentage(), 0.5);
    quota.current_usage = 1000;
    assert_eq!(quota.usage_percentage(), 1.0);
}

#[test]
fn test_quota_reset() {
    let org_id = OrganizationId::new();
    let mut quota = Quota::new(org_id, "api_calls", 100, QuotaPeriod::Daily);
    quota.current_usage = 50;

    quota.reset();
    assert_eq!(quota.current_usage, 0);
}

#[test]
fn test_period_bounds_daily() {
    let timestamp = chrono::Utc::now();
    let (start, end) = QuotaPeriod::Daily.calculate_bounds(timestamp);

    assert!(start <= timestamp);
    assert!(timestamp < end);
    assert_eq!((end - start).num_hours(), 24);
}

#[test]
fn test_period_bounds_hourly() {
    let timestamp = chrono::Utc::now();
    let (start, end) = QuotaPeriod::Hourly.calculate_bounds(timestamp);

    assert!(start <= timestamp);
    assert!(timestamp < end);
    assert_eq!((end - start).num_minutes(), 60);
}

#[test]
fn test_quota_enforcer_creation() {
    let enforcer = QuotaEnforcer::new();
    // Just verify it can be created
    let _ = enforcer;
}

#[tokio::test]
async fn test_quota_enforcer_check_allows() {
    let enforcer = QuotaEnforcer::new();
    let fixture = TestFixture::new();

    // Default implementation allows all
    let result = enforcer.check(
        fixture.org_id,
        fixture.agent_id,
        "api_calls",
        10,
    ).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_quota_enforcer_get_status() {
    let enforcer = QuotaEnforcer::new();
    let fixture = TestFixture::new();

    let status = enforcer.get_status(
        fixture.org_id,
        fixture.agent_id,
        "api_calls",
    ).await.unwrap();

    assert_eq!(status.metric_code, "api_calls");
    assert!(status.limit > 0);
}
