//! Policy trigger system for automatic oversight request creation.
//!
//! This module provides automatic oversight request creation based on
//! policy evaluation rules, integrating with Cedar policy engine.

use serde::{Deserialize, Serialize};

use crate::{
    policy::{PolicyContext, PolicyDecision},
    request::{ActionType, Priority},
};

/// Configuration for policy-based triggers.
///
/// Defines rules for when oversight requests should be automatically created
/// based on action characteristics and policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyTriggerConfig {
    /// Trigger conditions that activate oversight.
    pub conditions: Vec<TriggerCondition>,

    /// Priority to assign to auto-created requests.
    #[serde(default = "default_priority")]
    pub default_priority: Priority,

    /// Timeout in seconds for auto-created requests.
    #[serde(default = "default_timeout")]
    pub default_timeout_seconds: u64,

    /// Whether to automatically assign reviewers based on suggested roles.
    #[serde(default = "default_auto_assign")]
    pub auto_assign_reviewers: bool,

    /// Additional metadata to attach to triggered requests.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

fn default_priority() -> Priority {
    Priority::Normal
}

fn default_timeout() -> u64 {
    86400 // 24 hours
}

fn default_auto_assign() -> bool {
    true
}

impl PolicyTriggerConfig {
    /// Create a new trigger configuration with default settings.
    pub fn new() -> Self {
        Self {
            conditions: Vec::new(),
            default_priority: Priority::Normal,
            default_timeout_seconds: 86400,
            auto_assign_reviewers: true,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Add a trigger condition.
    pub fn with_condition(mut self, condition: TriggerCondition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Set the default priority for triggered requests.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.default_priority = priority;
        self
    }

    /// Set the default timeout for triggered requests.
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.default_timeout_seconds = seconds;
        self
    }

    /// Enable or disable auto-assignment of reviewers.
    pub fn with_auto_assign(mut self, enabled: bool) -> Self {
        self.auto_assign_reviewers = enabled;
        self
    }

    /// Add metadata to triggered requests.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

impl Default for PolicyTriggerConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Conditions that trigger automatic oversight creation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TriggerCondition {
    /// Trigger based on transaction amount threshold.
    AmountThreshold {
        /// Minimum amount in cents that triggers oversight.
        threshold_cents: i64,
        /// Currency code (optional, applies to all if None).
        currency: Option<String>,
    },

    /// Trigger based on action type.
    ActionType {
        /// Action type pattern to match.
        pattern: ActionTypePattern,
    },

    /// Trigger based on agent trust level.
    AgentTier {
        /// Minimum trust level required to bypass oversight.
        min_trust_level: TrustLevelThreshold,
    },

    /// Trigger based on time of day (after-hours actions).
    TimeWindow {
        /// Start hour (0-23, UTC).
        start_hour: u8,
        /// End hour (0-23, UTC).
        end_hour: u8,
        /// Days of week (0=Sunday, 6=Saturday).
        days_of_week: Vec<u8>,
    },

    /// Trigger based on quota usage.
    QuotaUsage {
        /// Percentage threshold (0.0-1.0).
        threshold_percentage: f64,
    },

    /// Trigger based on delegation depth.
    DelegationDepth {
        /// Maximum delegation depth before requiring oversight.
        max_depth: u8,
    },

    /// Trigger based on data scope/sensitivity.
    DataSensitivity {
        /// Sensitive data scopes that trigger oversight.
        scopes: Vec<String>,
    },

    /// Trigger based on risk level.
    RiskLevel {
        /// Risk levels that trigger oversight.
        levels: Vec<String>,
    },
}

/// Pattern matching for action types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionTypePattern {
    /// Match all transaction actions.
    Transaction,
    /// Match all data access actions.
    DataAccess,
    /// Match all external API actions.
    ExternalApi,
    /// Match all code execution actions.
    CodeExecution,
    /// Match all communication actions.
    Communication,
    /// Match specific custom type.
    Custom { type_id: String },
    /// Match any action.
    Any,
}

/// Trust level threshold for agent tier conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevelThreshold {
    /// Require oversight for low trust agents.
    Low,
    /// Require oversight for standard and below.
    Standard,
    /// Require oversight for elevated and below.
    Elevated,
    /// Require oversight for all agents.
    Full,
}

/// Evaluates policy triggers to determine if oversight is required.
pub struct PolicyEvaluator {
    /// Trigger configuration.
    config: PolicyTriggerConfig,
}

impl PolicyEvaluator {
    /// Create a new policy evaluator with the given configuration.
    pub fn new(config: PolicyTriggerConfig) -> Self {
        Self { config }
    }

    /// Check if any trigger conditions are met.
    ///
    /// Returns the matched trigger condition and suggested priority.
    pub fn evaluate(
        &self,
        action: &ActionType,
        context: &PolicyContext,
    ) -> Option<TriggerMatch> {
        for condition in &self.config.conditions {
            if self.check_condition(condition, action, context) {
                return Some(TriggerMatch {
                    condition: condition.clone(),
                    priority: self.determine_priority(condition, action, context),
                    timeout_seconds: self.config.default_timeout_seconds,
                    suggested_reviewers: self.suggest_reviewers(condition, action),
                });
            }
        }
        None
    }

    /// Check if a specific trigger condition is met.
    fn check_condition(
        &self,
        condition: &TriggerCondition,
        action: &ActionType,
        context: &PolicyContext,
    ) -> bool {
        match condition {
            TriggerCondition::AmountThreshold {
                threshold_cents,
                currency,
            } => self.check_amount_threshold(action, *threshold_cents, currency.as_deref()),

            TriggerCondition::ActionType { pattern } => self.check_action_type(action, pattern),

            TriggerCondition::AgentTier { min_trust_level } => {
                self.check_trust_level(context, *min_trust_level)
            }

            TriggerCondition::TimeWindow {
                start_hour,
                end_hour,
                days_of_week,
            } => self.check_time_window(*start_hour, *end_hour, days_of_week, context),

            TriggerCondition::QuotaUsage {
                threshold_percentage,
            } => context.quota_usage_percentage >= *threshold_percentage,

            TriggerCondition::DelegationDepth { max_depth } => {
                context.delegation_depth > *max_depth
            }

            TriggerCondition::DataSensitivity { scopes } => {
                self.check_data_sensitivity(action, scopes)
            }

            TriggerCondition::RiskLevel { levels } => self.check_risk_level(action, levels),
        }
    }

    /// Check if transaction amount exceeds threshold.
    fn check_amount_threshold(
        &self,
        action: &ActionType,
        threshold: i64,
        currency_filter: Option<&str>,
    ) -> bool {
        match action {
            ActionType::Transaction {
                amount_cents,
                currency,
            } => {
                let amount_check = *amount_cents > threshold;
                let currency_check = currency_filter
                    .map(|filter| currency == filter)
                    .unwrap_or(true);
                amount_check && currency_check
            }
            _ => false,
        }
    }

    /// Check if action matches the pattern.
    fn check_action_type(&self, action: &ActionType, pattern: &ActionTypePattern) -> bool {
        match pattern {
            ActionTypePattern::Transaction => matches!(action, ActionType::Transaction { .. }),
            ActionTypePattern::DataAccess => matches!(action, ActionType::DataAccess { .. }),
            ActionTypePattern::ExternalApi => matches!(action, ActionType::ExternalApi { .. }),
            ActionTypePattern::CodeExecution => matches!(action, ActionType::CodeExecution { .. }),
            ActionTypePattern::Communication => matches!(action, ActionType::Communication { .. }),
            ActionTypePattern::Custom { type_id } => match action {
                ActionType::Custom {
                    type_id: action_type_id,
                } => action_type_id == type_id,
                _ => false,
            },
            ActionTypePattern::Any => true,
        }
    }

    /// Check if agent trust level requires oversight.
    fn check_trust_level(&self, context: &PolicyContext, min_level: TrustLevelThreshold) -> bool {
        use crate::policy::TrustLevel;

        let agent_level = context.trust_level;
        let required = match min_level {
            TrustLevelThreshold::Low => TrustLevel::Low,
            TrustLevelThreshold::Standard => TrustLevel::Standard,
            TrustLevelThreshold::Elevated => TrustLevel::Elevated,
            TrustLevelThreshold::Full => TrustLevel::Full,
        };

        agent_level < required
    }

    /// Check if current time falls within restricted window.
    fn check_time_window(
        &self,
        start_hour: u8,
        end_hour: u8,
        days_of_week: &[u8],
        context: &PolicyContext,
    ) -> bool {
        // For now, parse from context.time_of_day if available
        // In production, this would use chrono::Utc::now()
        if let Some(time_str) = &context.time_of_day {
            if let Some(hour_str) = time_str.split(':').next() {
                if let Ok(hour) = hour_str.parse::<u8>() {
                    let in_hour_range = if start_hour <= end_hour {
                        hour >= start_hour && hour < end_hour
                    } else {
                        // Wraps midnight
                        hour >= start_hour || hour < end_hour
                    };

                    // For simplicity, assume day check passes if days_of_week is empty
                    let in_day_range = days_of_week.is_empty();

                    return in_hour_range && in_day_range;
                }
            }
        }
        false
    }

    /// Check if data access involves sensitive scopes.
    fn check_data_sensitivity(&self, action: &ActionType, scopes: &[String]) -> bool {
        match action {
            ActionType::DataAccess { scope, .. } => scopes.iter().any(|s| s == scope),
            _ => false,
        }
    }

    /// Check if action has high risk level.
    fn check_risk_level(&self, action: &ActionType, levels: &[String]) -> bool {
        match action {
            ActionType::CodeExecution { risk_level, .. } => levels.iter().any(|l| l == risk_level),
            _ => false,
        }
    }

    /// Determine priority based on trigger condition.
    fn determine_priority(
        &self,
        condition: &TriggerCondition,
        action: &ActionType,
        _context: &PolicyContext,
    ) -> Priority {
        match condition {
            TriggerCondition::AmountThreshold { threshold_cents, .. } => {
                // Higher amounts = higher priority
                if let ActionType::Transaction { amount_cents, .. } = action {
                    if *amount_cents > 10_000_000 {
                        // >$100k
                        Priority::Critical
                    } else if *amount_cents > 5_000_000 {
                        // >$50k
                        Priority::High
                    } else if *amount_cents > *threshold_cents * 2 {
                        Priority::High
                    } else {
                        Priority::Normal
                    }
                } else {
                    self.config.default_priority
                }
            }
            TriggerCondition::RiskLevel { levels } => {
                if levels.contains(&"critical".to_string()) {
                    Priority::Critical
                } else if levels.contains(&"high".to_string()) {
                    Priority::High
                } else {
                    Priority::Normal
                }
            }
            TriggerCondition::DataSensitivity { scopes } => {
                if scopes.contains(&"pii".to_string()) || scopes.contains(&"financial".to_string())
                {
                    Priority::High
                } else {
                    Priority::Normal
                }
            }
            _ => self.config.default_priority,
        }
    }

    /// Suggest reviewers based on trigger condition and action.
    fn suggest_reviewers(&self, condition: &TriggerCondition, action: &ActionType) -> Vec<String> {
        match condition {
            TriggerCondition::AmountThreshold { threshold_cents, .. } => {
                if let ActionType::Transaction { amount_cents, .. } = action {
                    if *amount_cents > 10_000_000 {
                        vec!["cfo".to_string(), "finance_director".to_string()]
                    } else if *amount_cents > *threshold_cents {
                        vec!["finance_manager".to_string()]
                    } else {
                        vec!["team_lead".to_string()]
                    }
                } else {
                    Vec::new()
                }
            }
            TriggerCondition::DataSensitivity { scopes } => {
                if scopes.contains(&"pii".to_string()) {
                    vec!["data_protection_officer".to_string()]
                } else if scopes.contains(&"financial".to_string()) {
                    vec!["finance_manager".to_string()]
                } else {
                    vec!["data_owner".to_string()]
                }
            }
            TriggerCondition::RiskLevel { .. } => {
                vec!["security_team".to_string(), "engineering_lead".to_string()]
            }
            _ => Vec::new(),
        }
    }
}

/// Result of trigger evaluation.
#[derive(Debug, Clone)]
pub struct TriggerMatch {
    /// The condition that was matched.
    pub condition: TriggerCondition,
    /// Suggested priority for the request.
    pub priority: Priority,
    /// Suggested timeout in seconds.
    pub timeout_seconds: u64,
    /// Suggested reviewers.
    pub suggested_reviewers: Vec<String>,
}

/// Mock Cedar policy client for testing trigger integration.
#[derive(Debug, Clone)]
pub struct MockCedarClient {
    /// Configured policy decision for testing.
    decision: PolicyDecision,
}

impl MockCedarClient {
    /// Create a new mock client that always returns the given decision.
    pub fn new(decision: PolicyDecision) -> Self {
        Self { decision }
    }

    /// Create a mock client that requires oversight.
    pub fn requires_oversight() -> Self {
        Self::new(PolicyDecision::RequiresOversight {
            reason: "Policy evaluation requires human oversight".to_string(),
            suggested_reviewers: vec!["policy_admin".to_string()],
        })
    }

    /// Create a mock client that allows actions.
    pub fn allows() -> Self {
        Self::new(PolicyDecision::Allow)
    }

    /// Create a mock client that denies actions.
    pub fn denies(reason: String) -> Self {
        Self::new(PolicyDecision::Deny { reason })
    }

    /// Evaluate authorization (mock Cedar isAuthorized call).
    pub async fn is_authorized(
        &self,
        _action: &ActionType,
        _context: &PolicyContext,
    ) -> PolicyDecision {
        self.decision.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::policy::TrustLevel;

    #[test]
    fn test_trigger_config_builder() {
        let config = PolicyTriggerConfig::new()
            .with_condition(TriggerCondition::AmountThreshold {
                threshold_cents: 1_000_000,
                currency: Some("USD".to_string()),
            })
            .with_priority(Priority::High)
            .with_timeout(3600)
            .with_auto_assign(true);

        assert_eq!(config.conditions.len(), 1);
        assert_eq!(config.default_priority, Priority::High);
        assert_eq!(config.default_timeout_seconds, 3600);
        assert!(config.auto_assign_reviewers);
    }

    #[test]
    fn test_amount_threshold_trigger() {
        let config = PolicyTriggerConfig::new().with_condition(TriggerCondition::AmountThreshold {
            threshold_cents: 1_000_000, // $10,000
            currency: None,
        });

        let evaluator = PolicyEvaluator::new(config);

        // High-value transaction should trigger
        let action = ActionType::Transaction {
            amount_cents: 5_000_000,
            currency: "USD".to_string(),
        };
        let context = PolicyContext::default();
        let result = evaluator.evaluate(&action, &context);

        assert!(result.is_some());
        let trigger = result.unwrap();
        assert!(matches!(
            trigger.condition,
            TriggerCondition::AmountThreshold { .. }
        ));
        assert!(trigger.priority >= Priority::Normal);
    }

    #[test]
    fn test_amount_threshold_currency_filter() {
        let config = PolicyTriggerConfig::new().with_condition(TriggerCondition::AmountThreshold {
            threshold_cents: 1_000_000,
            currency: Some("USD".to_string()),
        });

        let evaluator = PolicyEvaluator::new(config);

        // USD should trigger
        let action_usd = ActionType::Transaction {
            amount_cents: 5_000_000,
            currency: "USD".to_string(),
        };
        assert!(evaluator.evaluate(&action_usd, &PolicyContext::default()).is_some());

        // EUR should not trigger
        let action_eur = ActionType::Transaction {
            amount_cents: 5_000_000,
            currency: "EUR".to_string(),
        };
        assert!(evaluator.evaluate(&action_eur, &PolicyContext::default()).is_none());
    }

    #[test]
    fn test_low_value_no_trigger() {
        let config = PolicyTriggerConfig::new().with_condition(TriggerCondition::AmountThreshold {
            threshold_cents: 1_000_000,
            currency: None,
        });

        let evaluator = PolicyEvaluator::new(config);

        let action = ActionType::Transaction {
            amount_cents: 50_000, // $500
            currency: "USD".to_string(),
        };
        let context = PolicyContext::default();
        let result = evaluator.evaluate(&action, &context);

        assert!(result.is_none());
    }

    #[test]
    fn test_data_sensitivity_trigger() {
        let config = PolicyTriggerConfig::new().with_condition(TriggerCondition::DataSensitivity {
            scopes: vec!["pii".to_string(), "financial".to_string()],
        });

        let evaluator = PolicyEvaluator::new(config);

        let action = ActionType::DataAccess {
            data_type: "customer_records".to_string(),
            scope: "pii".to_string(),
        };
        let result = evaluator.evaluate(&action, &PolicyContext::default());

        assert!(result.is_some());
        let trigger = result.unwrap();
        assert_eq!(trigger.priority, Priority::High);
        assert!(trigger
            .suggested_reviewers
            .contains(&"data_protection_officer".to_string()));
    }

    #[test]
    fn test_risk_level_trigger() {
        let config = PolicyTriggerConfig::new().with_condition(TriggerCondition::RiskLevel {
            levels: vec!["high".to_string(), "critical".to_string()],
        });

        let evaluator = PolicyEvaluator::new(config);

        let action = ActionType::CodeExecution {
            runtime: "python".to_string(),
            risk_level: "high".to_string(),
        };
        let result = evaluator.evaluate(&action, &PolicyContext::default());

        assert!(result.is_some());
        let trigger = result.unwrap();
        assert!(trigger.suggested_reviewers.contains(&"security_team".to_string()));
    }

    #[test]
    fn test_trust_level_trigger() {
        let config = PolicyTriggerConfig::new().with_condition(TriggerCondition::AgentTier {
            min_trust_level: TrustLevelThreshold::Elevated,
        });

        let evaluator = PolicyEvaluator::new(config);

        // Low trust agent should trigger
        let mut context_low = PolicyContext::default();
        context_low.trust_level = TrustLevel::Low;
        let action = ActionType::Transaction {
            amount_cents: 100_000,
            currency: "USD".to_string(),
        };
        assert!(evaluator.evaluate(&action, &context_low).is_some());

        // Elevated trust agent should not trigger
        let mut context_elevated = PolicyContext::default();
        context_elevated.trust_level = TrustLevel::Elevated;
        assert!(evaluator.evaluate(&action, &context_elevated).is_none());
    }

    #[test]
    fn test_quota_usage_trigger() {
        let config = PolicyTriggerConfig::new().with_condition(TriggerCondition::QuotaUsage {
            threshold_percentage: 0.9,
        });

        let evaluator = PolicyEvaluator::new(config);

        // High quota usage should trigger
        let mut context_high = PolicyContext::default();
        context_high.quota_usage_percentage = 0.95;
        let action = ActionType::Transaction {
            amount_cents: 100_000,
            currency: "USD".to_string(),
        };
        assert!(evaluator.evaluate(&action, &context_high).is_some());

        // Low quota usage should not trigger
        let mut context_low = PolicyContext::default();
        context_low.quota_usage_percentage = 0.5;
        assert!(evaluator.evaluate(&action, &context_low).is_none());
    }

    #[test]
    fn test_delegation_depth_trigger() {
        let config = PolicyTriggerConfig::new().with_condition(TriggerCondition::DelegationDepth {
            max_depth: 2,
        });

        let evaluator = PolicyEvaluator::new(config);

        // Deep delegation should trigger
        let mut context_deep = PolicyContext::default();
        context_deep.delegation_depth = 3;
        let action = ActionType::Transaction {
            amount_cents: 100_000,
            currency: "USD".to_string(),
        };
        assert!(evaluator.evaluate(&action, &context_deep).is_some());

        // Shallow delegation should not trigger
        let mut context_shallow = PolicyContext::default();
        context_shallow.delegation_depth = 1;
        assert!(evaluator.evaluate(&action, &context_shallow).is_none());
    }

    #[test]
    fn test_time_window_trigger() {
        let config = PolicyTriggerConfig::new().with_condition(TriggerCondition::TimeWindow {
            start_hour: 22, // 10 PM
            end_hour: 6,    // 6 AM (after-hours)
            days_of_week: Vec::new(),
        });

        let evaluator = PolicyEvaluator::new(config);

        // During after-hours should trigger
        let mut context_night = PolicyContext::default();
        context_night.time_of_day = Some("23:30".to_string());
        let action = ActionType::Transaction {
            amount_cents: 100_000,
            currency: "USD".to_string(),
        };
        assert!(evaluator.evaluate(&action, &context_night).is_some());

        // During business hours should not trigger
        let mut context_day = PolicyContext::default();
        context_day.time_of_day = Some("14:00".to_string());
        assert!(evaluator.evaluate(&action, &context_day).is_none());
    }

    #[test]
    fn test_multiple_conditions() {
        let config = PolicyTriggerConfig::new()
            .with_condition(TriggerCondition::AmountThreshold {
                threshold_cents: 1_000_000,
                currency: None,
            })
            .with_condition(TriggerCondition::DataSensitivity {
                scopes: vec!["pii".to_string()],
            });

        let evaluator = PolicyEvaluator::new(config);

        // Should trigger on first condition
        let action_txn = ActionType::Transaction {
            amount_cents: 5_000_000,
            currency: "USD".to_string(),
        };
        assert!(evaluator.evaluate(&action_txn, &PolicyContext::default()).is_some());

        // Should trigger on second condition
        let action_data = ActionType::DataAccess {
            data_type: "customer".to_string(),
            scope: "pii".to_string(),
        };
        assert!(evaluator.evaluate(&action_data, &PolicyContext::default()).is_some());
    }

    #[tokio::test]
    async fn test_mock_cedar_client() {
        let client = MockCedarClient::requires_oversight();
        let action = ActionType::Transaction {
            amount_cents: 5_000_000,
            currency: "USD".to_string(),
        };
        let decision = client.is_authorized(&action, &PolicyContext::default()).await;

        assert!(decision.requires_oversight());
    }

    #[tokio::test]
    async fn test_mock_cedar_allows() {
        let client = MockCedarClient::allows();
        let action = ActionType::Transaction {
            amount_cents: 100,
            currency: "USD".to_string(),
        };
        let decision = client.is_authorized(&action, &PolicyContext::default()).await;

        assert!(decision.is_allowed());
    }

    #[tokio::test]
    async fn test_mock_cedar_denies() {
        let client = MockCedarClient::denies("Insufficient permissions".to_string());
        let action = ActionType::Transaction {
            amount_cents: 5_000_000,
            currency: "USD".to_string(),
        };
        let decision = client.is_authorized(&action, &PolicyContext::default()).await;

        assert!(decision.is_denied());
    }

    #[test]
    fn test_priority_escalation() {
        let config = PolicyTriggerConfig::new().with_condition(TriggerCondition::AmountThreshold {
            threshold_cents: 1_000_000,
            currency: None,
        });

        let evaluator = PolicyEvaluator::new(config);

        // Very high amount should get critical priority
        let action_critical = ActionType::Transaction {
            amount_cents: 50_000_000, // $500k
            currency: "USD".to_string(),
        };
        let result = evaluator.evaluate(&action_critical, &PolicyContext::default());
        assert!(result.is_some());
        assert_eq!(result.unwrap().priority, Priority::Critical);

        // High amount should get high priority
        let action_high = ActionType::Transaction {
            amount_cents: 10_000_000, // $100k
            currency: "USD".to_string(),
        };
        let result = evaluator.evaluate(&action_high, &PolicyContext::default());
        assert!(result.is_some());
        assert_eq!(result.unwrap().priority, Priority::High);
    }
}
