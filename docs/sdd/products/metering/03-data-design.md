---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# SDD-MTR-03: Metering Data Design

## Overview

This document specifies the data models, schemas, and storage strategies for **creto-metering**. The design prioritizes:

1. **Immutability**: Billing events are append-only (audit compliance)
2. **Query performance**: Aggregations complete in <100ms for 1M events
3. **Scalability**: Support 10B+ events with tiered storage
4. **Flexibility**: JSONB properties for extensibility without schema migrations

## Database Technology Stack

| Component | Technology | Rationale |
|-----------|------------|-----------|
| **Primary database** | PostgreSQL 14+ | Proven for Lago, JSONB support, excellent query planner |
| **Cache layer** | Redis 7+ (Cluster mode) | Sub-millisecond quota lookups, sharding support |
| **Message queue** | Kafka 3+ | Durable event buffering, horizontal scaling |
| **Cold storage** | S3-compatible (MinIO/AWS S3) | Cost-effective long-term retention |
| **Search (optional)** | Elasticsearch 8+ | Full-text search on event properties |

## Core Data Models

### 1. Events Table (Primary Billing Data)

**Purpose**: Immutable append-only log of all billable events.

```sql
CREATE TABLE events (
    -- Primary key
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Deduplication
    idempotency_key TEXT NOT NULL UNIQUE,

    -- Agent identity & attribution
    agent_nhi TEXT NOT NULL, -- e.g., "agent:nhi:ed25519:abc123..."
    delegation_chain JSONB NOT NULL, -- ["agent:worker", "agent:scheduler", "human:alice"]

    -- Subscription mapping
    subscription_id UUID NOT NULL,

    -- Event classification
    event_type TEXT NOT NULL, -- e.g., "llm_tokens", "api_call", "vector_query"

    -- Temporal
    timestamp TIMESTAMPTZ NOT NULL, -- Consensus-ordered timestamp
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT now(), -- Server ingestion time

    -- Flexible properties
    properties JSONB NOT NULL, -- e.g., {"tokens": 1500, "model": "gpt-4"}

    -- Cryptographic verification
    signature BYTEA NOT NULL, -- ML-DSA-65 signature
    signature_algorithm TEXT NOT NULL DEFAULT 'ML-DSA-65',

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Audit trail
    audit_log_id UUID, -- Reference to creto-audit entry

    -- Constraints
    CONSTRAINT valid_delegation_chain CHECK (jsonb_typeof(delegation_chain) = 'array'),
    CONSTRAINT valid_properties CHECK (jsonb_typeof(properties) = 'object'),
    CONSTRAINT timestamp_not_future CHECK (timestamp <= now() + interval '10 minutes')
) PARTITION BY RANGE (timestamp);

-- Indexes
CREATE INDEX idx_events_subscription_time ON events(subscription_id, timestamp DESC);
CREATE INDEX idx_events_type_time ON events(event_type, timestamp DESC);
CREATE INDEX idx_events_agent ON events(agent_nhi, timestamp DESC);
CREATE INDEX idx_events_idempotency ON events(idempotency_key) WHERE created_at > now() - interval '7 days';
CREATE INDEX idx_events_properties ON events USING GIN (properties jsonb_path_ops);

-- Full-text search on properties (optional)
CREATE INDEX idx_events_properties_fulltext ON events USING GIN (to_tsvector('english', properties::text));
```

**Partitioning Strategy**:
```sql
-- Automated monthly partitions (using pg_partman extension)
SELECT partman.create_parent(
    p_parent_table := 'public.events',
    p_control := 'timestamp',
    p_type := 'native',
    p_interval := 'monthly',
    p_premake := 3 -- Pre-create 3 months ahead
);

-- Sample partitions
CREATE TABLE events_2024_12 PARTITION OF events
    FOR VALUES FROM ('2024-12-01 00:00:00+00') TO ('2025-01-01 00:00:00+00');

CREATE TABLE events_2025_01 PARTITION OF events
    FOR VALUES FROM ('2025-01-01 00:00:00+00') TO ('2025-02-01 00:00:00+00');
```

**Example Data**:
```sql
INSERT INTO events (
    idempotency_key,
    agent_nhi,
    delegation_chain,
    subscription_id,
    event_type,
    timestamp,
    properties,
    signature
) VALUES (
    'llm-call-550e8400-e29b-41d4-a716-446655440000',
    'agent:nhi:ed25519:embed-worker-42',
    '["agent:nhi:ed25519:scheduler-001", "human:ops-team@example.com"]'::jsonb,
    '123e4567-e89b-12d3-a456-426614174000'::uuid,
    'llm_tokens',
    '2024-12-25T10:30:00Z',
    '{
        "tokens": 1500,
        "model": "gpt-4",
        "prompt_tokens": 1000,
        "completion_tokens": 500,
        "region": "us-east-1"
    }'::jsonb,
    decode('abcdef...', 'hex') -- ML-DSA signature bytes
);
```

---

### 2. Organizations Table (Multi-Tenant Hierarchy)

**Purpose**: Enable Platform → Organization → Team → Agent hierarchical billing with quota inheritance.

> **P0 Gap Resolution**: This table addresses the multi-tenant hierarchy gap identified in the OSS Alignment Report.

```sql
CREATE TABLE organizations (
    organization_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Hierarchy (self-referential for Platform → Org → Team structure)
    parent_org_id UUID REFERENCES organizations(organization_id),

    -- Organization details
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE, -- URL-safe identifier
    org_type TEXT NOT NULL CHECK (org_type IN ('platform', 'enterprise', 'team', 'project')),

    -- Contact
    billing_email TEXT NOT NULL,
    technical_contact TEXT,

    -- Billing configuration
    billing_currency TEXT NOT NULL DEFAULT 'USD',
    payment_method_id TEXT, -- Stripe payment method
    stripe_customer_id TEXT, -- Stripe customer ID

    -- Quota configuration (can override parent or inherit)
    quota_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    -- Example: {"llm_tokens": {"limit": 1000000, "period": "monthly"}, "inherit": ["storage_gb"]}

    -- Quota inheritance mode
    quota_inheritance TEXT NOT NULL DEFAULT 'inherit'
        CHECK (quota_inheritance IN ('inherit', 'override', 'additive')),

    -- Status
    status TEXT NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'suspended', 'cancelled', 'pending')),

    -- Temporal
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Audit
    created_by TEXT NOT NULL, -- NHI or email of creator

    -- Constraints
    CONSTRAINT no_self_parent CHECK (parent_org_id != organization_id),
    CONSTRAINT valid_quota_config CHECK (jsonb_typeof(quota_config) = 'object')
);

-- Indexes for hierarchy traversal
CREATE INDEX idx_organizations_parent ON organizations(parent_org_id);
CREATE INDEX idx_organizations_slug ON organizations(slug);
CREATE INDEX idx_organizations_type ON organizations(org_type);
CREATE INDEX idx_organizations_stripe ON organizations(stripe_customer_id) WHERE stripe_customer_id IS NOT NULL;

-- Recursive CTE for full hierarchy traversal
-- Usage: SELECT * FROM org_hierarchy WHERE organization_id = 'uuid';
CREATE VIEW org_hierarchy AS
WITH RECURSIVE hierarchy AS (
    -- Base case: start from leaf organizations
    SELECT
        organization_id,
        parent_org_id,
        name,
        org_type,
        quota_config,
        quota_inheritance,
        1 as depth,
        ARRAY[organization_id] as path
    FROM organizations

    UNION ALL

    -- Recursive case: join with parents
    SELECT
        o.organization_id,
        o.parent_org_id,
        o.name,
        o.org_type,
        o.quota_config,
        o.quota_inheritance,
        h.depth + 1,
        h.path || o.organization_id
    FROM organizations o
    JOIN hierarchy h ON o.organization_id = h.parent_org_id
)
SELECT * FROM hierarchy;
```

**Example Hierarchy**:
```sql
-- Platform (root)
INSERT INTO organizations (organization_id, name, slug, org_type, billing_email, created_by)
VALUES ('11111111-0000-0000-0000-000000000000', 'Creto Platform', 'creto', 'platform', 'billing@creto.io', 'system');

-- Enterprise customer
INSERT INTO organizations (organization_id, parent_org_id, name, slug, org_type, billing_email, stripe_customer_id, created_by)
VALUES (
    '22222222-0000-0000-0000-000000000000',
    '11111111-0000-0000-0000-000000000000',
    'Acme Corp',
    'acme',
    'enterprise',
    'billing@acme.com',
    'cus_Acme123',
    'admin@acme.com'
);

-- Team within enterprise
INSERT INTO organizations (organization_id, parent_org_id, name, slug, org_type, billing_email, quota_config, created_by)
VALUES (
    '33333333-0000-0000-0000-000000000000',
    '22222222-0000-0000-0000-000000000000',
    'Acme ML Team',
    'acme-ml',
    'team',
    'ml-team@acme.com',
    '{"llm_tokens": {"limit": 500000, "period": "monthly"}}'::jsonb,
    'ml-lead@acme.com'
);
```

**Quota Inheritance Logic**:
```rust
/// Calculate effective quota for an organization
pub async fn get_effective_quota(
    org_id: OrganizationId,
    metric: &str,
) -> Result<QuotaLimit, QuotaError> {
    // Traverse hierarchy from org to root
    let hierarchy = get_org_hierarchy(org_id).await?;

    let mut effective_limit: Option<QuotaLimit> = None;

    for org in hierarchy.iter().rev() { // root to leaf
        match org.quota_inheritance {
            QuotaInheritance::Inherit => {
                // Use parent quota if no override
                if let Some(limit) = org.quota_config.get(metric) {
                    effective_limit = Some(limit.clone());
                }
            }
            QuotaInheritance::Override => {
                // Always use this org's quota
                if let Some(limit) = org.quota_config.get(metric) {
                    effective_limit = Some(limit.clone());
                }
            }
            QuotaInheritance::Additive => {
                // Add to parent quota
                if let (Some(parent), Some(child)) = (&effective_limit, org.quota_config.get(metric)) {
                    effective_limit = Some(parent.add(child));
                }
            }
        }
    }

    effective_limit.ok_or(QuotaError::NoQuotaDefined { metric: metric.to_string() })
}
```

---

### 3. Subscriptions Table

**Purpose**: Map agents to billing entities and pricing configurations.

```sql
CREATE TABLE subscriptions (
    subscription_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Customer identification (references organizations hierarchy)
    organization_id UUID NOT NULL REFERENCES organizations(organization_id),
    customer_email TEXT NOT NULL,

    -- Subscription state
    status TEXT NOT NULL CHECK (status IN ('active', 'suspended', 'cancelled', 'expired')),

    -- Billing cycle
    billing_period TEXT NOT NULL CHECK (billing_period IN ('monthly', 'yearly', 'daily')),
    period_start_day INT CHECK (period_start_day BETWEEN 1 AND 31), -- e.g., bill on 1st of month
    timezone TEXT NOT NULL DEFAULT 'UTC',

    -- Temporal
    started_at TIMESTAMPTZ NOT NULL,
    cancelled_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_subscriptions_org ON subscriptions(organization_id);
CREATE INDEX idx_subscriptions_status ON subscriptions(status) WHERE status = 'active';
```

---

### 3. Billable Metrics Configuration

**Purpose**: Define how events are aggregated and priced.

```sql
CREATE TABLE billable_metrics (
    metric_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES subscriptions(subscription_id) ON DELETE CASCADE,

    -- Metric definition
    metric_code TEXT NOT NULL, -- Must match events.event_type
    metric_name TEXT NOT NULL,
    description TEXT,

    -- Aggregation configuration
    aggregation_type TEXT NOT NULL CHECK (aggregation_type IN ('COUNT', 'SUM', 'UNIQUE_COUNT', 'MAX')),
    aggregation_property TEXT, -- Required for SUM, UNIQUE_COUNT, MAX (e.g., "tokens", "model")

    -- Filtering (optional)
    filter_conditions JSONB, -- e.g., {"model": "gpt-4"} - only count events with model=gpt-4

    -- Grouping dimensions
    group_by TEXT[], -- e.g., ARRAY['model', 'region'] - break down usage by these dimensions

    -- Active/inactive
    active BOOLEAN NOT NULL DEFAULT true,

    -- Temporal
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(subscription_id, metric_code)
);

CREATE INDEX idx_billable_metrics_subscription ON billable_metrics(subscription_id);
CREATE INDEX idx_billable_metrics_code ON billable_metrics(metric_code);
```

**Example**:
```sql
INSERT INTO billable_metrics (
    subscription_id,
    metric_code,
    metric_name,
    description,
    aggregation_type,
    aggregation_property
) VALUES (
    '123e4567-e89b-12d3-a456-426614174000'::uuid,
    'llm_tokens',
    'LLM Token Consumption',
    'Total tokens consumed across all LLM API calls',
    'SUM',
    'tokens'
);
```

---

### 4. Pricing Models

**Purpose**: Define pricing rules for billable metrics.

```sql
CREATE TYPE pricing_model_type AS ENUM (
    'flat',
    'per_unit',
    'tiered_graduated',
    'tiered_volume',
    'package',
    'prepaid_credits'
);

CREATE TABLE pricing_models (
    pricing_model_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    metric_id UUID NOT NULL REFERENCES billable_metrics(metric_id) ON DELETE CASCADE,

    -- Model type
    model_type pricing_model_type NOT NULL,

    -- Flat fee configuration
    flat_amount DECIMAL(19, 4), -- For model_type = 'flat'

    -- Per-unit configuration
    unit_price DECIMAL(19, 4), -- For model_type = 'per_unit'

    -- Tiered configuration (graduated or volume)
    tiers JSONB, -- Array of {up_to: number|null, unit_price: decimal}

    -- Package configuration
    package_size BIGINT, -- For model_type = 'package'
    package_price DECIMAL(19, 4),
    overage_unit_price DECIMAL(19, 4),

    -- Prepaid credits configuration
    credit_rate DECIMAL(19, 4), -- $/credit
    auto_topup_threshold DECIMAL(19, 4),
    auto_topup_amount DECIMAL(19, 4),

    -- Currency
    currency TEXT NOT NULL DEFAULT 'USD' CHECK (currency IN ('USD', 'EUR', 'GBP')),

    -- Proration for partial periods
    prorated BOOLEAN NOT NULL DEFAULT true,

    -- Active period
    active_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    active_until TIMESTAMPTZ,

    -- Temporal
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_pricing_models_metric ON pricing_models(metric_id);
```

**Example (Tiered Graduated)**:
```sql
INSERT INTO pricing_models (
    metric_id,
    model_type,
    tiers,
    currency
) VALUES (
    'metric-uuid',
    'tiered_graduated',
    '[
        {"up_to": 1000, "unit_price": 0.01},
        {"up_to": 10000, "unit_price": 0.008},
        {"up_to": null, "unit_price": 0.005}
    ]'::jsonb,
    'USD'
);
```

---

### 5. Quotas Table

**Purpose**: Define spending limits enforced in real-time.

```sql
CREATE TYPE quota_period AS ENUM ('hourly', 'daily', 'monthly', 'total');
CREATE TYPE overflow_action AS ENUM ('block', 'allow_with_overage', 'notify_only');

CREATE TABLE quotas (
    quota_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES subscriptions(subscription_id) ON DELETE CASCADE,

    -- Quota scope
    event_type TEXT NOT NULL, -- e.g., "api_calls"

    -- Limit configuration
    limit_value BIGINT NOT NULL, -- e.g., 1000
    period quota_period NOT NULL,

    -- Behavior on overflow
    overflow_action overflow_action NOT NULL DEFAULT 'block',

    -- Overage pricing (if overflow_action = 'allow_with_overage')
    overage_rate DECIMAL(19, 4),

    -- Active/inactive
    active BOOLEAN NOT NULL DEFAULT true,

    -- Temporal
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE(subscription_id, event_type, period)
);

CREATE INDEX idx_quotas_subscription ON quotas(subscription_id);
CREATE INDEX idx_quotas_active ON quotas(subscription_id, event_type) WHERE active = true;
```

**Example**:
```sql
INSERT INTO quotas (
    subscription_id,
    event_type,
    limit_value,
    period,
    overflow_action
) VALUES (
    '123e4567-e89b-12d3-a456-426614174000'::uuid,
    'api_calls',
    1000,
    'hourly',
    'block'
);
```

---

### 6. Usage Summaries (Materialized View)

**Purpose**: Precomputed aggregations for fast queries.

```sql
CREATE TABLE usage_hourly (
    subscription_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    hour TIMESTAMPTZ NOT NULL, -- Truncated to hour

    -- Aggregated metrics
    event_count BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT, -- SUM(properties->>'tokens')
    unique_models TEXT[], -- COUNT(DISTINCT properties->>'model')
    max_latency_ms INT, -- MAX(properties->>'latency_ms')

    -- Metadata
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (subscription_id, event_type, hour)
) PARTITION BY RANGE (hour);

-- Partitions (monthly)
CREATE TABLE usage_hourly_2024_12 PARTITION OF usage_hourly
    FOR VALUES FROM ('2024-12-01') TO ('2025-01-01');

-- Indexes
CREATE INDEX idx_usage_hourly_lookup ON usage_hourly(subscription_id, hour DESC);
CREATE INDEX idx_usage_hourly_type ON usage_hourly(event_type, hour DESC);
```

**Incremental Update Trigger**:
```sql
-- Function to update usage_hourly on event insert
CREATE OR REPLACE FUNCTION update_usage_hourly()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO usage_hourly (subscription_id, event_type, hour, event_count, total_tokens, unique_models, max_latency_ms)
    VALUES (
        NEW.subscription_id,
        NEW.event_type,
        date_trunc('hour', NEW.timestamp),
        1,
        (NEW.properties->>'tokens')::bigint,
        ARRAY[NEW.properties->>'model'],
        (NEW.properties->>'latency_ms')::int
    )
    ON CONFLICT (subscription_id, event_type, hour)
    DO UPDATE SET
        event_count = usage_hourly.event_count + 1,
        total_tokens = usage_hourly.total_tokens + COALESCE(EXCLUDED.total_tokens, 0),
        unique_models = array_unique(array_cat(usage_hourly.unique_models, EXCLUDED.unique_models)),
        max_latency_ms = GREATEST(usage_hourly.max_latency_ms, EXCLUDED.max_latency_ms),
        updated_at = now();

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger (async to avoid blocking inserts)
CREATE TRIGGER trigger_update_usage_hourly
AFTER INSERT ON events
FOR EACH ROW
EXECUTE FUNCTION update_usage_hourly();
```

---

### 7. Invoices Table

**Purpose**: Generated invoices for billing periods.

```sql
CREATE TYPE invoice_status AS ENUM ('draft', 'issued', 'paid', 'void', 'disputed');

CREATE TABLE invoices (
    invoice_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES subscriptions(subscription_id),

    -- Billing period
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,

    -- Financial summary
    subtotal DECIMAL(19, 4) NOT NULL,
    tax DECIMAL(19, 4) NOT NULL DEFAULT 0,
    total DECIMAL(19, 4) NOT NULL,
    currency TEXT NOT NULL DEFAULT 'USD',

    -- Attribution data
    attribution JSONB, -- {"by_agent": {...}, "by_dimension": {...}}

    -- Invoice state
    status invoice_status NOT NULL DEFAULT 'draft',

    -- Payment gateway integration
    stripe_invoice_id TEXT,
    stripe_payment_intent_id TEXT,

    -- Timestamps
    issued_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ,
    voided_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT valid_period CHECK (period_end > period_start),
    CONSTRAINT valid_amounts CHECK (total = subtotal + tax)
);

CREATE INDEX idx_invoices_subscription ON invoices(subscription_id, period_start DESC);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_period ON invoices(period_start, period_end);
```

---

### 8. Invoice Line Items

**Purpose**: Detailed charges per billable metric.

```sql
CREATE TABLE invoice_line_items (
    line_item_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES invoices(invoice_id) ON DELETE CASCADE,

    -- Metric reference
    metric_code TEXT NOT NULL,
    description TEXT NOT NULL,

    -- Usage & pricing
    quantity DECIMAL(19, 4) NOT NULL, -- Aggregated usage
    unit_price DECIMAL(19, 4), -- For per-unit pricing
    amount DECIMAL(19, 4) NOT NULL, -- Total charge for this line item

    -- Metadata
    pricing_model_snapshot JSONB, -- Full pricing config at invoice time (audit trail)

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_line_items_invoice ON invoice_line_items(invoice_id);
```

---

### 9. Prepaid Wallets (Optional)

**Purpose**: Track prepaid credit balances for subscriptions.

```sql
CREATE TABLE prepaid_wallets (
    wallet_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    subscription_id UUID NOT NULL REFERENCES subscriptions(subscription_id) UNIQUE,

    -- Balance
    balance DECIMAL(19, 4) NOT NULL DEFAULT 0,
    credit_rate DECIMAL(19, 4) NOT NULL, -- $/credit conversion rate
    currency TEXT NOT NULL DEFAULT 'USD',

    -- Auto top-up configuration
    auto_topup_enabled BOOLEAN NOT NULL DEFAULT false,
    auto_topup_threshold DECIMAL(19, 4),
    auto_topup_amount DECIMAL(19, 4),

    -- Temporal
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT positive_balance CHECK (balance >= 0 OR NOT auto_topup_enabled)
);

-- Wallet transaction history
CREATE TABLE wallet_transactions (
    transaction_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_id UUID NOT NULL REFERENCES prepaid_wallets(wallet_id),

    -- Transaction details
    amount DECIMAL(19, 4) NOT NULL, -- Positive = credit, Negative = debit
    transaction_type TEXT NOT NULL CHECK (transaction_type IN ('topup', 'usage', 'refund', 'adjustment')),

    -- Context
    event_id UUID REFERENCES events(event_id), -- For usage debits
    invoice_id UUID REFERENCES invoices(invoice_id), -- For topup charges
    description TEXT,

    -- Balance snapshot
    balance_before DECIMAL(19, 4) NOT NULL,
    balance_after DECIMAL(19, 4) NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_wallet_transactions_wallet ON wallet_transactions(wallet_id, created_at DESC);
```

---

## Redis Cache Schema

**Purpose**: Fast in-memory quota checks and idempotency.

### Idempotency Cache

**Key**: `idempotency:{idempotency_key}`
**Value**: JSON `{"event_id": "uuid", "hash": "sha256", "created_at": "iso8601"}`
**TTL**: 7 days

**Example**:
```
SET idempotency:txn_001 '{"event_id": "550e8400-...", "hash": "abcdef...", "created_at": "2024-12-25T10:00:00Z"}' EX 604800
```

### Quota State Cache

**Key**: `quota:{subscription_id}:{event_type}:{period}`
**Value**: JSON `{"limit": 1000, "current_usage": 450, "period_start": "iso8601", "overflow_action": "block"}`
**TTL**: 60 seconds (ephemeral)

**Example**:
```
SET quota:123e4567:api_calls:hourly '{"limit": 1000, "current_usage": 450, "period_start": "2024-12-25T10:00:00Z", "overflow_action": "block"}' EX 60
```

### Aggregation Cache

**Key**: `agg:{subscription_id}:{event_type}:{period_key}`
**Value**: JSON `{"count": 15000, "sum_tokens": 2500000, "unique_models": ["gpt-4", "gpt-3.5"], "max_latency": 450}`
**TTL**: 5 minutes (current period), 1 day (historical)

---

## Kafka Topics Schema

### Topic: `events.ingestion`

**Purpose**: Durable buffer for event ingestion.

**Partitioning**: By `subscription_id` (ensures ordering per subscription)

**Message Format** (Avro/Protobuf):
```json
{
  "idempotency_key": "txn_001",
  "agent_nhi": "agent:nhi:ed25519:abc123",
  "delegation_chain": ["agent:scheduler", "human:alice"],
  "subscription_id": "123e4567-...",
  "event_type": "llm_tokens",
  "timestamp": "2024-12-25T10:00:00Z",
  "properties": {
    "tokens": 1500,
    "model": "gpt-4"
  },
  "signature": "base64encodedSignature",
  "signature_algorithm": "ML-DSA-65"
}
```

**Retention**: 7 days (after ingestion to PostgreSQL, events can be deleted from Kafka)

---

## S3 Cold Storage Schema

**Purpose**: Long-term retention for compliance (7 years).

### Bucket Structure

```
s3://creto-metering-cold-storage/
├── events/
│   ├── year=2024/
│   │   ├── month=12/
│   │   │   ├── day=01/
│   │   │   │   └── events_2024-12-01.parquet.gz
│   │   │   ├── day=02/
│   │   │   │   └── events_2024-12-02.parquet.gz
│   │   │   └── ...
│   │   └── month=11/
│   └── year=2023/
└── invoices/
    ├── year=2024/
    │   ├── month=12/
    │   │   └── invoices_2024-12.parquet.gz
```

### Parquet Schema (Events)

```
message Event {
  required binary event_id (STRING);
  required binary idempotency_key (STRING);
  required binary agent_nhi (STRING);
  required group delegation_chain (LIST) {
    repeated binary element (STRING);
  }
  required binary subscription_id (STRING);
  required binary event_type (STRING);
  required int64 timestamp (TIMESTAMP);
  required binary properties (JSON);
  required binary signature (BYTE_ARRAY);
  optional binary audit_log_id (STRING);
}
```

**Compression**: Gzip (10:1 ratio typical)
**Lifecycle**: Transition to Glacier after 1 year, delete after 7 years

---

## Query Patterns & Optimization

### Query 1: Get Current Period Usage

**Use Case**: Real-time quota check

```sql
-- Optimized with materialized view
SELECT SUM(event_count) AS total_usage
FROM usage_hourly
WHERE subscription_id = $1
  AND event_type = $2
  AND hour >= date_trunc('hour', now() - interval '1 hour');
```

**Performance**: <10ms (indexed lookup + small result set)

---

### Query 2: Generate Invoice

**Use Case**: Month-end billing

```sql
-- Aggregate usage for entire month
SELECT
    event_type,
    SUM(event_count) AS total_count,
    SUM(total_tokens) AS total_tokens,
    COUNT(DISTINCT unnest(unique_models)) AS unique_models
FROM usage_hourly
WHERE subscription_id = $1
  AND hour >= $2 -- period_start
  AND hour < $3  -- period_end
GROUP BY event_type;
```

**Performance**: <100ms (partition pruning + materialized view)

---

### Query 3: Cost Attribution by Agent

**Use Case**: Chargeback reporting

```sql
-- Extract costs by agent from delegation chains
WITH agent_events AS (
    SELECT
        event_id,
        jsonb_array_elements_text(delegation_chain) AS agent,
        (properties->>'tokens')::bigint AS tokens
    FROM events
    WHERE subscription_id = $1
      AND timestamp >= $2
      AND timestamp < $3
)
SELECT
    agent,
    SUM(tokens) AS total_tokens,
    SUM(tokens) * $4 AS estimated_cost -- $4 = unit_price
FROM agent_events
GROUP BY agent
ORDER BY total_tokens DESC;
```

**Performance**: <500ms (JSONB index + partition pruning)

---

### Query 4: Search Events by Property

**Use Case**: Debugging, dispute resolution

```sql
-- Find all GPT-4 events for a customer
SELECT event_id, timestamp, properties
FROM events
WHERE subscription_id = $1
  AND properties @> '{"model": "gpt-4"}'::jsonb
  AND timestamp >= $2
ORDER BY timestamp DESC
LIMIT 100;
```

**Performance**: <50ms (GIN index on properties)

---

## Data Lifecycle Management

### Tiered Storage Transitions

```sql
-- Function to archive warm data to S3 (run nightly)
CREATE OR REPLACE FUNCTION archive_warm_events() RETURNS void AS $$
DECLARE
    cutoff_date TIMESTAMPTZ := now() - interval '90 days';
BEGIN
    -- Export to S3 (via foreign data wrapper or pg_dump)
    COPY (
        SELECT * FROM events
        WHERE timestamp < cutoff_date
          AND timestamp >= cutoff_date - interval '1 day'
    ) TO PROGRAM 'gzip | aws s3 cp - s3://creto-metering-cold-storage/events/year=$(date +%Y)/month=$(date +%m)/events_$(date +%Y-%m-%d).parquet.gz'
    WITH (FORMAT parquet);

    -- Delete from hot storage (after successful S3 upload)
    DELETE FROM events
    WHERE timestamp < cutoff_date
      AND timestamp >= cutoff_date - interval '1 day';
END;
$$ LANGUAGE plpgsql;
```

### Retention Policy

| Data Type | Hot Storage | Warm Storage | Cold Storage | Total Retention |
|-----------|-------------|--------------|--------------|-----------------|
| **Events** | 7 days (PostgreSQL SSD) | 90 days (PostgreSQL cheaper SSD) | 7 years (S3 Glacier) | 7 years (compliance) |
| **Quotas** | Forever | N/A | N/A | Forever (operational) |
| **Invoices** | Forever | N/A | 7 years (S3) | 7 years (compliance) |
| **Aggregations** | 90 days | N/A | N/A | 90 days (ephemeral) |

---

## Backup & Disaster Recovery

### PostgreSQL Backup Strategy

**Continuous Archiving (WAL)**:
```bash
# Enable WAL archiving (postgresql.conf)
wal_level = replica
archive_mode = on
archive_command = 'aws s3 cp %p s3://creto-metering-backups/wal/%f'
```

**Daily Base Backups**:
```bash
# pg_basebackup to S3 (cron daily at 2am UTC)
pg_basebackup -D /tmp/backup -Ft -z -P | aws s3 cp - s3://creto-metering-backups/base/backup_$(date +%Y%m%d).tar.gz
```

**Point-in-Time Recovery**:
- RPO (Recovery Point Objective): 0 seconds (WAL streaming)
- RTO (Recovery Time Objective): <1 hour (restore base + replay WAL)

### Redis Persistence

**RDB Snapshots**:
```conf
# redis.conf
save 900 1      # Save after 900 sec if ≥1 key changed
save 300 10     # Save after 300 sec if ≥10 keys changed
save 60 10000   # Save after 60 sec if ≥10000 keys changed
dir /var/lib/redis
dbfilename dump.rdb
```

**AOF (Append-Only File)**:
```conf
appendonly yes
appendfsync everysec
```

**Trade-off**: Redis caches are ephemeral (acceptable data loss), but RDB provides faster recovery.

---

## Data Migration Strategy

### Schema Versioning

Use Flyway/Liquibase for migrations:

```sql
-- V001__initial_schema.sql
CREATE TABLE events (...);
CREATE TABLE subscriptions (...);

-- V002__add_signature_column.sql
ALTER TABLE events ADD COLUMN signature BYTEA NOT NULL DEFAULT '\x00';

-- V003__add_delegation_chain_index.sql
CREATE INDEX idx_events_delegation_chain ON events USING GIN (delegation_chain);
```

### Zero-Downtime Migrations

**Pattern**: Expand-Migrate-Contract

1. **Expand**: Add new column (nullable)
2. **Migrate**: Backfill data (background job)
3. **Contract**: Make column NOT NULL, drop old column

**Example**:
```sql
-- Step 1: Expand (add new column)
ALTER TABLE events ADD COLUMN signature_v2 BYTEA;

-- Step 2: Migrate (backfill, can run while system live)
UPDATE events SET signature_v2 = signature WHERE signature_v2 IS NULL;

-- Step 3: Contract (make non-null, drop old)
ALTER TABLE events ALTER COLUMN signature_v2 SET NOT NULL;
ALTER TABLE events DROP COLUMN signature;
ALTER TABLE events RENAME COLUMN signature_v2 TO signature;
```

---

## Security Considerations

### Encryption at Rest

**PostgreSQL**: Transparent Data Encryption (TDE) or disk-level encryption (LUKS)
**Redis**: Redis 6+ native encryption (TLS for in-transit)
**S3**: Server-side encryption (SSE-S3 or SSE-KMS)

### Encryption in Transit

**PostgreSQL**: `sslmode=require` for all connections
**Redis**: TLS enabled for cluster communication
**Kafka**: TLS + SASL for producer/consumer authentication

### Access Control

**PostgreSQL Roles**:
```sql
-- Read-only role for analytics
CREATE ROLE metering_readonly;
GRANT SELECT ON ALL TABLES IN SCHEMA public TO metering_readonly;

-- Write role for ingestion service
CREATE ROLE metering_ingest;
GRANT INSERT ON events TO metering_ingest;
GRANT SELECT, INSERT ON usage_hourly TO metering_ingest;

-- Admin role for billing service
CREATE ROLE metering_billing;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO metering_billing;
```

---

## Performance Benchmarks

### Expected Query Performance (1M events/month)

| Query | p50 | p99 | Notes |
|-------|-----|-----|-------|
| **Single event insert** | 5ms | 20ms | Includes idempotency check |
| **Batch insert (1000 events)** | 200ms | 500ms | Parallel inserts |
| **Quota check (cache hit)** | 2µs | 10µs | In-memory lookup |
| **Quota check (cache miss)** | 100µs | 500µs | Redis + DB query |
| **Current period usage** | 5ms | 20ms | Materialized view |
| **Invoice generation** | 200ms | 1s | 10 billable metrics |
| **Attribution query** | 100ms | 500ms | JSONB aggregation |

### Storage Estimates

**Events**:
- Average event size: 1KB (with JSONB properties)
- 1M events/month = 1GB/month
- 12 months (hot+warm) = 12GB
- 7 years (cold, compressed) = 10GB (10:1 compression)

**Quotas**: ~100 bytes/quota × 10K quotas = 1MB (negligible)

**Invoices**: ~10KB/invoice × 100K subscriptions/month = 1GB/month

---

## Observability

### Key Metrics to Track

```sql
-- Database statistics
SELECT
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) AS size,
    n_live_tup AS rows,
    n_dead_tup AS dead_rows,
    last_vacuum,
    last_autovacuum
FROM pg_stat_user_tables
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;

-- Query performance
SELECT
    query,
    calls,
    mean_exec_time,
    max_exec_time,
    stddev_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC
LIMIT 10;
```

### Alerts

- **Disk usage** >80% (trigger archival)
- **Query latency** p99 >1s (investigate slow queries)
- **Replication lag** >10s (promote replica)
- **Cache hit rate** <90% (increase cache size)

---

**Next Document**: SDD-MTR-04: API Design (gRPC/REST endpoints, request/response schemas)
