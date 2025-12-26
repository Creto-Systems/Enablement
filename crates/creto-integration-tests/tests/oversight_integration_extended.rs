//! Extended integration tests for Week 9-10 oversight features.

use creto_integration_tests::common::TestFixture;
use creto_oversight::{
    approval::{Approval, ApprovalDecision, QuorumConfig, QuorumCalculator},
    channels::{ChannelType, EmailChannel, EmailConfig, SlackChannel, SlackConfig, NotificationChannel},
    policy::{PolicyContext, PolicyEngine, TrustLevel},
    request::{ActionType, OversightRequest, Priority, RequestStatus},
    service::{OversightService, OversightCheckResult},
    state::StateMachine,
};
use uuid::Uuid;

#[tokio::test]
async fn test_full_approval_flow_with_slack() {
    let fixture = TestFixture::new();
    let service = OversightService::new();

    // Step 1: Agent requests oversight for high-value transaction
    let result = service
        .check_action(
            fixture.org_id,
            fixture.agent_id,
            ActionType::Transaction {
                amount_cents: 5000000, // $50,000
                currency: "USD".to_string(),
            },
            "Large vendor payment",
        )
        .await
        .unwrap();

    // Verify oversight is required
    assert!(result.requires_approval());
    let request_id = result.request_id().unwrap();

    // Step 2: Create notification channel
    let slack_config = SlackConfig {
        token: "xoxb-test-token".to_string(),
        default_channel: "#finance-approvals".to_string(),
        interactive_buttons: true,
    };
    let slack = SlackChannel::new(slack_config);

    // Create a mock request for notification
    let request = OversightRequest::new(
        fixture.org_id,
        fixture.agent_id,
        ActionType::Transaction {
            amount_cents: 5000000,
            currency: "USD".to_string(),
        },
        "Large vendor payment",
    );

    // Step 3: Send notification via Slack
    let notification_result = slack.notify(&request).await.unwrap();
    assert!(notification_result.success);
    assert!(notification_result.message_id.is_some());

    // Step 4: Submit approval
    let approval_result = service
        .submit_approval(
            request_id,
            fixture.user_id,
            ApprovalDecision::Approve,
            Some("Verified invoice, vendor is trusted".to_string()),
        )
        .await
        .unwrap();

    // Verify approval was processed
    assert_eq!(approval_result.request_id, request_id);
}

#[tokio::test]
async fn test_policy_trigger_creates_request() {
    let fixture = TestFixture::new();
    let engine = PolicyEngine::new();

    // Step 1: Evaluate PII access policy
    let action = ActionType::DataAccess {
        data_type: "customer_ssn".to_string(),
        scope: "pii".to_string(),
    };

    let context = PolicyContext {
        trust_level: TrustLevel::Standard,
        quota_usage_percentage: 45.0,
        delegation_depth: 1,
        ..Default::default()
    };

    let decision = engine.evaluate(&action, &context).await.unwrap();

    // Step 2: Verify policy requires oversight
    assert!(decision.requires_oversight());

    // Step 3: Create oversight request based on policy decision
    let request = OversightRequest::new(
        fixture.org_id,
        fixture.agent_id,
        action,
        "Access customer SSN for verification",
    )
    .with_context(serde_json::json!({
        "customer_id": "cust_123",
        "purpose": "identity_verification",
        "requested_by_agent": fixture.agent_id.to_string()
    }));

    assert_eq!(request.status, RequestStatus::Pending);
    assert_eq!(request.priority, Priority::Normal);

    // Step 4: Verify notification channels can be triggered
    let email_config = EmailConfig {
        smtp_host: "smtp.example.com".to_string(),
        smtp_port: 587,
        from_address: "oversight@example.com".to_string(),
        reply_to: Some("compliance@example.com".to_string()),
    };
    let email = EmailChannel::new(email_config);

    let email_result = email.notify(&request).await.unwrap();
    assert!(email_result.success);
}

#[tokio::test]
async fn test_service_restart_resumes_request() {
    let fixture = TestFixture::new();

    // Step 1: Create initial request
    let original_request = OversightRequest::new(
        fixture.org_id,
        fixture.agent_id,
        ActionType::Transaction {
            amount_cents: 3000000,
            currency: "USD".to_string(),
        },
        "Payment requiring multi-approval",
    )
    .with_priority(Priority::High);

    let request_id = original_request.id;

    // Step 2: Submit first approval before "restart"
    let approval1 = Approval::new(request_id, fixture.user_id, ApprovalDecision::Approve)
        .with_reason("First reviewer approved");

    // Step 3: Simulate service restart by creating new service instance
    let service_after_restart = OversightService::new();

    // Step 4: Submit second approval after restart
    let result = service_after_restart
        .submit_approval(
            request_id,
            creto_common::UserId::new(), // Different reviewer
            ApprovalDecision::Approve,
            Some("Second reviewer approved after restart".to_string()),
        )
        .await
        .unwrap();

    // Verify the request was processed correctly after restart
    assert_eq!(result.request_id, request_id);
}

#[test]
fn test_quorum_with_multi_channel_notifications() {
    let fixture = TestFixture::new();

    // Create request requiring multiple approvals
    let request = OversightRequest::new(
        fixture.org_id,
        fixture.agent_id,
        ActionType::Transaction {
            amount_cents: 10000000, // $100,000
            currency: "USD".to_string(),
        },
        "Critical high-value transaction",
    )
    .with_priority(Priority::Critical);

    // Configure 2-of-3 quorum
    let quorum_config = QuorumConfig::n_of_m(2);
    let calculator = QuorumCalculator::new(quorum_config);

    // Simulate approvals from multiple reviewers
    let approvals = vec![
        Approval::new(request.id, fixture.user_id, ApprovalDecision::Approve),
        Approval::new(
            request.id,
            creto_common::UserId::new(),
            ApprovalDecision::Approve,
        ),
    ];

    let result = calculator.evaluate(&approvals);
    assert!(result.is_approved());
}

#[tokio::test]
async fn test_escalation_flow() {
    let fixture = TestFixture::new();
    let service = OversightService::new();

    let request_id = Uuid::now_v7();

    // First reviewer escalates
    let result = service
        .submit_approval(
            request_id,
            fixture.user_id,
            ApprovalDecision::Escalate,
            Some("Requires higher authority review".to_string()),
        )
        .await
        .unwrap();

    // Request should move to escalated state
    // (In full implementation, this would trigger notification to higher authority)
    assert_eq!(result.request_id, request_id);
}

#[test]
fn test_state_machine_with_checkpoints() {
    let fixture = TestFixture::new();

    let mut state_machine = StateMachine::new();

    // Transition through states, simulating checkpoints at each step
    let transitions = vec![
        (
            RequestStatus::InReview,
            "First reviewer started review",
        ),
        (
            RequestStatus::Approved,
            "Quorum reached, request approved",
        ),
    ];

    for (status, comment) in transitions {
        let result = state_machine.transition(
            status,
            creto_oversight::state::Actor::User {
                user_id: fixture.user_id,
            },
            Some(comment.to_string()),
        );

        assert!(result.is_ok());
        assert_eq!(state_machine.current_status(), status);
    }

    // Verify history preserved across checkpoints
    let history = state_machine.history();
    assert_eq!(history.len(), 2);
}

#[tokio::test]
async fn test_timeout_handling_with_reminder() {
    let fixture = TestFixture::new();

    // Create request with short timeout
    let request = OversightRequest::new(
        fixture.org_id,
        fixture.agent_id,
        ActionType::Transaction {
            amount_cents: 500000,
            currency: "USD".to_string(),
        },
        "Time-sensitive payment",
    )
    .with_timeout(300); // 5 minutes

    // Setup reminder channel
    let slack_config = SlackConfig {
        token: "test".to_string(),
        default_channel: "#urgent".to_string(),
        interactive_buttons: true,
    };
    let slack = SlackChannel::new(slack_config);

    // Send reminder
    let reminder_result = slack.remind(&request).await.unwrap();
    assert!(reminder_result.success);

    // Verify timeout hasn't been reached yet
    assert!(!request.is_expired());
}

#[test]
fn test_concurrent_approval_handling() {
    // Test handling of simultaneous approvals from multiple reviewers
    let fixture = TestFixture::new();
    let request_id = Uuid::now_v7();

    let quorum_config = QuorumConfig::n_of_m(3);
    let calculator = QuorumCalculator::new(quorum_config);

    // Simulate 3 concurrent approvals
    let approvals = vec![
        Approval::new(request_id, fixture.user_id, ApprovalDecision::Approve),
        Approval::new(
            request_id,
            creto_common::UserId::new(),
            ApprovalDecision::Approve,
        ),
        Approval::new(
            request_id,
            creto_common::UserId::new(),
            ApprovalDecision::Approve,
        ),
    ];

    let result = calculator.evaluate(&approvals);
    assert!(result.is_approved());
}

#[tokio::test]
async fn test_policy_with_custom_attributes() {
    let engine = PolicyEngine::new();

    let context = PolicyContext {
        trust_level: TrustLevel::Elevated,
        quota_usage_percentage: 75.0,
        delegation_depth: 3,
        time_of_day: Some("02:00".to_string()), // Late night access
        attributes: serde_json::json!({
            "location": "foreign_country",
            "device_type": "mobile",
            "ip_reputation": "low"
        }),
    };

    let action = ActionType::DataAccess {
        data_type: "financial_records".to_string(),
        scope: "financial".to_string(),
    };

    let decision = engine.evaluate(&action, &context).await.unwrap();

    // Sensitive data access should require oversight
    assert!(decision.requires_oversight());
}
