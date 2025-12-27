//! Example demonstrating policy trigger integration.
//!
//! This example shows how to configure automatic oversight request creation
//! based on policy triggers, independent of Cedar policy evaluation.

use creto_common::{AgentId, OrganizationId};
use creto_oversight::{
    ActionType, OversightService, PolicyContext, PolicyTriggerConfig, TriggerCondition,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Policy Trigger Integration Example ===\n");

    // Example 1: Configure a service with amount threshold trigger
    println!("1. Creating service with $10,000 threshold trigger...");
    let trigger_config =
        PolicyTriggerConfig::new().with_condition(TriggerCondition::AmountThreshold {
            threshold_cents: 1_000_000, // $10,000
            currency: None,             // Apply to all currencies
        });

    let service = OversightService::new().with_triggers(trigger_config);

    // Test high-value transaction
    println!("2. Testing high-value transaction ($50,000)...");
    let high_value_action = ActionType::Transaction {
        amount_cents: 5_000_000,
        currency: "USD".to_string(),
    };

    let context = PolicyContext::default();
    let request_id = service
        .check_policy_trigger(
            OrganizationId::new(),
            AgentId::new(),
            high_value_action,
            context,
        )
        .await?;

    match request_id {
        Some(id) => println!("   ✓ Oversight request created: {}", id),
        None => println!("   ✗ No oversight request created"),
    }

    // Test low-value transaction
    println!("\n3. Testing low-value transaction ($500)...");
    let low_value_action = ActionType::Transaction {
        amount_cents: 50_000,
        currency: "USD".to_string(),
    };

    let request_id = service
        .check_policy_trigger(
            OrganizationId::new(),
            AgentId::new(),
            low_value_action,
            PolicyContext::default(),
        )
        .await?;

    match request_id {
        Some(id) => println!("   ✗ Unexpected oversight request: {}", id),
        None => println!("   ✓ No oversight required (below threshold)"),
    }

    // Example 2: Multiple trigger conditions
    println!("\n4. Creating service with multiple triggers...");
    let multi_trigger_config = PolicyTriggerConfig::new()
        .with_condition(TriggerCondition::AmountThreshold {
            threshold_cents: 1_000_000,
            currency: None,
        })
        .with_condition(TriggerCondition::DataSensitivity {
            scopes: vec!["pii".to_string(), "financial".to_string()],
        })
        .with_condition(TriggerCondition::RiskLevel {
            levels: vec!["high".to_string(), "critical".to_string()],
        });

    let multi_service = OversightService::new().with_triggers(multi_trigger_config);

    // Test PII data access
    println!("5. Testing PII data access...");
    let pii_action = ActionType::DataAccess {
        data_type: "customer_records".to_string(),
        scope: "pii".to_string(),
    };

    let request_id = multi_service
        .check_policy_trigger(
            OrganizationId::new(),
            AgentId::new(),
            pii_action,
            PolicyContext::default(),
        )
        .await?;

    match request_id {
        Some(id) => println!("   ✓ PII access triggered oversight: {}", id),
        None => println!("   ✗ No oversight request created"),
    }

    // Test high-risk code execution
    println!("\n6. Testing high-risk code execution...");
    let risky_action = ActionType::CodeExecution {
        runtime: "python".to_string(),
        risk_level: "high".to_string(),
    };

    let request_id = multi_service
        .check_policy_trigger(
            OrganizationId::new(),
            AgentId::new(),
            risky_action,
            PolicyContext::default(),
        )
        .await?;

    match request_id {
        Some(id) => println!("   ✓ High-risk execution triggered oversight: {}", id),
        None => println!("   ✗ No oversight request created"),
    }

    // Example 3: Context-based triggers
    println!("\n7. Testing quota usage trigger...");
    let quota_trigger = PolicyTriggerConfig::new().with_condition(TriggerCondition::QuotaUsage {
        threshold_percentage: 0.9, // 90% quota usage
    });

    let quota_service = OversightService::new().with_triggers(quota_trigger);

    let mut high_quota_context = PolicyContext::default();
    high_quota_context.quota_usage_percentage = 0.95; // 95% used

    let normal_action = ActionType::Transaction {
        amount_cents: 10000,
        currency: "USD".to_string(),
    };

    let request_id = quota_service
        .check_policy_trigger(
            OrganizationId::new(),
            AgentId::new(),
            normal_action.clone(),
            high_quota_context,
        )
        .await?;

    match request_id {
        Some(id) => println!("   ✓ High quota usage triggered oversight: {}", id),
        None => println!("   ✗ No oversight request created"),
    }

    // Example 4: Time window trigger (after-hours)
    println!("\n8. Testing after-hours time window trigger...");
    let time_trigger = PolicyTriggerConfig::new().with_condition(TriggerCondition::TimeWindow {
        start_hour: 22, // 10 PM
        end_hour: 6,    // 6 AM
        days_of_week: Vec::new(),
    });

    let time_service = OversightService::new().with_triggers(time_trigger);

    let mut after_hours_context = PolicyContext::default();
    after_hours_context.time_of_day = Some("23:30".to_string());

    let request_id = time_service
        .check_policy_trigger(
            OrganizationId::new(),
            AgentId::new(),
            normal_action,
            after_hours_context,
        )
        .await?;

    match request_id {
        Some(id) => println!("   ✓ After-hours action triggered oversight: {}", id),
        None => println!("   ✗ No oversight request created"),
    }

    println!("\n=== All examples completed ===");
    Ok(())
}
