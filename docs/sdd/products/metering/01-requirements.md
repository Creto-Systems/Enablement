---
status: draft
author: Metering Lead
created: 2024-12-25
updated: 2024-12-25
reviewers: []
issue: "#2 [MTR] Extract Metering product requirements"
oss_reference: getlago/lago
---

# SDD-MTR-01: Metering Requirements Specification

## Purpose

This document specifies the functional and non-functional requirements for **creto-metering**, the usage-based billing and quota enforcement product for AI agents. It defines what the system must do, how well it must do it, and the constraints under which it operates.

## Scope

**In Scope:**
- Event ingestion with NHI attribution
- Idempotency and deduplication
- Real-time quota enforcement
- Usage aggregation (COUNT, SUM, UNIQUE_COUNT, MAX)
- Pricing model calculations
- Invoice generation with delegation chain attribution
- Integration with Authorization service (inline quota checks)
- Integration with Audit service (immutable event logging)

**Out of Scope:**
- Payment processing (delegated to Stripe/external gateway)
- UI/dashboard implementation (separate frontend service)
- Credit card storage (PCI compliance delegated)
- Tax calculation (third-party integration)
- Dunning and collections (billing workflow management)

---

## 1. Functional Requirements

### 1.1 Event Ingestion

#### FR-MTR-001: BillableEvent Schema

**Priority:** CRITICAL

**Description:** The system SHALL accept billable events conforming to the BillableEvent schema with the following mandatory fields:

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| `idempotency_key` | String (UUID) | Unique identifier for deduplication | `550e8400-e29b-41d4-a716-446655440000` |
| `agent_nhi` | AgentIdentity | Cryptographic agent identity | `agent:nhi:ed25519:abc123...` |
| `delegation_chain` | Vec\<AgentIdentity\> | Full lineage to root human principal | `[agent_b, agent_a, human_c]` |
| `event_type` | String | Billable metric code | `"api_call"`, `"llm_token"`, `"vector_query"` |
| `timestamp` | Timestamp | Consensus-ordered event time | `2024-12-25T12:00:00Z` |
| `properties` | Properties (JSON) | Custom dimensions for aggregation | `{"tokens": 1500, "model": "gpt-4"}` |

**Acceptance Criteria:**
- [ ] System accepts events via HTTP/gRPC API
- [ ] All mandatory fields validated before acceptance
- [ ] Invalid events rejected with descriptive error codes
- [ ] Event schema versioning supported (forward/backward compatibility)
- [ ] Properties field supports nested JSON up to 3 levels deep

**Use Case:** Agent generates embeddings for 1,000 documents
```json
{
  "idempotency_key": "embed-batch-001",
  "agent_nhi": "agent:nhi:ed25519:embed-worker-42",
  "delegation_chain": ["agent:scheduler", "human:ops-team"],
  "event_type": "embedding_generation",
  "timestamp": "2024-12-25T10:30:00Z",
  "properties": {
    "document_count": 1000,
    "embedding_model": "text-embedding-ada-002",
    "dimensions": 1536,
    "total_tokens": 250000
  }
}
```

---

#### FR-MTR-002: Idempotency Handling

**Priority:** CRITICAL

**Description:** The system SHALL deduplicate events using the `idempotency_key` to prevent double-billing.

**Behavior:**
1. On event ingestion, check if `idempotency_key` exists in storage
2. If exists and event is identical: Return `202 Accepted` with existing `event_id` (no-op)
3. If exists and event differs: Return `409 Conflict` (same key, different data)
4. If new: Store event, return `201 Created` with new `event_id`

**Idempotency Window:** 7 days (configurable)

**Acceptance Criteria:**
- [ ] Duplicate submissions with identical data return 202
- [ ] Duplicate key with different data returns 409
- [ ] Idempotency check completes in <5ms (p99)
- [ ] Keys older than window can be reused
- [ ] Idempotency state survives service restarts

**Use Case:** Agent retries event submission due to network timeout
```
Attempt 1: POST /events → 500 Internal Server Error (network timeout)
Attempt 2: POST /events (same idempotency_key) → 202 Accepted (no duplicate charge)
```

---

#### FR-MTR-003: Batch Event Ingestion

**Priority:** HIGH

**Description:** The system SHALL support batch ingestion of up to 1,000 events in a single request for high-throughput scenarios.

**Batch Behavior:**
- Accept array of events in request body
- Process events in parallel (order not guaranteed)
- Return batch response with per-event status codes
- Partial success allowed (some events succeed, some fail)

**Acceptance Criteria:**
- [ ] Batch endpoint accepts up to 1,000 events per request
- [ ] Requests with >1,000 events return `413 Payload Too Large`
- [ ] Response includes per-event success/failure status
- [ ] Batch processing achieves >10,000 events/second throughput
- [ ] Failed events do not block successful events in same batch

**Batch Response Format:**
```json
{
  "batch_id": "batch-123",
  "total": 100,
  "succeeded": 98,
  "failed": 2,
  "results": [
    {"idempotency_key": "event-1", "status": "created", "event_id": "evt_001"},
    {"idempotency_key": "event-2", "status": "duplicate", "event_id": "evt_002"},
    {"idempotency_key": "event-99", "status": "failed", "error": "invalid_agent_nhi"}
  ]
}
```

---

### 1.2 Quota Enforcement

#### FR-MTR-004: Inline Quota Check

**Priority:** CRITICAL

**Description:** The system SHALL provide a `check_quota()` function callable from the Authorization service to enforce spending limits inline (before action execution).

**Interface:**
```rust
pub async fn check_quota(
    agent: &AgentIdentity,
    event_type: &str,
) -> Result<QuotaDecision, QuotaError>

pub enum QuotaDecision {
    Allow { remaining: u64 },
    Deny {
        reason: QuotaExceeded,
        retry_after: Option<Duration>
    }
}
```

**Check Logic:**
1. Identify subscription for `agent` (via owner mapping)
2. Load quota config for `event_type`
3. Query current usage for quota period (hourly/daily/monthly)
4. Compare `usage + 1` against `limit`
5. Return `Allow` if within quota, `Deny` if exceeded

**Acceptance Criteria:**
- [ ] Function completes in <10µs (p99) for cache hits
- [ ] Function completes in <500µs (p99) for cache misses
- [ ] Remaining quota returned in `Allow` response
- [ ] `Deny` response includes retry_after timestamp for periodic quotas
- [ ] Quota checks integrated into Authorization policy evaluation

**Integration Example:**
```rust
// Authorization service calls metering inline
let policy_result = authz.check(agent, "vector_query", resource).await?;

// Internally, Authorization calls:
let quota_decision = metering.check_quota(agent, "vector_query").await?;
match quota_decision {
    QuotaDecision::Allow { remaining } => // Proceed with authorization
    QuotaDecision::Deny { reason, .. } => return Err(AuthzError::QuotaExceeded(reason))
}
```

---

#### FR-MTR-005: Quota Period Types

**Priority:** HIGH

**Description:** The system SHALL support the following quota period types:

| Period | Reset Behavior | Use Case |
|--------|----------------|----------|
| `Hourly` | Resets at top of each hour | Rate limiting, burst protection |
| `Daily` | Resets at midnight UTC | Daily API call limits |
| `Monthly` | Resets on 1st of month | Subscription plan limits |
| `Total` | Never resets | Lifetime usage caps |

**Acceptance Criteria:**
- [ ] Each period type resets usage counters at defined intervals
- [ ] Timezone configurable per subscription (default: UTC)
- [ ] Multiple periods can apply to same event_type (e.g., 1000/hour AND 10000/day)
- [ ] Usage counters accurate across service restarts
- [ ] Period rollover completes in <100ms

---

#### FR-MTR-006: Quota Overflow Actions

**Priority:** MEDIUM

**Description:** The system SHALL support configurable actions when quota is exceeded:

| Action | Behavior |
|--------|----------|
| `Block` | Deny request with `403 Forbidden` |
| `AllowWithOverage` | Allow request, charge overage fee |
| `NotifyOnly` | Allow request, send notification, log warning |

**Acceptance Criteria:**
- [ ] Action configured per quota rule
- [ ] `Block` returns descriptive error with quota details
- [ ] `AllowWithOverage` charges configured overage rate
- [ ] `NotifyOnly` sends notification to subscription owner
- [ ] Overage events logged to Audit with `outcome: allowed_over_quota`

---

### 1.3 Usage Aggregation

#### FR-MTR-007: Aggregation Types

**Priority:** CRITICAL

**Description:** The system SHALL support the following aggregation types (from Lago):

| Type | Calculation | SQL Equivalent | Use Case |
|------|-------------|----------------|----------|
| `COUNT` | Number of events | `COUNT(*)` | API calls, transactions |
| `SUM` | Sum of property value | `SUM(properties->>'field')` | Tokens, bytes, compute-ms |
| `UNIQUE_COUNT` | Distinct values | `COUNT(DISTINCT properties->>'field')` | Active users, unique IPs |
| `MAX` | Maximum value | `MAX(properties->>'field')` | Peak memory, max latency |

**Acceptance Criteria:**
- [ ] Each aggregation type produces correct numerical result
- [ ] Aggregations support filtering by dimensions (e.g., `model=gpt-4`)
- [ ] Aggregations computed incrementally (not full table scan)
- [ ] Results cached for repeated queries within same billing period
- [ ] Aggregation latency <100ms for 1M events

**Example Configurations:**
```yaml
# COUNT: Track total API calls
- metric_code: "api_calls"
  aggregation: COUNT

# SUM: Track total tokens consumed
- metric_code: "llm_tokens"
  aggregation: SUM
  property: "tokens"

# UNIQUE_COUNT: Track distinct models used
- metric_code: "unique_models"
  aggregation: UNIQUE_COUNT
  property: "model"

# MAX: Track peak concurrent connections
- metric_code: "peak_connections"
  aggregation: MAX
  property: "connection_count"
```

---

#### FR-MTR-008: Aggregation Windows

**Priority:** HIGH

**Description:** The system SHALL aggregate usage across configurable time windows:

| Window | Use Case |
|--------|----------|
| Real-time (rolling 1 hour) | Quota enforcement |
| Daily | Daily usage reports |
| Monthly | Billing period invoices |
| Custom range | Analytics, reporting |

**Acceptance Criteria:**
- [ ] Real-time aggregations update within 1 second of event ingestion
- [ ] Historical aggregations precomputed and cached
- [ ] Custom range queries support up to 1 year of data
- [ ] Timezone-aware window boundaries

---

### 1.4 Pricing Models

#### FR-MTR-009: Flat Fee Pricing

**Priority:** HIGH

**Description:** The system SHALL support flat fee pricing: fixed charge per billing period, independent of usage.

**Configuration:**
```yaml
pricing_model: flat
amount: 99.00
currency: USD
```

**Calculation:** `charge = $99.00` (regardless of usage)

**Acceptance Criteria:**
- [ ] Flat fee charged at period start
- [ ] No usage tracking required for billing
- [ ] Prorated for partial periods (configurable)

---

#### FR-MTR-010: Per-Unit Pricing

**Priority:** CRITICAL

**Description:** The system SHALL support per-unit pricing: linear cost per unit of usage.

**Configuration:**
```yaml
pricing_model: per_unit
unit_price: 0.002
currency: USD
```

**Calculation:** `charge = usage × $0.002`

**Example:** 10,000 API calls × $0.002 = $20.00

**Acceptance Criteria:**
- [ ] Linear calculation for any usage amount
- [ ] Supports fractional units (e.g., $0.0001 per token)
- [ ] Handles high precision decimal arithmetic (no floating-point errors)

---

#### FR-MTR-011: Tiered Graduated Pricing

**Priority:** HIGH

**Description:** The system SHALL support graduated tier pricing where each tier is priced independently.

**Configuration:**
```yaml
pricing_model: tiered_graduated
tiers:
  - up_to: 1000
    unit_price: 0.01
  - up_to: 10000
    unit_price: 0.008
  - up_to: null  # unlimited
    unit_price: 0.005
```

**Calculation:**
- First 1,000 units: 1,000 × $0.01 = $10.00
- Next 9,000 units: 9,000 × $0.008 = $72.00
- Remaining units: N × $0.005

**Example:** 15,000 units = $10 + $72 + (5,000 × $0.005) = $107.00

**Acceptance Criteria:**
- [ ] Each tier priced at its own rate
- [ ] Supports unlimited tiers (null upper bound)
- [ ] Tier boundaries inclusive (up_to: 1000 includes the 1,000th unit)

---

#### FR-MTR-012: Tiered Volume Pricing

**Priority:** HIGH

**Description:** The system SHALL support volume tier pricing where ALL units are priced at the tier reached.

**Configuration:**
```yaml
pricing_model: tiered_volume
tiers:
  - up_to: 1000
    unit_price: 0.01
  - up_to: 10000
    unit_price: 0.008
  - up_to: null
    unit_price: 0.005
```

**Calculation:** Entire volume priced at the tier rate reached.

**Example:** 15,000 units → Tier 3 ($0.005/unit) → 15,000 × $0.005 = $75.00

**Acceptance Criteria:**
- [ ] All units priced at reached tier rate (not graduated)
- [ ] Tier jumps create pricing discontinuities (expected behavior)
- [ ] Volume discounts incentivize higher usage

---

#### FR-MTR-013: Package Pricing

**Priority:** MEDIUM

**Description:** The system SHALL support package pricing: fixed price for N units, overage charged separately.

**Configuration:**
```yaml
pricing_model: package
package_size: 1000
package_price: 50.00
overage_unit_price: 0.06
```

**Calculation:**
- Base package (up to 1,000 units): $50.00
- Overage (per unit over 1,000): $0.06

**Example:** 1,200 units = $50.00 + (200 × $0.06) = $62.00

**Acceptance Criteria:**
- [ ] Base package charged even if zero usage (configurable)
- [ ] Overage starts at package_size + 1
- [ ] Multiple packages allowed (e.g., 2× package = 2,000 units for $100)

---

#### FR-MTR-014: Prepaid Credits

**Priority:** MEDIUM

**Description:** The system SHALL support prepaid credit wallets with auto-top-up.

**Configuration:**
```yaml
pricing_model: prepaid_credits
credit_rate: 1.0  # $1 = 1 credit
initial_balance: 1000.0
auto_topup:
  threshold: 100.0  # Top up when < 100 credits
  amount: 500.0     # Add 500 credits
```

**Credit Consumption:**
- Event cost calculated from per-unit or tiered pricing
- Credits deducted from wallet balance in real-time
- Negative balance = service suspension (configurable)

**Acceptance Criteria:**
- [ ] Credits deducted atomically (no race conditions)
- [ ] Auto-top-up triggers when balance < threshold
- [ ] Wallet balance queryable in real-time
- [ ] Credit expiration supported (optional)

---

### 1.5 Invoice Generation

#### FR-MTR-015: Billing Period Invoice

**Priority:** CRITICAL

**Description:** The system SHALL generate invoices at the end of each billing period with the following structure:

**Invoice Structure:**
```json
{
  "invoice_id": "inv_2024_12_001",
  "subscription_id": "sub_abc123",
  "period_start": "2024-12-01T00:00:00Z",
  "period_end": "2024-12-31T23:59:59Z",
  "line_items": [
    {
      "description": "API Calls (per-unit pricing)",
      "metric_code": "api_calls",
      "quantity": 15000,
      "unit_price": 0.002,
      "amount": 30.00
    }
  ],
  "subtotal": 30.00,
  "tax": 2.70,
  "total": 32.70,
  "attribution": { /* cost breakdown by agent */ },
  "status": "draft"
}
```

**Acceptance Criteria:**
- [ ] Invoices generated automatically at period close
- [ ] All billable metrics included as line items
- [ ] Amounts match aggregated usage × pricing model
- [ ] Attribution breakdown included (see FR-MTR-016)
- [ ] Invoice status workflow: `draft → issued → paid → void`

---

#### FR-MTR-016: Cost Attribution by Agent

**Priority:** HIGH

**Description:** The system SHALL provide cost attribution breakdowns showing which agents (in delegation chains) consumed resources.

**Attribution Structure:**
```json
{
  "by_agent": {
    "agent:embed-worker-1": 15.00,
    "agent:embed-worker-2": 10.00,
    "agent:scheduler": 5.00
  },
  "by_dimension": {
    "model": {
      "gpt-4": 20.00,
      "gpt-3.5-turbo": 10.00
    },
    "region": {
      "us-east-1": 18.00,
      "eu-west-1": 12.00
    }
  }
}
```

**Use Case:** Multi-tenant cost allocation
- Organization has 10 teams, each with autonomous agents
- CFO wants to charge back costs to teams based on their agents' usage
- Attribution shows: Team A agents consumed $500, Team B consumed $300

**Acceptance Criteria:**
- [ ] Attribution computed from delegation chains in events
- [ ] Costs allocated to all agents in chain (not just leaf agent)
- [ ] Dimension-based attribution supports custom properties
- [ ] Attribution queryable without generating full invoice

---

### 1.6 Integration Requirements

#### FR-MTR-017: Authorization Service Integration

**Priority:** CRITICAL

**Description:** The system SHALL integrate with `creto-authz` for inline quota enforcement during policy evaluation.

**Integration Flow:**
1. Agent requests action → Authorization service
2. Authorization evaluates policy → `ALLOW` or `DENY`
3. **Before returning `ALLOW`:** Authorization calls `metering.check_quota()`
4. If quota exceeded: Override to `DENY` with reason `quota_exceeded`
5. If quota OK: Return `ALLOW` with remaining quota in metadata

**Acceptance Criteria:**
- [ ] Quota check adds <20µs to Authorization latency (total: 168ns + 20µs)
- [ ] Quota denial overrides policy ALLOW (security boundary)
- [ ] Authorization response includes remaining quota metadata
- [ ] Integration resilient to metering service unavailability (fail-open vs fail-closed configurable)

---

#### FR-MTR-018: Audit Service Integration

**Priority:** HIGH

**Description:** The system SHALL log all metering operations to `creto-audit` for immutable compliance trail.

**Events to Audit:**
- Event ingestion (who, what, when, properties)
- Quota checks (agent, result, remaining)
- Quota exceeded denials (agent, limit, usage)
- Invoice generation (subscription, period, total)
- Pricing model changes (who changed, old/new config)

**Acceptance Criteria:**
- [ ] All events logged with full delegation chain
- [ ] Audit records include event signatures (cryptographic proof)
- [ ] Logs queryable for compliance reports (SOX 404, HIPAA)
- [ ] Audit writes do not block metering operations (async)

---

## 2. Non-Functional Requirements

### 2.1 Performance

#### NFR-MTR-001: Event Ingestion Throughput

**Priority:** CRITICAL

**Requirement:** The system SHALL ingest >10,000 events per second (sustained) via batch API.

**Measurement:**
- Benchmark with 10,000 event batches
- Measure end-to-end latency (request received → database committed)
- Target: p99 latency <500ms for batch ingestion

**Acceptance Criteria:**
- [ ] Sustained 10K events/sec for 1 hour without degradation
- [ ] Horizontal scaling: 2× instances = 2× throughput
- [ ] Zero event loss under load (durability)

---

#### NFR-MTR-002: Quota Check Latency (Cache Hit)

**Priority:** CRITICAL

**Requirement:** Quota checks SHALL complete in <10µs (p99) when quota data is in memory cache.

**Measurement:**
- In-memory bloom filter or hash map lookup
- Target: <10µs for 99th percentile

**Implementation Strategy:**
- Preload "hot" quotas into memory at startup
- Use probabilistic data structures (bloom filter) for fast negative checks
- Update cache on quota config changes (max 1 second staleness)

**Acceptance Criteria:**
- [ ] p50 latency <2µs
- [ ] p99 latency <10µs
- [ ] p99.9 latency <50µs
- [ ] Cache hit rate >95% under normal load

---

#### NFR-MTR-003: Quota Check Latency (Cache Miss)

**Priority:** HIGH

**Requirement:** Quota checks SHALL complete in <500µs (p99) when quota data requires database lookup.

**Measurement:**
- Cold path: Redis lookup + usage query
- Target: <500µs for 99th percentile

**Acceptance Criteria:**
- [ ] p50 latency <100µs
- [ ] p99 latency <500µs
- [ ] Cache population completes in <1ms
- [ ] Subsequent checks hit cache (warm path)

---

#### NFR-MTR-004: Aggregation Performance

**Priority:** HIGH

**Requirement:** Usage aggregations SHALL complete in <100ms for up to 1 million events per billing period.

**Measurement:**
- Query: `SELECT SUM(properties->>'tokens') FROM events WHERE period = 'current_month'`
- Dataset: 1M events
- Target: <100ms p99 latency

**Implementation Strategy:**
- Precompute rolling aggregates (materialized views)
- Incremental updates on event ingestion
- Index on (subscription_id, period, event_type)

**Acceptance Criteria:**
- [ ] 1M events: <100ms
- [ ] 10M events: <500ms
- [ ] Aggregation accuracy 100% (no approximation unless configured)

---

#### NFR-MTR-005: Invoice Generation Performance

**Priority:** MEDIUM

**Requirement:** Invoice generation SHALL complete in <1 second per subscription.

**Measurement:**
- End-to-end: aggregate usage → apply pricing → generate PDF (optional)
- Target: <1s for subscription with 100 line items

**Acceptance Criteria:**
- [ ] Simple invoices (1-5 line items): <200ms
- [ ] Complex invoices (50+ line items): <1s
- [ ] Batch invoice generation: 100 subscriptions in <30s

---

### 2.2 Scalability

#### NFR-MTR-006: Horizontal Scalability

**Priority:** HIGH

**Requirement:** The system SHALL scale horizontally to support 1,000,000 subscriptions.

**Scaling Characteristics:**
- Event ingestion: Stateless, partition by subscription_id
- Quota checks: Shared cache (Redis Cluster), read replicas
- Aggregations: Partition by time window (daily/monthly)

**Acceptance Criteria:**
- [ ] Linear throughput scaling up to 10 instances
- [ ] No single point of contention (SPOF)
- [ ] Database partitioning by subscription_id
- [ ] Cache sharding by subscription_id

---

#### NFR-MTR-007: Storage Scalability

**Priority:** HIGH

**Requirement:** The system SHALL support 10 billion events (10TB+) with tiered storage.

**Storage Tiers:**
- **Hot (0-7 days):** SSD, full indexing, <10ms queries
- **Warm (8-90 days):** SSD, limited indexing, <100ms queries
- **Cold (>90 days):** S3/object storage, archival, batch queries only

**Acceptance Criteria:**
- [ ] Automated tiering based on event age
- [ ] Hot tier: 7-day retention, full query performance
- [ ] Warm tier: 90-day retention, acceptable query performance
- [ ] Cold tier: 7-year retention (compliance), batch export only

---

### 2.3 Availability

#### NFR-MTR-008: Service Availability

**Priority:** HIGH

**Requirement:** The system SHALL maintain 99.9% uptime (8.76 hours downtime/year).

**Failure Scenarios:**
- Single instance failure: Auto-restart, <30s recovery
- Database failure: Replica promotion, <60s recovery
- Cache failure: Degrade to database lookups, performance impact only

**Acceptance Criteria:**
- [ ] Health checks every 10 seconds
- [ ] Automated failover for database replicas
- [ ] Circuit breaker for degraded dependencies
- [ ] Graceful degradation: quota checks fail-open (configurable)

---

#### NFR-MTR-009: Data Durability

**Priority:** CRITICAL

**Requirement:** The system SHALL guarantee 99.999% durability for ingested events (no data loss).

**Durability Strategy:**
- Synchronous replication to 2+ database replicas
- Write-ahead logging (WAL) before response
- Daily snapshots to object storage

**Acceptance Criteria:**
- [ ] Events committed to persistent storage before 201 response
- [ ] Replica lag <1 second under normal load
- [ ] Point-in-time recovery (PITR) up to 30 days

---

### 2.4 Security

#### NFR-MTR-010: Cryptographic Event Signing

**Priority:** HIGH

**Requirement:** All billable events SHALL be cryptographically signed by the originating agent using ML-DSA (post-quantum signature).

**Signature Verification:**
- Verify `agent_nhi` signature on event ingestion
- Reject events with invalid or missing signatures
- Store signature with event for audit trail

**Acceptance Criteria:**
- [ ] Signature verification adds <1ms to ingestion latency
- [ ] Invalid signatures logged and rejected with 401 Unauthorized
- [ ] Signature algorithm: ML-DSA-65 (NIST PQC standard)
- [ ] Signature includes: event_type, properties, timestamp, delegation_chain

---

#### NFR-MTR-011: Authorization Integration

**Priority:** CRITICAL

**Requirement:** All metering API calls SHALL require Authorization service approval.

**Authorization Checks:**
- Event ingestion: `can agent submit events for subscription?`
- Quota queries: `can agent query quota for subscription?`
- Invoice access: `can human access invoices for subscription?`

**Acceptance Criteria:**
- [ ] 100% of API endpoints enforce authorization
- [ ] Unauthorized requests return 403 Forbidden
- [ ] Authorization latency <1ms (168ns policy check)

---

### 2.5 Compliance

#### NFR-MTR-012: SOX 404 Compliance

**Priority:** HIGH

**Requirement:** The system SHALL provide immutable audit trails for all billing operations (Sarbanes-Oxley Section 404).

**Audit Requirements:**
- Tamper-proof event storage (append-only)
- Full delegation chain for accountability
- Cryptographic signatures for non-repudiation
- Queryable audit logs for compliance reports

**Acceptance Criteria:**
- [ ] All events logged to `creto-audit` with Merkle anchoring
- [ ] Audit logs include: who, what, when, why (delegation chain)
- [ ] Audit trail retention: 7 years
- [ ] Compliance reports generated in <1 minute

---

#### NFR-MTR-013: HIPAA Compliance (Usage Audit)

**Priority:** MEDIUM

**Requirement:** For healthcare workloads, the system SHALL log usage of protected health information (PHI) for HIPAA compliance.

**HIPAA-Specific Fields:**
- `phi_accessed`: Boolean flag in properties
- `access_justification`: Required for PHI events
- `patient_id`: Anonymized identifier (hashed)

**Acceptance Criteria:**
- [ ] PHI events flagged in properties
- [ ] PHI access logged to audit with justification
- [ ] PHI usage queryable for compliance audits
- [ ] Patient consent verification (via Authorization)

---

## 3. User Stories

### 3.1 Agent Developer Stories

#### US-MTR-001: Track LLM Token Usage

**As an** AI agent developer
**I want** my agents to automatically track LLM token consumption
**So that** I can bill customers accurately and optimize model selection

**Acceptance Criteria:**
- [ ] Agent submits event with `event_type: "llm_tokens"` and `properties.tokens`
- [ ] System aggregates with `SUM` aggregation type
- [ ] Invoice includes line item: "LLM Tokens: 1,500,000 tokens × $0.002 = $3,000"

**Example Implementation:**
```rust
// Agent code (after LLM call)
metering.ingest_event(EventRequest {
    idempotency_key: format!("llm-call-{}", request_id),
    agent_nhi: self.identity(),
    delegation_chain: self.full_chain(),
    event_type: "llm_tokens",
    timestamp: None, // Use current time
    properties: json!({
        "tokens": response.usage.total_tokens,
        "model": "gpt-4",
        "prompt_tokens": response.usage.prompt_tokens,
        "completion_tokens": response.usage.completion_tokens,
    }),
}).await?;
```

---

#### US-MTR-002: Enforce API Rate Limits

**As a** platform operator
**I want** to enforce hourly API call quotas per customer
**So that** I prevent abuse and ensure fair resource allocation

**Acceptance Criteria:**
- [ ] Configure quota: `api_calls`, limit: 1000/hour, action: Block
- [ ] Agent makes 1,001st call in same hour → `403 Forbidden`
- [ ] Error response includes: `quota_exceeded`, `retry_after: 3600s`
- [ ] Quota resets at top of next hour

**Configuration:**
```yaml
subscription:
  quotas:
    - event_type: "api_calls"
      limit: 1000
      period: hourly
      action: block
```

---

### 3.2 Business Analyst Stories

#### US-MTR-003: Cost Attribution by Team

**As a** CFO
**I want** to see cost breakdown by development team
**So that** I can charge back cloud costs to department budgets

**Acceptance Criteria:**
- [ ] Each team's agents have unique NHIs (e.g., `team-a:agent-1`, `team-b:agent-2`)
- [ ] Invoice attribution shows: Team A: $5,000, Team B: $3,000
- [ ] Attribution queryable without waiting for month-end invoice

**Query API:**
```rust
let attribution = metering.get_attribution(
    subscription_id,
    BillingPeriod::current_month()
).await?;

println!("Team A: ${}", attribution.by_agent.get("team-a:*").sum());
```

---

#### US-MTR-004: Multi-Model Pricing

**As a** SaaS provider
**I want** to charge different rates for different LLM models
**So that** I can pass through vendor costs accurately

**Acceptance Criteria:**
- [ ] Configure per-unit pricing per model dimension
- [ ] GPT-4: $0.03/1K tokens, GPT-3.5: $0.001/1K tokens
- [ ] Invoice shows separate line items per model
- [ ] Attribution breakdown by `properties.model`

**Configuration:**
```yaml
pricing_rules:
  - event_type: "llm_tokens"
    dimension: "model"
    rates:
      "gpt-4": 0.00003  # per token
      "gpt-3.5-turbo": 0.000001
      "claude-3": 0.000015
```

---

### 3.3 Compliance Officer Stories

#### US-MTR-005: Audit Billing Disputes

**As a** compliance officer
**I want** to cryptographically verify billing events
**So that** I can resolve customer disputes with proof

**Acceptance Criteria:**
- [ ] Retrieve event by `event_id` with signature
- [ ] Verify signature against agent's public key
- [ ] Delegation chain proves which human authorized the agent
- [ ] Audit trail shows: when event submitted, when quota checked, when invoice generated

**Audit Query:**
```rust
let event = audit.get_event("evt_disputed_001").await?;

// Verify cryptographic signature
let signature_valid = crypto.verify(
    &event.signature,
    &event.agent_nhi.public_key,
    &event.canonical_hash()
)?;

// Show delegation chain
println!("Event signed by: {}", event.agent_nhi);
println!("Delegated from: {:?}", event.delegation_chain);
println!("Root principal: {}", event.delegation_chain.last());
```

---

## 4. Edge Cases and Error Handling

### 4.1 Clock Skew

**Scenario:** Agent submits event with timestamp 5 minutes in the future (clock skew).

**Handling:**
- Accept events with timestamps within ±10 minutes of server time
- Events outside window: Reject with `400 Bad Request`, `error: "timestamp_skew"`
- Use consensus-ordered timestamp for billing (not agent-provided timestamp)

---

### 4.2 Duplicate Events with Different Data

**Scenario:** Same `idempotency_key`, different `properties`.

**Handling:**
- Return `409 Conflict`
- Error message: "Event with this idempotency_key already exists with different data"
- Include hash of existing event in response for debugging

---

### 4.3 Quota Exceeded Mid-Batch

**Scenario:** Batch of 100 events, quota exceeded on event #50.

**Handling:**
- Process events 1-49: Success
- Process event 50: Quota exceeded, mark as failed
- Process events 51-100: Continue processing (independent events)
- Return batch response with 49 successes, 1 quota failure, 50 successes

---

### 4.4 Subscription Cancelled Mid-Period

**Scenario:** Customer cancels on day 15 of 30-day billing period.

**Handling:**
- Generate prorated invoice for days 1-15
- Disable quota checks immediately (no new events accepted)
- Existing events already ingested remain billable
- Refund calculation: `(unused_days / total_days) × flat_fee`

---

### 4.5 Pricing Model Change Mid-Period

**Scenario:** Pricing model updated from $0.01/unit to $0.02/unit on day 10.

**Handling:**
- Split invoice into two line items:
  - Line 1: Days 1-9 usage × $0.01
  - Line 2: Days 10-30 usage × $0.02
- Audit log records pricing change with timestamp
- No retroactive repricing (events billed at rate when ingested)

---

## 5. Constraints

### 5.1 Technical Constraints

- **Database:** PostgreSQL 14+ (for JSON property queries)
- **Cache:** Redis 7+ (for quota enforcement)
- **Message Queue:** Optional Kafka for high-throughput ingestion
- **Storage:** S3-compatible object storage for cold events

### 5.2 Dependency Constraints

- **creto-nhi:** Required for agent identity validation
- **creto-authz:** Required for inline quota enforcement
- **creto-audit:** Required for compliance logging
- **creto-crypto:** Required for event signature verification

### 5.3 Regulatory Constraints

- **SOX 404:** 7-year audit retention
- **GDPR:** Right to erasure (event anonymization, not deletion)
- **HIPAA:** PHI access logging
- **PCI DSS:** Out of scope (no payment card data storage)

---

## 6. Success Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Quota check latency (p99) | <10µs | Prometheus histogram |
| Event ingestion throughput | >10K/s | Load testing (k6) |
| Invoice generation time | <1s | End-to-end timer |
| Service uptime | 99.9% | Uptime monitor (Pingdom) |
| Event durability | 99.999% | Zero data loss in 1 year |
| Billing dispute rate | <0.1% | Customer support tickets |

---

## 7. Open Questions

1. **Q:** Should we support real-time streaming aggregation (e.g., Apache Flink) or batch-only?
   **Impact:** Real-time enables live dashboards but adds complexity.

2. **Q:** What's the event retention policy for hot/warm/cold storage?
   **Proposal:** 7 days hot, 90 days warm, 7 years cold (compliance).

3. **Q:** Should agents be able to query their own usage?
   **Impact:** Privacy vs. transparency trade-off.

4. **Q:** How do we handle multi-currency pricing?
   **Proposal:** Store all prices in USD, apply exchange rates at invoice time.

5. **Q:** Should we support usage-based credits (e.g., free tier: 1000 API calls/month)?
   **Proposal:** Yes, implement as "negative pricing" line item.

---

## 8. OSS Reference Mapping

### 8.1 Lago Event Model → Creto BillableEvent

| Lago Field | Creto Field | Difference |
|------------|-------------|------------|
| `transaction_id` | `idempotency_key` | Same concept |
| `external_subscription_id` | `agent_nhi` | Cryptographic identity |
| (none) | `delegation_chain` | **Creto extension** |
| `code` | `event_type` | Same |
| `timestamp` | `timestamp` | Consensus-ordered |
| `properties` | `properties` | Typed JSON |
| (none) | `signature` | **Creto extension** |

### 8.2 Lago Aggregations → Creto Aggregator Trait

- **COUNT, SUM, UNIQUE_COUNT, MAX:** Direct mapping
- **Lago weighted sum:** Future extension
- **Lago recurring count:** Future extension

### 8.3 Lago Pricing Models → Creto PricingModel Trait

- **Flat, Per-Unit, Tiered (Graduated), Tiered (Volume), Package, Prepaid Credits:** All supported
- **Lago percentage pricing:** Future extension
- **Lago custom aggregations:** Future extension

---

## 9. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Metering Lead | Initial requirements extraction from SDD-01 |

---

## 10. Appendix: Lago Reference Links

- **Lago Documentation:** https://getlago.com/docs/guide/events/ingesting-usage
- **Lago GitHub:** https://github.com/getlago/lago
- **Lago Event API:** https://getlago.com/docs/api-reference/events/create
- **Lago Billable Metrics:** https://getlago.com/docs/guide/billable-metrics/overview
- **Lago Pricing Models:** https://getlago.com/docs/guide/plans/charges
