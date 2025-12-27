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

impl CretoError {
    /// Return the error code for this error variant.
    pub fn code(&self) -> &'static str {
        match self {
            // Metering Errors (ENABLE-001 to ENABLE-004)
            Self::QuotaExceeded { .. } => "ENABLE-001",
            Self::InvalidUsageEvent(_) => "ENABLE-002",
            Self::DuplicateTransaction(_) => "ENABLE-003",
            Self::BillingPeriodNotFound(_) => "ENABLE-004",

            // Oversight Errors (ENABLE-005 to ENABLE-009)
            Self::ApprovalNotFound(_) => "ENABLE-005",
            Self::InvalidStateTransition { .. } => "ENABLE-006",
            Self::ApprovalTimeout { .. } => "ENABLE-007",
            Self::QuorumNotReached { .. } => "ENABLE-008",
            Self::UnauthorizedApprover(_) => "ENABLE-009",

            // Runtime Errors (ENABLE-010 to ENABLE-014)
            Self::SandboxNotFound(_) => "ENABLE-010",
            Self::SandboxCreationFailed(_) => "ENABLE-011",
            Self::ExecutionTimeout { .. } => "ENABLE-012",
            Self::ResourceLimitExceeded { .. } => "ENABLE-013",
            Self::NetworkEgressDenied { .. } => "ENABLE-014",

            // Messaging Errors (ENABLE-015 to ENABLE-019)
            Self::ChannelNotFound(_) => "ENABLE-015",
            Self::EncryptionFailed(_) => "ENABLE-016",
            Self::DecryptionFailed(_) => "ENABLE-017",
            Self::InvalidKeyBundle(_) => "ENABLE-018",
            Self::MessageDeliveryFailed(_) => "ENABLE-019",

            // Authorization Errors (ENABLE-020 to ENABLE-021)
            Self::AuthorizationDenied(_) => "ENABLE-020",
            Self::PolicyEvaluationFailed(_) => "ENABLE-021",

            // Infrastructure Errors (ENABLE-022 to ENABLE-024)
            Self::Database(_) => "ENABLE-022",
            Self::Configuration(_) => "ENABLE-023",
            Self::Internal(_) => "ENABLE-024",

            // Crypto Errors (ENABLE-025 to ENABLE-026)
            Self::CryptoError(_) => "ENABLE-025",
            Self::SecretResolutionFailed { .. } => "ENABLE-026",

            // Session Errors (ENABLE-027 to ENABLE-028)
            Self::SessionError(_) => "ENABLE-027",
            Self::ChannelError(_) => "ENABLE-028",

            // Additional Authorization Errors (ENABLE-029)
            Self::NotAuthorized { .. } => "ENABLE-029",

            // Serialization Errors (ENABLE-030)
            Self::SerializationError(_) => "ENABLE-030",

            // Generic Errors (ENABLE-031 to ENABLE-034)
            Self::NotFound(_) => "ENABLE-031",
            Self::Unauthorized(_) => "ENABLE-032",
            Self::LimitExceeded(_) => "ENABLE-033",
            Self::ValidationFailed(_) => "ENABLE-034",
        }
    }
}

#[cfg(feature = "sqlx")]
impl From<sqlx::Error> for CretoError {
    fn from(err: sqlx::Error) -> Self {
        CretoError::Database(err.to_string())
    }
}
