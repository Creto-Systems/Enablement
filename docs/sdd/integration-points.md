---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
---

# Integration Points Specification

## Purpose

This document specifies the detailed integration points between the Enablement Layer and the Platform/Security layers, including trait definitions, error handling patterns, fallback behaviors, and health check contracts.

## Scope

**In Scope:**
- Platform layer integration contracts (NHI, Crypto, Consensus, Audit)
- Security layer integration contracts (AuthZ, Memory, Storage)
- Error type definitions and propagation
- Fallback behavior specifications
- Health check protocols
- Circuit breaker patterns

**Out of Scope:**
- External service integrations (see 06-integration-design.md)
- Inter-crate integration within Enablement Layer (see 02-architecture.md)

---

## 1. Platform Layer Integration

### 1.1 creto-nhi (Non-Human Identity)

**Purpose**: Agent identity resolution, delegation chain verification, key management, secret delegation.

#### 1.1.1 Trait Definition

```rust
use creto_nhi::{AgentIdentity, DelegationChain, SigningKey, VerifyingKey, SecretRef, Secret};

pub trait NhiResolver {
    /// Resolve agent identity from token
    async fn resolve(&self, token: &str) -> Result<AgentIdentity, NhiError>;

    /// Get delegation chain for attribution
    async fn get_delegation_chain(
        &self,
        agent: &AgentIdentity,
    ) -> Result<DelegationChain, NhiError>;

    /// Verify delegation chain integrity
    async fn verify_delegation(&self, chain: &DelegationChain) -> Result<bool, NhiError>;

    /// Get agent's public keys for encryption/verification
    async fn get_public_keys(
        &self,
        agent: &AgentIdentity,
    ) -> Result<AgentPublicKeys, NhiError>;

    /// Delegate signing operation to NHI
    async fn sign(
        &self,
        agent: &AgentIdentity,
        data: &[u8],
    ) -> Result<HybridSignature, NhiError>;

    /// Get delegated secrets for sandbox injection
    async fn get_delegated_secrets(
        &self,
        agent: &AgentIdentity,
        secret_refs: &[SecretRef],
    ) -> Result<Vec<Secret>, NhiError>;
}

#[derive(Debug, Clone)]
pub struct AgentPublicKeys {
    pub signing_key: VerifyingKey,
    pub encryption_key: MlKemPublicKey,
}
```

#### 1.1.2 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum NhiError {
    #[error("Agent not found: {0}")]
    NotFound(String),

    #[error("Token expired")]
    TokenExpired,

    #[error("Token invalid: {0}")]
    TokenInvalid(String),

    #[error("Delegation chain verification failed: {0}")]
    DelegationVerificationFailed(String),

    #[error("Public key not found for agent {0}")]
    PublicKeyNotFound(String),

    #[error("Secret not found: {0}")]
    SecretNotFound(String),

    #[error("Secret access denied for agent {0}")]
    SecretAccessDenied(String),

    #[error("NHI service unavailable")]
    Unavailable,

    #[error("NHI timeout after {0:?}")]
    Timeout(Duration),

    #[error("Internal NHI error: {0}")]
    Internal(String),
}

impl NhiError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Unavailable | Self::Timeout(_))
    }
}
```

#### 1.1.3 Usage by Enablement Crates

| Crate | NHI Usage | Methods Used |
|-------|-----------|--------------|
| **Metering** | Attribution (agent_nhi, delegation_chain) | `resolve()`, `get_delegation_chain()` |
| **Oversight** | Identity context, approver verification | `resolve()`, `verify_delegation()` |
| **Runtime** | Sandbox binding, secret injection | `resolve()`, `get_public_keys()`, `get_delegated_secrets()` |
| **Messaging** | Key lookup, signing delegation | `get_public_keys()`, `sign()` |

#### 1.1.4 Fallback Behavior

| Failure Scenario | Fallback Behavior | Rationale |
|------------------|-------------------|-----------|
| NHI unavailable | Deny operation, return error | Cannot proceed without identity |
| Token expired | Return `TokenExpired` error | Client must refresh token |
| Delegation chain invalid | Deny operation | Security critical |
| Secret not found | Return error, log audit | Fail secure |

#### 1.1.5 Health Check

```rust
pub struct NhiHealthCheck {
    client: NhiClient,
}

impl HealthCheck for NhiHealthCheck {
    async fn check(&self) -> HealthStatus {
        match self.client.ping().await {
            Ok(_) => HealthStatus::Healthy,
            Err(NhiError::Timeout(_)) => HealthStatus::Degraded {
                reason: "NHI service slow to respond".to_string(),
            },
            Err(_) => HealthStatus::Unhealthy {
                reason: "NHI service unavailable".to_string(),
            },
        }
    }
}
```

---

### 1.2 creto-crypto (Cryptographic Operations)

**Purpose**: Algorithm-agile cryptographic primitives for encryption, signing, hashing, and key encapsulation.

#### 1.2.1 Trait Definition

```rust
use creto_crypto::{
    Algorithm, SymmetricKey, EncryptedData,
    MlKemPublicKey, MlKemPrivateKey, SharedSecret, Ciphertext,
    HybridSignature, HybridPublicKey, Hash
};

pub trait CryptoProvider {
    /// Symmetric encryption
    fn encrypt(
        &self,
        algorithm: Algorithm,
        plaintext: &[u8],
        key: &SymmetricKey,
    ) -> Result<EncryptedData, CryptoError>;

    /// Symmetric decryption
    fn decrypt(
        &self,
        ciphertext: &EncryptedData,
        key: &SymmetricKey,
    ) -> Result<Vec<u8>, CryptoError>;

    /// ML-KEM key encapsulation (for key wrapping)
    fn kem_encapsulate(
        &self,
        algorithm: Algorithm,
        public_key: &MlKemPublicKey,
    ) -> Result<(SharedSecret, Ciphertext), CryptoError>;

    /// ML-KEM decapsulation
    fn kem_decapsulate(
        &self,
        algorithm: Algorithm,
        ciphertext: &Ciphertext,
        private_key: &MlKemPrivateKey,
    ) -> Result<SharedSecret, CryptoError>;

    /// Verify hybrid signature (Ed25519 + ML-DSA)
    fn verify_hybrid(
        &self,
        signature: &HybridSignature,
        message: &[u8],
        public_keys: &HybridPublicKey,
    ) -> Result<(), CryptoError>;

    /// Sign with hybrid signature
    fn sign_hybrid(
        &self,
        message: &[u8],
        signing_key: &SigningKey,
    ) -> Result<HybridSignature, CryptoError>;

    /// Hash with configurable algorithm
    fn hash(&self, algorithm: Algorithm, data: &[u8]) -> Hash;

    /// Generate random symmetric key
    fn generate_symmetric_key(&self, algorithm: Algorithm) -> SymmetricKey;
}
```

#### 1.2.2 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Signature verification failed")]
    SignatureVerificationFailed,

    #[error("Signature generation failed: {0}")]
    SignatureGenerationFailed(String),

    #[error("Key encapsulation failed: {0}")]
    KeyEncapsulationFailed(String),

    #[error("Key decapsulation failed: {0}")]
    KeyDecapsulationFailed(String),

    #[error("Unsupported algorithm: {0:?}")]
    UnsupportedAlgorithm(Algorithm),

    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("Invalid ciphertext")]
    InvalidCiphertext,

    #[error("Internal crypto error: {0}")]
    Internal(String),
}

impl CryptoError {
    pub fn is_retryable(&self) -> bool {
        false // Crypto errors are deterministic
    }
}
```

#### 1.2.3 Algorithm Selection

```rust
pub mod algorithms {
    use creto_crypto::Algorithm;

    // Symmetric encryption
    pub const SYMMETRIC_DEFAULT: Algorithm = Algorithm::Aes256Gcm;

    // Key encapsulation mechanism (PQC)
    pub const KEM_DEFAULT: Algorithm = Algorithm::MlKem768;

    // Digital signature (hybrid)
    pub const SIGNATURE_CLASSICAL: Algorithm = Algorithm::Ed25519;
    pub const SIGNATURE_PQC: Algorithm = Algorithm::MlDsa65;

    // Hashing
    pub const HASH_DEFAULT: Algorithm = Algorithm::Sha3_256;
}
```

#### 1.2.4 Usage by Enablement Crates

| Crate | Crypto Usage | Operations |
|-------|--------------|------------|
| **Metering** | Invoice signing | `sign_hybrid()`, `verify_hybrid()` |
| **Oversight** | Token signing (action links) | `sign_hybrid()`, `hash()` |
| **Runtime** | Attestation signing, secret encryption | `sign_hybrid()`, `encrypt()`, `decrypt()` |
| **Messaging** | E2E encryption, message signing | All operations |

#### 1.2.5 Fallback Behavior

| Failure Scenario | Fallback Behavior | Rationale |
|------------------|-------------------|-----------|
| Unsupported algorithm | Return error | Fail early |
| Encryption failure | Return error, log | Cannot proceed without encryption |
| Signature verification failure | Deny operation | Security critical |
| Invalid key | Return error | Fail secure |

#### 1.2.6 Health Check

```rust
pub struct CryptoHealthCheck {
    provider: Arc<dyn CryptoProvider>,
}

impl HealthCheck for CryptoHealthCheck {
    async fn check(&self) -> HealthStatus {
        // Test encrypt/decrypt round-trip
        let key = self.provider.generate_symmetric_key(Algorithm::Aes256Gcm);
        let plaintext = b"health check";

        match self.provider.encrypt(Algorithm::Aes256Gcm, plaintext, &key) {
            Ok(ciphertext) => {
                match self.provider.decrypt(&ciphertext, &key) {
                    Ok(decrypted) if decrypted == plaintext => HealthStatus::Healthy,
                    _ => HealthStatus::Unhealthy {
                        reason: "Crypto round-trip failed".to_string(),
                    },
                }
            }
            Err(_) => HealthStatus::Unhealthy {
                reason: "Crypto encryption failed".to_string(),
            },
        }
    }
}
```

---

### 1.3 creto-consensus (Distributed Coordination)

**Purpose**: Consensus-ordered timestamps for consistent event ordering and aggregation.

#### 1.3.1 Trait Definition

```rust
use creto_consensus::{Timestamp, EventId};

pub trait ConsensusProvider {
    /// Get consensus-ordered timestamp
    async fn get_timestamp(&self) -> Result<Timestamp, ConsensusError>;

    /// Order events for consistent aggregation
    async fn order_events(
        &self,
        events: &[EventId],
    ) -> Result<Vec<(EventId, Timestamp)>, ConsensusError>;

    /// Compare timestamps for ordering
    fn compare(&self, a: &Timestamp, b: &Timestamp) -> std::cmp::Ordering;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    pub logical: u64,
    pub physical: i64, // Unix nanoseconds
}
```

#### 1.3.2 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConsensusError {
    #[error("Consensus service unavailable")]
    Unavailable,

    #[error("Consensus timeout after {0:?}")]
    Timeout(Duration),

    #[error("Event ordering conflict: {0}")]
    OrderingConflict(String),

    #[error("Internal consensus error: {0}")]
    Internal(String),
}

impl ConsensusError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Unavailable | Self::Timeout(_))
    }
}
```

#### 1.3.3 Usage by Enablement Crates

| Crate | Consensus Usage | Rationale |
|-------|-----------------|-----------|
| **Metering** | Timestamp billable events, order for aggregation | Ensure consistent billing across replicas |
| **Oversight** | Timestamp state transitions | Audit trail ordering |
| **Runtime** | Timestamp sandbox lifecycle events | Attestation ordering |
| **Messaging** | Timestamp message delivery | Message ordering |

#### 1.3.4 Fallback Behavior

| Failure Scenario | Fallback Behavior | Rationale |
|------------------|-------------------|-----------|
| Consensus unavailable | Use local timestamp + warning flag | Allow degraded operation |
| Timeout | Retry with backoff, then use local timestamp | Eventual consistency |
| Ordering conflict | Return error | Require manual resolution |

#### 1.3.5 Health Check

```rust
pub struct ConsensusHealthCheck {
    client: ConsensusClient,
}

impl HealthCheck for ConsensusHealthCheck {
    async fn check(&self) -> HealthStatus {
        match self.client.get_timestamp().await {
            Ok(_) => HealthStatus::Healthy,
            Err(ConsensusError::Timeout(_)) => HealthStatus::Degraded {
                reason: "Consensus service slow".to_string(),
            },
            Err(_) => HealthStatus::Unhealthy {
                reason: "Consensus service unavailable".to_string(),
            },
        }
    }
}
```

---

### 1.4 creto-audit (Immutable Audit Logging)

**Purpose**: Append-only audit trail with Merkle tree anchoring for non-repudiation.

#### 1.4.1 Trait Definition

```rust
use creto_audit::{AuditRecord, AuditId, SignedAuditId, MerkleProof, AuditFilter};

pub trait AuditLogger {
    /// Log operation to audit trail
    async fn log(&self, record: AuditRecord) -> Result<AuditId, AuditError>;

    /// Log with pre-generated signature
    async fn log_signed(
        &self,
        record: AuditRecord,
        signature: HybridSignature,
    ) -> Result<SignedAuditId, AuditError>;

    /// Get Merkle proof for verification
    async fn get_proof(&self, id: &AuditId) -> Result<MerkleProof, AuditError>;

    /// Query audit records
    async fn query(&self, filter: AuditFilter) -> Result<Vec<AuditRecord>, AuditError>;

    /// Verify record against Merkle root
    async fn verify(
        &self,
        id: &AuditId,
        proof: &MerkleProof,
        root: &Hash,
    ) -> Result<bool, AuditError>;
}

#[derive(Debug, Clone)]
pub struct AuditRecord {
    pub who: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,
    pub what: String,
    pub resource: String,
    pub why: Option<String>,
    pub outcome: Outcome,
    pub timestamp: Timestamp,
    pub signature: Option<HybridSignature>,
}

#[derive(Debug, Clone)]
pub enum Outcome {
    Success,
    Failure(String),
    Denied(String),
    Timeout,
}
```

#### 1.4.2 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Audit record validation failed: {0}")]
    ValidationFailed(String),

    #[error("Signature invalid")]
    InvalidSignature,

    #[error("Audit service unavailable")]
    Unavailable,

    #[error("Audit timeout after {0:?}")]
    Timeout(Duration),

    #[error("Merkle proof verification failed")]
    ProofVerificationFailed,

    #[error("Audit record not found: {0}")]
    NotFound(AuditId),

    #[error("Internal audit error: {0}")]
    Internal(String),
}

impl AuditError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Unavailable | Self::Timeout(_))
    }
}
```

#### 1.4.3 Async Write Pattern

```rust
/// Audit writes are async to avoid blocking hot path
pub struct AsyncAuditLogger {
    sender: mpsc::UnboundedSender<AuditRecord>,
}

impl AsyncAuditLogger {
    pub fn new(client: AuditClient) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut batch = Vec::new();
            let mut interval = tokio::time::interval(Duration::from_millis(1000));

            loop {
                tokio::select! {
                    Some(record) = rx.recv() => {
                        batch.push(record);
                        if batch.len() >= 100 {
                            client.log_batch(&batch).await.ok();
                            batch.clear();
                        }
                    }
                    _ = interval.tick() => {
                        if !batch.is_empty() {
                            client.log_batch(&batch).await.ok();
                            batch.clear();
                        }
                    }
                }
            }
        });

        Self { sender: tx }
    }

    pub fn log(&self, record: AuditRecord) {
        // Fire and forget - logs are async
        let _ = self.sender.send(record);
    }
}
```

#### 1.4.4 Usage by Enablement Crates

| Crate | Audit Usage | Events Logged |
|-------|-------------|---------------|
| **Metering** | Log quota checks, invoice generation | `quota_checked`, `quota_exceeded`, `invoice_generated` |
| **Oversight** | Log request creation, approvals, denials | `oversight_requested`, `approved`, `denied`, `escalated` |
| **Runtime** | Log sandbox lifecycle, egress attempts | `sandbox_spawned`, `sandbox_terminated`, `egress_attempted` |
| **Messaging** | Log message delivery, delivery failures | `message_sent`, `message_received`, `delivery_denied` |

#### 1.4.5 Fallback Behavior

| Failure Scenario | Fallback Behavior | Rationale |
|------------------|-------------------|-----------|
| Audit unavailable | Log locally, retry in background | Don't block operation |
| Timeout | Retry with backoff | Eventual audit consistency |
| Batch full | Drop oldest entries with warning | Prevent memory exhaustion |
| Signature verification fails | Log unsigned, flag for review | Maintain audit trail |

#### 1.4.6 Health Check

```rust
pub struct AuditHealthCheck {
    client: AuditClient,
}

impl HealthCheck for AuditHealthCheck {
    async fn check(&self) -> HealthStatus {
        // Test write and read
        let test_record = AuditRecord {
            who: AgentIdentity::system(),
            delegation_chain: vec![],
            what: "health_check".to_string(),
            resource: "audit".to_string(),
            why: None,
            outcome: Outcome::Success,
            timestamp: Timestamp::now(),
            signature: None,
        };

        match self.client.log(test_record).await {
            Ok(_) => HealthStatus::Healthy,
            Err(AuditError::Timeout(_)) => HealthStatus::Degraded {
                reason: "Audit service slow".to_string(),
            },
            Err(_) => HealthStatus::Unhealthy {
                reason: "Audit service unavailable".to_string(),
            },
        }
    }
}
```

---

## 2. Security Layer Integration

### 2.1 creto-authz (Authorization - 168ns Path)

**Purpose**: Policy evaluation, quota enforcement, oversight triggers, inline authorization checks.

#### 2.1.1 Trait Definition

```rust
use creto_authz::{CheckRequest, Decision, PolicyId};

pub trait AuthzChecker {
    /// Check authorization (168ns policy evaluation path)
    async fn check(&self, request: &CheckRequest) -> Result<Decision, AuthzError>;

    /// Batch authorization checks
    async fn check_batch(
        &self,
        requests: &[CheckRequest],
    ) -> Result<Vec<Decision>, AuthzError>;
}

#[derive(Debug, Clone)]
pub struct CheckRequest {
    pub agent: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,
    pub action: String,
    pub resource: String,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Allow,
    Deny {
        reason: String
    },
    RequiresOversight {
        policy_id: PolicyId,
        timeout: Option<Duration>,
    },
    QuotaExceeded {
        limit: u64,
        used: u64,
        reset_at: Option<Timestamp>,
    },
    RateLimited {
        retry_after: Duration
    },
}
```

#### 2.1.2 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthzError {
    #[error("Authorization denied: {0}")]
    Denied(String),

    #[error("Policy not found: {0}")]
    PolicyNotFound(PolicyId),

    #[error("Policy evaluation failed: {0}")]
    EvaluationFailed(String),

    #[error("AuthZ service unavailable")]
    Unavailable,

    #[error("AuthZ timeout after {0:?}")]
    Timeout(Duration),

    #[error("Invalid check request: {0}")]
    InvalidRequest(String),

    #[error("Internal authz error: {0}")]
    Internal(String),
}

impl AuthzError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Unavailable | Self::Timeout(_))
    }
}
```

#### 2.1.3 Inline Extension: Quota Enforcer

```rust
/// QuotaEnforcer is registered as an inline AuthZ extension
pub struct QuotaEnforcer {
    cache: Arc<QuotaCache>,
}

impl AuthzExtension for QuotaEnforcer {
    fn name(&self) -> &'static str {
        "quota_enforcer"
    }

    async fn evaluate(
        &self,
        request: &CheckRequest,
    ) -> Result<Option<Decision>, ExtensionError> {
        // Check quota in <10µs
        let quota = self.cache.get(&request.agent).await?;

        if quota.is_exceeded() {
            Ok(Some(Decision::QuotaExceeded {
                limit: quota.limit,
                used: quota.used,
                reset_at: Some(quota.reset_at),
            }))
        } else {
            Ok(None) // Allow policy to continue
        }
    }
}
```

#### 2.1.4 Integration Points

| Crate | Authorization Usage | Actions Checked |
|-------|---------------------|-----------------|
| **Metering** | Inline quota enforcement via QuotaEnforcer extension | `consume_quota` |
| **Oversight** | Policy triggers `RequiresOversight` decision | `*` (configurable) |
| **Runtime** | Network egress, sandbox spawn authorization | `spawn_sandbox`, `network_egress` |
| **Messaging** | Delivery authorization | `send_message` |

#### 2.1.5 Caching Strategy

```rust
pub struct AuthzCache {
    cache: Arc<RwLock<lru::LruCache<CacheKey, Decision>>>,
    ttl: Duration,
}

#[derive(Hash, Eq, PartialEq)]
struct CacheKey {
    agent: AgentIdentity,
    action: String,
    resource: String,
}

impl AuthzCache {
    pub async fn check_cached(
        &self,
        request: &CheckRequest,
        checker: &dyn AuthzChecker,
    ) -> Result<Decision, AuthzError> {
        let key = CacheKey {
            agent: request.agent.clone(),
            action: request.action.clone(),
            resource: request.resource.clone(),
        };

        // Try cache first
        if let Some(decision) = self.cache.read().await.peek(&key) {
            return Ok(decision.clone());
        }

        // Cache miss - check authz
        let decision = checker.check(request).await?;

        // Cache Allow decisions only (Deny may change)
        if matches!(decision, Decision::Allow) {
            self.cache.write().await.put(key, decision.clone());
        }

        Ok(decision)
    }
}
```

#### 2.1.6 Fallback Behavior

| Failure Scenario | Fallback Behavior | Rationale |
|------------------|-------------------|-----------|
| AuthZ unavailable | **Deny all operations** | Fail secure |
| Timeout (<10µs) | Use cached decision if available, else deny | Performance critical |
| Policy not found | Deny operation | Fail secure |
| Cache full | Evict LRU entries | Bounded memory |

#### 2.1.7 Health Check

```rust
pub struct AuthzHealthCheck {
    client: AuthzClient,
}

impl HealthCheck for AuthzHealthCheck {
    async fn check(&self) -> HealthStatus {
        let test_request = CheckRequest {
            agent: AgentIdentity::system(),
            delegation_chain: vec![],
            action: "health_check".to_string(),
            resource: "authz".to_string(),
            context: None,
        };

        let start = Instant::now();
        match self.client.check(&test_request).await {
            Ok(_) => {
                let latency = start.elapsed();
                if latency > Duration::from_micros(10) {
                    HealthStatus::Degraded {
                        reason: format!("AuthZ slow: {:?}", latency),
                    }
                } else {
                    HealthStatus::Healthy
                }
            }
            Err(_) => HealthStatus::Unhealthy {
                reason: "AuthZ service unavailable".to_string(),
            },
        }
    }
}
```

---

### 2.2 creto-memory (Agent Context)

**Purpose**: Vector-based agent memory for context retrieval in oversight decisions.

#### 2.2.1 Trait Definition

```rust
use creto_memory::{Query, MemorySnippet};

pub trait MemoryProvider {
    /// Get context for oversight request
    async fn get_context(
        &self,
        agent: &AgentIdentity,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemorySnippet>, MemoryError>;

    /// Get reasoning behind agent action
    async fn get_reasoning(
        &self,
        agent: &AgentIdentity,
        action_id: &str,
    ) -> Result<Option<String>, MemoryError>;

    /// Store memory snippet
    async fn store(
        &self,
        agent: &AgentIdentity,
        content: &str,
        metadata: serde_json::Value,
    ) -> Result<MemoryId, MemoryError>;
}

#[derive(Debug, Clone)]
pub struct MemorySnippet {
    pub content: String,
    pub relevance: f32,
    pub timestamp: Timestamp,
    pub metadata: serde_json::Value,
}
```

#### 2.2.2 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Memory service unavailable")]
    Unavailable,

    #[error("Memory timeout after {0:?}")]
    Timeout(Duration),

    #[error("Memory not found for agent {0}")]
    NotFound(AgentIdentity),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Internal memory error: {0}")]
    Internal(String),
}

impl MemoryError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Unavailable | Self::Timeout(_))
    }
}
```

#### 2.2.3 Usage by Enablement Crates

| Crate | Memory Usage | Use Case |
|-------|--------------|----------|
| **Metering** | None | N/A |
| **Oversight** | Get agent context for approval request | Provide approvers with agent reasoning |
| **Runtime** | None (future: store execution traces) | N/A |
| **Messaging** | None | N/A |

#### 2.2.4 Fallback Behavior

| Failure Scenario | Fallback Behavior | Rationale |
|------------------|-------------------|-----------|
| Memory unavailable | Proceed without context | Non-critical for operation |
| Timeout | Return empty results | Degrade gracefully |
| Not found | Return empty results | Agent may not have memory |

#### 2.2.5 Health Check

```rust
pub struct MemoryHealthCheck {
    client: MemoryClient,
}

impl HealthCheck for MemoryHealthCheck {
    async fn check(&self) -> HealthStatus {
        match self.client.ping().await {
            Ok(_) => HealthStatus::Healthy,
            Err(MemoryError::Timeout(_)) => HealthStatus::Degraded {
                reason: "Memory service slow".to_string(),
            },
            Err(_) => HealthStatus::Unhealthy {
                reason: "Memory service unavailable".to_string(),
            },
        }
    }
}
```

---

### 2.3 creto-storage (Encrypted Storage)

**Purpose**: Large payload storage (message payloads, sandbox artifacts, checkpoints).

#### 2.3.1 Trait Definition

```rust
use creto_storage::{StorageRef, Object, Classification};

pub trait StorageProvider {
    /// Store encrypted object
    async fn store(&self, object: Object) -> Result<StorageRef, StorageError>;

    /// Retrieve and decrypt object
    async fn retrieve(&self, reference: &StorageRef) -> Result<Object, StorageError>;

    /// Delete object
    async fn delete(&self, reference: &StorageRef) -> Result<(), StorageError>;

    /// List objects by owner
    async fn list(
        &self,
        owner: &AgentIdentity,
        classification: Option<Classification>,
    ) -> Result<Vec<StorageRef>, StorageError>;
}

#[derive(Debug, Clone)]
pub struct Object {
    pub data: Vec<u8>,
    pub owner: AgentIdentity,
    pub classification: Classification,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
pub enum Classification {
    AgentData,
    SandboxState,
    MessagePayload,
    Artifact,
}
```

#### 2.3.2 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Object not found: {0}")]
    NotFound(StorageRef),

    #[error("Access denied for {0}")]
    AccessDenied(StorageRef),

    #[error("Storage quota exceeded")]
    QuotaExceeded,

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Storage service unavailable")]
    Unavailable,

    #[error("Storage timeout after {0:?}")]
    Timeout(Duration),

    #[error("Internal storage error: {0}")]
    Internal(String),
}

impl StorageError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Unavailable | Self::Timeout(_))
    }
}
```

#### 2.3.3 Usage by Enablement Crates

| Crate | Storage Usage | Stored Objects |
|-------|---------------|----------------|
| **Metering** | None | N/A |
| **Oversight** | None | N/A |
| **Runtime** | Sandbox checkpoints, artifacts | `SandboxState`, `Artifact` |
| **Messaging** | Large message payloads (>1MB) | `MessagePayload` |

#### 2.3.4 Fallback Behavior

| Failure Scenario | Fallback Behavior | Rationale |
|------------------|-------------------|-----------|
| Storage unavailable | Return error for store, log warning for retrieve | Cannot proceed without storage |
| Quota exceeded | Return error, trigger cleanup | Prevent unbounded growth |
| Encryption failure | Return error | Security critical |
| Not found | Return error | Expected behavior |

#### 2.3.5 Health Check

```rust
pub struct StorageHealthCheck {
    client: StorageClient,
}

impl HealthCheck for StorageHealthCheck {
    async fn check(&self) -> HealthStatus {
        // Test write/read/delete cycle
        let test_object = Object {
            data: b"health_check".to_vec(),
            owner: AgentIdentity::system(),
            classification: Classification::AgentData,
            metadata: None,
        };

        match self.client.store(test_object).await {
            Ok(reference) => {
                self.client.delete(&reference).await.ok();
                HealthStatus::Healthy
            }
            Err(StorageError::Timeout(_)) => HealthStatus::Degraded {
                reason: "Storage service slow".to_string(),
            },
            Err(_) => HealthStatus::Unhealthy {
                reason: "Storage service unavailable".to_string(),
            },
        }
    }
}
```

---

## 3. Inference Layer Integration

### 3.1 InferenceProvider Trait

**Purpose**: Unified abstraction for AI model inference across cloud providers and local air-gapped deployments.

#### 3.1.1 Trait Definition

```rust
use async_trait::async_trait;
use tokio_stream::Stream;

/// Unified inference provider abstraction
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    /// Provider identifier
    fn id(&self) -> ProviderId;

    /// Capabilities this provider supports
    fn capabilities(&self) -> &ProviderCapabilities;

    /// Execute completion request
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, InferenceError>;

    /// Stream completion (for long responses)
    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Box<dyn Stream<Item = Result<CompletionChunk, InferenceError>> + Send>, InferenceError>;

    /// Generate embeddings
    async fn embed(&self, texts: &[String]) -> Result<Vec<Embedding>, InferenceError>;

    /// Health status
    async fn health(&self) -> HealthStatus;
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub model: ModelId,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stop_sequences: Vec<String>,
    pub metadata: RequestMetadata,
}

#[derive(Debug, Clone)]
pub struct RequestMetadata {
    pub sandbox_id: SandboxId,
    pub agent_nhi: AgentIdentity,
    pub trace_id: TraceId,
    pub classification: Option<DataClassification>,
}
```

#### 3.1.2 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum InferenceError {
    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(ProviderId),

    #[error("Model not found: {0}")]
    ModelNotFound(ModelId),

    #[error("Request timeout after {0:?}")]
    Timeout(Duration),

    #[error("Rate limited, retry after {0:?}")]
    RateLimited(Duration),

    #[error("Token limit exceeded: {used} > {limit}")]
    TokenLimitExceeded { used: u32, limit: u32 },

    #[error("Prompt injection detected: {risk:?}")]
    PromptInjection { risk: InjectionRisk },

    #[error("Provider error: {0}")]
    ProviderError(String),

    #[error("Content filtered: {reason}")]
    ContentFiltered { reason: String },

    #[error("Not supported: {0}")]
    NotSupported(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl InferenceError {
    pub fn is_retryable(&self) -> bool {
        matches!(self,
            Self::ProviderUnavailable(_) |
            Self::Timeout(_) |
            Self::RateLimited(_)
        )
    }
}
```

#### 3.1.3 Provider Implementations

| Provider | Type | Models | Integration |
|----------|------|--------|-------------|
| **AnthropicProvider** | Cloud | Claude 3.5, Claude 4 | `anthropic-rs` SDK |
| **AzureOpenAIProvider** | Cloud | GPT-4o, GPT-4 | `azure-openai-rs` |
| **BedrockProvider** | Cloud | Claude, Llama, Titan | AWS SDK |
| **VertexAIProvider** | Cloud | Gemini Pro, PaLM | GCP SDK |
| **OpenAIProvider** | Cloud | GPT-4o, o1 | `async-openai` |
| **LocalInferenceProvider** | Local | Llama 3.1, Qwen, Mistral | vLLM/TGI/Ollama |

#### 3.1.4 Usage by Enablement Crates

| Crate | Inference Usage | Use Case |
|-------|-----------------|----------|
| **Metering** | None (may meter inference tokens) | N/A |
| **Oversight** | Optional (AI-assisted approval decisions) | Context enrichment |
| **Runtime** | Primary consumer | Agent sandbox inference |
| **Messaging** | None | N/A |

#### 3.1.5 Routing Policy

```rust
pub enum RoutingPolicy {
    /// Always use cloud providers (default for connected environments)
    CloudFirst { fallback_to_local: bool },

    /// Always use local inference (air-gapped mode)
    LocalOnly,

    /// Route based on data classification
    ClassificationBased {
        local_classifications: Vec<DataClassification>,
    },

    /// Cost-optimized routing
    CostOptimized { max_cost_per_token: Decimal },

    /// Latency-optimized routing
    LatencyOptimized { max_latency_ms: u64 },
}
```

#### 3.1.6 Fallback Behavior

| Failure Scenario | Fallback Behavior | Rationale |
|------------------|-------------------|-----------|
| Cloud provider unavailable | Fall back to local (if configured) | Continue operation |
| Local provider unavailable | Return error | Cannot proceed in air-gap |
| Rate limited | Retry with backoff, then fallback | Respect provider limits |
| Prompt injection detected | Block request, log audit | Security critical |
| Timeout | Retry once, then fallback | Transient failure |

#### 3.1.7 Health Check

```rust
pub struct InferenceHealthCheck {
    router: Arc<InferenceRouter>,
}

impl HealthCheck for InferenceHealthCheck {
    async fn check(&self) -> HealthStatus {
        let mut healthy = 0;
        let mut total = 0;

        for provider in self.router.providers() {
            total += 1;
            if matches!(provider.health().await, HealthStatus::Healthy) {
                healthy += 1;
            }
        }

        match (healthy, total) {
            (0, _) => HealthStatus::Unhealthy {
                reason: "All inference providers unavailable".to_string(),
            },
            (h, t) if h < t => HealthStatus::Degraded {
                reason: format!("{}/{} providers available", h, t),
            },
            _ => HealthStatus::Healthy,
        }
    }
}
```

---

## 4. Error Handling Patterns

### 4.1 Error Categorization

```rust
pub trait ErrorCategory {
    fn category(&self) -> ErrorType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    Transient,   // Retry with backoff
    Permanent,   // Fail immediately
    Security,    // Log, alert, deny
    Degraded,    // Fallback or circuit break
}
```

### 4.2 Error Mapping

```rust
use creto_enablement_common::error::{Error, ErrorKind};

impl From<NhiError> for Error {
    fn from(e: NhiError) -> Self {
        match e {
            NhiError::NotFound(id) => Error::new(
                ErrorKind::NotFound,
                format!("Agent not found: {}", id),
            ),
            NhiError::TokenExpired => Error::new(
                ErrorKind::Unauthorized,
                "Token expired",
            ),
            NhiError::DelegationVerificationFailed(reason) => Error::new(
                ErrorKind::SecurityViolation,
                format!("Delegation verification failed: {}", reason),
            ),
            NhiError::Unavailable => Error::new(
                ErrorKind::ServiceUnavailable { service: "nhi" },
                "NHI service unavailable",
            ),
            NhiError::Timeout(duration) => Error::new(
                ErrorKind::Timeout { after: duration },
                "NHI timeout",
            ),
            _ => Error::new(
                ErrorKind::Internal,
                format!("NHI error: {}", e),
            ),
        }
    }
}

impl From<AuthzError> for Error {
    fn from(e: AuthzError) -> Self {
        match e {
            AuthzError::Denied(reason) => Error::new(
                ErrorKind::PermissionDenied,
                reason,
            ),
            AuthzError::PolicyNotFound(policy_id) => Error::new(
                ErrorKind::NotFound,
                format!("Policy not found: {}", policy_id),
            ),
            AuthzError::Unavailable => Error::new(
                ErrorKind::ServiceUnavailable { service: "authz" },
                "AuthZ service unavailable",
            ),
            _ => Error::new(
                ErrorKind::Internal,
                format!("AuthZ error: {}", e),
            ),
        }
    }
}

impl From<CryptoError> for Error {
    fn from(e: CryptoError) -> Self {
        match e {
            CryptoError::SignatureVerificationFailed => Error::new(
                ErrorKind::SecurityViolation,
                "Signature verification failed",
            ),
            CryptoError::EncryptionFailed(reason) => Error::new(
                ErrorKind::EncryptionFailed,
                reason,
            ),
            CryptoError::DecryptionFailed(reason) => Error::new(
                ErrorKind::DecryptionFailed,
                reason,
            ),
            _ => Error::new(
                ErrorKind::Internal,
                format!("Crypto error: {}", e),
            ),
        }
    }
}
```

### 4.3 Retry with Backoff

```rust
pub async fn with_retry<F, T, E>(
    operation: F,
    config: RetryConfig,
) -> Result<T, E>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
    E: ErrorCategory,
{
    let mut attempts = 0;
    let mut delay = config.initial_delay;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if e.category() == ErrorType::Transient && attempts < config.max_attempts => {
                attempts += 1;
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, config.max_delay);
            }
            Err(e) => return Err(e),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: usize,
    pub initial_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
        }
    }
}
```

---

## 5. Fallback Behaviors

### 5.1 Service Unavailability Matrix

| Service | Criticality | Fallback Strategy | Maximum Degradation |
|---------|-------------|-------------------|---------------------|
| **NHI** | Critical | Deny operation | None - must have identity |
| **AuthZ** | Critical | Deny operation (fail secure) | None - must have authorization |
| **Crypto** | Critical | Deny operation | None - security critical |
| **Consensus** | Medium | Use local timestamp + flag | Eventual consistency |
| **Audit** | Medium | Log locally, retry async | Delayed audit trail |
| **Memory** | Low | Proceed without context | Missing context |
| **Storage** | Medium | Error for writes, warn for reads | Temporary unavailability |

### 5.2 Circuit Breaker Pattern

```rust
pub struct CircuitBreaker<T> {
    client: T,
    state: Arc<AtomicState>,
    config: CircuitBreakerConfig,
    failure_count: Arc<AtomicUsize>,
    last_failure: Arc<Mutex<Option<Instant>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Closed,   // Normal operation
    Open,     // Failing, reject requests
    HalfOpen, // Testing recovery
}

impl<T> CircuitBreaker<T> {
    pub async fn call<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&T) -> Pin<Box<dyn Future<Output = Result<R, E>> + Send>>,
        E: ErrorCategory,
    {
        match self.state.load() {
            State::Closed => {
                match f(&self.client).await {
                    Ok(r) => {
                        self.reset();
                        Ok(r)
                    }
                    Err(e) if e.category() == ErrorType::Transient => {
                        self.record_failure();
                        Err(e)
                    }
                    Err(e) => Err(e),
                }
            }
            State::Open => {
                if self.should_attempt_reset() {
                    self.state.store(State::HalfOpen);
                    self.call(f).await
                } else {
                    Err(/* CircuitOpen error */)
                }
            }
            State::HalfOpen => {
                match f(&self.client).await {
                    Ok(r) => {
                        self.state.store(State::Closed);
                        self.reset();
                        Ok(r)
                    }
                    Err(e) => {
                        self.state.store(State::Open);
                        self.record_failure();
                        Err(e)
                    }
                }
            }
        }
    }

    fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure.lock().unwrap() = Some(Instant::now());

        if count >= self.config.failure_threshold {
            self.state.store(State::Open);
        }
    }

    fn reset(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
    }

    fn should_attempt_reset(&self) -> bool {
        if let Some(last) = *self.last_failure.lock().unwrap() {
            last.elapsed() >= self.config.reset_timeout
        } else {
            false
        }
    }
}
```

---

## 6. Health Check Contracts

### 6.1 Health Check Trait

```rust
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthStatus;

    fn name(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
}

impl HealthStatus {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded { .. })
    }
}
```

### 6.2 Composite Health Check

```rust
pub struct CompositeHealthCheck {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl CompositeHealthCheck {
    pub async fn check_all(&self) -> HealthReport {
        let mut results = HashMap::new();

        for check in &self.checks {
            let status = check.check().await;
            results.insert(check.name().to_string(), status);
        }

        HealthReport { results }
    }
}

pub struct HealthReport {
    pub results: HashMap<String, HealthStatus>,
}

impl HealthReport {
    pub fn overall_status(&self) -> HealthStatus {
        if self.results.values().any(|s| matches!(s, HealthStatus::Unhealthy { .. })) {
            HealthStatus::Unhealthy {
                reason: "One or more dependencies unhealthy".to_string(),
            }
        } else if self.results.values().any(|s| matches!(s, HealthStatus::Degraded { .. })) {
            HealthStatus::Degraded {
                reason: "One or more dependencies degraded".to_string(),
            }
        } else {
            HealthStatus::Healthy
        }
    }
}
```

### 6.3 Health Check HTTP Endpoint

```rust
#[get("/health")]
pub async fn health_check(
    checks: web::Data<CompositeHealthCheck>,
) -> impl Responder {
    let report = checks.check_all().await;
    let status_code = match report.overall_status() {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded { .. } => StatusCode::OK, // Still serve traffic
        HealthStatus::Unhealthy { .. } => StatusCode::SERVICE_UNAVAILABLE,
    };

    HttpResponse::build(status_code).json(report)
}
```

---

## 7. Integration Configuration

### 7.1 Configuration Schema

```toml
# /etc/creto/enablement-integrations.toml

[integrations.nhi]
endpoint = "grpc://nhi.creto-platform.svc:50051"
timeout_ms = 100
retry_max_attempts = 3
retry_initial_delay_ms = 50
retry_max_delay_ms = 500
circuit_breaker_enabled = true
circuit_breaker_failure_threshold = 5
circuit_breaker_reset_timeout_sec = 30

[integrations.authz]
endpoint = "grpc://authz.creto-security.svc:50051"
timeout_ms = 10  # Must be fast for inline checks
cache_enabled = true
cache_ttl_seconds = 60
cache_max_entries = 10000
circuit_breaker_enabled = true

[integrations.crypto]
# Crypto provider is local (no network calls)
# Algorithm selection
symmetric_algorithm = "aes-256-gcm"
kem_algorithm = "ml-kem-768"
signature_classical = "ed25519"
signature_pqc = "ml-dsa-65"
hash_algorithm = "sha3-256"

[integrations.consensus]
endpoint = "grpc://consensus.creto-platform.svc:50051"
timeout_ms = 50
fallback_to_local_timestamp = true
retry_max_attempts = 2

[integrations.audit]
endpoint = "grpc://audit.creto-platform.svc:50051"
async_writes = true
batch_size = 100
flush_interval_ms = 1000
retry_max_attempts = 5
local_buffer_max_size = 10000

[integrations.memory]
endpoint = "grpc://memory.creto-security.svc:50051"
timeout_ms = 200
optional = true  # Degrade gracefully if unavailable

[integrations.storage]
endpoint = "grpc://storage.creto-security.svc:50051"
timeout_ms = 1000
retry_max_attempts = 3
```

---

## 8. Decisions

| Decision | Rationale |
|----------|-----------|
| gRPC for all internal integrations | Performance, streaming, typed contracts |
| Circuit breaker for all external services | Prevent cascade failures |
| Async audit writes with batching | Don't block hot path |
| AuthZ cache with short TTL | Balance performance vs freshness |
| Fail-secure for NHI/AuthZ/Crypto | Security over availability |
| Degrade gracefully for Memory | Non-critical for core operations |
| Local timestamp fallback for Consensus | Eventual consistency acceptable |

---

## 9. Open Questions

1. Should we implement automatic fallback to secondary NHI instances?
2. What's the maximum acceptable local audit buffer size before dropping events?
3. Should AuthZ cache be shared across Enablement crates or per-crate?
4. Do we need cross-region replication for Storage integration?

---

## 10. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
| 2025-12-25 | 0.2 | Inference Architecture Agent | Added Section 3: Inference Layer Integration |
