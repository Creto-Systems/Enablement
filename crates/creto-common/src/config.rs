//! Configuration loading utilities.
//!
//! This module provides layered configuration loading using figment:
//! - Default values (compiled in)
//! - Configuration file (TOML)
//! - Environment variables
//!
//! # Example
//!
//! ```rust,ignore
//! use creto_common::config::{DatabaseConfig, load_config};
//!
//! // Load configuration from defaults + file + env
//! let config: DatabaseConfig = load_config("creto", None)?;
//! ```

#[cfg(feature = "config")]
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

/// Database connection configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// PostgreSQL connection URL.
    #[serde(default = "default_database_url")]
    pub url: String,

    /// Maximum connections in the pool.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum connections to maintain.
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Connection timeout in seconds.
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout_secs: u64,

    /// Idle connection timeout in seconds.
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout_secs: u64,
}

fn default_database_url() -> String {
    "postgres://localhost/creto_enablement".to_string()
}

fn default_max_connections() -> u32 {
    10
}

fn default_min_connections() -> u32 {
    1
}

fn default_connect_timeout() -> u64 {
    30
}

fn default_idle_timeout() -> u64 {
    600
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: default_database_url(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connect_timeout_secs: default_connect_timeout(),
            idle_timeout_secs: default_idle_timeout(),
        }
    }
}

/// Redis connection configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    /// Redis connection URL.
    #[serde(default = "default_redis_url")]
    pub url: String,

    /// Connection pool size.
    #[serde(default = "default_redis_pool_size")]
    pub pool_size: usize,
}

fn default_redis_url() -> String {
    "redis://localhost:6379".to_string()
}

fn default_redis_pool_size() -> usize {
    10
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: default_redis_url(),
            pool_size: default_redis_pool_size(),
        }
    }
}

/// Observability configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ObservabilityConfig {
    /// Log level (trace, debug, info, warn, error).
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Enable JSON logging.
    #[serde(default)]
    pub json_logs: bool,

    /// OpenTelemetry endpoint (optional).
    pub otlp_endpoint: Option<String>,

    /// Service name for tracing.
    #[serde(default = "default_service_name")]
    pub service_name: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_service_name() -> String {
    "creto-enablement".to_string()
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            log_level: default_log_level(),
            json_logs: false,
            otlp_endpoint: None,
            service_name: default_service_name(),
        }
    }
}

/// Metering service configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MeteringConfig {
    /// Batch size for event ingestion.
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,

    /// Flush interval in milliseconds.
    #[serde(default = "default_flush_interval_ms")]
    pub flush_interval_ms: u64,

    /// Enable Redis caching for quotas.
    #[serde(default = "default_redis_cache_enabled")]
    pub redis_cache_enabled: bool,
}

fn default_batch_size() -> usize {
    100
}

fn default_flush_interval_ms() -> u64 {
    1000
}

fn default_redis_cache_enabled() -> bool {
    true
}

/// Oversight service configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OversightConfig {
    /// Default timeout for approval requests in seconds.
    #[serde(default = "default_timeout_secs")]
    pub default_timeout_secs: u64,

    /// Default required approvals.
    #[serde(default = "default_required_approvals")]
    pub default_required_approvals: u32,
}

fn default_timeout_secs() -> u64 {
    86400 // 24 hours
}

fn default_required_approvals() -> u32 {
    1
}

/// Runtime service configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RuntimeConfig {
    /// Maximum concurrent sandboxes per organization.
    #[serde(default = "default_max_sandboxes")]
    pub max_sandboxes_per_org: usize,

    /// Warm pool size.
    #[serde(default = "default_warm_pool_size")]
    pub warm_pool_size: usize,

    /// Default sandbox timeout in seconds.
    #[serde(default = "default_sandbox_timeout")]
    pub default_timeout_secs: u64,
}

fn default_max_sandboxes() -> usize {
    100
}

fn default_warm_pool_size() -> usize {
    5
}

fn default_sandbox_timeout() -> u64 {
    300
}

/// Messaging service configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct MessagingConfig {
    /// Maximum message size in bytes.
    #[serde(default = "default_max_message_size")]
    pub max_message_size: usize,

    /// Message TTL in seconds.
    #[serde(default = "default_message_ttl")]
    pub message_ttl_secs: u64,

    /// Prekey replenishment threshold.
    #[serde(default = "default_prekey_threshold")]
    pub prekey_threshold: usize,
}

fn default_max_message_size() -> usize {
    1024 * 1024 // 1MB
}

fn default_message_ttl() -> u64 {
    604800 // 7 days
}

fn default_prekey_threshold() -> usize {
    10
}

/// Complete enablement configuration.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct EnablementConfig {
    /// Database configuration.
    #[serde(default)]
    pub database: DatabaseConfig,

    /// Redis configuration.
    #[serde(default)]
    pub redis: RedisConfig,

    /// Observability configuration.
    #[serde(default)]
    pub observability: ObservabilityConfig,

    /// Metering service configuration.
    #[serde(default)]
    pub metering: MeteringConfig,

    /// Oversight service configuration.
    #[serde(default)]
    pub oversight: OversightConfig,

    /// Runtime service configuration.
    #[serde(default)]
    pub runtime: RuntimeConfig,

    /// Messaging service configuration.
    #[serde(default)]
    pub messaging: MessagingConfig,
}

/// Load configuration from layered sources.
///
/// Sources (in order of precedence):
/// 1. Defaults (lowest)
/// 2. Configuration file (if provided)
/// 3. Environment variables (highest)
///
/// Environment variables are prefixed with the uppercase `prefix` followed by `_`.
/// For example, with prefix "creto", `CRETO_DATABASE_URL` sets `database.url`.
#[cfg(feature = "config")]
pub fn load_config<T: Default + Serialize + for<'de> Deserialize<'de>>(
    prefix: &str,
    config_file: Option<&str>,
) -> Result<T, figment::Error> {
    let mut figment = Figment::new().merge(Serialized::defaults(T::default()));

    if let Some(path) = config_file {
        figment = figment.merge(Toml::file(path));
    }

    figment = figment.merge(Env::prefixed(&format!("{}_", prefix.to_uppercase())).split("_"));

    figment.extract()
}

/// Load enablement configuration from defaults, file, and environment.
#[cfg(feature = "config")]
pub fn load_enablement_config(
    config_file: Option<&str>,
) -> Result<EnablementConfig, figment::Error> {
    load_config("creto", config_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_defaults() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
    }

    #[test]
    fn test_enablement_config_defaults() {
        let config = EnablementConfig::default();
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.metering.batch_size, 100);
        assert_eq!(config.oversight.default_required_approvals, 1);
        assert_eq!(config.runtime.warm_pool_size, 5);
    }

    #[cfg(feature = "config")]
    #[test]
    fn test_load_config_defaults() {
        let config: EnablementConfig = load_config("test", None).unwrap();
        assert_eq!(config.database.max_connections, 10);
    }
}
