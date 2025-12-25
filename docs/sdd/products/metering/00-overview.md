---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# SDD-MTR-00: Metering Product Overview

## Executive Summary

**creto-metering** is a usage-based billing and quota enforcement system designed for autonomous AI agents. It provides real-time event ingestion, cryptographic billing verification, inline quota enforcement integrated with the Authorization service, and flexible pricing models to support diverse business models in the AI agent economy.

The system combines patterns from Lago (open-source billing engine) with Creto's sovereign primitives (NHI, crypto-agility, consensus-ordering, immutable audit) to create a billing platform that is:

- **Cryptographically verifiable**: Every billable event is signed by the agent that generated it
- **Real-time enforced**: Quota checks complete in <10µs, integrated inline with authorization decisions
- **Highly scalable**: >10,000 events/second ingestion, 1M+ subscriptions, 10B+ events stored
- **Audit-compliant**: Immutable event trail with delegation chain attribution for SOX/HIPAA compliance

## Product Vision

> **Enable transparent, verifiable, and fair billing for AI agents at scale.**

In the Creto Sovereign platform, AI agents are first-class principals with cryptographic identities (NHI). They consume resources (LLM tokens, API calls, vector queries, storage) on behalf of human principals. Traditional billing systems designed for human users fail to provide:

1. **Attribution**: Which agent in a multi-agent workflow consumed resources?
2. **Verifiability**: Can we cryptographically prove an agent generated this charge?
3. **Real-time enforcement**: Can we prevent quota violations before they happen?
4. **Chain accountability**: Can we trace costs to the root human who delegated authority?

**creto-metering** solves these problems by integrating billing primitives directly with the platform's security layer:

- **NHI-based events**: Every billable event includes agent identity + delegation chain
- **Inline quota enforcement**: Authorization service checks quotas before granting access
- **Cryptographic signatures**: Events signed with ML-DSA (post-quantum) for non-repudiation
- **Consensus-ordered timestamps**: Events ordered by distributed consensus, not agent clocks

## Scope

### In Scope

#### Core Billing Engine
- **Event ingestion**: Accept billable events via HTTP/gRPC with idempotency
- **Usage aggregation**: COUNT, SUM, UNIQUE_COUNT, MAX over time windows
- **Pricing models**: Flat fee, per-unit, tiered (graduated/volume), package, prepaid credits
- **Invoice generation**: Period-based invoicing with delegation chain attribution
- **Multi-dimensional billing**: Charge different rates based on event properties (model, region, tier)

#### Quota Enforcement
- **Inline quota checks**: `check_quota()` function callable from Authorization service
- **Multiple period types**: Hourly, daily, monthly, total (lifetime)
- **Overflow actions**: Block, allow with overage fee, notify-only
- **Cache-first architecture**: <10µs latency for hot paths

#### Integration & Security
- **Authorization integration**: Quota checks embedded in policy evaluation (creto-authz)
- **Audit integration**: All events, quota checks, invoices logged immutably (creto-audit)
- **Cryptographic verification**: ML-DSA signatures on events, Merkle anchoring for audit trail
- **Delegation chain tracking**: Full lineage from leaf agent to root human principal

#### Data & Scale
- **High-throughput ingestion**: >10,000 events/second sustained
- **Horizontal scalability**: Support 1M+ subscriptions
- **Tiered storage**: Hot (7d) / Warm (90d) / Cold (7y) with automated lifecycle
- **99.999% durability**: Synchronous replication, zero acceptable data loss

### Out of Scope (Delegated to External Systems)

- **Payment processing**: Stripe/payment gateway integration (webhook-based)
- **UI/dashboard**: Customer-facing billing portal (separate frontend service)
- **Credit card storage**: PCI DSS compliance delegated to payment processor
- **Tax calculation**: Vertex/Avalara/TaxJar integration (third-party)
- **Dunning & collections**: Chargebee/Recurly patterns (future phase)
- **Revenue recognition**: GAAP/IFRS compliance (external accounting system)

## Success Metrics

### Performance Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **Quota check latency (cache hit)** | <10µs (p99) | Prometheus histogram, in-memory benchmark |
| **Quota check latency (cache miss)** | <500µs (p99) | Redis round-trip + DB query |
| **Event ingestion throughput** | >10,000 events/sec | k6 load test, sustained 1 hour |
| **Event ingestion latency** | <500ms (p99) | Batch API end-to-end timing |
| **Aggregation query latency** | <100ms for 1M events | PostgreSQL query plan analysis |
| **Invoice generation time** | <1s per subscription | End-to-end timer with 100 line items |

### Reliability Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **Service uptime** | 99.9% (8.76h/year) | Pingdom/UptimeRobot monitoring |
| **Event durability** | 99.999% (no data loss) | Zero lost events in production for 1 year |
| **Idempotency accuracy** | 100% | Zero double-billing incidents |
| **Quota enforcement accuracy** | 100% | Zero unauthorized over-quota access |

### Business Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **Billing dispute rate** | <0.1% of subscriptions | Customer support ticket analysis |
| **Invoice delivery SLA** | 100% delivered within 24h of period close | Automated monitoring |
| **Attribution accuracy** | 100% of costs traceable to root principal | Delegation chain audit |
| **Compliance audit success** | 100% pass rate | SOX 404 / HIPAA audit reports |

### Scalability Metrics

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| **Subscriptions supported** | 1,000,000+ | Horizontal scaling validation |
| **Events stored** | 10,000,000,000+ | Tiered storage capacity test |
| **Concurrent quota checks** | 100,000 req/sec | Load balancer + cache cluster perf test |
| **Horizontal scaling efficiency** | Linear to 10 instances | Throughput benchmarks |

## Lago Patterns Reference

**creto-metering** adapts proven patterns from Lago (https://getlago.com), the leading open-source billing engine, while integrating Creto's sovereign primitives.

### Event Ingestion (Lago → Creto)

| Lago Concept | Creto Equivalent | Creto Extension |
|--------------|------------------|-----------------|
| `transaction_id` | `idempotency_key` | Same concept: deduplication |
| `external_subscription_id` | `agent_nhi` | **Cryptographic agent identity** |
| `external_customer_id` | `subscription_id` | Lago concept preserved |
| `code` | `event_type` | Same: billable metric code |
| `timestamp` | `timestamp` | **Consensus-ordered** timestamp |
| `properties` (JSON) | `properties` (JSONB) | Same: custom dimensions |
| *(none)* | `delegation_chain` | **Creto extension**: agent lineage |
| *(none)* | `signature` | **Creto extension**: ML-DSA signature |

**Example Lago Event (JSON)**:
```json
{
  "transaction_id": "txn_001",
  "external_subscription_id": "sub_abc",
  "code": "api_call",
  "timestamp": 1703505600,
  "properties": {
    "region": "us-east-1",
    "method": "POST"
  }
}
```

**Example Creto Event (with NHI + Signature)**:
```json
{
  "idempotency_key": "txn_001",
  "agent_nhi": "agent:nhi:ed25519:abc123...",
  "delegation_chain": ["agent:worker", "agent:scheduler", "human:ops-team"],
  "event_type": "api_call",
  "timestamp": "2024-12-25T10:00:00Z",
  "properties": {
    "region": "us-east-1",
    "method": "POST"
  },
  "signature": "ML-DSA-65:base64encodedSig..."
}
```

### Aggregation Types (Lago → Creto)

| Aggregation | Lago SQL | Creto Trait | Use Case |
|-------------|----------|-------------|----------|
| **COUNT** | `COUNT(*)` | `CountAggregator` | API calls, transactions |
| **SUM** | `SUM(properties->>'field')` | `SumAggregator` | LLM tokens, bytes transferred |
| **UNIQUE_COUNT** | `COUNT(DISTINCT ...)` | `UniqueCountAggregator` | Active users, distinct models |
| **MAX** | `MAX(properties->>'field')` | `MaxAggregator` | Peak memory, max latency |
| *(future)* Weighted Sum | Custom | `WeightedSumAggregator` | Complexity-weighted costs |
| *(future)* Recurring Count | Custom | `RecurringCountAggregator` | Active subscription months |

**Implementation Strategy**:
- Each aggregation type implements `trait Aggregator<T>`
- Aggregations computed incrementally on event ingestion (not full table scans)
- Results cached in Redis for repeated queries within billing period
- Materialized views for historical periods (precomputed)

### Pricing Models (Lago → Creto)

#### 1. Flat Fee Pricing
**Lago**: `charge_model: "standard"` with fixed `amount`
**Creto**: `PricingModel::Flat { amount: Decimal }`

**Use Case**: Monthly subscription fee (e.g., $99/month)

```rust
PricingModel::Flat {
    amount: Decimal::from_str("99.00").unwrap(),
    currency: Currency::USD,
    prorated: true, // Prorate for partial months
}
```

#### 2. Per-Unit Pricing
**Lago**: `charge_model: "standard"` with `per_unit_amount`
**Creto**: `PricingModel::PerUnit { unit_price: Decimal }`

**Use Case**: $0.002 per API call

```rust
PricingModel::PerUnit {
    unit_price: Decimal::from_str("0.002").unwrap(),
    currency: Currency::USD,
}
```

**Calculation**: `charge = usage × unit_price`

#### 3. Tiered Graduated Pricing
**Lago**: `charge_model: "graduated"`
**Creto**: `PricingModel::TieredGraduated { tiers: Vec<Tier> }`

**Use Case**: First 1K units at $0.01, next 9K at $0.008, rest at $0.005

```rust
PricingModel::TieredGraduated {
    tiers: vec![
        Tier { up_to: Some(1000), unit_price: Decimal::from_str("0.01").unwrap() },
        Tier { up_to: Some(10000), unit_price: Decimal::from_str("0.008").unwrap() },
        Tier { up_to: None, unit_price: Decimal::from_str("0.005").unwrap() }, // Unlimited
    ],
    currency: Currency::USD,
}
```

**Calculation**: Each tier priced independently, summed.

#### 4. Tiered Volume Pricing
**Lago**: `charge_model: "volume"`
**Creto**: `PricingModel::TieredVolume { tiers: Vec<Tier> }`

**Use Case**: All units priced at tier rate reached

```rust
PricingModel::TieredVolume {
    tiers: vec![
        Tier { up_to: Some(1000), unit_price: Decimal::from_str("0.01").unwrap() },
        Tier { up_to: Some(10000), unit_price: Decimal::from_str("0.008").unwrap() },
        Tier { up_to: None, unit_price: Decimal::from_str("0.005").unwrap() },
    ],
    currency: Currency::USD,
}
```

**Calculation**: If usage = 15,000, price = 15,000 × $0.005 = $75.00

#### 5. Package Pricing
**Lago**: `charge_model: "package"`
**Creto**: `PricingModel::Package { package_size: u64, ... }`

**Use Case**: $50 for 1,000 units, $0.06 per overage unit

```rust
PricingModel::Package {
    package_size: 1000,
    package_price: Decimal::from_str("50.00").unwrap(),
    overage_unit_price: Decimal::from_str("0.06").unwrap(),
    currency: Currency::USD,
}
```

**Calculation**: Base $50 + (usage - 1,000) × $0.06 if usage > 1,000

#### 6. Prepaid Credits
**Lago**: `charge_model: "prepaid_credits"`
**Creto**: `PricingModel::PrepaidCredits { wallet: WalletConfig }`

**Use Case**: Customer buys $1,000 credits, consumption deducted in real-time

```rust
PricingModel::PrepaidCredits {
    credit_rate: Decimal::from_str("1.0").unwrap(), // $1 = 1 credit
    initial_balance: Decimal::from_str("1000.0").unwrap(),
    auto_topup: Some(AutoTopup {
        threshold: Decimal::from_str("100.0").unwrap(), // Top up when < 100 credits
        amount: Decimal::from_str("500.0").unwrap(),
    }),
}
```

**Credit Consumption**: Events deduct credits from wallet balance atomically.

### Idempotency (Lago Pattern)

**Lago Behavior**:
- Client provides `transaction_id` (UUID)
- Server checks if `transaction_id` exists
- If exists + identical data: Return existing event ID (no-op)
- If exists + different data: Return 409 Conflict
- If new: Create event, return 201 Created

**Creto Implementation**:
```rust
pub async fn ingest_event(&self, req: EventRequest) -> Result<EventResponse, Error> {
    // 1. Check idempotency cache (Redis)
    if let Some(existing) = self.idempotency_cache.get(&req.idempotency_key).await? {
        if existing.hash == req.canonical_hash() {
            return Ok(EventResponse::Accepted(existing.event_id)); // 202 Accepted
        } else {
            return Err(Error::Conflict { // 409 Conflict
                message: "Event with same idempotency_key has different data".into(),
                existing_hash: existing.hash,
            });
        }
    }

    // 2. Verify cryptographic signature
    self.verify_agent_signature(&req)?;

    // 3. Store event + idempotency record atomically
    let event_id = self.storage.create_event(req).await?;
    self.idempotency_cache.set(&req.idempotency_key, event_id, req.canonical_hash()).await?;

    // 4. Trigger async: aggregation update, quota refresh, audit log
    self.post_ingestion_pipeline.trigger(event_id).await?;

    Ok(EventResponse::Created(event_id)) // 201 Created
}
```

**Idempotency Window**: 7 days (configurable). After window expires, keys can be reused.

## Architecture Principles

### 1. Cache-First Quota Enforcement

**Problem**: Quota checks must be <10µs to not block Authorization (target: 168ns + quota overhead).

**Solution**: Multi-tier caching strategy

```
┌─────────────────────────────────────────────────────────────┐
│ Authorization Policy Evaluation (168ns)                     │
│  └─> check_quota(agent, event_type) ─────┐                 │
└───────────────────────────────────────────┼─────────────────┘
                                            ↓
                    ┌───────────────────────────────────────┐
                    │ QuotaEnforcer (Metering)              │
                    │  1. In-memory bloom filter (<2µs)    │
                    │     If "definitely no quota" → ALLOW  │
                    │  2. In-memory hash map (<5µs)         │
                    │     If quota cached → return result   │
                    │  3. Redis lookup (<200µs)             │
                    │     Cache miss → fetch from DB        │
                    │  4. PostgreSQL query (<500µs)         │
                    │     Cold path → update cache          │
                    └───────────────────────────────────────┘
```

**Cache Layers**:
1. **Bloom filter** (probabilistic): Fast negative check ("this agent definitely has no quota for this event type")
2. **In-memory hash map**: Hot quotas for frequently-checked agents
3. **Redis cluster**: Shared cache across instances, sharded by subscription_id
4. **PostgreSQL**: Source of truth, partitioned by time window

**Cache Invalidation**: On quota config change, purge relevant cache keys (<1s staleness allowed).

### 2. Incremental Aggregation

**Problem**: Aggregating 1M events via `SELECT SUM(...)` every query is slow.

**Solution**: Materialized views + incremental updates

```sql
-- Materialized view: hourly aggregates (precomputed)
CREATE MATERIALIZED VIEW usage_hourly AS
SELECT
    subscription_id,
    event_type,
    date_trunc('hour', timestamp) AS hour,
    COUNT(*) AS event_count,
    SUM((properties->>'tokens')::bigint) AS total_tokens,
    COUNT(DISTINCT properties->>'model') AS unique_models,
    MAX((properties->>'latency_ms')::int) AS max_latency
FROM events
GROUP BY subscription_id, event_type, date_trunc('hour', timestamp);

-- On event ingestion, update incrementally
INSERT INTO usage_hourly (subscription_id, event_type, hour, event_count, total_tokens)
VALUES (?, ?, date_trunc('hour', ?), 1, ?)
ON CONFLICT (subscription_id, event_type, hour)
DO UPDATE SET
    event_count = usage_hourly.event_count + 1,
    total_tokens = usage_hourly.total_tokens + EXCLUDED.total_tokens;
```

**Refresh Strategy**:
- **Real-time aggregates**: Updated on event ingestion (hot path)
- **Historical aggregates**: Precomputed nightly (batch job)
- **Custom range queries**: Combine materialized views (fast) + raw events (slow)

### 3. Tiered Storage Lifecycle

**Problem**: 10B events × 1KB average = 10TB. Storing all on hot SSD is expensive.

**Solution**: Automated lifecycle transitions

| Tier | Age | Storage | Indexing | Query Latency | Retention | Cost |
|------|-----|---------|----------|---------------|-----------|------|
| **Hot** | 0-7 days | PostgreSQL (SSD) | Full (all columns) | <10ms | Active billing | $$$$ |
| **Warm** | 8-90 days | PostgreSQL (cheaper SSD) | Partial (time + sub_id) | <100ms | Recent history | $$ |
| **Cold** | >90 days | S3/Glacier | None (full scan) | Batch only | Compliance (7y) | $ |

**Lifecycle Policy**:
```sql
-- Automated partition rotation (pg_partman)
SELECT partman.create_parent('public.events', 'timestamp', 'native', 'daily');

-- Weekly job: Archive warm → cold
INSERT INTO s3_events (SELECT * FROM events WHERE timestamp < now() - interval '90 days');
DELETE FROM events WHERE timestamp < now() - interval '90 days';

-- Compliance: Keep cold tier for 7 years (SOX 404)
```

### 4. Delegation Chain Attribution

**Problem**: In multi-agent workflows, which agent should be billed?

**Answer**: All agents in the chain, proportionally allocated.

**Example Workflow**:
```
Human (Alice) → Agent (Scheduler) → Agent (Worker-1) → LLM API call (1000 tokens)
                                  ↘ Agent (Worker-2) → LLM API call (500 tokens)
```

**Event 1**:
```json
{
  "agent_nhi": "agent:worker-1",
  "delegation_chain": ["agent:scheduler", "human:alice"],
  "event_type": "llm_tokens",
  "properties": {"tokens": 1000}
}
```

**Event 2**:
```json
{
  "agent_nhi": "agent:worker-2",
  "delegation_chain": ["agent:scheduler", "human:alice"],
  "event_type": "llm_tokens",
  "properties": {"tokens": 500}
}
```

**Attribution Report**:
```json
{
  "by_agent": {
    "agent:worker-1": {"tokens": 1000, "cost": "$2.00"},
    "agent:worker-2": {"tokens": 500, "cost": "$1.00"},
    "agent:scheduler": {"tokens": 0, "cost": "$0.00"} // Coordinator, no direct usage
  },
  "by_human": {
    "human:alice": {"tokens": 1500, "cost": "$3.00"} // Root principal pays all
  }
}
```

**Use Case**: Multi-tenant cost allocation (charge back costs to teams/projects based on their agents).

## Key Design Decisions

### Decision 1: Inline Quota Enforcement (vs. Post-Hoc)

**Options Considered**:
1. **Post-hoc billing**: Let agents consume resources, bill at month-end (traditional SaaS)
2. **Inline enforcement**: Check quota before granting access (real-time)

**Decision**: Inline enforcement via Authorization integration

**Rationale**:
- Prevents "bill shock" from runaway agents
- Enforces spending limits in real-time
- Enables quota-as-policy (deny access when budget exhausted)
- Aligns with Zero Trust model (every action authorized)

**Trade-off**: Adds <10µs latency to Authorization path (acceptable given 168ns baseline).

### Decision 2: Consensus-Ordered Timestamps (vs. Agent-Provided)

**Options Considered**:
1. **Agent-provided timestamps**: Trust agent's clock (Lago pattern)
2. **Server timestamps**: Use ingestion server's clock
3. **Consensus-ordered**: Distributed timestamp from consensus layer

**Decision**: Consensus-ordered timestamps

**Rationale**:
- Prevents timestamp manipulation (agent gaming billing by backdating events)
- Ensures global ordering across distributed ingestion fleet
- Aligns with Creto's consensus-based architecture
- Enables Byzantine fault tolerance (agents can't disagree on event order)

**Trade-off**: Requires integration with consensus layer (creto-consensus), adds ~5ms latency.

### Decision 3: Cryptographic Event Signatures (vs. Trust-Based)

**Options Considered**:
1. **Trust agent identity**: Assume agent_nhi is correct (traditional)
2. **Require signatures**: Agent must sign event with NHI private key

**Decision**: Require ML-DSA signatures on all events

**Rationale**:
- Prevents event injection attacks (malicious agent submitting events for other agents)
- Enables non-repudiation (agent can't deny generating a charge)
- Supports compliance audits (cryptographic proof for billing disputes)
- Future-proof with post-quantum signatures (ML-DSA-65)

**Trade-off**: Adds ~1ms signature verification latency (acceptable for billing accuracy).

### Decision 4: PostgreSQL + JSONB (vs. Timeseries DB)

**Options Considered**:
1. **PostgreSQL with JSONB**: Relational DB + flexible properties
2. **TimescaleDB**: Specialized timeseries DB
3. **ClickHouse**: Columnar OLAP DB
4. **InfluxDB**: Timeseries-only

**Decision**: PostgreSQL 14+ with JSONB properties

**Rationale**:
- Lago's proven architecture (battle-tested)
- JSONB supports flexible event properties (no schema migration for new dimensions)
- Excellent query planner for complex aggregations
- GIN indexes on JSONB fields for fast property filtering
- Native partitioning for tiered storage
- ACID guarantees for billing accuracy

**Trade-off**: Not optimized for pure timeseries workloads (acceptable given mixed query patterns).

## Risks & Mitigations

### Risk 1: Quota Check Latency Impact on Authorization

**Risk**: If quota checks are slow, they degrade Authorization service performance.

**Probability**: Medium
**Impact**: High (breaks Authorization's <1ms SLA)

**Mitigation**:
1. **Cache-first architecture**: 95%+ cache hit rate keeps p99 latency <10µs
2. **Fail-open mode**: Configurable fallback (if metering unavailable, allow requests but log)
3. **Circuit breaker**: Disable quota checks if latency exceeds threshold
4. **Async refresh**: Cache updates happen asynchronously (don't block critical path)

### Risk 2: Event Loss During Ingestion Overload

**Risk**: Under extreme load (>10K events/sec), events are dropped or lost.

**Probability**: Low
**Impact**: Critical (billing data loss)

**Mitigation**:
1. **Kafka buffer**: Events first written to Kafka (durable queue), then ingested to DB
2. **Backpressure**: Reject requests with 503 Service Unavailable when queue full (fail-safe)
3. **Horizontal scaling**: Auto-scale ingestion workers based on queue depth
4. **At-least-once delivery**: Retry failed events from Kafka (idempotency prevents duplicates)

### Risk 3: Clock Skew Causing Incorrect Billing Periods

**Risk**: Agent clocks skewed → events assigned to wrong billing period.

**Probability**: Medium
**Impact**: High (customer disputes, revenue leakage)

**Mitigation**:
1. **Consensus-ordered timestamps**: Use distributed consensus clock (not agent clock)
2. **Timestamp validation**: Reject events with timestamps >10 minutes from server time
3. **Billing period buffers**: Use UTC midnight + 1 hour grace period for period boundaries
4. **Audit trail**: Log original agent timestamp + consensus timestamp for dispute resolution

### Risk 4: Attribution Complexity for Deep Delegation Chains

**Risk**: Agent→Agent→Agent→...→Human chains become too deep to attribute efficiently.

**Probability**: Low
**Impact**: Medium (slow attribution queries)

**Mitigation**:
1. **Depth limit**: Enforce max delegation chain depth = 10 (configurable)
2. **Indexed chains**: Store delegation chains as JSONB array with GIN index
3. **Precomputed attribution**: Calculate attribution incrementally on event ingestion
4. **Materialized views**: `cost_by_agent` view refreshed hourly

## Open Questions

### Q1: Should we support real-time streaming aggregation?

**Context**: Lago uses batch aggregation (query database on-demand). Real-time streaming (Apache Flink) enables live dashboards.

**Options**:
- **A**: Batch-only (Lago pattern, simpler)
- **B**: Add Flink/Kafka Streams for real-time aggregates

**Recommendation**: Start with batch (A), evaluate streaming in Phase 2 if latency SLA missed.

**Decision Maker**: Product/Engineering leads

---

### Q2: How do we handle multi-currency pricing?

**Context**: Global customers want invoices in their local currency.

**Options**:
- **A**: Store all prices in USD, apply exchange rates at invoice time
- **B**: Support multi-currency pricing rules natively
- **C**: Delegate to payment gateway (Stripe multi-currency)

**Recommendation**: (A) for Phase 1 (simplest), (C) for production (accurate FX rates).

**Decision Maker**: Finance/Product leads

---

### Q3: Should agents be able to query their own usage?

**Context**: Transparency vs. information leakage (agent learns it's close to quota, games system).

**Options**:
- **A**: Allow agents to query usage (transparent)
- **B**: Only human principals can query usage (secure)
- **C**: Allow queries but rate-limit heavily

**Recommendation**: (B) for Phase 1 (secure), (C) if customer demand high.

**Decision Maker**: Security/Product leads

---

### Q4: What's the event retention policy for compliance?

**Context**: SOX 404 requires 7 years, GDPR allows deletion on request.

**Options**:
- **A**: Hard delete after 7 years (SOX minimum)
- **B**: Anonymize (replace agent_nhi with hash) instead of delete (GDPR right to erasure)
- **C**: Keep forever in cold storage (future ML training)

**Recommendation**: (B) for compliance balance.

**Decision Maker**: Legal/Compliance team

---

## Glossary

| Term | Definition |
|------|------------|
| **Agent NHI** | Non-Human Identity: cryptographic identity for AI agents (e.g., `agent:nhi:ed25519:abc123...`) |
| **Delegation Chain** | Ordered list of agents from leaf to root human principal (e.g., `[worker, scheduler, human]`) |
| **Billable Event** | Usage event submitted by agent, signed, with properties (tokens, API calls, etc.) |
| **Idempotency Key** | Unique identifier for deduplication (prevents double-billing on retries) |
| **Aggregation** | Computation over events (COUNT, SUM, UNIQUE_COUNT, MAX) to calculate usage |
| **Pricing Model** | Algorithm to convert usage into charges (flat, per-unit, tiered, package, prepaid) |
| **Quota** | Spending limit enforced in real-time (hourly/daily/monthly/total) |
| **Overflow Action** | Behavior when quota exceeded (block, allow with overage, notify-only) |
| **Attribution** | Cost breakdown by agent or dimension (which agent consumed resources?) |
| **Consensus-Ordered Timestamp** | Event timestamp from distributed consensus (prevents clock skew attacks) |
| **ML-DSA** | Post-quantum digital signature algorithm (NIST FIPS 204) |
| **Tiered Storage** | Hot (7d) / Warm (90d) / Cold (7y) data lifecycle management |

## References

- **Lago Documentation**: https://getlago.com/docs/guide/events/ingesting-usage
- **Lago GitHub**: https://github.com/getlago/lago
- **Lago Billable Metrics**: https://getlago.com/docs/guide/billable-metrics/overview
- **Lago Pricing Models**: https://getlago.com/docs/guide/plans/charges
- **Creto NHI Specification**: (internal) `creto-nhi/docs/spec.md`
- **Creto Authorization Integration**: (internal) `creto-authz/docs/integration.md`
- **Creto Audit Service**: (internal) `creto-audit/docs/api.md`

---

**Next Steps**:
1. Review and approve this overview (SDD-MTR-00)
2. Proceed to Architecture Design (SDD-MTR-02)
3. Define Data Models (SDD-MTR-03)
4. Specify API Contracts (SDD-MTR-04)
