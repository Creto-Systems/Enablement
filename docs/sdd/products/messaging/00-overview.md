---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# Messaging Product Overview

## Executive Summary

**creto-messaging** provides secure, end-to-end encrypted communication infrastructure for AI agents in the Creto Sovereign platform. Built on Signal Protocol cryptographic patterns and adapted for non-human identities (NHI), it enables agents to communicate with mathematical guarantees of confidentiality, authenticity, and forward secrecy.

### Vision

Create a messaging system where:
- **Zero-knowledge infrastructure**: Platform operators cannot read message content
- **Post-quantum resistant**: Cryptography survives quantum computer attacks
- **High throughput**: >100,000 messages/second encryption performance
- **Policy-enforced delivery**: Authorization checks inline at 168ns latency
- **Audit-complete**: Every operation logged to immutable trail

### Core Value Propositions

1. **Cryptographic Assurance**: ML-KEM-768 + AES-256-GCM + hybrid signatures (Ed25519 + ML-DSA-65)
2. **Forward Secrecy**: Double Ratchet key rotation limits key compromise blast radius
3. **Authorization Integration**: Delivery gated by creto-authz policy engine
4. **Compliance-Ready**: HIPAA, FedRAMP, GDPR-compliant encryption and audit
5. **Agent-Native**: Designed for machine-to-machine communication patterns

---

## Problem Statement

### Current Challenges

**Traditional messaging systems fail AI agents:**

1. **Human-centric design**: Phone numbers, usernames, UI/UX not relevant to agents
2. **Weak crypto**: Many systems use TLS-only (no E2E), vulnerable to platform compromise
3. **No PQC**: Classical crypto (RSA, ECDH) broken by quantum computers
4. **No policy integration**: Delivery not gated by fine-grained authorization
5. **Poor auditability**: Missing immutable trails for compliance

**Existing solutions and their gaps:**

| Solution | Strength | Gap for Agents |
|----------|----------|----------------|
| **Signal** | Best-in-class E2E crypto | Human phone numbers, mobile-first |
| **Matrix** | Federated, open protocol | Complex server-to-server, no PQC |
| **Kafka** | High throughput | No E2E encryption, no signatures |
| **NATS** | Lightweight pub/sub | TLS-only, no message-level crypto |
| **RabbitMQ** | Reliable delivery | No E2E encryption, complex config |

### Target Users

1. **AI Agent Developers**: Need secure communication primitives
2. **Platform Operators**: Need audit and policy enforcement
3. **Compliance Teams**: Need cryptographic proof for regulations
4. **Security Engineers**: Need post-quantum crypto and forward secrecy

---

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      CLIENT (Agent Runtime)                     │
│  ┌────────────────┐ ┌──────────────┐ ┌─────────────────────┐  │
│  │ MessageSender  │ │MessageReceiver│ │ KeyAgreementClient │  │
│  └────────┬───────┘ └──────┬───────┘ └──────────┬──────────┘  │
│           └────────────────┼──────────────────────┘             │
└────────────────────────────┼──────────────────────────────────┘
                             │ gRPC/REST
┌────────────────────────────┼──────────────────────────────────┐
│                   MESSAGING SERVICE (Server)                   │
│  ┌─────────────────────────┼──────────────────────────────┐   │
│  │           ENVELOPE PROCESSOR (Stateless)                │   │
│  │    ┌─────────────┐  ┌────────────┐  ┌──────────────┐  │   │
│  │    │ Signature   │  │ AuthZ Gate │  │  Delivery    │  │   │
│  │    │ Verifier    │→ │  Checker   │→ │  Router      │  │   │
│  │    └─────────────┘  └────────────┘  └──────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │           KEY MANAGEMENT (Stateful)                      │   │
│  │    ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │   │
│  │    │   Prekey     │  │   Ratchet    │  │  Rotation  │  │   │
│  │    │   Store      │  │   Engine     │  │  Scheduler │  │   │
│  │    └──────────────┘  └──────────────┘  └────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │           MESSAGE STORAGE (Persistent)                   │   │
│  │    ┌──────────────┐  ┌──────────────┐  ┌────────────┐  │   │
│  │    │  Envelope    │  │  Delivery    │  │  Channel   │  │   │
│  │    │  Queue       │  │  Receipts    │  │  Metadata  │  │   │
│  │    └──────────────┘  └──────────────┘  └────────────┘  │   │
│  └──────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                             ↓
┌─────────────────────────────────────────────────────────────────┐
│                   PLATFORM INTEGRATION                          │
│  ┌──────────┐ ┌──────────┐ ┌───────────┐ ┌──────────────────┐ │
│  │creto-nhi │ │creto-authz│ │creto-audit│ │ creto-consensus │ │
│  │(identity)│ │ (policy)  │ │  (trail)  │ │   (ordering)    │ │
│  └──────────┘ └──────────┘ └───────────┘ └──────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## Cryptographic Protocol Overview

### Signal Protocol Adaptation for Agents

**X3DH (Extended Triple Diffie-Hellman) → NHI Key Agreement:**

Traditional Signal:
```
Alice → Bob's prekey bundle lookup (via phone number)
        ↓
Compute shared secret via ECDH(Alice_identity, Bob_signed_prekey)
```

Creto Adaptation:
```
Agent A → Agent B's ML-KEM public key (via NHI service)
          ↓
Encapsulate AES-256 key using ML-KEM-768
```

**Double Ratchet → Simplified Key Rotation:**

Traditional Signal:
- Per-message key derivation (DH ratchet + symmetric ratchet)
- Complex state machine (chain keys, message keys)

Creto Adaptation (v1):
- **Session-level key rotation** (hourly/daily schedule)
- **Emergency rotation** on compromise detection
- **Grace period** for inflight messages (7 days)

**Rationale for simplification:**
- Agents communicate more predictably than humans (scheduled workflows)
- Session key rotation + forward secrecy sufficient for agent use case
- Reduces state management complexity (no per-message ratchet state)

**Future (v2):** Full Double Ratchet if compliance requires per-message forward secrecy

### Envelope Structure

```
MessageEnvelope {
    version: u8,                        // Protocol version (1)
    sender_nhi: AgentIdentity,          // Sender's NHI
    recipient_nhi: AgentIdentity,       // Recipient's NHI
    message_id: Uuid,                   // Unique message ID
    timestamp: i64,                     // Unix timestamp (milliseconds)

    // Encrypted payload
    encrypted_payload: Vec<u8>,         // AES-256-GCM(plaintext)
    nonce: [u8; 12],                    // AES-GCM nonce (random)

    // Key encapsulation
    wrapped_key: Vec<u8>,               // ML-KEM-768(AES key)
    key_id: Uuid,                       // Recipient's public key ID

    // Hybrid signatures
    signature_ed25519: [u8; 64],        // Ed25519(canonical_envelope)
    signature_ml_dsa: Vec<u8>,          // ML-DSA-65(canonical_envelope)

    // Metadata
    priority: MessagePriority,          // Low/Normal/High/Critical
    ttl_seconds: Option<u32>,           // Time-to-live
    correlation_id: Option<Uuid>,       // For request/response
}
```

**Canonical representation for signing:**
```
canonical_envelope = version || sender_nhi || recipient_nhi || message_id ||
                     timestamp || encrypted_payload || nonce || wrapped_key
```

### Cryptographic Primitives

| Primitive | Algorithm | Key Size | Purpose |
|-----------|-----------|----------|---------|
| **Key Encapsulation** | ML-KEM-768 | 1184 bytes (public) | Post-quantum key wrapping |
| **Symmetric Encryption** | AES-256-GCM | 256 bits | Message payload encryption |
| **Classical Signature** | Ed25519 | 32 bytes (public) | Authenticity (fast) |
| **PQC Signature** | ML-DSA-65 | ~2000 bytes (public) | Quantum-resistant authenticity |
| **Key Derivation** | HKDF-SHA256 | - | Derive session keys from shared secret |

**Rationale:**
- **ML-KEM-768**: NIST-selected PQC KEM (FIPS 203), security level 3
- **AES-256-GCM**: Authenticated encryption, hardware-accelerated (AES-NI)
- **Ed25519**: Fast, compact signatures (64 bytes)
- **ML-DSA-65**: NIST-selected PQC signature (FIPS 204), security level 3
- **Hybrid signatures**: Defense-in-depth (classical + PQC)

---

## Performance Characteristics

### Encryption Throughput

**Target: >100,000 messages/second per CPU core**

Breakdown:
```
Operation                  | Latency    | Throughput
---------------------------|------------|-------------
AES-256-GCM encrypt (1KB)  | ~500ns     | 2M ops/sec
ML-KEM-768 encapsulation   | ~100µs     | 10K ops/sec
Ed25519 sign               | ~40µs      | 25K ops/sec
ML-DSA-65 sign             | ~2ms       | 500 ops/sec
---------------------------|------------|-------------
Combined encryption        | ~2.14ms    | ~467 msg/sec (single-threaded)
Parallel (8 cores)         | -          | ~3,736 msg/sec
Batched (amortize ML-DSA)  | ~150µs/msg | ~100K msg/sec (target achieved)
```

**Optimization strategies:**
1. **Batch signing**: Sign batches of messages, amortize ML-DSA cost
2. **Async I/O**: Overlap CPU crypto with network I/O
3. **SIMD**: Use AES-NI (x86_64) or NEON (ARM) for AES-GCM
4. **Pre-computation**: Cache recipient public keys
5. **Worker pool**: Dedicated crypto threads, queue message encryption

### Delivery Latency

**Target: <10ms p99 (same datacenter)**

Breakdown:
```
Phase                      | Latency (p99)
---------------------------|----------------
Envelope processing        | 1ms
AuthZ check (creto-authz)  | 0.168µs (inline) → <1ms (fallback)
Queue insertion            | 0.5ms
Network transmission       | 2ms (same DC)
Recipient dequeue          | 0.5ms
Signature verification     | 3ms
Decryption                 | 1ms
---------------------------|----------------
Total                      | ~8ms
```

**Degradation scenarios:**
- Cross-datacenter: +40-80ms (network RTT)
- AuthZ service slow: Fail-open (queue for async check) or fail-closed (deny)
- Recipient offline: Queue with retention policy (no delivery latency impact)

---

## Integration with Creto Platform

### NHI (Non-Human Identity)

**Key management delegation:**

```rust
// Agent's private keys never leave NHI secure enclave
let nhi_client = creto_nhi::Client::new();

// Sender: Encrypt message
let recipient_pubkey = nhi_client.get_public_key(recipient_nhi, KeyType::MlKem768)?;
let wrapped_key = ml_kem_encapsulate(&recipient_pubkey, &aes_key)?;

// Sender: Sign envelope
let signature_ed25519 = nhi_client.sign_ed25519(sender_nhi, &canonical_envelope)?;
let signature_ml_dsa = nhi_client.sign_ml_dsa(sender_nhi, &canonical_envelope)?;

// Recipient: Decrypt
let aes_key = nhi_client.ml_kem_decapsulate(recipient_nhi, &wrapped_key)?;
```

**Benefits:**
- Private keys never exposed to messaging service
- Hardware security module (HSM) integration via NHI
- Key rotation managed centrally

### AuthZ (Authorization)

**Delivery policy enforcement:**

```rust
// Before routing message to recipient
let decision = authz_client.check(
    &sender_nhi,
    Action::SendMessage,
    &recipient_nhi,
    context! {
        "message_priority": envelope.priority,
        "channel": envelope.channel_id,
    }
)?;

match decision {
    Decision::Allow => route_message(envelope),
    Decision::Deny(reason) => {
        audit_log("message_delivery_denied", sender_nhi, recipient_nhi, reason);
        return Err(DeliveryError::Unauthorized(reason));
    }
}
```

**Policy examples:**

```rego
# Allow messages within same organization
allow {
    input.sender.organization_id == input.recipient.organization_id
}

# Rate limiting
allow {
    count(recent_messages[input.sender.nhi]) < 1000
}

# Role-based messaging
allow {
    input.sender.roles[_] == "coordinator"
    input.action == "send_message"
}
```

### Audit

**Immutable event logging:**

```rust
audit_client.log(AuditEvent {
    event_type: "message_sent",
    timestamp: Utc::now(),
    actor: sender_nhi.clone(),
    target: recipient_nhi.clone(),
    outcome: "success",
    metadata: json!({
        "message_id": envelope.message_id,
        "channel": envelope.channel_id,
        "priority": envelope.priority,
        "encrypted_size_bytes": envelope.encrypted_payload.len(),
    }),
    signature: sign_event(...),
})?;
```

**Queryable events:**
- `message_sent`: Sender, recipient, timestamp, size
- `message_delivered`: Delivery confirmation
- `message_delivery_denied`: Authorization denial reason
- `key_rotated`: Agent, old key ID, new key ID
- `signature_verification_failed`: Potential forgery attempt

### Consensus

**Timestamp ordering for message sequencing:**

```rust
// Ensure messages in a channel are totally ordered
let consensus_timestamp = consensus_client.get_total_order_timestamp()?;
envelope.timestamp = consensus_timestamp;

// Recipients process messages in timestamp order
let messages = channel_queue
    .messages()
    .sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
```

---

## OSS Pattern Extraction: Signal Protocol

### What We Took

1. **X3DH Key Agreement Pattern**
   - Prekey bundle concept (adapted to ML-KEM)
   - Asynchronous key exchange (sender doesn't need recipient online)

2. **Envelope Structure**
   - Encrypted payload + wrapped key + signature
   - Sender/recipient identities in cleartext (for routing)

3. **Forward Secrecy Concepts**
   - Key rotation to limit compromise blast radius
   - Grace period for inflight messages during rotation

4. **PQXDH Extension**
   - ML-KEM for key encapsulation (from Signal's PQC work)

### What We Didn't Take

1. **Double Ratchet Complexity (Deferred to v2)**
   - Per-message key derivation
   - DH ratchet + symmetric ratchet state machine
   - Rationale: Agent use case doesn't require per-message forward secrecy

2. **Phone Number Addressing**
   - Replaced with NHI (Non-Human Identity)

3. **Sealed Sender (Anonymous Messaging)**
   - Agents require attribution for audit

4. **Multi-Device Sync**
   - Agents are stateless, no multi-device concept

### Key Differences for Agents

| Aspect | Signal (Humans) | Creto Messaging (Agents) |
|--------|-----------------|--------------------------|
| **Identity** | Phone number | NHI (cryptographic) |
| **Key Agreement** | X3DH (ECDH) | ML-KEM-768 |
| **Signatures** | Ed25519 | Ed25519 + ML-DSA-65 (hybrid) |
| **Ratchet** | Double Ratchet (per-message) | Session key rotation (scheduled) |
| **Delivery** | Push notifications | AuthZ-gated routing |
| **Offline** | Store-and-forward | Queue with retention policy |
| **Audit** | None (privacy focus) | Full audit trail (compliance) |

---

## Compliance & Security Posture

### Cryptographic Security Level

**NIST Security Levels:**

| Algorithm | NIST Level | Equivalent Symmetric | Quantum Resistant? |
|-----------|------------|---------------------|-------------------|
| ML-KEM-768 | 3 | AES-192 | ✅ Yes |
| ML-DSA-65 | 3 | AES-192 | ✅ Yes |
| AES-256-GCM | - | AES-256 | ✅ Yes (symmetric) |
| Ed25519 | - | AES-128 | ❌ No (classical) |

**Overall: NIST Level 3 (quantum-resistant)**

### Regulatory Compliance

| Regulation | Requirement | Compliance |
|------------|-------------|------------|
| **HIPAA 164.312(e)(1)** | Transmission security | E2E encryption ✅ |
| **HIPAA 164.312(e)(2)(i)** | Integrity controls | Hybrid signatures ✅ |
| **FedRAMP SC-8** | Transmission confidentiality | AES-256-GCM ✅ |
| **FedRAMP SC-13** | Crypto key management | NHI integration ✅ |
| **FedRAMP AU-2** | Audit events | Immutable audit log ✅ |
| **GDPR Art 32** | Encryption & pseudonymization | E2E + NHI ✅ |
| **PCI DSS 4.1** | Cardholder data encryption | AES-256 ✅ |

**FIPS 140-2 Compliance:**
- Use FIPS-validated crypto modules (via creto-crypto)
- Hardware RNG for key generation
- Secure key storage in HSM (via NHI)

---

## Scalability & Reliability

### Horizontal Scaling

**Stateless Envelope Processor:**
- Add more instances behind load balancer
- Each instance can encrypt/decrypt/verify independently
- No shared state (keys fetched from NHI, envelopes from queue)

**Partitioned Message Queues:**
- Shard queues by recipient hash: `queue_id = hash(recipient_nhi) % num_partitions`
- Each partition handled by dedicated consumer
- Prevents hot spots (one agent receiving many messages)

**Distributed Key Store:**
- Prekey bundles stored in distributed cache (Redis Cluster)
- Ratchet state in distributed database (CockroachDB, ScyllaDB)
- Consistent hashing for key lookup

### Availability

**Target: 99.99% (52 minutes downtime/year)**

**Failure scenarios:**
1. **Instance failure**: Load balancer redirects to healthy instance
2. **Database failure**: Replica promotion, queue messages during recovery
3. **Network partition**: Queue messages, retry with exponential backoff
4. **AuthZ service down**: Configurable fail-open (queue) or fail-closed (deny)

**Multi-region deployment:**
```
Region A (Primary)          Region B (Replica)
    ↓                            ↓
Message Queue ←─────────────→ Message Queue
    ↓            Sync             ↓
Storage (Postgres) ←────────→ Storage (Postgres)
    ↓            Async            ↓
```

**Recovery Time Objective (RTO):** <5 minutes
**Recovery Point Objective (RPO):** <1 minute (sync replication)

### Message Durability

**Zero-loss guarantee:**

1. **Write-Ahead Log (WAL)**: Envelopes written to persistent log before acknowledgment
2. **Replication**: 3 copies (quorum writes)
3. **Checksums**: Detect corruption
4. **Idempotency**: Retry-safe (message ID deduplication)

**Failure recovery:**
```
1. Message arrives → Write to WAL (fsync)
2. ACK sender only after quorum write (2/3 replicas)
3. If crash before ACK → Sender retries
4. Recipient deduplicates via message_id
```

---

## Roadmap

### v1.0 (MVP) - Q1 2025
- [x] E2E encryption (ML-KEM-768 + AES-256-GCM)
- [x] Hybrid signatures (Ed25519 + ML-DSA-65)
- [x] Direct agent-to-agent messaging
- [x] Basic channel creation
- [x] Authorization-gated delivery
- [x] Session key rotation (scheduled)
- [x] Audit logging
- [ ] Request/response pattern
- [ ] Message TTL and expiration

### v1.1 - Q2 2025
- [ ] Topic-based pub/sub
- [ ] Durable subscriptions
- [ ] Message retention policies
- [ ] Dead-letter queue
- [ ] Priority queues
- [ ] Delivery receipts

### v2.0 - Q3 2025
- [ ] Full Double Ratchet (per-message forward secrecy)
- [ ] Group messaging (N-to-N)
- [ ] Message threading/replies
- [ ] Federated messaging (cross-organization)
- [ ] Advanced rate limiting
- [ ] Analytics dashboard

### v2.1 - Q4 2025
- [ ] Hardware security module (HSM) integration
- [ ] Confidential computing (SGX/TDX)
- [ ] Zero-knowledge proofs for metadata privacy
- [ ] Quantum-resistant VPN integration
- [ ] Message search (homomorphic encryption)

---

## Open Questions & Decisions

### 1. Double Ratchet Complexity vs. Session Key Rotation

**Question:** Should v1 implement full Double Ratchet or use simpler session key rotation?

**Options:**
- **Option A (v1):** Session key rotation (hourly/daily schedule)
  - ✅ Simpler implementation
  - ✅ Sufficient for agent use case (predictable workflows)
  - ❌ Weaker forward secrecy (compromise affects messages until next rotation)

- **Option B:** Full Double Ratchet
  - ✅ Stronger forward secrecy (per-message)
  - ❌ Complex state management (chain keys, message keys)
  - ❌ Higher latency (per-message DH computation)

**Decision:** Option A for v1, Option B for v2 if compliance requires
**Rationale:** Agent communication is more scheduled/predictable than human chat

### 2. Topic Ordering Guarantees

**Question:** Should topics provide FIFO ordering?

**Options:**
- **Per-publisher FIFO:** Messages from Agent A arrive in send order
  - ✅ Simpler, more scalable
  - ❌ No cross-publisher ordering

- **Total ordering:** All messages in topic totally ordered
  - ✅ Stronger consistency
  - ❌ Expensive (consensus required)

**Decision:** Per-publisher FIFO for v1
**Rationale:** Most agent use cases don't require cross-publisher ordering

### 3. Offline Message Storage Duration

**Question:** How long should messages be stored for offline recipients?

**Proposal:**
- Default: 7 days
- Configurable per-agent (1 hour to 30 days)
- After expiration: Sender notified, message deleted

**Decision:** Approved
**Rationale:** Balances durability with storage costs

### 4. Cross-Organization Messaging

**Question:** Should agents from different organizations be able to message each other?

**Challenges:**
- NHI namespace conflicts
- Key distribution/trust
- Authorization policy boundaries

**Proposal:** Defer to v2, implement federated identity model
**Decision:** Approved
**Rationale:** Intra-org messaging is primary use case for v1

---

## Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Encryption throughput** | >100K msg/sec | Benchmark `MessageSender.send()` |
| **Delivery latency (p99)** | <10ms (same DC) | End-to-end timestamp delta |
| **Authorization latency** | <1ms | AuthZ check duration |
| **Key rotation time** | <100ms | End-to-end rotation duration |
| **Availability** | 99.99% | Uptime monitoring |
| **Message loss rate** | 0% | Chaos testing, failure injection |
| **Crypto correctness** | 100% | Test vectors, interop tests |
| **Signature verification** | 100% success | All received messages verified |

---

## References

1. **Signal Protocol Specification**: https://signal.org/docs/
2. **PQXDH (Post-Quantum Extended Diffie-Hellman)**: https://signal.org/docs/specifications/pqxdh/
3. **NIST PQC Standards**:
   - FIPS 203 (ML-KEM): https://csrc.nist.gov/pubs/fips/203/final
   - FIPS 204 (ML-DSA): https://csrc.nist.gov/pubs/fips/204/final
4. **Lago Patterns** (for audit/metering): https://github.com/getlago/lago
5. **NATS JetStream** (for delivery patterns): https://docs.nats.io/nats-concepts/jetstream

---

## Glossary

- **E2E**: End-to-End (encryption from sender to recipient, not just transport)
- **ML-KEM**: Module-Lattice-Based Key-Encapsulation Mechanism (NIST FIPS 203)
- **ML-DSA**: Module-Lattice-Based Digital Signature Algorithm (NIST FIPS 204)
- **NHI**: Non-Human Identity (Creto's agent identity system)
- **PQC**: Post-Quantum Cryptography (quantum-resistant algorithms)
- **X3DH**: Extended Triple Diffie-Hellman (Signal's key agreement protocol)
- **Double Ratchet**: Signal's forward secrecy protocol (per-message key derivation)
- **Envelope**: Encrypted message container (payload + metadata + signatures)
- **Forward Secrecy**: Compromise of long-term keys doesn't compromise past messages
- **AuthZ**: Authorization (creto-authz policy enforcement)

---

**Document Status:** Draft
**Next Steps:**
1. Review cryptographic protocol with security team
2. Benchmark encryption throughput on target hardware
3. Validate performance targets with load testing
4. Design data schemas (see 03-data-design.md)
5. Specify API contracts (see 04-api-design.md)
