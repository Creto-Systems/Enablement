//! Cryptographic key types for secure messaging.

use creto_common::AgentId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An identity key pair for an agent.
///
/// This is the long-term key that identifies an agent.
#[derive(Clone, Serialize, Deserialize)]
pub struct IdentityKey {
    /// Key identifier.
    pub id: Uuid,

    /// Agent this key belongs to.
    pub agent_id: AgentId,

    /// Public key bytes.
    pub public_key: Vec<u8>,

    /// Private key bytes (only present for own keys).
    #[serde(skip_serializing)]
    pub private_key: Option<Vec<u8>>,
}

impl std::fmt::Debug for IdentityKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdentityKey")
            .field("id", &self.id)
            .field("agent_id", &self.agent_id)
            .field("public_key_len", &self.public_key.len())
            .field("has_private_key", &self.private_key.is_some())
            .finish()
    }
}

impl IdentityKey {
    /// Create a new identity key pair.
    pub fn generate(agent_id: AgentId) -> Self {
        // TODO: Use creto-crypto for key generation
        // For now, generate placeholder keys
        Self {
            id: Uuid::now_v7(),
            agent_id,
            public_key: vec![0u8; 32], // Placeholder
            private_key: Some(vec![0u8; 32]), // Placeholder
        }
    }

    /// Create from existing public key (no private key).
    pub fn from_public(agent_id: AgentId, public_key: Vec<u8>) -> Self {
        Self {
            id: Uuid::now_v7(),
            agent_id,
            public_key,
            private_key: None,
        }
    }

    /// Check if this key has the private component.
    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
    }
}

/// A one-time pre-key.
///
/// Pre-keys are uploaded in batches and used once for initial key agreement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreKey {
    /// Key identifier.
    pub id: u32,

    /// Public key bytes.
    pub public_key: Vec<u8>,

    /// Private key bytes (only present for own keys).
    #[serde(skip_serializing)]
    pub private_key: Option<Vec<u8>>,
}

impl PreKey {
    /// Generate a new pre-key.
    pub fn generate(id: u32) -> Self {
        // TODO: Use creto-crypto
        Self {
            id,
            public_key: vec![0u8; 32], // Placeholder
            private_key: Some(vec![0u8; 32]), // Placeholder
        }
    }

    /// Generate a batch of pre-keys.
    pub fn generate_batch(start_id: u32, count: u32) -> Vec<Self> {
        (start_id..start_id + count)
            .map(PreKey::generate)
            .collect()
    }
}

/// A signed pre-key.
///
/// Signed pre-keys are medium-term keys that provide deniability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedPreKey {
    /// Key identifier.
    pub id: u32,

    /// Public key bytes.
    pub public_key: Vec<u8>,

    /// Signature over the public key (by identity key).
    pub signature: Vec<u8>,

    /// When this key was generated.
    pub timestamp: i64,

    /// Private key bytes (only present for own keys).
    #[serde(skip_serializing)]
    pub private_key: Option<Vec<u8>>,
}

impl SignedPreKey {
    /// Generate a new signed pre-key.
    ///
    /// In production, this should sign with the identity key.
    pub fn generate(id: u32, _identity_key: &IdentityKey) -> Self {
        // TODO: Use creto-crypto for signing
        Self {
            id,
            public_key: vec![0u8; 32], // Placeholder
            signature: vec![0u8; 64],  // Placeholder
            timestamp: chrono::Utc::now().timestamp(),
            private_key: Some(vec![0u8; 32]), // Placeholder
        }
    }

    /// Verify the signature on this key.
    pub fn verify(&self, _identity_key: &IdentityKey) -> bool {
        // TODO: Implement signature verification
        true
    }
}

/// A complete key bundle for establishing sessions.
///
/// Published to the key server for other agents to initiate sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyBundle {
    /// Agent identity.
    pub agent_id: AgentId,

    /// Identity key (long-term).
    pub identity_key: IdentityKey,

    /// Current signed pre-key (medium-term).
    pub signed_pre_key: SignedPreKey,

    /// One-time pre-key (optional, for forward secrecy).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one_time_pre_key: Option<PreKey>,
}

impl KeyBundle {
    /// Create a new key bundle.
    pub fn new(agent_id: AgentId) -> Self {
        let identity_key = IdentityKey::generate(agent_id);
        let signed_pre_key = SignedPreKey::generate(1, &identity_key);
        let one_time_pre_key = Some(PreKey::generate(1));

        Self {
            agent_id,
            identity_key,
            signed_pre_key,
            one_time_pre_key,
        }
    }

    /// Create a public-only bundle (for sending to others).
    pub fn public_bundle(&self) -> Self {
        Self {
            agent_id: self.agent_id,
            identity_key: IdentityKey::from_public(
                self.agent_id,
                self.identity_key.public_key.clone(),
            ),
            signed_pre_key: SignedPreKey {
                id: self.signed_pre_key.id,
                public_key: self.signed_pre_key.public_key.clone(),
                signature: self.signed_pre_key.signature.clone(),
                timestamp: self.signed_pre_key.timestamp,
                private_key: None,
            },
            one_time_pre_key: self.one_time_pre_key.as_ref().map(|pk| PreKey {
                id: pk.id,
                public_key: pk.public_key.clone(),
                private_key: None,
            }),
        }
    }
}

/// Key storage trait.
#[async_trait::async_trait]
pub trait KeyStore: Send + Sync {
    /// Store an identity key.
    async fn store_identity_key(
        &self,
        key: &IdentityKey,
    ) -> creto_common::CretoResult<()>;

    /// Get identity key for an agent.
    async fn get_identity_key(
        &self,
        agent_id: AgentId,
    ) -> creto_common::CretoResult<Option<IdentityKey>>;

    /// Store a key bundle.
    async fn store_bundle(
        &self,
        bundle: &KeyBundle,
    ) -> creto_common::CretoResult<()>;

    /// Get a key bundle for an agent.
    async fn get_bundle(
        &self,
        agent_id: AgentId,
    ) -> creto_common::CretoResult<Option<KeyBundle>>;

    /// Consume a one-time pre-key (returns and removes it).
    async fn consume_pre_key(
        &self,
        agent_id: AgentId,
    ) -> creto_common::CretoResult<Option<PreKey>>;

    /// Upload new pre-keys.
    async fn upload_pre_keys(
        &self,
        agent_id: AgentId,
        keys: Vec<PreKey>,
    ) -> creto_common::CretoResult<()>;

    /// Get the count of remaining pre-keys.
    async fn pre_key_count(
        &self,
        agent_id: AgentId,
    ) -> creto_common::CretoResult<u32>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_key_generation() {
        let agent_id = AgentId::new();
        let key = IdentityKey::generate(agent_id);

        assert!(key.has_private_key());
        assert_eq!(key.agent_id, agent_id);
    }

    #[test]
    fn test_prekey_batch_generation() {
        let keys = PreKey::generate_batch(100, 10);
        assert_eq!(keys.len(), 10);
        assert_eq!(keys[0].id, 100);
        assert_eq!(keys[9].id, 109);
    }

    #[test]
    fn test_key_bundle() {
        let agent_id = AgentId::new();
        let bundle = KeyBundle::new(agent_id);

        assert_eq!(bundle.agent_id, agent_id);
        assert!(bundle.one_time_pre_key.is_some());

        let public = bundle.public_bundle();
        assert!(!public.identity_key.has_private_key());
    }
}
