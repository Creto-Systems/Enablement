//! # creto-common
//!
//! Shared types and utilities for the Creto Enablement Layer.
//!
//! This crate provides foundational types used across all Enablement products:
//! - `creto-metering`: Usage-based billing and quota enforcement
//! - `creto-oversight`: Human-in-the-loop approval workflows
//! - `creto-runtime`: Sandboxed agent execution
//! - `creto-messaging`: Secure agent-to-agent communication

pub mod error;
pub mod identity;
pub mod types;

#[cfg(feature = "config")]
pub mod config;

pub use error::{CretoError, CretoResult};
pub use identity::{AgentId, OrganizationId, UserId};
pub use types::{Money, Timestamp};

#[cfg(feature = "config")]
pub use config::{
    DatabaseConfig, RedisConfig, ObservabilityConfig,
    MeteringConfig, OversightConfig, RuntimeConfig, MessagingConfig,
    EnablementConfig, load_config, load_enablement_config,
};
