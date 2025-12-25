---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# SDD-MTR-05: Metering Security Design

## Overview

This document specifies the security architecture, threat model, and mitigation strategies for **creto-metering**. As a billing system, metering is a high-value target for attackers seeking to:

1. **Avoid payment**: Suppress or manipulate billable events
2. **Exceed quotas**: Bypass spending limits
3. **Deny service**: Exhaust quotas for other users (DoS)
4. **Steal data**: Access billing information or usage patterns
5. **Repudiate charges**: Dispute legitimate bills

The security design integrates Creto's sovereign primitives (NHI, ML-DSA signatures, consensus-ordering) to provide **cryptographic verifiability** and **non-repudiation** for billing operations.

## Security Principles

1. **Zero Trust**: Every API call authenticated and authorized
2. **Cryptographic Verifiability**: All events signed with ML-DSA (post-quantum)
3. **Immutability**: Append-only event log (no deletion or modification)
4. **Consensus Ordering**: Timestamps from distributed consensus (not agent clocks)
5. **Defense in Depth**: Multiple layers (signature, authz, audit, rate limiting)
6. **Least Privilege**: Agents can only submit events for their own identity
7. **Audit Logging**: All operations logged to immutable audit trail

## Threat Model

### Threat Actors

| Actor | Motivation | Capabilities | Risk Level |
|-------|------------|--------------|------------|
| **Malicious Agent** | Avoid payment, exceed quota | Submit fake events, replay attacks | High |
| **Compromised Agent** | Controlled by attacker | Valid credentials, can sign events | Critical |
| **Insider (Customer)** | Reduce bill, steal data | API access, valid subscriptions | Medium |
| **External Attacker** | Disrupt service, steal data | Network access, no valid credentials | Medium |
| **Rogue Employee** | Fraud, sabotage | Database access, admin privileges | Low (mitigated by audit) |

---

### Threat 1: Event Injection Attack

**Description**: Attacker submits fake billable events for another agent to inflate costs or exhaust quotas.

**Attack Scenario**:
```
Attacker submits event:
{
  "agent_nhi": "agent:victim-agent",  // Not attacker's identity
  "event_type": "llm_tokens",
  "properties": {"tokens": 1000000000},  // Massive usage
  "signature": "fake_signature"
}
```

**Impact**:
- Victim charged for usage they didn't consume
- Victim's quota exhausted (DoS)
- Trust in billing system undermined

**Mitigations**:

1. **Cryptographic Signature Verification** (Primary Defense)
   ```rust
   async fn verify_event_signature(event: &EventRequest) -> Result<(), Error> {
       // Step 1: Extract public key from agent_nhi
       let public_key = nhi_registry.get_public_key(&event.agent_nhi).await?;

       // Step 2: Construct canonical message (deterministic serialization)
       let canonical_msg = construct_canonical_message(event);

       // Step 3: Verify ML-DSA-65 signature
       let valid = crypto_client.verify_signature(
           &event.signature,
           &canonical_msg,
           &public_key,
           SignatureAlgorithm::MLDSA65,
       )?;

       if !valid {
           return Err(Error::InvalidSignature {
               agent_nhi: event.agent_nhi.clone(),
           });
       }

       Ok(())
   }
   ```

2. **Authorization Check**
   - Before accepting event, verify agent has permission to submit events for this subscription
   ```rust
   let authz_result = authz.check(
       &event.agent_nhi,
       "events:write",
       &event.subscription_id,
   ).await?;

   if !authz_result.allowed {
       return Err(Error::Unauthorized);
   }
   ```

3. **Delegation Chain Validation**
   - Verify all agents in delegation chain have valid signatures
   ```rust
   for (idx, delegator) in event.delegation_chain.iter().enumerate() {
       let delegation_proof = extract_delegation_proof(&event, idx)?;
       verify_delegation_signature(delegator, &delegation_proof)?;
   }
   ```

4. **Rate Limiting**
   - Limit events per agent per minute (detect flooding)
   - 1,000 events/minute per agent (configurable)

**Residual Risk**: **Low** (requires compromising agent's private key)

---

### Threat 2: Quota Bypass Attack

**Description**: Agent circumvents quota enforcement to consume unlimited resources.

**Attack Scenarios**:

**A. Clock Manipulation**:
- Agent backdates events to previous billing period
- Agent submits events with future timestamps to prepay quota

**B. Cache Poisoning**:
- Attacker manipulates Redis quota cache to show unlimited quota
- Attacker invalidates quota cache to force recalculation (DoS)

**C. Race Condition**:
- Multiple agents simultaneously consume last quota unit
- Quota counter not atomically updated

**Impact**:
- Uncontrolled resource consumption
- Revenue loss
- Service degradation for other users

**Mitigations**:

1. **Consensus-Ordered Timestamps** (Prevents Clock Manipulation)
   ```rust
   async fn assign_consensus_timestamp(event: &mut EventRequest) -> Result<(), Error> {
       // Ignore agent-provided timestamp
       let consensus_time = consensus_client.get_current_time().await?;

       // Validate agent timestamp within ±10 minutes (detect gross clock skew)
       if let Some(agent_ts) = event.timestamp {
           let skew = (agent_ts - consensus_time).abs();
           if skew > Duration::from_secs(600) {
               return Err(Error::TimestampSkew {
                   agent_timestamp: agent_ts,
                   consensus_timestamp: consensus_time,
                   skew,
               });
           }
       }

       // Override with consensus timestamp (source of truth)
       event.timestamp = Some(consensus_time);
       Ok(())
   }
   ```

2. **Atomic Quota Updates** (Prevents Race Conditions)
   ```sql
   -- PostgreSQL transaction with row-level locking
   BEGIN;

   -- Lock quota row (prevents concurrent modification)
   SELECT * FROM quotas
   WHERE subscription_id = $1 AND event_type = $2
   FOR UPDATE;

   -- Check current usage atomically
   SELECT SUM(event_count) AS usage FROM usage_hourly
   WHERE subscription_id = $1 AND event_type = $2 AND hour >= $3;

   -- If usage < limit, allow event
   -- Else, deny

   COMMIT;
   ```

3. **Cache Integrity Protection**
   ```rust
   // Redis quota cache with HMAC signature
   struct SignedQuotaState {
       state: QuotaState,
       signature: Vec<u8>,  // HMAC-SHA256(state || secret_key)
   }

   async fn get_quota_from_cache(key: &QuotaKey) -> Result<Option<QuotaState>, Error> {
       let signed: Option<SignedQuotaState> = redis.get(key).await?;

       if let Some(signed) = signed {
           // Verify HMAC signature (detect tampering)
           let valid = verify_hmac(&signed.state, &signed.signature)?;
           if !valid {
               // Cache poisoning detected!
               audit_log.log_security_event(SecurityEvent::CacheTampering { key }).await;
               return Ok(None); // Invalidate cache, force DB lookup
           }
           Ok(Some(signed.state))
       } else {
           Ok(None)
       }
   }
   ```

4. **Quota Reservation Pattern** (Prevents TOCTOU)
   ```rust
   // Reserve quota BEFORE executing action
   let reservation_id = metering.reserve_quota(agent, event_type, 1).await?;

   // Execute action
   match execute_action().await {
       Ok(_) => {
           // Commit reservation (deduct quota)
           metering.commit_reservation(reservation_id).await?;
       }
       Err(_) => {
           // Rollback reservation (return quota)
           metering.rollback_reservation(reservation_id).await?;
       }
   }
   ```

5. **Quota Check Idempotency**
   - QuotaEnforcer checks are read-only (safe to retry)
   - Actual quota deduction happens on event ingestion (idempotent via idempotency_key)

**Residual Risk**: **Low** (requires compromising consensus layer or database)

---

### Threat 3: Event Replay Attack

**Description**: Attacker intercepts signed event and replays it to inflate bill.

**Attack Scenario**:
```
1. Agent A submits legitimate event (signature valid)
2. Attacker intercepts event on network
3. Attacker replays same event 1000 times
4. Without idempotency: Customer billed 1000x for single action
```

**Impact**:
- Overcharging customers
- Loss of trust in billing system
- Potential legal liability

**Mitigations**:

1. **Idempotency Key (Primary Defense)**
   ```rust
   async fn check_idempotency(event: &EventRequest) -> Result<IdempotencyResult, Error> {
       // Check if idempotency_key exists in cache
       if let Some(existing) = idempotency_cache.get(&event.idempotency_key).await? {
           // Replay detected!
           if existing.hash == event.canonical_hash() {
               // Same data: Return existing event_id (no-op)
               return Ok(IdempotencyResult::Duplicate(existing.event_id));
           } else {
               // Different data: Conflict (possible attack or client bug)
               return Err(Error::IdempotencyConflict {
                   existing_hash: existing.hash,
                   submitted_hash: event.canonical_hash(),
               });
           }
       }

       Ok(IdempotencyResult::New)
   }
   ```

2. **Nonce in Signature**
   ```rust
   // Agent includes unique nonce in signed message
   struct SignedEventData {
       idempotency_key: String,  // UUID (unique per event)
       agent_nhi: String,
       event_type: String,
       timestamp: DateTime<Utc>,
       properties: serde_json::Value,
       nonce: [u8; 32],  // Random 256-bit nonce
   }

   // Attacker cannot replay without changing signature
   ```

3. **Idempotency Window**
   - Store idempotency keys for 7 days (configurable)
   - After window expires, keys can be reused (acceptable for billing)
   - Old events archived to cold storage (cannot be replayed)

4. **Rate Limiting Per Idempotency Key**
   ```rust
   // Detect rapid replay attempts
   let attempts = rate_limiter.count_attempts(&event.idempotency_key).await?;

   if attempts > 5 {
       // Likely replay attack (same key submitted >5 times in 1 minute)
       audit_log.log_security_event(SecurityEvent::ReplayAttack {
           idempotency_key: event.idempotency_key.clone(),
           attempts,
       }).await;

       return Err(Error::TooManyRequests);
   }
   ```

**Residual Risk**: **Very Low** (idempotency provides strong protection)

---

### Threat 4: Subscription Impersonation

**Description**: Attacker submits events claiming to belong to victim's subscription to:
- Inflate victim's bill
- Exhaust victim's quota

**Attack Scenario**:
```
Attacker submits event:
{
  "agent_nhi": "agent:attacker",  // Attacker's identity (valid signature)
  "delegation_chain": ["human:victim@example.com"],  // Falsified delegation
  "event_type": "llm_tokens",
  "properties": {"tokens": 1000000}
}
```

**Impact**:
- Victim charged for attacker's usage
- Quota exhaustion (DoS)

**Mitigations**:

1. **Delegation Chain Verification**
   ```rust
   async fn verify_delegation_chain(event: &EventRequest) -> Result<(), Error> {
       // Step 1: Verify leaf agent signed the event
       verify_event_signature(event).await?;

       // Step 2: Verify each delegation in chain has valid proof
       for (idx, delegator) in event.delegation_chain.iter().enumerate() {
           // Extract delegation certificate (signed by delegator)
           let cert = extract_delegation_cert(event, idx)?;

           // Verify delegator signed certificate delegating to agent
           let delegator_pubkey = nhi_registry.get_public_key(delegator).await?;
           let valid = crypto_client.verify_signature(
               &cert.signature,
               &cert.canonical_data(),
               &delegator_pubkey,
               SignatureAlgorithm::MLDSA65,
           )?;

           if !valid {
               return Err(Error::InvalidDelegation {
                   delegator: delegator.clone(),
                   delegatee: event.agent_nhi.clone(),
               });
           }

           // Verify delegation is not expired
           if cert.expires_at < Utc::now() {
               return Err(Error::DelegationExpired {
                   delegator: delegator.clone(),
                   expired_at: cert.expires_at,
               });
           }
       }

       Ok(())
   }
   ```

2. **Subscription-Agent Binding**
   ```sql
   -- Table: subscription_agents (whitelist)
   CREATE TABLE subscription_agents (
       subscription_id UUID NOT NULL,
       agent_nhi TEXT NOT NULL,
       authorized_by TEXT NOT NULL,  -- Human who authorized agent
       authorized_at TIMESTAMPTZ NOT NULL,
       expires_at TIMESTAMPTZ,
       PRIMARY KEY (subscription_id, agent_nhi)
   );

   -- Before accepting event, verify agent authorized for subscription
   SELECT 1 FROM subscription_agents
   WHERE subscription_id = $1
     AND agent_nhi = $2
     AND (expires_at IS NULL OR expires_at > now());
   ```

3. **Root Principal Verification**
   ```rust
   // Verify root of delegation chain owns the subscription
   async fn verify_root_principal(event: &EventRequest) -> Result<(), Error> {
       let root_principal = event.delegation_chain.last()
           .ok_or(Error::EmptyDelegationChain)?;

       // Look up subscription owner
       let subscription = storage.get_subscription(&event.subscription_id).await?;

       // Verify root principal matches subscription owner
       if subscription.owner != *root_principal {
           return Err(Error::UnauthorizedSubscription {
               root_principal: root_principal.clone(),
               subscription_owner: subscription.owner.clone(),
           });
       }

       Ok(())
   }
   ```

**Residual Risk**: **Low** (requires forging delegation certificates)

---

### Threat 5: Pricing Manipulation

**Description**: Attacker modifies pricing configuration to reduce bill or gain competitive advantage.

**Attack Scenarios**:

**A. Admin Privilege Escalation**:
- Attacker compromises admin account
- Changes pricing model from $0.01/unit to $0.00/unit

**B. Database Injection**:
- SQL injection in pricing configuration endpoint
- Direct modification of `pricing_models` table

**C. Time-of-Check-to-Time-of-Use (TOCTOU)**:
- Pricing changed after usage calculated but before invoice issued
- Inconsistent invoice amounts

**Impact**:
- Revenue loss
- Competitive advantage for attackers
- Billing disputes

**Mitigations**:

1. **Immutable Pricing Snapshots**
   ```rust
   // Store pricing config with invoice (audit trail)
   struct LineItem {
       pub metric_code: String,
       pub quantity: Decimal,
       pub amount: Decimal,
       pub pricing_snapshot: PricingModel,  // Full config at invoice time
   }

   // Pricing changes only affect FUTURE usage
   async fn calculate_line_item_charge(
       usage: Decimal,
       metric_config: &BillableMetric,
       invoice_time: DateTime<Utc>,
   ) -> Result<LineItem, Error> {
       // Get pricing model active at invoice time (not current)
       let pricing = storage.get_pricing_model_at_time(
           &metric_config.metric_id,
           invoice_time,
       ).await?;

       let amount = pricing.calculate_charge(usage);

       Ok(LineItem {
           metric_code: metric_config.metric_code.clone(),
           quantity: usage,
           amount,
           pricing_snapshot: pricing, // Immutable record
       })
   }
   ```

2. **Pricing Change Audit Log**
   ```rust
   async fn update_pricing_model(
       admin: &AdminIdentity,
       pricing_model_id: Uuid,
       new_config: PricingModel,
   ) -> Result<(), Error> {
       // Load existing pricing
       let old_pricing = storage.get_pricing_model(pricing_model_id).await?;

       // Log change to audit
       audit_log.log_pricing_change(AuditEvent {
           actor: admin.identity.clone(),
           action: "pricing.update",
           resource: pricing_model_id.to_string(),
           old_value: serde_json::to_value(&old_pricing)?,
           new_value: serde_json::to_value(&new_config)?,
           timestamp: Utc::now(),
       }).await?;

       // Create new pricing record (versioned)
       storage.create_pricing_model_version(PricingModelVersion {
           pricing_model_id,
           config: new_config,
           active_from: Utc::now(),
           active_until: None,
           created_by: admin.identity.clone(),
       }).await?;

       // Deactivate old pricing
       storage.deactivate_pricing_model(pricing_model_id, Utc::now()).await?;

       Ok(())
   }
   ```

3. **Role-Based Access Control (RBAC)**
   ```rust
   // Only admin role can modify pricing
   #[require_role("billing_admin")]
   async fn update_pricing_model(...) { }

   // Billing service has read-only access
   #[require_role("billing_readonly")]
   async fn generate_invoice(...) { }
   ```

4. **Multi-Party Approval for Critical Changes**
   ```rust
   // Pricing changes >10% require two admin approvals
   async fn approve_pricing_change(
       approver: &AdminIdentity,
       change_request_id: Uuid,
   ) -> Result<(), Error> {
       let request = storage.get_pricing_change_request(change_request_id).await?;

       // Check if change is critical (>10% difference)
       let pct_change = calculate_price_change_percentage(&request)?;

       if pct_change > 0.10 {
           // Require second approval
           if request.approvals.len() < 2 {
               storage.add_approval(change_request_id, approver).await?;
               return Ok(());
           }
       }

       // Apply change
       apply_pricing_change(&request).await?;
       Ok(())
   }
   ```

**Residual Risk**: **Low** (requires admin compromise + audit trail provides deterrence)

---

### Threat 6: Quota Starvation (DoS)

**Description**: Attacker exhausts quotas for legitimate users to deny service.

**Attack Scenarios**:

**A. Quota Flooding**:
- Attacker with valid subscription floods events to exhaust own quota
- Side effect: Degrades service for others (database overload)

**B. Distributed Quota Exhaustion**:
- Attacker creates multiple free-tier accounts
- Each account exhausts free quota, then creates new account

**C. Quota Cache Eviction**:
- Attacker submits events for many different event types
- Evicts hot quotas from cache, forcing DB lookups (latency spike)

**Impact**:
- Legitimate users denied access (quota exceeded)
- Service degradation (slow quota checks)
- Revenue loss (users churn to competitors)

**Mitigations**:

1. **Per-Subscription Rate Limiting**
   ```rust
   // Limit event ingestion rate per subscription
   let rate_limit = RateLimit {
       max_events_per_minute: 1000,
       max_events_per_hour: 50000,
   };

   if !rate_limiter.check(&event.subscription_id, &rate_limit).await? {
       return Err(Error::RateLimitExceeded {
           limit: rate_limit.max_events_per_minute,
           window: Duration::from_secs(60),
       });
   }
   ```

2. **Quota Exhaustion Alerts**
   ```rust
   // Monitor quotas approaching limits
   async fn check_quota_health(quota_state: &QuotaState) -> Result<(), Error> {
       let usage_pct = (quota_state.current_usage as f64) / (quota_state.limit as f64);

       if usage_pct > 0.9 {
           // Send alert to subscription owner
           notification_service.send_quota_warning(QuotaWarning {
               subscription_id: quota_state.subscription_id,
               event_type: quota_state.event_type.clone(),
               current_usage: quota_state.current_usage,
               limit: quota_state.limit,
               usage_percentage: usage_pct,
               time_until_reset: quota_state.time_until_reset(),
           }).await?;
       }

       Ok(())
   }
   ```

3. **Quota Overflow Actions**
   ```rust
   // Allow overage with fee (instead of hard block)
   enum OverflowAction {
       Block,  // Deny access (DoS risk)
       AllowWithOverage,  // Allow but charge premium ($$$/unit)
       NotifyOnly,  // Allow, send notification (soft limit)
   }

   // Example: Free tier uses Block, paid tier uses AllowWithOverage
   ```

4. **Graduated Quota Limits**
   ```rust
   // Free tier: 1,000 events/month
   // Starter: 10,000 events/month
   // Pro: 100,000 events/month
   // Enterprise: Unlimited (or very high limit)

   // Attackers less motivated to exhaust quotas if easy to upgrade
   ```

5. **IP-Based Rate Limiting**
   ```rust
   // Detect distributed attacks from same IP range
   let ip_rate_limit = RateLimit {
       max_subscriptions_per_ip_per_hour: 10,
   };

   // If single IP creates >10 subscriptions in 1 hour, block IP
   ```

**Residual Risk**: **Medium** (difficult to fully prevent without affecting legitimate bursts)

---

## Authentication & Authorization

### Agent Authentication

**Method**: NHI-based authentication with ML-DSA signatures

```rust
async fn authenticate_agent(request: &Request) -> Result<AgentIdentity, Error> {
    // Step 1: Extract Bearer token from Authorization header
    let token = extract_bearer_token(request)?;

    // Step 2: Verify JWT (issued by creto-authz)
    let claims = jwt_verifier.verify(token).await?;

    // Step 3: Extract agent_nhi from claims
    let agent_nhi = claims.subject()
        .ok_or(Error::InvalidToken)?;

    // Step 4: Verify agent NHI is registered
    let agent = nhi_registry.get_agent(&agent_nhi).await?;

    Ok(agent)
}
```

### API Authorization

**Scopes**:
- `events:write` - Submit billable events
- `events:read` - Query events
- `quotas:read` - Query quota status
- `quotas:write` - Modify quota configuration (admin only)
- `invoices:read` - View invoices
- `invoices:write` - Generate invoices (billing service only)

**Authorization Check**:
```rust
#[require_scope("events:write")]
async fn ingest_event(request: IngestEventRequest) -> Result<IngestEventResponse, Error> {
    // Authorization already verified by middleware
    // ...
}
```

---

## Data Protection

### Encryption at Rest

**PostgreSQL**: Transparent Data Encryption (TDE)
```sql
-- Enable encryption (PostgreSQL 14+)
ALTER DATABASE creto_metering SET encryption = on;
```

**Redis**: Encryption via Linux dm-crypt (disk-level)
```bash
# Encrypt Redis data directory
cryptsetup luksFormat /dev/sdb
cryptsetup luksOpen /dev/sdb redis_encrypted
mkfs.ext4 /dev/mapper/redis_encrypted
mount /dev/mapper/redis_encrypted /var/lib/redis
```

**S3**: Server-Side Encryption (SSE-KMS)
```bash
aws s3api put-bucket-encryption \
  --bucket creto-metering-cold-storage \
  --server-side-encryption-configuration '{
    "Rules": [{
      "ApplyServerSideEncryptionByDefault": {
        "SSEAlgorithm": "aws:kms",
        "KMSMasterKeyID": "arn:aws:kms:us-east-1:123456789012:key/abc-def"
      }
    }]
  }'
```

### Encryption in Transit

**TLS 1.3** for all network communication:
```rust
// PostgreSQL connection with TLS
let pg_config = "host=db.creto.io sslmode=require sslrootcert=/etc/ssl/certs/ca.crt";

// Redis connection with TLS
let redis_config = RedisConfig {
    url: "rediss://redis.creto.io:6380",  // rediss:// = TLS
    tls_config: Some(TlsConfig {
        ca_cert: "/etc/ssl/certs/ca.crt",
        client_cert: Some("/etc/ssl/certs/client.crt"),
        client_key: Some("/etc/ssl/private/client.key"),
    }),
};

// Kafka connection with TLS + SASL
let kafka_config = KafkaConfig {
    brokers: vec!["kafka.creto.io:9093"],
    security_protocol: "SASL_SSL",
    sasl_mechanism: "PLAIN",
    ssl_ca_location: "/etc/ssl/certs/ca.crt",
};
```

---

## Audit Logging

### Events Logged

| Event Type | Trigger | Fields |
|------------|---------|--------|
| `event.ingested` | Event submitted | agent_nhi, subscription_id, event_type, properties, signature |
| `quota.checked` | Quota check performed | agent_nhi, event_type, decision, remaining |
| `quota.exceeded` | Quota limit reached | agent_nhi, event_type, limit, current_usage |
| `invoice.generated` | Invoice created | subscription_id, period, total, line_items |
| `pricing.changed` | Pricing updated | admin, old_config, new_config, affected_subscriptions |
| `signature.failed` | Invalid signature | agent_nhi, event_id, reason |
| `cache.tampered` | Cache HMAC mismatch | cache_key, expected_hmac, actual_hmac |

### Audit Log Format

```rust
struct AuditEvent {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor: String,  // agent_nhi or human identity
    pub action: String,  // e.g., "event.ingested"
    pub resource: String,  // e.g., "subscription:123"
    pub outcome: AuditOutcome,
    pub metadata: serde_json::Value,
    pub signature: Vec<u8>,  // Event signed by metering service
}

enum AuditOutcome {
    Success,
    Denied { reason: String },
    Failed { error: String },
}
```

### Audit Storage

**Backend**: creto-audit service (Merkle tree + blockchain anchoring)

```rust
async fn log_audit_event(event: AuditEvent) -> Result<(), Error> {
    // Submit to audit service (async, non-blocking)
    audit_client.log_event(event).await?;

    // Audit service handles:
    // 1. Append to Merkle tree
    // 2. Anchor root to blockchain (Ethereum/Solana)
    // 3. Store in immutable log (append-only DB)

    Ok(())
}
```

---

## Incident Response Plan

### Security Event Categories

| Severity | Examples | Response Time |
|----------|----------|---------------|
| **Critical** | Database breach, mass quota bypass | <15 minutes |
| **High** | Invalid signature detected, cache tampering | <1 hour |
| **Medium** | Quota exceeded, rate limit exceeded | <4 hours |
| **Low** | Failed login attempt, idempotency conflict | <24 hours |

### Response Procedures

**Critical Incident (e.g., Database Breach)**:
1. **Immediate** (0-15 min):
   - Isolate affected systems (firewall rules)
   - Revoke all API tokens
   - Enable read-only mode for database

2. **Short-term** (15 min - 4 hours):
   - Analyze audit logs for breach scope
   - Notify affected customers
   - Rotate all cryptographic keys

3. **Long-term** (4 hours+):
   - Conduct forensic analysis
   - Implement additional mitigations
   - Publish incident report

**High Incident (e.g., Invalid Signature Detected)**:
1. Block agent (revoke credentials)
2. Analyze recent events from agent
3. Void fraudulent invoices
4. Notify customer

---

## Compliance Requirements

### SOX 404 (Sarbanes-Oxley)

**Requirements**:
- Immutable audit trail for all billing operations (7-year retention)
- Segregation of duties (billing admins ≠ finance team)
- Multi-party approval for pricing changes

**Implementation**:
```rust
// Audit log stored in append-only table
CREATE TABLE audit_log (
    event_id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    actor TEXT NOT NULL,
    action TEXT NOT NULL,
    resource TEXT NOT NULL,
    outcome TEXT NOT NULL,
    metadata JSONB NOT NULL,
    signature BYTEA NOT NULL
) WITH (fillfactor = 100);  -- No updates allowed

-- Prevent deletion (only INSERT allowed)
REVOKE DELETE, UPDATE ON audit_log FROM ALL;
GRANT INSERT ON audit_log TO metering_service;
GRANT SELECT ON audit_log TO audit_readonly;
```

### GDPR (General Data Protection Regulation)

**Right to Erasure**:
- Events cannot be deleted (billing integrity)
- Solution: Anonymize PII (replace agent_nhi with hash)

```sql
-- Anonymize events for deleted user
UPDATE events
SET agent_nhi = 'agent:anonymized:' || md5(agent_nhi),
    delegation_chain = '["anonymized"]'::jsonb,
    properties = '{}'::jsonb  -- Remove PII from properties
WHERE agent_nhi LIKE 'agent:nhi:%:user-to-delete';
```

### PCI DSS (Payment Card Industry)

**Out of Scope**: creto-metering does not store payment card data
- Payment processing delegated to Stripe (PCI Level 1 certified)
- Metering only stores Stripe invoice IDs (not card numbers)

---

## Security Testing

### Penetration Testing

**Quarterly Tests**:
1. Signature bypass attempts
2. Quota exhaustion attacks
3. SQL injection in API endpoints
4. Cache poisoning
5. Replay attacks

**Tools**:
- Burp Suite (API fuzzing)
- SQLMap (SQL injection)
- Custom scripts (quota bypass, replay)

### Fuzz Testing

```rust
#[test]
fn fuzz_event_ingestion() {
    let mut fuzzer = Fuzzer::new();

    for _ in 0..10000 {
        let random_event = fuzzer.generate_random_event();

        // Should not crash or leak data
        let result = ingestion_service.ingest_event(random_event).await;

        // Valid errors expected, but no panics
        assert!(!matches!(result, Err(Error::Panic)));
    }
}
```

---

**Next Document**: SDD-MTR-06: Testing Strategy (unit, integration, load tests, benchmarks)
