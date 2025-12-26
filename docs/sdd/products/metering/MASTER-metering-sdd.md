---
status: approved
version: 1.0.0
author: Architecture Team
created: 2025-12-25
updated: 2025-12-25
reviewers: [Security Team, Billing Operations, ML Infrastructure]
---

# MASTER Metering Software Design Document

**Authoritative Reference for creto-metering Implementation**

This document consolidates all Metering SDD sections into a single, complete reference. It addresses all P0 gaps from the OSS Alignment Report and provides implementation-ready specifications.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Multi-Tenant Hierarchy (P0 Gap Resolution)](#2-multi-tenant-hierarchy-p0-gap-resolution)
3. [Core Data Model](#3-core-data-model)
4. [Event Ingestion Pipeline](#4-event-ingestion-pipeline)
5. [Quota Enforcement System](#5-quota-enforcement-system)
6. [Aggregation Engine](#6-aggregation-engine)
7. [Pricing Models](#7-pricing-models)
8. [AI-Native Metrics (Flexprice Patterns)](#8-ai-native-metrics-flexprice-patterns)
9. [Cross-Product Attribution API](#9-cross-product-attribution-api)
10. [API Contracts](#10-api-contracts)
11. [Integration Contracts](#11-integration-contracts)
12. [Error Taxonomy](#12-error-taxonomy)
13. [Edge Cases & Failure Modes](#13-edge-cases--failure-modes)
14. [Sequence Diagrams](#14-sequence-diagrams)
15. [Performance Specifications](#15-performance-specifications)
16. [Security Model](#16-security-model)
17. [Operational Runbook](#17-operational-runbook)

---

## 1. Executive Summary

### 1.1 Purpose

Creto Metering provides **usage-based billing** and **quota enforcement** for AI agent workloads. It captures every billable event with cryptographic proof, enforces spending limits inline, and generates accurate invoices with full cost attribution.

### 1.2 Design Philosophy

**Pattern: Extract from OSS, Rebuild with Sovereign Primitives**

| OSS Reference | What We Extracted | Creto Enhancement |
|---------------|-------------------|-------------------|
| **Lago** | Event model, pricing types, aggregation | NHI attribution, ML-DSA signatures |
| **OpenMeter** | Real-time streaming, quota enforcement | 168ns inline authorization |
| **Flexprice** | AI billing patterns | Token metering, GPU accounting |
| **Stripe Billing** | Invoice workflows, webhook patterns | Cryptographic non-repudiation |

### 1.2.1 CRITICAL: Pattern Sources vs. Runtime Dependencies

> **⚠️ NO CODE FROM THESE OSS PROJECTS IS USED IN creto-metering**
>
> The table above shows **design pattern sources** studied during architecture.
> creto-metering is a **complete Rust rebuild** using only Creto Sovereign primitives.
>
> **Actual Dependencies (Cargo.toml):**
> - `creto-nhi` (agent identity)
> - `creto-crypto` (ML-DSA signatures)
> - `creto-authz` (168ns quota enforcement)
> - `creto-storage` (event persistence)
> - `creto-audit` (Merkle-anchored logs)
> - `creto-consensus` (ordering timestamps)
>
> **NOT Dependencies:** Lago, OpenMeter, Flexprice, Stripe SDK

### 1.3 Key Capabilities

| Capability | Specification | Notes |
|------------|---------------|-------|
| Event Ingestion | >10,000 events/sec | Batch + streaming |
| Quota Check Latency | <10us (cache hit) | Inline with AuthZ |
| Billing Accuracy | 99.999% | Zero double-billing |
| Attribution Depth | Full delegation chain | Agent -> Agent -> Human |
| Invoice Generation | <1s per subscription | Monthly/custom periods |

### 1.4 Creto Differentiators

1. **Agent NHI Attribution**: Every event tied to cryptographic agent identity
2. **Delegation Chain Billing**: Costs roll up through spawn hierarchy
3. **Cryptographic Verifiability**: ML-DSA-65 signatures on all events
4. **Consensus Timestamps**: Prevents clock manipulation attacks
5. **Inline Quota Enforcement**: Sub-20us total (AuthZ + Quota)

---

## 2. Multi-Tenant Hierarchy (P0 Gap Resolution)

### 2.1 Problem Statement

OSS metering tools (Lago, OpenMeter) assume flat tenant model:
- One customer = one subscription
- No concept of organizational hierarchy
- No quota inheritance

**Creto requires**:
- Platform -> Organization -> Team -> Agent hierarchy
- Quota inheritance from parent to child
- Aggregated billing at any level
- Per-agent cost attribution within organization

### 2.2 Hierarchical Data Model

```sql
-- ============================================================================
-- ORGANIZATION HIERARCHY (P0 Gap Resolution)
-- ============================================================================

-- Organizations table with parent reference for hierarchy
CREATE TABLE organizations (
    organization_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Hierarchy: NULL parent_id = root organization (Platform)
    parent_organization_id UUID REFERENCES organizations(organization_id),

    -- Organization details
    name TEXT NOT NULL,
    slug TEXT UNIQUE NOT NULL,  -- URL-safe identifier
    organization_type organization_type_enum NOT NULL,

    -- Billing configuration
    billing_email TEXT NOT NULL,
    stripe_customer_id TEXT,
    default_currency currency_code DEFAULT 'USD',

    -- Quota inheritance settings
    quota_inheritance_mode quota_inheritance_enum DEFAULT 'inherit_with_override',

    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ,  -- Soft delete

    -- Constraints
    CONSTRAINT valid_hierarchy CHECK (
        organization_id != parent_organization_id
    )
);

CREATE TYPE organization_type_enum AS ENUM (
    'platform',      -- Root: Creto Platform itself
    'enterprise',    -- Top-level customer
    'organization',  -- Department within enterprise
    'team',          -- Team within organization
    'project'        -- Project within team
);

CREATE TYPE quota_inheritance_enum AS ENUM (
    'inherit_strict',        -- Child gets exactly parent quota
    'inherit_with_override', -- Child can have lower limits (default)
    'independent'            -- No inheritance, explicit quotas only
);

-- Efficient hierarchy queries
CREATE INDEX idx_org_parent ON organizations(parent_organization_id);
CREATE INDEX idx_org_type ON organizations(organization_type);
CREATE INDEX idx_org_slug ON organizations(slug);

-- Materialized path for fast ancestor/descendant queries
ALTER TABLE organizations ADD COLUMN path ltree;
CREATE INDEX idx_org_path ON organizations USING GIST (path);

-- ============================================================================
-- SUBSCRIPTIONS (Linked to Organization)
-- ============================================================================

CREATE TABLE subscriptions (
    subscription_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Link to organization (not direct customer)
    organization_id UUID NOT NULL REFERENCES organizations(organization_id),

    -- Subscription details
    plan_id UUID NOT NULL REFERENCES plans(plan_id),
    status subscription_status DEFAULT 'active',

    -- Billing period
    billing_anchor_day INT CHECK (billing_anchor_day BETWEEN 1 AND 28),
    current_period_start TIMESTAMPTZ NOT NULL,
    current_period_end TIMESTAMPTZ NOT NULL,

    -- Stripe integration
    stripe_subscription_id TEXT,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TYPE subscription_status AS ENUM (
    'active', 'past_due', 'canceled', 'trialing', 'suspended'
);

-- ============================================================================
-- AGENT -> ORGANIZATION BINDING
-- ============================================================================

CREATE TABLE organization_agents (
    organization_id UUID NOT NULL REFERENCES organizations(organization_id),
    agent_nhi TEXT NOT NULL,  -- agent:nhi:ed25519:xxx

    -- Authorization
    authorized_by TEXT NOT NULL,  -- Human who added agent
    authorized_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,

    -- Role within organization
    role agent_role_enum DEFAULT 'member',

    PRIMARY KEY (organization_id, agent_nhi)
);

CREATE TYPE agent_role_enum AS ENUM (
    'owner',    -- Full control, can add agents
    'admin',    -- Manage quotas, view billing
    'member',   -- Submit events
    'readonly'  -- View only
);

CREATE INDEX idx_org_agents_nhi ON organization_agents(agent_nhi);
```

### 2.3 Quota Inheritance Logic

```rust
/// Quota inheritance for hierarchical organizations
pub struct QuotaInheritanceEngine {
    org_store: Arc<OrganizationStore>,
    quota_store: Arc<QuotaStore>,
}

impl QuotaInheritanceEngine {
    /// Resolve effective quota for an agent
    /// Traverses hierarchy from agent -> team -> org -> enterprise -> platform
    pub async fn resolve_effective_quota(
        &self,
        agent_nhi: &str,
        event_type: &str,
    ) -> Result<EffectiveQuota, QuotaError> {
        // Step 1: Find agent's organization
        let org = self.org_store.get_agent_organization(agent_nhi).await?;

        // Step 2: Collect quota chain (child to root)
        let mut quota_chain = Vec::new();
        let mut current_org = Some(org.clone());

        while let Some(org) = current_org {
            // Get quota at this level (if defined)
            if let Some(quota) = self.quota_store
                .get_quota(&org.organization_id, event_type)
                .await?
            {
                quota_chain.push(QuotaLevel {
                    organization_id: org.organization_id,
                    organization_type: org.organization_type,
                    quota,
                    inheritance_mode: org.quota_inheritance_mode,
                });
            }

            // Move to parent
            current_org = match org.parent_organization_id {
                Some(parent_id) => self.org_store.get_organization(&parent_id).await.ok(),
                None => None,
            };
        }

        // Step 3: Calculate effective quota based on inheritance rules
        self.calculate_effective_quota(&quota_chain, event_type)
    }

    fn calculate_effective_quota(
        &self,
        chain: &[QuotaLevel],
        event_type: &str,
    ) -> Result<EffectiveQuota, QuotaError> {
        if chain.is_empty() {
            return Err(QuotaError::NoQuotaDefined {
                event_type: event_type.to_string(),
            });
        }

        // Start from root (last in chain)
        let mut effective_limit: u64 = chain.last()
            .ok_or(QuotaError::EmptyChain)?
            .quota.limit_value;

        let mut effective_overflow = OverflowAction::Block;
        let mut source_org = chain.last().unwrap().organization_id;

        // Apply inheritance rules from root to leaf
        for level in chain.iter().rev().skip(1) {
            match level.inheritance_mode {
                QuotaInheritanceMode::InheritStrict => {
                    // Child must use parent quota exactly
                    // effective_limit unchanged
                }
                QuotaInheritanceMode::InheritWithOverride => {
                    // Child can reduce quota (but not exceed parent)
                    if level.quota.limit_value < effective_limit {
                        effective_limit = level.quota.limit_value;
                        source_org = level.organization_id;
                    }
                }
                QuotaInheritanceMode::Independent => {
                    // Child quota replaces parent completely
                    effective_limit = level.quota.limit_value;
                    source_org = level.organization_id;
                    effective_overflow = level.quota.overflow_action;
                }
            }
        }

        Ok(EffectiveQuota {
            limit: effective_limit,
            overflow_action: effective_overflow,
            source_organization_id: source_org,
            inheritance_chain: chain.iter()
                .map(|l| l.organization_id)
                .collect(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct EffectiveQuota {
    pub limit: u64,
    pub overflow_action: OverflowAction,
    pub source_organization_id: Uuid,
    pub inheritance_chain: Vec<Uuid>,
}
```

### 2.4 Hierarchical Usage Aggregation

```sql
-- Usage aggregation at any hierarchy level
CREATE OR REPLACE FUNCTION get_hierarchical_usage(
    p_organization_id UUID,
    p_event_type TEXT,
    p_period_start TIMESTAMPTZ,
    p_period_end TIMESTAMPTZ,
    p_include_descendants BOOLEAN DEFAULT true
) RETURNS TABLE (
    organization_id UUID,
    organization_name TEXT,
    organization_type organization_type_enum,
    depth INT,
    event_count BIGINT,
    total_value NUMERIC,
    agent_count BIGINT
) AS $$
BEGIN
    RETURN QUERY
    WITH RECURSIVE org_tree AS (
        -- Base: target organization
        SELECT
            o.organization_id,
            o.name,
            o.organization_type,
            0 AS depth,
            o.path
        FROM organizations o
        WHERE o.organization_id = p_organization_id

        UNION ALL

        -- Recursive: descendants (if requested)
        SELECT
            child.organization_id,
            child.name,
            child.organization_type,
            parent.depth + 1,
            child.path
        FROM organizations child
        JOIN org_tree parent ON child.parent_organization_id = parent.organization_id
        WHERE p_include_descendants = true
    ),
    org_usage AS (
        SELECT
            ot.organization_id,
            ot.name AS organization_name,
            ot.organization_type,
            ot.depth,
            COALESCE(SUM(uh.event_count), 0) AS event_count,
            COALESCE(SUM(uh.total_value), 0) AS total_value,
            COUNT(DISTINCT e.agent_nhi) AS agent_count
        FROM org_tree ot
        LEFT JOIN subscriptions s ON s.organization_id = ot.organization_id
        LEFT JOIN usage_hourly uh ON uh.subscription_id = s.subscription_id
            AND uh.event_type = p_event_type
            AND uh.hour >= p_period_start
            AND uh.hour < p_period_end
        LEFT JOIN events e ON e.subscription_id = s.subscription_id
            AND e.event_type = p_event_type
            AND e.timestamp >= p_period_start
            AND e.timestamp < p_period_end
        GROUP BY ot.organization_id, ot.name, ot.organization_type, ot.depth
    )
    SELECT * FROM org_usage
    ORDER BY depth, organization_name;
END;
$$ LANGUAGE plpgsql;

-- Example: Get all usage under an enterprise
SELECT * FROM get_hierarchical_usage(
    'enterprise-abc-uuid',
    'llm_tokens',
    '2024-12-01',
    '2025-01-01',
    true
);
```

---

## 3. Core Data Model

### 3.1 Events Table

```sql
CREATE TABLE events (
    -- Primary key: partitioned by month
    event_id UUID NOT NULL,

    -- Tenant (resolved from agent)
    subscription_id UUID NOT NULL REFERENCES subscriptions(subscription_id),
    organization_id UUID NOT NULL REFERENCES organizations(organization_id),

    -- Agent identity (NHI)
    agent_nhi TEXT NOT NULL,
    delegation_chain JSONB NOT NULL DEFAULT '[]',

    -- Event classification
    event_type TEXT NOT NULL,

    -- Timestamps
    agent_timestamp TIMESTAMPTZ,  -- Agent-provided (informational)
    consensus_timestamp TIMESTAMPTZ NOT NULL,  -- Consensus-ordered (authoritative)

    -- Event data
    properties JSONB NOT NULL DEFAULT '{}',

    -- Cryptographic proof
    signature BYTEA NOT NULL,
    signature_algorithm TEXT NOT NULL DEFAULT 'ML-DSA-65',

    -- Idempotency
    idempotency_key TEXT NOT NULL,
    idempotency_hash BYTEA NOT NULL,  -- SHA3-256 of canonical event

    -- Audit
    audit_log_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Partition key
    PRIMARY KEY (event_id, consensus_timestamp)
) PARTITION BY RANGE (consensus_timestamp);

-- Monthly partitions (auto-created by pg_partman)
CREATE TABLE events_2024_12 PARTITION OF events
    FOR VALUES FROM ('2024-12-01') TO ('2025-01-01');

-- Indexes for common queries
CREATE INDEX idx_events_subscription ON events(subscription_id, event_type, consensus_timestamp);
CREATE INDEX idx_events_agent ON events(agent_nhi, consensus_timestamp);
CREATE INDEX idx_events_idempotency ON events(idempotency_key);
CREATE INDEX idx_events_org ON events(organization_id, event_type, consensus_timestamp);
```

### 3.2 Quotas Table

```sql
CREATE TABLE quotas (
    quota_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Target (organization OR subscription)
    organization_id UUID REFERENCES organizations(organization_id),
    subscription_id UUID REFERENCES subscriptions(subscription_id),

    -- Quota definition
    event_type TEXT NOT NULL,
    limit_value BIGINT NOT NULL,
    period quota_period NOT NULL,
    overflow_action overflow_action_enum NOT NULL DEFAULT 'block',

    -- Effective dates
    effective_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    effective_until TIMESTAMPTZ,

    -- Metadata
    created_by TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Constraint: must have either org or subscription
    CONSTRAINT quota_target CHECK (
        (organization_id IS NOT NULL AND subscription_id IS NULL) OR
        (organization_id IS NULL AND subscription_id IS NOT NULL)
    )
);

CREATE TYPE quota_period AS ENUM (
    'hourly', 'daily', 'weekly', 'monthly', 'total'
);

CREATE TYPE overflow_action_enum AS ENUM (
    'block',              -- Deny requests
    'allow_with_overage', -- Allow but charge premium
    'notify_only',        -- Soft limit, send alert
    'throttle'            -- Reduce rate
);

CREATE INDEX idx_quotas_org ON quotas(organization_id, event_type) WHERE organization_id IS NOT NULL;
CREATE INDEX idx_quotas_sub ON quotas(subscription_id, event_type) WHERE subscription_id IS NOT NULL;
```

### 3.3 Usage Aggregation Tables

```sql
-- Hourly rollup (hot data: 7 days)
CREATE TABLE usage_hourly (
    subscription_id UUID NOT NULL,
    organization_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    hour TIMESTAMPTZ NOT NULL,

    -- Aggregations
    event_count BIGINT NOT NULL DEFAULT 0,
    sum_value NUMERIC(20, 6) DEFAULT 0,
    unique_count INT DEFAULT 0,
    max_value NUMERIC(20, 6) DEFAULT 0,

    -- Agent breakdown (top 100 agents)
    agent_breakdown JSONB DEFAULT '{}',

    -- Dimension breakdown
    dimension_breakdown JSONB DEFAULT '{}',

    -- Metadata
    last_event_id UUID,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (subscription_id, event_type, hour)
);

-- Daily rollup (warm data: 90 days)
CREATE TABLE usage_daily (
    subscription_id UUID NOT NULL,
    organization_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    date DATE NOT NULL,

    event_count BIGINT NOT NULL DEFAULT 0,
    sum_value NUMERIC(20, 6) DEFAULT 0,
    unique_count INT DEFAULT 0,
    max_value NUMERIC(20, 6) DEFAULT 0,

    agent_breakdown JSONB DEFAULT '{}',
    dimension_breakdown JSONB DEFAULT '{}',

    PRIMARY KEY (subscription_id, event_type, date)
);

-- Monthly rollup (cold data: indefinite)
CREATE TABLE usage_monthly (
    subscription_id UUID NOT NULL,
    organization_id UUID NOT NULL,
    event_type TEXT NOT NULL,
    month DATE NOT NULL,

    event_count BIGINT NOT NULL DEFAULT 0,
    sum_value NUMERIC(20, 6) DEFAULT 0,
    unique_count INT DEFAULT 0,
    max_value NUMERIC(20, 6) DEFAULT 0,

    -- Full attribution for billing
    agent_attribution JSONB DEFAULT '{}',  -- {agent_nhi: {count, sum, cost}}
    dimension_attribution JSONB DEFAULT '{}',

    PRIMARY KEY (subscription_id, event_type, month)
);
```

---

## 4. Event Ingestion Pipeline

### 4.1 Pipeline Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                          EVENT INGESTION PIPELINE                                │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────┐    ┌─────────────────┐   │
│  │  Agent   │───▶│   Validate   │───▶│   Verify     │───▶│   Deduplicate   │   │
│  │  Event   │    │   Schema     │    │   Signature  │    │   (Idempotency) │   │
│  └──────────┘    └──────────────┘    └──────────────┘    └────────┬────────┘   │
│                                                                    │            │
│                                                                    ▼            │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────┐    ┌─────────────────┐   │
│  │  Audit   │◀───│   Persist    │◀───│   Assign     │◀───│   Resolve       │   │
│  │  Log     │    │   to DB      │    │   Timestamp  │    │   Subscription  │   │
│  └──────────┘    └──────────────┘    └──────────────┘    └─────────────────┘   │
│       │                │                                                        │
│       │                ▼                                                        │
│       │         ┌──────────────┐    ┌──────────────┐    ┌─────────────────┐   │
│       │         │   Kafka      │───▶│  Aggregation │───▶│   Update        │   │
│       │         │   Publish    │    │   Worker     │    │   Hourly Table  │   │
│       │         └──────────────┘    └──────────────┘    └─────────────────┘   │
│       │                                                                        │
│       └────────────────────────────────────────────────────────────────────────│
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 Event Validation Implementation

```rust
use chrono::{DateTime, Duration, Utc};
use serde_json::Value as JsonValue;

pub struct EventValidator {
    max_timestamp_skew: Duration,
    max_properties_depth: usize,
    max_properties_size: usize,
    allowed_event_types: HashSet<String>,
}

impl EventValidator {
    /// Validate event schema and constraints
    pub fn validate(&self, event: &EventRequest) -> Result<(), ValidationError> {
        // 1. Required fields
        self.validate_required_fields(event)?;

        // 2. Agent NHI format
        self.validate_nhi_format(&event.agent_nhi)?;

        // 3. Event type whitelist
        self.validate_event_type(&event.event_type)?;

        // 4. Timestamp skew (if provided)
        if let Some(ts) = event.timestamp {
            self.validate_timestamp_skew(ts)?;
        }

        // 5. Properties constraints
        self.validate_properties(&event.properties)?;

        // 6. Signature present
        if event.signature.is_empty() {
            return Err(ValidationError::MissingSignature);
        }

        Ok(())
    }

    fn validate_required_fields(&self, event: &EventRequest) -> Result<(), ValidationError> {
        if event.idempotency_key.is_empty() {
            return Err(ValidationError::MissingField {
                field: "idempotency_key".to_string(),
            });
        }
        if event.agent_nhi.is_empty() {
            return Err(ValidationError::MissingField {
                field: "agent_nhi".to_string(),
            });
        }
        if event.event_type.is_empty() {
            return Err(ValidationError::MissingField {
                field: "event_type".to_string(),
            });
        }
        Ok(())
    }

    fn validate_nhi_format(&self, nhi: &str) -> Result<(), ValidationError> {
        // Format: agent:nhi:<algorithm>:<public_key_hash>
        let parts: Vec<&str> = nhi.split(':').collect();
        if parts.len() != 4 {
            return Err(ValidationError::InvalidNhiFormat {
                nhi: nhi.to_string(),
                expected: "agent:nhi:<algorithm>:<hash>".to_string(),
            });
        }
        if parts[0] != "agent" || parts[1] != "nhi" {
            return Err(ValidationError::InvalidNhiFormat {
                nhi: nhi.to_string(),
                expected: "Must start with 'agent:nhi:'".to_string(),
            });
        }
        Ok(())
    }

    fn validate_timestamp_skew(&self, ts: DateTime<Utc>) -> Result<(), ValidationError> {
        let now = Utc::now();
        let skew = if ts > now { ts - now } else { now - ts };

        if skew > self.max_timestamp_skew {
            return Err(ValidationError::TimestampSkew {
                agent_timestamp: ts,
                server_timestamp: now,
                skew: skew.to_std().unwrap_or_default(),
                max_allowed: self.max_timestamp_skew.to_std().unwrap_or_default(),
            });
        }
        Ok(())
    }

    fn validate_properties(&self, props: &JsonValue) -> Result<(), ValidationError> {
        // Check size
        let size = serde_json::to_vec(props)
            .map(|v| v.len())
            .unwrap_or(0);

        if size > self.max_properties_size {
            return Err(ValidationError::PropertiesTooLarge {
                size,
                max_size: self.max_properties_size,
            });
        }

        // Check depth
        let depth = self.calculate_depth(props, 0);
        if depth > self.max_properties_depth {
            return Err(ValidationError::PropertiesTooDeep {
                depth,
                max_depth: self.max_properties_depth,
            });
        }

        Ok(())
    }

    fn calculate_depth(&self, value: &JsonValue, current: usize) -> usize {
        match value {
            JsonValue::Object(map) => {
                map.values()
                    .map(|v| self.calculate_depth(v, current + 1))
                    .max()
                    .unwrap_or(current + 1)
            }
            JsonValue::Array(arr) => {
                arr.iter()
                    .map(|v| self.calculate_depth(v, current + 1))
                    .max()
                    .unwrap_or(current + 1)
            }
            _ => current,
        }
    }
}
```

### 4.3 Signature Verification

```rust
use creto_crypto::{CryptoProvider, SignatureAlgorithm, VerifyingKey};

pub struct SignatureVerifier {
    crypto: Arc<dyn CryptoProvider>,
    nhi_registry: Arc<NhiRegistry>,
}

impl SignatureVerifier {
    /// Verify event signature using agent's NHI public key
    pub async fn verify_event_signature(
        &self,
        event: &EventRequest,
    ) -> Result<VerificationResult, SignatureError> {
        // 1. Extract public key from NHI registry
        let public_key = self.nhi_registry
            .get_public_key(&event.agent_nhi)
            .await
            .map_err(|_| SignatureError::AgentNotFound {
                agent_nhi: event.agent_nhi.clone(),
            })?;

        // 2. Determine signature algorithm
        let algorithm = SignatureAlgorithm::from_str(&event.signature_algorithm)
            .map_err(|_| SignatureError::UnsupportedAlgorithm {
                algorithm: event.signature_algorithm.clone(),
            })?;

        // 3. Construct canonical message (deterministic serialization)
        let canonical_message = self.construct_canonical_message(event)?;

        // 4. Verify signature
        let valid = self.crypto.verify_signature(
            &public_key,
            &canonical_message,
            &event.signature,
            algorithm,
        )?;

        if !valid {
            return Err(SignatureError::VerificationFailed {
                agent_nhi: event.agent_nhi.clone(),
                algorithm: event.signature_algorithm.clone(),
            });
        }

        Ok(VerificationResult {
            agent_nhi: event.agent_nhi.clone(),
            algorithm,
            verified_at: Utc::now(),
        })
    }

    /// Canonical message format for signing
    fn construct_canonical_message(&self, event: &EventRequest) -> Result<Vec<u8>, SignatureError> {
        // Deterministic JSON serialization (sorted keys)
        let canonical = CanonicalEvent {
            idempotency_key: &event.idempotency_key,
            agent_nhi: &event.agent_nhi,
            delegation_chain: &event.delegation_chain,
            event_type: &event.event_type,
            timestamp: event.timestamp,
            properties: &event.properties,
        };

        serde_json::to_vec(&canonical)
            .map_err(|e| SignatureError::SerializationError(e.to_string()))
    }
}

#[derive(Serialize)]
struct CanonicalEvent<'a> {
    idempotency_key: &'a str,
    agent_nhi: &'a str,
    delegation_chain: &'a [String],
    event_type: &'a str,
    timestamp: Option<DateTime<Utc>>,
    #[serde(serialize_with = "ordered_json")]
    properties: &'a JsonValue,
}

/// Serialize JSON with sorted keys (deterministic)
fn ordered_json<S>(value: &JsonValue, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // Implementation: sort keys recursively
    let sorted = sort_json_keys(value);
    sorted.serialize(serializer)
}
```

### 4.4 Idempotency Handling

```rust
use sha3::{Sha3_256, Digest};

pub struct IdempotencyHandler {
    cache: Arc<RedisPool>,
    db: Arc<DatabasePool>,
    ttl: Duration,  // 7 days
}

impl IdempotencyHandler {
    /// Check if event is duplicate, return existing event_id if so
    pub async fn check_idempotency(
        &self,
        event: &EventRequest,
    ) -> Result<IdempotencyResult, IdempotencyError> {
        let hash = self.compute_event_hash(event);

        // 1. Check Redis cache first (fast path)
        if let Some(existing) = self.check_cache(&event.idempotency_key).await? {
            if existing.hash == hash {
                // Exact duplicate - return existing event_id
                return Ok(IdempotencyResult::Duplicate {
                    existing_event_id: existing.event_id,
                });
            } else {
                // Same key, different content - conflict!
                return Err(IdempotencyError::Conflict {
                    idempotency_key: event.idempotency_key.clone(),
                    existing_hash: hex::encode(&existing.hash),
                    submitted_hash: hex::encode(&hash),
                });
            }
        }

        // 2. Check database (slow path, for cache misses)
        if let Some(existing) = self.check_database(&event.idempotency_key).await? {
            // Cache miss but exists in DB - update cache
            self.update_cache(&event.idempotency_key, &existing).await?;

            if existing.idempotency_hash == hash {
                return Ok(IdempotencyResult::Duplicate {
                    existing_event_id: existing.event_id,
                });
            } else {
                return Err(IdempotencyError::Conflict {
                    idempotency_key: event.idempotency_key.clone(),
                    existing_hash: hex::encode(&existing.idempotency_hash),
                    submitted_hash: hex::encode(&hash),
                });
            }
        }

        // 3. New event
        Ok(IdempotencyResult::New { hash })
    }

    fn compute_event_hash(&self, event: &EventRequest) -> [u8; 32] {
        let mut hasher = Sha3_256::new();

        // Hash canonical representation (excludes server-assigned fields)
        hasher.update(event.idempotency_key.as_bytes());
        hasher.update(event.agent_nhi.as_bytes());
        hasher.update(event.event_type.as_bytes());

        // Hash properties (sorted JSON)
        let props_json = serde_json::to_string(&sort_json_keys(&event.properties))
            .unwrap_or_default();
        hasher.update(props_json.as_bytes());

        hasher.finalize().into()
    }
}

pub enum IdempotencyResult {
    New { hash: [u8; 32] },
    Duplicate { existing_event_id: Uuid },
}
```

---

## 5. Quota Enforcement System

### 5.1 Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           QUOTA ENFORCEMENT SYSTEM                               │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                  │
│   Authorization Service                  Metering Service                        │
│   ┌─────────────────────┐               ┌─────────────────────────────────────┐ │
│   │                     │    gRPC       │                                     │ │
│   │  check_authorization│──────────────▶│  QuotaEnforcer.check_quota()       │ │
│   │      (168ns)        │               │         (<10µs)                     │ │
│   │                     │◀──────────────│                                     │ │
│   │                     │   Allow/Deny  │                                     │ │
│   └─────────────────────┘               └──────────────┬──────────────────────┘ │
│                                                        │                         │
│                                                        ▼                         │
│                                         ┌──────────────────────────────────────┐│
│                                         │         Quota Resolution              ││
│                                         │  ┌────────────────────────────────┐  ││
│                                         │  │ 1. Check L1 Cache (in-memory)  │  ││
│                                         │  │    TTL: 5 seconds              │  ││
│                                         │  │    Hit rate: ~80%              │  ││
│                                         │  └────────────┬───────────────────┘  ││
│                                         │               │ Miss                  ││
│                                         │               ▼                       ││
│                                         │  ┌────────────────────────────────┐  ││
│                                         │  │ 2. Check L2 Cache (Redis)      │  ││
│                                         │  │    TTL: 60 seconds             │  ││
│                                         │  │    Hit rate: ~95%              │  ││
│                                         │  └────────────┬───────────────────┘  ││
│                                         │               │ Miss                  ││
│                                         │               ▼                       ││
│                                         │  ┌────────────────────────────────┐  ││
│                                         │  │ 3. Query Database              │  ││
│                                         │  │    + Populate caches           │  ││
│                                         │  └────────────────────────────────┘  ││
│                                         └──────────────────────────────────────┘│
│                                                                                  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### 5.2 QuotaEnforcer Implementation

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub struct QuotaEnforcer {
    // L1 Cache: In-memory (per-process)
    l1_cache: Arc<DashMap<QuotaCacheKey, CachedQuota>>,
    l1_ttl: Duration,

    // L2 Cache: Redis (shared)
    redis: Arc<RedisPool>,
    l2_ttl: Duration,

    // Database
    db: Arc<DatabasePool>,

    // Inheritance engine
    inheritance_engine: Arc<QuotaInheritanceEngine>,

    // Metrics
    metrics: QuotaMetrics,
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct QuotaCacheKey {
    agent_nhi: String,
    event_type: String,
    period_key: String,  // e.g., "2024-12-25T10" for hourly
}

#[derive(Clone)]
struct CachedQuota {
    effective_quota: EffectiveQuota,
    current_usage: AtomicU64,
    cached_at: Instant,
}

impl QuotaEnforcer {
    /// Check quota for agent - called inline from Authorization
    /// Target latency: <10µs for cache hit
    pub async fn check_quota(
        &self,
        agent_nhi: &str,
        event_type: &str,
    ) -> Result<QuotaDecision, QuotaError> {
        let start = Instant::now();

        // Resolve effective quota (with hierarchy)
        let effective = self.inheritance_engine
            .resolve_effective_quota(agent_nhi, event_type)
            .await?;

        // Build cache key
        let cache_key = QuotaCacheKey {
            agent_nhi: agent_nhi.to_string(),
            event_type: event_type.to_string(),
            period_key: self.get_period_key(&effective.period),
        };

        // L1 Cache check
        if let Some(cached) = self.check_l1_cache(&cache_key) {
            self.metrics.l1_hits.fetch_add(1, Ordering::Relaxed);
            return self.evaluate_cached_quota(&cached, &effective);
        }

        // L2 Cache check (Redis)
        if let Some(usage) = self.check_l2_cache(&cache_key).await? {
            self.metrics.l2_hits.fetch_add(1, Ordering::Relaxed);
            let cached = self.populate_l1_cache(&cache_key, &effective, usage);
            return self.evaluate_cached_quota(&cached, &effective);
        }

        // Database query (slow path)
        self.metrics.db_queries.fetch_add(1, Ordering::Relaxed);
        let usage = self.query_current_usage(agent_nhi, event_type, &effective.period).await?;

        // Populate both caches
        let cached = self.populate_caches(&cache_key, &effective, usage).await?;

        let decision = self.evaluate_cached_quota(&cached, &effective);

        self.metrics.latency_histogram.record(start.elapsed());

        decision
    }

    fn evaluate_cached_quota(
        &self,
        cached: &CachedQuota,
        effective: &EffectiveQuota,
    ) -> Result<QuotaDecision, QuotaError> {
        let current_usage = cached.current_usage.load(Ordering::Relaxed);
        let remaining = effective.limit.saturating_sub(current_usage);

        if current_usage < effective.limit {
            Ok(QuotaDecision::Allow {
                remaining,
                limit: effective.limit,
                period_end: self.get_period_end(&effective.period),
                source_organization_id: effective.source_organization_id,
            })
        } else {
            // Quota exceeded - apply overflow action
            match effective.overflow_action {
                OverflowAction::Block => {
                    Ok(QuotaDecision::Deny {
                        reason: QuotaExceededReason::LimitReached,
                        current_usage,
                        limit: effective.limit,
                        retry_after: self.calculate_retry_after(&effective.period),
                    })
                }
                OverflowAction::AllowWithOverage => {
                    Ok(QuotaDecision::AllowWithOverage {
                        overage_units: current_usage - effective.limit + 1,
                        overage_rate: self.get_overage_rate(&effective.event_type),
                    })
                }
                OverflowAction::NotifyOnly => {
                    Ok(QuotaDecision::AllowWithWarning {
                        remaining: 0,
                        warning: "Quota exceeded, usage will be billed at overage rate".to_string(),
                    })
                }
                OverflowAction::Throttle => {
                    Ok(QuotaDecision::Throttle {
                        delay_ms: self.calculate_throttle_delay(current_usage, effective.limit),
                    })
                }
            }
        }
    }

    /// Reserve quota for upcoming action (two-phase: reserve -> commit/rollback)
    pub async fn reserve_quota(
        &self,
        agent_nhi: &str,
        event_type: &str,
        quantity: u64,
    ) -> Result<QuotaReservation, QuotaError> {
        // Check if quota available
        let decision = self.check_quota(agent_nhi, event_type).await?;

        match decision {
            QuotaDecision::Allow { remaining, .. } if remaining >= quantity => {
                // Create reservation
                let reservation_id = Uuid::new_v4();
                let expires_at = Utc::now() + chrono::Duration::minutes(5);

                // Store reservation in Redis (atomic)
                self.redis.set_ex(
                    format!("quota_reservation:{}", reservation_id),
                    serde_json::to_string(&QuotaReservationData {
                        agent_nhi: agent_nhi.to_string(),
                        event_type: event_type.to_string(),
                        quantity,
                        expires_at,
                    })?,
                    300, // 5 minutes
                ).await?;

                // Increment usage atomically (optimistic)
                self.increment_usage(agent_nhi, event_type, quantity).await?;

                Ok(QuotaReservation {
                    reservation_id,
                    quantity,
                    expires_at,
                })
            }
            QuotaDecision::Allow { remaining, .. } => {
                Err(QuotaError::InsufficientQuota {
                    requested: quantity,
                    available: remaining,
                })
            }
            QuotaDecision::Deny { reason, .. } => {
                Err(QuotaError::QuotaExceeded { reason })
            }
            _ => {
                Err(QuotaError::ReservationNotSupported)
            }
        }
    }

    /// Commit reservation (deduct quota permanently)
    pub async fn commit_reservation(&self, reservation_id: Uuid) -> Result<(), QuotaError> {
        // Just delete reservation - usage already incremented
        self.redis.del(format!("quota_reservation:{}", reservation_id)).await?;
        Ok(())
    }

    /// Rollback reservation (return quota)
    pub async fn rollback_reservation(&self, reservation_id: Uuid) -> Result<(), QuotaError> {
        // Get reservation data
        let reservation_key = format!("quota_reservation:{}", reservation_id);
        let data: Option<QuotaReservationData> = self.redis
            .get(&reservation_key)
            .await?
            .and_then(|s: String| serde_json::from_str(&s).ok());

        if let Some(data) = data {
            // Decrement usage
            self.decrement_usage(&data.agent_nhi, &data.event_type, data.quantity).await?;

            // Delete reservation
            self.redis.del(&reservation_key).await?;
        }

        Ok(())
    }
}
```

---

## 6. Aggregation Engine

### 6.1 Real-Time Aggregation

```rust
pub struct AggregationEngine {
    db: Arc<DatabasePool>,
    kafka_consumer: Arc<KafkaConsumer>,
    aggregator_cache: Arc<DashMap<AggregatorKey, InMemoryAggregator>>,
    flush_interval: Duration,
}

impl AggregationEngine {
    /// Process event and update aggregations
    pub async fn process_event(&self, event: &Event) -> Result<(), AggregationError> {
        // Get aggregator key
        let key = AggregatorKey {
            subscription_id: event.subscription_id,
            event_type: event.event_type.clone(),
            hour: event.consensus_timestamp.date_naive()
                .and_hms_opt(event.consensus_timestamp.hour(), 0, 0)
                .unwrap()
                .and_utc(),
        };

        // Update in-memory aggregator
        let aggregator = self.aggregator_cache
            .entry(key.clone())
            .or_insert_with(|| InMemoryAggregator::new(&key));

        aggregator.accumulate(event)?;

        // Flush if needed
        if aggregator.should_flush() {
            self.flush_aggregator(&key, &aggregator).await?;
        }

        Ok(())
    }

    /// Flush in-memory aggregator to database
    async fn flush_aggregator(
        &self,
        key: &AggregatorKey,
        aggregator: &InMemoryAggregator,
    ) -> Result<(), AggregationError> {
        let state = aggregator.get_state();

        sqlx::query(r#"
            INSERT INTO usage_hourly (
                subscription_id, organization_id, event_type, hour,
                event_count, sum_value, unique_count, max_value,
                agent_breakdown, dimension_breakdown, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
            ON CONFLICT (subscription_id, event_type, hour)
            DO UPDATE SET
                event_count = usage_hourly.event_count + EXCLUDED.event_count,
                sum_value = usage_hourly.sum_value + EXCLUDED.sum_value,
                unique_count = (
                    SELECT COUNT(DISTINCT value)
                    FROM jsonb_array_elements_text(
                        usage_hourly.dimension_breakdown || EXCLUDED.dimension_breakdown
                    )
                ),
                max_value = GREATEST(usage_hourly.max_value, EXCLUDED.max_value),
                agent_breakdown = usage_hourly.agent_breakdown || EXCLUDED.agent_breakdown,
                dimension_breakdown = usage_hourly.dimension_breakdown || EXCLUDED.dimension_breakdown,
                updated_at = now()
        "#)
        .bind(&key.subscription_id)
        .bind(&state.organization_id)
        .bind(&key.event_type)
        .bind(&key.hour)
        .bind(state.event_count as i64)
        .bind(&state.sum_value)
        .bind(state.unique_values.len() as i32)
        .bind(&state.max_value)
        .bind(&state.agent_breakdown)
        .bind(&state.dimension_breakdown)
        .execute(&*self.db)
        .await?;

        aggregator.reset();

        Ok(())
    }
}

struct InMemoryAggregator {
    event_count: AtomicU64,
    sum_value: Mutex<Decimal>,
    unique_values: Mutex<HashSet<String>>,
    max_value: Mutex<Decimal>,
    agent_counts: DashMap<String, u64>,
    dimension_counts: DashMap<String, DashMap<String, u64>>,
    last_flush: Mutex<Instant>,
    flush_threshold: u64,
}

impl InMemoryAggregator {
    fn accumulate(&self, event: &Event) -> Result<(), AggregationError> {
        // Count
        self.event_count.fetch_add(1, Ordering::Relaxed);

        // Sum (extract from properties based on metric config)
        if let Some(value) = event.properties.get("value").and_then(|v| v.as_f64()) {
            let mut sum = self.sum_value.lock().unwrap();
            *sum += Decimal::from_f64(value).unwrap_or_default();
        }

        // Max
        if let Some(value) = event.properties.get("value").and_then(|v| v.as_f64()) {
            let mut max = self.max_value.lock().unwrap();
            let decimal = Decimal::from_f64(value).unwrap_or_default();
            if decimal > *max {
                *max = decimal;
            }
        }

        // Unique count
        if let Some(unique_field) = event.properties.get("unique_field") {
            let mut unique = self.unique_values.lock().unwrap();
            unique.insert(unique_field.to_string());
        }

        // Agent breakdown
        *self.agent_counts.entry(event.agent_nhi.clone()).or_insert(0) += 1;

        // Dimension breakdown (e.g., by model, region)
        for (dim_key, dim_value) in event.properties.as_object().unwrap_or(&serde_json::Map::new()) {
            if let Some(value_str) = dim_value.as_str() {
                let dim_map = self.dimension_counts
                    .entry(dim_key.clone())
                    .or_insert_with(DashMap::new);
                *dim_map.entry(value_str.to_string()).or_insert(0) += 1;
            }
        }

        Ok(())
    }

    fn should_flush(&self) -> bool {
        let count = self.event_count.load(Ordering::Relaxed);
        let elapsed = self.last_flush.lock().unwrap().elapsed();

        count >= self.flush_threshold || elapsed >= Duration::from_secs(5)
    }
}
```

---

## 7. Pricing Models

### 7.1 Pricing Model Implementations

```rust
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Unified pricing model interface
pub trait PricingModel: Send + Sync {
    fn calculate_charge(&self, usage: Decimal) -> Decimal;
    fn model_type(&self) -> PricingModelType;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModelType {
    FlatFee,
    PerUnit,
    TieredGraduated,
    TieredVolume,
    Package,
    Credits,
}

// ============================================================================
// 1. FLAT FEE PRICING
// ============================================================================

pub struct FlatFeePricing {
    pub amount: Decimal,
    pub currency: Currency,
    pub prorated: bool,
}

impl PricingModel for FlatFeePricing {
    fn calculate_charge(&self, _usage: Decimal) -> Decimal {
        self.amount
    }

    fn model_type(&self) -> PricingModelType {
        PricingModelType::FlatFee
    }
}

// ============================================================================
// 2. PER-UNIT PRICING
// ============================================================================

pub struct PerUnitPricing {
    pub unit_price: Decimal,
    pub currency: Currency,
    pub minimum_charge: Option<Decimal>,
}

impl PricingModel for PerUnitPricing {
    fn calculate_charge(&self, usage: Decimal) -> Decimal {
        let charge = usage * self.unit_price;

        match self.minimum_charge {
            Some(min) if charge < min => min,
            _ => charge,
        }
    }

    fn model_type(&self) -> PricingModelType {
        PricingModelType::PerUnit
    }
}

// ============================================================================
// 3. TIERED GRADUATED PRICING (Lago model)
// ============================================================================

pub struct TieredGraduatedPricing {
    pub tiers: Vec<PricingTier>,
    pub currency: Currency,
}

#[derive(Clone)]
pub struct PricingTier {
    pub up_to: Option<Decimal>,  // None = unlimited
    pub unit_price: Decimal,
    pub flat_fee: Option<Decimal>,  // Optional flat fee per tier
}

impl PricingModel for TieredGraduatedPricing {
    fn calculate_charge(&self, usage: Decimal) -> Decimal {
        let mut remaining = usage;
        let mut total_charge = Decimal::ZERO;
        let mut previous_upper = Decimal::ZERO;

        for tier in &self.tiers {
            if remaining <= Decimal::ZERO {
                break;
            }

            // Calculate units in this tier
            let tier_upper = tier.up_to.unwrap_or(Decimal::MAX);
            let tier_capacity = tier_upper - previous_upper;
            let units_in_tier = remaining.min(tier_capacity);

            // Add charge for this tier
            total_charge += units_in_tier * tier.unit_price;

            // Add flat fee if applicable
            if units_in_tier > Decimal::ZERO {
                if let Some(flat) = tier.flat_fee {
                    total_charge += flat;
                }
            }

            remaining -= units_in_tier;
            previous_upper = tier_upper;
        }

        total_charge
    }

    fn model_type(&self) -> PricingModelType {
        PricingModelType::TieredGraduated
    }
}

// ============================================================================
// 4. TIERED VOLUME PRICING
// ============================================================================

pub struct TieredVolumePricing {
    pub tiers: Vec<PricingTier>,
    pub currency: Currency,
}

impl PricingModel for TieredVolumePricing {
    fn calculate_charge(&self, usage: Decimal) -> Decimal {
        // Find applicable tier based on total volume
        let applicable_tier = self.tiers.iter()
            .find(|tier| {
                tier.up_to.map(|limit| usage <= limit).unwrap_or(true)
            })
            .or(self.tiers.last())
            .unwrap();

        // All units charged at tier rate
        let mut charge = usage * applicable_tier.unit_price;

        if let Some(flat) = applicable_tier.flat_fee {
            charge += flat;
        }

        charge
    }

    fn model_type(&self) -> PricingModelType {
        PricingModelType::TieredVolume
    }
}

// ============================================================================
// 5. PACKAGE PRICING
// ============================================================================

pub struct PackagePricing {
    pub package_size: u64,
    pub package_price: Decimal,
    pub overage_unit_price: Decimal,
    pub currency: Currency,
}

impl PricingModel for PackagePricing {
    fn calculate_charge(&self, usage: Decimal) -> Decimal {
        let package_size = Decimal::from(self.package_size);

        if usage <= package_size {
            // Within package
            self.package_price
        } else {
            // Package + overage
            let overage = usage - package_size;
            self.package_price + (overage * self.overage_unit_price)
        }
    }

    fn model_type(&self) -> PricingModelType {
        PricingModelType::Package
    }
}

// ============================================================================
// 6. PREPAID CREDITS
// ============================================================================

pub struct CreditsPricing {
    pub credit_rate: Decimal,  // Credits per unit
    pub currency: Currency,
}

impl PricingModel for CreditsPricing {
    fn calculate_charge(&self, usage: Decimal) -> Decimal {
        usage * self.credit_rate
    }

    fn model_type(&self) -> PricingModelType {
        PricingModelType::Credits
    }
}
```

---

## 8. AI-Native Metrics (Flexprice Patterns)

### 8.1 Standard AI Metrics

```rust
/// Pre-defined AI billable metrics (Flexprice patterns)
pub enum StandardAiMetric {
    /// LLM token consumption
    LlmTokens {
        model: String,
        token_type: TokenType,
    },

    /// GPU compute time
    GpuSeconds {
        gpu_type: GpuType,
        memory_gb: u32,
    },

    /// Vector embedding generation
    EmbeddingVectors {
        model: String,
        dimensions: u32,
    },

    /// Image generation
    ImageGeneration {
        model: String,
        resolution: ImageResolution,
    },

    /// Audio transcription
    AudioTranscription {
        model: String,
        duration_seconds: f64,
    },

    /// API calls (generic)
    ApiCalls {
        endpoint: String,
    },

    /// Storage (bytes)
    StorageBytes {
        storage_class: StorageClass,
    },

    /// Bandwidth (bytes)
    BandwidthBytes {
        direction: BandwidthDirection,
    },
}

#[derive(Debug, Clone)]
pub enum TokenType {
    Input,
    Output,
    Cached,  // Cached prompt tokens (cheaper)
}

#[derive(Debug, Clone)]
pub enum GpuType {
    H100,
    A100_80GB,
    A100_40GB,
    A10G,
    T4,
}

/// AI metric configurations with default pricing
pub fn get_standard_ai_metrics() -> Vec<BillableMetricConfig> {
    vec![
        // LLM Tokens (Claude)
        BillableMetricConfig {
            metric_code: "llm_tokens_claude_sonnet_input".to_string(),
            display_name: "Claude Sonnet Input Tokens".to_string(),
            aggregation: AggregationType::Sum,
            aggregation_property: Some("input_tokens".to_string()),
            unit: "tokens".to_string(),
            default_pricing: Some(PricingModel::PerUnit(PerUnitPricing {
                unit_price: dec!(0.000003),  // $3 per 1M tokens
                currency: Currency::USD,
                minimum_charge: None,
            })),
        },

        BillableMetricConfig {
            metric_code: "llm_tokens_claude_sonnet_output".to_string(),
            display_name: "Claude Sonnet Output Tokens".to_string(),
            aggregation: AggregationType::Sum,
            aggregation_property: Some("output_tokens".to_string()),
            unit: "tokens".to_string(),
            default_pricing: Some(PricingModel::PerUnit(PerUnitPricing {
                unit_price: dec!(0.000015),  // $15 per 1M tokens
                currency: Currency::USD,
                minimum_charge: None,
            })),
        },

        // GPU Compute
        BillableMetricConfig {
            metric_code: "gpu_seconds_h100".to_string(),
            display_name: "H100 GPU Seconds".to_string(),
            aggregation: AggregationType::Sum,
            aggregation_property: Some("duration_seconds".to_string()),
            unit: "seconds".to_string(),
            default_pricing: Some(PricingModel::PerUnit(PerUnitPricing {
                unit_price: dec!(0.001388),  // ~$5/hour
                currency: Currency::USD,
                minimum_charge: Some(dec!(0.01)),  // Minimum 1 cent
            })),
        },

        // Embeddings
        BillableMetricConfig {
            metric_code: "embedding_vectors".to_string(),
            display_name: "Embedding Vectors".to_string(),
            aggregation: AggregationType::Count,
            aggregation_property: None,
            unit: "vectors".to_string(),
            default_pricing: Some(PricingModel::TieredGraduated(TieredGraduatedPricing {
                tiers: vec![
                    PricingTier { up_to: Some(dec!(1000000)), unit_price: dec!(0.0001), flat_fee: None },
                    PricingTier { up_to: Some(dec!(10000000)), unit_price: dec!(0.00008), flat_fee: None },
                    PricingTier { up_to: None, unit_price: dec!(0.00005), flat_fee: None },
                ],
                currency: Currency::USD,
            })),
        },

        // Vector storage
        BillableMetricConfig {
            metric_code: "vector_storage_gb".to_string(),
            display_name: "Vector Storage (GB-month)".to_string(),
            aggregation: AggregationType::Max,
            aggregation_property: Some("storage_gb".to_string()),
            unit: "GB".to_string(),
            default_pricing: Some(PricingModel::PerUnit(PerUnitPricing {
                unit_price: dec!(0.25),  // $0.25/GB/month
                currency: Currency::USD,
                minimum_charge: None,
            })),
        },

        // Vector queries
        BillableMetricConfig {
            metric_code: "vector_queries".to_string(),
            display_name: "Vector Search Queries".to_string(),
            aggregation: AggregationType::Count,
            aggregation_property: None,
            unit: "queries".to_string(),
            default_pricing: Some(PricingModel::Package(PackagePricing {
                package_size: 10000,
                package_price: dec!(10.00),  // $10 for first 10K queries
                overage_unit_price: dec!(0.001),  // $0.001 per additional query
                currency: Currency::USD,
            })),
        },
    ]
}
```

### 8.2 Event Schema for AI Metrics

```rust
/// Standard event properties for AI workloads
#[derive(Debug, Serialize, Deserialize)]
pub struct LlmTokenEvent {
    pub model: String,
    pub provider: String,  // anthropic, openai, etc.
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cached_tokens: Option<u64>,
    pub latency_ms: u64,
    pub request_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GpuComputeEvent {
    pub gpu_type: String,
    pub instance_id: String,
    pub duration_seconds: f64,
    pub memory_used_gb: f64,
    pub job_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmbeddingEvent {
    pub model: String,
    pub dimensions: u32,
    pub vector_count: u64,
    pub input_tokens: u64,
    pub latency_ms: u64,
}

/// Example: Agent submitting LLM token event
fn create_llm_token_event(
    agent_nhi: &str,
    response: &LlmResponse,
) -> EventRequest {
    EventRequest {
        idempotency_key: format!("llm-{}", response.request_id),
        agent_nhi: agent_nhi.to_string(),
        delegation_chain: vec![],  // Populated by caller
        event_type: "llm_tokens".to_string(),
        timestamp: Some(Utc::now()),
        properties: serde_json::to_value(LlmTokenEvent {
            model: response.model.clone(),
            provider: "anthropic".to_string(),
            input_tokens: response.usage.input_tokens,
            output_tokens: response.usage.output_tokens,
            cached_tokens: response.usage.cached_tokens,
            latency_ms: response.latency.as_millis() as u64,
            request_id: response.request_id.clone(),
        }).unwrap(),
        signature: vec![],  // Signed by agent
        signature_algorithm: "ML-DSA-65".to_string(),
    }
}
```

---

## 9. Cross-Product Attribution API

### 9.1 Unified Cost Attribution

```rust
/// Cross-product attribution aggregator
/// Combines costs from: Metering, Runtime, Messaging, Inference
pub struct CrossProductAttributionService {
    metering: Arc<MeteringClient>,
    runtime: Arc<RuntimeClient>,
    messaging: Arc<MessagingClient>,
    inference: Arc<InferenceClient>,
}

impl CrossProductAttributionService {
    /// Get unified cost view across all Creto products
    pub async fn get_unified_attribution(
        &self,
        organization_id: Uuid,
        period: TimePeriod,
    ) -> Result<UnifiedAttributionReport, AttributionError> {
        // Parallel queries to all product services
        let (metering_costs, runtime_costs, messaging_costs, inference_costs) = tokio::try_join!(
            self.metering.get_attribution(organization_id, &period),
            self.runtime.get_attribution(organization_id, &period),
            self.messaging.get_attribution(organization_id, &period),
            self.inference.get_attribution(organization_id, &period),
        )?;

        // Merge agent-level attribution
        let mut agent_totals: HashMap<String, AgentCostSummary> = HashMap::new();

        // Add metering costs
        for (agent_nhi, cost) in metering_costs.by_agent {
            agent_totals.entry(agent_nhi.clone())
                .or_default()
                .metering_cost = cost;
        }

        // Add runtime costs
        for (agent_nhi, cost) in runtime_costs.by_agent {
            agent_totals.entry(agent_nhi.clone())
                .or_default()
                .runtime_cost = cost;
        }

        // Add messaging costs
        for (agent_nhi, cost) in messaging_costs.by_agent {
            agent_totals.entry(agent_nhi.clone())
                .or_default()
                .messaging_cost = cost;
        }

        // Add inference costs
        for (agent_nhi, cost) in inference_costs.by_agent {
            agent_totals.entry(agent_nhi.clone())
                .or_default()
                .inference_cost = cost;
        }

        // Calculate totals
        for (_, summary) in agent_totals.iter_mut() {
            summary.total_cost = summary.metering_cost
                + summary.runtime_cost
                + summary.messaging_cost
                + summary.inference_cost;
        }

        // Build delegation tree rollup
        let delegation_tree = self.build_delegation_tree_rollup(&agent_totals).await?;

        Ok(UnifiedAttributionReport {
            organization_id,
            period,
            by_agent: agent_totals,
            by_product: ProductCostSummary {
                metering: metering_costs.total,
                runtime: runtime_costs.total,
                messaging: messaging_costs.total,
                inference: inference_costs.total,
                total: metering_costs.total + runtime_costs.total
                    + messaging_costs.total + inference_costs.total,
            },
            delegation_tree,
            generated_at: Utc::now(),
        })
    }

    /// Build hierarchical cost rollup by delegation chain
    async fn build_delegation_tree_rollup(
        &self,
        agent_costs: &HashMap<String, AgentCostSummary>,
    ) -> Result<DelegationTreeRollup, AttributionError> {
        let mut tree = DelegationTreeRollup::new();

        for (agent_nhi, costs) in agent_costs {
            // Get delegation chain for this agent
            let chain = self.metering.get_delegation_chain(agent_nhi).await?;

            // Roll up costs through the chain
            tree.add_cost_with_rollup(&chain, costs.total_cost);
        }

        Ok(tree)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UnifiedAttributionReport {
    pub organization_id: Uuid,
    pub period: TimePeriod,
    pub by_agent: HashMap<String, AgentCostSummary>,
    pub by_product: ProductCostSummary,
    pub delegation_tree: DelegationTreeRollup,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct AgentCostSummary {
    pub metering_cost: Decimal,
    pub runtime_cost: Decimal,
    pub messaging_cost: Decimal,
    pub inference_cost: Decimal,
    pub total_cost: Decimal,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProductCostSummary {
    pub metering: Decimal,
    pub runtime: Decimal,
    pub messaging: Decimal,
    pub inference: Decimal,
    pub total: Decimal,
}

/// Delegation tree with cost rollup
/// Each node shows direct costs + rolled-up costs from spawned agents
#[derive(Debug, Clone, Serialize)]
pub struct DelegationTreeRollup {
    pub root_principal: String,
    pub direct_cost: Decimal,
    pub rolled_up_cost: Decimal,  // Including all descendants
    pub children: Vec<DelegationTreeRollup>,
}
```

### 9.2 REST API for Cross-Product Attribution

```yaml
# OpenAPI specification
paths:
  /v1/attribution/unified:
    get:
      summary: Get unified cost attribution across all Creto products
      parameters:
        - name: organization_id
          in: query
          required: true
          schema:
            type: string
            format: uuid
        - name: period_start
          in: query
          required: true
          schema:
            type: string
            format: date-time
        - name: period_end
          in: query
          required: true
          schema:
            type: string
            format: date-time
        - name: group_by
          in: query
          schema:
            type: array
            items:
              type: string
              enum: [agent, product, delegation_chain, dimension]
      responses:
        '200':
          description: Unified attribution report
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/UnifiedAttributionReport'

  /v1/attribution/delegation-tree:
    get:
      summary: Get cost rollup by delegation tree
      parameters:
        - name: root_principal
          in: query
          required: true
          schema:
            type: string
        - name: period_start
          in: query
          required: true
          schema:
            type: string
            format: date-time
        - name: period_end
          in: query
          required: true
          schema:
            type: string
            format: date-time
        - name: depth
          in: query
          schema:
            type: integer
            default: 10
            maximum: 50
      responses:
        '200':
          description: Delegation tree with cost rollup
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/DelegationTreeRollup'
```

---

## 10. API Contracts

### 10.1 gRPC Service Definitions

```protobuf
syntax = "proto3";

package creto.metering.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";
import "google/protobuf/duration.proto";

// ============================================================================
// Event Ingestion Service
// ============================================================================

service EventIngestionService {
  rpc IngestEvent(IngestEventRequest) returns (IngestEventResponse);
  rpc IngestBatch(IngestBatchRequest) returns (IngestBatchResponse);
  rpc GetEvent(GetEventRequest) returns (Event);
  rpc QueryEvents(QueryEventsRequest) returns (QueryEventsResponse);
}

message IngestEventRequest {
  string idempotency_key = 1;
  string agent_nhi = 2;
  repeated string delegation_chain = 3;
  string event_type = 4;
  google.protobuf.Timestamp timestamp = 5;
  google.protobuf.Struct properties = 6;
  bytes signature = 7;
  string signature_algorithm = 8;
}

message IngestEventResponse {
  string event_id = 1;
  EventStatus status = 2;

  enum EventStatus {
    CREATED = 0;
    ACCEPTED = 1;  // Idempotent duplicate
    CONFLICT = 2;  // Same key, different data
  }

  string existing_hash = 3;  // For CONFLICT
}

// ============================================================================
// Quota Enforcement Service
// ============================================================================

service QuotaEnforcementService {
  rpc CheckQuota(CheckQuotaRequest) returns (CheckQuotaResponse);
  rpc ReserveQuota(ReserveQuotaRequest) returns (ReserveQuotaResponse);
  rpc CommitReservation(CommitReservationRequest) returns (CommitReservationResponse);
  rpc RollbackReservation(RollbackReservationRequest) returns (RollbackReservationResponse);
  rpc GetQuotaUsage(GetQuotaUsageRequest) returns (GetQuotaUsageResponse);
}

message CheckQuotaRequest {
  string agent_nhi = 1;
  string event_type = 2;
}

message CheckQuotaResponse {
  QuotaDecision decision = 1;

  message QuotaDecision {
    bool allowed = 1;
    int64 remaining = 2;
    int64 limit = 3;
    google.protobuf.Timestamp period_end = 4;

    // If denied
    QuotaExceededReason reason = 5;
    google.protobuf.Duration retry_after = 6;
    int64 current_usage = 7;

    // Hierarchy info
    string source_organization_id = 8;
  }

  enum QuotaExceededReason {
    QUOTA_EXCEEDED_REASON_UNSPECIFIED = 0;
    LIMIT_REACHED = 1;
    SUBSCRIPTION_SUSPENDED = 2;
    QUOTA_NOT_CONFIGURED = 3;
    ORGANIZATION_LIMIT_REACHED = 4;  // Parent org quota exceeded
  }
}

// ============================================================================
// Attribution Service
// ============================================================================

service AttributionService {
  rpc GetUsage(GetUsageRequest) returns (GetUsageResponse);
  rpc GetAttribution(GetAttributionRequest) returns (GetAttributionResponse);
  rpc GetUnifiedAttribution(GetUnifiedAttributionRequest) returns (UnifiedAttributionResponse);
  rpc GetDelegationTreeRollup(GetDelegationTreeRequest) returns (DelegationTreeResponse);
}

message GetUnifiedAttributionRequest {
  string organization_id = 1;
  google.protobuf.Timestamp period_start = 2;
  google.protobuf.Timestamp period_end = 3;
  bool include_products = 4;  // Metering, Runtime, Messaging, Inference
}

message UnifiedAttributionResponse {
  map<string, AgentCostSummary> by_agent = 1;
  ProductCostSummary by_product = 2;
  DelegationTreeNode delegation_tree = 3;
  google.protobuf.Timestamp generated_at = 4;
}

message AgentCostSummary {
  double metering_cost = 1;
  double runtime_cost = 2;
  double messaging_cost = 3;
  double inference_cost = 4;
  double total_cost = 5;
}

message ProductCostSummary {
  double metering = 1;
  double runtime = 2;
  double messaging = 3;
  double inference = 4;
  double total = 5;
}

message DelegationTreeNode {
  string principal = 1;
  double direct_cost = 2;
  double rolled_up_cost = 3;
  repeated DelegationTreeNode children = 4;
}
```

---

## 11. Integration Contracts

### 11.1 Authorization Service Integration

```rust
/// Contract: Metering <-> Authorization
/// Called inline during authorization check for quota enforcement

#[async_trait]
pub trait MeteringAuthzIntegration {
    /// Check if agent has remaining quota for action
    /// Called by Authorization service during policy evaluation
    /// Latency budget: <10µs
    async fn check_quota_inline(
        &self,
        agent_nhi: &str,
        action: &str,  // Maps to event_type
    ) -> QuotaCheckResult;
}

pub struct QuotaCheckResult {
    pub allowed: bool,
    pub remaining: Option<u64>,
    pub limit: Option<u64>,
    pub denial_reason: Option<String>,
}

// Authorization service calls metering:
// authz.check(agent, action, resource)
//   -> policy.evaluate()
//   -> metering.check_quota_inline(agent, action)
//   -> combined_decision
```

### 11.2 Audit Service Integration

```rust
/// Contract: Metering <-> Audit
/// All billable events logged to immutable audit trail

#[async_trait]
pub trait MeteringAuditIntegration {
    /// Log event ingestion to audit trail
    /// Returns audit_log_id for cross-reference
    async fn log_event_ingestion(
        &self,
        event: &Event,
        result: &IngestResult,
    ) -> Result<Uuid, AuditError>;

    /// Log quota decision to audit trail
    async fn log_quota_decision(
        &self,
        agent_nhi: &str,
        event_type: &str,
        decision: &QuotaDecision,
    ) -> Result<Uuid, AuditError>;

    /// Log invoice generation to audit trail
    async fn log_invoice_generated(
        &self,
        invoice: &Invoice,
    ) -> Result<Uuid, AuditError>;
}

// Audit event structure
pub struct MeteringAuditEvent {
    pub event_type: MeteringAuditEventType,
    pub actor: String,  // agent_nhi or system
    pub resource: String,  // subscription_id or invoice_id
    pub action: String,
    pub outcome: AuditOutcome,
    pub metadata: serde_json::Value,
    pub signature: Vec<u8>,  // Signed by metering service
}

pub enum MeteringAuditEventType {
    EventIngested,
    EventRejected,
    QuotaChecked,
    QuotaExceeded,
    InvoiceGenerated,
    InvoiceFinalized,
    PricingChanged,
}
```

### 11.3 NHI Service Integration

```rust
/// Contract: Metering <-> NHI Registry
/// Resolve agent identity and delegation chains

#[async_trait]
pub trait MeteringNhiIntegration {
    /// Resolve agent identity from NHI
    async fn resolve_agent(
        &self,
        agent_nhi: &str,
    ) -> Result<AgentIdentity, NhiError>;

    /// Get public key for signature verification
    async fn get_public_key(
        &self,
        agent_nhi: &str,
    ) -> Result<VerifyingKey, NhiError>;

    /// Verify delegation chain
    async fn verify_delegation_chain(
        &self,
        chain: &[String],
    ) -> Result<DelegationValidation, NhiError>;

    /// Resolve agent to organization
    async fn get_agent_organization(
        &self,
        agent_nhi: &str,
    ) -> Result<OrganizationId, NhiError>;
}

pub struct DelegationValidation {
    pub valid: bool,
    pub root_principal: String,
    pub chain_depth: usize,
    pub expired_delegations: Vec<String>,
}
```

---

## 12. Error Taxonomy

### 12.1 Error Code Registry

| Code | HTTP | gRPC | Category | Description | Retry |
|------|------|------|----------|-------------|-------|
| `MTR-001` | 400 | INVALID_ARGUMENT | Validation | Missing required field | No |
| `MTR-002` | 400 | INVALID_ARGUMENT | Validation | Invalid NHI format | No |
| `MTR-003` | 400 | INVALID_ARGUMENT | Validation | Invalid event type | No |
| `MTR-004` | 400 | INVALID_ARGUMENT | Validation | Timestamp too far in future | No |
| `MTR-005` | 400 | INVALID_ARGUMENT | Validation | Properties too large | No |
| `MTR-006` | 400 | INVALID_ARGUMENT | Validation | Properties too deeply nested | No |
| `MTR-007` | 401 | UNAUTHENTICATED | Auth | Missing or invalid token | No |
| `MTR-008` | 403 | PERMISSION_DENIED | Auth | Insufficient permissions | No |
| `MTR-009` | 403 | PERMISSION_DENIED | Auth | Agent not authorized for subscription | No |
| `MTR-010` | 409 | ALREADY_EXISTS | Idempotency | Idempotency key conflict | No |
| `MTR-011` | 400 | INVALID_ARGUMENT | Signature | Invalid signature | No |
| `MTR-012` | 400 | INVALID_ARGUMENT | Signature | Unsupported algorithm | No |
| `MTR-013` | 404 | NOT_FOUND | Resolution | Agent not found in NHI registry | No |
| `MTR-014` | 404 | NOT_FOUND | Resolution | Subscription not found | No |
| `MTR-015` | 404 | NOT_FOUND | Resolution | Event not found | No |
| `MTR-016` | 429 | RESOURCE_EXHAUSTED | Quota | Quota exceeded (block) | Backoff |
| `MTR-017` | 429 | RESOURCE_EXHAUSTED | Rate | Rate limit exceeded | Backoff |
| `MTR-018` | 500 | INTERNAL | Database | Database error | Yes |
| `MTR-019` | 500 | INTERNAL | Cache | Cache error | Yes |
| `MTR-020` | 503 | UNAVAILABLE | Service | Service unavailable | Yes |

### 12.2 Error Response Structure

```rust
#[derive(Debug, Serialize)]
pub struct MeteringError {
    pub code: String,           // e.g., "MTR-016"
    pub message: String,        // Human-readable message
    pub category: ErrorCategory,
    pub details: ErrorDetails,
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub enum ErrorCategory {
    Validation,
    Authentication,
    Authorization,
    Idempotency,
    Signature,
    Resolution,
    Quota,
    RateLimit,
    Internal,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetails {
    // Validation errors
    pub field: Option<String>,
    pub expected: Option<String>,
    pub actual: Option<String>,

    // Quota errors
    pub limit: Option<u64>,
    pub current_usage: Option<u64>,
    pub retry_after_seconds: Option<u64>,

    // Idempotency errors
    pub existing_hash: Option<String>,
    pub submitted_hash: Option<String>,

    // Additional context
    pub metadata: HashMap<String, String>,
}
```

---

## 13. Edge Cases & Failure Modes

### 13.1 Edge Cases

| # | Scenario | Expected Behavior | Test Coverage |
|---|----------|-------------------|---------------|
| E1 | Event with exactly max properties size | Accept event | `test_max_properties_size` |
| E2 | Event with properties depth = max | Accept event | `test_max_properties_depth` |
| E3 | Timestamp exactly at skew boundary | Accept event | `test_timestamp_boundary` |
| E4 | Quota at exactly limit | Deny next event | `test_quota_at_limit` |
| E5 | Batch with 1000 events (max) | Accept all | `test_max_batch_size` |
| E6 | Concurrent quota reservations | Atomic handling | `test_concurrent_reservations` |
| E7 | Delegation chain at max depth | Accept event | `test_max_delegation_depth` |
| E8 | Organization hierarchy at max depth | Resolve correctly | `test_max_org_hierarchy` |
| E9 | Invoice spanning DST transition | Correct period | `test_dst_transition_invoice` |
| E10 | Zero-usage billing period | Generate $0 invoice | `test_zero_usage_invoice` |
| E11 | Negative price in credit note | Handle correctly | `test_credit_note` |
| E12 | Unicode in event properties | Accept and store | `test_unicode_properties` |
| E13 | Null values in JSON properties | Handle gracefully | `test_null_properties` |
| E14 | Very large numeric values | Use Decimal precision | `test_large_numbers` |
| E15 | Rapid sequential events (same ms) | Preserve order | `test_same_timestamp_ordering` |

### 13.2 Failure Modes

| # | Failure | Detection | Mitigation | Recovery |
|---|---------|-----------|------------|----------|
| F1 | Redis cache unavailable | Health check, latency spike | Fall through to DB | Auto-recover when Redis up |
| F2 | PostgreSQL primary down | Connection error | Promote replica | Replay from Kafka |
| F3 | Kafka broker failure | Producer timeout | Retry with backoff | Auto-rebalance |
| F4 | Signature verification timeout | Request timeout | Return error | Client retry |
| F5 | NHI registry unavailable | gRPC error | Cache recent lookups | Degrade to cached |
| F6 | Aggregation worker crash | Missing heartbeat | Restart worker | Backfill from events |
| F7 | Invoice generation OOM | Process killed | Chunk processing | Restart from checkpoint |
| F8 | Stripe webhook failure | Delivery failure | Retry queue | Manual reconciliation |
| F9 | Clock skew between nodes | Consensus timestamp drift | Use consensus time | Re-sync from leader |
| F10 | Quota cache poisoning | HMAC mismatch | Invalidate cache | Rebuild from DB |

### 13.3 Failure Mode Handling

```rust
/// Circuit breaker for external dependencies
pub struct CircuitBreaker {
    failure_threshold: u32,
    recovery_timeout: Duration,
    state: AtomicU8,  // 0=Closed, 1=Open, 2=HalfOpen
    failure_count: AtomicU32,
    last_failure: AtomicU64,
}

impl CircuitBreaker {
    pub fn can_execute(&self) -> bool {
        match self.state.load(Ordering::Relaxed) {
            0 => true,  // Closed: allow all
            1 => {      // Open: check if recovery timeout passed
                let last = self.last_failure.load(Ordering::Relaxed);
                let now = Instant::now().elapsed().as_millis() as u64;
                if now - last > self.recovery_timeout.as_millis() as u64 {
                    self.state.store(2, Ordering::Relaxed);  // Move to half-open
                    true
                } else {
                    false
                }
            }
            2 => true,  // Half-open: allow probe request
            _ => false,
        }
    }

    pub fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.state.store(0, Ordering::Relaxed);  // Close circuit
    }

    pub fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::Relaxed);
        if count >= self.failure_threshold {
            self.state.store(1, Ordering::Relaxed);  // Open circuit
            self.last_failure.store(
                Instant::now().elapsed().as_millis() as u64,
                Ordering::Relaxed
            );
        }
    }
}

/// Graceful degradation for quota checks
pub async fn check_quota_with_fallback(
    enforcer: &QuotaEnforcer,
    agent_nhi: &str,
    event_type: &str,
) -> QuotaDecision {
    // Try normal path
    match enforcer.check_quota(agent_nhi, event_type).await {
        Ok(decision) => decision,
        Err(e) => {
            // Log error
            tracing::error!(?e, "Quota check failed, using fallback");

            // Fallback: allow with warning (fail-open for availability)
            QuotaDecision::AllowWithWarning {
                remaining: 0,
                warning: "Quota service temporarily unavailable".to_string(),
            }
        }
    }
}
```

---

## 14. Sequence Diagrams

### 14.1 Event Ingestion Flow

```
┌─────┐          ┌─────────┐          ┌──────────┐          ┌────────┐          ┌────────┐
│Agent│          │ Gateway │          │ Metering │          │  NHI   │          │Database│
└──┬──┘          └────┬────┘          └────┬─────┘          └───┬────┘          └───┬────┘
   │                  │                     │                    │                   │
   │ POST /events     │                     │                    │                   │
   │─────────────────▶│                     │                    │                   │
   │                  │                     │                    │                   │
   │                  │ IngestEvent(req)    │                    │                   │
   │                  │────────────────────▶│                    │                   │
   │                  │                     │                    │                   │
   │                  │                     │ GetPublicKey(nhi)  │                   │
   │                  │                     │───────────────────▶│                   │
   │                  │                     │                    │                   │
   │                  │                     │     public_key     │                   │
   │                  │                     │◀───────────────────│                   │
   │                  │                     │                    │                   │
   │                  │                     │ Verify Signature   │                   │
   │                  │                     │──────────┐         │                   │
   │                  │                     │          │         │                   │
   │                  │                     │◀─────────┘         │                   │
   │                  │                     │                    │                   │
   │                  │                     │ Check Idempotency  │                   │
   │                  │                     │───────────────────────────────────────▶│
   │                  │                     │                    │                   │
   │                  │                     │    (not exists)    │                   │
   │                  │                     │◀───────────────────────────────────────│
   │                  │                     │                    │                   │
   │                  │                     │ Resolve Org        │                   │
   │                  │                     │───────────────────▶│                   │
   │                  │                     │                    │                   │
   │                  │                     │   organization_id  │                   │
   │                  │                     │◀───────────────────│                   │
   │                  │                     │                    │                   │
   │                  │                     │ INSERT Event       │                   │
   │                  │                     │───────────────────────────────────────▶│
   │                  │                     │                    │                   │
   │                  │                     │     event_id       │                   │
   │                  │                     │◀───────────────────────────────────────│
   │                  │                     │                    │                   │
   │                  │ {event_id, CREATED} │                    │                   │
   │                  │◀────────────────────│                    │                   │
   │                  │                     │                    │                   │
   │ 201 {event_id}   │                     │                    │                   │
   │◀─────────────────│                     │                    │                   │
   │                  │                     │                    │                   │
```

### 14.2 Quota Check with Hierarchy

```
┌─────┐          ┌───────┐          ┌─────────┐          ┌───────┐          ┌────────┐
│AuthZ│          │Metering│          │Hierarchy│          │ Redis │          │Database│
└──┬──┘          └───┬───┘          └────┬────┘          └───┬───┘          └───┬────┘
   │                 │                    │                   │                  │
   │ CheckQuota      │                    │                   │                  │
   │────────────────▶│                    │                   │                  │
   │                 │                    │                   │                  │
   │                 │ ResolveEffective   │                   │                  │
   │                 │───────────────────▶│                   │                  │
   │                 │                    │                   │                  │
   │                 │                    │ GetOrgChain       │                  │
   │                 │                    │──────────────────────────────────────▶│
   │                 │                    │                   │                  │
   │                 │                    │   [team, org, enterprise]            │
   │                 │                    │◀──────────────────────────────────────│
   │                 │                    │                   │                  │
   │                 │                    │ GetQuotas(each)   │                  │
   │                 │                    │──────────────────────────────────────▶│
   │                 │                    │                   │                  │
   │                 │                    │   quota_configs   │                  │
   │                 │                    │◀──────────────────────────────────────│
   │                 │                    │                   │                  │
   │                 │                    │ ApplyInheritance  │                  │
   │                 │                    │──────────┐        │                  │
   │                 │                    │          │        │                  │
   │                 │                    │◀─────────┘        │                  │
   │                 │                    │                   │                  │
   │                 │ effective_quota    │                   │                  │
   │                 │◀───────────────────│                   │                  │
   │                 │                    │                   │                  │
   │                 │ GET L1 Cache       │                   │                  │
   │                 │──────────┐         │                   │                  │
   │                 │          │ MISS    │                   │                  │
   │                 │◀─────────┘         │                   │                  │
   │                 │                    │                   │                  │
   │                 │ GET L2 Cache       │                   │                  │
   │                 │───────────────────────────────────────▶│                  │
   │                 │                    │                   │                  │
   │                 │     current_usage  │                   │                  │
   │                 │◀───────────────────────────────────────│                  │
   │                 │                    │                   │                  │
   │                 │ Evaluate           │                   │                  │
   │                 │──────────┐         │                   │                  │
   │                 │          │         │                   │                  │
   │                 │◀─────────┘         │                   │                  │
   │                 │                    │                   │                  │
   │ {allowed, 550}  │                    │                   │                  │
   │◀────────────────│                    │                   │                  │
   │                 │                    │                   │                  │
```

---

## 15. Performance Specifications

### 15.1 Latency Targets

| Operation | P50 | P95 | P99 | Notes |
|-----------|-----|-----|-----|-------|
| `check_quota` (L1 hit) | 2us | 5us | 10us | In-memory cache |
| `check_quota` (L2 hit) | 50us | 200us | 500us | Redis cache |
| `check_quota` (DB) | 1ms | 5ms | 10ms | With index |
| `ingest_event` (single) | 5ms | 15ms | 30ms | Including signature verify |
| `ingest_batch` (1000) | 200ms | 400ms | 600ms | Parallel processing |
| `get_usage` | 10ms | 50ms | 100ms | Aggregated query |
| `generate_invoice` | 200ms | 500ms | 1s | Complex aggregation |

### 15.2 Throughput Targets

| Operation | Target | Sustained | Burst |
|-----------|--------|-----------|-------|
| Event ingestion | 10,000/sec | 8 hours | 50,000/sec for 1 min |
| Quota checks | 100,000/sec | Indefinite | 500,000/sec for 1 min |
| API requests | 5,000/sec | Indefinite | 20,000/sec for 1 min |

### 15.3 Capacity Planning

```rust
/// Capacity planning formulas

// Events per month = agents * events_per_agent_per_day * 30
// Example: 10,000 agents * 1,000 events/day * 30 = 300M events/month

// Storage per month (events table):
// = events_per_month * avg_event_size_bytes
// = 300M * 500 bytes = 150 GB/month

// Redis memory for quota cache:
// = subscriptions * event_types * quota_state_size
// = 100,000 * 10 * 256 bytes = 256 MB

// PostgreSQL connections:
// = ingestion_workers * 2 + api_servers * pool_size
// = 10 * 2 + 5 * 20 = 120 connections

pub struct CapacityEstimate {
    pub events_per_month: u64,
    pub storage_gb_per_month: f64,
    pub redis_memory_mb: f64,
    pub pg_connections: u32,
}

pub fn estimate_capacity(
    agents: u64,
    events_per_agent_per_day: u64,
    subscriptions: u64,
    event_types: u64,
    ingestion_workers: u32,
    api_servers: u32,
) -> CapacityEstimate {
    let events_per_month = agents * events_per_agent_per_day * 30;
    let storage_gb = (events_per_month as f64 * 500.0) / 1_000_000_000.0;
    let redis_mb = (subscriptions * event_types * 256) as f64 / 1_000_000.0;
    let pg_conns = ingestion_workers * 2 + api_servers * 20;

    CapacityEstimate {
        events_per_month,
        storage_gb_per_month: storage_gb,
        redis_memory_mb: redis_mb,
        pg_connections: pg_conns,
    }
}
```

---

## 16. Security Model

### 16.1 Threat Model Summary

| Threat | Likelihood | Impact | Mitigation |
|--------|------------|--------|------------|
| Event injection | Medium | High | ML-DSA signature verification |
| Quota bypass | Low | High | Consensus timestamps, atomic updates |
| Replay attack | Medium | Medium | Idempotency keys (7-day window) |
| Subscription impersonation | Low | High | Delegation chain verification |
| Pricing manipulation | Low | Critical | Immutable pricing snapshots, audit |
| Quota starvation (DoS) | Medium | Medium | Rate limiting, graduated quotas |

### 16.2 Cryptographic Controls

| Control | Algorithm | Key Size | Notes |
|---------|-----------|----------|-------|
| Event signing | ML-DSA-65 | 256-bit | Post-quantum |
| Cache integrity | HMAC-SHA256 | 256-bit | Detect tampering |
| Token verification | Ed25519 | 256-bit | JWT from AuthZ |
| Data at rest | AES-256-GCM | 256-bit | PostgreSQL TDE |
| Data in transit | TLS 1.3 | 256-bit | All connections |

### 16.3 Access Control

```rust
/// Role-based access control for metering APIs
pub enum MeteringPermission {
    // Event operations
    EventsWrite,    // Submit events (agents)
    EventsRead,     // Query events (billing admins)

    // Quota operations
    QuotasRead,     // View quota status (agents, admins)
    QuotasWrite,    // Modify quotas (billing admins)

    // Billing operations
    InvoicesRead,   // View invoices (billing admins)
    InvoicesWrite,  // Generate/finalize invoices (billing service)

    // Admin operations
    PricingWrite,   // Modify pricing (requires 2-person approval)
    OrganizationsWrite, // Manage org hierarchy
}

/// Permission matrix by role
pub fn get_role_permissions(role: &str) -> Vec<MeteringPermission> {
    match role {
        "agent" => vec![
            MeteringPermission::EventsWrite,
            MeteringPermission::QuotasRead,
        ],
        "billing_admin" => vec![
            MeteringPermission::EventsRead,
            MeteringPermission::QuotasRead,
            MeteringPermission::QuotasWrite,
            MeteringPermission::InvoicesRead,
        ],
        "billing_service" => vec![
            MeteringPermission::EventsRead,
            MeteringPermission::InvoicesRead,
            MeteringPermission::InvoicesWrite,
        ],
        "super_admin" => vec![
            MeteringPermission::EventsRead,
            MeteringPermission::EventsWrite,
            MeteringPermission::QuotasRead,
            MeteringPermission::QuotasWrite,
            MeteringPermission::InvoicesRead,
            MeteringPermission::InvoicesWrite,
            MeteringPermission::PricingWrite,
            MeteringPermission::OrganizationsWrite,
        ],
        _ => vec![],
    }
}
```

---

## 17. Operational Runbook

### 17.1 Health Check Endpoints

```yaml
# Kubernetes liveness/readiness probes
livenessProbe:
  httpGet:
    path: /health/live
    port: 8080
  initialDelaySeconds: 10
  periodSeconds: 5

readinessProbe:
  httpGet:
    path: /health/ready
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 3
```

```rust
/// Health check implementation
pub async fn health_ready(state: &AppState) -> HealthStatus {
    let pg_ok = state.db.ping().await.is_ok();
    let redis_ok = state.redis.ping().await.is_ok();
    let kafka_ok = state.kafka.is_connected();

    HealthStatus {
        status: if pg_ok && redis_ok && kafka_ok { "ready" } else { "degraded" },
        checks: vec![
            Check { name: "postgresql", status: pg_ok },
            Check { name: "redis", status: redis_ok },
            Check { name: "kafka", status: kafka_ok },
        ],
    }
}
```

### 17.2 Key Metrics

```yaml
# Prometheus metrics to monitor
metering_events_ingested_total:
  labels: [subscription_id, event_type, status]

metering_quota_checks_total:
  labels: [event_type, decision, cache_layer]

metering_quota_check_duration_seconds:
  type: histogram
  buckets: [0.000001, 0.00001, 0.0001, 0.001, 0.01]

metering_event_ingestion_duration_seconds:
  type: histogram
  buckets: [0.001, 0.005, 0.01, 0.05, 0.1, 0.5]

metering_aggregation_lag_seconds:
  type: gauge

metering_invoice_generation_duration_seconds:
  type: histogram
```

### 17.3 Alert Rules

```yaml
groups:
  - name: metering_critical
    rules:
      - alert: QuotaCheckLatencyHigh
        expr: histogram_quantile(0.99, metering_quota_check_duration_seconds) > 0.01
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: "Quota check p99 latency >10ms"

      - alert: EventIngestionBacklog
        expr: metering_aggregation_lag_seconds > 300
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Aggregation lag >5 minutes"

      - alert: QuotaCacheHitRateLow
        expr: sum(rate(metering_quota_checks_total{cache_layer="l1"}[5m])) / sum(rate(metering_quota_checks_total[5m])) < 0.8
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: "Quota cache L1 hit rate <80%"
```

### 17.4 Common Operations

#### Reset Quota for Agent

```sql
-- Clear current period usage
DELETE FROM usage_hourly
WHERE subscription_id = (
    SELECT subscription_id FROM subscriptions s
    JOIN organization_agents oa ON oa.organization_id = s.organization_id
    WHERE oa.agent_nhi = '<AGENT_NHI>'
)
AND event_type = '<EVENT_TYPE>'
AND hour >= date_trunc('hour', now());

-- Invalidate cache
-- Redis: DEL quota:<subscription_id>:<event_type>:*
```

#### Regenerate Invoice

```bash
# Void existing invoice
curl -X POST /v1/invoices/<invoice_id>/void \
  -H "Authorization: Bearer $ADMIN_TOKEN"

# Regenerate
curl -X POST /v1/invoices \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -d '{
    "subscription_id": "<subscription_id>",
    "period_start": "2024-12-01T00:00:00Z",
    "period_end": "2025-01-01T00:00:00Z"
  }'
```

---

## Appendix A: Migration Guide

### A.1 From Single-Tenant to Multi-Tenant

```sql
-- Step 1: Create organizations for existing customers
INSERT INTO organizations (name, slug, organization_type, billing_email)
SELECT DISTINCT
    customer_name,
    lower(regexp_replace(customer_name, '[^a-zA-Z0-9]', '-', 'g')),
    'enterprise',
    billing_email
FROM customers;

-- Step 2: Link subscriptions to organizations
ALTER TABLE subscriptions ADD COLUMN organization_id UUID;

UPDATE subscriptions s
SET organization_id = o.organization_id
FROM organizations o
JOIN customers c ON o.slug = lower(regexp_replace(c.customer_name, '[^a-zA-Z0-9]', '-', 'g'))
WHERE s.customer_id = c.customer_id;

ALTER TABLE subscriptions ALTER COLUMN organization_id SET NOT NULL;

-- Step 3: Migrate quotas to organization level
INSERT INTO quotas (organization_id, event_type, limit_value, period, overflow_action)
SELECT DISTINCT
    s.organization_id,
    q.event_type,
    q.limit_value,
    q.period,
    q.overflow_action
FROM old_quotas q
JOIN subscriptions s ON q.subscription_id = s.subscription_id;
```

---

## Appendix B: Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-12-25 | Architecture Team | Initial MASTER SDD |

---

**End of Document**

This MASTER SDD is the authoritative reference for creto-metering implementation. All implementation must conform to this specification.
