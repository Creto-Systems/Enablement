---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
oss_reference: getlago/lago
---

# Product SDD: creto-metering

## Purpose

Usage-based billing and inline quota enforcement for AI agents. Enables organizations to track agent resource consumption, enforce spending limits in real-time, and generate invoices with full delegation chain attribution.

## Scope

**In Scope:**
- BillableEvent ingestion and deduplication
- Aggregation (COUNT, SUM, UNIQUE_COUNT, MAX)
- Pricing models (flat, per-unit, tiered, package, credits)
- Inline quota enforcement (<10µs)
- Invoice generation
- NHI/delegation chain attribution

**Out of Scope:**
- Payment processing (delegated to Stripe/external)
- UI/dashboard (separate service)
- Credit card storage (PCI compliance out of scope)

---

## 1. OSS Reference: Lago

**Repository:** https://github.com/getlago/lago

**Key Patterns to Extract:**
- Event schema with idempotency
- Aggregation types and formulas
- Pricing model abstractions
- Subscription/plan hierarchies

**Differences from Lago:**
- NHI integration (agent identity + delegation chain)
- Inline quota enforcement via Authorization (not batch)
- Consensus-ordered timestamps
- Crypto-signed events

---

## 2. Core Traits

### 2.1 BillableEvent

```rust
/// Every billable action produces an event
pub trait BillableEvent: Send + Sync {
    /// The agent that performed the action
    fn agent_nhi(&self) -> &AgentIdentity;

    /// Full delegation chain to root human principal
    fn delegation_chain(&self) -> &[AgentIdentity];

    /// Event classification (e.g., "api_call", "vector_query", "llm_token")
    fn event_type(&self) -> &str;

    /// Dimensions for aggregation (model, region, tier, etc.)
    fn properties(&self) -> &Properties;

    /// Consensus-ordered timestamp
    fn timestamp(&self) -> Timestamp;

    /// Deduplication key (idempotency)
    fn idempotency_key(&self) -> &str;
}
```

### 2.2 QuotaEnforcer

```rust
/// Called inline from Authorization service (~10µs budget)
#[async_trait]
pub trait QuotaEnforcer: Send + Sync {
    /// Check if agent has quota remaining for action
    async fn check_quota(
        &self,
        agent: &AgentIdentity,
        action: &Action,
    ) -> Result<QuotaDecision, QuotaError>;

    /// Record usage after action completes
    async fn record_usage(&self, event: &dyn BillableEvent) -> Result<(), Error>;
}

pub enum QuotaDecision {
    /// Action allowed, remaining quota provided
    Allow { remaining: u64 },

    /// Action denied due to quota exhaustion
    Deny {
        reason: QuotaExceeded,
        retry_after: Option<Duration>,
    },
}

pub struct QuotaExceeded {
    pub limit: u64,
    pub used: u64,
    pub period: QuotaPeriod,
    pub resets_at: Option<Timestamp>,
}

pub enum QuotaPeriod {
    Hourly,
    Daily,
    Monthly,
    Total,
}
```

### 2.3 Aggregator

```rust
/// Aggregates events into billable metrics
pub trait Aggregator: Send + Sync {
    fn aggregate_type(&self) -> AggregationType;
    fn aggregate(&self, events: &[&dyn BillableEvent]) -> AggregatedMetric;
}

pub enum AggregationType {
    /// Count of events
    Count,
    /// Sum of property value
    Sum { property: String },
    /// Count of unique property values
    UniqueCount { property: String },
    /// Maximum property value
    Max { property: String },
}

pub struct AggregatedMetric {
    pub event_type: String,
    pub aggregation: AggregationType,
    pub value: Decimal,
    pub period_start: Timestamp,
    pub period_end: Timestamp,
    pub dimensions: HashMap<String, String>,
}
```

### 2.4 PricingModel

```rust
/// Calculates cost from aggregated metrics
pub trait PricingModel: Send + Sync {
    fn calculate(&self, metric: &AggregatedMetric) -> Money;
}

pub enum PricingModelType {
    /// Fixed amount per period
    Flat { amount: Money },

    /// Amount per unit
    PerUnit { unit_price: Money },

    /// Graduated tiers (each tier priced independently)
    TieredGraduated { tiers: Vec<PriceTier> },

    /// Volume tiers (entire volume priced at tier rate)
    TieredVolume { tiers: Vec<PriceTier> },

    /// Package pricing (N units for fixed price)
    Package { package_size: u64, package_price: Money },

    /// Prepaid credit drawdown
    PrepaidCredits { credit_rate: Decimal },
}

pub struct PriceTier {
    pub up_to: Option<u64>,  // None = unlimited
    pub unit_price: Money,
    pub flat_fee: Option<Money>,
}
```

---

## 3. Data Models

### 3.1 Event Storage

```rust
pub struct StoredEvent {
    // Identity
    pub id: EventId,
    pub idempotency_key: String,

    // Who
    pub agent_nhi: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,
    pub subscription_id: SubscriptionId,

    // What
    pub event_type: String,
    pub properties: Properties,

    // When
    pub timestamp: Timestamp,        // Consensus-ordered
    pub ingested_at: Timestamp,      // Server time

    // Verification
    pub signature: Option<Signature>, // Agent signature
}
```

### 3.2 Subscription

```rust
pub struct Subscription {
    pub id: SubscriptionId,

    // Owner (human principal at root of delegation chains)
    pub owner_nhi: AgentIdentity,

    // Plan reference
    pub plan_id: PlanId,

    // Billing details
    pub billing_period: BillingPeriod,
    pub started_at: Timestamp,
    pub ends_at: Option<Timestamp>,

    // Quota configuration
    pub quotas: Vec<QuotaConfig>,

    // Status
    pub status: SubscriptionStatus,
}

pub struct QuotaConfig {
    pub event_type: String,
    pub limit: u64,
    pub period: QuotaPeriod,
    pub action_on_exceed: QuotaAction,
}

pub enum QuotaAction {
    Block,
    AllowWithOverage { overage_price: Money },
    NotifyOnly,
}
```

### 3.3 Invoice

```rust
pub struct Invoice {
    pub id: InvoiceId,
    pub subscription_id: SubscriptionId,

    // Period
    pub period_start: Timestamp,
    pub period_end: Timestamp,

    // Line items
    pub line_items: Vec<InvoiceLineItem>,

    // Totals
    pub subtotal: Money,
    pub tax: Money,
    pub total: Money,

    // Attribution (for multi-tenant cost allocation)
    pub attribution: CostAttribution,

    // Status
    pub status: InvoiceStatus,
    pub issued_at: Option<Timestamp>,
    pub paid_at: Option<Timestamp>,
}

pub struct CostAttribution {
    /// Cost breakdown by agent in delegation chain
    pub by_agent: HashMap<AgentIdentity, Money>,

    /// Cost breakdown by dimension (model, region, etc.)
    pub by_dimension: HashMap<String, HashMap<String, Money>>,
}
```

---

## 4. Architecture

### 4.1 Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      creto-metering                         │
│                                                             │
│  ┌─────────────┐   ┌─────────────┐   ┌─────────────────┐   │
│  │   Event     │   │   Quota     │   │   Invoice       │   │
│  │   Ingestion │   │   Enforcer  │   │   Generator     │   │
│  └──────┬──────┘   └──────┬──────┘   └────────┬────────┘   │
│         │                 │                    │            │
│         ▼                 ▼                    ▼            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Aggregation Engine                      │   │
│  │  (COUNT | SUM | UNIQUE_COUNT | MAX)                  │   │
│  └─────────────────────────────────────────────────────┘   │
│         │                 │                    │            │
│         ▼                 ▼                    ▼            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                Storage Layer                         │   │
│  │   Events (hot) │ Aggregates (warm) │ Invoices (cold) │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         │                 │
         ▼                 ▼
    ┌─────────┐       ┌─────────┐
    │  AuthZ  │       │  Audit  │
    │ (inline)│       │  (log)  │
    └─────────┘       └─────────┘
```

### 4.2 Quota Enforcement Flow (Hot Path)

```
Authorization Check
        │
        ▼
┌───────────────────┐
│ QuotaEnforcer     │
│ .check_quota()    │
└────────┬──────────┘
         │
         ▼
┌───────────────────┐     ┌─────────────────┐
│ Bloom Filter      │────►│ Redis (if miss) │
│ (in-memory)       │     │                 │
└────────┬──────────┘     └────────┬────────┘
         │                         │
         └─────────┬───────────────┘
                   │
                   ▼
              QuotaDecision
              (Allow/Deny)
```

**Latency Budget: <10µs**
- Bloom filter check: ~1µs
- Redis fallback (on miss): ~100µs (amortized rare)

---

## 5. API Design

### 5.1 Event Ingestion

```rust
impl MeteringService {
    /// Ingest a single billable event
    pub async fn ingest_event(&self, event: EventRequest) -> Result<EventId, Error>;

    /// Batch ingest (for high-throughput scenarios)
    pub async fn ingest_batch(&self, events: Vec<EventRequest>) -> Result<BatchResult, Error>;
}

pub struct EventRequest {
    pub idempotency_key: String,
    pub agent_nhi: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,
    pub event_type: String,
    pub properties: Properties,
    pub timestamp: Option<Timestamp>,  // Default: now
}
```

### 5.2 Quota Management

```rust
impl MeteringService {
    /// Check quota (called from Authorization)
    pub async fn check_quota(
        &self,
        agent: &AgentIdentity,
        event_type: &str,
    ) -> Result<QuotaDecision, Error>;

    /// Get current usage
    pub async fn get_usage(
        &self,
        subscription_id: &SubscriptionId,
        period: &BillingPeriod,
    ) -> Result<UsageSummary, Error>;
}
```

### 5.3 Invoice Generation

```rust
impl MeteringService {
    /// Generate invoice for subscription period
    pub async fn generate_invoice(
        &self,
        subscription_id: &SubscriptionId,
        period: &BillingPeriod,
    ) -> Result<Invoice, Error>;

    /// Get cost attribution breakdown
    pub async fn get_attribution(
        &self,
        subscription_id: &SubscriptionId,
        period: &BillingPeriod,
    ) -> Result<CostAttribution, Error>;
}
```

---

## 6. Integration Points

### 6.1 Authorization Integration

```rust
// Called inline during every authorization check
let decision = authz.check(CheckRequest {
    agent: agent_nhi,
    action: "vector_query",
    resource: "memory://embeddings/*",
}).await?;

// Authorization internally calls:
// metering.check_quota(agent_nhi, "vector_query")
```

### 6.2 Audit Integration

```rust
// Every event ingestion is audited
audit.log(AuditRecord {
    who: event.agent_nhi,
    delegation_chain: event.delegation_chain,
    what: "metering_event_ingested",
    resource: format!("event://{}", event.id),
    outcome: Outcome::Success,
    ..
}).await?;
```

---

## 7. Performance Requirements

| Metric | Target | Measurement |
|--------|--------|-------------|
| Quota check latency | <10µs p99 | Bloom filter hit path |
| Event ingestion throughput | >100K events/s | Batch mode |
| Aggregation latency | <100ms | Per billing period |
| Invoice generation | <1s | Per subscription |

---

## 8. Open Questions

1. Should we support real-time streaming aggregation or batch-only?
2. What's the event retention policy (hot vs cold storage)?
3. Should agents be able to query their own usage?
4. How do we handle clock skew between event timestamp and server time?

---

## 9. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
