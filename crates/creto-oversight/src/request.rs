//! Oversight request types.

use chrono::{DateTime, Utc};
use creto_common::{AgentId, OrganizationId, UserId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A request for human oversight of an agent action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversightRequest {
    /// Unique request ID.
    pub id: Uuid,

    /// Organization this request belongs to.
    pub organization_id: OrganizationId,

    /// Agent that initiated the request.
    pub agent_id: AgentId,

    /// Type of action requiring oversight.
    pub action_type: ActionType,

    /// Human-readable description of what the agent wants to do.
    pub description: String,

    /// Structured context for the reviewer.
    pub context: serde_json::Value,

    /// Current status of the request.
    pub status: RequestStatus,

    /// Priority level.
    pub priority: Priority,

    /// When the request was created.
    pub created_at: DateTime<Utc>,

    /// When the request was last updated.
    pub updated_at: DateTime<Utc>,

    /// Timeout duration in seconds.
    pub timeout_seconds: u64,

    /// When the request will expire if not handled.
    pub expires_at: DateTime<Utc>,

    /// Assigned reviewers.
    #[serde(default)]
    pub assigned_reviewers: Vec<UserId>,

    /// Metadata for routing and filtering.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl OversightRequest {
    /// Create a new oversight request.
    pub fn new(
        organization_id: OrganizationId,
        agent_id: AgentId,
        action_type: ActionType,
        description: impl Into<String>,
    ) -> Self {
        let now = Utc::now();
        let timeout_seconds = 86400; // 24 hours default

        Self {
            id: Uuid::now_v7(),
            organization_id,
            agent_id,
            action_type,
            description: description.into(),
            context: serde_json::Value::Null,
            status: RequestStatus::Pending,
            priority: Priority::Normal,
            created_at: now,
            updated_at: now,
            timeout_seconds,
            expires_at: now + chrono::Duration::seconds(timeout_seconds as i64),
            assigned_reviewers: Vec::new(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Set the context for this request.
    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = context;
        self
    }

    /// Set the priority.
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the timeout in seconds.
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self.expires_at = self.created_at + chrono::Duration::seconds(seconds as i64);
        self
    }

    /// Add a reviewer.
    pub fn add_reviewer(&mut self, reviewer: UserId) {
        if !self.assigned_reviewers.contains(&reviewer) {
            self.assigned_reviewers.push(reviewer);
        }
    }

    /// Check if the request has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the request is still pending.
    pub fn is_pending(&self) -> bool {
        matches!(self.status, RequestStatus::Pending | RequestStatus::InReview)
    }
}

/// Status of an oversight request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    /// Awaiting review.
    Pending,
    /// Currently being reviewed.
    InReview,
    /// Approved by reviewer(s).
    Approved,
    /// Rejected by reviewer(s).
    Rejected,
    /// Escalated to higher authority.
    Escalated,
    /// Timed out without decision.
    TimedOut,
    /// Cancelled by agent or system.
    Cancelled,
}

impl RequestStatus {
    /// Check if this is a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            RequestStatus::Approved
                | RequestStatus::Rejected
                | RequestStatus::TimedOut
                | RequestStatus::Cancelled
        )
    }
}

/// Type of action requiring oversight.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    /// Financial transaction.
    Transaction {
        /// Amount in cents.
        amount_cents: i64,
        /// Currency code.
        currency: String,
    },
    /// Data access request.
    DataAccess {
        /// Type of data being accessed.
        data_type: String,
        /// Scope of access.
        scope: String,
    },
    /// External API call.
    ExternalApi {
        /// Target service.
        service: String,
        /// Operation being performed.
        operation: String,
    },
    /// Code execution.
    CodeExecution {
        /// Language/runtime.
        runtime: String,
        /// Risk level.
        risk_level: String,
    },
    /// Communication with human.
    Communication {
        /// Recipient type.
        recipient_type: String,
        /// Message category.
        category: String,
    },
    /// Generic action.
    Custom {
        /// Custom action type identifier.
        type_id: String,
    },
}

/// Priority level for oversight requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// Low priority, can wait.
    Low,
    /// Normal priority.
    Normal,
    /// High priority, review soon.
    High,
    /// Critical, needs immediate attention.
    Critical,
}

impl Priority {
    /// Get the default timeout for this priority level.
    pub fn default_timeout_seconds(&self) -> u64 {
        match self {
            Priority::Low => 604800,     // 7 days
            Priority::Normal => 86400,   // 24 hours
            Priority::High => 3600,      // 1 hour
            Priority::Critical => 300,   // 5 minutes
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let request = OversightRequest::new(
            OrganizationId::new(),
            AgentId::new(),
            ActionType::Transaction {
                amount_cents: 100000,
                currency: "USD".to_string(),
            },
            "Transfer $1,000 to external account",
        );

        assert_eq!(request.status, RequestStatus::Pending);
        assert!(!request.is_expired());
    }

    #[test]
    fn test_request_status_terminal() {
        assert!(!RequestStatus::Pending.is_terminal());
        assert!(!RequestStatus::InReview.is_terminal());
        assert!(RequestStatus::Approved.is_terminal());
        assert!(RequestStatus::Rejected.is_terminal());
        assert!(RequestStatus::TimedOut.is_terminal());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Critical > Priority::High);
        assert!(Priority::High > Priority::Normal);
        assert!(Priority::Normal > Priority::Low);
    }
}
