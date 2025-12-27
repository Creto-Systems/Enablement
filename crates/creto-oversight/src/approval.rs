//! Approval decisions and quorum logic.

use chrono::{DateTime, Utc};
use creto_common::UserId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An approval decision by a reviewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    /// Unique approval ID.
    pub id: Uuid,

    /// Request this approval is for.
    pub request_id: Uuid,

    /// Reviewer who made the decision.
    pub reviewer_id: UserId,

    /// The decision.
    pub decision: ApprovalDecision,

    /// Optional reason or comment.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    /// When the decision was made.
    pub decided_at: DateTime<Utc>,

    /// Weight of this approval (for weighted quorum).
    #[serde(default = "default_weight")]
    pub weight: u32,
}

fn default_weight() -> u32 {
    1
}

impl Approval {
    /// Create a new approval decision.
    pub fn new(request_id: Uuid, reviewer_id: UserId, decision: ApprovalDecision) -> Self {
        Self {
            id: Uuid::now_v7(),
            request_id,
            reviewer_id,
            decision,
            reason: None,
            decided_at: Utc::now(),
            weight: 1,
        }
    }

    /// Set the reason for this decision.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Set the weight for this approval.
    pub fn with_weight(mut self, weight: u32) -> Self {
        self.weight = weight;
        self
    }
}

/// The decision made by a reviewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    /// Action approved.
    Approve,
    /// Action rejected.
    Reject,
    /// Abstain from decision.
    Abstain,
    /// Request more information.
    RequestInfo,
    /// Escalate to higher authority.
    Escalate,
}

/// Quorum configuration for approval decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuorumConfig {
    /// Minimum approvals required.
    pub required_approvals: u32,

    /// Minimum total weight required (for weighted quorum).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_weight: Option<u32>,

    /// Whether any rejection immediately rejects the request.
    #[serde(default)]
    pub any_rejection_rejects: bool,

    /// Whether quorum must be unanimous.
    #[serde(default)]
    pub require_unanimous: bool,
}

impl Default for QuorumConfig {
    fn default() -> Self {
        Self {
            required_approvals: 1,
            required_weight: None,
            any_rejection_rejects: false,
            require_unanimous: false,
        }
    }
}

impl QuorumConfig {
    /// Create a simple N-of-M quorum.
    pub fn n_of_m(required: u32) -> Self {
        Self {
            required_approvals: required,
            ..Default::default()
        }
    }

    /// Create a unanimous quorum.
    pub fn unanimous() -> Self {
        Self {
            required_approvals: 1,
            require_unanimous: true,
            any_rejection_rejects: true,
            ..Default::default()
        }
    }

    /// Create a weighted quorum.
    pub fn weighted(required_weight: u32) -> Self {
        Self {
            required_approvals: 1,
            required_weight: Some(required_weight),
            ..Default::default()
        }
    }
}

/// Calculator for quorum decisions.
pub struct QuorumCalculator {
    config: QuorumConfig,
}

impl QuorumCalculator {
    /// Create a new quorum calculator with the given configuration.
    pub fn new(config: QuorumConfig) -> Self {
        Self { config }
    }

    /// Evaluate the current approvals against the quorum.
    pub fn evaluate(&self, approvals: &[Approval]) -> QuorumResult {
        let approve_count = approvals
            .iter()
            .filter(|a| a.decision == ApprovalDecision::Approve)
            .count() as u32;

        let reject_count = approvals
            .iter()
            .filter(|a| a.decision == ApprovalDecision::Reject)
            .count() as u32;

        let approve_weight: u32 = approvals
            .iter()
            .filter(|a| a.decision == ApprovalDecision::Approve)
            .map(|a| a.weight)
            .sum();

        let reject_weight: u32 = approvals
            .iter()
            .filter(|a| a.decision == ApprovalDecision::Reject)
            .map(|a| a.weight)
            .sum();

        // Check for immediate rejection
        if self.config.any_rejection_rejects && reject_count > 0 {
            return QuorumResult::Rejected {
                approve_count,
                reject_count,
            };
        }

        // Check weighted quorum
        if let Some(required_weight) = self.config.required_weight {
            if reject_weight >= required_weight {
                return QuorumResult::Rejected {
                    approve_count,
                    reject_count,
                };
            }
            if approve_weight >= required_weight {
                return QuorumResult::Approved {
                    approve_count,
                    total_weight: approve_weight,
                };
            }
            return QuorumResult::Pending {
                approve_count,
                required: self.config.required_approvals,
                current_weight: approve_weight,
                required_weight: Some(required_weight),
            };
        }

        // Check count-based quorum
        if approve_count >= self.config.required_approvals {
            // For unanimous, check no rejections
            if self.config.require_unanimous && reject_count > 0 {
                return QuorumResult::Rejected {
                    approve_count,
                    reject_count,
                };
            }
            return QuorumResult::Approved {
                approve_count,
                total_weight: approve_weight,
            };
        }

        QuorumResult::Pending {
            approve_count,
            required: self.config.required_approvals,
            current_weight: approve_weight,
            required_weight: self.config.required_weight,
        }
    }
}

/// Result of a quorum evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuorumResult {
    /// Quorum reached, request approved.
    Approved {
        approve_count: u32,
        total_weight: u32,
    },
    /// Request rejected.
    Rejected {
        approve_count: u32,
        reject_count: u32,
    },
    /// Still waiting for more approvals.
    Pending {
        approve_count: u32,
        required: u32,
        current_weight: u32,
        required_weight: Option<u32>,
    },
}

impl QuorumResult {
    /// Check if the quorum resulted in approval.
    pub fn is_approved(&self) -> bool {
        matches!(self, QuorumResult::Approved { .. })
    }

    /// Check if the quorum resulted in rejection.
    pub fn is_rejected(&self) -> bool {
        matches!(self, QuorumResult::Rejected { .. })
    }

    /// Check if still pending.
    pub fn is_pending(&self) -> bool {
        matches!(self, QuorumResult::Pending { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_approval(decision: ApprovalDecision, weight: u32) -> Approval {
        let mut approval = Approval::new(Uuid::new_v4(), UserId::new(), decision);
        approval.weight = weight;
        approval
    }

    #[test]
    fn test_simple_quorum() {
        let calc = QuorumCalculator::new(QuorumConfig::n_of_m(2));

        // 1 approval: pending
        let result = calc.evaluate(&[make_approval(ApprovalDecision::Approve, 1)]);
        assert!(result.is_pending());

        // 2 approvals: approved
        let result = calc.evaluate(&[
            make_approval(ApprovalDecision::Approve, 1),
            make_approval(ApprovalDecision::Approve, 1),
        ]);
        assert!(result.is_approved());
    }

    #[test]
    fn test_any_rejection_rejects() {
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
    fn test_weighted_quorum() {
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
}
