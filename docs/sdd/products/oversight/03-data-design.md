---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
parent_sdd: docs/sdd/products/oversight/01-requirements.md
---

# SDD-OVS-03: Oversight Data Design

## 1. Data Model Overview

### 1.1 Entity Relationship Diagram

```
┌──────────────────────┐
│ oversight_requests   │
│──────────────────────│
│ request_id (PK)      │◄──────────┐
│ agent_nhi            │           │
│ delegation_chain     │           │
│ action               │           │
│ resource             │           │
│ policy_id            │           │
│ state                │           │
│ tier_index           │           │
│ created_at           │           │
│ updated_at           │           │
│ version              │           │
└──────────────────────┘           │
         │                         │
         │ 1:N                     │ 1:N
         │                         │
         v                         │
┌──────────────────────┐           │
│ approval_responses   │           │
│──────────────────────│           │
│ response_id (PK)     │           │
│ request_id (FK)      ├───────────┘
│ approver_subject     │
│ approver_name        │
│ approver_public_key  │
│ decision             │
│ reason               │
│ signature_algorithm  │
│ signature_value      │
│ channel_type         │
│ channel_metadata     │
│ responded_at         │
└──────────────────────┘

┌──────────────────────┐
│ policies             │
│──────────────────────│
│ policy_id (PK)       │◄──────────┐
│ agent_pattern        │           │
│ action_pattern       │           │
│ resource_pattern     │           │
│ approver_spec        │           │
│ escalation_chain_id  ├───────────┐
│ quorum_type          │           │
│ quorum_value         │           │
│ created_at           │           │
│ updated_at           │           │
└──────────────────────┘           │
                                   │ 1:1
                                   │
                                   v
                        ┌──────────────────────┐
                        │ escalation_chains    │
                        │──────────────────────│
                        │ chain_id (PK)        │
                        │ name                 │
                        │ final_action         │
                        │ created_at           │
                        └──────────────────────┘
                                   │
                                   │ 1:N
                                   │
                                   v
                        ┌──────────────────────┐
                        │ escalation_tiers     │
                        │──────────────────────│
                        │ tier_id (PK)         │
                        │ chain_id (FK)        │
                        │ tier_index           │
                        │ approvers            │
                        │ timeout_seconds      │
                        │ channels             │
                        │ quorum_type          │
                        │ quorum_value         │
                        └──────────────────────┘

┌──────────────────────┐
│ notification_log     │
│──────────────────────│
│ delivery_id (PK)     │
│ request_id (FK)      │
│ tier_index           │
│ channel_type         │
│ recipient            │
│ message_id           │
│ status               │
│ delivered_at         │
│ error_message        │
└──────────────────────┘

┌──────────────────────┐
│ escalation_history   │
│──────────────────────│
│ event_id (PK)        │
│ request_id (FK)      │
│ from_tier            │
│ to_tier              │
│ reason               │
│ escalated_at         │
└──────────────────────┘

┌──────────────────────┐
│ approval_audit       │
│──────────────────────│
│ audit_id (PK)        │
│ request_id (FK)      │
│ event_type           │
│ agent_nhi            │
│ approver_subject     │
│ old_state            │
│ new_state            │
│ decision             │
│ signature_value      │
│ merkle_proof         │
│ timestamp            │
└──────────────────────┘
```

### 1.2 Data Flow

```
Authorization Service
       │
       │ Decision::RequiresOversight
       v
┌─────────────────┐
│ PolicyEngine    │ ──► policies table (lookup)
└────────┬────────┘
         │ OversightRequirement
         v
┌─────────────────┐
│ RequestManager  │ ──► oversight_requests table (INSERT)
│                 │ ──► notification_log table (INSERT)
└────────┬────────┘
         │
         │ Notifications sent
         v
  ┌──────────────┐
  │  Approver    │
  └──────┬───────┘
         │ Approval response
         v
┌─────────────────┐
│ResponseHandler  │ ──► approval_responses table (INSERT)
│                 │ ──► oversight_requests table (UPDATE state)
│                 │ ──► approval_audit table (INSERT)
└─────────────────┘
```

---

## 2. PostgreSQL Schema Design

### 2.1 Core Tables

#### 2.1.1 `oversight_requests`

**Purpose:** Store request state and metadata

```sql
CREATE TABLE oversight_requests (
    -- Primary Key
    request_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Agent Identity
    agent_nhi TEXT NOT NULL,
    delegation_chain JSONB NOT NULL,  -- Array of agent NHI identifiers

    -- Action Context
    action TEXT NOT NULL,  -- JSON-serialized Action enum
    resource JSONB NOT NULL,  -- Resource metadata
    policy_id TEXT NOT NULL,

    -- Request Context
    action_description TEXT NOT NULL,
    reasoning TEXT,
    risk_factors JSONB,  -- Array of RiskFactor objects
    impact_assessment JSONB,

    -- State Machine
    state TEXT NOT NULL CHECK (state IN ('PENDING', 'APPROVED', 'DENIED', 'ESCALATED', 'TIMEOUT', 'CANCELLED')),
    tier_index INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Optimistic Concurrency Control
    version BIGINT NOT NULL DEFAULT 0
);

-- Indexes
CREATE INDEX idx_requests_state ON oversight_requests(state) WHERE state IN ('PENDING', 'ESCALATED');
CREATE INDEX idx_requests_agent_nhi ON oversight_requests(agent_nhi);
CREATE INDEX idx_requests_policy_id ON oversight_requests(policy_id);
CREATE INDEX idx_requests_created_at ON oversight_requests(created_at);

-- Partial index for active requests (performance optimization)
CREATE INDEX idx_requests_active ON oversight_requests(request_id, state, updated_at)
WHERE state IN ('PENDING', 'ESCALATED');

-- Update trigger for updated_at
CREATE TRIGGER update_oversight_requests_updated_at
BEFORE UPDATE ON oversight_requests
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
```

**Sample Data:**
```sql
INSERT INTO oversight_requests (
    request_id,
    agent_nhi,
    delegation_chain,
    action,
    resource,
    policy_id,
    action_description,
    reasoning,
    risk_factors,
    state,
    tier_index
) VALUES (
    'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    'agent:payment-bot-v3@company.creto',
    '["agent:payment-bot-v3@company.creto", "agent:accounts-payable@company.creto", "human:alice@company.com"]'::jsonb,
    '{"type":"TransferFunds","params":{"amount":50000,"currency":"USD","recipient":"vendor@example.com"}}',
    '{"type":"bank_transfer","amount":50000.00,"currency":"USD","recipient":"vendor@example.com","invoice_id":"INV-2024-1234"}'::jsonb,
    'pol_large_transfer_cfo_approval',
    'Transfer $50,000 to vendor invoice #INV-2024-1234',
    'Invoice approved in AP system. Vendor contract requires payment within 30 days. Due date: 2024-12-31.',
    '[{"category":"Financial","severity":"High","description":"Large transaction exceeding $10K threshold"},{"category":"Compliance","severity":"Medium","description":"SOX dual control required"}]'::jsonb,
    'PENDING',
    0
);
```

#### 2.1.2 `approval_responses`

**Purpose:** Store individual approver responses with cryptographic signatures

```sql
CREATE TABLE approval_responses (
    -- Primary Key
    response_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign Key to Request
    request_id UUID NOT NULL REFERENCES oversight_requests(request_id) ON DELETE CASCADE,

    -- Approver Identity
    approver_subject TEXT NOT NULL,  -- Email or NHI identifier
    approver_name TEXT NOT NULL,
    approver_public_key BYTEA NOT NULL,  -- ML-DSA or Ed25519 public key

    -- Decision
    decision TEXT NOT NULL CHECK (decision IN ('APPROVE', 'DENY', 'REQUEST_MORE_INFO')),
    reason TEXT,  -- Optional justification
    question TEXT,  -- For REQUEST_MORE_INFO decision

    -- Cryptographic Signature
    signature_algorithm TEXT NOT NULL,  -- "ML-DSA-65", "Ed25519", etc.
    signature_value BYTEA NOT NULL,  -- Signature over (request_id || decision || responded_at)

    -- Channel Metadata
    channel_type TEXT NOT NULL CHECK (channel_type IN ('SLACK', 'EMAIL', 'WEBHOOK')),
    channel_metadata JSONB,  -- Channel-specific data (Slack message TS, email message ID, etc.)

    -- Timestamp
    responded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_responses_request_id ON approval_responses(request_id);
CREATE INDEX idx_responses_approver ON approval_responses(approver_subject);
CREATE INDEX idx_responses_decision ON approval_responses(decision);

-- Unique constraint: one response per approver per request (idempotency)
CREATE UNIQUE INDEX idx_responses_unique_approver ON approval_responses(request_id, approver_subject);
```

**Sample Data:**
```sql
INSERT INTO approval_responses (
    response_id,
    request_id,
    approver_subject,
    approver_name,
    approver_public_key,
    decision,
    reason,
    signature_algorithm,
    signature_value,
    channel_type,
    channel_metadata,
    responded_at
) VALUES (
    'b2c3d4e5-f6a7-8901-bcde-f12345678901',
    'a1b2c3d4-e5f6-7890-abcd-ef1234567890',
    'cfo@company.com',
    'Alice CFO',
    decode('abcdef0123456789', 'hex'),  -- Public key bytes
    'APPROVE',
    'Invoice verified, payment authorized per contract',
    'ML-DSA-65',
    decode('0123456789abcdef', 'hex'),  -- Signature bytes
    'SLACK',
    '{"slack_user_id":"U123456","message_ts":"1703520000.123456","channel_id":"C123456"}'::jsonb,
    '2024-12-25 14:30:00+00'
);
```

#### 2.1.3 `policies`

**Purpose:** Store oversight policy configurations

```sql
CREATE TABLE policies (
    -- Primary Key
    policy_id TEXT PRIMARY KEY,

    -- Policy Trigger Patterns
    agent_pattern TEXT NOT NULL,  -- Regex or glob pattern (e.g., "agent:payment-*")
    action_pattern TEXT NOT NULL,  -- Action type pattern (e.g., "TransferFunds")
    resource_pattern JSONB,  -- Resource matching criteria (e.g., {"amount": {">": 10000}})

    -- Approver Specification
    approver_spec JSONB NOT NULL,  -- {type: "individual|role|dynamic", value: [...]}

    -- Escalation Chain
    escalation_chain_id TEXT REFERENCES escalation_chains(chain_id),

    -- Quorum Configuration
    quorum_type TEXT NOT NULL CHECK (quorum_type IN ('ANY', 'ALL', 'THRESHOLD')),
    quorum_value INTEGER,  -- For THRESHOLD type

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by TEXT,  -- Human who created this policy
    description TEXT
);

-- Indexes
CREATE INDEX idx_policies_agent_pattern ON policies(agent_pattern);
CREATE INDEX idx_policies_action_pattern ON policies(action_pattern);
CREATE INDEX idx_policies_chain_id ON policies(escalation_chain_id);
```

**Sample Data:**
```sql
INSERT INTO policies (
    policy_id,
    agent_pattern,
    action_pattern,
    resource_pattern,
    approver_spec,
    escalation_chain_id,
    quorum_type,
    quorum_value,
    description
) VALUES (
    'pol_large_transfer_cfo_approval',
    'agent:payment-*',
    'TransferFunds',
    '{"amount": {">": 10000, "<=": 100000}}'::jsonb,
    '{"type": "individual", "value": ["cfo@company.com"]}'::jsonb,
    'chain_cfo_escalation',
    'ANY',
    NULL,
    'Large transfers ($10K-$100K) require CFO approval'
);
```

#### 2.1.4 `escalation_chains`

**Purpose:** Define multi-tier escalation chains

```sql
CREATE TABLE escalation_chains (
    -- Primary Key
    chain_id TEXT PRIMARY KEY,

    -- Metadata
    name TEXT NOT NULL,
    description TEXT,

    -- Final Timeout Action
    final_action TEXT NOT NULL CHECK (final_action IN ('AUTO_DENY', 'AUTO_APPROVE', 'BLOCK_INDEFINITELY')),

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Sample Data
INSERT INTO escalation_chains (chain_id, name, final_action, description) VALUES
('chain_cfo_escalation', 'CFO → CEO Escalation', 'AUTO_DENY', 'Escalate from CFO to CEO with auto-deny on final timeout');
```

#### 2.1.5 `escalation_tiers`

**Purpose:** Define individual tiers within escalation chains

```sql
CREATE TABLE escalation_tiers (
    -- Primary Key
    tier_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign Key to Chain
    chain_id TEXT NOT NULL REFERENCES escalation_chains(chain_id) ON DELETE CASCADE,

    -- Tier Configuration
    tier_index INTEGER NOT NULL,  -- 0-based index (0 = first tier)
    approvers JSONB NOT NULL,  -- Array of approver identifiers
    timeout_seconds INTEGER NOT NULL,  -- Timeout duration for this tier
    channels JSONB NOT NULL,  -- Array of channel types ["SLACK", "EMAIL"]

    -- Tier-Specific Quorum (optional override)
    quorum_type TEXT CHECK (quorum_type IN ('ANY', 'ALL', 'THRESHOLD')),
    quorum_value INTEGER,

    -- Unique constraint: one tier per index per chain
    UNIQUE(chain_id, tier_index)
);

-- Indexes
CREATE INDEX idx_tiers_chain_id ON escalation_tiers(chain_id);

-- Sample Data
INSERT INTO escalation_tiers (chain_id, tier_index, approvers, timeout_seconds, channels, quorum_type) VALUES
('chain_cfo_escalation', 0, '["cfo@company.com"]'::jsonb, 7200, '["SLACK", "EMAIL"]'::jsonb, 'ANY'),
('chain_cfo_escalation', 1, '["ceo@company.com"]'::jsonb, 14400, '["EMAIL", "SMS"]'::jsonb, 'ANY');
```

### 2.2 Audit and History Tables

#### 2.2.1 `notification_log`

**Purpose:** Track all notification deliveries with status

```sql
CREATE TABLE notification_log (
    -- Primary Key
    delivery_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign Key to Request
    request_id UUID NOT NULL REFERENCES oversight_requests(request_id) ON DELETE CASCADE,

    -- Delivery Details
    tier_index INTEGER NOT NULL,
    channel_type TEXT NOT NULL CHECK (channel_type IN ('SLACK', 'EMAIL', 'WEBHOOK')),
    recipient TEXT NOT NULL,  -- Email, Slack user ID, webhook URL
    message_id TEXT,  -- Channel-specific message identifier

    -- Status
    status TEXT NOT NULL CHECK (status IN ('DELIVERED', 'FAILED', 'PENDING', 'RETRYING')),
    error_message TEXT,  -- For FAILED status

    -- Timestamps
    delivered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    retry_count INTEGER NOT NULL DEFAULT 0
);

-- Indexes
CREATE INDEX idx_notification_log_request_id ON notification_log(request_id);
CREATE INDEX idx_notification_log_status ON notification_log(status);
CREATE INDEX idx_notification_log_channel_type ON notification_log(channel_type);
```

#### 2.2.2 `escalation_history`

**Purpose:** Track escalation events for audit trail

```sql
CREATE TABLE escalation_history (
    -- Primary Key
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign Key to Request
    request_id UUID NOT NULL REFERENCES oversight_requests(request_id) ON DELETE CASCADE,

    -- Escalation Details
    from_tier INTEGER NOT NULL,
    to_tier INTEGER NOT NULL,
    reason TEXT NOT NULL,  -- "TIER_TIMEOUT", "MANUAL_ESCALATION", etc.

    -- Timestamp
    escalated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_escalation_history_request_id ON escalation_history(request_id);
CREATE INDEX idx_escalation_history_escalated_at ON escalation_history(escalated_at);
```

#### 2.2.3 `approval_audit`

**Purpose:** Immutable audit trail for all state transitions

```sql
CREATE TABLE approval_audit (
    -- Primary Key
    audit_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Foreign Key to Request
    request_id UUID NOT NULL REFERENCES oversight_requests(request_id) ON DELETE RESTRICT,  -- Never delete audit records

    -- Event Details
    event_type TEXT NOT NULL,  -- "request.created", "request.approved", "request.denied", etc.
    agent_nhi TEXT NOT NULL,
    approver_subject TEXT,  -- NULL for non-response events

    -- State Transition
    old_state TEXT,
    new_state TEXT NOT NULL,

    -- Decision Context
    decision TEXT,  -- "APPROVE", "DENY", NULL
    signature_value BYTEA,  -- Cryptographic signature

    -- Merkle Proof (for immutability)
    merkle_proof BYTEA,  -- Proof linking this record to Merkle tree

    -- Timestamp
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_approval_audit_request_id ON approval_audit(request_id);
CREATE INDEX idx_approval_audit_event_type ON approval_audit(event_type);
CREATE INDEX idx_approval_audit_timestamp ON approval_audit(timestamp);
CREATE INDEX idx_approval_audit_approver ON approval_audit(approver_subject) WHERE approver_subject IS NOT NULL;

-- Prevent updates and deletes (immutable)
CREATE RULE approval_audit_immutable AS ON UPDATE TO approval_audit DO INSTEAD NOTHING;
CREATE RULE approval_audit_no_delete AS ON DELETE TO approval_audit DO INSTEAD NOTHING;
```

---

## 3. State Transition History

### 3.1 State Transition Tracking

Every state transition is logged to `approval_audit` table with before/after states:

```sql
-- Function to log state transition
CREATE OR REPLACE FUNCTION log_state_transition()
RETURNS TRIGGER AS $$
BEGIN
    -- Only log state changes
    IF NEW.state != OLD.state THEN
        INSERT INTO approval_audit (
            request_id,
            event_type,
            agent_nhi,
            old_state,
            new_state,
            timestamp
        ) VALUES (
            NEW.request_id,
            'request.state_transition',
            NEW.agent_nhi,
            OLD.state,
            NEW.state,
            NOW()
        );
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger on oversight_requests table
CREATE TRIGGER trigger_log_state_transition
AFTER UPDATE ON oversight_requests
FOR EACH ROW
EXECUTE FUNCTION log_state_transition();
```

### 3.2 State History Query

```sql
-- Query: Get complete state history for a request
SELECT
    audit_id,
    event_type,
    old_state,
    new_state,
    approver_subject,
    decision,
    timestamp
FROM approval_audit
WHERE request_id = 'a1b2c3d4-e5f6-7890-abcd-ef1234567890'
ORDER BY timestamp ASC;

-- Result:
-- audit_id                             | event_type              | old_state | new_state | approver_subject | decision | timestamp
-- -------------------------------------|-------------------------|-----------|-----------|------------------|----------|---------------------------
-- 123e4567-e89b-12d3-a456-426614174000 | request.created         | NULL      | PENDING   | NULL             | NULL     | 2024-12-25 14:00:00+00
-- 223e4567-e89b-12d3-a456-426614174001 | request.state_transition| PENDING   | APPROVED  | cfo@company.com  | APPROVE  | 2024-12-25 14:30:00+00
```

---

## 4. Redis Cache Design

### 4.1 Cache Schema

**Key Patterns:**
```
checkpoint:{request_id}        → JSON-serialized RequestState
policy:{policy_key_hash}       → JSON-serialized OversightRequirement
active_requests                → Set of request_ids in PENDING/ESCALATED state
approver:{approver_subject}    → Set of request_ids awaiting this approver
```

### 4.2 Cache Operations

```rust
// Cache checkpoint for fast lookup
pub async fn cache_checkpoint(
    &self,
    state: &RequestState,
) -> Result<()> {
    let key = format!("checkpoint:{}", state.request_id);
    let value = serde_json::to_string(state)?;

    self.redis.set_ex(
        key,
        value,
        3600,  // 1 hour TTL
    ).await?;

    // Add to active requests set if PENDING or ESCALATED
    if matches!(state.state, State::Pending | State::Escalated) {
        self.redis.sadd("active_requests", state.request_id.to_string()).await?;
    }

    Ok(())
}

// Query: Get all active requests
pub async fn get_active_requests(&self) -> Result<Vec<RequestId>> {
    let request_ids: Vec<String> = self.redis.smembers("active_requests").await?;
    Ok(request_ids.into_iter().map(|id| RequestId::from_str(&id).unwrap()).collect())
}

// Query: Get requests awaiting specific approver
pub async fn get_requests_for_approver(
    &self,
    approver: &str,
) -> Result<Vec<RequestId>> {
    let key = format!("approver:{}", approver);
    let request_ids: Vec<String> = self.redis.smembers(key).await?;
    Ok(request_ids.into_iter().map(|id| RequestId::from_str(&id).unwrap()).collect())
}
```

### 4.3 Cache Invalidation

**On State Transition:**
```rust
// Remove from active_requests set when reaching terminal state
if matches!(new_state, State::Approved | State::Denied | State::Timeout | State::Cancelled) {
    self.redis.srem("active_requests", request_id.to_string()).await?;
}

// Update checkpoint cache
self.cache_checkpoint(&state).await?;
```

**On Policy Update (via Pub/Sub):**
```rust
// Subscribe to policy update events
let mut pubsub = self.redis.get_async_connection().await?.into_pubsub();
pubsub.subscribe("policy_updates").await?;

while let Some(msg) = pubsub.on_message().next().await {
    let policy_id: String = msg.get_payload()?;

    // Invalidate cached policies
    let pattern = format!("policy:*:{}", policy_id);
    let keys: Vec<String> = self.redis.keys(pattern).await?;
    for key in keys {
        self.redis.del(key).await?;
    }
}
```

---

## 5. Data Retention and Archival

### 5.1 Retention Policy

| Table | Retention Period | Archival Strategy |
|-------|-----------------|-------------------|
| `oversight_requests` | 7 years (financial), 3 years (other) | Partition by year, move to cold storage |
| `approval_responses` | Same as requests | Foreign key cascade |
| `approval_audit` | Indefinite | Immutable, never delete |
| `notification_log` | 90 days | Delete after retention period |
| `escalation_history` | Same as requests | Foreign key cascade |

### 5.2 Partitioning Strategy

**Time-Based Partitioning (by created_at):**
```sql
-- Create parent table
CREATE TABLE oversight_requests (
    -- columns as defined above
) PARTITION BY RANGE (created_at);

-- Create partitions
CREATE TABLE oversight_requests_2024 PARTITION OF oversight_requests
FOR VALUES FROM ('2024-01-01') TO ('2025-01-01');

CREATE TABLE oversight_requests_2025 PARTITION OF oversight_requests
FOR VALUES FROM ('2025-01-01') TO ('2026-01-01');

-- Automatic partition creation (via pg_partman extension)
SELECT partman.create_parent('public.oversight_requests', 'created_at', 'native', 'yearly');
```

### 5.3 Archival Job

```sql
-- Archive completed requests older than retention period
INSERT INTO oversight_requests_archive
SELECT * FROM oversight_requests
WHERE state IN ('APPROVED', 'DENIED', 'TIMEOUT', 'CANCELLED')
  AND created_at < NOW() - INTERVAL '3 years'
  AND policy_id NOT LIKE 'pol_financial_%';  -- Financial requests have 7-year retention

-- Delete archived requests from main table
DELETE FROM oversight_requests
WHERE request_id IN (SELECT request_id FROM oversight_requests_archive);
```

---

## 6. Performance Optimizations

### 6.1 Index Strategy

**Covering Index for Active Requests Query:**
```sql
-- Covers: SELECT * FROM oversight_requests WHERE state IN ('PENDING', 'ESCALATED')
CREATE INDEX idx_requests_active_covering ON oversight_requests(state, request_id, updated_at)
INCLUDE (agent_nhi, action_description, tier_index)
WHERE state IN ('PENDING', 'ESCALATED');
```

**Composite Index for Approver Lookup:**
```sql
-- Query: Find all requests awaiting a specific approver
CREATE INDEX idx_responses_approver_decision ON approval_responses(approver_subject, decision)
WHERE decision IN ('APPROVE', 'DENY');
```

### 6.2 Query Optimization Examples

**Query 1: Get request with responses (JOIN)**
```sql
-- Efficient query using indexes
SELECT
    r.request_id,
    r.state,
    r.action_description,
    r.created_at,
    json_agg(
        json_build_object(
            'approver', resp.approver_subject,
            'decision', resp.decision,
            'responded_at', resp.responded_at
        )
    ) FILTER (WHERE resp.response_id IS NOT NULL) AS responses
FROM oversight_requests r
LEFT JOIN approval_responses resp ON r.request_id = resp.request_id
WHERE r.request_id = 'a1b2c3d4-e5f6-7890-abcd-ef1234567890'
GROUP BY r.request_id, r.state, r.action_description, r.created_at;

-- Query plan uses primary key index, O(1) lookup
```

**Query 2: Get all pending requests for dashboard**
```sql
-- Uses partial index idx_requests_active
SELECT
    request_id,
    agent_nhi,
    action_description,
    state,
    tier_index,
    updated_at
FROM oversight_requests
WHERE state IN ('PENDING', 'ESCALATED')
ORDER BY updated_at DESC
LIMIT 100;

-- Query plan: Index Scan using idx_requests_active (cost=0.15..8.17 rows=1 width=...)
```

**Query 3: Approver dashboard query**
```sql
-- Find all requests awaiting my approval
WITH my_requests AS (
    SELECT DISTINCT request_id
    FROM escalation_tiers et
    WHERE 'cfo@company.com' = ANY(et.approvers::text[])
)
SELECT
    r.request_id,
    r.action_description,
    r.reasoning,
    r.risk_factors,
    r.state,
    r.created_at,
    EXTRACT(EPOCH FROM (NOW() - r.created_at)) AS age_seconds
FROM oversight_requests r
INNER JOIN my_requests mr ON r.request_id = mr.request_id
WHERE r.state IN ('PENDING', 'ESCALATED')
  AND NOT EXISTS (
      SELECT 1 FROM approval_responses resp
      WHERE resp.request_id = r.request_id
        AND resp.approver_subject = 'cfo@company.com'
  )
ORDER BY r.created_at ASC;

-- Uses: idx_requests_active, idx_responses_unique_approver
```

### 6.3 Connection Pooling

**PgBouncer Configuration:**
```ini
[databases]
oversight_db = host=postgres-primary port=5432 dbname=creto_oversight

[pgbouncer]
pool_mode = transaction
max_client_conn = 1000
default_pool_size = 25
reserve_pool_size = 5
reserve_pool_timeout = 3
```

**Application-Side Pooling (SQLx):**
```rust
let pool = PgPoolOptions::new()
    .max_connections(25)
    .min_connections(5)
    .acquire_timeout(Duration::from_secs(3))
    .idle_timeout(Duration::from_secs(600))
    .max_lifetime(Duration::from_secs(1800))
    .connect(&database_url)
    .await?;
```

---

## 7. Data Consistency and Integrity

### 7.1 ACID Guarantees

**State Transition Transaction:**
```rust
pub async fn submit_response_transactional(
    &self,
    request_id: RequestId,
    response: ApprovalResponse,
) -> Result<StateTransitionOutcome> {
    // Begin transaction
    let mut tx = self.pool.begin().await?;

    // 1. Load current state with SELECT FOR UPDATE (row lock)
    let state = sqlx::query_as!(
        RequestState,
        r#"
        SELECT * FROM oversight_requests
        WHERE request_id = $1
        FOR UPDATE
        "#,
        request_id
    )
    .fetch_one(&mut *tx)
    .await?;

    // 2. Insert approval response
    sqlx::query!(
        r#"
        INSERT INTO approval_responses (
            request_id, approver_subject, decision, signature_value, ...
        ) VALUES ($1, $2, $3, $4, ...)
        "#,
        request_id,
        response.approver.subject,
        response.decision.to_string(),
        response.signature.value,
        // ... other fields
    )
    .execute(&mut *tx)
    .await?;

    // 3. Update request state
    let new_state = self.compute_new_state(&state, &response);
    sqlx::query!(
        r#"
        UPDATE oversight_requests
        SET state = $1, updated_at = $2, version = version + 1
        WHERE request_id = $3
        "#,
        new_state.to_string(),
        Timestamp::now(),
        request_id
    )
    .execute(&mut *tx)
    .await?;

    // 4. Insert audit record
    sqlx::query!(
        r#"
        INSERT INTO approval_audit (
            request_id, event_type, old_state, new_state, ...
        ) VALUES ($1, $2, $3, $4, ...)
        "#,
        request_id,
        "request.state_transition",
        state.state.to_string(),
        new_state.to_string(),
        // ... other fields
    )
    .execute(&mut *tx)
    .await?;

    // Commit transaction (atomic)
    tx.commit().await?;

    Ok(StateTransitionOutcome::Success { new_state })
}
```

### 7.2 Referential Integrity

**Foreign Key Constraints:**
- `approval_responses.request_id` → `oversight_requests.request_id` (CASCADE DELETE)
- `notification_log.request_id` → `oversight_requests.request_id` (CASCADE DELETE)
- `escalation_history.request_id` → `oversight_requests.request_id` (CASCADE DELETE)
- `approval_audit.request_id` → `oversight_requests.request_id` (RESTRICT DELETE - never delete audit)

### 7.3 Idempotency Constraints

**Unique Constraint on Responses:**
```sql
-- Prevent duplicate responses from same approver
CREATE UNIQUE INDEX idx_responses_unique_approver ON approval_responses(request_id, approver_subject);

-- Application-level handling:
INSERT INTO approval_responses (...)
VALUES (...)
ON CONFLICT (request_id, approver_subject) DO NOTHING;
```

---

## 8. Data Migration Strategy

### 8.1 Schema Versioning

Use **Flyway** or **sqlx-cli** for schema migrations:

```sql
-- V1__initial_schema.sql
CREATE TABLE oversight_requests (...);
CREATE TABLE approval_responses (...);
-- ... other tables

-- V2__add_risk_factors.sql
ALTER TABLE oversight_requests
ADD COLUMN risk_factors JSONB;

-- V3__add_escalation_history.sql
CREATE TABLE escalation_history (...);
```

### 8.2 Zero-Downtime Migrations

**Expand-Contract Pattern:**
1. **Expand:** Add new column/table (old code ignores it)
2. **Migrate:** Backfill data in background
3. **Contract:** Deploy new code using new schema
4. **Cleanup:** Drop old column/table

**Example: Add `reasoning` column**
```sql
-- Step 1: Expand (add column, nullable)
ALTER TABLE oversight_requests ADD COLUMN reasoning TEXT;

-- Step 2: Migrate (backfill from Memory service, background job)
UPDATE oversight_requests
SET reasoning = get_reasoning_from_memory(agent_nhi, action)
WHERE reasoning IS NULL;

-- Step 3: Contract (deploy code requiring reasoning)
-- Step 4: Cleanup (make column NOT NULL)
ALTER TABLE oversight_requests ALTER COLUMN reasoning SET NOT NULL;
```

---

## 9. Backup and Disaster Recovery

### 9.1 Backup Strategy

**PostgreSQL Continuous Archiving:**
```bash
# postgresql.conf
wal_level = replica
archive_mode = on
archive_command = 'cp %p /backup/wal/%f'

# Base backup (daily)
pg_basebackup -D /backup/base -Fp -Xs -P

# Point-in-time recovery (PITR)
# Can restore to any point in time with WAL replay
```

**Backup Schedule:**
- Full base backup: Daily at 02:00 UTC
- WAL archiving: Continuous (every 16 MB)
- Retention: 30 days

### 9.2 Recovery Procedures

**Scenario 1: Restore from corruption**
```bash
# Stop PostgreSQL
systemctl stop postgresql

# Restore base backup
rm -rf /var/lib/postgresql/data/*
tar -xzf /backup/base/backup-2024-12-25.tar.gz -C /var/lib/postgresql/data/

# Create recovery.conf
cat > /var/lib/postgresql/data/recovery.conf <<EOF
restore_command = 'cp /backup/wal/%f %p'
recovery_target_time = '2024-12-25 14:30:00'
EOF

# Start PostgreSQL (will replay WAL to recovery_target_time)
systemctl start postgresql
```

**Scenario 2: Restore single table**
```sql
-- Export from backup database
pg_dump -t oversight_requests backup_db > requests_backup.sql

-- Import to production
psql production_db < requests_backup.sql
```

---

## 10. Data Security

### 10.1 Encryption at Rest

**PostgreSQL Transparent Data Encryption (TDE):**
```bash
# Enable encryption for tablespace
CREATE TABLESPACE encrypted_tablespace
LOCATION '/var/lib/postgresql/encrypted'
WITH (encryption = 'AES256');

-- Create sensitive tables in encrypted tablespace
CREATE TABLE approval_responses (...) TABLESPACE encrypted_tablespace;
```

### 10.2 Column-Level Encryption

**Encrypt sensitive fields:**
```sql
-- Use pgcrypto extension
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Encrypt signature values
INSERT INTO approval_responses (signature_value, ...)
VALUES (
    pgp_sym_encrypt('signature_data', 'encryption_key'),
    ...
);

-- Decrypt on read
SELECT
    pgp_sym_decrypt(signature_value, 'encryption_key') AS signature,
    ...
FROM approval_responses;
```

### 10.3 Access Control

**Row-Level Security (RLS):**
```sql
-- Enable RLS
ALTER TABLE oversight_requests ENABLE ROW LEVEL SECURITY;

-- Policy: Approvers can only see requests they're eligible to approve
CREATE POLICY approver_access_policy ON oversight_requests
FOR SELECT
USING (
    EXISTS (
        SELECT 1 FROM escalation_tiers et
        WHERE et.chain_id = (SELECT escalation_chain_id FROM policies WHERE policy_id = oversight_requests.policy_id)
          AND current_user = ANY(et.approvers::text[])
    )
);
```

---

**END OF DOCUMENT**
