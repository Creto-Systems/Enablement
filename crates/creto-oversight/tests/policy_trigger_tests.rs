//! Comprehensive policy trigger tests for Week 9 features.

use creto_oversight::{
    policy::{PolicyContext, PolicyDecision, PolicyEngine, TrustLevel},
    request::ActionType,
};

#[tokio::test]
async fn test_amount_threshold_triggers_oversight() {
    let engine = PolicyEngine::new();
    let context = PolicyContext::default();

    // Test high-value transaction (> $10,000)
    let action = ActionType::Transaction {
        amount_cents: 2000000, // $20,000
        currency: "USD".to_string(),
    };

    let decision = engine.evaluate(&action, &context).await.unwrap();

    assert!(decision.requires_oversight());
    if let PolicyDecision::RequiresOversight { reason, suggested_reviewers } = decision {
        assert!(reason.contains("High-value"));
        assert!(suggested_reviewers.contains(&"finance_manager".to_string()));
    } else {
        panic!("Expected RequiresOversight decision");
    }
}

#[tokio::test]
async fn test_below_threshold_no_trigger() {
    let engine = PolicyEngine::new();
    let context = PolicyContext::default();

    // Test small transaction (< $1,000)
    let action = ActionType::Transaction {
        amount_cents: 50000, // $500
        currency: "USD".to_string(),
    };

    let decision = engine.evaluate(&action, &context).await.unwrap();

    assert!(decision.is_allowed());
    assert!(!decision.requires_oversight());
    assert!(!decision.is_denied());
}

#[tokio::test]
async fn test_action_type_trigger() {
    let engine = PolicyEngine::new();
    let context = PolicyContext::default();

    // Test PII data access (always requires oversight)
    let pii_action = ActionType::DataAccess {
        data_type: "customer_records".to_string(),
        scope: "pii".to_string(),
    };

    let decision = engine.evaluate(&pii_action, &context).await.unwrap();
    assert!(decision.requires_oversight());

    // Test financial data access
    let financial_action = ActionType::DataAccess {
        data_type: "transactions".to_string(),
        scope: "financial".to_string(),
    };

    let decision = engine.evaluate(&financial_action, &context).await.unwrap();
    assert!(decision.requires_oversight());

    // Test non-sensitive data access
    let public_action = ActionType::DataAccess {
        data_type: "blog_posts".to_string(),
        scope: "public".to_string(),
    };

    let decision = engine.evaluate(&public_action, &context).await.unwrap();
    assert!(decision.is_allowed());
}

#[tokio::test]
async fn test_combined_conditions() {
    let engine = PolicyEngine::new();

    // Test medium-value transaction with different trust levels
    let action = ActionType::Transaction {
        amount_cents: 150000, // $1,500 (medium threshold)
        currency: "USD".to_string(),
    };

    // Low trust - should require oversight
    let low_trust_context = PolicyContext {
        trust_level: TrustLevel::Low,
        ..Default::default()
    };

    let decision = engine.evaluate(&action, &low_trust_context).await.unwrap();
    assert!(decision.requires_oversight());

    // Full trust - currently still requires oversight at medium threshold
    let high_trust_context = PolicyContext {
        trust_level: TrustLevel::Full,
        ..Default::default()
    };

    let decision = engine.evaluate(&action, &high_trust_context).await.unwrap();
    // Medium amount still requires oversight regardless of trust
    assert!(decision.requires_oversight());
}

#[tokio::test]
async fn test_policy_evaluator_integration() {
    let engine = PolicyEngine::new();

    // Test high-risk code execution
    let high_risk_code = ActionType::CodeExecution {
        runtime: "python".to_string(),
        risk_level: "high".to_string(),
    };

    let decision = engine.evaluate(&high_risk_code, &PolicyContext::default()).await.unwrap();
    assert!(decision.requires_oversight());
    if let PolicyDecision::RequiresOversight { suggested_reviewers, .. } = decision {
        assert!(suggested_reviewers.contains(&"security_team".to_string()));
    }

    // Test low-risk code execution
    let low_risk_code = ActionType::CodeExecution {
        runtime: "python".to_string(),
        risk_level: "low".to_string(),
    };

    let decision = engine.evaluate(&low_risk_code, &PolicyContext::default()).await.unwrap();
    assert!(decision.is_allowed());
}

#[tokio::test]
async fn test_policy_decision_methods() {
    // Test PolicyDecision helper methods
    let allow = PolicyDecision::Allow;
    assert!(allow.is_allowed());
    assert!(!allow.requires_oversight());
    assert!(!allow.is_denied());

    let oversight = PolicyDecision::RequiresOversight {
        reason: "Test".to_string(),
        suggested_reviewers: vec![],
    };
    assert!(!oversight.is_allowed());
    assert!(oversight.requires_oversight());
    assert!(!oversight.is_denied());

    let deny = PolicyDecision::Deny {
        reason: "Blocked".to_string(),
    };
    assert!(!deny.is_allowed());
    assert!(!deny.requires_oversight());
    assert!(deny.is_denied());
}

#[tokio::test]
async fn test_context_attributes() {
    let engine = PolicyEngine::new();

    // Test with quota usage
    let context = PolicyContext {
        quota_usage_percentage: 95.0,
        trust_level: TrustLevel::Standard,
        delegation_depth: 2,
        time_of_day: Some("23:30".to_string()),
        attributes: serde_json::json!({
            "department": "engineering",
            "location": "us-west"
        }),
    };

    let action = ActionType::Transaction {
        amount_cents: 150000,
        currency: "USD".to_string(),
    };

    let decision = engine.evaluate(&action, &context).await.unwrap();
    assert!(decision.requires_oversight());
}

#[tokio::test]
async fn test_multiple_action_types() {
    let engine = PolicyEngine::new();
    let context = PolicyContext::default();

    // Test all action type variants for coverage
    let actions = vec![
        ActionType::Transaction {
            amount_cents: 5000,
            currency: "EUR".to_string(),
        },
        ActionType::DataAccess {
            data_type: "logs".to_string(),
            scope: "read".to_string(),
        },
        ActionType::ExternalApi {
            service: "stripe".to_string(),
            operation: "create_charge".to_string(),
        },
        ActionType::CodeExecution {
            runtime: "nodejs".to_string(),
            risk_level: "medium".to_string(),
        },
        ActionType::Communication {
            recipient_type: "customer".to_string(),
            category: "support".to_string(),
        },
        ActionType::Custom {
            type_id: "custom_workflow".to_string(),
        },
    ];

    for action in actions {
        let decision = engine.evaluate(&action, &context).await;
        assert!(decision.is_ok(), "Failed to evaluate action: {:?}", action);
    }
}

#[tokio::test]
async fn test_trust_level_serialization() {
    // Verify trust levels can be serialized/deserialized
    let levels = vec![
        TrustLevel::Low,
        TrustLevel::Standard,
        TrustLevel::Elevated,
        TrustLevel::Full,
    ];

    for level in levels {
        let json = serde_json::to_string(&level).unwrap();
        let deserialized: TrustLevel = serde_json::from_str(&json).unwrap();
        assert_eq!(level, deserialized);
    }
}
