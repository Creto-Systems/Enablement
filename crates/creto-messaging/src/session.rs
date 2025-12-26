//! Messaging sessions between agents.

use chrono::{DateTime, Utc};
use creto_common::{AgentId, CretoResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    envelope::Envelope,
    ratchet::{DoubleRatchet, EncryptedMessage, RatchetState},
    x3dh::X3DHResult,
};

/// A messaging session between two agents.
pub struct Session {
    /// Session identifier.
    pub id: Uuid,

    /// Local agent ID.
    pub local_agent: AgentId,

    /// Remote agent ID.
    pub remote_agent: AgentId,

    /// Double Ratchet state.
    ratchet: DoubleRatchet,

    /// Session state.
    pub state: SessionState,

    /// Creation timestamp.
    pub created_at: DateTime<Utc>,

    /// Last activity timestamp.
    pub last_active_at: DateTime<Utc>,
}

impl Session {
    /// Create a new session as initiator.
    pub fn new_initiator(
        local_agent: AgentId,
        remote_agent: AgentId,
        x3dh_result: &X3DHResult,
    ) -> Self {
        let their_dh = &x3dh_result.params.recipient_identity;
        let ratchet = DoubleRatchet::new_sender(&x3dh_result.shared_secret, their_dh);

        Self {
            id: Uuid::now_v7(),
            local_agent,
            remote_agent,
            ratchet,
            state: SessionState::Active,
            created_at: Utc::now(),
            last_active_at: Utc::now(),
        }
    }

    /// Create a new session as responder.
    pub fn new_responder(
        local_agent: AgentId,
        remote_agent: AgentId,
        x3dh_result: &X3DHResult,
        our_signed_prekey_public: &[u8],
        our_signed_prekey_private: &[u8],
    ) -> Self {
        let ratchet = DoubleRatchet::new_receiver(
            &x3dh_result.shared_secret,
            our_signed_prekey_public,
            our_signed_prekey_private,
        );

        Self {
            id: Uuid::now_v7(),
            local_agent,
            remote_agent,
            ratchet,
            state: SessionState::Active,
            created_at: Utc::now(),
            last_active_at: Utc::now(),
        }
    }

    /// Encrypt a message.
    pub fn encrypt(&mut self, plaintext: &[u8]) -> CretoResult<Envelope> {
        if self.state != SessionState::Active {
            return Err(creto_common::CretoError::SessionError(
                "Session is not active".to_string(),
            ));
        }

        let encrypted = self.ratchet.encrypt(plaintext)?;

        self.last_active_at = Utc::now();

        Ok(Envelope::new(
            self.local_agent,
            self.remote_agent,
            encrypted.header,
            encrypted.ciphertext,
        ))
    }

    /// Decrypt a message.
    pub fn decrypt(&mut self, envelope: &Envelope) -> CretoResult<Vec<u8>> {
        if self.state != SessionState::Active {
            return Err(creto_common::CretoError::SessionError(
                "Session is not active".to_string(),
            ));
        }

        // Verify sender
        if envelope.header.sender_id != self.remote_agent {
            return Err(creto_common::CretoError::SessionError(
                "Invalid sender for this session".to_string(),
            ));
        }

        let encrypted = EncryptedMessage {
            header: envelope.header.ratchet_header.clone(),
            ciphertext: envelope.payload.ciphertext.clone(),
        };

        let plaintext = self.ratchet.decrypt(&encrypted)?;

        self.last_active_at = Utc::now();

        Ok(plaintext)
    }

    /// Get the session's ratchet state for persistence.
    pub fn ratchet_state(&self) -> &RatchetState {
        self.ratchet.state()
    }

    /// Check if the session is active.
    pub fn is_active(&self) -> bool {
        self.state == SessionState::Active
    }

    /// Close the session.
    pub fn close(&mut self) {
        self.state = SessionState::Closed;
    }

    /// Check if session has been idle too long.
    pub fn is_idle(&self, timeout_seconds: i64) -> bool {
        let idle_duration = Utc::now().signed_duration_since(self.last_active_at);
        idle_duration.num_seconds() > timeout_seconds
    }
}

/// State of a messaging session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionState {
    /// Session is being established.
    Establishing,
    /// Session is active and can send/receive.
    Active,
    /// Session is suspended (can be resumed).
    Suspended,
    /// Session is closed (cannot be reused).
    Closed,
    /// Session failed during establishment.
    Failed,
}

/// Serializable session metadata (for storage).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session ID.
    pub id: Uuid,

    /// Local agent.
    pub local_agent: AgentId,

    /// Remote agent.
    pub remote_agent: AgentId,

    /// Session state.
    pub state: SessionState,

    /// Created timestamp.
    pub created_at: DateTime<Utc>,

    /// Last active timestamp.
    pub last_active_at: DateTime<Utc>,

    /// Ratchet state (encrypted for storage).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratchet_state: Option<RatchetState>,
}

impl From<&Session> for SessionMetadata {
    fn from(session: &Session) -> Self {
        Self {
            id: session.id,
            local_agent: session.local_agent,
            remote_agent: session.remote_agent,
            state: session.state,
            created_at: session.created_at,
            last_active_at: session.last_active_at,
            ratchet_state: Some(session.ratchet_state().clone()),
        }
    }
}

/// Session storage trait.
#[async_trait::async_trait]
pub trait SessionStore: Send + Sync {
    /// Store a session.
    async fn store_session(&self, session: &Session) -> CretoResult<()>;

    /// Load a session by ID.
    async fn load_session(&self, id: Uuid) -> CretoResult<Option<SessionMetadata>>;

    /// Find session for a pair of agents.
    async fn find_session(
        &self,
        local_agent: AgentId,
        remote_agent: AgentId,
    ) -> CretoResult<Option<SessionMetadata>>;

    /// List sessions for an agent.
    async fn list_sessions(&self, agent_id: AgentId) -> CretoResult<Vec<SessionMetadata>>;

    /// Delete a session.
    async fn delete_session(&self, id: Uuid) -> CretoResult<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_state_transitions() {
        // Note: This is a limited test without actual X3DH
        assert!(SessionState::Active != SessionState::Closed);
        assert!(SessionState::Establishing != SessionState::Active);
    }

    #[test]
    fn test_session_metadata() {
        let meta = SessionMetadata {
            id: Uuid::now_v7(),
            local_agent: AgentId::new(),
            remote_agent: AgentId::new(),
            state: SessionState::Active,
            created_at: Utc::now(),
            last_active_at: Utc::now(),
            ratchet_state: None,
        };

        assert_eq!(meta.state, SessionState::Active);
    }
}
