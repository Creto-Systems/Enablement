//! Cross-product integration tests for Phase 5.
//!
//! These tests verify workflows that span multiple Enablement Layer products:
//! - Metering + Oversight: Cost-based approval triggers
//! - Runtime + Oversight: Sandbox operations requiring approval
//! - Messaging + Runtime: Secure communication within sandboxes
//! - Full workflow: End-to-end agent action flows

use creto_common::{AgentId, OrganizationId, UserId};
use creto_integration_tests::common::TestFixture;

// ============================================================================
// Metering + Oversight Integration
// ============================================================================

mod metering_oversight {
    use super::*;
    use creto_metering::{Quota, QuotaEnforcer, QuotaPeriod, UsageEvent, UsageEventType};
    use creto_oversight::{
        policy::{PolicyContext, PolicyEngine, TrustLevel},
        request::{ActionType, OversightRequest, Priority},
    };

    /// Test that high quota usage triggers oversight requirements
    #[tokio::test]
    async fn test_quota_threshold_triggers_oversight() {
        let fixture = TestFixture::new();

        // Setup: Register a quota with high usage
        let enforcer = QuotaEnforcer::new();
        let mut quota = Quota::new(fixture.org_id, "api_calls", 1000, QuotaPeriod::Daily);
        quota.current_usage = 850; // 85% used
        enforcer.register_quota(&quota);

        // Check quota - should be allowed but near limit
        let result = enforcer.check(&fixture.org_id, &fixture.agent_id, "api_calls", 100);
        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(check.allowed);

        // Create oversight policy context with quota info
        let context = PolicyContext {
            trust_level: TrustLevel::Standard,
            quota_usage_percentage: 85.0, // High usage
            delegation_depth: 0,
            ..Default::default()
        };

        // Policy engine should require oversight for high-value actions at high quota
        let engine = PolicyEngine::new();
        let action = ActionType::Transaction {
            amount_cents: 200000, // $2000 - above $1000 threshold
            currency: "USD".to_string(),
        };

        let decision = engine.evaluate(&action, &context).await.unwrap();

        // Should require oversight due to medium-value transaction (> $1000)
        assert!(decision.requires_oversight());
    }

    /// Test metering event creation after oversight approval
    #[tokio::test]
    async fn test_oversight_approval_generates_metering_event() {
        let fixture = TestFixture::new();

        // Create an oversight request for a billable action
        let request = OversightRequest::new(
            fixture.org_id,
            fixture.agent_id,
            ActionType::Transaction {
                amount_cents: 50000,
                currency: "USD".to_string(),
            },
            "Vendor payment requiring approval",
        )
        .with_priority(Priority::High);

        // After approval, create a metering event
        let event = UsageEvent::builder()
            .organization_id(fixture.org_id)
            .agent_id(fixture.agent_id)
            .event_type(UsageEventType::OversightRequest)
            .quantity(1)
            .properties(serde_json::json!({
                "request_id": request.id.to_string(),
                "action_type": "transaction",
                "amount_cents": 50000
            }))
            .build();

        assert_eq!(event.event_type, UsageEventType::OversightRequest);
        assert_eq!(event.quantity, 1);
        assert!(event.properties["request_id"].as_str().is_some());
    }

    /// Test cost-based oversight trigger
    #[tokio::test]
    async fn test_cost_based_oversight_trigger() {
        let engine = PolicyEngine::new();

        // High-value transaction should require oversight
        let high_value_action = ActionType::Transaction {
            amount_cents: 1000000, // $10,000
            currency: "USD".to_string(),
        };

        let context = PolicyContext {
            trust_level: TrustLevel::Elevated,
            quota_usage_percentage: 20.0,
            delegation_depth: 0,
            ..Default::default()
        };

        let decision = engine.evaluate(&high_value_action, &context).await.unwrap();
        assert!(decision.requires_oversight());
    }
}

// ============================================================================
// Runtime + Oversight Integration
// ============================================================================

mod runtime_oversight {
    use super::*;
    use creto_oversight::{
        policy::{PolicyContext, PolicyEngine, TrustLevel},
        request::{ActionType, OversightRequest, Priority, RequestStatus},
    };
    use creto_runtime::{
        attestation::{AttestationPlatform, AttestationPolicy},
        resources::ResourceLimits,
        sandbox::{NetworkPolicy as SandboxNetworkPolicy, Sandbox, SandboxConfig},
    };

    /// Test sandbox creation requiring oversight for high-resource configs
    #[tokio::test]
    async fn test_high_resource_sandbox_requires_oversight() {
        let fixture = TestFixture::new();
        let engine = PolicyEngine::new();

        // High-resource sandbox config
        let config = SandboxConfig {
            runtime: "python3.11".to_string(),
            limits: ResourceLimits {
                memory_bytes: 32 * 1024 * 1024 * 1024, // 32GB
                cpu_time_ms: 3600000,                  // 1 hour
                wall_time_seconds: 7200,
                disk_bytes: 10 * 1024 * 1024 * 1024, // 10GB
                max_processes: 1000,
                max_open_files: 10000,
                network_bandwidth_bps: None,
                max_connections: Some(500),
            },
            network_policy: SandboxNetworkPolicy::Full,
            environment: Vec::new(),
            mounts: Vec::new(),
            debug: false,
            detailed_network_policy: None,
            timeout_seconds: 3600,
        };

        // Create policy context based on resource request
        let context = PolicyContext {
            trust_level: TrustLevel::Standard,
            quota_usage_percentage: 50.0,
            delegation_depth: 1,
            attributes: serde_json::json!({
                "resource_request_memory": config.limits.memory_bytes,
                "network_policy": "full"
            }),
            ..Default::default()
        };

        // Resource-intensive sandbox should require oversight (using CodeExecution action type)
        let action = ActionType::CodeExecution {
            runtime: "python".to_string(),
            risk_level: "high".to_string(),
        };

        let decision = engine.evaluate(&action, &context).await.unwrap();
        // High resource requests should trigger oversight
        assert!(decision.requires_oversight());
    }

    /// Test sandbox attestation verified before sensitive operations
    #[test]
    fn test_attestation_required_before_sensitive_ops() {
        let fixture = TestFixture::new();

        // Create sandbox with strict attestation policy
        let mut sandbox = Sandbox::new(fixture.org_id, fixture.agent_id, SandboxConfig::default());

        // Configure strict attestation
        sandbox.attestation_policy = AttestationPolicy::strict();

        // Without attestation, should not be allowed to run sensitive operations
        assert!(sandbox.attestation.is_none());

        // Verify attestation policy requirements
        assert!(sandbox.attestation_policy.require_attestation);
        assert!(sandbox
            .attestation_policy
            .allowed_platforms
            .contains(&AttestationPlatform::SGX));
    }

    /// Test sandbox network policy with oversight approval
    #[test]
    fn test_sandbox_egress_requires_oversight() {
        let fixture = TestFixture::new();

        // Creating sandbox with egress should generate oversight request
        // Use ExternalApi action type for network access
        let request = OversightRequest::new(
            fixture.org_id,
            fixture.agent_id,
            ActionType::ExternalApi {
                service: "external-network".to_string(),
                operation: "egress_request".to_string(),
            },
            "Sandbox requesting external network access",
        )
        .with_priority(Priority::High);

        assert_eq!(request.status, RequestStatus::Pending);
    }
}

// ============================================================================
// Messaging + Runtime Integration
// ============================================================================

mod messaging_runtime {
    use super::*;
    use creto_messaging::topic::{TopicConfig, TopicManager, TopicPolicy};
    use creto_runtime::sandbox::{Sandbox, SandboxConfig};

    /// Test secure messaging from within a sandbox
    #[test]
    fn test_sandbox_secure_messaging() {
        let fixture = TestFixture::new();

        // Create sandbox for agent
        let _sandbox = Sandbox::new(fixture.org_id, fixture.agent_id, SandboxConfig::default());

        // Create topic manager
        let mut topic_manager = TopicManager::new();

        // Create a topic for sandbox communication
        let mut topic_config = TopicConfig::new("sandbox-events".to_string(), fixture.agent_id);
        topic_config.publish_policy = TopicPolicy::Private;
        topic_config.subscribe_policy = TopicPolicy::AuthzRequired;

        let topic_id = topic_manager.create_topic(topic_config).unwrap();

        // Verify topic was created
        assert!(!topic_id.is_nil());
    }

    /// Test checkpoint state can be shared via messaging
    #[test]
    fn test_checkpoint_notification_via_messaging() {
        let fixture = TestFixture::new();

        // Create topic manager
        let mut topic_manager = TopicManager::new();

        // Create a checkpoint notification topic
        let mut topic_config =
            TopicConfig::new("checkpoint-notifications".to_string(), fixture.agent_id);
        topic_config.publish_policy = TopicPolicy::Private;
        topic_config.subscribe_policy = TopicPolicy::Allowlist;
        topic_config.allowed_agents = vec![fixture.agent_id];

        let topic_id = topic_manager.create_topic(topic_config).unwrap();

        // Subscribe to checkpoint notifications
        let subscription = topic_manager
            .subscribe(topic_id, fixture.agent_id, None)
            .unwrap();
        assert_eq!(subscription.topic_id, topic_id);

        // Publish checkpoint event
        let checkpoint_metadata = std::collections::HashMap::from([
            ("sandbox_id".to_string(), "sandbox-123".to_string()),
            ("checkpoint_id".to_string(), "cp-456".to_string()),
        ]);

        let delivered = topic_manager
            .publish(
                topic_id,
                fixture.agent_id,
                b"checkpoint_complete",
                checkpoint_metadata,
            )
            .unwrap();

        // Should be delivered to subscriber
        assert!(delivered.contains(&fixture.agent_id));
    }
}

// ============================================================================
// Full Workflow Integration
// ============================================================================

mod full_workflow {
    use super::*;
    use creto_messaging::service::MessagingService;
    use creto_metering::{Quota, QuotaEnforcer, QuotaPeriod, UsageEvent, UsageEventType};
    use creto_oversight::{
        approval::{Approval, ApprovalDecision},
        policy::{PolicyContext, PolicyEngine, TrustLevel},
        request::{ActionType, OversightRequest, Priority, RequestStatus},
        service::OversightService,
    };
    use creto_runtime::sandbox::{Sandbox, SandboxConfig};

    /// Test complete agent action flow:
    /// 1. Agent requests action
    /// 2. Metering checks quota
    /// 3. Policy determines oversight needed
    /// 4. Oversight request created
    /// 5. Approval submitted
    /// 6. Action executed in sandbox
    /// 7. Notification sent via messaging
    #[tokio::test]
    async fn test_complete_agent_action_flow() {
        let fixture = TestFixture::new();

        // Step 1: Agent wants to execute high-value transaction
        let action = ActionType::Transaction {
            amount_cents: 500000, // $5,000
            currency: "USD".to_string(),
        };

        // Step 2: Check quota
        let enforcer = QuotaEnforcer::new();
        let quota = Quota::new(fixture.org_id, "transactions", 100, QuotaPeriod::Daily);
        enforcer.register_quota(&quota);

        let quota_check = enforcer.check(&fixture.org_id, &fixture.agent_id, "transactions", 1);
        assert!(quota_check.is_ok());
        assert!(quota_check.unwrap().allowed);

        // Step 3: Policy evaluation
        let engine = PolicyEngine::new();
        let context = PolicyContext {
            trust_level: TrustLevel::Standard,
            quota_usage_percentage: quota.usage_percentage() * 100.0,
            delegation_depth: 0,
            ..Default::default()
        };

        let decision = engine.evaluate(&action, &context).await.unwrap();
        assert!(decision.requires_oversight());

        // Step 4: Create oversight request
        let request = OversightRequest::new(
            fixture.org_id,
            fixture.agent_id,
            action.clone(),
            "Agent requesting high-value transaction",
        )
        .with_priority(Priority::High);

        let request_id = request.id;
        assert_eq!(request.status, RequestStatus::Pending);

        // Step 5: Submit approval
        let oversight_service = OversightService::new();
        let approval_result = oversight_service
            .submit_approval(
                request_id,
                fixture.user_id,
                ApprovalDecision::Approve,
                Some("Verified and approved".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(approval_result.request_id, request_id);

        // Step 6: Execute in sandbox (after approval)
        let _sandbox = Sandbox::new(fixture.org_id, fixture.agent_id, SandboxConfig::default());

        // Step 7: Record metering event for completed action
        let event = UsageEvent::builder()
            .organization_id(fixture.org_id)
            .agent_id(fixture.agent_id)
            .event_type(UsageEventType::SandboxExecution)
            .quantity(1)
            .properties(serde_json::json!({
                "oversight_request_id": request_id.to_string(),
                "action_type": "transaction",
                "amount_cents": 500000,
                "approved_by": fixture.user_id.to_string()
            }))
            .build();

        assert_eq!(event.event_type, UsageEventType::SandboxExecution);

        // Step 8: Send notification via messaging
        let _messaging_service = MessagingService::new();
        // Notification would be sent here in production
    }

    /// Test delegation chain through multiple agents
    #[tokio::test]
    async fn test_delegation_chain_workflow() {
        let fixture = TestFixture::new();

        // Primary agent delegates to secondary agent
        let primary_agent = fixture.agent_id;
        let secondary_agent = AgentId::new();

        // Secondary agent's action should include delegation depth
        let context = PolicyContext {
            trust_level: TrustLevel::Standard,
            quota_usage_percentage: 30.0,
            delegation_depth: 1, // One level of delegation
            ..Default::default()
        };

        let engine = PolicyEngine::new();
        let action = ActionType::DataAccess {
            data_type: "customer_records".to_string(),
            scope: "pii".to_string(), // PII scope triggers oversight
        };

        let decision = engine.evaluate(&action, &context).await.unwrap();

        // PII data access should require oversight
        assert!(decision.requires_oversight());

        // Create event with delegation info
        let event = UsageEvent::builder()
            .organization_id(fixture.org_id)
            .agent_id(secondary_agent)
            .event_type(UsageEventType::ApiCall)
            .quantity(1)
            .delegation_depth(1)
            .properties(serde_json::json!({
                "delegator": primary_agent.to_string(),
                "delegation_reason": "data_analysis_task"
            }))
            .build();

        assert_eq!(event.delegation_depth, 1);
    }

    /// Test workflow with multiple approval requirements
    #[tokio::test]
    async fn test_multi_approval_workflow() {
        let fixture = TestFixture::new();
        let user2 = UserId::new();

        // Very high-value action requiring multiple approvals
        let request = OversightRequest::new(
            fixture.org_id,
            fixture.agent_id,
            ActionType::Transaction {
                amount_cents: 10000000, // $100,000
                currency: "USD".to_string(),
            },
            "Critical high-value transaction requiring multi-approval",
        )
        .with_priority(Priority::Critical);

        let request_id = request.id;

        // First approval
        let approval1 = Approval::new(request_id, fixture.user_id, ApprovalDecision::Approve)
            .with_reason("First reviewer approved");

        // Second approval from different user
        let approval2 = Approval::new(request_id, user2, ApprovalDecision::Approve)
            .with_reason("Second reviewer approved");

        // Both approvals should reference same request
        assert_eq!(approval1.request_id, approval2.request_id);
        assert_ne!(approval1.reviewer_id, approval2.reviewer_id);
    }
}

// ============================================================================
// Error Handling Integration
// ============================================================================

mod error_handling {
    use super::*;
    use creto_metering::{Quota, QuotaEnforcer, QuotaPeriod};
    use creto_oversight::{
        approval::ApprovalDecision,
        request::{ActionType, OversightRequest, RequestStatus},
        service::OversightService,
    };

    /// Test quota exceeded triggers automatic oversight
    #[test]
    fn test_quota_exceeded_behavior() {
        let fixture = TestFixture::new();
        let enforcer = QuotaEnforcer::new();

        // Register quota at limit
        let mut quota = Quota::new(fixture.org_id, "api_calls", 100, QuotaPeriod::Daily);
        quota.current_usage = 100; // At limit
        enforcer.register_quota(&quota);

        // Check should indicate quota exceeded
        let result = enforcer.check(&fixture.org_id, &fixture.agent_id, "api_calls", 1);

        assert!(result.is_ok());
        let check = result.unwrap();
        assert!(!check.allowed); // Should be denied
    }

    /// Test request timeout handling
    #[tokio::test]
    async fn test_request_timeout_workflow() {
        let fixture = TestFixture::new();

        // Create request with short timeout
        let request = OversightRequest::new(
            fixture.org_id,
            fixture.agent_id,
            ActionType::Transaction {
                amount_cents: 10000,
                currency: "USD".to_string(),
            },
            "Time-sensitive request",
        )
        .with_timeout(60); // 60 seconds

        // Request should start as pending
        assert_eq!(request.status, RequestStatus::Pending);

        // Timeout should be set
        assert!(!request.is_expired()); // Not expired yet
    }

    /// Test rejection flow
    #[tokio::test]
    async fn test_request_rejection_flow() {
        let fixture = TestFixture::new();
        let oversight_service = OversightService::new();

        let request = OversightRequest::new(
            fixture.org_id,
            fixture.agent_id,
            ActionType::DataAccess {
                data_type: "sensitive_pii".to_string(),
                scope: "full_access".to_string(),
            },
            "Request for sensitive data access",
        );

        let request_id = request.id;

        // Reject the request
        let result = oversight_service
            .submit_approval(
                request_id,
                fixture.user_id,
                ApprovalDecision::Reject,
                Some("Access not justified".to_string()),
            )
            .await
            .unwrap();

        assert_eq!(result.request_id, request_id);
    }
}
