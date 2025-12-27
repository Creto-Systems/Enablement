//! Health check endpoint
//!
//! Provides a standardized health check response following Sovereign patterns.

use serde::Serialize;
use tracing::instrument;

/// Health check response
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    /// Service health status
    pub status: String,
    /// Crate version from Cargo.toml
    pub version: String,
}

impl HealthResponse {
    /// Create a new healthy response with the current crate version
    pub fn healthy() -> Self {
        Self {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Create a new healthy response with a custom version
    pub fn healthy_with_version(version: impl Into<String>) -> Self {
        Self {
            status: "healthy".to_string(),
            version: version.into(),
        }
    }
}

impl Default for HealthResponse {
    fn default() -> Self {
        Self::healthy()
    }
}

/// Health check function returning a HealthResponse
///
/// Use this in your HTTP handlers:
/// ```ignore
/// use axum::Json;
/// use creto_common::health::{health_check, HealthResponse};
///
/// async fn health_handler() -> Json<HealthResponse> {
///     Json(health_check())
/// }
/// ```
#[instrument(name = "health.check")]
pub fn health_check() -> HealthResponse {
    HealthResponse::healthy()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_healthy_response() {
        let response = HealthResponse::healthy();
        assert_eq!(response.status, "healthy");
        assert!(!response.version.is_empty());
    }

    #[test]
    fn test_healthy_with_version() {
        let response = HealthResponse::healthy_with_version("1.2.3");
        assert_eq!(response.status, "healthy");
        assert_eq!(response.version, "1.2.3");
    }

    #[test]
    fn test_health_check() {
        let response = health_check();
        assert_eq!(response.status, "healthy");
    }

    #[test]
    fn test_serialization() {
        let response = HealthResponse::healthy_with_version("0.1.0");
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains(r#""status":"healthy""#));
        assert!(json.contains(r#""version":"0.1.0""#));
    }
}
