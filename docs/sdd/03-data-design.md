---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
---

# SDD-03: Data Design

## Purpose

This document defines the data models, schemas, and storage strategy for the Enablement Layer products.

## Scope

**In Scope:**
- Core data structures per crate
- Storage backends and persistence
- Serialization formats
- Data retention policies
- Cross-crate shared types

**Out of Scope:**
- Platform layer data models (creto-nhi, creto-crypto)
- Security layer data models (creto-authz, creto-memory, creto-storage)

---

## 1. Shared Types (creto-enablement-common)

### 1.1 Identity Types

```rust
// Re-exported from creto-nhi
pub use creto_nhi::{AgentIdentity, DelegationChain, HumanIdentity};

/// Unique identifiers with type safety
#[derive(Clone, Debug, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TypedId<T> {
    pub value: Uuid,
    #[serde(skip)]
    _phantom: PhantomData<T>,
}

// Type aliases for domain IDs
pub type EventId = TypedId<BillableEvent>;
pub type RequestId = TypedId<OversightRequest>;
pub type SandboxId = TypedId<Sandbox>;
pub type MessageId = TypedId<MessageEnvelope>;
pub type SubscriptionId = TypedId<Subscription>;
pub type PolicyId = TypedId<Policy>;
pub type TopicId = TypedId<Topic>;
```

### 1.2 Temporal Types

```rust
/// Consensus-ordered timestamp (from creto-consensus)
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct Timestamp {
    /// Epoch milliseconds
    pub millis: u64,
    /// Logical clock for ordering within same millisecond
    pub sequence: u32,
    /// Node that issued timestamp
    pub node_id: u16,
}

/// Duration with serialization support
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Duration {
    pub seconds: u64,
    pub nanos: u32,
}

/// Billing period definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillingPeriod {
    pub start: Timestamp,
    pub end: Timestamp,
    pub period_type: PeriodType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PeriodType {
    Hourly,
    Daily,
    Weekly,
    Monthly,
    Custom,
}
```

### 1.3 Cryptographic Types

```rust
// Re-exported from creto-crypto
pub use creto_crypto::{
    Signature,      // Ed25519 + ML-DSA hybrid
    PublicKey,      // Algorithm-agile
    Hash,           // SHA-256/SHA-3/BLAKE3
    EncryptedData,  // AES-256-GCM ciphertext
    WrappedKey,     // ML-KEM-768 encapsulated key
};

/// Attestation proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Attestation {
    pub subject_id: String,
    pub claims: HashMap<String, Value>,
    pub platform: AttestationPlatform,
    pub evidence: Vec<u8>,
    pub timestamp: Timestamp,
    pub signature: Signature,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AttestationPlatform {
    GVisor { version: String },
    Kata { version: String, hypervisor: String },
    Sgx { mrenclave: [u8; 32], mrsigner: [u8; 32] },
    Sev { measurement: [u8; 48] },
    None,
}
```

### 1.4 Money Types

```rust
/// Monetary amount with currency
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Money {
    /// Amount in minor units (cents, pence, etc.)
    pub amount: i64,
    /// ISO 4217 currency code
    pub currency: Currency,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Currency {
    USD,
    EUR,
    GBP,
    // ... other currencies
}

/// Decimal for precise calculations
pub use rust_decimal::Decimal;
```

---

## 2. Metering Data Models

### 2.1 Billable Event

```rust
/// Stored billable event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredEvent {
    // Identification
    pub id: EventId,
    pub idempotency_key: String,

    // Attribution (Creto extension)
    pub agent_nhi: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,

    // Billing context
    pub subscription_id: SubscriptionId,
    pub customer_id: String,

    // Event data
    pub event_type: String,
    pub properties: Properties,

    // Timing
    pub timestamp: Timestamp,
    pub ingested_at: Timestamp,

    // Verification
    pub signature: Option<Signature>,

    // Processing state
    pub aggregated: bool,
    pub invoiced: bool,
}

/// Event properties (dimensions + values)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Properties {
    pub values: HashMap<String, PropertyValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PropertyValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
}
```

### 2.2 Subscription & Plan

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub customer_id: String,
    pub owner_nhi: AgentIdentity,
    pub plan_id: PlanId,
    pub status: SubscriptionStatus,
    pub billing_period: PeriodType,
    pub started_at: Timestamp,
    pub current_period_start: Timestamp,
    pub current_period_end: Timestamp,
    pub ended_at: Option<Timestamp>,
    pub quotas: Vec<QuotaConfig>,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum SubscriptionStatus {
    Active,
    PastDue,
    Canceled,
    Suspended,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QuotaConfig {
    pub event_type: String,
    pub limit: u64,
    pub period: PeriodType,
    pub action_on_exceed: QuotaAction,
    pub warning_threshold: Option<f64>,  // 0.0-1.0
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum QuotaAction {
    Block,
    AllowWithOverage,
    NotifyOnly,
}
```

### 2.3 Invoice

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Invoice {
    pub id: InvoiceId,
    pub subscription_id: SubscriptionId,
    pub customer_id: String,
    pub period: BillingPeriod,
    pub line_items: Vec<InvoiceLineItem>,
    pub subtotal: Money,
    pub tax: Money,
    pub total: Money,
    pub credits_applied: Money,
    pub status: InvoiceStatus,
    pub issued_at: Option<Timestamp>,
    pub due_at: Option<Timestamp>,
    pub paid_at: Option<Timestamp>,
    pub attribution: CostAttribution,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub event_type: String,
    pub quantity: Decimal,
    pub unit_price: Money,
    pub amount: Money,
    pub period: BillingPeriod,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostAttribution {
    pub by_agent: HashMap<String, Money>,
    pub by_dimension: HashMap<String, HashMap<String, Money>>,
}
```

---

## 3. Oversight Data Models

### 3.1 Oversight Request

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OversightRequest {
    pub id: RequestId,

    // Identity context
    pub agent_nhi: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,

    // Action details
    pub pending_action: Action,
    pub resource: String,
    pub context: RequestContext,

    // Policy
    pub policy_id: PolicyId,
    pub policy_trigger_reason: String,

    // Approval requirements
    pub required_approvers: Vec<ApproverId>,
    pub quorum: ApprovalQuorum,
    pub escalation: Option<EscalationChain>,

    // State
    pub state: OversightState,
    pub responses: Vec<ApprovalResponse>,

    // Timing
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    pub state_changed_at: Timestamp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestContext {
    pub action_description: String,
    pub reasoning: Option<String>,
    pub memory_snippets: Vec<MemorySnippet>,
    pub risk_factors: Vec<RiskFactor>,
    pub impact: ImpactAssessment,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OversightState {
    Pending,
    Approved { final_at: Timestamp },
    Denied { reason: String },
    Escalated { to_tier: u8, at: Timestamp },
    TimedOut { action_taken: TimeoutAction },
    Cancelled { reason: String },
}
```

### 3.2 Approval Response

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApprovalResponse {
    pub approver: HumanIdentity,
    pub decision: ApprovalDecision,
    pub reason: Option<String>,
    pub timestamp: Timestamp,
    pub channel: ChannelId,
    pub signature: Signature,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ApprovalDecision {
    Approve,
    Deny,
    Escalate,
    RequestInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ApprovalQuorum {
    Any,
    All,
    Threshold { required: usize, total: usize },
}
```

### 3.3 Checkpoint

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OversightCheckpoint {
    pub request_id: RequestId,
    pub state: OversightState,
    pub responses: Vec<ApprovalResponse>,
    pub notifications_sent: Vec<NotificationRecord>,
    pub escalation_history: Vec<EscalationEvent>,
    pub checkpoint_at: Timestamp,
    pub version: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationRecord {
    pub channel_id: ChannelId,
    pub sent_at: Timestamp,
    pub notification_id: String,
    pub status: NotificationStatus,
}
```

---

## 4. Runtime Data Models

### 4.1 Sandbox Spec & Handle

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SandboxSpec {
    // Image
    pub image: ImageRef,
    pub image_pull_policy: ImagePullPolicy,

    // Identity (bound at spawn)
    pub agent_nhi: Option<AgentIdentity>,
    pub delegation_chain: Vec<AgentIdentity>,

    // Security
    pub attestation_policy: AttestationPolicy,
    pub syscall_filter: SyscallFilter,
    pub capabilities: Vec<Capability>,
    pub read_only_root: bool,

    // Resources
    pub cpu_millicores: u32,
    pub memory_bytes: u64,
    pub storage_bytes: u64,

    // Network
    pub network_policy: NetworkPolicy,
    pub egress_rules: Vec<EgressRule>,

    // Secrets
    pub secrets: Vec<SecretRef>,
    pub env_vars: HashMap<String, String>,

    // Lifecycle
    pub ttl: Duration,
    pub idle_timeout: Option<Duration>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SandboxHandle {
    pub id: SandboxId,
    pub spec: SandboxSpec,
    pub status: SandboxStatus,
    pub attestation: Option<Attestation>,
    pub created_at: Timestamp,
    pub bound_at: Option<Timestamp>,
    pub runtime_backend: String,
    pub node_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SandboxStatus {
    Creating,
    Warming,
    Ready,
    Running { pid: u32 },
    Paused,
    Checkpointed { checkpoint_id: String },
    Terminated { exit_code: i32, at: Timestamp },
    Failed { error: String, at: Timestamp },
}
```

### 4.2 Warm Pool

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WarmPool {
    pub id: PoolId,
    pub template: SandboxSpec,
    pub min_ready: usize,
    pub max_ready: usize,
    pub max_age: Duration,
    pub reusable: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PoolStats {
    pub pool_id: PoolId,
    pub ready: usize,
    pub in_use: usize,
    pub warming: usize,
    pub total_claims: u64,
    pub avg_claim_latency_ms: f64,
    pub cold_starts: u64,
}
```

---

## 5. Messaging Data Models

### 5.1 Message Envelope

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageEnvelope {
    // Identification
    pub message_id: MessageId,
    pub correlation_id: Option<MessageId>,

    // Routing
    pub sender_nhi: AgentIdentity,
    pub recipient: MessageRecipient,

    // Encrypted content
    pub encrypted_payload: Vec<u8>,
    pub encryption_nonce: [u8; 12],
    pub wrapped_key: Vec<u8>,

    // Signatures (non-repudiation)
    pub signature_ed25519: [u8; 64],
    pub signature_ml_dsa: Vec<u8>,

    // Metadata
    pub timestamp: Timestamp,
    pub ttl: Option<Duration>,
    pub content_type: ContentType,
    pub priority: MessagePriority,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum MessageRecipient {
    Agent(AgentIdentity),
    Topic(TopicId),
    Multicast(Vec<AgentIdentity>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContentType {
    Binary,
    Json,
    Protobuf { schema: String },
    StorageRef { storage_id: String, hash: Hash },
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}
```

### 5.2 Topic & Subscription

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Topic {
    pub id: TopicId,
    pub name: String,
    pub owner_nhi: AgentIdentity,
    pub publish_policy: TopicPolicy,
    pub subscribe_policy: TopicPolicy,
    pub retention: Duration,
    pub max_message_size: usize,
    pub max_subscribers: Option<usize>,
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TopicPolicy {
    Open,
    Private,
    AuthzRequired,
    Allowlist(Vec<AgentIdentity>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MessageSubscription {
    pub id: SubscriptionId,
    pub topic_id: TopicId,
    pub subscriber_nhi: AgentIdentity,
    pub filter: Option<MessageFilter>,
    pub created_at: Timestamp,
    pub last_ack_offset: u64,
}
```

---

## 6. Storage Strategy

### 6.1 Storage Backends

| Data Type | Primary Storage | Secondary/Archive | Retention |
|-----------|-----------------|-------------------|-----------|
| **Events (hot)** | Redis + TimescaleDB | S3 (Parquet) | 90 days hot, 7 years archive |
| **Events (aggregated)** | TimescaleDB | S3 (Parquet) | 7 years |
| **Invoices** | PostgreSQL | S3 (encrypted) | 7 years |
| **Oversight requests** | PostgreSQL | S3 | 7 years |
| **Oversight checkpoints** | Redis (durable) | PostgreSQL | 30 days |
| **Sandbox state** | etcd | PostgreSQL | 30 days |
| **Warm pool metadata** | etcd | — | Ephemeral |
| **Messages (in-flight)** | Redis Streams | — | TTL-based |
| **Messages (persisted)** | ScyllaDB | S3 | Configurable |
| **Topics** | PostgreSQL | — | Indefinite |

### 6.2 Serialization Formats

| Use Case | Format | Rationale |
|----------|--------|-----------|
| API payloads | Protocol Buffers | Schema evolution, compact |
| Storage (structured) | Protocol Buffers | Compact, typed |
| Storage (archive) | Parquet | Columnar, compression |
| Configuration | TOML | Human-readable |
| Logs | JSON | Structured, tooling |
| Wire (internal) | MessagePack | Fast, compact |

### 6.3 Schema Versioning

```rust
/// All stored data includes version for migration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Versioned<T> {
    pub version: u32,
    pub data: T,
}

/// Schema registry for evolution
pub trait SchemaVersion {
    const VERSION: u32;
    fn migrate_from(version: u32, data: &[u8]) -> Result<Self, MigrationError>
    where
        Self: Sized;
}
```

---

## 7. Data Retention & Compliance

### 7.1 Retention Policies

| Data Category | Hot Retention | Archive Retention | Deletion |
|---------------|---------------|-------------------|----------|
| Billing events | 90 days | 7 years | Anonymize after 7y |
| Invoices | 7 years | 7 years | Anonymize after 7y |
| Oversight requests | 1 year | 7 years | Anonymize after 7y |
| Approval signatures | 7 years | 7 years | Never delete |
| Sandbox attestations | 90 days | 7 years | Delete after 7y |
| Messages (content) | TTL or 30 days | — | Hard delete |
| Messages (metadata) | 1 year | 7 years | Anonymize after 7y |

### 7.2 PII Handling

```rust
/// Fields marked as PII for compliance
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PiiField<T> {
    #[serde(skip_serializing_if = "is_redacted")]
    pub value: Option<T>,
    pub redacted: bool,
}

impl<T> PiiField<T> {
    pub fn redact(&mut self) {
        self.value = None;
        self.redacted = true;
    }
}
```

---

## 8. Decisions

| Decision | Rationale |
|----------|-----------|
| TimescaleDB for events | Time-series optimized, PostgreSQL compatible |
| Redis for hot path | Sub-millisecond latency, Streams for messaging |
| Parquet for archive | Columnar compression, analytics-ready |
| Protocol Buffers for wire | Schema evolution, language-neutral |
| etcd for sandbox state | Kubernetes-native, strong consistency |

---

## 9. Open Questions

1. Should we use CockroachDB for global distribution?
2. What's the replication factor for message persistence?
3. Should archived data be encrypted at rest with customer keys?

---

## 10. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
