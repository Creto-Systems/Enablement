//! # creto-oversight
//!
//! Human-in-the-loop approval workflows for AI agents.
//!
//! ## Overview
//!
//! This crate provides the oversight infrastructure for the Creto Enablement Layer,
//! enabling organizations to:
//!
//! - **Request Approvals**: Agents submit actions for human review
//! - **Define Policies**: Cedar policies determine what requires oversight
//! - **Route Reviews**: Match requests to appropriate reviewers
//! - **Track Decisions**: Immutable audit trail of all approvals
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                      creto-oversight                            │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
//! │  │   Request    │  │    State     │  │   Channel    │          │
//! │  │   Manager    │→ │   Machine    │→ │   Adapters   │          │
//! │  └──────────────┘  └──────────────┘  └──────────────┘          │
//! │         ↓                  ↓                  ↓                 │
//! │  ┌──────────────────────────────────────────────────┐          │
//! │  │           Cedar Policy Engine + Audit            │          │
//! │  └──────────────────────────────────────────────────┘          │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Pattern Source
//!
//! Inspired by [HumanLayer](https://github.com/humanlayer/humanlayer) patterns,
//! rebuilt with Creto Sovereign primitives (NHI, Cedar authorization, audit logging).

pub mod approval;
pub mod channels;
pub mod checkpoint;
pub mod context;
pub mod metering;
pub mod policy;
pub mod repository;
pub mod request;
pub mod service;
pub mod state;
pub mod triggers;

pub use approval::{Approval, ApprovalDecision, QuorumConfig, QuorumCalculator, QuorumResult};
pub use checkpoint::{Checkpoint, CheckpointManager, CheckpointRepository, CHECKPOINT_VERSION};
pub use repository::{
    RequestRepository, PgRequestRepository,
    ApprovalRepository, PgApprovalRepository, ApprovalCounts,
    StateTransitionRepository, PgStateTransitionRepository, StateTransitionRecord,
    QuorumConfigRepository, PgQuorumConfigRepository, QuorumConfigRecord,
    PgCheckpointRepository,
};
pub use request::{OversightRequest, RequestStatus, ActionType, Priority};
pub use service::OversightService;
pub use state::{StateMachine, StateTransition};
pub use policy::{PolicyContext, PolicyDecision, TrustLevel};
pub use triggers::{
    PolicyTriggerConfig, TriggerCondition, PolicyEvaluator, TriggerMatch,
    ActionTypePattern, TrustLevelThreshold, MockCedarClient,
};
