//! Creto Runtime - Sandboxed AI Agent Execution
//!
//! This crate provides the sandbox execution environment for AI agents,
//! implementing secure isolation, resource management, and secret injection.
//!
//! # Architecture
//!
//! The runtime follows the Agent Sandbox patterns:
//! - **Sandbox**: Isolated execution environment with resource limits
//! - **Warm Pool**: Pre-initialized sandboxes for fast cold start
//! - **Secret Injection**: Secure credential handling via runtime mounts
//! - **Lifecycle Management**: Create, execute, pause, resume, terminate
//!
//! # Example
//!
//! ```rust,ignore
//! use creto_runtime::{RuntimeService, SandboxConfig};
//!
//! let service = RuntimeService::new();
//! let sandbox = service.create_sandbox(SandboxConfig::default()).await?;
//! let result = sandbox.execute("print('Hello')").await?;
//! ```

pub mod attestation;
pub mod checkpoint;
pub mod execution;
pub mod metering;
pub mod network;
pub mod pool;
pub mod repository;
pub mod resources;
pub mod sandbox;
pub mod secrets;
pub mod service;

pub use attestation::{
    Attestation, AttestationGenerator, AttestationPlatform, AttestationPolicy, AttestationVerifier,
    MockAttestationProvider,
};
pub use checkpoint::{
    Checkpoint, CheckpointConfig, CheckpointError, CheckpointId, CheckpointManager,
    CompressionAlgorithm, InMemoryCheckpointStore,
};
pub use execution::{
    ExecutionError, ExecutionRequest, ExecutionResult, ExecutionStatus, ExecutionTiming,
};
pub use network::{
    DnsPolicy, EgressDecision, EgressDestination, EgressRule, NetworkAction, NetworkPolicy,
    NetworkPolicyEnforcer,
};
pub use pool::{PoolConfig, WarmPool};
pub use repository::{
    ExecutionRepository, PgExecutionRepository, PgResourceUsageRepository, PgSandboxRepository,
    ResourceUsageRepository, SandboxRepository,
};
pub use resources::{ResourceLimits, ResourceUsage};
pub use sandbox::{Sandbox, SandboxConfig, SandboxId, SandboxState};
pub use secrets::{SecretMount, SecretProvider};
pub use service::RuntimeService;
