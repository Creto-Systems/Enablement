//! Integration tests for creto-runtime.
//!
//! These tests verify the runtime service functionality including
//! sandbox management, execution tracking, and resource limits.

use creto_common::{AgentId, OrganizationId};
use creto_integration_tests::common::TestFixture;
use creto_runtime::{
    Sandbox, SandboxConfig, SandboxState, SandboxId,
    ExecutionRequest, ExecutionStatus, ExecutionTiming, ExecutionResult, ExecutionError,
    ResourceLimits, ResourceUsage,
    PoolConfig, WarmPool,
};

#[test]
fn test_sandbox_creation() {
    let fixture = TestFixture::new();
    let sandbox = Sandbox::new(
        fixture.org_id,
        fixture.agent_id,
        SandboxConfig::default(),
    );

    assert_eq!(sandbox.state, SandboxState::Creating);
    assert!(sandbox.runtime_handle.is_none());
}

#[test]
fn test_sandbox_state_transitions() {
    let fixture = TestFixture::new();
    let mut sandbox = Sandbox::new(
        fixture.org_id,
        fixture.agent_id,
        SandboxConfig::default(),
    );

    sandbox.mark_ready("handle_123".to_string());
    assert_eq!(sandbox.state, SandboxState::Ready);
    assert!(sandbox.state.can_execute());

    sandbox.mark_running();
    assert_eq!(sandbox.state, SandboxState::Running);

    sandbox.mark_terminated();
    assert_eq!(sandbox.state, SandboxState::Terminated);
    assert!(sandbox.state.is_terminal());
}

#[test]
fn test_sandbox_id() {
    let id = SandboxId::new();
    let uuid = id.as_uuid();

    // Verify it's a valid UUID
    assert!(!uuid.is_nil());

    // Test from_uuid roundtrip
    let id2 = SandboxId::from_uuid(uuid);
    assert_eq!(id.as_uuid(), id2.as_uuid());
}

#[test]
fn test_execution_request_builder() {
    let sandbox_id = SandboxId::new();
    let request = ExecutionRequest::new(sandbox_id, "print('hello')")
        .with_entry_point("main")
        .with_timeout(60);

    assert_eq!(request.code, "print('hello')");
    assert_eq!(request.entry_point, Some("main".to_string()));
    assert_eq!(request.timeout_seconds, Some(60));
}

#[test]
fn test_execution_status_values() {
    // Verify all status values exist
    let _ = ExecutionStatus::Queued;
    let _ = ExecutionStatus::Running;
    let _ = ExecutionStatus::Completed;
    let _ = ExecutionStatus::Failed;
    let _ = ExecutionStatus::TimedOut;
    let _ = ExecutionStatus::Cancelled;
}

#[test]
fn test_execution_timing() {
    let mut timing = ExecutionTiming::new();
    assert!(timing.started_at.is_none());

    timing.mark_started();
    assert!(timing.started_at.is_some());

    timing.mark_completed();
    assert!(timing.completed_at.is_some());
    assert!(timing.duration_ms.is_some());
}

#[test]
fn test_execution_error_helpers() {
    let timeout_error = ExecutionError::timeout(300);
    assert_eq!(timeout_error.code, "TIMEOUT");
    assert!(timeout_error.message.contains("300"));

    let not_found = ExecutionError::sandbox_not_found("sandbox_123");
    assert_eq!(not_found.code, "SANDBOX_NOT_FOUND");
    assert!(not_found.message.contains("sandbox_123"));
}

#[test]
fn test_execution_result_success() {
    let request_id = uuid::Uuid::now_v7();
    let timing = ExecutionTiming::new();
    let result = ExecutionResult::success(
        request_id,
        serde_json::json!({"output": "hello"}),
        timing,
    );

    assert!(result.is_success());
    assert_eq!(result.status, ExecutionStatus::Completed);
    assert!(result.error.is_none());
}

#[test]
fn test_execution_result_failure() {
    let request_id = uuid::Uuid::now_v7();
    let timing = ExecutionTiming::new();
    let error = ExecutionError::new("TEST_ERROR", "Test failure");
    let result = ExecutionResult::failure(request_id, error, timing);

    assert!(!result.is_success());
    assert_eq!(result.status, ExecutionStatus::Failed);
    assert!(result.error.is_some());
}

#[test]
fn test_resource_limits_default() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.memory_bytes, 512 * 1024 * 1024); // 512 MB
    assert_eq!(limits.wall_time_seconds, 300); // 5 minutes
}

#[test]
fn test_resource_limits_presets() {
    let minimal = ResourceLimits::minimal();
    assert_eq!(minimal.memory_bytes, 128 * 1024 * 1024); // 128 MB

    let generous = ResourceLimits::generous();
    assert_eq!(generous.memory_bytes, 4 * 1024 * 1024 * 1024); // 4 GB
}

#[test]
fn test_resource_limits_builder() {
    let limits = ResourceLimits::minimal()
        .with_memory(256 * 1024 * 1024)
        .with_wall_time(60);

    assert_eq!(limits.memory_bytes, 256 * 1024 * 1024);
    assert_eq!(limits.wall_time_seconds, 60);
}

#[test]
fn test_resource_usage_exceeds() {
    let limits = ResourceLimits::minimal();
    let usage = ResourceUsage {
        memory_bytes: 200 * 1024 * 1024, // 200 MB > 128 MB limit
        ..Default::default()
    };

    let violation = usage.exceeds(&limits);
    assert!(violation.is_some());
}

#[test]
fn test_resource_usage_within_limits() {
    let limits = ResourceLimits::default();
    let usage = ResourceUsage {
        memory_bytes: 100 * 1024 * 1024,
        cpu_time_ms: 1000,
        wall_time_ms: 5000,
        ..Default::default()
    };

    assert!(usage.exceeds(&limits).is_none());
}

#[test]
fn test_pool_config_default() {
    let config = PoolConfig::default();
    assert!(config.max_warm >= config.min_warm);
    assert!(!config.runtimes.is_empty());
}

#[tokio::test]
async fn test_warm_pool_creation() {
    let pool = WarmPool::new(PoolConfig::default());
    let stats = pool.stats().await;
    assert_eq!(stats.total, 0);
}

#[tokio::test]
async fn test_warm_pool_miss() {
    let pool = WarmPool::new(PoolConfig::default());

    // Try to acquire from empty pool
    let acquired = pool.acquire("python3.11").await;
    assert!(acquired.is_none());

    let stats = pool.stats().await;
    assert_eq!(stats.misses, 1);
}

#[tokio::test]
async fn test_warm_pool_add_and_acquire() {
    let pool = WarmPool::new(PoolConfig::default());

    // Create and add a sandbox
    let mut sandbox = Sandbox::new(
        OrganizationId::new(),
        AgentId::new(),
        SandboxConfig {
            runtime: "python3.11".to_string(),
            ..Default::default()
        },
    );
    sandbox.mark_ready("handle_123".to_string());

    pool.add(sandbox).await.unwrap();

    let stats = pool.stats().await;
    assert_eq!(stats.total, 1);
    assert_eq!(stats.ready, 1);

    // Acquire the sandbox
    let acquired = pool.acquire("python3.11").await;
    assert!(acquired.is_some());

    let stats = pool.stats().await;
    assert_eq!(stats.ready, 0);
    assert_eq!(stats.in_use, 1);
    assert_eq!(stats.hits, 1);
}
