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

pub mod sandbox;
pub mod execution;
pub mod metering;
pub mod network;
pub mod pool;
pub mod repository;
pub mod secrets;
pub mod resources;
pub mod service;
pub mod attestation;
pub mod checkpoint;

pub use sandbox::{Sandbox, SandboxConfig, SandboxState, SandboxId};
pub use execution::{ExecutionRequest, ExecutionResult, ExecutionStatus, ExecutionTiming, ExecutionError};
pub use network::{
    NetworkPolicy, EgressRule, EgressDestination, NetworkAction,
    DnsPolicy, NetworkPolicyEnforcer, EgressDecision,
};
pub use pool::{WarmPool, PoolConfig};
pub use repository::{
    SandboxRepository, PgSandboxRepository,
    ExecutionRepository, PgExecutionRepository,
    ResourceUsageRepository, PgResourceUsageRepository,
};
pub use secrets::{SecretMount, SecretProvider};
pub use resources::{ResourceLimits, ResourceUsage};
pub use service::RuntimeService;
pub use attestation::{
    Attestation, AttestationPlatform, AttestationPolicy,
    AttestationGenerator, AttestationVerifier, MockAttestationProvider,
};
pub use checkpoint::{
    Checkpoint, CheckpointConfig, CheckpointId, CheckpointManager,
    CheckpointError, InMemoryCheckpointStore, CompressionAlgorithm,
};
