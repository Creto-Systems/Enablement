---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
---

# SDD-05: Security Design

## Purpose

This document defines the security architecture for the Enablement Layer, including threat model, cryptographic design, authentication/authorization, and security controls.

## Scope

**In Scope:**
- Threat model per product
- Cryptographic algorithms and key management
- Authentication and authorization
- Secure communication
- Audit and non-repudiation
- Security controls and mitigations

**Out of Scope:**
- Platform layer security (creto-crypto internals)
- Physical security
- Operational security procedures

---

## 1. Security Principles

### 1.1 Design Principles

| Principle | Implementation |
|-----------|----------------|
| **Defense in depth** | Multiple security layers, no single point of failure |
| **Least privilege** | Minimal permissions, scoped access |
| **Zero trust** | Verify every request, assume breach |
| **Crypto-agility** | No hardcoded algorithms, PQC migration path |
| **Non-repudiation** | Signed operations, immutable audit |
| **Fail secure** | Default deny, safe error handling |

### 1.2 Trust Boundaries

```
┌─────────────────────────────────────────────────────────────────┐
│                     UNTRUSTED ZONE                              │
│  (External clients, agent code, user input)                     │
└───────────────────────────┬─────────────────────────────────────┘
                            │ mTLS + NHI authentication
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     API GATEWAY                                 │
│  (Rate limiting, request validation, AuthZ check)               │
└───────────────────────────┬─────────────────────────────────────┘
                            │ Authenticated context
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     ENABLEMENT LAYER                            │
│  (Metering, Oversight, Runtime, Messaging)                      │
└───────────────────────────┬─────────────────────────────────────┘
                            │ Internal mTLS
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                     PLATFORM/SECURITY LAYERS                    │
│  (AuthZ, Memory, Storage, NHI, Crypto, Audit)                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Threat Model

### 2.1 Threat Actors

| Actor | Capabilities | Motivation |
|-------|--------------|------------|
| **Malicious agent** | Code execution in sandbox | Data exfiltration, resource abuse |
| **Compromised credential** | Valid NHI token | Unauthorized access |
| **Network attacker** | Traffic interception | Eavesdropping, tampering |
| **Insider threat** | System access | Data theft, sabotage |
| **State actor** | Advanced capabilities | Long-term access, cryptographic attacks |

### 2.2 Per-Product Threats

#### Metering Threats

| Threat | Impact | Mitigation |
|--------|--------|------------|
| Event forgery | Billing manipulation | Signed events, audit trail |
| Quota bypass | Resource exhaustion | Inline enforcement, reservation |
| Replay attacks | Duplicate billing | Idempotency keys, timestamp validation |
| Attribution spoofing | Blame shifting | NHI verification, delegation chain |

#### Oversight Threats

| Threat | Impact | Mitigation |
|--------|--------|------------|
| Approval forgery | Unauthorized actions | Cryptographic signatures |
| Notification spoofing | Fake approvals | Channel verification, signed responses |
| Timeout manipulation | Forced auto-approve | Server-side timeout enforcement |
| Context tampering | Misleading approvers | Signed request context |

#### Runtime Threats

| Threat | Impact | Mitigation |
|--------|--------|------------|
| Sandbox escape | Host compromise | gVisor/Kata isolation, syscall filtering |
| Identity spoofing | Unauthorized access | NHI binding, attestation |
| Network exfiltration | Data leakage | AuthZ-gated egress |
| Supply chain attack | Malicious code | Image signing, attestation |

#### Messaging Threats

| Threat | Impact | Mitigation |
|--------|--------|------------|
| Eavesdropping | Confidentiality breach | E2E encryption (AES-256-GCM) |
| Message forgery | Impersonation | Hybrid signatures (Ed25519 + ML-DSA) |
| Replay attacks | Duplicate processing | Message IDs, timestamps |
| Unauthorized delivery | Spam, abuse | AuthZ-gated delivery |

---

## 3. Cryptographic Design

### 3.1 Algorithm Selection

| Use Case | Algorithm | Key Size | Notes |
|----------|-----------|----------|-------|
| **Symmetric encryption** | AES-256-GCM | 256-bit | AEAD, NIST approved |
| **Key encapsulation** | ML-KEM-768 | Level 3 | Post-quantum, NIST FIPS 203 |
| **Digital signatures (classical)** | Ed25519 | 256-bit | Fast, compact |
| **Digital signatures (PQ)** | ML-DSA-65 | Level 3 | Post-quantum, NIST FIPS 204 |
| **Hashing** | SHA-256 / BLAKE3 | 256-bit | Configurable via crypto-agility |
| **Key derivation** | HKDF-SHA256 | — | RFC 5869 |

### 3.2 Hybrid Signatures

All signed operations use hybrid signatures for PQ transition:

```rust
pub struct HybridSignature {
    pub ed25519: [u8; 64],      // Classical signature
    pub ml_dsa: Vec<u8>,         // Post-quantum signature (~2.4KB)
}

impl HybridSignature {
    pub fn verify(&self, message: &[u8], public_keys: &HybridPublicKey) -> Result<(), Error> {
        // Both must verify
        verify_ed25519(&self.ed25519, message, &public_keys.ed25519)?;
        verify_ml_dsa(&self.ml_dsa, message, &public_keys.ml_dsa)?;
        Ok(())
    }
}
```

### 3.3 Key Management

| Key Type | Storage | Rotation | Access |
|----------|---------|----------|--------|
| Agent signing keys | NHI HSM | On-demand | NHI delegation |
| Agent encryption keys | NHI HSM | On-demand | NHI delegation |
| Service TLS keys | Kubernetes Secrets | 90 days | Auto-rotation |
| Attestation keys | TPM/HSM | Yearly | Platform-specific |

### 3.4 Crypto-Agility

```rust
// Algorithm selection via configuration, not hardcoding
pub struct CryptoConfig {
    pub symmetric_algorithm: SymmetricAlgorithm,
    pub kem_algorithm: KemAlgorithm,
    pub signature_algorithm: SignatureAlgorithm,
    pub hash_algorithm: HashAlgorithm,
}

pub enum SymmetricAlgorithm {
    Aes256Gcm,
    ChaCha20Poly1305,
}

pub enum KemAlgorithm {
    MlKem768,
    MlKem1024,
    X25519,  // Fallback only
}

pub enum SignatureAlgorithm {
    Ed25519,
    MlDsa65,
    Hybrid { classical: Box<Self>, pq: Box<Self> },
}
```

---

## 4. Authentication & Authorization

### 4.1 Authentication Flow

```
┌──────────┐     ┌─────────────┐     ┌─────────────┐     ┌──────────────┐
│  Agent   │────►│  API GW     │────►│  NHI        │────►│  Enablement  │
└──────────┘     └─────────────┘     └─────────────┘     └──────────────┘
     │                 │                   │                    │
     │ 1. Present NHI token               │                    │
     │────────────────►│                  │                    │
     │                 │ 2. Validate token│                    │
     │                 │─────────────────►│                    │
     │                 │                  │ 3. Return identity │
     │                 │◄─────────────────│                    │
     │                 │ 4. Attach context│                    │
     │                 │─────────────────────────────────────►│
     │                 │                                      │
```

### 4.2 Authorization Integration

Every Enablement operation checks Authorization:

```rust
async fn authorized_operation<T>(
    authz: &AuthzClient,
    context: &RequestContext,
    action: &str,
    resource: &str,
) -> Result<T, Error> {
    let decision = authz.check(CheckRequest {
        agent: context.agent_nhi.clone(),
        delegation_chain: context.delegation_chain.clone(),
        action: action.to_string(),
        resource: resource.to_string(),
    }).await?;

    match decision {
        Decision::Allow => Ok(()),
        Decision::Deny { reason } => Err(Error::PermissionDenied(reason)),
        Decision::RequiresOversight { policy_id } => {
            Err(Error::OversightRequired(policy_id))
        }
        Decision::QuotaExceeded { .. } => {
            Err(Error::QuotaExceeded)
        }
    }
}
```

### 4.3 Service-to-Service Auth

Internal services use mTLS with SPIFFE IDs:

```
SPIFFE ID: spiffe://creto.io/enablement/metering
SPIFFE ID: spiffe://creto.io/enablement/oversight
SPIFFE ID: spiffe://creto.io/enablement/runtime
SPIFFE ID: spiffe://creto.io/enablement/messaging
```

---

## 5. Secure Communication

### 5.1 Transport Security

| Channel | Protocol | Cipher Suites |
|---------|----------|---------------|
| External API | TLS 1.3 | AES-256-GCM-SHA384, ChaCha20-Poly1305 |
| Internal gRPC | mTLS 1.3 | Same as above |
| Sandbox egress | TLS 1.3 | Same as above |
| Message payload | E2E (AES-256-GCM + ML-KEM) | Application layer |

### 5.2 Certificate Management

```yaml
# Cert-manager configuration
apiVersion: cert-manager.io/v1
kind: Certificate
metadata:
  name: creto-metering
spec:
  secretName: creto-metering-tls
  duration: 720h      # 30 days
  renewBefore: 360h   # Renew at 15 days
  issuerRef:
    name: creto-ca
    kind: ClusterIssuer
  dnsNames:
    - metering.creto-enablement.svc
    - metering.creto.io
```

---

## 6. Audit & Non-Repudiation

### 6.1 Audit Events

| Event Type | Data Captured | Retention |
|------------|---------------|-----------|
| `metering.event_ingested` | Event ID, agent NHI, type, timestamp | 7 years |
| `metering.quota_checked` | Agent NHI, action, decision | 1 year |
| `oversight.request_created` | Request ID, action, policy | 7 years |
| `oversight.response_submitted` | Request ID, decision, signature | 7 years |
| `runtime.sandbox_spawned` | Sandbox ID, agent NHI, attestation | 7 years |
| `runtime.egress_checked` | Sandbox ID, destination, decision | 1 year |
| `messaging.message_sent` | Message ID, sender, recipient (metadata only) | 7 years |

### 6.2 Non-Repudiation

```rust
/// Signed audit record for non-repudiation
pub struct SignedAuditRecord {
    pub record: AuditRecord,
    pub signature: HybridSignature,
    pub merkle_proof: MerkleProof,  // Anchored to immutable log
}

pub struct AuditRecord {
    pub id: AuditId,
    pub timestamp: Timestamp,
    pub who: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,
    pub what: String,
    pub resource: String,
    pub outcome: Outcome,
    pub metadata: HashMap<String, String>,
}
```

---

## 7. Security Controls

### 7.1 Input Validation

```rust
/// All API inputs validated before processing
pub trait Validated {
    fn validate(&self) -> Result<(), ValidationError>;
}

impl Validated for BillableEvent {
    fn validate(&self) -> Result<(), ValidationError> {
        // Idempotency key format
        ensure!(self.idempotency_key.len() <= 256, "idempotency_key too long");
        ensure!(!self.idempotency_key.is_empty(), "idempotency_key required");

        // Timestamp bounds
        let now = Timestamp::now();
        let max_past = now - Duration::hours(1);
        let max_future = now + Duration::minutes(5);
        ensure!(self.timestamp >= max_past, "timestamp too old");
        ensure!(self.timestamp <= max_future, "timestamp in future");

        // Event type format
        ensure!(is_valid_event_type(&self.event_type), "invalid event_type");

        // Properties size
        ensure!(self.properties.len() <= 100, "too many properties");

        Ok(())
    }
}
```

### 7.2 Rate Limiting

```rust
/// Per-agent rate limiting with token bucket
pub struct RateLimiter {
    limits: HashMap<RateLimitKey, TokenBucket>,
}

impl RateLimiter {
    pub fn check(&self, agent: &AgentIdentity, operation: &str) -> Result<(), RateLimitError> {
        let key = RateLimitKey::new(agent, operation);
        let bucket = self.limits.get_or_create(&key);

        if bucket.try_consume(1) {
            Ok(())
        } else {
            Err(RateLimitError {
                retry_after: bucket.next_available(),
                limit: bucket.limit(),
                remaining: 0,
            })
        }
    }
}
```

### 7.3 Secrets Management

```rust
/// Secrets never logged, displayed, or serialized
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Secret<T: Zeroize> {
    value: T,
}

impl<T: Zeroize> Debug for Secret<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Secret([REDACTED])")
    }
}

impl<T: Zeroize + Serialize> Serialize for Secret<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("[REDACTED]")
    }
}
```

### 7.4 Sandbox Isolation

```rust
/// Sandbox security configuration
pub struct SandboxSecurityConfig {
    // Syscall filtering
    pub syscall_filter: SyscallFilter,

    // Capabilities (empty by default)
    pub capabilities: Vec<Capability>,

    // Filesystem
    pub read_only_root: bool,
    pub no_new_privs: bool,

    // Namespaces
    pub user_namespace: bool,
    pub network_namespace: bool,
    pub pid_namespace: bool,

    // Resource limits
    pub rlimits: ResourceLimits,
}

impl Default for SandboxSecurityConfig {
    fn default() -> Self {
        Self {
            syscall_filter: SyscallFilter::default_allow_list(),
            capabilities: vec![],  // No capabilities by default
            read_only_root: true,
            no_new_privs: true,
            user_namespace: true,
            network_namespace: true,
            pid_namespace: true,
            rlimits: ResourceLimits::default(),
        }
    }
}
```

---

## 8. Vulnerability Management

### 8.1 Dependency Scanning

```bash
# CI pipeline checks
cargo audit                     # Known vulnerabilities
cargo deny check                # License and security policies
cargo outdated                  # Outdated dependencies
```

### 8.2 Security Testing

| Type | Frequency | Tools |
|------|-----------|-------|
| SAST | Every PR | cargo clippy, rust-analyzer |
| Dependency scan | Daily | cargo audit, Dependabot |
| Fuzz testing | Weekly | cargo-fuzz, libFuzzer |
| Penetration testing | Quarterly | External vendor |

### 8.3 Incident Response

1. **Detection**: Anomaly detection on audit logs
2. **Containment**: Revoke compromised credentials
3. **Investigation**: Audit trail analysis
4. **Recovery**: Restore from known-good state
5. **Post-mortem**: Update threat model

---

## 9. Compliance Controls

| Control | Metering | Oversight | Runtime | Messaging |
|---------|----------|-----------|---------|-----------|
| **Encryption at rest** | ✓ | ✓ | ✓ | ✓ |
| **Encryption in transit** | ✓ | ✓ | ✓ | ✓ |
| **Access logging** | ✓ | ✓ | ✓ | ✓ |
| **MFA for approvers** | — | ✓ | — | — |
| **Key rotation** | ✓ | ✓ | ✓ | ✓ |
| **Data retention** | ✓ | ✓ | ✓ | ✓ |

---

## 10. Decisions

| Decision | Rationale |
|----------|-----------|
| Hybrid signatures | PQ transition without breaking compatibility |
| ML-KEM-768 (not 1024) | Balance of security and performance |
| mTLS for internal | Zero trust, defense in depth |
| Signed audit records | Legal admissibility, non-repudiation |

---

## 11. Open Questions

1. Should we support customer-managed encryption keys (CMEK)?
2. What's the HSM strategy for attestation keys?
3. Should messaging support ephemeral keys per conversation?

---

## 12. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
