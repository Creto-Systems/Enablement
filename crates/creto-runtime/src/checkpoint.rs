//! Checkpoint and restore functionality for sandbox migration.
//!
//! This module provides the ability to checkpoint a running sandbox's state
//! and restore it later, potentially on a different host. This enables:
//! - Live migration of sandboxes across hosts
//! - Disaster recovery and fault tolerance
//! - Development workflow snapshots
//! - Cost optimization through sandbox parking

use chrono::{DateTime, Utc};
use creto_common::{AgentId, CretoError, CretoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

use crate::sandbox::SandboxId;

/// Unique identifier for a checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckpointId(Uuid);

impl CheckpointId {
    /// Create a new random checkpoint ID.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Create a checkpoint ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for CheckpointId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CheckpointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "checkpoint_{}", self.0)
    }
}

/// A checkpoint of a sandbox's state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Unique checkpoint identifier.
    pub id: CheckpointId,

    /// ID of the sandbox that was checkpointed.
    pub sandbox_id: SandboxId,

    /// Agent that owns the sandbox.
    pub agent_id: AgentId,

    /// Serialized state snapshot (opaque blob).
    pub state_snapshot: Vec<u8>,

    /// Hash of filesystem contents (for integrity verification).
    pub filesystem_hash: String,

    /// Memory size in bytes at time of checkpoint.
    pub memory_size: u64,

    /// When the checkpoint was created.
    pub created_at: DateTime<Utc>,

    /// Optional metadata for the checkpoint.
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Compression algorithm used (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression: Option<CompressionAlgorithm>,
}

impl Checkpoint {
    /// Create a new checkpoint.
    pub fn new(
        sandbox_id: SandboxId,
        agent_id: AgentId,
        state_snapshot: Vec<u8>,
        filesystem_hash: String,
        memory_size: u64,
    ) -> Self {
        Self {
            id: CheckpointId::new(),
            sandbox_id,
            agent_id,
            state_snapshot,
            filesystem_hash,
            memory_size,
            created_at: Utc::now(),
            metadata: HashMap::new(),
            compression: None,
        }
    }

    /// Get the size of the checkpoint in bytes.
    pub fn size_bytes(&self) -> u64 {
        self.state_snapshot.len() as u64
    }

    /// Check if the checkpoint is compressed.
    pub fn is_compressed(&self) -> bool {
        self.compression.is_some()
    }

    /// Add metadata to the checkpoint.
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set compression algorithm.
    pub fn with_compression(mut self, compression: CompressionAlgorithm) -> Self {
        self.compression = Some(compression);
        self
    }
}

/// Compression algorithms supported for checkpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum CompressionAlgorithm {
    /// No compression.
    #[default]
    None,
    /// Gzip compression.
    Gzip,
    /// Zstandard compression.
    Zstd,
    /// LZ4 compression (fast).
    Lz4,
}

/// Configuration for creating checkpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Compression algorithm to use.
    #[serde(default)]
    pub compression: CompressionAlgorithm,

    /// Whether to include memory contents in checkpoint.
    #[serde(default = "default_true")]
    pub include_memory: bool,

    /// Whether to include filesystem contents in checkpoint.
    #[serde(default = "default_true")]
    pub include_filesystem: bool,

    /// Additional metadata to attach to the checkpoint.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

impl Default for CheckpointConfig {
    fn default() -> Self {
        Self {
            compression: CompressionAlgorithm::None,
            include_memory: true,
            include_filesystem: true,
            metadata: HashMap::new(),
        }
    }
}

/// Errors that can occur during checkpoint operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CheckpointError {
    /// Checkpoint not found.
    NotFound { checkpoint_id: CheckpointId },
    /// Sandbox is not in a valid state for checkpointing.
    InvalidSandboxState {
        sandbox_id: SandboxId,
        current_state: String,
    },
    /// Checkpoint creation failed.
    CreationFailed {
        sandbox_id: SandboxId,
        reason: String,
    },
    /// Checkpoint restoration failed.
    RestoreFailed {
        checkpoint_id: CheckpointId,
        reason: String,
    },
    /// Filesystem hash mismatch (corruption detected).
    IntegrityCheckFailed {
        checkpoint_id: CheckpointId,
        expected: String,
        actual: String,
    },
    /// Compression/decompression error.
    CompressionError {
        algorithm: CompressionAlgorithm,
        message: String,
    },
    /// Storage backend error.
    StorageError { message: String },
}

impl std::fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound { checkpoint_id } => {
                write!(f, "Checkpoint {} not found", checkpoint_id)
            }
            Self::InvalidSandboxState {
                sandbox_id,
                current_state,
            } => {
                write!(
                    f,
                    "Sandbox {} is in state '{}' and cannot be checkpointed",
                    sandbox_id, current_state
                )
            }
            Self::CreationFailed { sandbox_id, reason } => {
                write!(
                    f,
                    "Failed to create checkpoint for sandbox {}: {}",
                    sandbox_id, reason
                )
            }
            Self::RestoreFailed {
                checkpoint_id,
                reason,
            } => {
                write!(
                    f,
                    "Failed to restore checkpoint {}: {}",
                    checkpoint_id, reason
                )
            }
            Self::IntegrityCheckFailed {
                checkpoint_id,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Integrity check failed for checkpoint {}: expected hash {}, got {}",
                    checkpoint_id, expected, actual
                )
            }
            Self::CompressionError { algorithm, message } => {
                write!(f, "Compression error ({:?}): {}", algorithm, message)
            }
            Self::StorageError { message } => {
                write!(f, "Storage error: {}", message)
            }
        }
    }
}

impl std::error::Error for CheckpointError {}

impl CheckpointError {
    /// Return the error code for this error variant.
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound { .. } => "ENABLE-500",
            Self::InvalidSandboxState { .. } => "ENABLE-501",
            Self::CreationFailed { .. } => "ENABLE-502",
            Self::RestoreFailed { .. } => "ENABLE-503",
            Self::IntegrityCheckFailed { .. } => "ENABLE-504",
            Self::CompressionError { .. } => "ENABLE-505",
            Self::StorageError { .. } => "ENABLE-506",
        }
    }
}

impl From<CheckpointError> for CretoError {
    fn from(err: CheckpointError) -> Self {
        CretoError::Internal(err.to_string())
    }
}

/// Manager for checkpoint operations.
#[async_trait::async_trait]
pub trait CheckpointManager: Send + Sync {
    /// Create a checkpoint of a sandbox.
    async fn checkpoint(
        &self,
        sandbox_id: SandboxId,
        config: CheckpointConfig,
    ) -> CretoResult<CheckpointId>;

    /// Restore a sandbox from a checkpoint.
    async fn restore(&self, checkpoint_id: CheckpointId) -> CretoResult<SandboxId>;

    /// List all checkpoints, optionally filtered by sandbox or agent.
    async fn list_checkpoints(
        &self,
        sandbox_id: Option<SandboxId>,
        agent_id: Option<AgentId>,
    ) -> CretoResult<Vec<Checkpoint>>;

    /// Delete a checkpoint.
    async fn delete_checkpoint(&self, checkpoint_id: CheckpointId) -> CretoResult<()>;

    /// Get checkpoint metadata.
    async fn get_checkpoint(&self, checkpoint_id: CheckpointId) -> CretoResult<Checkpoint>;
}

/// In-memory checkpoint store for testing and development.
#[derive(Debug, Default)]
pub struct InMemoryCheckpointStore {
    checkpoints: Arc<RwLock<HashMap<CheckpointId, Checkpoint>>>,
}

impl InMemoryCheckpointStore {
    /// Create a new in-memory checkpoint store.
    pub fn new() -> Self {
        Self {
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the number of stored checkpoints.
    pub fn count(&self) -> usize {
        self.checkpoints.read().unwrap().len()
    }

    /// Clear all checkpoints.
    pub fn clear(&self) {
        self.checkpoints.write().unwrap().clear();
    }
}

#[async_trait::async_trait]
impl CheckpointManager for InMemoryCheckpointStore {
    async fn checkpoint(
        &self,
        sandbox_id: SandboxId,
        config: CheckpointConfig,
    ) -> CretoResult<CheckpointId> {
        // Mock checkpoint creation
        let checkpoint = Checkpoint {
            id: CheckpointId::new(),
            sandbox_id,
            agent_id: AgentId::new(),        // Mock agent ID
            state_snapshot: vec![0u8; 1024], // Mock 1KB snapshot
            filesystem_hash: "mock_hash_123".to_string(),
            memory_size: 1024 * 1024 * 128, // Mock 128MB
            created_at: Utc::now(),
            metadata: config.metadata,
            compression: Some(config.compression),
        };

        let checkpoint_id = checkpoint.id;
        self.checkpoints
            .write()
            .unwrap()
            .insert(checkpoint_id, checkpoint);

        tracing::debug!(
            checkpoint_id = %checkpoint_id,
            sandbox_id = %sandbox_id,
            "Checkpoint created"
        );

        Ok(checkpoint_id)
    }

    async fn restore(&self, checkpoint_id: CheckpointId) -> CretoResult<SandboxId> {
        let checkpoints = self.checkpoints.read().unwrap();
        let checkpoint = checkpoints
            .get(&checkpoint_id)
            .ok_or(CheckpointError::NotFound { checkpoint_id })?;

        tracing::debug!(
            checkpoint_id = %checkpoint_id,
            sandbox_id = %checkpoint.sandbox_id,
            "Checkpoint restored"
        );

        // Return the original sandbox ID (in a real implementation,
        // this might be a new sandbox ID)
        Ok(checkpoint.sandbox_id)
    }

    async fn list_checkpoints(
        &self,
        sandbox_id: Option<SandboxId>,
        agent_id: Option<AgentId>,
    ) -> CretoResult<Vec<Checkpoint>> {
        let checkpoints = self.checkpoints.read().unwrap();
        let mut result: Vec<Checkpoint> = checkpoints
            .values()
            .filter(|cp| {
                if let Some(sid) = sandbox_id {
                    if cp.sandbox_id != sid {
                        return false;
                    }
                }
                if let Some(aid) = agent_id {
                    if cp.agent_id != aid {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        // Sort by creation time (newest first)
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(result)
    }

    async fn delete_checkpoint(&self, checkpoint_id: CheckpointId) -> CretoResult<()> {
        let mut checkpoints = self.checkpoints.write().unwrap();
        checkpoints
            .remove(&checkpoint_id)
            .ok_or(CheckpointError::NotFound { checkpoint_id })?;

        tracing::debug!(checkpoint_id = %checkpoint_id, "Checkpoint deleted");
        Ok(())
    }

    async fn get_checkpoint(&self, checkpoint_id: CheckpointId) -> CretoResult<Checkpoint> {
        let checkpoints = self.checkpoints.read().unwrap();
        checkpoints
            .get(&checkpoint_id)
            .cloned()
            .ok_or_else(|| CheckpointError::NotFound { checkpoint_id }.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sandbox::SandboxId;

    #[test]
    fn test_checkpoint_id_creation() {
        let id1 = CheckpointId::new();
        let id2 = CheckpointId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_checkpoint_creation() {
        let sandbox_id = SandboxId::new();
        let agent_id = AgentId::new();
        let checkpoint = Checkpoint::new(
            sandbox_id,
            agent_id,
            vec![1, 2, 3, 4],
            "hash123".to_string(),
            1024,
        );

        assert_eq!(checkpoint.sandbox_id, sandbox_id);
        assert_eq!(checkpoint.agent_id, agent_id);
        assert_eq!(checkpoint.memory_size, 1024);
        assert_eq!(checkpoint.size_bytes(), 4);
        assert!(!checkpoint.is_compressed());
    }

    #[test]
    fn test_checkpoint_with_metadata() {
        let checkpoint = Checkpoint::new(
            SandboxId::new(),
            AgentId::new(),
            vec![],
            "hash".to_string(),
            0,
        )
        .with_metadata("key".to_string(), "value".to_string())
        .with_compression(CompressionAlgorithm::Gzip);

        assert_eq!(checkpoint.metadata.get("key"), Some(&"value".to_string()));
        assert_eq!(checkpoint.compression, Some(CompressionAlgorithm::Gzip));
        assert!(checkpoint.is_compressed());
    }

    #[tokio::test]
    async fn test_in_memory_store_checkpoint_creation() {
        let store = InMemoryCheckpointStore::new();
        let sandbox_id = SandboxId::new();
        let config = CheckpointConfig::default();

        let checkpoint_id = store.checkpoint(sandbox_id, config).await.unwrap();
        assert_eq!(store.count(), 1);

        let checkpoint = store.get_checkpoint(checkpoint_id).await.unwrap();
        assert_eq!(checkpoint.sandbox_id, sandbox_id);
    }

    #[tokio::test]
    async fn test_in_memory_store_restore() {
        let store = InMemoryCheckpointStore::new();
        let sandbox_id = SandboxId::new();

        let checkpoint_id = store
            .checkpoint(sandbox_id, CheckpointConfig::default())
            .await
            .unwrap();

        let restored_sandbox_id = store.restore(checkpoint_id).await.unwrap();
        assert_eq!(restored_sandbox_id, sandbox_id);
    }

    #[tokio::test]
    async fn test_in_memory_store_list_checkpoints() {
        let store = InMemoryCheckpointStore::new();
        let sandbox_id1 = SandboxId::new();
        let sandbox_id2 = SandboxId::new();

        store
            .checkpoint(sandbox_id1, CheckpointConfig::default())
            .await
            .unwrap();
        store
            .checkpoint(sandbox_id2, CheckpointConfig::default())
            .await
            .unwrap();
        store
            .checkpoint(sandbox_id1, CheckpointConfig::default())
            .await
            .unwrap();

        let all = store.list_checkpoints(None, None).await.unwrap();
        assert_eq!(all.len(), 3);

        let for_sandbox1 = store
            .list_checkpoints(Some(sandbox_id1), None)
            .await
            .unwrap();
        assert_eq!(for_sandbox1.len(), 2);
    }

    #[tokio::test]
    async fn test_in_memory_store_delete() {
        let store = InMemoryCheckpointStore::new();
        let checkpoint_id = store
            .checkpoint(SandboxId::new(), CheckpointConfig::default())
            .await
            .unwrap();

        assert_eq!(store.count(), 1);

        store.delete_checkpoint(checkpoint_id).await.unwrap();
        assert_eq!(store.count(), 0);

        // Deleting again should fail
        let result = store.delete_checkpoint(checkpoint_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_checkpoint_not_found_error() {
        let store = InMemoryCheckpointStore::new();
        let fake_id = CheckpointId::new();

        let result = store.restore(fake_id).await;
        assert!(result.is_err());

        let result = store.get_checkpoint(fake_id).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_checkpoint_config_defaults() {
        let config = CheckpointConfig::default();
        assert_eq!(config.compression, CompressionAlgorithm::None);
        assert!(config.include_memory);
        assert!(config.include_filesystem);
        assert!(config.metadata.is_empty());
    }

    #[test]
    fn test_checkpoint_error_display() {
        let err = CheckpointError::NotFound {
            checkpoint_id: CheckpointId::new(),
        };
        assert!(err.to_string().contains("not found"));

        let err = CheckpointError::InvalidSandboxState {
            sandbox_id: SandboxId::new(),
            current_state: "Running".to_string(),
        };
        assert!(err.to_string().contains("cannot be checkpointed"));
    }
}
