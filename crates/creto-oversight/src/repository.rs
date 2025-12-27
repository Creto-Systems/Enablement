//! Repository traits and PostgreSQL implementations for oversight.
//!
//! This module provides the persistence layer for oversight requests,
//! approvals, state transitions, and notification management.
//!
//! Uses runtime SQL queries to avoid requiring DATABASE_URL at compile time.

use chrono::{DateTime, Utc};
use creto_common::{AgentId, CretoError, OrganizationId, UserId};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::approval::{Approval, ApprovalDecision};
use crate::request::{ActionType, OversightRequest, Priority, RequestStatus};

// ─────────────────────────────────────────────────────────────────────────────
// Enum Serialization Helpers
// ─────────────────────────────────────────────────────────────────────────────

impl RequestStatus {
    /// Convert to database string.
    pub fn as_str(&self) -> &'static str {
        match self {
            RequestStatus::Pending => "pending",
            RequestStatus::InReview => "in_review",
            RequestStatus::Approved => "approved",
            RequestStatus::Rejected => "rejected",
            RequestStatus::Escalated => "escalated",
            RequestStatus::TimedOut => "timed_out",
            RequestStatus::Cancelled => "cancelled",
        }
    }

    /// Parse from database string.
    pub fn parse_db_str(s: &str) -> Self {
        match s {
            "pending" => RequestStatus::Pending,
            "in_review" => RequestStatus::InReview,
            "approved" => RequestStatus::Approved,
            "rejected" => RequestStatus::Rejected,
            "escalated" => RequestStatus::Escalated,
            "timed_out" => RequestStatus::TimedOut,
            "cancelled" => RequestStatus::Cancelled,
            _ => RequestStatus::Pending,
        }
    }
}

impl Priority {
    /// Convert to database string.
    pub fn as_str(&self) -> &'static str {
        match self {
            Priority::Low => "low",
            Priority::Normal => "medium", // DB uses "medium" for Normal
            Priority::High => "high",
            Priority::Critical => "critical",
        }
    }

    /// Parse from database string.
    pub fn parse_db_str(s: &str) -> Self {
        match s {
            "low" => Priority::Low,
            "medium" => Priority::Normal,
            "high" => Priority::High,
            "critical" => Priority::Critical,
            _ => Priority::Normal,
        }
    }
}

impl ApprovalDecision {
    /// Convert to database string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ApprovalDecision::Approve => "approve",
            ApprovalDecision::Reject => "reject",
            ApprovalDecision::Abstain => "abstain",
            ApprovalDecision::RequestInfo => "request_info",
            ApprovalDecision::Escalate => "escalate",
        }
    }

    /// Parse from database string.
    pub fn parse_db_str(s: &str) -> Self {
        match s {
            "approve" => ApprovalDecision::Approve,
            "reject" => ApprovalDecision::Reject,
            "abstain" => ApprovalDecision::Abstain,
            "request_info" => ApprovalDecision::RequestInfo,
            "escalate" => ApprovalDecision::Escalate,
            _ => ApprovalDecision::Abstain,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Request Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Repository for oversight request persistence.
#[async_trait::async_trait]
pub trait RequestRepository: Send + Sync {
    /// Create a new oversight request.
    async fn create(&self, request: &OversightRequest) -> Result<Uuid, CretoError>;

    /// Get a request by ID.
    async fn get(&self, id: Uuid) -> Result<Option<OversightRequest>, CretoError>;

    /// Update request status.
    async fn update_status(&self, id: Uuid, status: RequestStatus) -> Result<(), CretoError>;

    /// List pending requests for an organization.
    async fn list_pending(
        &self,
        org_id: OrganizationId,
    ) -> Result<Vec<OversightRequest>, CretoError>;

    /// List requests by agent.
    async fn list_by_agent(
        &self,
        agent_id: AgentId,
        limit: i64,
    ) -> Result<Vec<OversightRequest>, CretoError>;

    /// Find timed-out requests.
    async fn find_timed_out(&self) -> Result<Vec<Uuid>, CretoError>;
}

/// PostgreSQL implementation of RequestRepository.
pub struct PgRequestRepository {
    pool: PgPool,
}

impl PgRequestRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RequestRepository for PgRequestRepository {
    async fn create(&self, request: &OversightRequest) -> Result<Uuid, CretoError> {
        // Serialize action_type as JSON for flexibility
        let action_type_json = serde_json::to_value(&request.action_type)
            .map_err(|e| CretoError::SerializationError(e.to_string()))?;

        let row = sqlx::query(
            r#"
            INSERT INTO oversight_requests (
                organization_id, agent_id, action_type, action_data,
                description, status, priority, context, timeout_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id
            "#,
        )
        .bind(request.organization_id.as_uuid())
        .bind(request.agent_id.as_uuid())
        .bind(action_type_to_str(&request.action_type))
        .bind(&action_type_json)
        .bind(&request.description)
        .bind(request.status.as_str())
        .bind(request.priority.as_str())
        .bind(&request.context)
        .bind(request.expires_at)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn get(&self, id: Uuid) -> Result<Option<OversightRequest>, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT organization_id, agent_id, action_type, action_data,
                   description, status, priority, context, timeout_at, created_at, updated_at
            FROM oversight_requests
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        match row {
            Some(r) => {
                let action_data: serde_json::Value = r.get("action_data");
                let action_type: ActionType =
                    serde_json::from_value(action_data).unwrap_or(ActionType::Custom {
                        type_id: "unknown".to_string(),
                    });

                Ok(Some(OversightRequest {
                    id,
                    organization_id: OrganizationId::from_uuid(r.get::<Uuid, _>("organization_id")),
                    agent_id: AgentId::from_uuid(r.get::<Uuid, _>("agent_id")),
                    action_type,
                    description: r.get("description"),
                    context: r.get("context"),
                    status: RequestStatus::parse_db_str(r.get::<&str, _>("status")),
                    priority: Priority::parse_db_str(r.get::<&str, _>("priority")),
                    created_at: r.get("created_at"),
                    updated_at: r.get("updated_at"),
                    timeout_seconds: 86400, // Default
                    expires_at: r.get("timeout_at"),
                    assigned_reviewers: vec![],
                    metadata: serde_json::Value::Object(serde_json::Map::new()),
                }))
            }
            None => Ok(None),
        }
    }

    async fn update_status(&self, id: Uuid, status: RequestStatus) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            UPDATE oversight_requests
            SET status = $2, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(status.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_pending(
        &self,
        org_id: OrganizationId,
    ) -> Result<Vec<OversightRequest>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id, agent_id, action_type, action_data, description,
                   status, priority, context, timeout_at, created_at, updated_at
            FROM oversight_requests
            WHERE organization_id = $1 AND status IN ('pending', 'in_review')
            ORDER BY
                CASE priority
                    WHEN 'critical' THEN 1
                    WHEN 'high' THEN 2
                    WHEN 'medium' THEN 3
                    WHEN 'low' THEN 4
                END,
                created_at ASC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        let mut requests = Vec::with_capacity(rows.len());
        for r in rows {
            let action_data: serde_json::Value = r.get("action_data");
            let action_type: ActionType =
                serde_json::from_value(action_data).unwrap_or(ActionType::Custom {
                    type_id: "unknown".to_string(),
                });

            requests.push(OversightRequest {
                id: r.get("id"),
                organization_id: org_id,
                agent_id: AgentId::from_uuid(r.get::<Uuid, _>("agent_id")),
                action_type,
                description: r.get("description"),
                context: r.get("context"),
                status: RequestStatus::parse_db_str(r.get::<&str, _>("status")),
                priority: Priority::parse_db_str(r.get::<&str, _>("priority")),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                timeout_seconds: 86400,
                expires_at: r.get("timeout_at"),
                assigned_reviewers: vec![],
                metadata: serde_json::Value::Object(serde_json::Map::new()),
            });
        }

        Ok(requests)
    }

    async fn list_by_agent(
        &self,
        agent_id: AgentId,
        limit: i64,
    ) -> Result<Vec<OversightRequest>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id, organization_id, action_type, action_data, description,
                   status, priority, context, timeout_at, created_at, updated_at
            FROM oversight_requests
            WHERE agent_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(agent_id.as_uuid())
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        let mut requests = Vec::with_capacity(rows.len());
        for r in rows {
            let action_data: serde_json::Value = r.get("action_data");
            let action_type: ActionType =
                serde_json::from_value(action_data).unwrap_or(ActionType::Custom {
                    type_id: "unknown".to_string(),
                });

            requests.push(OversightRequest {
                id: r.get("id"),
                organization_id: OrganizationId::from_uuid(r.get::<Uuid, _>("organization_id")),
                agent_id,
                action_type,
                description: r.get("description"),
                context: r.get("context"),
                status: RequestStatus::parse_db_str(r.get::<&str, _>("status")),
                priority: Priority::parse_db_str(r.get::<&str, _>("priority")),
                created_at: r.get("created_at"),
                updated_at: r.get("updated_at"),
                timeout_seconds: 86400,
                expires_at: r.get("timeout_at"),
                assigned_reviewers: vec![],
                metadata: serde_json::Value::Object(serde_json::Map::new()),
            });
        }

        Ok(requests)
    }

    async fn find_timed_out(&self) -> Result<Vec<Uuid>, CretoError> {
        let now = Utc::now();

        let rows = sqlx::query(
            r#"
            SELECT id
            FROM oversight_requests
            WHERE status = 'pending' AND timeout_at < $1
            "#,
        )
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.get("id")).collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Approval Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Repository for approval persistence.
#[async_trait::async_trait]
pub trait ApprovalRepository: Send + Sync {
    /// Record an approval decision.
    async fn create(&self, approval: &Approval) -> Result<Uuid, CretoError>;

    /// Get approvals for a request.
    async fn list_by_request(&self, request_id: Uuid) -> Result<Vec<Approval>, CretoError>;

    /// Count approvals by decision for a request.
    async fn count_by_decision(&self, request_id: Uuid) -> Result<ApprovalCounts, CretoError>;
}

/// Counts of approvals by decision type.
#[derive(Debug, Clone, Default)]
pub struct ApprovalCounts {
    pub approve: i64,
    pub reject: i64,
    pub abstain: i64,
    pub request_info: i64,
    pub escalate: i64,
    pub total_weight: i64,
}

/// PostgreSQL implementation of ApprovalRepository.
pub struct PgApprovalRepository {
    pool: PgPool,
}

impl PgApprovalRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ApprovalRepository for PgApprovalRepository {
    async fn create(&self, approval: &Approval) -> Result<Uuid, CretoError> {
        let row = sqlx::query(
            r#"
            INSERT INTO approvals (request_id, reviewer_id, decision, reason, weight)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (request_id, reviewer_id)
            DO UPDATE SET decision = $3, reason = $4, decided_at = NOW()
            RETURNING id
            "#,
        )
        .bind(approval.request_id)
        .bind(approval.reviewer_id.as_uuid())
        .bind(approval.decision.as_str())
        .bind(&approval.reason)
        .bind(approval.weight as i32)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn list_by_request(&self, request_id: Uuid) -> Result<Vec<Approval>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id, reviewer_id, decision, reason, weight, decided_at
            FROM approvals
            WHERE request_id = $1
            ORDER BY decided_at ASC
            "#,
        )
        .bind(request_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Approval {
                id: r.get("id"),
                request_id,
                reviewer_id: UserId::from_uuid(r.get::<Uuid, _>("reviewer_id")),
                decision: ApprovalDecision::parse_db_str(r.get::<&str, _>("decision")),
                reason: r.get("reason"),
                decided_at: r.get("decided_at"),
                weight: r.get::<i32, _>("weight") as u32,
            })
            .collect())
    }

    async fn count_by_decision(&self, request_id: Uuid) -> Result<ApprovalCounts, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE decision = 'approve') as approve_count,
                COUNT(*) FILTER (WHERE decision = 'reject') as reject_count,
                COUNT(*) FILTER (WHERE decision = 'abstain') as abstain_count,
                COUNT(*) FILTER (WHERE decision = 'request_info') as request_info_count,
                COUNT(*) FILTER (WHERE decision = 'escalate') as escalate_count,
                COALESCE(SUM(weight) FILTER (WHERE decision = 'approve'), 0) as total_weight
            FROM approvals
            WHERE request_id = $1
            "#,
        )
        .bind(request_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(ApprovalCounts {
            approve: row.get("approve_count"),
            reject: row.get("reject_count"),
            abstain: row.get("abstain_count"),
            request_info: row.get("request_info_count"),
            escalate: row.get("escalate_count"),
            total_weight: row.get("total_weight"),
        })
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// State Transition Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Record of a state transition in the audit trail.
#[derive(Debug, Clone)]
pub struct StateTransitionRecord {
    pub id: Uuid,
    pub request_id: Uuid,
    pub from_status: RequestStatus,
    pub to_status: RequestStatus,
    pub actor_type: String,
    pub actor_id: Option<Uuid>,
    pub reason: Option<String>,
    pub transitioned_at: DateTime<Utc>,
}

/// Repository for state transition audit trail.
#[async_trait::async_trait]
pub trait StateTransitionRepository: Send + Sync {
    /// Record a state transition.
    async fn create(&self, record: &StateTransitionRecord) -> Result<Uuid, CretoError>;

    /// Get transition history for a request.
    async fn list_by_request(
        &self,
        request_id: Uuid,
    ) -> Result<Vec<StateTransitionRecord>, CretoError>;
}

/// PostgreSQL implementation of StateTransitionRepository.
pub struct PgStateTransitionRepository {
    pool: PgPool,
}

impl PgStateTransitionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl StateTransitionRepository for PgStateTransitionRepository {
    async fn create(&self, record: &StateTransitionRecord) -> Result<Uuid, CretoError> {
        let row = sqlx::query(
            r#"
            INSERT INTO state_transitions (
                request_id, from_status, to_status, actor_type, actor_id, reason
            ) VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#,
        )
        .bind(record.request_id)
        .bind(record.from_status.as_str())
        .bind(record.to_status.as_str())
        .bind(&record.actor_type)
        .bind(record.actor_id)
        .bind(&record.reason)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn list_by_request(
        &self,
        request_id: Uuid,
    ) -> Result<Vec<StateTransitionRecord>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id, from_status, to_status, actor_type, actor_id, reason, transitioned_at
            FROM state_transitions
            WHERE request_id = $1
            ORDER BY transitioned_at ASC
            "#,
        )
        .bind(request_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| StateTransitionRecord {
                id: r.get("id"),
                request_id,
                from_status: RequestStatus::parse_db_str(r.get::<&str, _>("from_status")),
                to_status: RequestStatus::parse_db_str(r.get::<&str, _>("to_status")),
                actor_type: r.get("actor_type"),
                actor_id: r.get("actor_id"),
                reason: r.get("reason"),
                transitioned_at: r.get("transitioned_at"),
            })
            .collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Checkpoint Repository
// ─────────────────────────────────────────────────────────────────────────────

use crate::checkpoint::Checkpoint;

/// PostgreSQL implementation of CheckpointRepository.
pub struct PgCheckpointRepository {
    pool: PgPool,
}

impl PgCheckpointRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl crate::checkpoint::CheckpointRepository for PgCheckpointRepository {
    async fn create(&self, checkpoint: &Checkpoint) -> Result<Uuid, CretoError> {
        // Serialize checkpoint data as JSONB for flexibility
        let checkpoint_data = serde_json::to_value(checkpoint)
            .map_err(|e| CretoError::SerializationError(e.to_string()))?;

        let row = sqlx::query(
            r#"
            INSERT INTO checkpoints (
                id, request_id, status, checkpoint_data, version, reason, timestamp
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id
            "#,
        )
        .bind(checkpoint.id)
        .bind(checkpoint.request_id)
        .bind(checkpoint.status.as_str())
        .bind(&checkpoint_data)
        .bind(checkpoint.version as i32)
        .bind(&checkpoint.reason)
        .bind(checkpoint.timestamp)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn get(&self, id: Uuid) -> Result<Option<Checkpoint>, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT checkpoint_data
            FROM checkpoints
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        match row {
            Some(r) => {
                let checkpoint_data: serde_json::Value = r.get("checkpoint_data");
                let checkpoint: Checkpoint = serde_json::from_value(checkpoint_data)
                    .map_err(|e| CretoError::SerializationError(e.to_string()))?;
                Ok(Some(checkpoint))
            }
            None => Ok(None),
        }
    }

    async fn get_latest(&self, request_id: Uuid) -> Result<Option<Checkpoint>, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT checkpoint_data
            FROM checkpoints
            WHERE request_id = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(request_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        match row {
            Some(r) => {
                let checkpoint_data: serde_json::Value = r.get("checkpoint_data");
                let checkpoint: Checkpoint = serde_json::from_value(checkpoint_data)
                    .map_err(|e| CretoError::SerializationError(e.to_string()))?;
                Ok(Some(checkpoint))
            }
            None => Ok(None),
        }
    }

    async fn list_by_request(&self, request_id: Uuid) -> Result<Vec<Checkpoint>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT checkpoint_data
            FROM checkpoints
            WHERE request_id = $1
            ORDER BY timestamp DESC
            "#,
        )
        .bind(request_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        let mut checkpoints = Vec::with_capacity(rows.len());
        for r in rows {
            let checkpoint_data: serde_json::Value = r.get("checkpoint_data");
            let checkpoint: Checkpoint = serde_json::from_value(checkpoint_data)
                .map_err(|e| CretoError::SerializationError(e.to_string()))?;
            checkpoints.push(checkpoint);
        }

        Ok(checkpoints)
    }

    async fn delete_before(&self, before: DateTime<Utc>) -> Result<usize, CretoError> {
        let result = sqlx::query(
            r#"
            DELETE FROM checkpoints
            WHERE timestamp < $1
            "#,
        )
        .bind(before)
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(result.rows_affected() as usize)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Quorum Config Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Quorum configuration record for database storage.
#[derive(Debug, Clone)]
pub struct QuorumConfigRecord {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub name: String,
    pub required_approvals: i32,
    pub required_weight: Option<i32>,
    pub any_rejection_rejects: bool,
    pub require_unanimous: bool,
}

/// Repository for quorum configuration.
#[async_trait::async_trait]
pub trait QuorumConfigRepository: Send + Sync {
    /// Get default quorum config for an organization.
    async fn get_default(&self, org_id: OrganizationId) -> Result<QuorumConfigRecord, CretoError>;

    /// Create or update a quorum config.
    async fn upsert(&self, config: &QuorumConfigRecord) -> Result<Uuid, CretoError>;
}

/// PostgreSQL implementation of QuorumConfigRepository.
pub struct PgQuorumConfigRepository {
    pool: PgPool,
}

impl PgQuorumConfigRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl QuorumConfigRepository for PgQuorumConfigRepository {
    async fn get_default(&self, org_id: OrganizationId) -> Result<QuorumConfigRecord, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, required_approvals, required_weight,
                   any_rejection_rejects, require_unanimous
            FROM quorum_configs
            WHERE organization_id = $1 AND action_type IS NULL
            ORDER BY created_at ASC
            LIMIT 1
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(QuorumConfigRecord {
                id: r.get("id"),
                organization_id: org_id,
                name: r.get("name"),
                required_approvals: r.get("required_approvals"),
                required_weight: r.get("required_weight"),
                any_rejection_rejects: r.get("any_rejection_rejects"),
                require_unanimous: r.get("require_unanimous"),
            }),
            None => {
                // Return default config
                Ok(QuorumConfigRecord {
                    id: Uuid::nil(),
                    organization_id: org_id,
                    name: "default".to_string(),
                    required_approvals: 1,
                    required_weight: None,
                    any_rejection_rejects: false,
                    require_unanimous: false,
                })
            }
        }
    }

    async fn upsert(&self, config: &QuorumConfigRecord) -> Result<Uuid, CretoError> {
        let row = sqlx::query(
            r#"
            INSERT INTO quorum_configs (
                organization_id, name, required_approvals, required_weight,
                any_rejection_rejects, require_unanimous
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (organization_id, name)
            DO UPDATE SET
                required_approvals = $3,
                required_weight = $4,
                any_rejection_rejects = $5,
                require_unanimous = $6,
                updated_at = NOW()
            RETURNING id
            "#,
        )
        .bind(config.organization_id.as_uuid())
        .bind(&config.name)
        .bind(config.required_approvals)
        .bind(config.required_weight)
        .bind(config.any_rejection_rejects)
        .bind(config.require_unanimous)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("id"))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────────────────────

fn action_type_to_str(action_type: &ActionType) -> &'static str {
    match action_type {
        ActionType::Transaction { .. } => "transaction",
        ActionType::DataAccess { .. } => "data_access",
        ActionType::ExternalApi { .. } => "external_api",
        ActionType::CodeExecution { .. } => "code_execution",
        ActionType::Communication { .. } => "communication",
        ActionType::Custom { .. } => "custom",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_status_roundtrip() {
        assert_eq!(
            RequestStatus::parse_db_str("pending"),
            RequestStatus::Pending
        );
        assert_eq!(RequestStatus::Pending.as_str(), "pending");
    }

    #[test]
    fn test_priority_roundtrip() {
        assert_eq!(Priority::parse_db_str("high"), Priority::High);
        assert_eq!(Priority::High.as_str(), "high");
    }

    #[test]
    fn test_approval_decision_roundtrip() {
        assert_eq!(
            ApprovalDecision::parse_db_str("approve"),
            ApprovalDecision::Approve
        );
        assert_eq!(ApprovalDecision::Approve.as_str(), "approve");
    }

    #[test]
    fn test_default_approval_counts() {
        let counts = ApprovalCounts::default();
        assert_eq!(counts.approve, 0);
        assert_eq!(counts.reject, 0);
    }
}
