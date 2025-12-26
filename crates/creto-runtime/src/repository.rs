//! Repository traits and PostgreSQL implementations for runtime.
//!
//! This module provides the persistence layer for sandboxes, execution
//! requests, resource usage, and warm pool management.
//!
//! Uses runtime SQL queries to avoid requiring DATABASE_URL at compile time.

use chrono::{DateTime, Utc};
use creto_common::{AgentId, CretoError, OrganizationId};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::sandbox::{SandboxId, SandboxState};
use crate::execution::ExecutionStatus;
use crate::resources::ResourceUsage;

// ─────────────────────────────────────────────────────────────────────────────
// Enum Serialization Helpers
// ─────────────────────────────────────────────────────────────────────────────

impl SandboxState {
    /// Convert to database string.
    pub fn as_str(&self) -> &'static str {
        match self {
            SandboxState::Creating => "creating",
            SandboxState::Ready => "ready",
            SandboxState::Running => "running",
            SandboxState::Paused => "paused",
            SandboxState::Stopped => "stopped",
            SandboxState::Failed => "failed",
            SandboxState::Terminated => "terminated",
        }
    }

    /// Parse from database string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "creating" => SandboxState::Creating,
            "ready" => SandboxState::Ready,
            "running" => SandboxState::Running,
            "paused" => SandboxState::Paused,
            "stopped" => SandboxState::Stopped,
            "failed" => SandboxState::Failed,
            "terminated" => SandboxState::Terminated,
            _ => SandboxState::Creating,
        }
    }
}

impl ExecutionStatus {
    /// Convert to database string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutionStatus::Queued => "queued",
            ExecutionStatus::Running => "running",
            ExecutionStatus::Completed => "completed",
            ExecutionStatus::Failed => "failed",
            ExecutionStatus::TimedOut => "timed_out",
            ExecutionStatus::Cancelled => "cancelled",
        }
    }

    /// Parse from database string.
    pub fn from_str(s: &str) -> Self {
        match s {
            "queued" => ExecutionStatus::Queued,
            "running" => ExecutionStatus::Running,
            "completed" => ExecutionStatus::Completed,
            "failed" => ExecutionStatus::Failed,
            "timed_out" => ExecutionStatus::TimedOut,
            "cancelled" => ExecutionStatus::Cancelled,
            _ => ExecutionStatus::Queued,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Sandbox Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Simplified sandbox record for database storage.
#[derive(Debug, Clone)]
pub struct SandboxRecord {
    pub id: SandboxId,
    pub organization_id: OrganizationId,
    pub agent_id: AgentId,
    pub runtime: String,
    pub state: SandboxState,
    pub network_policy: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Repository for sandbox persistence.
#[async_trait::async_trait]
pub trait SandboxRepository: Send + Sync {
    /// Create a new sandbox record.
    async fn create(
        &self,
        org_id: OrganizationId,
        agent_id: AgentId,
        runtime: &str,
        network_policy: &str,
    ) -> Result<SandboxId, CretoError>;

    /// Get a sandbox by ID.
    async fn get(&self, id: SandboxId) -> Result<Option<SandboxRecord>, CretoError>;

    /// Update sandbox state.
    async fn update_state(&self, id: SandboxId, state: SandboxState) -> Result<(), CretoError>;

    /// Mark sandbox as terminated.
    async fn terminate(&self, id: SandboxId) -> Result<(), CretoError>;

    /// List active sandboxes by organization.
    async fn list_active_by_org(&self, org_id: OrganizationId) -> Result<Vec<SandboxRecord>, CretoError>;

    /// Find idle sandboxes for cleanup.
    async fn find_idle(&self, idle_since: DateTime<Utc>) -> Result<Vec<SandboxId>, CretoError>;
}

/// PostgreSQL implementation of SandboxRepository.
pub struct PgSandboxRepository {
    pool: PgPool,
}

impl PgSandboxRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SandboxRepository for PgSandboxRepository {
    async fn create(
        &self,
        org_id: OrganizationId,
        agent_id: AgentId,
        runtime: &str,
        network_policy: &str,
    ) -> Result<SandboxId, CretoError> {
        let row = sqlx::query(
            r#"
            INSERT INTO sandboxes (
                organization_id, agent_id, runtime, state, config,
                resource_limits, network_policy
            ) VALUES ($1, $2, $3, 'creating', '{}', '{}', $4)
            RETURNING id
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(agent_id.as_uuid())
        .bind(runtime)
        .bind(network_policy)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(SandboxId::from_uuid(row.get::<Uuid, _>("id")))
    }

    async fn get(&self, id: SandboxId) -> Result<Option<SandboxRecord>, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT organization_id, agent_id, runtime, state, network_policy,
                   created_at, last_used_at
            FROM sandboxes
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.map(|r| SandboxRecord {
            id,
            organization_id: OrganizationId::from_uuid(r.get::<Uuid, _>("organization_id")),
            agent_id: AgentId::from_uuid(r.get::<Uuid, _>("agent_id")),
            runtime: r.get("runtime"),
            state: SandboxState::from_str(r.get::<&str, _>("state")),
            network_policy: r.get("network_policy"),
            created_at: r.get("created_at"),
            last_used_at: r.get("last_used_at"),
        }))
    }

    async fn update_state(&self, id: SandboxId, state: SandboxState) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            UPDATE sandboxes
            SET state = $2, last_used_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .bind(state.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }

    async fn terminate(&self, id: SandboxId) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            UPDATE sandboxes
            SET state = 'terminated', terminated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id.as_uuid())
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_active_by_org(&self, org_id: OrganizationId) -> Result<Vec<SandboxRecord>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id, agent_id, runtime, state, network_policy, created_at, last_used_at
            FROM sandboxes
            WHERE organization_id = $1 AND state NOT IN ('terminated', 'failed')
            ORDER BY created_at DESC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| SandboxRecord {
                id: SandboxId::from_uuid(r.get::<Uuid, _>("id")),
                organization_id: org_id,
                agent_id: AgentId::from_uuid(r.get::<Uuid, _>("agent_id")),
                runtime: r.get("runtime"),
                state: SandboxState::from_str(r.get::<&str, _>("state")),
                network_policy: r.get("network_policy"),
                created_at: r.get("created_at"),
                last_used_at: r.get("last_used_at"),
            })
            .collect())
    }

    async fn find_idle(&self, idle_since: DateTime<Utc>) -> Result<Vec<SandboxId>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id
            FROM sandboxes
            WHERE state IN ('ready', 'paused')
              AND (last_used_at < $1 OR (last_used_at IS NULL AND created_at < $1))
            "#,
        )
        .bind(idle_since)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| SandboxId::from_uuid(r.get::<Uuid, _>("id"))).collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Execution Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Simplified execution request record.
#[derive(Debug, Clone)]
pub struct ExecutionRecord {
    pub id: Uuid,
    pub sandbox_id: SandboxId,
    pub status: ExecutionStatus,
    pub queued_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
}

/// Repository for execution request persistence.
#[async_trait::async_trait]
pub trait ExecutionRepository: Send + Sync {
    /// Create an execution request.
    async fn create(&self, sandbox_id: SandboxId, code: &str, timeout_seconds: i32) -> Result<Uuid, CretoError>;

    /// Get execution request by ID.
    async fn get(&self, id: Uuid) -> Result<Option<ExecutionRecord>, CretoError>;

    /// Update execution status.
    async fn update_status(&self, id: Uuid, status: ExecutionStatus) -> Result<(), CretoError>;

    /// Mark execution as started.
    async fn mark_started(&self, id: Uuid) -> Result<(), CretoError>;

    /// Mark execution as completed.
    async fn mark_completed(&self, id: Uuid, duration_ms: i64) -> Result<(), CretoError>;

    /// List pending executions for a sandbox.
    async fn list_pending_by_sandbox(&self, sandbox_id: SandboxId) -> Result<Vec<ExecutionRecord>, CretoError>;
}

/// PostgreSQL implementation of ExecutionRepository.
pub struct PgExecutionRepository {
    pool: PgPool,
}

impl PgExecutionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ExecutionRepository for PgExecutionRepository {
    async fn create(&self, sandbox_id: SandboxId, code: &str, timeout_seconds: i32) -> Result<Uuid, CretoError> {
        let row = sqlx::query(
            r#"
            INSERT INTO execution_requests (
                sandbox_id, code, timeout_seconds, status
            ) VALUES ($1, $2, $3, 'queued')
            RETURNING id
            "#,
        )
        .bind(sandbox_id.as_uuid())
        .bind(code)
        .bind(timeout_seconds)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn get(&self, id: Uuid) -> Result<Option<ExecutionRecord>, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT sandbox_id, status, queued_at, started_at, completed_at, duration_ms
            FROM execution_requests
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.map(|r| ExecutionRecord {
            id,
            sandbox_id: SandboxId::from_uuid(r.get::<Uuid, _>("sandbox_id")),
            status: ExecutionStatus::from_str(r.get::<&str, _>("status")),
            queued_at: r.get("queued_at"),
            started_at: r.get("started_at"),
            completed_at: r.get("completed_at"),
            duration_ms: r.get("duration_ms"),
        }))
    }

    async fn update_status(&self, id: Uuid, status: ExecutionStatus) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            UPDATE execution_requests
            SET status = $2
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

    async fn mark_started(&self, id: Uuid) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            UPDATE execution_requests
            SET status = 'running', started_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }

    async fn mark_completed(&self, id: Uuid, duration_ms: i64) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            UPDATE execution_requests
            SET status = 'completed', completed_at = NOW(), duration_ms = $2
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(duration_ms)
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_pending_by_sandbox(&self, sandbox_id: SandboxId) -> Result<Vec<ExecutionRecord>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id, status, queued_at, started_at, completed_at, duration_ms
            FROM execution_requests
            WHERE sandbox_id = $1 AND status IN ('queued', 'running')
            ORDER BY queued_at ASC
            "#,
        )
        .bind(sandbox_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| ExecutionRecord {
                id: r.get("id"),
                sandbox_id,
                status: ExecutionStatus::from_str(r.get::<&str, _>("status")),
                queued_at: r.get("queued_at"),
                started_at: r.get("started_at"),
                completed_at: r.get("completed_at"),
                duration_ms: r.get("duration_ms"),
            })
            .collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Resource Usage Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Repository for resource usage tracking.
#[async_trait::async_trait]
pub trait ResourceUsageRepository: Send + Sync {
    /// Record a resource usage snapshot.
    async fn record(&self, sandbox_id: SandboxId, usage: &ResourceUsage) -> Result<(), CretoError>;

    /// Get latest resource usage for a sandbox.
    async fn get_latest(&self, sandbox_id: SandboxId) -> Result<Option<ResourceUsage>, CretoError>;
}

/// PostgreSQL implementation of ResourceUsageRepository.
pub struct PgResourceUsageRepository {
    pool: PgPool,
}

impl PgResourceUsageRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ResourceUsageRepository for PgResourceUsageRepository {
    async fn record(&self, sandbox_id: SandboxId, usage: &ResourceUsage) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            INSERT INTO resource_usage (
                sandbox_id, memory_bytes, peak_memory_bytes, cpu_time_ms,
                wall_time_ms, disk_bytes, process_count, open_file_count,
                network_bytes_sent, network_bytes_received, connection_count
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(sandbox_id.as_uuid())
        .bind(usage.memory_bytes as i64)
        .bind(usage.peak_memory_bytes as i64)
        .bind(usage.cpu_time_ms as i64)
        .bind(usage.wall_time_ms as i64)
        .bind(usage.disk_bytes as i64)
        .bind(usage.process_count as i32)
        .bind(usage.open_file_count as i32)
        .bind(usage.network_bytes_sent as i64)
        .bind(usage.network_bytes_received as i64)
        .bind(usage.connection_count as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }

    async fn get_latest(&self, sandbox_id: SandboxId) -> Result<Option<ResourceUsage>, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT memory_bytes, peak_memory_bytes, cpu_time_ms, wall_time_ms,
                   disk_bytes, process_count, open_file_count,
                   network_bytes_sent, network_bytes_received, connection_count
            FROM resource_usage
            WHERE sandbox_id = $1
            ORDER BY recorded_at DESC
            LIMIT 1
            "#,
        )
        .bind(sandbox_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.map(|r| ResourceUsage {
            memory_bytes: r.get::<i64, _>("memory_bytes") as u64,
            peak_memory_bytes: r.get::<i64, _>("peak_memory_bytes") as u64,
            cpu_time_ms: r.get::<i64, _>("cpu_time_ms") as u64,
            wall_time_ms: r.get::<i64, _>("wall_time_ms") as u64,
            disk_bytes: r.get::<i64, _>("disk_bytes") as u64,
            process_count: r.get::<i32, _>("process_count") as u32,
            open_file_count: r.get::<i32, _>("open_file_count") as u32,
            network_bytes_sent: r.get::<i64, _>("network_bytes_sent") as u64,
            network_bytes_received: r.get::<i64, _>("network_bytes_received") as u64,
            connection_count: r.get::<i32, _>("connection_count") as u32,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_state_roundtrip() {
        assert_eq!(SandboxState::from_str("creating"), SandboxState::Creating);
        assert_eq!(SandboxState::Creating.as_str(), "creating");
    }

    #[test]
    fn test_execution_status_roundtrip() {
        assert_eq!(ExecutionStatus::from_str("queued"), ExecutionStatus::Queued);
        assert_eq!(ExecutionStatus::Queued.as_str(), "queued");
    }
}
