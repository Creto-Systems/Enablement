//! Main oversight service facade.

use creto_common::{AgentId, CretoError, CretoResult, OrganizationId, UserId};
use uuid::Uuid;

use crate::{
    approval::{Approval, ApprovalDecision, QuorumCalculator, QuorumConfig, QuorumResult},
    checkpoint::{Checkpoint, CheckpointManager},
    policy::{PolicyContext, PolicyDecision, PolicyEngine},
    request::{ActionType, OversightRequest, RequestStatus},
    state::{Actor, StateMachine},
    triggers::{PolicyEvaluator, PolicyTriggerConfig},
};

/// Main entry point for the oversight system.
pub struct OversightService {
    /// Policy engine for determining oversight requirements.
    pub policy_engine: PolicyEngine,

    /// Default quorum configuration.
    pub default_quorum: QuorumConfig,

    /// Checkpoint manager for durability.
    pub checkpoint_manager: Option<CheckpointManager>,

    /// Policy trigger evaluator for automatic oversight creation.
    pub trigger_evaluator: Option<PolicyEvaluator>,
}

impl OversightService {
    /// Create a new oversight service.
    pub fn new() -> Self {
        Self {
            policy_engine: PolicyEngine::new(),
            default_quorum: QuorumConfig::default(),
            checkpoint_manager: None,
            trigger_evaluator: None,
        }
    }

    /// Create an oversight service with checkpoint support.
    pub fn with_checkpoints(checkpoint_manager: CheckpointManager) -> Self {
        Self {
            policy_engine: PolicyEngine::new(),
            default_quorum: QuorumConfig::default(),
            checkpoint_manager: Some(checkpoint_manager),
            trigger_evaluator: None,
        }
    }

    /// Configure policy triggers for automatic oversight creation.
    pub fn with_triggers(mut self, config: PolicyTriggerConfig) -> Self {
        self.trigger_evaluator = Some(PolicyEvaluator::new(config));
        self
    }

    /// Check if an action requires oversight and create a request if needed.
    ///
    /// This is the main entry point called by agents before executing actions.
    pub async fn check_action(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        action: ActionType,
        description: impl Into<String>,
    ) -> CretoResult<OversightCheckResult> {
        let context = PolicyContext::default();
        self.check_action_with_context(organization_id, agent_id, action, description, context)
            .await
    }

    /// Check if an action requires oversight with custom policy context.
    ///
    /// This variant allows callers to provide custom context for policy evaluation.
    pub async fn check_action_with_context(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        action: ActionType,
        description: impl Into<String>,
        context: PolicyContext,
    ) -> CretoResult<OversightCheckResult> {
        let decision = self.policy_engine.evaluate(&action, &context).await?;

        match decision {
            PolicyDecision::Allow => Ok(OversightCheckResult::Allowed),

            PolicyDecision::RequiresOversight {
                reason,
                suggested_reviewers,
            } => {
                let request = OversightRequest::new(organization_id, agent_id, action, description);

                // TODO: Look up actual user IDs from suggested_reviewers roles
                let _ = suggested_reviewers;

                // TODO: Persist request to database
                // TODO: Send notifications via channels

                Ok(OversightCheckResult::RequiresApproval {
                    request_id: request.id,
                    reason,
                })
            }

            PolicyDecision::Deny { reason } => Ok(OversightCheckResult::Denied { reason }),
        }
    }

    /// Check if policy triggers require creating an oversight request.
    ///
    /// This evaluates trigger conditions independently of Cedar policy evaluation.
    /// Returns Some(request_id) if a trigger was matched and a request was created.
    pub async fn check_policy_trigger(
        &self,
        organization_id: OrganizationId,
        agent_id: AgentId,
        action: ActionType,
        context: PolicyContext,
    ) -> CretoResult<Option<Uuid>> {
        // Early return if no trigger evaluator configured
        let evaluator = match &self.trigger_evaluator {
            Some(e) => e,
            None => return Ok(None),
        };

        // Check if any triggers match
        let trigger_match = match evaluator.evaluate(&action, &context) {
            Some(m) => m,
            None => return Ok(None),
        };

        // Create oversight request based on trigger
        let description = format!(
            "Automatic oversight: {}",
            self.describe_trigger(&trigger_match.condition)
        );

        let mut request = OversightRequest::new(organization_id, agent_id, action, description)
            .with_priority(trigger_match.priority)
            .with_timeout(trigger_match.timeout_seconds);

        // Add policy context to request metadata
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            "trigger_type".to_string(),
            serde_json::to_value(&trigger_match.condition).unwrap_or(serde_json::Value::Null),
        );
        metadata.insert(
            "policy_context".to_string(),
            serde_json::to_value(&context).unwrap_or(serde_json::Value::Null),
        );
        metadata.insert("auto_triggered".to_string(), serde_json::Value::Bool(true));
        request.metadata = serde_json::Value::Object(metadata);

        // TODO: Auto-assign reviewers based on suggested_reviewers
        // For now, store them in metadata
        if !trigger_match.suggested_reviewers.is_empty() {
            let reviewers_value = serde_json::to_value(&trigger_match.suggested_reviewers)
                .unwrap_or(serde_json::Value::Null);
            if let Some(obj) = request.metadata.as_object_mut() {
                obj.insert("suggested_reviewer_roles".to_string(), reviewers_value);
            }
        }

        // TODO: Persist request to database
        // TODO: Send notifications via channels

        Ok(Some(request.id))
    }

    /// Generate a human-readable description of a trigger condition.
    fn describe_trigger(&self, condition: &crate::triggers::TriggerCondition) -> String {
        use crate::triggers::TriggerCondition;

        match condition {
            TriggerCondition::AmountThreshold {
                threshold_cents,
                currency,
            } => {
                let amount = *threshold_cents as f64 / 100.0;
                let curr = currency.as_deref().unwrap_or("any currency");
                format!("Transaction exceeds ${:.2} {}", amount, curr)
            }
            TriggerCondition::ActionType { pattern } => {
                format!("Action type matches pattern: {:?}", pattern)
            }
            TriggerCondition::AgentTier { min_trust_level } => {
                format!("Agent trust level below {:?}", min_trust_level)
            }
            TriggerCondition::TimeWindow { .. } => {
                "Action attempted during restricted time window".to_string()
            }
            TriggerCondition::QuotaUsage {
                threshold_percentage,
            } => {
                format!("Quota usage exceeds {:.0}%", threshold_percentage * 100.0)
            }
            TriggerCondition::DelegationDepth { max_depth } => {
                format!("Delegation depth exceeds {}", max_depth)
            }
            TriggerCondition::DataSensitivity { scopes } => {
                format!("Access to sensitive data: {}", scopes.join(", "))
            }
            TriggerCondition::RiskLevel { levels } => {
                format!("Risk level matches: {}", levels.join(", "))
            }
        }
    }

    /// Submit an approval decision for a request.
    pub async fn submit_approval(
        &self,
        request_id: Uuid,
        reviewer_id: UserId,
        decision: ApprovalDecision,
        reason: Option<String>,
    ) -> CretoResult<ApprovalSubmitResult> {
        // TODO: Load request from database
        // For now, create a mock
        let mut state_machine = StateMachine::new();

        // Create approval
        let mut approval = Approval::new(request_id, reviewer_id, decision);
        if let Some(r) = reason {
            approval = approval.with_reason(r);
        }

        // TODO: Load all approvals for this request
        let approvals = vec![approval];

        // Calculate quorum
        let calculator = QuorumCalculator::new(self.default_quorum.clone());
        let quorum_result = calculator.evaluate(&approvals);

        // Update state based on quorum result
        let new_status = match &quorum_result {
            QuorumResult::Approved { .. } => {
                state_machine.transition(
                    RequestStatus::Approved,
                    Actor::User {
                        user_id: reviewer_id,
                    },
                    Some("Quorum reached".to_string()),
                )?;
                RequestStatus::Approved
            }
            QuorumResult::Rejected { .. } => {
                state_machine.transition(
                    RequestStatus::Rejected,
                    Actor::User {
                        user_id: reviewer_id,
                    },
                    Some("Request rejected".to_string()),
                )?;
                RequestStatus::Rejected
            }
            QuorumResult::Pending { .. } => {
                state_machine.transition(
                    RequestStatus::InReview,
                    Actor::User {
                        user_id: reviewer_id,
                    },
                    None,
                )?;
                RequestStatus::InReview
            }
        };

        // TODO: Persist approval and updated request state
        // TODO: Notify relevant parties

        Ok(ApprovalSubmitResult {
            request_id,
            new_status,
            quorum_result,
        })
    }

    /// Get the status of an oversight request.
    pub async fn get_request_status(
        &self,
        request_id: Uuid,
    ) -> CretoResult<Option<OversightRequest>> {
        // TODO: Load from database
        let _ = request_id;
        Ok(None)
    }

    /// List pending requests for a reviewer.
    pub async fn list_pending_for_reviewer(
        &self,
        reviewer_id: UserId,
    ) -> CretoResult<Vec<OversightRequest>> {
        // TODO: Query database for requests assigned to this reviewer
        let _ = reviewer_id;
        Ok(Vec::new())
    }

    /// Save a checkpoint for a request.
    ///
    /// This creates an atomic snapshot of the request's current state,
    /// including conversation context and state machine history.
    pub async fn save_checkpoint(
        &self,
        request: &OversightRequest,
        state_machine: &StateMachine,
        context: serde_json::Value,
        reason: Option<String>,
    ) -> CretoResult<Uuid> {
        let manager = self.checkpoint_manager.as_ref().ok_or_else(|| {
            CretoError::Configuration("Checkpoint manager not configured".to_string())
        })?;

        let mut checkpoint = Checkpoint::new(
            request.id,
            state_machine.current(), // Use state machine's current state
            state_machine,
            context,
        );

        if let Some(r) = reason {
            checkpoint = checkpoint.with_reason(r);
        }

        manager.save_checkpoint(&checkpoint).await
    }

    /// Load the latest checkpoint for a request.
    pub async fn load_checkpoint(&self, request_id: Uuid) -> CretoResult<Option<Checkpoint>> {
        let manager = self.checkpoint_manager.as_ref().ok_or_else(|| {
            CretoError::Configuration("Checkpoint manager not configured".to_string())
        })?;

        manager.load_checkpoint(request_id).await
    }

    /// Recover all pending requests after a system restart.
    ///
    /// This scans for incomplete oversight requests and restores their state
    /// from the most recent checkpoint, allowing the system to resume where
    /// it left off.
    pub async fn recover_pending_requests(&self) -> CretoResult<Vec<RecoveredRequest>> {
        // TODO: Implement full recovery logic with repository
        // This is a placeholder showing the intended flow:

        // 1. Query for all requests in non-terminal states
        // 2. For each request, load the latest checkpoint
        // 3. Restore state machine from checkpoint
        // 4. Validate that the checkpoint is compatible
        // 5. Resume processing from the restored state

        let manager = self.checkpoint_manager.as_ref().ok_or_else(|| {
            CretoError::Configuration("Checkpoint manager not configured".to_string())
        })?;

        let _ = manager; // Suppress warning until full implementation

        Ok(Vec::new())
    }
}

/// Result of recovering a request from a checkpoint.
#[derive(Debug, Clone)]
pub struct RecoveredRequest {
    /// The request that was recovered.
    pub request_id: Uuid,

    /// Restored state machine.
    pub state_machine: StateMachine,

    /// Restored context.
    pub context: serde_json::Value,

    /// Checkpoint that was used for recovery.
    pub checkpoint_id: Uuid,

    /// When the checkpoint was created.
    pub checkpoint_timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for OversightService {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of checking whether an action requires oversight.
#[derive(Debug, Clone)]
pub enum OversightCheckResult {
    /// Action is allowed without oversight.
    Allowed,
    /// Action requires human approval.
    RequiresApproval { request_id: Uuid, reason: String },
    /// Action is denied by policy.
    Denied { reason: String },
}

impl OversightCheckResult {
    /// Check if the action is allowed.
    pub fn is_allowed(&self) -> bool {
        matches!(self, OversightCheckResult::Allowed)
    }

    /// Check if approval is required.
    pub fn requires_approval(&self) -> bool {
        matches!(self, OversightCheckResult::RequiresApproval { .. })
    }

    /// Get the request ID if approval is required.
    pub fn request_id(&self) -> Option<Uuid> {
        match self {
            OversightCheckResult::RequiresApproval { request_id, .. } => Some(*request_id),
            _ => None,
        }
    }
}

/// Result of submitting an approval.
#[derive(Debug, Clone)]
pub struct ApprovalSubmitResult {
    /// Request ID.
    pub request_id: Uuid,
    /// New status after the approval.
    pub new_status: RequestStatus,
    /// Current quorum state.
    pub quorum_result: QuorumResult,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::triggers::TriggerCondition;

    #[tokio::test]
    async fn test_check_action_allowed() {
        let service = OversightService::new();

        let result = service
            .check_action(
                OrganizationId::new(),
                AgentId::new(),
                ActionType::Transaction {
                    amount_cents: 1000, // $10
                    currency: "USD".to_string(),
                },
                "Small purchase",
            )
            .await
            .unwrap();

        assert!(result.is_allowed());
    }

    #[tokio::test]
    async fn test_check_action_requires_approval() {
        let service = OversightService::new();

        let result = service
            .check_action(
                OrganizationId::new(),
                AgentId::new(),
                ActionType::Transaction {
                    amount_cents: 5000000, // $50,000
                    currency: "USD".to_string(),
                },
                "Large transfer",
            )
            .await
            .unwrap();

        assert!(result.requires_approval());
    }

    #[tokio::test]
    async fn test_policy_trigger_creates_request() {
        // Configure service with policy trigger
        let trigger_config =
            PolicyTriggerConfig::new().with_condition(TriggerCondition::AmountThreshold {
                threshold_cents: 1_000_000, // $10,000
                currency: None,
            });

        let service = OversightService::new().with_triggers(trigger_config);

        // High-value transaction should trigger oversight
        let action = ActionType::Transaction {
            amount_cents: 5_000_000, // $50,000
            currency: "USD".to_string(),
        };

        let context = PolicyContext::default();
        let request_id = service
            .check_policy_trigger(OrganizationId::new(), AgentId::new(), action, context)
            .await
            .unwrap();

        assert!(request_id.is_some());
    }

    #[tokio::test]
    async fn test_policy_trigger_no_match() {
        // Configure service with policy trigger
        let trigger_config =
            PolicyTriggerConfig::new().with_condition(TriggerCondition::AmountThreshold {
                threshold_cents: 10_000_000, // $100,000
                currency: None,
            });

        let service = OversightService::new().with_triggers(trigger_config);

        // Low-value transaction should not trigger
        let action = ActionType::Transaction {
            amount_cents: 50_000, // $500
            currency: "USD".to_string(),
        };

        let context = PolicyContext::default();
        let request_id = service
            .check_policy_trigger(OrganizationId::new(), AgentId::new(), action, context)
            .await
            .unwrap();

        assert!(request_id.is_none());
    }

    #[tokio::test]
    async fn test_policy_trigger_without_config() {
        // Service without trigger configuration
        let service = OversightService::new();

        let action = ActionType::Transaction {
            amount_cents: 50_000_000,
            currency: "USD".to_string(),
        };

        let context = PolicyContext::default();
        let request_id = service
            .check_policy_trigger(OrganizationId::new(), AgentId::new(), action, context)
            .await
            .unwrap();

        // Should return None because no trigger evaluator is configured
        assert!(request_id.is_none());
    }

    #[tokio::test]
    async fn test_trigger_with_multiple_conditions() {
        let trigger_config = PolicyTriggerConfig::new()
            .with_condition(TriggerCondition::AmountThreshold {
                threshold_cents: 1_000_000,
                currency: None,
            })
            .with_condition(TriggerCondition::DataSensitivity {
                scopes: vec!["pii".to_string()],
            });

        let service = OversightService::new().with_triggers(trigger_config);

        // Test first condition (amount threshold)
        let action_txn = ActionType::Transaction {
            amount_cents: 5_000_000,
            currency: "USD".to_string(),
        };
        let result1 = service
            .check_policy_trigger(
                OrganizationId::new(),
                AgentId::new(),
                action_txn,
                PolicyContext::default(),
            )
            .await
            .unwrap();
        assert!(result1.is_some());

        // Test second condition (data sensitivity)
        let action_data = ActionType::DataAccess {
            data_type: "customer".to_string(),
            scope: "pii".to_string(),
        };
        let result2 = service
            .check_policy_trigger(
                OrganizationId::new(),
                AgentId::new(),
                action_data,
                PolicyContext::default(),
            )
            .await
            .unwrap();
        assert!(result2.is_some());
    }

    #[test]
    fn test_describe_trigger() {
        let service = OversightService::new();

        let condition = TriggerCondition::AmountThreshold {
            threshold_cents: 1_000_000,
            currency: Some("USD".to_string()),
        };
        let description = service.describe_trigger(&condition);
        assert!(description.contains("$10000.00"));
        assert!(description.contains("USD"));
    }
}
