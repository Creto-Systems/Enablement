//! Main oversight service facade.

use creto_common::{AgentId, CretoResult, OrganizationId, UserId};
use uuid::Uuid;

use crate::{
    approval::{Approval, ApprovalDecision, QuorumCalculator, QuorumConfig, QuorumResult},
    policy::{PolicyContext, PolicyDecision, PolicyEngine},
    request::{ActionType, OversightRequest, RequestStatus},
    state::{Actor, StateMachine},
};

/// Main entry point for the oversight system.
pub struct OversightService {
    /// Policy engine for determining oversight requirements.
    pub policy_engine: PolicyEngine,

    /// Default quorum configuration.
    pub default_quorum: QuorumConfig,
}

impl OversightService {
    /// Create a new oversight service.
    pub fn new() -> Self {
        Self {
            policy_engine: PolicyEngine::new(),
            default_quorum: QuorumConfig::default(),
        }
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
        let decision = self.policy_engine.evaluate(&action, &context).await?;

        match decision {
            PolicyDecision::Allow => Ok(OversightCheckResult::Allowed),

            PolicyDecision::RequiresOversight {
                reason,
                suggested_reviewers,
            } => {
                let mut request = OversightRequest::new(
                    organization_id,
                    agent_id,
                    action,
                    description,
                );

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
                    Actor::User { user_id: reviewer_id },
                    Some("Quorum reached".to_string()),
                )?;
                RequestStatus::Approved
            }
            QuorumResult::Rejected { .. } => {
                state_machine.transition(
                    RequestStatus::Rejected,
                    Actor::User { user_id: reviewer_id },
                    Some("Request rejected".to_string()),
                )?;
                RequestStatus::Rejected
            }
            QuorumResult::Pending { .. } => {
                state_machine.transition(
                    RequestStatus::InReview,
                    Actor::User { user_id: reviewer_id },
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
    RequiresApproval {
        request_id: Uuid,
        reason: String,
    },
    /// Action is denied by policy.
    Denied {
        reason: String,
    },
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
}
