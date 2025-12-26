---
status: approved
author: Claude (AI Assistant)
created: 2025-12-26
updated: 2025-12-26
reviewers: []
---

# Sovereign Platform Integration SDD

## Executive Summary

This document specifies the complete integration between the **Creto Enablement Layer** and the **Creto Sovereign Platform** sibling repositories. It defines API contracts, version compatibility, testing strategies, deployment coordination, and failure mode analysis for each platform dependency.

### Sovereign Platform Repositories

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         CRETO SOVEREIGN PLATFORM                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    ENABLEMENT LAYER (This Repo)                      │   │
│  │  creto-metering │ creto-oversight │ creto-runtime │ creto-messaging │   │
│  └────────────────────────────────┬────────────────────────────────────┘   │
│                                   │                                         │
│                                   ▼                                         │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    SECURITY LAYER (Sibling Repos)                    │   │
│  │  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐         │   │
│  │  │  creto-authz   │  │  creto-memory  │  │ creto-storage  │         │   │
│  │  │   (168ns)      │  │  (RuVector)    │  │  (Encrypted)   │         │   │
│  │  └───────┬────────┘  └───────┬────────┘  └───────┬────────┘         │   │
│  └──────────┼───────────────────┼───────────────────┼──────────────────┘   │
│             │                   │                   │                       │
│             ▼                   ▼                   ▼                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    PLATFORM LAYER (Sibling Repos)                    │   │
│  │  ┌──────────┐  ┌────────────┐  ┌─────────────┐  ┌─────────────────┐ │   │
│  │  │creto-nhi │  │creto-crypto│  │creto-consens│  │  creto-audit    │ │   │
│  │  │(Identity)│  │ (PQC)      │  │   (Raft)    │  │  (Merkle)       │ │   │
│  │  └──────────┘  └────────────┘  └─────────────┘  └─────────────────┘ │   │
│  │                                                                      │   │
│  │  ┌──────────────────────────────────────────────────────────────┐   │   │
│  │  │                    creto-vault (Secrets)                      │   │   │
│  │  │  HSM Integration │ Secret Rotation │ Delegated Access         │   │   │
│  │  └──────────────────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Table of Contents

1. [Repository Overview](#1-repository-overview)
2. [creto-authz Integration](#2-creto-authz-integration)
3. [creto-storage Integration](#3-creto-storage-integration)
4. [creto-memory Integration](#4-creto-memory-integration)
5. [creto-vault Integration](#5-creto-vault-integration)
6. [creto-nhi Integration](#6-creto-nhi-integration)
7. [creto-crypto Integration](#7-creto-crypto-integration)
8. [creto-consensus Integration](#8-creto-consensus-integration)
9. [creto-audit Integration](#9-creto-audit-integration)
10. [Version Compatibility Matrix](#10-version-compatibility-matrix)
11. [Integration Testing Strategy](#11-integration-testing-strategy)
12. [Deployment Coordination](#12-deployment-coordination)
13. [Failure Mode Analysis](#13-failure-mode-analysis)
14. [Operational Procedures](#14-operational-procedures)

---

## 1. Repository Overview

### 1.1 Repository Map

| Repository | Layer | Purpose | Criticality |
|------------|-------|---------|-------------|
| **creto-authz** | Security | Policy evaluation, quota enforcement | Critical |
| **creto-storage** | Security | Encrypted object storage | High |
| **creto-memory** | Security | Vector-based agent memory | Medium |
| **creto-vault** | Platform | Secret management, HSM integration | Critical |
| **creto-nhi** | Platform | Non-Human Identity management | Critical |
| **creto-crypto** | Platform | Cryptographic primitives (PQC) | Critical |
| **creto-consensus** | Platform | Distributed consensus (Raft) | High |
| **creto-audit** | Platform | Immutable audit logging | High |

### 1.2 Dependency Direction

```
Enablement Layer
      │
      ├──────────────────────────────────────────────────────┐
      │                                                      │
      ▼                                                      ▼
Security Layer ◄────────────────────────────────────► Platform Layer
(creto-authz, creto-memory, creto-storage)     (creto-nhi, creto-crypto,
                                                creto-consensus, creto-audit,
                                                creto-vault)
```

**Rules:**
- Enablement depends on Security and Platform
- Security depends on Platform
- Platform has no Enablement dependencies (one-way)
- No circular dependencies allowed

### 1.3 CRITICAL: These Are the ONLY Runtime Dependencies

> **⚠️ IMPORTANT: No External OSS Runtime Dependencies**
>
> The 8 Creto sibling repositories listed above are the **complete and exhaustive**
> list of runtime dependencies for all Enablement Layer products.
>
> **Pattern Sources (NOT Dependencies):**
> The Enablement SDDs reference OSS projects (Lago, HumanLayer, Signal Protocol,
> Agent Sandbox) for **design pattern analysis only**. These patterns were studied
> and then **completely rebuilt in Rust** using the Sovereign primitives above.
>
> **What This Means:**
> - `cargo build` pulls ONLY Creto crates
> - No `pip install`, `npm install`, or Go modules for OSS projects
> - All cryptography from `creto-crypto`
> - All authorization from `creto-authz`
> - All identity from `creto-nhi`
>
> See `docs/sdd/01-requirements.md` Section 1.2 for complete details.

---

## 2. creto-authz Integration

### 2.1 Overview

**Repository**: `github.com/creto-systems/creto-authz`
**Purpose**: Cedar-based policy evaluation with 168ns authorization path
**Protocol**: gRPC with optional REST gateway
**Criticality**: CRITICAL - All operations require authorization

### 2.2 API Contract

#### 2.2.1 gRPC Service Definition

```protobuf
// authz/v1/authz.proto
syntax = "proto3";
package creto.authz.v1;

service AuthorizationService {
  // Primary authorization check (168ns target)
  rpc Check(CheckRequest) returns (CheckResponse);

  // Batch authorization checks
  rpc CheckBatch(CheckBatchRequest) returns (CheckBatchResponse);

  // Register inline extension (e.g., QuotaEnforcer)
  rpc RegisterExtension(RegisterExtensionRequest) returns (RegisterExtensionResponse);

  // Policy management
  rpc CreatePolicy(CreatePolicyRequest) returns (Policy);
  rpc UpdatePolicy(UpdatePolicyRequest) returns (Policy);
  rpc DeletePolicy(DeletePolicyRequest) returns (Empty);
  rpc ListPolicies(ListPoliciesRequest) returns (ListPoliciesResponse);
}

message CheckRequest {
  // Agent making the request
  AgentIdentity principal = 1;

  // Full delegation chain for attribution
  repeated AgentIdentity delegation_chain = 2;

  // Action being performed
  string action = 3;

  // Resource being accessed
  string resource = 4;

  // Optional context for policy evaluation
  google.protobuf.Struct context = 5;

  // Trace ID for distributed tracing
  string trace_id = 6;
}

message CheckResponse {
  Decision decision = 1;

  // Time taken for policy evaluation (target: <168ns)
  uint64 evaluation_time_ns = 2;

  // Policies that contributed to decision
  repeated string matched_policies = 3;
}

message Decision {
  oneof result {
    AllowDecision allow = 1;
    DenyDecision deny = 2;
    OversightDecision requires_oversight = 3;
    QuotaDecision quota_exceeded = 4;
    RateLimitDecision rate_limited = 5;
  }
}

message AllowDecision {
  // Empty - action is allowed
}

message DenyDecision {
  string reason = 1;
  string policy_id = 2;
}

message OversightDecision {
  string policy_id = 1;
  Duration timeout = 2;
  repeated string required_approvers = 3;
  OversightRequirement requirement = 4;
}

message QuotaDecision {
  string quota_id = 1;
  int64 limit = 2;
  int64 used = 3;
  google.protobuf.Timestamp reset_at = 4;
}

message RateLimitDecision {
  Duration retry_after = 1;
  int32 requests_remaining = 2;
}
```

#### 2.2.2 Rust Client Trait

```rust
use async_trait::async_trait;
use std::time::Duration;

/// Authorization client for Enablement Layer
#[async_trait]
pub trait AuthzClient: Send + Sync + 'static {
    /// Check authorization (168ns target latency)
    async fn check(&self, request: CheckRequest) -> Result<Decision, AuthzError>;

    /// Batch authorization check
    async fn check_batch(&self, requests: Vec<CheckRequest>) -> Result<Vec<Decision>, AuthzError>;

    /// Register an inline extension (e.g., QuotaEnforcer)
    async fn register_extension(
        &self,
        name: &str,
        extension: Box<dyn AuthzExtension>,
    ) -> Result<ExtensionId, AuthzError>;

    /// Health check
    async fn health(&self) -> HealthStatus;
}

#[derive(Debug, Clone)]
pub struct CheckRequest {
    pub principal: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,
    pub action: String,
    pub resource: String,
    pub context: Option<serde_json::Value>,
    pub trace_id: TraceId,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Allow,
    Deny { reason: String, policy_id: PolicyId },
    RequiresOversight {
        policy_id: PolicyId,
        timeout: Duration,
        required_approvers: Vec<String>,
    },
    QuotaExceeded {
        quota_id: String,
        limit: i64,
        used: i64,
        reset_at: Option<DateTime<Utc>>,
    },
    RateLimited {
        retry_after: Duration,
    },
}
```

#### 2.2.3 Inline Extension Interface

```rust
/// Extension interface for inline authorization checks
#[async_trait]
pub trait AuthzExtension: Send + Sync {
    /// Extension name (must be unique)
    fn name(&self) -> &str;

    /// Priority (lower = evaluated first)
    fn priority(&self) -> u32;

    /// Evaluate extension logic
    /// Returns Some(Decision) to short-circuit, None to continue
    async fn evaluate(&self, request: &CheckRequest) -> Result<Option<Decision>, ExtensionError>;
}

/// QuotaEnforcer - Metering's inline extension
pub struct QuotaEnforcer {
    quota_cache: Arc<QuotaCache>,
    bloom_filter: Arc<BloomFilter>,
}

#[async_trait]
impl AuthzExtension for QuotaEnforcer {
    fn name(&self) -> &str {
        "quota_enforcer"
    }

    fn priority(&self) -> u32 {
        100 // Run after core policy evaluation
    }

    async fn evaluate(&self, request: &CheckRequest) -> Result<Option<Decision>, ExtensionError> {
        // Fast path: Bloom filter check (O(1), ~1µs)
        if !self.bloom_filter.might_have_quota(&request.principal) {
            return Ok(None); // No quota configured
        }

        // Check quota cache (<10µs target)
        let quota = self.quota_cache.get(&request.principal, &request.action).await?;

        if quota.is_exceeded() {
            Ok(Some(Decision::QuotaExceeded {
                quota_id: quota.id.clone(),
                limit: quota.limit,
                used: quota.used,
                reset_at: Some(quota.reset_at),
            }))
        } else {
            Ok(None) // Within quota
        }
    }
}
```

### 2.3 Usage by Enablement Products

| Product | AuthZ Usage | Actions | Target Latency |
|---------|-------------|---------|----------------|
| **Metering** | Quota enforcement | `consume_quota`, `query_usage` | <10µs |
| **Oversight** | Approval policies | `create_request`, `approve`, `deny` | <1ms |
| **Runtime** | Egress control | `spawn_sandbox`, `network_egress` | <1ms |
| **Messaging** | Delivery authorization | `send_message`, `create_topic` | <1ms |

### 2.4 Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthzError {
    #[error("Authorization denied: {reason}")]
    Denied { reason: String, policy_id: PolicyId },

    #[error("Policy not found: {0}")]
    PolicyNotFound(PolicyId),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Service unavailable")]
    Unavailable,

    #[error("Request timeout after {0:?}")]
    Timeout(Duration),

    #[error("Extension error: {0}")]
    ExtensionError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl AuthzError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Unavailable | Self::Timeout(_))
    }

    pub fn is_security_critical(&self) -> bool {
        matches!(self, Self::Denied { .. } | Self::PolicyNotFound(_))
    }
}
```

### 2.5 Fallback Behavior

| Scenario | Behavior | Rationale |
|----------|----------|-----------|
| AuthZ unavailable | **DENY ALL** | Fail secure |
| Timeout (>10ms) | Use cached decision if available | Performance |
| Cache miss + timeout | **DENY** | Fail secure |
| Extension error | Log, continue without extension | Graceful degradation |
| Invalid policy | **DENY** | Fail secure |

### 2.6 Configuration

```toml
[authz]
# gRPC endpoint
endpoint = "grpc://authz.creto-security.svc:50051"

# Connection settings
timeout_ms = 10          # 10ms max for inline checks
connect_timeout_ms = 100
max_retries = 2
retry_backoff_ms = 5

# Caching
cache_enabled = true
cache_ttl_seconds = 60
cache_max_entries = 10000
cache_allow_only = true   # Only cache Allow decisions

# Circuit breaker
circuit_breaker_enabled = true
failure_threshold = 5
reset_timeout_seconds = 30

# Extensions
extensions_enabled = true
quota_enforcer_enabled = true
```

---

## 3. creto-storage Integration

### 3.1 Overview

**Repository**: `github.com/creto-systems/creto-storage`
**Purpose**: Encrypted object storage with agent-scoped access
**Protocol**: gRPC with S3-compatible REST gateway
**Criticality**: HIGH - Stores checkpoints, artifacts, large payloads

### 3.2 API Contract

#### 3.2.1 gRPC Service Definition

```protobuf
// storage/v1/storage.proto
syntax = "proto3";
package creto.storage.v1;

service StorageService {
  // Object operations
  rpc Store(StoreRequest) returns (StoreResponse);
  rpc Retrieve(RetrieveRequest) returns (RetrieveResponse);
  rpc Delete(DeleteRequest) returns (DeleteResponse);
  rpc List(ListRequest) returns (ListResponse);

  // Streaming for large objects
  rpc StoreStream(stream StoreChunk) returns (StoreResponse);
  rpc RetrieveStream(RetrieveRequest) returns (stream RetrieveChunk);

  // Metadata operations
  rpc GetMetadata(GetMetadataRequest) returns (ObjectMetadata);
  rpc UpdateMetadata(UpdateMetadataRequest) returns (ObjectMetadata);

  // Presigned URLs (for direct client access)
  rpc GeneratePresignedUrl(PresignedUrlRequest) returns (PresignedUrlResponse);
}

message StoreRequest {
  // Object data
  bytes data = 1;

  // Owner agent identity
  AgentIdentity owner = 2;

  // Classification for access control
  Classification classification = 3;

  // Optional metadata
  map<string, string> metadata = 4;

  // Encryption configuration
  EncryptionConfig encryption = 5;

  // TTL (0 = no expiry)
  google.protobuf.Duration ttl = 6;
}

message StoreResponse {
  // Unique storage reference
  StorageRef reference = 1;

  // Object metadata
  ObjectMetadata metadata = 2;
}

message StorageRef {
  string bucket = 1;
  string key = 2;
  string version_id = 3;
  bytes content_hash = 4;  // BLAKE3 hash
}

enum Classification {
  CLASSIFICATION_UNSPECIFIED = 0;
  AGENT_DATA = 1;           // General agent data
  SANDBOX_STATE = 2;        // Sandbox checkpoints
  MESSAGE_PAYLOAD = 3;      // Large message payloads
  ARTIFACT = 4;             // Build artifacts, logs
  SENSITIVE = 5;            // Requires additional encryption
  COMPLIANCE = 6;           // Retention required
}

message EncryptionConfig {
  // Encryption algorithm (default: AES-256-GCM)
  string algorithm = 1;

  // Key wrapping (default: ML-KEM-768)
  string key_wrap_algorithm = 2;

  // Optional customer-managed key
  string customer_key_id = 3;
}

message ObjectMetadata {
  StorageRef reference = 1;
  AgentIdentity owner = 2;
  Classification classification = 3;
  int64 size_bytes = 4;
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp expires_at = 6;
  bytes content_hash = 7;
  map<string, string> custom_metadata = 8;
}
```

#### 3.2.2 Rust Client Trait

```rust
#[async_trait]
pub trait StorageClient: Send + Sync + 'static {
    /// Store object with encryption
    async fn store(&self, object: Object) -> Result<StorageRef, StorageError>;

    /// Store large object via streaming
    async fn store_stream(
        &self,
        metadata: ObjectMetadata,
        stream: impl Stream<Item = Result<Bytes, StorageError>> + Send,
    ) -> Result<StorageRef, StorageError>;

    /// Retrieve and decrypt object
    async fn retrieve(&self, reference: &StorageRef) -> Result<Object, StorageError>;

    /// Retrieve large object via streaming
    async fn retrieve_stream(
        &self,
        reference: &StorageRef,
    ) -> Result<impl Stream<Item = Result<Bytes, StorageError>>, StorageError>;

    /// Delete object
    async fn delete(&self, reference: &StorageRef) -> Result<(), StorageError>;

    /// List objects by owner
    async fn list(
        &self,
        owner: &AgentIdentity,
        classification: Option<Classification>,
        pagination: Pagination,
    ) -> Result<ListResponse, StorageError>;

    /// Generate presigned URL for direct access
    async fn presigned_url(
        &self,
        reference: &StorageRef,
        operation: PresignedOperation,
        ttl: Duration,
    ) -> Result<String, StorageError>;

    /// Health check
    async fn health(&self) -> HealthStatus;
}

#[derive(Debug, Clone)]
pub struct Object {
    pub data: Bytes,
    pub owner: AgentIdentity,
    pub classification: Classification,
    pub metadata: HashMap<String, String>,
    pub encryption: Option<EncryptionConfig>,
    pub ttl: Option<Duration>,
}

#[derive(Debug, Clone)]
pub enum Classification {
    AgentData,
    SandboxState,
    MessagePayload,
    Artifact,
    Sensitive,
    Compliance,
}

#[derive(Debug, Clone)]
pub enum PresignedOperation {
    Get,
    Put { content_type: String, max_size: u64 },
}
```

### 3.3 Usage by Enablement Products

| Product | Storage Usage | Classification | Size Limits |
|---------|---------------|----------------|-------------|
| **Metering** | Invoice PDFs | `Compliance` | 10MB |
| **Oversight** | Approval attachments | `AgentData` | 50MB |
| **Runtime** | Checkpoints, artifacts | `SandboxState`, `Artifact` | 1GB |
| **Messaging** | Large payloads (>1MB) | `MessagePayload` | 100MB |

### 3.4 Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Object not found: {0}")]
    NotFound(StorageRef),

    #[error("Access denied for agent {agent} to {reference}")]
    AccessDenied { agent: AgentIdentity, reference: StorageRef },

    #[error("Quota exceeded: {used}/{limit} bytes")]
    QuotaExceeded { used: u64, limit: u64 },

    #[error("Object too large: {size} > {max} bytes")]
    TooLarge { size: u64, max: u64 },

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },

    #[error("Service unavailable")]
    Unavailable,

    #[error("Request timeout after {0:?}")]
    Timeout(Duration),

    #[error("Internal error: {0}")]
    Internal(String),
}
```

### 3.5 Encryption Model

```rust
/// Storage encryption configuration
pub struct StorageEncryption {
    /// Symmetric encryption for data
    pub data_algorithm: Algorithm,  // AES-256-GCM

    /// Key encapsulation for wrapping data key
    pub wrap_algorithm: Algorithm,  // ML-KEM-768

    /// Envelope structure
    pub envelope: EncryptedEnvelope,
}

/// Encrypted object envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedEnvelope {
    /// Encrypted data
    pub ciphertext: Vec<u8>,

    /// AES-GCM nonce (12 bytes)
    pub nonce: [u8; 12],

    /// ML-KEM encapsulated key
    pub wrapped_key: Vec<u8>,

    /// BLAKE3 hash of plaintext (for integrity)
    pub content_hash: [u8; 32],

    /// Algorithm identifiers
    pub algorithms: AlgorithmIdentifiers,
}

impl StorageClient for StorageClientImpl {
    async fn store(&self, object: Object) -> Result<StorageRef, StorageError> {
        // 1. Generate random data encryption key (DEK)
        let dek = self.crypto.generate_symmetric_key(Algorithm::Aes256Gcm);

        // 2. Encrypt data with DEK
        let nonce = self.crypto.generate_nonce();
        let ciphertext = self.crypto.encrypt(
            Algorithm::Aes256Gcm,
            &object.data,
            &dek,
            &nonce,
        )?;

        // 3. Wrap DEK with owner's ML-KEM public key
        let owner_pubkey = self.nhi.get_public_keys(&object.owner).await?.encryption_key;
        let (shared_secret, wrapped_key) = self.crypto.kem_encapsulate(
            Algorithm::MlKem768,
            &owner_pubkey,
        )?;

        // 4. Derive key encryption key (KEK) from shared secret
        let kek = self.crypto.kdf(&shared_secret, b"storage-kek");
        let encrypted_dek = self.crypto.encrypt(
            Algorithm::Aes256Gcm,
            &dek,
            &kek,
            &self.crypto.generate_nonce(),
        )?;

        // 5. Compute content hash
        let content_hash = self.crypto.hash(Algorithm::Blake3, &object.data);

        // 6. Store envelope
        let envelope = EncryptedEnvelope {
            ciphertext,
            nonce,
            wrapped_key: encrypted_dek,
            content_hash,
            algorithms: AlgorithmIdentifiers::default(),
        };

        self.backend.store(envelope, object.metadata).await
    }
}
```

### 3.6 Configuration

```toml
[storage]
# gRPC endpoint
endpoint = "grpc://storage.creto-security.svc:50051"

# Connection settings
timeout_ms = 5000
connect_timeout_ms = 1000
max_retries = 3

# Streaming
chunk_size_bytes = 1048576  # 1MB chunks
max_concurrent_streams = 10

# Encryption
default_algorithm = "aes-256-gcm"
default_key_wrap = "ml-kem-768"

# Quotas (per agent)
default_quota_bytes = 10737418240  # 10GB
max_object_size_bytes = 1073741824  # 1GB

# Retention
default_ttl_days = 90
compliance_retention_days = 2555  # 7 years
```

---

## 4. creto-memory Integration

### 4.1 Overview

**Repository**: `github.com/creto-systems/creto-memory`
**Purpose**: Vector-based agent memory with semantic search (RuVector)
**Protocol**: gRPC
**Criticality**: MEDIUM - Provides context for oversight decisions

### 4.2 API Contract

#### 4.2.1 gRPC Service Definition

```protobuf
// memory/v1/memory.proto
syntax = "proto3";
package creto.memory.v1;

service MemoryService {
  // Store memory
  rpc Store(StoreMemoryRequest) returns (StoreMemoryResponse);

  // Query by semantic similarity
  rpc Query(QueryRequest) returns (QueryResponse);

  // Get specific memory by ID
  rpc Get(GetRequest) returns (Memory);

  // Delete memory
  rpc Delete(DeleteRequest) returns (DeleteResponse);

  // Get agent's reasoning for action
  rpc GetReasoning(GetReasoningRequest) returns (ReasoningResponse);

  // Bulk operations
  rpc StoreBatch(StoreBatchRequest) returns (StoreBatchResponse);
}

message StoreMemoryRequest {
  // Owner agent
  AgentIdentity agent = 1;

  // Memory content (will be embedded)
  string content = 2;

  // Memory type for filtering
  MemoryType type = 3;

  // Custom metadata
  map<string, string> metadata = 4;

  // Optional embedding (if pre-computed)
  repeated float embedding = 5;

  // TTL (0 = no expiry)
  google.protobuf.Duration ttl = 6;
}

message QueryRequest {
  // Agent to query memories for
  AgentIdentity agent = 1;

  // Query text (will be embedded for similarity search)
  string query = 2;

  // Maximum results
  int32 limit = 3;

  // Minimum relevance threshold (0.0-1.0)
  float relevance_threshold = 4;

  // Optional type filter
  repeated MemoryType type_filter = 5;

  // Time range filter
  TimeRange time_range = 6;
}

message QueryResponse {
  repeated MemoryWithScore results = 1;

  // Query statistics
  QueryStats stats = 2;
}

message MemoryWithScore {
  Memory memory = 1;
  float relevance_score = 2;
}

message Memory {
  string id = 1;
  AgentIdentity agent = 2;
  string content = 3;
  MemoryType type = 4;
  map<string, string> metadata = 5;
  google.protobuf.Timestamp created_at = 6;
  google.protobuf.Timestamp expires_at = 7;
}

enum MemoryType {
  MEMORY_TYPE_UNSPECIFIED = 0;
  OBSERVATION = 1;      // Agent observations
  REASONING = 2;        // Decision reasoning
  CONVERSATION = 3;     // Conversation history
  TOOL_RESULT = 4;      // Tool execution results
  CONTEXT = 5;          // Context for current task
  LONG_TERM = 6;        // Persistent knowledge
}

message GetReasoningRequest {
  AgentIdentity agent = 1;
  string action_id = 2;  // Action to get reasoning for
}

message ReasoningResponse {
  // Found reasoning
  bool found = 1;

  // Reasoning text
  string reasoning = 2;

  // Related memories
  repeated Memory related_memories = 3;

  // Confidence score
  float confidence = 4;
}
```

#### 4.2.2 Rust Client Trait

```rust
#[async_trait]
pub trait MemoryClient: Send + Sync + 'static {
    /// Store memory with automatic embedding
    async fn store(&self, request: StoreMemoryRequest) -> Result<MemoryId, MemoryError>;

    /// Store batch of memories
    async fn store_batch(&self, requests: Vec<StoreMemoryRequest>) -> Result<Vec<MemoryId>, MemoryError>;

    /// Query memories by semantic similarity
    async fn query(&self, request: QueryRequest) -> Result<Vec<MemoryWithScore>, MemoryError>;

    /// Get specific memory by ID
    async fn get(&self, agent: &AgentIdentity, id: &MemoryId) -> Result<Memory, MemoryError>;

    /// Delete memory
    async fn delete(&self, agent: &AgentIdentity, id: &MemoryId) -> Result<(), MemoryError>;

    /// Get reasoning for agent action
    async fn get_reasoning(
        &self,
        agent: &AgentIdentity,
        action_id: &str,
    ) -> Result<Option<ReasoningResponse>, MemoryError>;

    /// Health check
    async fn health(&self) -> HealthStatus;
}

#[derive(Debug, Clone)]
pub struct StoreMemoryRequest {
    pub agent: AgentIdentity,
    pub content: String,
    pub memory_type: MemoryType,
    pub metadata: HashMap<String, String>,
    pub embedding: Option<Vec<f32>>,
    pub ttl: Option<Duration>,
}

#[derive(Debug, Clone)]
pub struct QueryRequest {
    pub agent: AgentIdentity,
    pub query: String,
    pub limit: usize,
    pub relevance_threshold: f32,
    pub type_filter: Option<Vec<MemoryType>>,
    pub time_range: Option<TimeRange>,
}

#[derive(Debug, Clone)]
pub struct MemoryWithScore {
    pub memory: Memory,
    pub relevance_score: f32,
}
```

### 4.3 Usage by Enablement Products

| Product | Memory Usage | Memory Types | Purpose |
|---------|--------------|--------------|---------|
| **Metering** | None | - | - |
| **Oversight** | Query agent context | `Reasoning`, `Context` | Provide approvers with agent's reasoning |
| **Runtime** | Store execution traces | `ToolResult`, `Observation` | Debug, replay |
| **Messaging** | None | - | - |

### 4.4 Oversight Context Retrieval

```rust
impl OversightService {
    /// Get context for oversight request
    pub async fn get_agent_context(
        &self,
        agent: &AgentIdentity,
        action_description: &str,
    ) -> Result<OversightContext, Error> {
        // 1. Query relevant memories
        let memories = self.memory.query(QueryRequest {
            agent: agent.clone(),
            query: action_description.to_string(),
            limit: 5,
            relevance_threshold: 0.7,
            type_filter: Some(vec![MemoryType::Reasoning, MemoryType::Context]),
            time_range: Some(TimeRange::last_hours(24)),
        }).await?;

        // 2. Get specific reasoning if action ID provided
        let reasoning = if let Some(action_id) = self.extract_action_id(action_description) {
            self.memory.get_reasoning(agent, &action_id).await?
        } else {
            None
        };

        // 3. Build oversight context
        Ok(OversightContext {
            relevant_memories: memories.into_iter().map(|m| m.memory).collect(),
            reasoning: reasoning.map(|r| r.reasoning),
            confidence: reasoning.map(|r| r.confidence),
        })
    }
}
```

### 4.5 Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    #[error("Memory not found: {0}")]
    NotFound(MemoryId),

    #[error("Access denied: agent {agent} cannot access memory {memory_id}")]
    AccessDenied { agent: AgentIdentity, memory_id: MemoryId },

    #[error("Embedding failed: {0}")]
    EmbeddingFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),

    #[error("Service unavailable")]
    Unavailable,

    #[error("Request timeout after {0:?}")]
    Timeout(Duration),

    #[error("Internal error: {0}")]
    Internal(String),
}
```

### 4.6 Fallback Behavior

| Scenario | Behavior | Rationale |
|----------|----------|-----------|
| Memory unavailable | Proceed without context | Non-critical |
| Query timeout | Return empty results | Graceful degradation |
| Embedding failed | Skip storage, log warning | Non-blocking |
| Not found | Return None | Expected behavior |

### 4.7 Configuration

```toml
[memory]
# gRPC endpoint
endpoint = "grpc://memory.creto-security.svc:50051"

# Connection settings
timeout_ms = 500
connect_timeout_ms = 200
max_retries = 2

# Query settings
default_limit = 10
default_relevance_threshold = 0.7
max_query_limit = 100

# Embedding (if client-side)
embedding_model = "text-embedding-3-small"
embedding_dimensions = 1536

# Caching
cache_enabled = true
cache_ttl_seconds = 300
cache_max_entries = 1000
```

---

## 5. creto-vault Integration

### 5.1 Overview

**Repository**: `github.com/creto-systems/creto-vault`
**Purpose**: Secret management, HSM integration, delegated secret access
**Protocol**: gRPC with mTLS
**Criticality**: CRITICAL - Manages all secrets for agent execution

### 5.2 API Contract

#### 5.2.1 gRPC Service Definition

```protobuf
// vault/v1/vault.proto
syntax = "proto3";
package creto.vault.v1;

service VaultService {
  // Secret operations (authenticated by agent identity)
  rpc GetSecret(GetSecretRequest) returns (Secret);
  rpc GetSecrets(GetSecretsRequest) returns (GetSecretsResponse);
  rpc CreateSecret(CreateSecretRequest) returns (Secret);
  rpc UpdateSecret(UpdateSecretRequest) returns (Secret);
  rpc DeleteSecret(DeleteSecretRequest) returns (DeleteResponse);

  // Delegated access (for sandbox secret injection)
  rpc GetDelegatedSecrets(GetDelegatedSecretsRequest) returns (DelegatedSecretsResponse);
  rpc CreateDelegation(CreateDelegationRequest) returns (Delegation);
  rpc RevokeDelegation(RevokeDelegationRequest) returns (RevokeResponse);

  // Secret rotation
  rpc RotateSecret(RotateSecretRequest) returns (RotateSecretResponse);
  rpc GetRotationSchedule(GetRotationScheduleRequest) returns (RotationSchedule);

  // Audit
  rpc GetAccessLog(GetAccessLogRequest) returns (AccessLogResponse);
}

message GetSecretRequest {
  // Requesting agent identity
  AgentIdentity requester = 1;

  // Secret reference
  SecretRef reference = 2;

  // Version (empty = latest)
  string version = 3;
}

message Secret {
  SecretRef reference = 1;
  bytes value = 2;  // Encrypted in transit
  string version = 3;
  SecretMetadata metadata = 4;
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp expires_at = 6;
}

message SecretRef {
  string path = 1;      // e.g., "agents/agent-123/api-keys/openai"
  string name = 2;      // e.g., "OPENAI_API_KEY"
}

message SecretMetadata {
  SecretType type = 1;
  string description = 2;
  map<string, string> labels = 3;
  RotationPolicy rotation = 4;
}

enum SecretType {
  SECRET_TYPE_UNSPECIFIED = 0;
  API_KEY = 1;
  PASSWORD = 2;
  CERTIFICATE = 3;
  PRIVATE_KEY = 4;
  TOKEN = 5;
  CONNECTION_STRING = 6;
  GENERIC = 7;
}

// Delegated access for sandbox injection
message GetDelegatedSecretsRequest {
  // Agent requesting secrets for sandbox
  AgentIdentity requester = 1;

  // Target sandbox (will be bound to these secrets)
  SandboxId sandbox_id = 2;

  // Secrets to retrieve
  repeated SecretRef references = 3;

  // TTL for delegated access
  google.protobuf.Duration ttl = 4;
}

message DelegatedSecretsResponse {
  // Secrets with limited TTL
  repeated DelegatedSecret secrets = 1;

  // Delegation record ID (for audit)
  string delegation_id = 2;

  // Expiry of delegation
  google.protobuf.Timestamp expires_at = 3;
}

message DelegatedSecret {
  SecretRef reference = 1;
  bytes value = 2;
  google.protobuf.Timestamp expires_at = 3;

  // Restrictions
  DelegationRestrictions restrictions = 4;
}

message DelegationRestrictions {
  // Secret can only be used by this sandbox
  SandboxId bound_sandbox = 1;

  // Secret expires after first use
  bool single_use = 2;

  // Secret cannot be persisted to disk
  bool memory_only = 3;

  // Maximum number of uses
  int32 max_uses = 4;
}

message CreateDelegationRequest {
  AgentIdentity delegator = 1;
  AgentIdentity delegatee = 2;
  repeated SecretRef secrets = 3;
  google.protobuf.Duration ttl = 4;
  DelegationRestrictions restrictions = 5;
}

message Delegation {
  string id = 1;
  AgentIdentity delegator = 2;
  AgentIdentity delegatee = 3;
  repeated SecretRef secrets = 4;
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp expires_at = 6;
  DelegationRestrictions restrictions = 7;
  DelegationStatus status = 8;
}

enum DelegationStatus {
  DELEGATION_STATUS_UNSPECIFIED = 0;
  ACTIVE = 1;
  EXPIRED = 2;
  REVOKED = 3;
  EXHAUSTED = 4;  // max_uses reached
}
```

#### 5.2.2 Rust Client Trait

```rust
#[async_trait]
pub trait VaultClient: Send + Sync + 'static {
    /// Get secret by reference
    async fn get_secret(
        &self,
        requester: &AgentIdentity,
        reference: &SecretRef,
        version: Option<&str>,
    ) -> Result<Secret, VaultError>;

    /// Get multiple secrets
    async fn get_secrets(
        &self,
        requester: &AgentIdentity,
        references: &[SecretRef],
    ) -> Result<Vec<Secret>, VaultError>;

    /// Get delegated secrets for sandbox injection
    async fn get_delegated_secrets(
        &self,
        requester: &AgentIdentity,
        sandbox_id: &SandboxId,
        references: &[SecretRef],
        ttl: Duration,
    ) -> Result<DelegatedSecretsResponse, VaultError>;

    /// Create delegation for another agent
    async fn create_delegation(
        &self,
        delegator: &AgentIdentity,
        delegatee: &AgentIdentity,
        secrets: &[SecretRef],
        ttl: Duration,
        restrictions: DelegationRestrictions,
    ) -> Result<Delegation, VaultError>;

    /// Revoke delegation
    async fn revoke_delegation(
        &self,
        delegator: &AgentIdentity,
        delegation_id: &str,
    ) -> Result<(), VaultError>;

    /// Health check
    async fn health(&self) -> HealthStatus;
}

#[derive(Debug, Clone)]
pub struct SecretRef {
    pub path: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Secret {
    pub reference: SecretRef,
    pub value: SecretValue,
    pub version: String,
    pub metadata: SecretMetadata,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// Secret value is zeroized on drop
#[derive(Clone)]
pub struct SecretValue(zeroize::Zeroizing<Vec<u8>>);

impl Drop for SecretValue {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

#[derive(Debug, Clone)]
pub struct DelegationRestrictions {
    pub bound_sandbox: Option<SandboxId>,
    pub single_use: bool,
    pub memory_only: bool,
    pub max_uses: Option<u32>,
}

impl Default for DelegationRestrictions {
    fn default() -> Self {
        Self {
            bound_sandbox: None,
            single_use: false,
            memory_only: true,  // Default: never write to disk
            max_uses: None,
        }
    }
}
```

### 5.3 Usage by Enablement Products

| Product | Vault Usage | Secret Types | Use Case |
|---------|-------------|--------------|----------|
| **Metering** | API keys for payment processors | `API_KEY` | Stripe, etc. |
| **Oversight** | Channel credentials | `API_KEY`, `TOKEN` | Slack, Email SMTP |
| **Runtime** | Delegated secrets for sandboxes | All | Sandbox secret injection |
| **Messaging** | None (uses NHI keys) | - | - |

### 5.4 Runtime Sandbox Secret Injection

```rust
impl SandboxRuntime {
    /// Inject secrets into sandbox environment
    pub async fn inject_secrets(
        &self,
        sandbox: &SandboxHandle,
        secret_refs: &[SecretRef],
    ) -> Result<(), RuntimeError> {
        // 1. Get delegated secrets from vault
        let delegated = self.vault.get_delegated_secrets(
            &sandbox.agent_nhi,
            &sandbox.id,
            secret_refs,
            Duration::from_secs(3600), // 1 hour TTL
        ).await?;

        // 2. Log delegation for audit
        self.audit.log(AuditRecord {
            who: sandbox.agent_nhi.clone(),
            what: "secret_delegation",
            resource: format!("sandbox:{}", sandbox.id),
            context: json!({
                "delegation_id": delegated.delegation_id,
                "secret_count": delegated.secrets.len(),
                "expires_at": delegated.expires_at,
            }),
            ..default()
        }).await?;

        // 3. Inject into sandbox (memory-only, not persisted)
        for secret in &delegated.secrets {
            self.sandbox_inject_env(
                &sandbox.id,
                &secret.reference.name,
                &secret.value,
                secret.restrictions.memory_only,
            ).await?;
        }

        // 4. Schedule cleanup on sandbox termination
        self.register_cleanup(&sandbox.id, delegated.delegation_id).await;

        Ok(())
    }

    /// Inject environment variable into sandbox
    async fn sandbox_inject_env(
        &self,
        sandbox_id: &SandboxId,
        name: &str,
        value: &SecretValue,
        memory_only: bool,
    ) -> Result<(), RuntimeError> {
        if memory_only {
            // Use tmpfs mount that's never written to disk
            self.inject_via_tmpfs(sandbox_id, name, value).await
        } else {
            // Write to sandbox filesystem (encrypted)
            self.inject_via_file(sandbox_id, name, value).await
        }
    }
}
```

### 5.5 Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("Secret not found: {0:?}")]
    NotFound(SecretRef),

    #[error("Access denied: agent {agent} cannot access {secret:?}")]
    AccessDenied { agent: AgentIdentity, secret: SecretRef },

    #[error("Delegation not found: {0}")]
    DelegationNotFound(String),

    #[error("Delegation expired")]
    DelegationExpired,

    #[error("Delegation exhausted (max uses reached)")]
    DelegationExhausted,

    #[error("Secret expired")]
    SecretExpired,

    #[error("HSM unavailable")]
    HsmUnavailable,

    #[error("Service unavailable")]
    Unavailable,

    #[error("Request timeout after {0:?}")]
    Timeout(Duration),

    #[error("Internal error: {0}")]
    Internal(String),
}
```

### 5.6 Security Requirements

| Requirement | Implementation |
|-------------|----------------|
| **Transport** | mTLS required (client cert auth) |
| **At-rest** | HSM-backed encryption (FIPS 140-2 Level 3) |
| **Audit** | All access logged with agent identity |
| **Rotation** | Automatic rotation with configurable schedule |
| **Delegation** | Time-limited, sandbox-bound, single-use options |
| **Zeroization** | SecretValue zeroized on drop |

### 5.7 Configuration

```toml
[vault]
# gRPC endpoint
endpoint = "grpc://vault.creto-platform.svc:50051"

# mTLS configuration
tls_enabled = true
tls_cert = "/etc/creto/certs/client.crt"
tls_key = "/etc/creto/certs/client.key"
tls_ca = "/etc/creto/certs/ca.crt"

# Connection settings
timeout_ms = 1000
connect_timeout_ms = 500
max_retries = 3

# Delegation defaults
default_delegation_ttl_seconds = 3600  # 1 hour
max_delegation_ttl_seconds = 86400     # 24 hours

# Security
require_memory_only = true
audit_all_access = true
```

---

## 6. creto-nhi Integration

### 6.1 Overview

**Repository**: `github.com/creto-systems/creto-nhi`
**Purpose**: Non-Human Identity management, delegation chains, key management
**Protocol**: gRPC
**Criticality**: CRITICAL - All operations require agent identity

### 6.2 API Contract

```protobuf
// nhi/v1/nhi.proto
service NhiService {
  // Identity resolution
  rpc ResolveToken(ResolveTokenRequest) returns (AgentIdentity);
  rpc GetAgent(GetAgentRequest) returns (AgentIdentity);
  rpc IsRegistered(IsRegisteredRequest) returns (IsRegisteredResponse);
  rpc IsRevoked(IsRevokedRequest) returns (IsRevokedResponse);

  // Delegation
  rpc GetDelegationChain(GetDelegationChainRequest) returns (DelegationChain);
  rpc VerifyDelegation(VerifyDelegationRequest) returns (VerifyDelegationResponse);
  rpc CreateDelegation(CreateDelegationRequest) returns (Delegation);
  rpc RevokeDelegation(RevokeDelegationRequest) returns (RevokeResponse);

  // Key management
  rpc GetPublicKeys(GetPublicKeysRequest) returns (AgentPublicKeys);
  rpc Sign(SignRequest) returns (HybridSignature);
  rpc Verify(VerifyRequest) returns (VerifyResponse);

  // Agent lifecycle
  rpc RegisterAgent(RegisterAgentRequest) returns (AgentIdentity);
  rpc UpdateAgent(UpdateAgentRequest) returns (AgentIdentity);
  rpc RevokeAgent(RevokeAgentRequest) returns (RevokeResponse);
}

message AgentIdentity {
  string id = 1;                    // Unique agent ID
  string name = 2;                  // Human-readable name
  AgentType type = 3;               // Type of agent
  string owner_id = 4;              // Human or org owner
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp expires_at = 6;
  AgentStatus status = 7;
  map<string, string> attributes = 8;
}

message AgentPublicKeys {
  // Ed25519 signing key (classical)
  bytes signing_key_ed25519 = 1;

  // ML-DSA-65 signing key (post-quantum)
  bytes signing_key_ml_dsa = 2;

  // X25519 key agreement (classical)
  bytes key_agreement_x25519 = 3;

  // ML-KEM-768 encapsulation (post-quantum)
  bytes encapsulation_key_ml_kem = 4;

  // Key validity period
  google.protobuf.Timestamp valid_from = 5;
  google.protobuf.Timestamp valid_until = 6;
}

message DelegationChain {
  AgentIdentity root = 1;           // Original principal (human or service)
  repeated Delegation delegations = 2;
  google.protobuf.Timestamp chain_valid_until = 3;
}

message Delegation {
  string id = 1;
  AgentIdentity delegator = 2;
  AgentIdentity delegatee = 3;
  repeated string permissions = 4;  // Scoped permissions
  google.protobuf.Timestamp created_at = 5;
  google.protobuf.Timestamp expires_at = 6;
  HybridSignature signature = 7;    // Signed by delegator
}

message HybridSignature {
  bytes ed25519_signature = 1;
  bytes ml_dsa_signature = 2;
  string algorithm_id = 3;          // "hybrid-ed25519-ml-dsa-65"
}
```

### 6.3 Rust Client Trait

```rust
#[async_trait]
pub trait NhiClient: Send + Sync + 'static {
    /// Resolve agent identity from token
    async fn resolve_token(&self, token: &str) -> Result<AgentIdentity, NhiError>;

    /// Get agent by ID
    async fn get_agent(&self, id: &str) -> Result<AgentIdentity, NhiError>;

    /// Check if agent is registered
    async fn is_registered(&self, id: &str) -> Result<bool, NhiError>;

    /// Check if agent is revoked
    async fn is_revoked(&self, id: &str) -> Result<bool, NhiError>;

    /// Get delegation chain
    async fn get_delegation_chain(
        &self,
        agent: &AgentIdentity,
    ) -> Result<DelegationChain, NhiError>;

    /// Verify delegation chain integrity
    async fn verify_delegation(
        &self,
        chain: &DelegationChain,
    ) -> Result<bool, NhiError>;

    /// Get agent's public keys
    async fn get_public_keys(
        &self,
        agent: &AgentIdentity,
    ) -> Result<AgentPublicKeys, NhiError>;

    /// Sign data with agent's private key (HSM-backed)
    async fn sign(
        &self,
        agent: &AgentIdentity,
        data: &[u8],
    ) -> Result<HybridSignature, NhiError>;

    /// Verify signature
    async fn verify(
        &self,
        agent: &AgentIdentity,
        data: &[u8],
        signature: &HybridSignature,
    ) -> Result<bool, NhiError>;

    /// Health check
    async fn health(&self) -> HealthStatus;
}
```

### 6.4 Usage by Enablement Products

| Product | NHI Usage | Operations |
|---------|-----------|------------|
| **Metering** | Attribution | `resolve_token`, `get_delegation_chain` |
| **Oversight** | Identity verification | `resolve_token`, `verify_delegation` |
| **Runtime** | Sandbox binding | `get_public_keys`, `sign` (attestation) |
| **Messaging** | Key lookup | `get_public_keys`, `sign`, `verify` |

### 6.5 Configuration

```toml
[nhi]
endpoint = "grpc://nhi.creto-platform.svc:50051"
timeout_ms = 100
max_retries = 3

# Token validation
token_validation_enabled = true
delegation_chain_max_depth = 10

# Caching
cache_agent_ttl_seconds = 300
cache_keys_ttl_seconds = 60
```

---

## 7. creto-crypto Integration

### 7.1 Overview

**Repository**: `github.com/creto-systems/creto-crypto`
**Purpose**: Cryptographic primitives with algorithm agility and PQC support
**Protocol**: Local library (no network calls)
**Criticality**: CRITICAL - All encryption/signing operations

### 7.2 Algorithm Support

| Category | Classical | Post-Quantum | Default |
|----------|-----------|--------------|---------|
| **Symmetric** | AES-256-GCM | - | AES-256-GCM |
| **Signing** | Ed25519 | ML-DSA-65 | Hybrid |
| **Key Agreement** | X25519 | ML-KEM-768 | Hybrid |
| **Hashing** | SHA-256 | SHA3-256 | SHA3-256 |
| **MAC** | HMAC-SHA256 | - | HMAC-SHA256 |

### 7.3 Rust Trait

```rust
pub trait CryptoProvider: Send + Sync + 'static {
    // Symmetric encryption
    fn encrypt(
        &self,
        algorithm: Algorithm,
        plaintext: &[u8],
        key: &SymmetricKey,
        nonce: &[u8],
    ) -> Result<Vec<u8>, CryptoError>;

    fn decrypt(
        &self,
        algorithm: Algorithm,
        ciphertext: &[u8],
        key: &SymmetricKey,
        nonce: &[u8],
    ) -> Result<Vec<u8>, CryptoError>;

    // Key encapsulation (PQC)
    fn kem_encapsulate(
        &self,
        algorithm: Algorithm,
        public_key: &[u8],
    ) -> Result<(SharedSecret, Ciphertext), CryptoError>;

    fn kem_decapsulate(
        &self,
        algorithm: Algorithm,
        ciphertext: &Ciphertext,
        private_key: &[u8],
    ) -> Result<SharedSecret, CryptoError>;

    // Hybrid signatures
    fn sign_hybrid(
        &self,
        message: &[u8],
        ed25519_key: &Ed25519SigningKey,
        ml_dsa_key: &MlDsaSigningKey,
    ) -> Result<HybridSignature, CryptoError>;

    fn verify_hybrid(
        &self,
        message: &[u8],
        signature: &HybridSignature,
        ed25519_key: &Ed25519VerifyingKey,
        ml_dsa_key: &MlDsaVerifyingKey,
    ) -> Result<bool, CryptoError>;

    // Hashing
    fn hash(&self, algorithm: Algorithm, data: &[u8]) -> Hash;

    // Key derivation
    fn kdf(&self, shared_secret: &SharedSecret, context: &[u8]) -> SymmetricKey;

    // Random generation
    fn generate_nonce(&self) -> [u8; 12];
    fn generate_symmetric_key(&self, algorithm: Algorithm) -> SymmetricKey;
}

#[derive(Debug, Clone, Copy)]
pub enum Algorithm {
    // Symmetric
    Aes256Gcm,
    ChaCha20Poly1305,

    // Signing
    Ed25519,
    MlDsa65,
    HybridEd25519MlDsa65,

    // Key encapsulation
    X25519,
    MlKem768,
    HybridX25519MlKem768,

    // Hashing
    Sha256,
    Sha3_256,
    Blake3,
}
```

### 7.4 Usage by Enablement Products

| Product | Crypto Usage | Algorithms |
|---------|--------------|------------|
| **Metering** | Event signing | Hybrid signing |
| **Oversight** | Token signing | HMAC-SHA256, Hybrid |
| **Runtime** | Attestation signing | Hybrid signing |
| **Messaging** | E2E encryption | All |

---

## 8. creto-consensus Integration

### 8.1 Overview

**Repository**: `github.com/creto-systems/creto-consensus`
**Purpose**: Distributed consensus for timestamp ordering (Raft-based)
**Protocol**: gRPC
**Criticality**: HIGH - Ensures consistent event ordering

### 8.2 API Contract

```protobuf
service ConsensusService {
  // Get consensus-ordered timestamp
  rpc GetTimestamp(GetTimestampRequest) returns (Timestamp);

  // Order batch of events
  rpc OrderEvents(OrderEventsRequest) returns (OrderEventsResponse);

  // Get cluster status
  rpc GetClusterStatus(GetClusterStatusRequest) returns (ClusterStatus);
}

message Timestamp {
  uint64 logical = 1;     // Logical clock (monotonic)
  int64 physical = 2;     // Unix nanoseconds
  string node_id = 3;     // Issuing node
}
```

### 8.3 Usage by Enablement Products

| Product | Consensus Usage | Purpose |
|---------|-----------------|---------|
| **Metering** | Event timestamps | Consistent billing aggregation |
| **Oversight** | State transitions | Audit trail ordering |
| **Runtime** | Lifecycle events | Attestation timestamps |
| **Messaging** | Message ordering | Delivery sequence |

### 8.4 Fallback Behavior

| Scenario | Behavior | Rationale |
|----------|----------|-----------|
| Consensus unavailable | Use local timestamp + warning flag | Eventual consistency |
| Timeout | Retry once, then local timestamp | Graceful degradation |

---

## 9. creto-audit Integration

### 9.1 Overview

**Repository**: `github.com/creto-systems/creto-audit`
**Purpose**: Immutable audit logging with Merkle tree anchoring
**Protocol**: gRPC (async writes)
**Criticality**: HIGH - Compliance and forensics

### 9.2 API Contract

```protobuf
service AuditService {
  // Log audit record
  rpc Log(LogRequest) returns (LogResponse);

  // Log batch (for efficiency)
  rpc LogBatch(LogBatchRequest) returns (LogBatchResponse);

  // Query audit records
  rpc Query(QueryRequest) returns (QueryResponse);

  // Get Merkle proof
  rpc GetProof(GetProofRequest) returns (MerkleProof);

  // Verify record
  rpc Verify(VerifyRequest) returns (VerifyResponse);
}

message AuditRecord {
  AgentIdentity who = 1;
  repeated AgentIdentity delegation_chain = 2;
  string what = 3;
  string resource = 4;
  string why = 5;
  Outcome outcome = 6;
  Timestamp timestamp = 7;
  HybridSignature signature = 8;
}

enum Outcome {
  SUCCESS = 0;
  FAILURE = 1;
  DENIED = 2;
  TIMEOUT = 3;
}
```

### 9.3 Async Write Pattern

```rust
pub struct AsyncAuditLogger {
    sender: mpsc::UnboundedSender<AuditRecord>,
}

impl AsyncAuditLogger {
    pub fn log(&self, record: AuditRecord) {
        // Fire-and-forget, non-blocking
        let _ = self.sender.send(record);
    }
}
```

### 9.4 Events by Product

| Product | Events Logged |
|---------|---------------|
| **Metering** | `event_ingested`, `quota_checked`, `quota_exceeded`, `invoice_generated` |
| **Oversight** | `request_created`, `approved`, `denied`, `escalated`, `timeout` |
| **Runtime** | `sandbox_spawned`, `sandbox_terminated`, `egress_allowed`, `egress_denied` |
| **Messaging** | `message_sent`, `message_received`, `delivery_denied`, `topic_created` |

---

## 10. Version Compatibility Matrix

### 10.1 Supported Versions

| Enablement Version | AuthZ | Storage | Memory | Vault | NHI | Crypto | Consensus | Audit |
|-------------------|-------|---------|--------|-------|-----|--------|-----------|-------|
| **0.1.x** | 0.1.x | 0.1.x | 0.1.x | 0.1.x | 0.1.x | 0.1.x | 0.1.x | 0.1.x |
| **0.2.x** | 0.1.x-0.2.x | 0.1.x-0.2.x | 0.1.x-0.2.x | 0.1.x-0.2.x | 0.1.x-0.2.x | 0.1.x-0.2.x | 0.1.x-0.2.x | 0.1.x-0.2.x |
| **1.0.x** | 1.0.x | 1.0.x | 0.2.x+ | 1.0.x | 1.0.x | 1.0.x | 1.0.x | 1.0.x |

### 10.2 Breaking Change Policy

| Change Type | Enablement Response |
|-------------|---------------------|
| **Platform patch** (0.1.1 → 0.1.2) | Automatic compatibility |
| **Platform minor** (0.1.x → 0.2.x) | Integration test required |
| **Platform major** (0.x → 1.0) | Enablement major release required |
| **Deprecation** | 6-month migration window |

### 10.3 Compatibility Verification

```bash
# CI job to verify compatibility
creto-compat-check:
  runs-on: ubuntu-latest
  strategy:
    matrix:
      authz-version: ["0.1.0", "0.1.1", "0.2.0"]
      storage-version: ["0.1.0", "0.1.1"]
  steps:
    - name: Run compatibility tests
      run: |
        AUTHZ_VERSION=${{ matrix.authz-version }} \
        STORAGE_VERSION=${{ matrix.storage-version }} \
        cargo test --features compat-testing
```

---

## 11. Integration Testing Strategy

### 11.1 Test Categories

| Category | Scope | Frequency | Environment |
|----------|-------|-----------|-------------|
| **Unit** | Single crate mocks | Every commit | Local |
| **Integration** | Crate + real service | Every PR | Docker Compose |
| **Contract** | API compatibility | Nightly | Staging |
| **End-to-End** | Full system | Weekly | Production-like |
| **Chaos** | Failure injection | Weekly | Staging |

### 11.2 Contract Tests

```rust
/// Contract tests verify API compatibility with platform services
#[cfg(test)]
mod contract_tests {
    use super::*;

    /// AuthZ contract: Check must return Decision in <10ms
    #[tokio::test]
    async fn authz_check_latency_contract() {
        let client = create_test_authz_client().await;

        let request = CheckRequest {
            principal: test_agent(),
            action: "test_action".into(),
            resource: "test_resource".into(),
            ..default()
        };

        let start = Instant::now();
        let result = client.check(request).await;
        let elapsed = start.elapsed();

        assert!(result.is_ok());
        assert!(elapsed < Duration::from_millis(10), "AuthZ check took {:?}", elapsed);
    }

    /// Storage contract: Store must accept Classification enum
    #[tokio::test]
    async fn storage_classification_contract() {
        let client = create_test_storage_client().await;

        for classification in [
            Classification::AgentData,
            Classification::SandboxState,
            Classification::MessagePayload,
        ] {
            let result = client.store(Object {
                data: b"test".to_vec().into(),
                owner: test_agent(),
                classification,
                ..default()
            }).await;

            assert!(result.is_ok(), "Classification {:?} not accepted", classification);
        }
    }

    /// Memory contract: Query must return MemoryWithScore
    #[tokio::test]
    async fn memory_query_contract() {
        let client = create_test_memory_client().await;

        // Store test memory
        client.store(StoreMemoryRequest {
            agent: test_agent(),
            content: "Test memory content".into(),
            memory_type: MemoryType::Context,
            ..default()
        }).await.unwrap();

        // Query must return with score
        let results = client.query(QueryRequest {
            agent: test_agent(),
            query: "test".into(),
            limit: 10,
            ..default()
        }).await.unwrap();

        assert!(!results.is_empty());
        assert!(results[0].relevance_score >= 0.0 && results[0].relevance_score <= 1.0);
    }

    /// Vault contract: Delegated secrets must be sandbox-bound
    #[tokio::test]
    async fn vault_delegation_contract() {
        let client = create_test_vault_client().await;

        let response = client.get_delegated_secrets(
            &test_agent(),
            &SandboxId::new(),
            &[SecretRef { path: "test".into(), name: "TEST_SECRET".into() }],
            Duration::from_secs(3600),
        ).await.unwrap();

        assert!(response.secrets[0].restrictions.memory_only);
        assert!(response.secrets[0].restrictions.bound_sandbox.is_some());
    }
}
```

### 11.3 Docker Compose for Integration Tests

```yaml
# docker-compose.integration.yml
version: '3.8'

services:
  authz:
    image: creto/authz:${AUTHZ_VERSION:-latest}
    ports:
      - "50051:50051"
    environment:
      - POLICY_STORE=memory
    healthcheck:
      test: ["CMD", "grpc-health-probe", "-addr=:50051"]

  storage:
    image: creto/storage:${STORAGE_VERSION:-latest}
    ports:
      - "50052:50051"
    environment:
      - BACKEND=memory
    healthcheck:
      test: ["CMD", "grpc-health-probe", "-addr=:50051"]

  memory:
    image: creto/memory:${MEMORY_VERSION:-latest}
    ports:
      - "50053:50051"
    environment:
      - EMBEDDING_MODEL=mock
    healthcheck:
      test: ["CMD", "grpc-health-probe", "-addr=:50051"]

  vault:
    image: creto/vault:${VAULT_VERSION:-latest}
    ports:
      - "50054:50051"
    environment:
      - HSM_BACKEND=software
    healthcheck:
      test: ["CMD", "grpc-health-probe", "-addr=:50051"]

  nhi:
    image: creto/nhi:${NHI_VERSION:-latest}
    ports:
      - "50055:50051"
    healthcheck:
      test: ["CMD", "grpc-health-probe", "-addr=:50051"]

  integration-tests:
    build:
      context: .
      dockerfile: Dockerfile.test
    depends_on:
      authz:
        condition: service_healthy
      storage:
        condition: service_healthy
      memory:
        condition: service_healthy
      vault:
        condition: service_healthy
      nhi:
        condition: service_healthy
    environment:
      - AUTHZ_ENDPOINT=authz:50051
      - STORAGE_ENDPOINT=storage:50051
      - MEMORY_ENDPOINT=memory:50051
      - VAULT_ENDPOINT=vault:50051
      - NHI_ENDPOINT=nhi:50051
    command: cargo test --features integration
```

### 11.4 Chaos Testing

```rust
/// Chaos tests inject failures to verify resilience
#[cfg(test)]
mod chaos_tests {
    use super::*;
    use toxiproxy::ToxiProxy;

    /// Test behavior when AuthZ is slow
    #[tokio::test]
    async fn authz_slow_response() {
        let toxiproxy = ToxiProxy::new("authz:50051");
        toxiproxy.add_latency(500).await; // 500ms latency

        let client = create_client_with_proxy(&toxiproxy).await;
        let result = client.authz.check(test_request()).await;

        // Should timeout and deny
        assert!(matches!(result, Err(AuthzError::Timeout(_))));
    }

    /// Test behavior when Memory is unavailable
    #[tokio::test]
    async fn memory_unavailable() {
        let toxiproxy = ToxiProxy::new("memory:50051");
        toxiproxy.disconnect().await;

        let oversight = create_oversight_service().await;
        let context = oversight.get_agent_context(&test_agent(), "test action").await;

        // Should succeed with empty context (graceful degradation)
        assert!(context.is_ok());
        assert!(context.unwrap().relevant_memories.is_empty());
    }

    /// Test behavior when Vault returns expired delegation
    #[tokio::test]
    async fn vault_expired_delegation() {
        let client = create_test_vault_client().await;

        // Create delegation with 1ms TTL (already expired)
        let delegation = client.create_delegation(
            &test_delegator(),
            &test_delegatee(),
            &[test_secret_ref()],
            Duration::from_millis(1),
            default(),
        ).await.unwrap();

        // Wait for expiry
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Should fail with DelegationExpired
        let result = client.get_delegated_secrets(
            &test_delegatee(),
            &SandboxId::new(),
            &[test_secret_ref()],
            Duration::from_secs(60),
        ).await;

        assert!(matches!(result, Err(VaultError::DelegationExpired)));
    }
}
```

---

## 12. Deployment Coordination

### 12.1 Deployment Order

```
1. Platform Layer (creto-crypto, creto-nhi, creto-consensus, creto-audit, creto-vault)
   ↓
2. Security Layer (creto-authz, creto-storage, creto-memory)
   ↓
3. Enablement Layer (creto-metering, creto-oversight, creto-runtime, creto-messaging)
```

### 12.2 Rolling Upgrade Procedure

```bash
#!/bin/bash
# rolling-upgrade.sh

# 1. Verify platform layer health
echo "Checking platform layer..."
for service in nhi crypto consensus audit vault; do
  grpc-health-probe -addr="$service.creto-platform.svc:50051" || exit 1
done

# 2. Verify security layer health
echo "Checking security layer..."
for service in authz storage memory; do
  grpc-health-probe -addr="$service.creto-security.svc:50051" || exit 1
done

# 3. Deploy enablement layer with rolling update
echo "Deploying enablement layer..."
kubectl rollout restart deployment/creto-metering -n creto-enablement
kubectl rollout status deployment/creto-metering -n creto-enablement --timeout=300s

kubectl rollout restart deployment/creto-oversight -n creto-enablement
kubectl rollout status deployment/creto-oversight -n creto-enablement --timeout=300s

kubectl rollout restart deployment/creto-runtime -n creto-enablement
kubectl rollout status deployment/creto-runtime -n creto-enablement --timeout=300s

kubectl rollout restart deployment/creto-messaging -n creto-enablement
kubectl rollout status deployment/creto-messaging -n creto-enablement --timeout=300s

# 4. Verify all services healthy
echo "Verifying deployment..."
for service in metering oversight runtime messaging; do
  grpc-health-probe -addr="$service.creto-enablement.svc:50051" || exit 1
done

echo "Deployment complete!"
```

### 12.3 Version Lock

```yaml
# version-lock.yaml - Deployed to each environment
apiVersion: v1
kind: ConfigMap
metadata:
  name: creto-version-lock
  namespace: creto-enablement
data:
  # Platform layer versions
  nhi: "0.1.2"
  crypto: "0.1.1"
  consensus: "0.1.0"
  audit: "0.1.1"
  vault: "0.1.3"

  # Security layer versions
  authz: "0.1.2"
  storage: "0.1.1"
  memory: "0.1.0"

  # Enablement layer versions
  metering: "0.1.0"
  oversight: "0.1.0"
  runtime: "0.1.0"
  messaging: "0.1.0"
```

### 12.4 Compatibility Gates

```yaml
# GitHub Actions workflow for release
name: Release Gate

on:
  release:
    types: [created]

jobs:
  compatibility-check:
    runs-on: ubuntu-latest
    steps:
      - name: Check platform compatibility
        run: |
          # Fetch minimum required versions
          MIN_AUTHZ=$(yq '.authz.min_version' version-requirements.yaml)
          MIN_NHI=$(yq '.nhi.min_version' version-requirements.yaml)

          # Verify current production versions meet requirements
          CURRENT_AUTHZ=$(kubectl get configmap creto-version-lock -o jsonpath='{.data.authz}')
          CURRENT_NHI=$(kubectl get configmap creto-version-lock -o jsonpath='{.data.nhi}')

          if ! semver-check "$CURRENT_AUTHZ" ">=" "$MIN_AUTHZ"; then
            echo "AuthZ version $CURRENT_AUTHZ does not meet minimum $MIN_AUTHZ"
            exit 1
          fi

          if ! semver-check "$CURRENT_NHI" ">=" "$MIN_NHI"; then
            echo "NHI version $CURRENT_NHI does not meet minimum $MIN_NHI"
            exit 1
          fi

      - name: Run integration tests against production versions
        run: |
          docker-compose -f docker-compose.integration.yml up -d
          cargo test --features integration
```

---

## 13. Failure Mode Analysis

### 13.1 Failure Mode Matrix

| Service | Failure Mode | Detection | Impact | Mitigation | Recovery |
|---------|-------------|-----------|--------|------------|----------|
| **AuthZ** | Unavailable | Health probe | CRITICAL: All operations blocked | Fail secure (deny all) | Automatic reconnect |
| **AuthZ** | Slow (>10ms) | Latency monitoring | HIGH: Degraded performance | Use cached decisions | Scale up |
| **Storage** | Unavailable | Health probe | HIGH: No checkpoints/artifacts | Return error | Retry with backoff |
| **Storage** | Quota exceeded | API response | MEDIUM: Write blocked | Cleanup old objects | Increase quota |
| **Memory** | Unavailable | Health probe | LOW: Missing context | Proceed without context | Automatic reconnect |
| **Memory** | Slow | Latency monitoring | LOW: Delayed context | Empty results | Scale up |
| **Vault** | Unavailable | Health probe | CRITICAL: No secrets | Fail sandbox spawn | Automatic reconnect |
| **Vault** | Delegation expired | API response | MEDIUM: Secret access denied | Request new delegation | Refresh delegation |
| **NHI** | Unavailable | Health probe | CRITICAL: No identity | Deny all operations | Automatic reconnect |
| **NHI** | Token invalid | API response | HIGH: Auth failure | Return 401 | Re-authenticate |
| **Crypto** | Algorithm unsupported | Error response | CRITICAL: Can't encrypt/sign | Fail operation | Deploy update |
| **Consensus** | Unavailable | Health probe | MEDIUM: No consensus timestamps | Use local timestamps | Automatic reconnect |
| **Consensus** | Split brain | Cluster status | HIGH: Inconsistent ordering | Pause writes | Cluster healing |
| **Audit** | Unavailable | Health probe | MEDIUM: Missing audit records | Local buffering | Replay buffer |
| **Audit** | Buffer full | Buffer size | LOW: Dropped records | Drop oldest | Increase buffer |

### 13.2 Cascading Failure Prevention

```rust
/// Circuit breakers prevent cascading failures
pub struct ServiceCircuitBreakers {
    authz: CircuitBreaker<AuthzClient>,
    storage: CircuitBreaker<StorageClient>,
    memory: CircuitBreaker<MemoryClient>,
    vault: CircuitBreaker<VaultClient>,
}

impl ServiceCircuitBreakers {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            authz: CircuitBreaker::new(
                AuthzClient::new(),
                CircuitBreakerConfig {
                    failure_threshold: 5,
                    reset_timeout: Duration::from_secs(30),
                    half_open_requests: 3,
                },
            ),
            storage: CircuitBreaker::new(
                StorageClient::new(),
                config.clone(),
            ),
            memory: CircuitBreaker::new(
                MemoryClient::new(),
                CircuitBreakerConfig {
                    failure_threshold: 10,  // Higher tolerance
                    reset_timeout: Duration::from_secs(60),
                    half_open_requests: 5,
                },
            ),
            vault: CircuitBreaker::new(
                VaultClient::new(),
                config.clone(),
            ),
        }
    }
}
```

### 13.3 Graceful Degradation Matrix

| Service | Degradation Level | Behavior |
|---------|------------------|----------|
| **AuthZ** | None | Fail secure - no degradation allowed |
| **Storage** | Partial | Return errors, don't cache |
| **Memory** | Full | Proceed with empty context |
| **Vault** | Partial | Use cached secrets if available |
| **NHI** | None | Fail secure - no degradation allowed |
| **Crypto** | None | Fail secure - no degradation allowed |
| **Consensus** | Partial | Use local timestamps with warning |
| **Audit** | Full | Buffer locally, replay later |

---

## 14. Operational Procedures

### 14.1 Health Check Aggregation

```rust
pub struct SovereignHealthChecker {
    clients: ServiceClients,
}

impl SovereignHealthChecker {
    pub async fn check_all(&self) -> SovereignHealthReport {
        let (authz, storage, memory, vault, nhi, audit) = tokio::join!(
            self.clients.authz.health(),
            self.clients.storage.health(),
            self.clients.memory.health(),
            self.clients.vault.health(),
            self.clients.nhi.health(),
            self.clients.audit.health(),
        );

        SovereignHealthReport {
            authz,
            storage,
            memory,
            vault,
            nhi,
            audit,
            overall: self.compute_overall(&[authz, storage, memory, vault, nhi, audit]),
        }
    }

    fn compute_overall(&self, statuses: &[HealthStatus]) -> HealthStatus {
        // Any critical service unhealthy = unhealthy
        if statuses.iter().any(|s| {
            matches!(s, HealthStatus::Unhealthy { .. }) &&
            self.is_critical_service(s)
        }) {
            return HealthStatus::Unhealthy {
                reason: "Critical service unavailable".into(),
            };
        }

        // Any service degraded = degraded
        if statuses.iter().any(|s| matches!(s, HealthStatus::Degraded { .. })) {
            return HealthStatus::Degraded {
                reason: "One or more services degraded".into(),
            };
        }

        HealthStatus::Healthy
    }

    fn is_critical_service(&self, service: &str) -> bool {
        matches!(service, "authz" | "nhi" | "vault" | "crypto")
    }
}
```

### 14.2 Monitoring Dashboard Metrics

```yaml
# prometheus-rules.yaml
groups:
  - name: sovereign-integration
    rules:
      # AuthZ latency SLO (p99 < 10ms)
      - alert: AuthzLatencyHigh
        expr: histogram_quantile(0.99, rate(authz_check_duration_seconds_bucket[5m])) > 0.01
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "AuthZ latency above SLO"

      # Storage error rate
      - alert: StorageErrorRateHigh
        expr: rate(storage_requests_total{status="error"}[5m]) / rate(storage_requests_total[5m]) > 0.01
        for: 5m
        labels:
          severity: warning

      # Memory unavailable
      - alert: MemoryUnavailable
        expr: up{job="creto-memory"} == 0
        for: 1m
        labels:
          severity: info  # Non-critical

      # Vault unavailable
      - alert: VaultUnavailable
        expr: up{job="creto-vault"} == 0
        for: 30s
        labels:
          severity: critical
```

### 14.3 Runbook References

| Scenario | Runbook |
|----------|---------|
| AuthZ unavailable | [RUNBOOK: AuthZ Outage](../operations/ENABLEMENT_RUNBOOK.md#authz-outage) |
| Storage quota exceeded | [RUNBOOK: Storage Cleanup](../operations/ENABLEMENT_RUNBOOK.md#storage-cleanup) |
| Vault HSM failure | [RUNBOOK: HSM Recovery](../operations/ENABLEMENT_RUNBOOK.md#hsm-recovery) |
| NHI token refresh failure | [RUNBOOK: Token Refresh](../operations/ENABLEMENT_RUNBOOK.md#token-refresh) |
| Consensus split brain | [RUNBOOK: Cluster Healing](../operations/ENABLEMENT_RUNBOOK.md#cluster-healing) |

---

## Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2025-12-26 | 1.0 | Claude | Initial Sovereign Integration SDD |
