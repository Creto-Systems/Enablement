---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# Messaging Security Design

## Table of Contents

1. [Cryptographic Protocol](#cryptographic-protocol)
2. [Key Management](#key-management)
3. [Threat Model](#threat-model)
4. [Attack Mitigation](#attack-mitigation)
5. [Compliance](#compliance)
6. [Security Testing](#security-testing)
7. [Incident Response](#incident-response)

---

## Cryptographic Protocol

### Overview

The messaging system uses a **hybrid post-quantum cryptographic protocol** combining classical and quantum-resistant algorithms for defense-in-depth.

```
┌─────────────────────────────────────────────────────────────┐
│              CRYPTOGRAPHIC PROTOCOL STACK                   │
├─────────────────────────────────────────────────────────────┤
│ Layer 4: Hybrid Signatures (Ed25519 + ML-DSA-65)           │
│   Purpose: Authenticity, integrity, non-repudiation        │
│   Security: Classical + PQC (NIST Level 3)                 │
├─────────────────────────────────────────────────────────────┤
│ Layer 3: Key Encapsulation (ML-KEM-768)                    │
│   Purpose: Wrap AES-256 keys for recipient                 │
│   Security: PQC (NIST Level 3, ~192-bit classical)         │
├─────────────────────────────────────────────────────────────┤
│ Layer 2: Authenticated Encryption (AES-256-GCM)            │
│   Purpose: Confidentiality + integrity of payload          │
│   Security: 256-bit symmetric, quantum-resistant           │
├─────────────────────────────────────────────────────────────┤
│ Layer 1: Transport Security (TLS 1.3)                      │
│   Purpose: Encrypted channel to messaging service          │
│   Security: X25519 ECDH + AES-256-GCM                      │
└─────────────────────────────────────────────────────────────┘
```

### Encryption Protocol

**Step-by-step encryption process:**

```
┌─────────────────────────────────────────────────────────────┐
│                    SENDER (Agent A)                         │
└─────────────────────────────────────────────────────────────┘

1. Generate ephemeral AES-256 key
   aes_key = random(32 bytes)

2. Encrypt plaintext with AES-256-GCM
   nonce = random(12 bytes)
   (ciphertext, auth_tag) = AES256GCM.encrypt(
       key=aes_key,
       nonce=nonce,
       plaintext=message_payload,
       aad=canonical_envelope_metadata
   )

3. Fetch recipient's ML-KEM-768 public key from NHI
   pubkey_b = NHI.get_public_key(agent_b, KeyType::MlKem768)

4. Encapsulate AES key using ML-KEM-768
   (wrapped_key, shared_secret) = ML_KEM_768.encapsulate(pubkey_b)
   # wrapped_key sent to recipient, shared_secret discarded

5. Build envelope
   envelope = MessageEnvelope {
       sender: agent_a,
       recipient: agent_b,
       ciphertext: ciphertext,
       nonce: nonce,
       auth_tag: auth_tag,
       wrapped_key: wrapped_key,
       key_id: pubkey_b.id,
       ...
   }

6. Compute canonical representation (for signing)
   canonical = concat(
       envelope.version,
       envelope.sender,
       envelope.recipient,
       envelope.message_id,
       envelope.timestamp,
       envelope.ciphertext,
       envelope.nonce,
       envelope.wrapped_key
   )

7. Sign with Ed25519 (classical)
   sig_ed25519 = NHI.sign_ed25519(agent_a, canonical)

8. Sign with ML-DSA-65 (post-quantum)
   sig_ml_dsa = NHI.sign_ml_dsa_65(agent_a, canonical)

9. Attach signatures to envelope
   envelope.signature_ed25519 = sig_ed25519
   envelope.signature_ml_dsa = sig_ml_dsa

10. Send envelope to messaging service
    MessagingService.SendMessage(envelope)
```

### Decryption Protocol

```
┌─────────────────────────────────────────────────────────────┐
│                   RECIPIENT (Agent B)                       │
└─────────────────────────────────────────────────────────────┘

1. Receive envelope from messaging service
   envelope = MessagingService.ReceiveMessages()

2. Compute canonical representation
   canonical = concat(
       envelope.version,
       envelope.sender,
       envelope.recipient,
       envelope.message_id,
       envelope.timestamp,
       envelope.ciphertext,
       envelope.nonce,
       envelope.wrapped_key
   )

3. Fetch sender's Ed25519 public key from NHI
   pubkey_ed25519 = NHI.get_public_key(agent_a, KeyType::Ed25519)

4. Verify Ed25519 signature
   is_valid = Ed25519.verify(
       public_key=pubkey_ed25519,
       message=canonical,
       signature=envelope.signature_ed25519
   )
   if !is_valid:
       REJECT envelope (potential forgery)
       AUDIT_LOG("signature_verification_failed", envelope)
       RETURN error

5. Fetch sender's ML-DSA-65 public key from NHI
   pubkey_ml_dsa = NHI.get_public_key(agent_a, KeyType::MlDsa65)

6. Verify ML-DSA-65 signature
   is_valid = ML_DSA_65.verify(
       public_key=pubkey_ml_dsa,
       message=canonical,
       signature=envelope.signature_ml_dsa
   )
   if !is_valid:
       REJECT envelope (potential forgery)
       AUDIT_LOG("signature_verification_failed", envelope)
       RETURN error

7. Decapsulate AES key using ML-KEM-768
   aes_key = NHI.ml_kem_decapsulate(
       agent_b,
       wrapped_key=envelope.wrapped_key,
       key_id=envelope.key_id
   )

8. Decrypt ciphertext with AES-256-GCM
   plaintext = AES256GCM.decrypt(
       key=aes_key,
       nonce=envelope.nonce,
       ciphertext=envelope.ciphertext,
       auth_tag=envelope.auth_tag,
       aad=canonical_envelope_metadata
   )
   if decryption_failed:
       REJECT envelope (tampered or wrong key)
       AUDIT_LOG("decryption_failed", envelope)
       RETURN error

9. Audit log successful receipt
   AUDIT_LOG("message_received", {
       sender: envelope.sender,
       recipient: envelope.recipient,
       message_id: envelope.message_id,
       timestamp: envelope.timestamp
   })

10. Return plaintext to application
    RETURN plaintext
```

### Cryptographic Primitives Specification

#### ML-KEM-768 (FIPS 203)

**Purpose:** Post-quantum key encapsulation mechanism

**Parameters:**
- **Security level:** NIST Level 3 (~192-bit classical security)
- **Public key size:** 1184 bytes
- **Ciphertext (wrapped key) size:** 1088 bytes
- **Shared secret size:** 32 bytes
- **Encapsulation time:** ~100µs
- **Decapsulation time:** ~100µs

**Algorithm:**
```
Encapsulate(pk):
    (c, K) = ML-KEM-768.Encaps(pk)
    return (c, K)  # c = ciphertext (wrapped key), K = shared secret

Decapsulate(sk, c):
    K = ML-KEM-768.Decaps(sk, c)
    return K
```

**Security properties:**
- **IND-CCA2 secure:** Indistinguishable under adaptive chosen-ciphertext attack
- **Quantum-resistant:** Based on Module Learning With Errors (MLWE)
- **Standardized:** NIST FIPS 203 (final)

#### AES-256-GCM

**Purpose:** Authenticated encryption of message payload

**Parameters:**
- **Key size:** 256 bits (32 bytes)
- **Nonce size:** 96 bits (12 bytes)
- **Authentication tag size:** 128 bits (16 bytes)
- **Maximum plaintext size:** 64 GB - 1 byte
- **Encryption time:** ~500ns for 1KB payload (hardware-accelerated)

**Algorithm:**
```
Encrypt(key, nonce, plaintext, aad):
    ciphertext, tag = AES256GCM.encrypt(
        key=key,
        nonce=nonce,
        plaintext=plaintext,
        associated_data=aad
    )
    return (ciphertext, tag)

Decrypt(key, nonce, ciphertext, tag, aad):
    plaintext = AES256GCM.decrypt(
        key=key,
        nonce=nonce,
        ciphertext=ciphertext,
        tag=tag,
        associated_data=aad
    )
    if authentication_failed:
        raise DecryptionError
    return plaintext
```

**Security properties:**
- **Authenticated encryption:** Confidentiality + integrity
- **Nonce uniqueness critical:** Never reuse nonce with same key
- **Quantum-resistant:** Symmetric encryption (256-bit key)

#### Ed25519 (Classical Signature)

**Purpose:** Fast, compact classical signatures

**Parameters:**
- **Public key size:** 32 bytes
- **Private key size:** 32 bytes (seed) + 32 bytes (scalar)
- **Signature size:** 64 bytes
- **Signing time:** ~40µs
- **Verification time:** ~100µs

**Algorithm:**
```
Sign(sk, message):
    signature = Ed25519.sign(sk, message)
    return signature  # 64 bytes

Verify(pk, message, signature):
    is_valid = Ed25519.verify(pk, message, signature)
    return is_valid
```

**Security properties:**
- **EUF-CMA secure:** Existentially unforgeable under chosen-message attack
- **Deterministic:** Same message + key = same signature
- **Not quantum-resistant:** Shor's algorithm breaks ECDLP

#### ML-DSA-65 (FIPS 204)

**Purpose:** Post-quantum digital signatures

**Parameters:**
- **Public key size:** ~1952 bytes
- **Private key size:** ~4032 bytes
- **Signature size:** ~3293 bytes
- **Signing time:** ~2ms
- **Verification time:** ~1ms

**Algorithm:**
```
Sign(sk, message):
    signature = ML_DSA_65.sign(sk, message)
    return signature

Verify(pk, message, signature):
    is_valid = ML_DSA_65.verify(pk, message, signature)
    return is_valid
```

**Security properties:**
- **EUF-CMA secure:** Existentially unforgeable under chosen-message attack
- **Quantum-resistant:** Based on Module Learning With Errors (MLWE)
- **Standardized:** NIST FIPS 204 (final)
- **Security level:** NIST Level 3 (~192-bit classical security)

---

## Key Management

### Key Lifecycle

```
┌──────────────────────────────────────────────────────────┐
│                    KEY LIFECYCLE                         │
└──────────────────────────────────────────────────────────┘

1. GENERATION
   ↓
   Agent registers with NHI service
   NHI generates key pairs:
     - ML-KEM-768 (encryption)
     - Ed25519 (signing, classical)
     - ML-DSA-65 (signing, post-quantum)
   Private keys stored in HSM
   Public keys published to key bundle store

2. DISTRIBUTION
   ↓
   Public keys uploaded to messaging service
   Cached in Redis (1 hour TTL)
   Fetched by senders on-demand

3. ROTATION
   ↓
   Scheduled rotation (daily/weekly) or manual trigger
   New key pair generated
   Old private key moved to grace period (7 days)
   New public key published
   Cache invalidated

4. GRACE PERIOD
   ↓
   Old private key retained for 7 days
   Inflight messages encrypted with old key can still be decrypted
   After 7 days, old key is deleted

5. REVOCATION
   ↓
   Triggered on compromise detection
   Key immediately marked as revoked
   No grace period (emergency rotation)
   All sessions using revoked key are invalidated
   Audit log created

6. EXPIRATION
   ↓
   Keys have expiration timestamp (1 year default)
   Automatic rotation before expiration
   Alert if rotation fails
```

### Key Storage

**Private keys (NHI HSM):**
```
┌────────────────────────────────────────────┐
│      Hardware Security Module (HSM)       │
│  ┌──────────────────────────────────────┐ │
│  │ agent-alice-01                       │ │
│  │   ├─ ML-KEM-768 private key (sealed) │ │
│  │   ├─ Ed25519 private key (sealed)    │ │
│  │   └─ ML-DSA-65 private key (sealed)  │ │
│  └──────────────────────────────────────┘ │
│  ┌──────────────────────────────────────┐ │
│  │ agent-bob-02                         │ │
│  │   ├─ ML-KEM-768 private key (sealed) │ │
│  │   ├─ Ed25519 private key (sealed)    │ │
│  │   └─ ML-DSA-65 private key (sealed)  │ │
│  └──────────────────────────────────────┘ │
└────────────────────────────────────────────┘
```

**Public keys (Redis cache):**
```redis
# Active key bundle
HSET key:agent-alice-01:active
    ml_kem_768_pubkey   <1184 bytes>
    ed25519_pubkey      <32 bytes>
    ml_dsa_65_pubkey    <1952 bytes>
    key_id              "7c9e6679-7425-40de-944b-e07fc1f90ae7"
    created_at          "2025-12-25T10:00:00Z"
    expires_at          "2026-12-25T10:00:00Z"
    status              "active"

EXPIRE key:agent-alice-01:active 3600  # 1 hour TTL
```

### Key Rotation Strategy

**Scheduled rotation:**

```rust
pub struct RotationPolicy {
    pub interval: RotationInterval,
    pub grace_period: Duration,
    pub auto_rotate: bool,
}

pub enum RotationInterval {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

impl RotationScheduler {
    pub async fn rotate_keys(&self, agent_nhi: &str) -> Result<()> {
        // 1. Generate new key pairs via NHI
        let new_ml_kem_key = self.nhi_client
            .generate_ml_kem_keypair(agent_nhi)
            .await?;
        let new_ed25519_key = self.nhi_client
            .generate_ed25519_keypair(agent_nhi)
            .await?;
        let new_ml_dsa_key = self.nhi_client
            .generate_ml_dsa_keypair(agent_nhi)
            .await?;

        // 2. Fetch current keys
        let current_keys = self.key_store.get_active_keys(agent_nhi).await?;

        // 3. Move current keys to grace period
        self.key_store.add_grace_period_key(
            agent_nhi,
            current_keys,
            Utc::now() + Duration::days(7)
        ).await?;

        // 4. Publish new keys
        self.key_store.set_active_keys(
            agent_nhi,
            KeyBundle {
                ml_kem_pubkey: new_ml_kem_key.public_key,
                ed25519_pubkey: new_ed25519_key.public_key,
                ml_dsa_pubkey: new_ml_dsa_key.public_key,
                key_id: new_ml_kem_key.id,
                created_at: Utc::now(),
                expires_at: Utc::now() + Duration::days(365),
                status: KeyStatus::Active,
            }
        ).await?;

        // 5. Invalidate cache
        self.redis_client.del(format!("key:{}:active", agent_nhi)).await?;

        // 6. Audit log
        self.audit_client.log(AuditEvent {
            event_type: "key_rotated",
            actor: agent_nhi.to_string(),
            old_key_id: current_keys.key_id,
            new_key_id: new_ml_kem_key.id,
            ..Default::default()
        }).await?;

        Ok(())
    }
}
```

**Emergency rotation (key compromise):**

```rust
pub async fn emergency_rotate(&self, agent_nhi: &str, reason: &str) -> Result<()> {
    // 1. Generate new keys
    let new_keys = self.generate_all_keys(agent_nhi).await?;

    // 2. Revoke old keys (no grace period)
    let old_keys = self.key_store.get_active_keys(agent_nhi).await?;
    self.key_store.revoke_key(agent_nhi, &old_keys.key_id, reason).await?;

    // 3. Publish new keys
    self.key_store.set_active_keys(agent_nhi, new_keys).await?;

    // 4. Invalidate all sessions using old keys
    self.session_manager.invalidate_sessions_with_key(&old_keys.key_id).await?;

    // 5. Alert security team
    self.alert_client.send(Alert {
        severity: Severity::Critical,
        title: format!("Emergency key rotation: {}", agent_nhi),
        description: reason.to_string(),
        ..Default::default()
    }).await?;

    // 6. Audit log
    self.audit_client.log(AuditEvent {
        event_type: "emergency_key_rotation",
        actor: agent_nhi.to_string(),
        reason: reason.to_string(),
        old_key_id: old_keys.key_id,
        new_key_id: new_keys.key_id,
        ..Default::default()
    }).await?;

    Ok(())
}
```

---

## Threat Model

### Threat Actors

| Actor | Motivation | Capabilities |
|-------|------------|--------------|
| **External Attacker** | Data theft, disruption | Network access, malware |
| **Malicious Agent** | Impersonation, forgery | Valid NHI, send messages |
| **Compromised Agent** | Data exfiltration | Private keys, message access |
| **Insider Threat** | Data theft, sabotage | Platform access, credentials |
| **Quantum Computer** | Break classical crypto | Shor's algorithm (future) |
| **Nation-State** | Espionage, APT | Advanced persistent threat |

### Threat Scenarios

#### 1. Eavesdropping (Passive Attack)

**Threat:** Attacker intercepts network traffic to read message content.

**Mitigations:**
- **E2E encryption (AES-256-GCM):** Ciphertext unreadable without AES key
- **ML-KEM-768 key encapsulation:** AES key wrapped, requires private key to unwrap
- **TLS 1.3 transport:** Encrypted channel to messaging service
- **No plaintext storage:** Server never sees plaintext

**Attack vector blocked:** ✅

#### 2. Man-in-the-Middle (Active Attack)

**Threat:** Attacker intercepts and modifies messages in transit.

**Mitigations:**
- **Hybrid signatures (Ed25519 + ML-DSA-65):** Message tampering detected
- **AES-GCM authentication tag:** Ciphertext integrity verified
- **TLS 1.3 certificate pinning:** MITM on transport layer prevented
- **NHI public key infrastructure:** Sender identity cryptographically verified

**Attack vector blocked:** ✅

#### 3. Replay Attack

**Threat:** Attacker captures valid message and replays it later.

**Mitigations:**
- **Timestamp validation:** Messages older than 5 minutes rejected
- **Nonce uniqueness:** AES-GCM nonce never reused, replay detected
- **Message ID deduplication:** Duplicate message IDs rejected
- **Audit trail:** All deliveries logged, replay attempts detected

**Attack vector blocked:** ✅

#### 4. Forgery Attack

**Threat:** Attacker crafts message pretending to be another agent.

**Mitigations:**
- **Hybrid signatures:** Ed25519 + ML-DSA-65 verify sender identity
- **NHI key binding:** Private keys in HSM, only NHI can sign
- **Signature verification required:** Both signatures must validate
- **Audit logging:** Failed verifications logged as potential forgery

**Attack vector blocked:** ✅

#### 5. Key Compromise

**Threat:** Attacker obtains agent's private keys.

**Mitigations:**
- **Forward secrecy:** Key rotation limits compromise window
- **Grace period keys:** Old messages still decryptable during transition
- **Emergency rotation:** Compromised keys immediately revoked
- **HSM storage:** Private keys never exposed to application
- **Audit trail:** Key rotation events logged

**Impact limited:** ⚠️ (only affects messages during compromise window)

#### 6. Quantum Attack (Future)

**Threat:** Quantum computer breaks classical crypto (Ed25519, X25519).

**Mitigations:**
- **ML-KEM-768:** Post-quantum key encapsulation
- **ML-DSA-65:** Post-quantum signatures
- **Hybrid mode:** Classical + PQC (both must break for full compromise)
- **Crypto-agility:** Algorithm swappable via protocol version

**Attack vector blocked (future-proof):** ✅

#### 7. Denial of Service

**Threat:** Attacker floods messaging service with requests.

**Mitigations:**
- **Rate limiting:** Per-agent quotas enforced
- **AuthZ integration:** Unauthorized messages denied before encryption
- **Priority queues:** Critical messages bypass normal queue
- **Auto-scaling:** Horizontal scaling handles load spikes

**Impact limited:** ⚠️ (service degradation possible, no data compromise)

#### 8. Unauthorized Delivery

**Threat:** Agent sends message to unauthorized recipient.

**Mitigations:**
- **AuthZ policy checks:** creto-authz validates sender → recipient
- **Inline enforcement:** 168ns latency, no bypass
- **Audit logging:** All denials logged
- **Fail-closed:** AuthZ failure = deny delivery

**Attack vector blocked:** ✅

---

## Attack Mitigation

### Signature Verification Failures

**Detection:**
```rust
if !verify_ed25519(envelope) || !verify_ml_dsa(envelope) {
    audit_client.log(AuditEvent {
        event_type: "signature_verification_failed",
        severity: Severity::Critical,
        actor: envelope.sender_nhi,
        target: envelope.recipient_nhi,
        message_id: envelope.message_id,
        details: json!({
            "ed25519_valid": verify_ed25519(envelope),
            "ml_dsa_valid": verify_ml_dsa(envelope),
            "canonical_hash": sha256(&canonical),
        }),
        ..Default::default()
    }).await?;

    // Reject message
    return Err(MessagingError::SignatureVerificationFailed);
}
```

**Response:**
1. Reject message immediately
2. Log to audit with full context
3. Alert security team if threshold exceeded (>10 failures/hour)
4. Rate limit sender (potential attack)
5. Investigate potential key compromise

### Decryption Failures

**Detection:**
```rust
match aes_gcm.decrypt(&envelope.nonce, &envelope.ciphertext, &envelope.auth_tag) {
    Ok(plaintext) => Ok(plaintext),
    Err(DecryptionError::AuthenticationFailed) => {
        // Ciphertext tampered or wrong key
        audit_client.log(AuditEvent {
            event_type: "decryption_failed",
            severity: Severity::High,
            message_id: envelope.message_id,
            reason: "authentication_tag_mismatch",
            ..Default::default()
        }).await?;

        Err(MessagingError::DecryptionFailed)
    }
}
```

**Response:**
1. Reject message
2. Log to audit
3. Check if recipient key was rotated (re-fetch key, retry)
4. If persistent failure, alert operations team

### Rate Limiting Enforcement

**Per-agent quotas:**
```rust
pub struct RateLimiter {
    redis: Arc<RedisClient>,
    quotas: HashMap<QuotaTier, RateLimit>,
}

pub struct RateLimit {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_size: u32,
}

impl RateLimiter {
    pub async fn check_limit(&self, agent_nhi: &str) -> Result<RateLimitDecision> {
        let tier = self.get_agent_tier(agent_nhi).await?;
        let quota = &self.quotas[&tier];

        // Token bucket algorithm
        let key_minute = format!("ratelimit:{}:minute", agent_nhi);
        let key_hour = format!("ratelimit:{}:hour", agent_nhi);

        let count_minute: u32 = self.redis.incr(&key_minute).await?;
        if count_minute == 1 {
            self.redis.expire(&key_minute, 60).await?;
        }

        let count_hour: u32 = self.redis.incr(&key_hour).await?;
        if count_hour == 1 {
            self.redis.expire(&key_hour, 3600).await?;
        }

        if count_minute > quota.requests_per_minute {
            return Ok(RateLimitDecision::Exceeded {
                retry_after: 60 - (Utc::now().timestamp() % 60),
            });
        }

        if count_hour > quota.requests_per_hour {
            return Ok(RateLimitDecision::Exceeded {
                retry_after: 3600 - (Utc::now().timestamp() % 3600),
            });
        }

        Ok(RateLimitDecision::Allowed)
    }
}
```

### Timestamp Validation

**Reject stale messages:**
```rust
const MAX_MESSAGE_AGE_SECONDS: i64 = 300;  // 5 minutes

pub fn validate_timestamp(envelope: &MessageEnvelope) -> Result<()> {
    let now = Utc::now().timestamp_millis();
    let age_ms = now - envelope.created_at.timestamp_millis();

    if age_ms > MAX_MESSAGE_AGE_SECONDS * 1000 {
        return Err(MessagingError::MessageTooOld {
            created_at: envelope.created_at,
            age_seconds: age_ms / 1000,
        });
    }

    if age_ms < -60000 {  // 1 minute in the future (clock skew tolerance)
        return Err(MessagingError::MessageFromFuture {
            created_at: envelope.created_at,
        });
    }

    Ok(())
}
```

---

## Compliance

### FIPS 140-2 Cryptographic Modules

**Requirements:**
- Use FIPS-validated cryptographic libraries
- Hardware RNG for key generation
- Secure key storage (HSM Level 2+)
- Zeroization of plaintext keys

**Implementation:**
```rust
// Use FIPS-validated OpenSSL module
extern crate openssl_fips;

// Verify FIPS mode enabled
if !openssl_fips::is_fips_mode() {
    panic!("FIPS mode required for production");
}

// Use hardware RNG
use rand::rngs::OsRng;
let mut rng = OsRng;
let aes_key = AesKey::generate(&mut rng);

// Zeroize keys after use
drop(aes_key);  // Zeroize trait automatically clears memory
```

### HIPAA Compliance (164.312(e))

| Requirement | Implementation | Status |
|-------------|----------------|--------|
| **Transmission security** | E2E encryption (AES-256-GCM) | ✅ |
| **Integrity controls** | Hybrid signatures (Ed25519 + ML-DSA) | ✅ |
| **Encryption** | ML-KEM-768 key encapsulation | ✅ |
| **Access controls** | AuthZ policy enforcement | ✅ |
| **Audit controls** | Immutable audit trail | ✅ |

### FedRAMP Controls

| Control | Description | Implementation |
|---------|-------------|----------------|
| **SC-8** | Transmission confidentiality | E2E encryption + TLS 1.3 |
| **SC-8(1)** | Cryptographic protection | FIPS 140-2 modules |
| **SC-13** | Cryptographic key management | NHI HSM storage, rotation |
| **AU-2** | Audit events | All operations logged |
| **AC-3** | Access enforcement | AuthZ inline checks |
| **IA-5(1)** | Password-based authentication | NHI token-based auth |

### GDPR Compliance (Article 32)

| Requirement | Implementation |
|-------------|----------------|
| **Pseudonymization** | Agent NHI (no PII) |
| **Encryption** | E2E encryption, TDE |
| **Confidentiality** | Access controls, AuthZ |
| **Integrity** | Signatures, audit trail |
| **Availability** | 99.99% SLA, multi-region |
| **Resilience** | Auto-recovery, failover |
| **Testing** | Continuous security testing |

---

## Security Testing

### Cryptographic Test Vectors

**ML-KEM-768:**
```rust
#[test]
fn test_ml_kem_768_kat() {
    // NIST Known Answer Test vectors
    let kat_vectors = load_kat_vectors("ml_kem_768_kat.txt");

    for vector in kat_vectors {
        let (pubkey, privkey) = ml_kem_768::keypair_from_seed(&vector.seed);
        assert_eq!(pubkey, vector.expected_pubkey);

        let (ciphertext, shared_secret) = ml_kem_768::encapsulate(&pubkey);
        assert_eq!(ciphertext, vector.expected_ciphertext);

        let decapsulated = ml_kem_768::decapsulate(&privkey, &ciphertext);
        assert_eq!(decapsulated, shared_secret);
    }
}
```

**AES-256-GCM:**
```rust
#[test]
fn test_aes_gcm_vectors() {
    let vectors = load_nist_gcm_vectors("gcmEncryptExtIV256.rsp");

    for vector in vectors {
        let cipher = Aes256Gcm::new(&vector.key);
        let ciphertext = cipher.encrypt(&vector.nonce, vector.plaintext).unwrap();
        assert_eq!(ciphertext, vector.expected_ciphertext);

        let plaintext = cipher.decrypt(&vector.nonce, &ciphertext).unwrap();
        assert_eq!(plaintext, vector.plaintext);
    }
}
```

### Penetration Testing

**Scope:**
1. **Network layer:** MITM attacks, traffic analysis
2. **Application layer:** Signature forgery, replay attacks
3. **AuthZ bypass:** Unauthorized message delivery
4. **Rate limit bypass:** DoS attempts
5. **Key compromise scenarios:** Forward secrecy validation

**Tools:**
- **Burp Suite:** HTTP/gRPC fuzzing
- **Wireshark:** Traffic analysis
- **Metasploit:** Exploit testing
- **Custom tools:** Crypto attack scripts

### Fuzzing

**Protocol buffer fuzzing:**
```rust
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(envelope) = MessageEnvelope::decode(data) {
        // Attempt to process malformed envelope
        let _ = verify_signatures(&envelope);
        let _ = decrypt_payload(&envelope);
    }
});
```

**Crypto fuzzing:**
```rust
fuzz_target!(|data: &[u8]| {
    if data.len() < 100 {
        return;
    }

    let (wrapped_key, nonce, ciphertext) = split_data(data);

    // Attempt decryption with random data
    let _ = ml_kem_decapsulate(&wrapped_key);
    let _ = aes_gcm_decrypt(&nonce, &ciphertext);
});
```

---

## Incident Response

### Key Compromise Procedure

**Phase 1: Detection (T+0)**
1. Compromise detected (anomalous signing activity, leaked key dump)
2. Security team alerted (PagerDuty)
3. Incident severity assessed (P1 if active agent)

**Phase 2: Containment (T+5 minutes)**
1. Trigger emergency key rotation:
   ```bash
   curl -X POST https://messaging.creto.io/v1/keys/emergency-rotate \
     -H "Authorization: Bearer <admin-token>" \
     -d '{"agent_nhi": "agent-alice-01", "reason": "key_compromise_detected"}'
   ```
2. Revoke compromised keys (no grace period)
3. Invalidate all active sessions using compromised key
4. Rate limit agent to prevent abuse

**Phase 3: Investigation (T+15 minutes)**
1. Query audit logs for anomalous activity:
   ```sql
   SELECT * FROM audit_events
   WHERE actor_nhi = 'agent-alice-01'
   AND event_time > NOW() - INTERVAL '24 hours'
   ORDER BY event_time DESC;
   ```
2. Identify messages signed with compromised key
3. Notify affected recipients
4. Preserve evidence for forensics

**Phase 4: Recovery (T+1 hour)**
1. Verify new keys deployed
2. Test message send/receive with new keys
3. Monitor for continued anomalous activity
4. Update incident timeline

**Phase 5: Post-Mortem (T+1 week)**
1. Root cause analysis
2. Identify security gaps
3. Update procedures
4. Implement preventive measures

### Signature Verification Failure Runbook

See `07-runbook.md` for detailed operational procedures.

---

## Summary

### Security Guarantees

1. **Confidentiality:** E2E encryption (AES-256-GCM + ML-KEM-768)
2. **Authenticity:** Hybrid signatures (Ed25519 + ML-DSA-65)
3. **Integrity:** Authenticated encryption (GCM tag) + signatures
4. **Forward secrecy:** Key rotation limits compromise window
5. **Non-repudiation:** Cryptographic signatures provide proof
6. **Quantum resistance:** PQC (ML-KEM-768, ML-DSA-65)
7. **Authorization:** Policy-gated delivery (creto-authz)
8. **Auditability:** Immutable audit trail

### Next Steps

1. **Performance benchmarks** (see 06-testing.md)
2. **Operational runbooks** (see 07-runbook.md)
3. **Penetration testing plan**
4. **FIPS 140-2 validation**
5. **Compliance audit (HIPAA, FedRAMP)**
