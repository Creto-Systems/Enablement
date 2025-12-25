---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
parent_sdd: docs/sdd/products/oversight/01-requirements.md
---

# SDD-OVS-05: Oversight Security Design

## 1. Security Overview

### 1.1 Security Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     SECURITY PERIMETER                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Authentication Layer                       │  │
│  │  • mTLS for gRPC (service-to-service)                   │  │
│  │  • API keys for REST (external services)                │  │
│  │  • OAuth 2.0 for user-facing apps                       │  │
│  │  • HMAC signatures for webhooks                         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              ↓                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Authorization Layer                        │  │
│  │  • Verify caller identity (NHI, human, service)         │  │
│  │  • Check permissions (via creto-authz)                  │  │
│  │  • Validate approver eligibility                        │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              ↓                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Cryptographic Verification                 │  │
│  │  • Signature verification (ML-DSA, Ed25519)             │  │
│  │  • Approval non-repudiation                             │  │
│  │  • Merkle proof generation (audit trail)               │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              ↓                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Data Protection                            │  │
│  │  • Encryption at rest (PostgreSQL TDE)                  │  │
│  │  • Encryption in transit (TLS 1.3)                      │  │
│  │  • Sensitive field encryption (pgcrypto)                │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              ↓                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Audit & Compliance                         │  │
│  │  • Immutable audit log (Merkle-anchored)                │  │
│  │  • All state transitions logged                         │  │
│  │  • Cryptographic proofs stored                          │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### 1.2 Security Principles

1. **Zero Trust:** Verify every request, never trust implicit identity
2. **Defense in Depth:** Multiple layers of security controls
3. **Least Privilege:** Minimal permissions required for each operation
4. **Non-Repudiation:** Cryptographic proof of all approvals
5. **Auditability:** Complete, immutable audit trail
6. **Crypto-Agility:** Algorithm flexibility via creto-crypto layer

---

## 2. Threat Model

### 2.1 Threat Actors

| Actor | Motivation | Capabilities | Likelihood |
|-------|-----------|-------------|-----------|
| **Malicious Insider** | Bypass oversight, approve unauthorized actions | Valid credentials, system access | Medium |
| **External Attacker** | Disrupt approvals, steal data | Network access, phishing | Low |
| **Compromised Agent** | Execute unauthorized actions without oversight | Agent NHI credentials | Medium |
| **Malicious Approver** | Approve harmful requests | Approver credentials, signing keys | Low |
| **Channel Impersonator** | Forge approval responses | Network interception, webhook access | Medium |

### 2.2 Threat Scenarios

#### 2.2.1 Approval Bypass Attacks

**Threat:** Agent attempts to execute action without oversight requirement

**Attack Vectors:**
1. **Direct API Call:** Agent calls action API directly, bypassing Authorization service
2. **Tampered Policy:** Modify policy to remove oversight requirement
3. **Override Token Replay:** Reuse override token from previous approval
4. **State Manipulation:** Directly update database state to `APPROVED`

**Mitigations:**

| Mitigation | Description | Effectiveness |
|-----------|-------------|---------------|
| **Policy Enforcement Point** | All actions MUST go through Authorization service first | High |
| **Immutable Policies** | Policies stored in Authorization service, not modifiable by agents | High |
| **One-Time Override Tokens** | Override tokens single-use, expire after 60 seconds | High |
| **Database Access Control** | Application-only database access, no direct agent access | High |
| **Audit Correlation** | Correlate override token usage with approval audit records | Medium |

**Implementation:**
```rust
// Override token generation (Authorization service)
pub async fn generate_override_token(
    &self,
    request_id: RequestId,
    approval_proof: &ApprovalProof,
) -> Result<OverrideToken> {
    // Verify approval proof signature
    self.crypto.verify_signature(approval_proof)?;

    // Generate one-time token
    let token = OverrideToken {
        request_id,
        approval_proof_hash: hash(approval_proof),
        expires_at: Timestamp::now() + Duration::from_secs(60),
        nonce: generate_random_nonce(),
    };

    // Store in Redis (single-use enforcement)
    self.redis.set_ex(
        format!("override:{}", token.nonce),
        serde_json::to_string(&token)?,
        60,
    ).await?;

    // Sign token
    let signed_token = self.crypto.sign(&token)?;

    Ok(signed_token)
}

// Override token verification (action execution)
pub async fn verify_override_token(
    &self,
    token: &OverrideToken,
    signature: &Signature,
) -> Result<()> {
    // Verify signature
    self.crypto.verify_signature(token, signature)?;

    // Check expiration
    if token.expires_at < Timestamp::now() {
        return Err(Error::TokenExpired);
    }

    // Check one-time use
    let key = format!("override:{}", token.nonce);
    if !self.redis.exists(&key).await? {
        return Err(Error::TokenAlreadyUsed);
    }

    // Consume token (delete from Redis)
    self.redis.del(&key).await?;

    Ok(())
}
```

#### 2.2.2 Channel Impersonation

**Threat:** Attacker forges approval response pretending to be Slack, email, or webhook

**Attack Vectors:**
1. **Webhook Replay:** Capture legitimate webhook, replay with modified data
2. **Slack Signature Forgery:** Forge `X-Slack-Signature` header
3. **Email Token Theft:** Steal approval link from approver's email
4. **Cross-Channel Forgery:** Submit Slack response via email endpoint

**Mitigations:**

| Mitigation | Description | Effectiveness |
|-----------|-------------|---------------|
| **Channel-Specific Verification** | Each channel verifies authenticity (HMAC, signing secret, token) | High |
| **Signature Binding** | Approver signature bound to channel metadata (message TS, nonce) | High |
| **Token Nonce Tracking** | One-time use enforcement via Redis | High |
| **Timestamp Validation** | Reject requests with timestamp skew >5 minutes | Medium |
| **IP Allowlisting** | Webhook callbacks only from known IPs (Slack, ServiceNow) | Medium |

**Implementation:**
```rust
// Slack signature verification
pub async fn verify_slack_signature(
    &self,
    body: &str,
    timestamp: &str,
    signature: &str,
) -> Result<()> {
    // Check timestamp freshness (prevent replay attacks)
    let request_time = timestamp.parse::<i64>()?;
    let current_time = Timestamp::now().as_secs();

    if (current_time - request_time).abs() > 300 {  // 5 minutes
        return Err(Error::TimestampTooOld);
    }

    // Compute expected signature
    let signing_base = format!("v0:{}:{}", timestamp, body);
    let expected_signature = format!(
        "v0={}",
        hex::encode(hmac::Mac::new_from_slice(
            SLACK_SIGNING_SECRET.as_bytes()
        )?.chain_update(signing_base.as_bytes()).finalize().into_bytes())
    );

    // Constant-time comparison (prevent timing attacks)
    if !constant_time_eq(signature.as_bytes(), expected_signature.as_bytes()) {
        return Err(Error::InvalidSignature);
    }

    Ok(())
}

// Email token verification with nonce tracking
pub async fn verify_email_token(
    &self,
    token: &str,
) -> Result<ApprovalTokenClaims> {
    // Decode JWT
    let claims: ApprovalTokenClaims = jsonwebtoken::decode(
        token,
        &DecodingKey::from_secret(EMAIL_TOKEN_SECRET),
        &Validation::default(),
    )?.claims;

    // Check expiration
    if claims.exp < Timestamp::now().as_secs() {
        return Err(Error::TokenExpired);
    }

    // Check one-time use (nonce in Redis)
    let nonce_key = format!("email_nonce:{}", claims.nonce);
    if self.redis.exists(&nonce_key).await? {
        return Err(Error::TokenAlreadyUsed);
    }

    // Mark nonce as used
    self.redis.set_ex(&nonce_key, "1", claims.exp - Timestamp::now().as_secs()).await?;

    Ok(claims)
}
```

#### 2.2.3 Signature Forgery

**Threat:** Attacker forges approver signature to approve malicious request

**Attack Vectors:**
1. **Private Key Theft:** Steal approver's private key
2. **Signature Replay:** Reuse signature from different request
3. **Weak Signature Scheme:** Exploit cryptographic weakness
4. **Public Key Substitution:** Replace approver's public key with attacker's

**Mitigations:**

| Mitigation | Description | Effectiveness |
|-----------|-------------|---------------|
| **Post-Quantum Signatures** | ML-DSA-65 (NIST FIPS 204) resistant to quantum attacks | High |
| **Signature Binding** | Signature over (request_id \|\| decision \|\| timestamp) prevents replay | High |
| **NHI Registry Trust** | Public keys fetched from trusted NHI registry, not request payload | High |
| **Key Rotation** | Approver keys rotated every 90 days | Medium |
| **Hardware Security Modules** | Private keys stored in HSM, never exposed | High |

**Implementation:**
```rust
// Signature verification
pub async fn verify_approval_signature(
    &self,
    request_id: RequestId,
    decision: &ApprovalDecision,
    timestamp: Timestamp,
    signature: &Signature,
    approver: &ApproverIdentity,
) -> Result<()> {
    // Fetch public key from NHI registry (trusted source)
    let public_key = self.nhi_registry.get_public_key(&approver.subject).await?;

    // Verify public key matches provided key (prevent substitution)
    if public_key != approver.public_key {
        return Err(Error::PublicKeyMismatch);
    }

    // Construct signed message (request_id || decision || timestamp)
    let message = format!(
        "{}||{}||{}",
        request_id,
        decision.to_string(),
        timestamp.as_secs()
    );

    // Verify signature using crypto-agility layer
    self.crypto_agility.verify(
        &public_key,
        &signature.algorithm,
        message.as_bytes(),
        &signature.value,
    ).await?;

    Ok(())
}
```

#### 2.2.4 Timeout Manipulation

**Threat:** Attacker manipulates timeout to force auto-approval or auto-denial

**Attack Vectors:**
1. **Clock Skew:** Manipulate system clock to trigger premature timeout
2. **Database Tampering:** Modify timeout values in database
3. **Policy Injection:** Inject malicious policy with favorable timeout
4. **Scheduler Bypass:** Prevent timeout scheduler from running

**Mitigations:**

| Mitigation | Description | Effectiveness |
|-----------|-------------|---------------|
| **Consensus Time Source** | Use consensus-ordered time from creto-consensus, not system clock | High |
| **Immutable Timeouts** | Timeout values set at request creation, cannot be modified | High |
| **Policy Integrity** | Policies signed by administrator, tamper-evident | High |
| **Heartbeat Monitoring** | Timeout scheduler health checks, alerting on failures | Medium |
| **Checkpoint Recovery** | On restart, recalculate remaining timeout from creation time | High |

**Implementation:**
```rust
// Timeout calculation with consensus time
pub async fn schedule_timeout(
    &self,
    request_id: RequestId,
    tier_index: usize,
    timeout_duration: Duration,
) -> Result<()> {
    // Use consensus-ordered time (not system clock)
    let creation_time = self.consensus.get_ordered_timestamp().await?;
    let expiration_time = creation_time + timeout_duration;

    // Store timeout in persistent scheduler
    self.timeout_scheduler.schedule(TimeoutTask {
        request_id,
        tier_index,
        expiration_time,  // Absolute time, not relative
        scheduled_at: creation_time,
    }).await?;

    Ok(())
}

// On restart: recalculate remaining timeout
pub async fn recover_timeouts(&self) -> Result<()> {
    let pending_requests = self.state_store.load_pending_requests().await?;

    for request in pending_requests {
        // Fetch original timeout duration from policy
        let policy = self.policy_engine.get_policy(&request.policy_id).await?;
        let tier = &policy.escalation_chain.tiers[request.tier_index];

        // Calculate remaining timeout
        let elapsed = self.consensus.get_ordered_timestamp().await? - request.created_at;
        let remaining = tier.timeout.saturating_sub(elapsed);

        if remaining == Duration::ZERO {
            // Timeout already expired, trigger immediately
            self.request_manager.handle_tier_timeout(request.request_id, request.tier_index).await?;
        } else {
            // Reschedule with remaining time
            self.schedule_timeout(request.request_id, request.tier_index, remaining).await?;
        }
    }

    Ok(())
}
```

#### 2.2.5 SQL Injection

**Threat:** Attacker injects malicious SQL via user-controlled inputs

**Attack Vectors:**
1. **Request ID Injection:** Malicious UUID in `/requests/{id}` endpoint
2. **Filter Injection:** SQL in `state`, `agent_nhi`, `approver_subject` query parameters
3. **Channel Metadata Injection:** JSON fields in `channel_metadata`

**Mitigations:**

| Mitigation | Description | Effectiveness |
|-----------|-------------|---------------|
| **Parameterized Queries** | All database queries use prepared statements (sqlx) | High |
| **Input Validation** | Strict validation of UUIDs, enums, email addresses | High |
| **Type Safety** | Rust type system prevents string concatenation in queries | High |
| **Least Privilege DB User** | Application DB user has minimal permissions (no DDL) | Medium |
| **SQL Audit Logging** | All queries logged for forensic analysis | Low |

**Implementation:**
```rust
// Parameterized query (safe from SQL injection)
pub async fn get_request(
    &self,
    request_id: RequestId,
) -> Result<RequestState> {
    // request_id is strongly-typed UUID, cannot contain SQL
    let state = sqlx::query_as!(
        RequestState,
        r#"
        SELECT * FROM oversight_requests
        WHERE request_id = $1
        "#,
        request_id  // Bound parameter, not string concatenation
    )
    .fetch_one(&self.pool)
    .await?;

    Ok(state)
}

// Input validation for list endpoint
pub async fn list_requests(
    &self,
    filters: ListRequestsFilters,
) -> Result<Vec<RequestState>> {
    // Validate state enum
    for state in &filters.states {
        if !["PENDING", "APPROVED", "DENIED", "ESCALATED", "TIMEOUT", "CANCELLED"].contains(&state.as_str()) {
            return Err(Error::InvalidState);
        }
    }

    // Validate agent_nhi format (must be valid NHI)
    if let Some(agent_nhi) = &filters.agent_nhi {
        AgentNhi::from_str(agent_nhi)
            .map_err(|_| Error::InvalidAgentNhi)?;
    }

    // Parameterized query with validated inputs
    let states = sqlx::query_as!(
        RequestState,
        r#"
        SELECT * FROM oversight_requests
        WHERE state = ANY($1)
          AND ($2::TEXT IS NULL OR agent_nhi = $2)
        "#,
        &filters.states,
        filters.agent_nhi.as_deref()
    )
    .fetch_all(&self.pool)
    .await?;

    Ok(states)
}
```

---

## 3. Cryptographic Security

### 3.1 Signature Algorithms

**Supported Algorithms (via Crypto-Agility):**

| Algorithm | Key Size | Security Level | Use Case | Status |
|-----------|----------|---------------|----------|--------|
| **ML-DSA-65** | 1952 bytes (public), 4032 bytes (private) | NIST Level 3 (quantum-resistant) | Production approvals | Recommended |
| **ML-DSA-87** | 2592 bytes (public), 4896 bytes (private) | NIST Level 5 (quantum-resistant) | High-security approvals | Supported |
| **Ed25519** | 32 bytes (public), 64 bytes (private) | 128-bit classical | Legacy compatibility | Deprecated (2026) |

**Algorithm Selection:**
```rust
pub async fn sign_approval(
    &self,
    request_id: RequestId,
    decision: &ApprovalDecision,
    private_key: &PrivateKey,
) -> Result<Signature> {
    // Construct message to sign
    let message = format!(
        "{}||{}||{}",
        request_id,
        decision.to_string(),
        Timestamp::now().as_secs()
    );

    // Determine algorithm based on key type
    let algorithm = match private_key.algorithm() {
        KeyAlgorithm::MlDsa65 => "ML-DSA-65",
        KeyAlgorithm::MlDsa87 => "ML-DSA-87",
        KeyAlgorithm::Ed25519 => "Ed25519",
    };

    // Sign via crypto-agility layer
    let signature_value = self.crypto_agility.sign(
        private_key,
        algorithm,
        message.as_bytes(),
    ).await?;

    Ok(Signature {
        algorithm: algorithm.to_string(),
        value: signature_value,
    })
}
```

### 3.2 Encryption

**Data at Rest:**
- PostgreSQL Transparent Data Encryption (TDE) with AES-256
- Column-level encryption for sensitive fields (pgcrypto)

**Data in Transit:**
- TLS 1.3 for all HTTPS traffic
- mTLS for gRPC (service-to-service)
- Perfect Forward Secrecy (PFS) enabled

**Encryption Configuration:**
```rust
// TLS configuration
let tls_config = RustlsConfig::from_pem_file(
    "/etc/certs/oversight.crt",
    "/etc/certs/oversight.key",
).await?;

// Cipher suites (PFS-enabled only)
let cipher_suites = vec![
    CipherSuite::TLS13_AES_256_GCM_SHA384,
    CipherSuite::TLS13_CHACHA20_POLY1305_SHA256,
];

axum_server::bind_rustls(addr, tls_config)
    .serve(app.into_make_service())
    .await?;
```

### 3.3 Key Management

**Key Storage:**
- **Production:** AWS KMS, Google Cloud KMS, or HashiCorp Vault
- **Development:** Local keystore (encrypted with master password)

**Key Rotation:**
- Approver signing keys: 90 days
- API keys: 365 days
- Database encryption keys: 180 days (with re-encryption)

**Key Rotation Procedure:**
```rust
// Rotate approver signing key
pub async fn rotate_approver_key(
    &self,
    approver_subject: &str,
) -> Result<()> {
    // Generate new key pair
    let (new_private_key, new_public_key) = self.crypto_agility.generate_keypair(KeyAlgorithm::MlDsa65).await?;

    // Update NHI registry (atomic swap)
    self.nhi_registry.update_public_key(
        approver_subject,
        &new_public_key,
    ).await?;

    // Securely deliver new private key to approver (out-of-band)
    self.secure_delivery.send_private_key(
        approver_subject,
        &new_private_key,
    ).await?;

    // Old public key remains valid for 30 days (grace period)
    self.nhi_registry.mark_key_deprecated(
        approver_subject,
        &old_public_key,
        Duration::from_secs(30 * 86400),
    ).await?;

    Ok(())
}
```

---

## 4. Access Control

### 4.1 Role-Based Access Control (RBAC)

**Roles:**

| Role | Permissions | Description |
|------|------------|-------------|
| **OversightAdmin** | Create policies, view all requests, manage escalation chains | Platform administrator |
| **Approver** | View assigned requests, submit approvals/denials | Human approver |
| **Agent** | Create requests (via Authorization service) | AI agent |
| **Auditor** | View audit logs, read-only access | Compliance auditor |
| **ServiceAccount** | Create requests, query status | Internal service (Authorization, Memory) |

**Permission Matrix:**

| Operation | Admin | Approver | Agent | Auditor | ServiceAccount |
|-----------|-------|----------|-------|---------|----------------|
| Create request | ✓ | ✗ | ✗ | ✗ | ✓ |
| View request | ✓ | ✓ (assigned) | ✗ | ✓ | ✓ |
| Submit response | ✓ | ✓ | ✗ | ✗ | ✗ |
| Cancel request | ✓ | ✗ | ✓ (own) | ✗ | ✗ |
| View audit log | ✓ | ✗ | ✗ | ✓ | ✗ |
| Manage policies | ✓ | ✗ | ✗ | ✗ | ✗ |

**Implementation:**
```rust
#[derive(Debug, Clone)]
pub enum Permission {
    CreateRequest,
    ViewRequest,
    SubmitResponse,
    CancelRequest,
    ViewAuditLog,
    ManagePolicies,
}

pub async fn check_permission(
    &self,
    principal: &Principal,
    permission: Permission,
    resource: Option<&Resource>,
) -> Result<bool> {
    // Call Authorization service
    let decision = self.authz_client.authorize(AuthzRequest {
        principal: principal.clone(),
        action: permission.to_action(),
        resource: resource.cloned(),
    }).await?;

    match decision {
        Decision::Allow => Ok(true),
        Decision::Deny => Ok(false),
        Decision::RequiresOversight { .. } => {
            // Oversight cannot require oversight (infinite loop prevention)
            Err(Error::RecursiveOversightNotAllowed)
        }
    }
}
```

### 4.2 Row-Level Security (RLS)

**PostgreSQL RLS Policies:**
```sql
-- Enable RLS on oversight_requests
ALTER TABLE oversight_requests ENABLE ROW LEVEL SECURITY;

-- Policy: Approvers can only see requests they're eligible to approve
CREATE POLICY approver_read_policy ON oversight_requests
FOR SELECT
TO approver_role
USING (
    -- Check if current user is in the approver list for this request's tier
    EXISTS (
        SELECT 1
        FROM policies p
        INNER JOIN escalation_chains ec ON p.escalation_chain_id = ec.chain_id
        INNER JOIN escalation_tiers et ON ec.chain_id = et.chain_id
        WHERE p.policy_id = oversight_requests.policy_id
          AND et.tier_index = oversight_requests.tier_index
          AND current_user IN (SELECT jsonb_array_elements_text(et.approvers))
    )
);

-- Policy: Admins can see all requests
CREATE POLICY admin_read_policy ON oversight_requests
FOR SELECT
TO admin_role
USING (true);

-- Policy: Service accounts can see requests they created
CREATE POLICY service_read_policy ON oversight_requests
FOR SELECT
TO service_role
USING (agent_nhi = current_setting('app.service_nhi'));
```

---

## 5. Audit and Compliance

### 5.1 Immutable Audit Trail

**Merkle Tree Structure:**
```
                      Root Hash
                     /         \
                    /           \
            Hash(A,B)          Hash(C,D)
           /        \          /        \
       Hash(A)   Hash(B)   Hash(C)   Hash(D)
         |         |         |         |
    Audit    Audit      Audit      Audit
    Event A  Event B    Event C    Event D
```

**Implementation:**
```rust
pub struct MerkleAuditLog {
    tree: MerkleTree,
    root_hash: Hash,
}

impl MerkleAuditLog {
    pub async fn log_event(
        &mut self,
        event: AuditEvent,
    ) -> Result<AuditProof> {
        // Hash event
        let event_hash = blake3::hash(&serde_json::to_vec(&event)?);

        // Append to Merkle tree
        self.tree.append(event_hash);

        // Recompute root hash
        self.root_hash = self.tree.root();

        // Generate Merkle proof
        let proof = self.tree.generate_proof(event_hash)?;

        // Store event with proof in database
        sqlx::query!(
            r#"
            INSERT INTO approval_audit (
                audit_id, request_id, event_type, merkle_proof, timestamp
            ) VALUES ($1, $2, $3, $4, $5)
            "#,
            event.audit_id,
            event.request_id,
            event.event_type,
            proof.to_bytes(),
            event.timestamp,
        )
        .execute(&self.pool)
        .await?;

        Ok(proof)
    }

    pub async fn verify_event(
        &self,
        event: &AuditEvent,
        proof: &AuditProof,
    ) -> Result<bool> {
        let event_hash = blake3::hash(&serde_json::to_vec(event)?);
        Ok(proof.verify(event_hash, &self.root_hash))
    }
}
```

### 5.2 Compliance Requirements

**SOX 404 (Sarbanes-Oxley):**
- ✓ Dual control for financial transactions (2-of-2 quorum)
- ✓ Complete audit trail of approvals
- ✓ Segregation of duties (approver ≠ requester)
- ✓ 7-year retention of financial approval records

**EU AI Act Article 14:**
- ✓ Human oversight for high-risk AI systems
- ✓ Transparency (delegation chain, reasoning visible to approver)
- ✓ Human-in-the-loop decision points
- ✓ Audit trail of AI-assisted decisions

**HIPAA Minimum Necessary:**
- ✓ Oversight for PHI access requests
- ✓ Justification required (reasoning field)
- ✓ Audit log of all PHI access approvals

---

## 6. Security Testing

### 6.1 Penetration Testing

**Scope:**
- Approval bypass attempts
- Signature forgery
- Channel impersonation
- SQL injection
- Timeout manipulation

**Tools:**
- Burp Suite (web application testing)
- sqlmap (SQL injection)
- Custom scripts (approval bypass, signature replay)

**Schedule:**
- Annual penetration test by external firm
- Quarterly internal security review
- Continuous automated security scanning

### 6.2 Security Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Signature Verification Failure Rate** | <0.01% | % of responses with invalid signatures |
| **Channel Impersonation Attempts** | 0 | Count of detected impersonation attempts |
| **Timeout Manipulation Attempts** | 0 | Count of detected clock skew/tampering |
| **SQL Injection Attempts** | 0 | Count of detected injection patterns |
| **Approval Bypass Attempts** | 0 | Count of direct action calls without oversight |
| **Mean Time to Detect (MTTD)** | <5 minutes | Time from security event to alert |
| **Mean Time to Respond (MTTR)** | <30 minutes | Time from alert to incident response |

---

## 7. Incident Response

### 7.1 Security Incident Playbook

**Incident Types:**

1. **Approval Bypass Detected**
   - Alert: Authorization service received action call without override token
   - Response: Block agent, audit all recent actions, review policy

2. **Signature Verification Failure**
   - Alert: Invalid signature on approval response
   - Response: Reject response, notify approver, investigate key compromise

3. **Channel Impersonation**
   - Alert: HMAC signature mismatch on webhook
   - Response: Block IP, review all recent webhook responses, rotate secrets

4. **Timeout Manipulation**
   - Alert: Clock skew detected (consensus time vs system time >5s)
   - Response: Sync time source, review all timeout events in window

**Incident Response Flow:**
```
Detection → Alerting → Triage → Containment → Investigation → Remediation → Post-Mortem
```

### 7.2 Security Contacts

| Role | Contact | Escalation |
|------|---------|-----------|
| **Security Engineer (On-Call)** | security-oncall@company.com | 24/7 PagerDuty |
| **Security Lead** | security-lead@company.com | Business hours |
| **CISO** | ciso@company.com | Critical incidents |

---

**END OF DOCUMENT**
