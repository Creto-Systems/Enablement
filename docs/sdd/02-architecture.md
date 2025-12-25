---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
---

# SDD-02: System Architecture

## Purpose

This document defines the system architecture for the Enablement Layer, including component design, interaction patterns, and integration points with Platform and Security layers.

## Scope

**In Scope:**
- Crate structure and dependencies
- Inter-crate communication patterns
- External layer integration
- Data flow diagrams
- Error handling strategy

**Out of Scope:**
- Internal implementation of individual crates (see product SDDs)
- Platform/Security layer internals

---

## 1. Crate Architecture

```
creto-enablement/
├── crates/
│   ├── creto-enablement-common/    # Shared types, errors, config
│   │   └── depends on: creto-nhi, creto-crypto
│   │
│   ├── creto-metering/             # Usage billing & quota
│   │   └── depends on: common, creto-authz, creto-audit
│   │
│   ├── creto-oversight/            # Human-in-the-loop
│   │   └── depends on: common, creto-authz, creto-memory, creto-audit
│   │
│   ├── creto-runtime/              # Sandboxed execution
│   │   └── depends on: common, creto-authz, creto-audit
│   │
│   └── creto-messaging/            # Secure messaging
│       └── depends on: common, creto-authz, creto-crypto, creto-audit
```

### 1.1 Dependency Graph

```
                    ┌─────────────────────────┐
                    │ creto-enablement-common │
                    └───────────┬─────────────┘
                                │
        ┌───────────┬───────────┼───────────┬───────────┐
        ▼           ▼           ▼           ▼           │
   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐     │
   │metering │ │oversight│ │ runtime │ │messaging│     │
   └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘     │
        │           │           │           │           │
        └───────────┴───────────┴───────────┴───────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        ▼                       ▼                       ▼
   ┌─────────┐            ┌─────────┐            ┌─────────┐
   │  authz  │            │  memory │            │  audit  │
   │ (168ns) │            │(vector) │            │(append) │
   └─────────┘            └─────────┘            └─────────┘
        │                       │                       │
        └───────────────────────┼───────────────────────┘
                                ▼
   ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
   │   nhi   │ │ crypto  │ │consensus│ │  audit  │
   └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

---

## 2. Component Design

### 2.1 creto-enablement-common

Shared types used across all Enablement crates:

```rust
// Re-exports from Platform layer
pub use creto_nhi::{AgentIdentity, DelegationChain};
pub use creto_crypto::{Algorithm, Signature};

// Common error types
pub mod error {
    pub struct Error { kind: ErrorKind, source: Option<Box<dyn Error>> }
    pub enum ErrorKind {
        NotFound,
        AlreadyExists,
        InvalidInput(String),
        Unauthorized,
        QuotaExceeded { limit: u64, used: u64 },
        OversightRequired { policy_id: PolicyId },
        SandboxSpawnFailed,
        AttestationFailed,
        EgressDenied { destination: String },
        EncryptionFailed,
        SignatureVerificationFailed,
        DeliveryDenied,
        Timeout,
        ConnectionFailed,
    }
}

// Configuration patterns
pub mod config {
    pub trait EnablementConfig: DeserializeOwned {
        fn validate(&self) -> Result<(), ConfigError>;
    }
}
```

### 2.2 Authorization Integration Pattern

All four products integrate with Authorization using the same pattern:

```rust
use creto_authz::{AuthzClient, CheckRequest, Decision};

pub async fn authorized_operation<T, F>(
    authz: &AuthzClient,
    agent: &AgentIdentity,
    action: &str,
    resource: &str,
    operation: F,
) -> Result<T, Error>
where
    F: FnOnce() -> Future<Output = Result<T, Error>>,
{
    // 1. Check authorization (168ns policy + inline extensions)
    let decision = authz.check(CheckRequest {
        agent: agent.clone(),
        action: action.to_string(),
        resource: resource.to_string(),
    }).await?;

    // 2. Handle decision
    match decision {
        Decision::Allow => {
            let result = operation().await?;
            Ok(result)
        }
        Decision::Deny { reason } => {
            Err(Error::unauthorized(reason))
        }
        Decision::RequiresOversight { policy_id } => {
            Err(Error::oversight_required(policy_id))
        }
    }
}
```

### 2.3 Audit Integration Pattern

All significant operations log to audit:

```rust
use creto_audit::{AuditClient, AuditRecord, Outcome};

pub async fn audited_operation<T, F>(
    audit: &AuditClient,
    agent: &AgentIdentity,
    delegation_chain: &[AgentIdentity],
    action: &str,
    resource: &str,
    operation: F,
) -> Result<T, Error>
where
    F: FnOnce() -> Future<Output = Result<T, Error>>,
{
    let result = operation().await;

    // Log outcome (success or failure)
    audit.log(AuditRecord {
        who: agent.clone(),
        delegation_chain: delegation_chain.to_vec(),
        what: action.to_string(),
        resource: resource.to_string(),
        why: None,
        outcome: match &result {
            Ok(_) => Outcome::Success,
            Err(e) => Outcome::Failure(e.to_string()),
        },
        timestamp: consensus_timestamp(),
        signature: None, // Added by audit service
    }).await?;

    result
}
```

---

## 3. Data Flow Diagrams

### 3.1 Metering Flow

```
Agent Action
     │
     ▼
┌─────────────────┐
│ Authorization   │ ──────► QuotaEnforcer.check_quota()
│ (check action)  │               │
└────────┬────────┘               │
         │ Allow                  │ <10µs
         ▼                        │
┌─────────────────┐               │
│ Perform Action  │ ◄─────────────┘
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Emit Billable   │
│ Event           │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────┐
│ Aggregation     │────►│ Invoice Gen │
│ (SUM/COUNT/etc) │     │ (periodic)  │
└─────────────────┘     └─────────────┘
```

### 3.2 Oversight Flow

```
Agent Action
     │
     ▼
┌─────────────────┐
│ Authorization   │
│ (check action)  │
└────────┬────────┘
         │ RequiresOversight
         ▼
┌─────────────────┐
│ Create Oversight│
│ Request         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Notify Channels │──────► Slack / Email / Webhook
│                 │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Wait for        │◄─────── Human Response
│ Response        │
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌───────┐ ┌───────┐
│Approve│ │ Deny  │
└───┬───┘ └───┬───┘
    │         │
    ▼         ▼
 Resume     Abort
 Action     Action
```

### 3.3 Runtime Flow

```
Agent Spawn Request
     │
     ▼
┌─────────────────┐
│ Authorization   │
│ (check spawn)   │
└────────┬────────┘
         │ Allow
         ▼
┌─────────────────┐      ┌──────────────┐
│ Warm Pool       │◄────►│ Pre-warmed   │
│ Manager         │      │ Sandboxes    │
└────────┬────────┘      └──────────────┘
         │ <100ms
         ▼
┌─────────────────┐
│ Bind NHI to     │
│ Sandbox         │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Generate        │
│ Attestation     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Inject Secrets  │◄───── NHI-delegated secrets
│ (via NHI)       │
└────────┬────────┘
         │
         ▼
    Sandbox Ready
         │
         ▼
┌─────────────────┐
│ Agent Exec      │──────► Egress checked via AuthZ
│                 │
└─────────────────┘
```

### 3.4 Messaging Flow

```
Agent A                                          Agent B
   │                                                │
   ▼                                                │
┌─────────────────┐                                 │
│ Compose Message │                                 │
└────────┬────────┘                                 │
         │                                          │
         ▼                                          │
┌─────────────────┐                                 │
│ Authorization   │                                 │
│ (can_send?)     │                                 │
└────────┬────────┘                                 │
         │ Allow                                    │
         ▼                                          │
┌─────────────────┐                                 │
│ Generate Symkey │                                 │
│ (AES-256-GCM)   │                                 │
└────────┬────────┘                                 │
         │                                          │
         ▼                                          │
┌─────────────────┐                                 │
│ Encrypt Payload │                                 │
└────────┬────────┘                                 │
         │                                          │
         ▼                                          │
┌─────────────────┐                                 │
│ Wrap Key with   │◄───── B's ML-KEM-768 public key │
│ ML-KEM-768      │                                 │
└────────┬────────┘                                 │
         │                                          │
         ▼                                          │
┌─────────────────┐                                 │
│ Sign Envelope   │                                 │
│ (Ed25519+ML-DSA)│                                 │
└────────┬────────┘                                 │
         │                                          │
         ▼                                          │
┌─────────────────────────────────────────┐         │
│            Message Envelope             │         │
│  sender_nhi | recipient_nhi | wrapped   │────────►│
│  encrypted_payload | signature | ts     │         │
└─────────────────────────────────────────┘         │
                                                    ▼
                                          ┌─────────────────┐
                                          │ Verify Signature│
                                          └────────┬────────┘
                                                   │
                                                   ▼
                                          ┌─────────────────┐
                                          │ Unwrap Key      │
                                          │ (ML-KEM-768)    │
                                          └────────┬────────┘
                                                   │
                                                   ▼
                                          ┌─────────────────┐
                                          │ Decrypt Payload │
                                          └────────┬────────┘
                                                   │
                                                   ▼
                                              Message
```

---

## 4. Error Handling Strategy

### 4.1 Error Categories

| Category | Behavior | Example |
|----------|----------|---------|
| **Transient** | Retry with backoff | Network timeout, rate limit |
| **Permanent** | Fail immediately | Invalid input, not found |
| **Authorization** | Return to caller | Unauthorized, quota exceeded |
| **Oversight** | Block and wait | Approval required |

### 4.2 Error Propagation

```rust
// All crates use the common Error type
use creto_enablement_common::error::{Error, ErrorKind, Result};

// Errors include context for debugging
impl Error {
    pub fn with_context(self, context: impl Into<String>) -> Self;
    pub fn chain(self, source: impl Into<Box<dyn Error>>) -> Self;
}

// Transient errors implement RetryPolicy
pub trait RetryPolicy {
    fn should_retry(&self) -> bool;
    fn retry_after(&self) -> Option<Duration>;
}
```

---

## 5. Configuration Architecture

### 5.1 Configuration Sources

```
Environment Variables (highest priority)
         │
         ▼
┌─────────────────┐
│ Config Files    │ (TOML/YAML)
│ /etc/creto/     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Default Values  │ (lowest priority)
└─────────────────┘
```

### 5.2 Per-Crate Configuration

```toml
# /etc/creto/enablement.toml

[common]
authz_endpoint = "grpc://authz.creto.local:50051"
audit_endpoint = "grpc://audit.creto.local:50052"

[metering]
quota_cache_ttl_seconds = 60
aggregation_interval_seconds = 300

[oversight]
default_timeout_seconds = 3600
slack_webhook_url = "${SLACK_WEBHOOK_URL}"
email_smtp_host = "smtp.creto.local"

[runtime]
default_backend = "gvisor"
warm_pool_min_size = 10
warm_pool_max_size = 100

[messaging]
delivery_timeout_seconds = 30
max_message_size_bytes = 1048576
```

---

## 6. Decisions

| Decision | Rationale |
|----------|-----------|
| All crates depend on common | Consistent types, errors, NHI integration |
| Authorization inline, not sidecar | <10µs requirement for quota checks |
| Async-first API | Non-blocking for high-throughput messaging |
| Feature flags for backends | gVisor default, Kata/SGX optional |
| No direct crate-to-crate dependencies | Loose coupling via traits |

---

## 7. Open Questions

1. Should crates share a connection pool to Authorization/Audit?
2. What's the circuit breaker strategy for external service failures?
3. Should configuration support hot-reload?

---

## 8. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
