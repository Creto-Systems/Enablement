//! Repository traits and PostgreSQL implementations for messaging.
//!
//! This module provides the persistence layer for key bundles, sessions,
//! message envelopes, and delivery tracking.
//!
//! Uses runtime SQL queries to avoid requiring DATABASE_URL at compile time.

use chrono::{DateTime, Utc};
use creto_common::{AgentId, CretoError, OrganizationId};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::channel::ChannelType;
use crate::session::SessionState;

// ─────────────────────────────────────────────────────────────────────────────
// Enum Serialization Helpers
// ─────────────────────────────────────────────────────────────────────────────

impl SessionState {
    /// Convert to database string.
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionState::Establishing => "establishing",
            SessionState::Active => "active",
            SessionState::Suspended => "suspended",
            SessionState::Closed => "closed",
            SessionState::Failed => "failed",
        }
    }

    /// Parse from database string.
    pub fn parse_db_str(s: &str) -> Self {
        match s {
            "establishing" => SessionState::Establishing,
            "active" => SessionState::Active,
            "suspended" => SessionState::Suspended,
            "closed" => SessionState::Closed,
            "failed" => SessionState::Failed,
            _ => SessionState::Establishing,
        }
    }
}

impl ChannelType {
    /// Convert to database string.
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelType::Direct => "direct",
            ChannelType::Queue => "queue",
            ChannelType::PubSub => "pubsub",
            ChannelType::StoreForward => "store_forward",
            ChannelType::Webhook => "webhook",
        }
    }

    /// Parse from database string.
    pub fn parse_db_str(s: &str) -> Self {
        match s {
            "direct" => ChannelType::Direct,
            "queue" => ChannelType::Queue,
            "pubsub" => ChannelType::PubSub,
            "store_forward" => ChannelType::StoreForward,
            "webhook" => ChannelType::Webhook,
            _ => ChannelType::Direct,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Key Bundle Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Simplified key bundle record for database storage.
#[derive(Debug, Clone)]
pub struct KeyBundleRecord {
    pub id: Uuid,
    pub agent_id: AgentId,
    pub identity_public_key: Vec<u8>,
    pub signed_prekey_public: Vec<u8>,
    pub signed_prekey_signature: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

/// Repository for key bundle persistence.
#[async_trait::async_trait]
pub trait KeyBundleRepository: Send + Sync {
    /// Store or update a key bundle for an agent.
    async fn upsert(
        &self,
        agent_id: AgentId,
        identity_public_key: &[u8],
        signed_prekey_public: &[u8],
        signed_prekey_signature: &[u8],
    ) -> Result<Uuid, CretoError>;

    /// Get key bundle for an agent.
    async fn get_by_agent(&self, agent_id: AgentId) -> Result<Option<KeyBundleRecord>, CretoError>;

    /// Delete key bundle.
    async fn delete(&self, agent_id: AgentId) -> Result<(), CretoError>;
}

/// PostgreSQL implementation of KeyBundleRepository.
pub struct PgKeyBundleRepository {
    pool: PgPool,
}

impl PgKeyBundleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl KeyBundleRepository for PgKeyBundleRepository {
    async fn upsert(
        &self,
        agent_id: AgentId,
        identity_public_key: &[u8],
        signed_prekey_public: &[u8],
        signed_prekey_signature: &[u8],
    ) -> Result<Uuid, CretoError> {
        let row = sqlx::query(
            r#"
            INSERT INTO key_bundles (
                agent_id, identity_key_id, identity_public_key,
                signed_prekey_id, signed_prekey_public, signed_prekey_signature,
                signed_prekey_timestamp
            ) VALUES ($1, $2, $3, 1, $4, $5, $6)
            ON CONFLICT (agent_id) DO UPDATE SET
                identity_public_key = $3,
                signed_prekey_public = $4,
                signed_prekey_signature = $5,
                signed_prekey_timestamp = $6,
                updated_at = NOW()
            RETURNING id
            "#,
        )
        .bind(agent_id.as_uuid())
        .bind(agent_id.as_uuid()) // identity_key_id
        .bind(identity_public_key)
        .bind(signed_prekey_public)
        .bind(signed_prekey_signature)
        .bind(Utc::now().timestamp())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn get_by_agent(&self, agent_id: AgentId) -> Result<Option<KeyBundleRecord>, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT id, identity_public_key, signed_prekey_public, signed_prekey_signature, created_at
            FROM key_bundles
            WHERE agent_id = $1
            "#,
        )
        .bind(agent_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.map(|r| KeyBundleRecord {
            id: r.get("id"),
            agent_id,
            identity_public_key: r.get("identity_public_key"),
            signed_prekey_public: r.get("signed_prekey_public"),
            signed_prekey_signature: r.get("signed_prekey_signature"),
            created_at: r.get("created_at"),
        }))
    }

    async fn delete(&self, agent_id: AgentId) -> Result<(), CretoError> {
        sqlx::query(r#"DELETE FROM key_bundles WHERE agent_id = $1"#)
            .bind(agent_id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PreKey Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Repository for one-time prekey persistence.
#[async_trait::async_trait]
pub trait PreKeyRepository: Send + Sync {
    /// Store a prekey.
    async fn store(
        &self,
        agent_id: AgentId,
        prekey_id: i32,
        public_key: &[u8],
    ) -> Result<(), CretoError>;

    /// Consume a prekey (mark as used and return it).
    async fn consume(&self, agent_id: AgentId) -> Result<Option<(i32, Vec<u8>)>, CretoError>;

    /// Count available prekeys.
    async fn count_available(&self, agent_id: AgentId) -> Result<i64, CretoError>;
}

/// PostgreSQL implementation of PreKeyRepository.
pub struct PgPreKeyRepository {
    pool: PgPool,
}

impl PgPreKeyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl PreKeyRepository for PgPreKeyRepository {
    async fn store(
        &self,
        agent_id: AgentId,
        prekey_id: i32,
        public_key: &[u8],
    ) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            INSERT INTO prekeys (agent_id, prekey_id, public_key)
            VALUES ($1, $2, $3)
            ON CONFLICT (agent_id, prekey_id) DO NOTHING
            "#,
        )
        .bind(agent_id.as_uuid())
        .bind(prekey_id)
        .bind(public_key)
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }

    async fn consume(&self, agent_id: AgentId) -> Result<Option<(i32, Vec<u8>)>, CretoError> {
        let row = sqlx::query(
            r#"
            UPDATE prekeys
            SET consumed = true, consumed_at = NOW()
            WHERE id = (
                SELECT id FROM prekeys
                WHERE agent_id = $1 AND consumed = false
                ORDER BY prekey_id ASC
                LIMIT 1
                FOR UPDATE SKIP LOCKED
            )
            RETURNING prekey_id, public_key
            "#,
        )
        .bind(agent_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.map(|r| {
            (
                r.get::<i32, _>("prekey_id"),
                r.get::<Vec<u8>, _>("public_key"),
            )
        }))
    }

    async fn count_available(&self, agent_id: AgentId) -> Result<i64, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM prekeys
            WHERE agent_id = $1 AND consumed = false
            "#,
        )
        .bind(agent_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("count"))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Session Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Simplified session record.
#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub id: Uuid,
    pub local_agent_id: AgentId,
    pub remote_agent_id: AgentId,
    pub state: SessionState,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

/// Repository for messaging session persistence.
#[async_trait::async_trait]
pub trait SessionRepository: Send + Sync {
    /// Create or update a session.
    async fn upsert(
        &self,
        local_agent_id: AgentId,
        remote_agent_id: AgentId,
    ) -> Result<Uuid, CretoError>;

    /// Get session by local and remote agent IDs.
    async fn get(
        &self,
        local_agent_id: AgentId,
        remote_agent_id: AgentId,
    ) -> Result<Option<SessionRecord>, CretoError>;

    /// Update session state.
    async fn update_state(&self, id: Uuid, state: SessionState) -> Result<(), CretoError>;

    /// List active sessions for an agent.
    async fn list_active(&self, agent_id: AgentId) -> Result<Vec<SessionRecord>, CretoError>;
}

/// PostgreSQL implementation of SessionRepository.
pub struct PgSessionRepository {
    pool: PgPool,
}

impl PgSessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl SessionRepository for PgSessionRepository {
    async fn upsert(
        &self,
        local_agent_id: AgentId,
        remote_agent_id: AgentId,
    ) -> Result<Uuid, CretoError> {
        let row = sqlx::query(
            r#"
            INSERT INTO messaging_sessions (local_agent_id, remote_agent_id, state)
            VALUES ($1, $2, 'establishing')
            ON CONFLICT (local_agent_id, remote_agent_id) DO UPDATE SET
                last_active_at = NOW()
            RETURNING id
            "#,
        )
        .bind(local_agent_id.as_uuid())
        .bind(remote_agent_id.as_uuid())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn get(
        &self,
        local_agent_id: AgentId,
        remote_agent_id: AgentId,
    ) -> Result<Option<SessionRecord>, CretoError> {
        let row = sqlx::query(
            r#"
            SELECT id, state, created_at, last_active_at
            FROM messaging_sessions
            WHERE local_agent_id = $1 AND remote_agent_id = $2
            "#,
        )
        .bind(local_agent_id.as_uuid())
        .bind(remote_agent_id.as_uuid())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.map(|r| SessionRecord {
            id: r.get("id"),
            local_agent_id,
            remote_agent_id,
            state: SessionState::parse_db_str(r.get::<&str, _>("state")),
            created_at: r.get("created_at"),
            last_active_at: r.get("last_active_at"),
        }))
    }

    async fn update_state(&self, id: Uuid, state: SessionState) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            UPDATE messaging_sessions
            SET state = $2, last_active_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(state.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }

    async fn list_active(&self, agent_id: AgentId) -> Result<Vec<SessionRecord>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id, local_agent_id, remote_agent_id, state, created_at, last_active_at
            FROM messaging_sessions
            WHERE (local_agent_id = $1 OR remote_agent_id = $1)
              AND state IN ('establishing', 'active')
            ORDER BY last_active_at DESC
            "#,
        )
        .bind(agent_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| SessionRecord {
                id: r.get("id"),
                local_agent_id: AgentId::from_uuid(r.get::<Uuid, _>("local_agent_id")),
                remote_agent_id: AgentId::from_uuid(r.get::<Uuid, _>("remote_agent_id")),
                state: SessionState::parse_db_str(r.get::<&str, _>("state")),
                created_at: r.get("created_at"),
                last_active_at: r.get("last_active_at"),
            })
            .collect())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Envelope Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Simplified envelope record.
#[derive(Debug, Clone)]
pub struct EnvelopeRecord {
    pub id: Uuid,
    pub sender_id: AgentId,
    pub recipient_id: AgentId,
    pub ciphertext: Vec<u8>,
    pub delivered: bool,
    pub created_at: DateTime<Utc>,
}

/// Repository for message envelope persistence (store-and-forward).
#[async_trait::async_trait]
pub trait EnvelopeRepository: Send + Sync {
    /// Store a message envelope.
    async fn store(
        &self,
        sender_id: AgentId,
        recipient_id: AgentId,
        ciphertext: &[u8],
        dh_public: &[u8],
        mac: &[u8],
    ) -> Result<Uuid, CretoError>;

    /// Get undelivered envelopes for a recipient.
    async fn get_undelivered(
        &self,
        recipient_id: AgentId,
        limit: i64,
    ) -> Result<Vec<EnvelopeRecord>, CretoError>;

    /// Mark envelope as delivered.
    async fn mark_delivered(&self, id: Uuid) -> Result<(), CretoError>;

    /// Delete expired envelopes.
    async fn cleanup_expired(&self) -> Result<i64, CretoError>;
}

/// PostgreSQL implementation of EnvelopeRepository.
pub struct PgEnvelopeRepository {
    pool: PgPool,
}

impl PgEnvelopeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl EnvelopeRepository for PgEnvelopeRepository {
    async fn store(
        &self,
        sender_id: AgentId,
        recipient_id: AgentId,
        ciphertext: &[u8],
        dh_public: &[u8],
        mac: &[u8],
    ) -> Result<Uuid, CretoError> {
        let row = sqlx::query(
            r#"
            INSERT INTO message_envelopes (
                sender_id, recipient_id, envelope_version, content_type,
                dh_public, prev_chain_length, message_number,
                ciphertext, mac
            ) VALUES ($1, $2, 1, 'message', $3, 0, 0, $4, $5)
            RETURNING id
            "#,
        )
        .bind(sender_id.as_uuid())
        .bind(recipient_id.as_uuid())
        .bind(dh_public)
        .bind(ciphertext)
        .bind(mac)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn get_undelivered(
        &self,
        recipient_id: AgentId,
        limit: i64,
    ) -> Result<Vec<EnvelopeRecord>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id, sender_id, ciphertext, delivered, created_at
            FROM message_envelopes
            WHERE recipient_id = $1
              AND delivered = false
              AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY created_at ASC
            LIMIT $2
            "#,
        )
        .bind(recipient_id.as_uuid())
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| EnvelopeRecord {
                id: r.get("id"),
                sender_id: AgentId::from_uuid(r.get::<Uuid, _>("sender_id")),
                recipient_id,
                ciphertext: r.get("ciphertext"),
                delivered: r.get("delivered"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    async fn mark_delivered(&self, id: Uuid) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            UPDATE message_envelopes
            SET delivered = true, delivered_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }

    async fn cleanup_expired(&self) -> Result<i64, CretoError> {
        let result = sqlx::query(
            r#"
            DELETE FROM message_envelopes
            WHERE expires_at < NOW() AND delivered = false
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(result.rows_affected() as i64)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Channel Repository
// ─────────────────────────────────────────────────────────────────────────────

/// Simplified channel record.
#[derive(Debug, Clone)]
pub struct ChannelRecord {
    pub id: Uuid,
    pub organization_id: OrganizationId,
    pub channel_type: ChannelType,
    pub name: String,
    pub active: bool,
}

/// Repository for messaging channel configuration.
#[async_trait::async_trait]
pub trait ChannelRepository: Send + Sync {
    /// Create a channel.
    async fn create(
        &self,
        org_id: OrganizationId,
        channel_type: ChannelType,
        name: &str,
    ) -> Result<Uuid, CretoError>;

    /// List active channels for an organization.
    async fn list_active(&self, org_id: OrganizationId) -> Result<Vec<ChannelRecord>, CretoError>;

    /// Deactivate a channel.
    async fn deactivate(&self, id: Uuid) -> Result<(), CretoError>;
}

/// PostgreSQL implementation of ChannelRepository.
pub struct PgChannelRepository {
    pool: PgPool,
}

impl PgChannelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl ChannelRepository for PgChannelRepository {
    async fn create(
        &self,
        org_id: OrganizationId,
        channel_type: ChannelType,
        name: &str,
    ) -> Result<Uuid, CretoError> {
        let row = sqlx::query(
            r#"
            INSERT INTO messaging_channels (organization_id, channel_type, name, config, retry_policy)
            VALUES ($1, $2, $3, '{}', '{"max_attempts": 3, "initial_backoff_ms": 100}')
            RETURNING id
            "#,
        )
        .bind(org_id.as_uuid())
        .bind(channel_type.as_str())
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(row.get("id"))
    }

    async fn list_active(&self, org_id: OrganizationId) -> Result<Vec<ChannelRecord>, CretoError> {
        let rows = sqlx::query(
            r#"
            SELECT id, channel_type, name, active
            FROM messaging_channels
            WHERE organization_id = $1 AND active = true
            ORDER BY name ASC
            "#,
        )
        .bind(org_id.as_uuid())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| ChannelRecord {
                id: r.get("id"),
                organization_id: org_id,
                channel_type: ChannelType::parse_db_str(r.get::<&str, _>("channel_type")),
                name: r.get("name"),
                active: r.get("active"),
            })
            .collect())
    }

    async fn deactivate(&self, id: Uuid) -> Result<(), CretoError> {
        sqlx::query(
            r#"
            UPDATE messaging_channels
            SET active = false, updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| CretoError::Database(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_roundtrip() {
        assert_eq!(
            SessionState::parse_db_str("establishing"),
            SessionState::Establishing
        );
        assert_eq!(SessionState::Establishing.as_str(), "establishing");
    }

    #[test]
    fn test_channel_type_roundtrip() {
        assert_eq!(ChannelType::parse_db_str("direct"), ChannelType::Direct);
        assert_eq!(ChannelType::Direct.as_str(), "direct");
    }
}
