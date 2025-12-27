//! Sandbox environment for isolated agent execution.

use chrono::{DateTime, Utc};
use creto_common::{AgentId, CretoResult, OrganizationId};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::attestation::{Attestation, AttestationPolicy};
use crate::network::NetworkPolicy as DetailedNetworkPolicy;
use crate::resources::ResourceLimits;

/// Unique identifier for a sandbox instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SandboxId(Uuid);

impl SandboxId {
    /// Create a new random sandbox ID.
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Create a sandbox ID from an existing UUID.
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID.
    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for SandboxId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for SandboxId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "sandbox_{}", self.0)
    }
}

/// Configuration for creating a new sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Runtime environment (e.g., "python3.11", "node20", "deno").
    pub runtime: String,

    /// Resource limits for the sandbox.
    pub limits: ResourceLimits,

    /// Network access policy (simple enum).
    pub network_policy: NetworkPolicy,

    /// Detailed network policy for egress enforcement (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detailed_network_policy: Option<DetailedNetworkPolicy>,

    /// Filesystem mounts.
    #[serde(default)]
    pub mounts: Vec<Mount>,

    /// Environment variables.
    #[serde(default)]
    pub environment: Vec<EnvVar>,

    /// Maximum execution time in seconds.
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,

    /// Whether to enable debugging.
    #[serde(default)]
    pub debug: bool,
}

fn default_timeout() -> u32 {
    300 // 5 minutes
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            runtime: "python3.11".to_string(),
            limits: ResourceLimits::default(),
            network_policy: NetworkPolicy::Restricted,
            detailed_network_policy: None,
            mounts: Vec::new(),
            environment: Vec::new(),
            timeout_seconds: default_timeout(),
            debug: false,
        }
    }
}

/// Network access policy for sandboxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkPolicy {
    /// No network access.
    None,
    /// Only access to allowed hosts.
    Restricted,
    /// Full network access (requires elevated trust).
    Full,
}

/// Filesystem mount configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mount {
    /// Source path (host or volume).
    pub source: String,
    /// Target path inside sandbox.
    pub target: String,
    /// Whether the mount is read-only.
    #[serde(default)]
    pub read_only: bool,
}

/// Environment variable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    /// Variable name.
    pub name: String,
    /// Variable value.
    pub value: String,
    /// Whether this is a secret (should be redacted in logs).
    #[serde(default)]
    pub secret: bool,
}

/// Current state of a sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "data")]
pub enum SandboxState {
    /// Being created.
    Creating,
    /// Ready to execute.
    Ready,
    /// Currently executing code.
    Running,
    /// Paused (can be resumed).
    Paused,
    /// Stopped (needs restart).
    Stopped,
    /// Failed to create or crashed.
    Failed,
    /// Terminated and cleaned up.
    Terminated,
    /// Checkpointed with the given checkpoint ID.
    Checkpointed { checkpoint_id: String },
}

impl SandboxState {
    /// Check if the sandbox can execute code.
    pub fn can_execute(&self) -> bool {
        matches!(self, SandboxState::Ready | SandboxState::Paused)
    }

    /// Check if the sandbox is terminal (can't be used anymore).
    pub fn is_terminal(&self) -> bool {
        matches!(self, SandboxState::Failed | SandboxState::Terminated)
    }

    /// Check if the sandbox can be checkpointed.
    pub fn can_checkpoint(&self) -> bool {
        matches!(
            self,
            SandboxState::Ready | SandboxState::Paused | SandboxState::Stopped
        )
    }
}

/// A sandbox instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sandbox {
    /// Unique identifier.
    pub id: SandboxId,

    /// Organization owning this sandbox.
    pub organization_id: OrganizationId,

    /// Agent using this sandbox.
    pub agent_id: AgentId,

    /// Configuration used to create the sandbox.
    pub config: SandboxConfig,

    /// Current state.
    pub state: SandboxState,

    /// When the sandbox was created.
    pub created_at: DateTime<Utc>,

    /// When the sandbox was last used.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<DateTime<Utc>>,

    /// Internal runtime handle (opaque string).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_handle: Option<String>,

    /// Cryptographic attestation proving sandbox security.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attestation: Option<Attestation>,

    /// Policy controlling attestation requirements.
    #[serde(default)]
    pub attestation_policy: AttestationPolicy,
}

impl Sandbox {
    /// Create a new sandbox (in Creating state).
    pub fn new(organization_id: OrganizationId, agent_id: AgentId, config: SandboxConfig) -> Self {
        Self {
            id: SandboxId::new(),
            organization_id,
            agent_id,
            config,
            state: SandboxState::Creating,
            created_at: Utc::now(),
            last_used_at: None,
            runtime_handle: None,
            attestation: None,
            attestation_policy: AttestationPolicy::default(),
        }
    }

    /// Mark sandbox as ready.
    pub fn mark_ready(&mut self, handle: String) {
        self.state = SandboxState::Ready;
        self.runtime_handle = Some(handle);
    }

    /// Mark sandbox as running.
    pub fn mark_running(&mut self) {
        self.state = SandboxState::Running;
        self.last_used_at = Some(Utc::now());
    }

    /// Mark sandbox as failed.
    pub fn mark_failed(&mut self) {
        self.state = SandboxState::Failed;
    }

    /// Mark sandbox as terminated.
    pub fn mark_terminated(&mut self) {
        self.state = SandboxState::Terminated;
        self.runtime_handle = None;
    }

    /// Check if sandbox has exceeded its idle timeout.
    pub fn is_idle_expired(&self, idle_timeout_seconds: u64) -> bool {
        let last_activity = self.last_used_at.unwrap_or(self.created_at);
        let idle_duration = Utc::now().signed_duration_since(last_activity);
        idle_duration.num_seconds() as u64 > idle_timeout_seconds
    }

    /// Create a checkpoint of this sandbox.
    ///
    /// This is a placeholder method that will be implemented by the checkpoint module.
    /// In a real implementation, this would delegate to CheckpointManager.
    pub async fn checkpoint(&self) -> CretoResult<String> {
        use creto_common::CretoError;

        if !self.state.can_checkpoint() {
            return Err(CretoError::InvalidStateTransition {
                from: format!("{:?}", self.state),
                to: "checkpointed".to_string(),
            });
        }

        // Mock checkpoint creation - returns a checkpoint ID
        let checkpoint_id = Uuid::now_v7();
        Ok(format!("checkpoint_{}", checkpoint_id))
    }
}

/// Trait for sandbox backends.
#[async_trait::async_trait]
pub trait SandboxBackend: Send + Sync {
    /// Create a new sandbox.
    async fn create(&self, config: &SandboxConfig) -> CretoResult<String>;

    /// Start a sandbox.
    async fn start(&self, handle: &str) -> CretoResult<()>;

    /// Stop a sandbox.
    async fn stop(&self, handle: &str) -> CretoResult<()>;

    /// Terminate and cleanup a sandbox.
    async fn terminate(&self, handle: &str) -> CretoResult<()>;

    /// Execute code in a sandbox.
    async fn execute(&self, handle: &str, code: &str) -> CretoResult<String>;

    /// Get sandbox status.
    async fn status(&self, handle: &str) -> CretoResult<SandboxState>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = Sandbox::new(
            OrganizationId::new(),
            AgentId::new(),
            SandboxConfig::default(),
        );

        assert_eq!(sandbox.state, SandboxState::Creating);
        assert!(sandbox.runtime_handle.is_none());
    }

    #[test]
    fn test_sandbox_state_transitions() {
        let mut sandbox = Sandbox::new(
            OrganizationId::new(),
            AgentId::new(),
            SandboxConfig::default(),
        );

        sandbox.mark_ready("handle_123".to_string());
        assert_eq!(sandbox.state, SandboxState::Ready);
        assert!(sandbox.state.can_execute());

        sandbox.mark_running();
        assert_eq!(sandbox.state, SandboxState::Running);

        sandbox.mark_terminated();
        assert_eq!(sandbox.state, SandboxState::Terminated);
        assert!(sandbox.state.is_terminal());
    }

    #[test]
    fn test_network_policy() {
        let policy = NetworkPolicy::Restricted;
        assert_eq!(policy, NetworkPolicy::Restricted);
    }
}
