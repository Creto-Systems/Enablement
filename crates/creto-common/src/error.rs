//! Error types for the Creto Enablement Layer.

use thiserror::Error;

/// Result type alias for Creto operations.
pub type CretoResult<T> = Result<T, CretoError>;

/// Unified error type for all Creto Enablement operations.
#[derive(Error, Debug)]
pub enum CretoError {
    // ─────────────────────────────────────────────────────────────────────────
    // Metering Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Quota exceeded for {resource}: used {used}, limit {limit}")]
    QuotaExceeded {
        resource: String,
        used: u64,
        limit: u64,
    },

    #[error("Invalid usage event: {0}")]
    InvalidUsageEvent(String),

    #[error("Duplicate transaction ID: {0}")]
    DuplicateTransaction(String),

    #[error("Billing period not found: {0}")]
    BillingPeriodNotFound(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Oversight Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Approval request not found: {0}")]
    ApprovalNotFound(String),

    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Approval timeout after {seconds} seconds")]
    ApprovalTimeout { seconds: u64 },

    #[error("Quorum not reached: {votes}/{required} votes")]
    QuorumNotReached { votes: u32, required: u32 },

    #[error("Unauthorized approver: {0}")]
    UnauthorizedApprover(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Runtime Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Sandbox not found: {0}")]
    SandboxNotFound(String),

    #[error("Sandbox creation failed: {0}")]
    SandboxCreationFailed(String),

    #[error("Execution timeout after {seconds} seconds")]
    ExecutionTimeout { seconds: u64 },

    #[error("Resource limit exceeded: {resource}")]
    ResourceLimitExceeded { resource: String },

    #[error("Network egress denied to: {destination}")]
    NetworkEgressDenied { destination: String },

    // ─────────────────────────────────────────────────────────────────────────
    // Messaging Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid key bundle: {0}")]
    InvalidKeyBundle(String),

    #[error("Message delivery failed: {0}")]
    MessageDeliveryFailed(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Authorization Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),

    #[error("Policy evaluation failed: {0}")]
    PolicyEvaluationFailed(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Infrastructure Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Database error: {0}")]
    Database(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal error: {0}")]
    Internal(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Crypto Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Cryptography error: {0}")]
    CryptoError(String),

    #[error("Secret resolution failed for {secret_name}")]
    SecretResolutionFailed {
        secret_name: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    // ─────────────────────────────────────────────────────────────────────────
    // Session Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Session error: {0}")]
    SessionError(String),

    #[error("Channel error: {0}")]
    ChannelError(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Authorization Errors (additional)
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Not authorized to {action} on {resource}")]
    NotAuthorized { resource: String, action: String },

    // ─────────────────────────────────────────────────────────────────────────
    // Serialization Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Serialization error: {0}")]
    SerializationError(String),

    // ─────────────────────────────────────────────────────────────────────────
    // Generic Errors
    // ─────────────────────────────────────────────────────────────────────────
    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Limit exceeded: {0}")]
    LimitExceeded(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),
}

#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for CretoError {
    fn from(err: sqlx::Error) -> Self {
        CretoError::Database(err.to_string())
    }
}
