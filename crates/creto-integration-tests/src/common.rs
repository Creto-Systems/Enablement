//! Common test utilities for integration tests.
//!
//! This module provides shared test infrastructure for the Creto Enablement Layer.

use creto_common::{AgentId, OrganizationId, UserId};
use uuid::Uuid;

/// Test database configuration.
/// Uses environment variable TEST_DATABASE_URL or falls back to a test database.
pub fn test_database_url() -> String {
    std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/creto_enablement_test".to_string())
}

/// Create a test organization ID.
pub fn test_org_id() -> OrganizationId {
    OrganizationId::from_uuid(Uuid::now_v7())
}

/// Create a test agent ID.
pub fn test_agent_id() -> AgentId {
    AgentId::from_uuid(Uuid::now_v7())
}

/// Create a test user ID.
pub fn test_user_id() -> UserId {
    UserId::from_uuid(Uuid::now_v7())
}

/// Test fixture for common test data.
pub struct TestFixture {
    pub org_id: OrganizationId,
    pub agent_id: AgentId,
    pub user_id: UserId,
}

impl TestFixture {
    /// Create a new test fixture with random IDs.
    pub fn new() -> Self {
        Self {
            org_id: test_org_id(),
            agent_id: test_agent_id(),
            user_id: test_user_id(),
        }
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Database test context that handles setup and cleanup.
#[cfg(feature = "database")]
pub struct TestDatabase {
    pub pool: sqlx::PgPool,
    pub fixture: TestFixture,
}

#[cfg(feature = "database")]
impl TestDatabase {
    /// Create a new test database context.
    pub async fn new() -> Result<Self, sqlx::Error> {
        let pool = sqlx::PgPool::connect(&test_database_url()).await?;
        Ok(Self {
            pool,
            fixture: TestFixture::new(),
        })
    }

    /// Run migrations on the test database.
    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        // Run migrations from the migrations directory
        sqlx::migrate!("../../migrations").run(&self.pool).await?;
        Ok(())
    }

    /// Clean up test data (truncate tables).
    pub async fn cleanup(&self) -> Result<(), sqlx::Error> {
        // Truncate in correct order to respect foreign keys
        sqlx::query("TRUNCATE TABLE messaging_messages CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE messaging_prekeys CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE messaging_identity_keys CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE runtime_resource_usage CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE runtime_executions CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE runtime_sandboxes CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE oversight_approvals CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE oversight_state_transitions CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE oversight_requests CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE oversight_quorum_configs CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE metering_quota_limits CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE metering_quota_usage CASCADE")
            .execute(&self.pool)
            .await?;
        sqlx::query("TRUNCATE TABLE metering_events CASCADE")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

/// Assert that a result is Ok and return the value.
#[macro_export]
macro_rules! assert_ok {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
}

/// Assert that a result is Err.
#[macro_export]
macro_rules! assert_err {
    ($result:expr) => {
        match $result {
            Ok(value) => panic!("Expected Err, got Ok: {:?}", value),
            Err(e) => e,
        }
    };
}
