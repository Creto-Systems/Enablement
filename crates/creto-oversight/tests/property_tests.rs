//! Property-based tests for oversight system.

use creto_common::{AgentId, OrganizationId, UserId};
use creto_oversight::{
    approval::{Approval, ApprovalDecision, QuorumCalculator, QuorumConfig},
    request::{ActionType, OversightRequest, Priority, RequestStatus},
};
use proptest::prelude::*;

// Property test: Any approval count >= required should result in approval
proptest! {
    #[test]
    fn prop_quorum_approval_threshold(
        required in 1u32..10,
        approve_count in 1u32..20,
    ) {
        let config = QuorumConfig::n_of_m(required);
        let calculator = QuorumCalculator::new(config);

        let approvals: Vec<Approval> = (0..approve_count)
            .map(|_| {
                let mut approval = Approval::new(
                    uuid::Uuid::new_v4(),
                    UserId::new(),
                    ApprovalDecision::Approve
                );
                approval.weight = 1;
                approval
            })
            .collect();

        let result = calculator.evaluate(&approvals);

        if approve_count >= required {
            prop_assert!(result.is_approved());
        } else {
            prop_assert!(result.is_pending());
        }
    }
}

proptest! {
    #[test]
    fn prop_weighted_quorum(
        required_weight in 10u32..100,
        individual_weights in prop::collection::vec(1u32..20, 1..10),
    ) {
        let config = QuorumConfig::weighted(required_weight);
        let calculator = QuorumCalculator::new(config);

        let approvals: Vec<Approval> = individual_weights
            .iter()
            .map(|&weight| {
                let mut approval = Approval::new(
                    uuid::Uuid::new_v4(),
                    UserId::new(),
                    ApprovalDecision::Approve
                );
                approval.weight = weight;
                approval
            })
            .collect();

        let total_weight: u32 = individual_weights.iter().sum();
        let result = calculator.evaluate(&approvals);

        if total_weight >= required_weight {
            prop_assert!(result.is_approved());
        } else {
            prop_assert!(result.is_pending());
        }
    }
}

proptest! {
    #[test]
    fn prop_any_rejection_rejects(
        approve_count in 1u32..10,
        has_rejection in prop::bool::ANY,
    ) {
        let mut config = QuorumConfig::n_of_m(5);
        config.any_rejection_rejects = true;
        let calculator = QuorumCalculator::new(config);

        let mut approvals: Vec<Approval> = (0..approve_count)
            .map(|_| {
                let mut approval = Approval::new(
                    uuid::Uuid::new_v4(),
                    UserId::new(),
                    ApprovalDecision::Approve
                );
                approval.weight = 1;
                approval
            })
            .collect();

        if has_rejection {
            let mut reject = Approval::new(
                uuid::Uuid::new_v4(),
                UserId::new(),
                ApprovalDecision::Reject
            );
            reject.weight = 1;
            approvals.push(reject);
        }

        let result = calculator.evaluate(&approvals);

        if has_rejection {
            prop_assert!(result.is_rejected());
        }
    }
}

proptest! {
    #[test]
    fn prop_request_timeout_calculation(
        timeout_seconds in 60u64..604800, // 1 minute to 7 days
    ) {
        let request = OversightRequest::new(
            OrganizationId::new(),
            AgentId::new(),
            ActionType::Custom { type_id: "test".to_string() },
            "Test request"
        ).with_timeout(timeout_seconds);

        let expected_duration = chrono::Duration::seconds(timeout_seconds as i64);
        let actual_duration = request.expires_at - request.created_at;

        // Allow 1 second tolerance for execution time
        prop_assert!((actual_duration.num_seconds() - expected_duration.num_seconds()).abs() <= 1);
    }
}

proptest! {
    #[test]
    fn prop_priority_timeout_consistency(
        priority_val in 0u8..4,
    ) {
        let priority = match priority_val {
            0 => Priority::Low,
            1 => Priority::Normal,
            2 => Priority::High,
            _ => Priority::Critical,
        };

        let timeout = priority.default_timeout_seconds();

        // Verify higher priority = shorter timeout
        match priority {
            Priority::Low => prop_assert_eq!(timeout, 604800),
            Priority::Normal => prop_assert_eq!(timeout, 86400),
            Priority::High => prop_assert_eq!(timeout, 3600),
            Priority::Critical => prop_assert_eq!(timeout, 300),
        }
    }
}

proptest! {
    #[test]
    fn prop_transaction_amount_positive(
        amount in 1i64..i64::MAX,
        currency in "[A-Z]{3}",
    ) {
        let action = ActionType::Transaction {
            amount_cents: amount,
            currency: currency.clone(),
        };

        // Verify we can create and serialize any positive transaction
        let serialized = serde_json::to_string(&action);
        prop_assert!(serialized.is_ok());

        let deserialized: Result<ActionType, _> =
            serde_json::from_str(&serialized.unwrap());
        prop_assert!(deserialized.is_ok());
    }
}

proptest! {
    #[test]
    fn prop_request_status_terminal_invariant(
        status_val in 0u8..7,
    ) {
        let status = match status_val {
            0 => RequestStatus::Pending,
            1 => RequestStatus::InReview,
            2 => RequestStatus::Approved,
            3 => RequestStatus::Rejected,
            4 => RequestStatus::Escalated,
            5 => RequestStatus::TimedOut,
            _ => RequestStatus::Cancelled,
        };

        let is_terminal = status.is_terminal();
        let is_pending_or_review = matches!(
            status,
            RequestStatus::Pending | RequestStatus::InReview | RequestStatus::Escalated
        );

        // Terminal states should never be pending/review
        prop_assert_eq!(is_terminal, !is_pending_or_review);
    }
}

proptest! {
    #[test]
    fn prop_approval_decision_serialization(
        decision_val in 0u8..5,
    ) {
        let decision = match decision_val {
            0 => ApprovalDecision::Approve,
            1 => ApprovalDecision::Reject,
            2 => ApprovalDecision::Abstain,
            3 => ApprovalDecision::RequestInfo,
            _ => ApprovalDecision::Escalate,
        };

        let serialized = serde_json::to_string(&decision).unwrap();
        let deserialized: ApprovalDecision =
            serde_json::from_str(&serialized).unwrap();

        prop_assert_eq!(decision, deserialized);
    }
}

proptest! {
    #[test]
    fn prop_quorum_monotonic_approvals(
        required in 2u32..10,
        step_count in 1usize..15,
    ) {
        let config = QuorumConfig::n_of_m(required);
        let calculator = QuorumCalculator::new(config);

        let mut approvals = Vec::new();
        let mut was_approved = false;

        for _ in 0..step_count {
            let mut approval = Approval::new(
                uuid::Uuid::new_v4(),
                UserId::new(),
                ApprovalDecision::Approve
            );
            approval.weight = 1;
            approvals.push(approval);

            let result = calculator.evaluate(&approvals);

            // Once approved, adding more approvals doesn't change it
            if was_approved {
                prop_assert!(result.is_approved());
            }

            if result.is_approved() {
                was_approved = true;
            }
        }
    }
}

proptest! {
    #[test]
    fn prop_reviewer_assignment_no_duplicates(
        reviewer_count in 1usize..20,
    ) {
        let mut request = OversightRequest::new(
            OrganizationId::new(),
            AgentId::new(),
            ActionType::Custom { type_id: "test".to_string() },
            "Test"
        );

        let reviewers: Vec<UserId> = (0..reviewer_count)
            .map(|_| UserId::new())
            .collect();

        // Add same reviewer multiple times
        for reviewer in &reviewers {
            request.add_reviewer(*reviewer);
            request.add_reviewer(*reviewer); // Duplicate
        }

        // Should only have unique reviewers
        prop_assert_eq!(request.assigned_reviewers.len(), reviewer_count);
    }
}
