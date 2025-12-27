//! Policy evaluation for determining oversight requirements.

use serde::{Deserialize, Serialize};

use crate::request::ActionType;

/// Result of policy evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision {
    /// Action is allowed without oversight.
    Allow,
    /// Action requires human oversight.
    RequiresOversight {
        /// Reason oversight is required.
        reason: String,
        /// Suggested reviewers (role-based).
        suggested_reviewers: Vec<String>,
    },
    /// Action is denied by policy.
    Deny {
        /// Reason for denial.
        reason: String,
    },
}

impl PolicyDecision {
    /// Check if this decision allows the action.
    pub fn is_allowed(&self) -> bool {
        matches!(self, PolicyDecision::Allow)
    }

    /// Check if oversight is required.
    pub fn requires_oversight(&self) -> bool {
        matches!(self, PolicyDecision::RequiresOversight { .. })
    }

    /// Check if the action is denied.
    pub fn is_denied(&self) -> bool {
        matches!(self, PolicyDecision::Deny { .. })
    }
}

/// Engine for evaluating Cedar policies to determine oversight requirements.
///
/// Integrates with creto-authz for Cedar policy evaluation.
pub struct PolicyEngine {
    // TODO: Add Cedar client, policy cache
    _private: (),
}

impl PolicyEngine {
    /// Create a new policy engine.
    pub fn new() -> Self {
        Self { _private: () }
    }

    /// Evaluate whether an action requires oversight.
    ///
    /// Returns a policy decision based on Cedar policy evaluation.
    pub async fn evaluate(
        &self,
        action: &ActionType,
        context: &PolicyContext,
    ) -> Result<PolicyDecision, creto_common::CretoError> {
        // TODO: Implement actual Cedar policy evaluation
        // 1. Build Cedar request from action and context
        // 2. Evaluate against policies
        // 3. Return decision

        // Placeholder: Simple threshold-based logic
        match action {
            ActionType::Transaction { amount_cents, .. } => {
                if *amount_cents > 1000000 {
                    // > $10,000
                    Ok(PolicyDecision::RequiresOversight {
                        reason: "High-value transaction requires approval".to_string(),
                        suggested_reviewers: vec!["finance_manager".to_string()],
                    })
                } else if *amount_cents > 100000 {
                    // > $1,000
                    Ok(PolicyDecision::RequiresOversight {
                        reason: "Medium-value transaction requires approval".to_string(),
                        suggested_reviewers: vec!["team_lead".to_string()],
                    })
                } else {
                    Ok(PolicyDecision::Allow)
                }
            }
            ActionType::DataAccess { scope, .. } => {
                if scope == "pii" || scope == "financial" {
                    Ok(PolicyDecision::RequiresOversight {
                        reason: "Sensitive data access requires approval".to_string(),
                        suggested_reviewers: vec!["data_owner".to_string()],
                    })
                } else {
                    Ok(PolicyDecision::Allow)
                }
            }
            ActionType::CodeExecution { risk_level, .. } => {
                if risk_level == "high" {
                    Ok(PolicyDecision::RequiresOversight {
                        reason: "High-risk code execution requires approval".to_string(),
                        suggested_reviewers: vec!["security_team".to_string()],
                    })
                } else {
                    Ok(PolicyDecision::Allow)
                }
            }
            _ => {
                let _ = context;
                Ok(PolicyDecision::Allow)
            }
        }
    }
}

impl Default for PolicyEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Context provided for policy evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyContext {
    /// Agent's current quota usage percentage.
    #[serde(default)]
    pub quota_usage_percentage: f64,

    /// Agent's trust level.
    #[serde(default)]
    pub trust_level: TrustLevel,

    /// Delegation depth from original human.
    #[serde(default)]
    pub delegation_depth: u8,

    /// Time of day (for time-based policies).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_of_day: Option<String>,

    /// Additional attributes for policy evaluation.
    #[serde(default)]
    pub attributes: serde_json::Value,
}

impl Default for PolicyContext {
    fn default() -> Self {
        Self {
            quota_usage_percentage: 0.0,
            trust_level: TrustLevel::Standard,
            delegation_depth: 0,
            time_of_day: None,
            attributes: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

/// Trust level of an agent.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    /// New or untrusted agent.
    Low,
    /// Standard trust level.
    #[default]
    Standard,
    /// Elevated trust (proven track record).
    Elevated,
    /// Fully trusted (minimal oversight).
    Full,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_high_value_transaction_requires_oversight() {
        let engine = PolicyEngine::new();

        let action = ActionType::Transaction {
            amount_cents: 5000000, // $50,000
            currency: "USD".to_string(),
        };

        let decision = engine
            .evaluate(&action, &PolicyContext::default())
            .await
            .unwrap();

        assert!(decision.requires_oversight());
    }

    #[tokio::test]
    async fn test_low_value_transaction_allowed() {
        let engine = PolicyEngine::new();

        let action = ActionType::Transaction {
            amount_cents: 5000, // $50
            currency: "USD".to_string(),
        };

        let decision = engine
            .evaluate(&action, &PolicyContext::default())
            .await
            .unwrap();

        assert!(decision.is_allowed());
    }

    #[tokio::test]
    async fn test_pii_access_requires_oversight() {
        let engine = PolicyEngine::new();

        let action = ActionType::DataAccess {
            data_type: "customer_records".to_string(),
            scope: "pii".to_string(),
        };

        let decision = engine
            .evaluate(&action, &PolicyContext::default())
            .await
            .unwrap();

        assert!(decision.requires_oversight());
    }
}
