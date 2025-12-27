//! Integration tests for creto-oversight.
//!
//! These tests verify the oversight service functionality including
//! approval workflows, state transitions, and quorum management.

use chrono::Utc;
use creto_integration_tests::common::TestFixture;
use creto_oversight::{
    ActionType, Approval, ApprovalDecision, OversightRequest, Priority, QuorumCalculator,
    QuorumConfig, QuorumResult, RequestStatus, StateMachine, StateTransition,
};
use uuid::Uuid;

#[test]
fn test_oversight_request_creation() {
    let fixture = TestFixture::new();
    let request = OversightRequest::new(
        fixture.org_id,
        fixture.agent_id,
        ActionType::Transaction {
            amount_cents: 100000,
            currency: "USD".to_string(),
        },
        "Transfer $1,000 to external account",
    );

    assert_eq!(request.status, RequestStatus::Pending);
    assert_eq!(request.priority, Priority::Normal);
    assert!(!request.is_expired());
}

#[test]
fn test_request_status_terminal() {
    assert!(!RequestStatus::Pending.is_terminal());
    assert!(!RequestStatus::InReview.is_terminal());
    assert!(RequestStatus::Approved.is_terminal());
    assert!(RequestStatus::Rejected.is_terminal());
    assert!(RequestStatus::TimedOut.is_terminal());
    assert!(RequestStatus::Cancelled.is_terminal());
}

#[test]
fn test_action_types() {
    // Test all action type variants
    let _ = ActionType::Transaction {
        amount_cents: 1000,
        currency: "USD".to_string(),
    };
    let _ = ActionType::DataAccess {
        data_type: "user_data".to_string(),
        scope: "read".to_string(),
    };
    let _ = ActionType::ExternalApi {
        service: "stripe".to_string(),
        operation: "charge".to_string(),
    };
    let _ = ActionType::CodeExecution {
        runtime: "python".to_string(),
        risk_level: "medium".to_string(),
    };
    let _ = ActionType::Communication {
        recipient_type: "customer".to_string(),
        category: "support".to_string(),
    };
    let _ = ActionType::Custom {
        type_id: "custom_action".to_string(),
    };
}

#[test]
fn test_priority_ordering() {
    assert!(Priority::Critical > Priority::High);
    assert!(Priority::High > Priority::Normal);
    assert!(Priority::Normal > Priority::Low);
}

#[test]
fn test_priority_default_timeout() {
    assert_eq!(Priority::Low.default_timeout_seconds(), 604800); // 7 days
    assert_eq!(Priority::Normal.default_timeout_seconds(), 86400); // 24 hours
    assert_eq!(Priority::High.default_timeout_seconds(), 3600); // 1 hour
    assert_eq!(Priority::Critical.default_timeout_seconds(), 300); // 5 minutes
}

#[test]
fn test_approval_creation() {
    let fixture = TestFixture::new();
    let request_id = Uuid::now_v7();

    let approval = Approval::new(request_id, fixture.user_id, ApprovalDecision::Approve);

    assert_eq!(approval.request_id, request_id);
    assert_eq!(approval.decision, ApprovalDecision::Approve);
    assert_eq!(approval.weight, 1);
}

#[test]
fn test_approval_with_reason() {
    let fixture = TestFixture::new();
    let request_id = Uuid::now_v7();

    let approval = Approval::new(request_id, fixture.user_id, ApprovalDecision::Reject)
        .with_reason("Amount exceeds policy limit");

    assert_eq!(
        approval.reason,
        Some("Amount exceeds policy limit".to_string())
    );
}

#[test]
fn test_approval_decisions() {
    // Verify all decision types exist
    let _ = ApprovalDecision::Approve;
    let _ = ApprovalDecision::Reject;
    let _ = ApprovalDecision::Abstain;
    let _ = ApprovalDecision::RequestInfo;
    let _ = ApprovalDecision::Escalate;
}

#[test]
fn test_quorum_config_simple() {
    let config = QuorumConfig::n_of_m(2);
    assert_eq!(config.required_approvals, 2);
    assert!(!config.any_rejection_rejects);
    assert!(!config.require_unanimous);
}

#[test]
fn test_quorum_config_unanimous() {
    let config = QuorumConfig::unanimous();
    assert!(config.require_unanimous);
    assert!(config.any_rejection_rejects);
}

#[test]
fn test_quorum_config_weighted() {
    let config = QuorumConfig::weighted(10);
    assert_eq!(config.required_weight, Some(10));
}

fn make_approval(decision: ApprovalDecision, weight: u32) -> Approval {
    let mut approval = Approval::new(Uuid::new_v4(), creto_common::UserId::new(), decision);
    approval.weight = weight;
    approval
}

#[test]
fn test_quorum_calculator_pending() {
    let calc = QuorumCalculator::new(QuorumConfig::n_of_m(2));

    // 1 approval: pending
    let result = calc.evaluate(&[make_approval(ApprovalDecision::Approve, 1)]);
    assert!(result.is_pending());
}

#[test]
fn test_quorum_calculator_approved() {
    let calc = QuorumCalculator::new(QuorumConfig::n_of_m(2));

    // 2 approvals: approved
    let result = calc.evaluate(&[
        make_approval(ApprovalDecision::Approve, 1),
        make_approval(ApprovalDecision::Approve, 1),
    ]);
    assert!(result.is_approved());
}

#[test]
fn test_quorum_any_rejection_rejects() {
    let mut config = QuorumConfig::n_of_m(2);
    config.any_rejection_rejects = true;
    let calc = QuorumCalculator::new(config);

    // Any rejection immediately rejects
    let result = calc.evaluate(&[
        make_approval(ApprovalDecision::Approve, 1),
        make_approval(ApprovalDecision::Reject, 1),
    ]);
    assert!(result.is_rejected());
}

#[test]
fn test_quorum_weighted() {
    let calc = QuorumCalculator::new(QuorumConfig::weighted(10));

    // Weight 5: pending
    let result = calc.evaluate(&[make_approval(ApprovalDecision::Approve, 5)]);
    assert!(result.is_pending());

    // Weight 10: approved
    let result = calc.evaluate(&[
        make_approval(ApprovalDecision::Approve, 5),
        make_approval(ApprovalDecision::Approve, 5),
    ]);
    assert!(result.is_approved());
}

#[test]
fn test_request_with_rich_context() {
    let fixture = TestFixture::new();
    let request = OversightRequest::new(
        fixture.org_id,
        fixture.agent_id,
        ActionType::Transaction {
            amount_cents: 100000,
            currency: "USD".to_string(),
        },
        "Transfer funds",
    )
    .with_context(serde_json::json!({
        "action": "send_payment",
        "amount": 1000.00,
        "recipient": "vendor@example.com"
    }))
    .with_priority(Priority::High);

    assert_eq!(request.context["action"], "send_payment");
    assert_eq!(request.context["amount"], 1000.00);
    assert_eq!(request.priority, Priority::High);
}

#[test]
fn test_request_with_timeout() {
    let fixture = TestFixture::new();
    let request = OversightRequest::new(
        fixture.org_id,
        fixture.agent_id,
        ActionType::Custom {
            type_id: "test".to_string(),
        },
        "Test request",
    )
    .with_timeout(3600); // 1 hour

    assert_eq!(request.timeout_seconds, 3600);
}

#[test]
fn test_request_add_reviewer() {
    let fixture = TestFixture::new();
    let mut request = OversightRequest::new(
        fixture.org_id,
        fixture.agent_id,
        ActionType::Custom {
            type_id: "test".to_string(),
        },
        "Test request",
    );

    let reviewer_id = creto_common::UserId::new();
    request.add_reviewer(reviewer_id);

    assert!(request.assigned_reviewers.contains(&reviewer_id));

    // Adding same reviewer again should not duplicate
    request.add_reviewer(reviewer_id);
    assert_eq!(request.assigned_reviewers.len(), 1);
}
