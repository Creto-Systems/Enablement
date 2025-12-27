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
pub mod health;
pub mod identity;
pub mod types;

#[cfg(feature = "config")]
pub mod config;

pub use error::{CretoError, CretoResult};
pub use health::{health_check, HealthResponse};
pub use identity::{AgentId, OrganizationId, UserId};
pub use types::{Money, Timestamp};

#[cfg(feature = "config")]
pub use config::{
    load_config, load_enablement_config, DatabaseConfig, EnablementConfig, MessagingConfig,
    MeteringConfig, ObservabilityConfig, OversightConfig, RedisConfig, RuntimeConfig,
};
