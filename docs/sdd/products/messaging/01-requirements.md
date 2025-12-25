---
status: draft
author: Creto Team - Messaging Lead Agent
created: 2024-12-25
updated: 2024-12-25
reviewers: []
github_issue: "#5"
oss_reference: signal-protocol
---

# Messaging Requirements Specification

## Purpose

This document defines the functional and non-functional requirements for **creto-messaging**, the secure end-to-end encrypted communication system for AI agents in the Creto Sovereign platform.

## Scope

**In Scope:**
- End-to-end encrypted message exchange between agents
- Post-quantum cryptography (ML-KEM-768, ML-DSA-65)
- Hybrid cryptographic signatures (Ed25519 + ML-DSA)
- Message delivery with priority and expiration
- Channel and topic creation/management
- Cryptographic key rotation
- Request/response messaging patterns
- Publish/subscribe topic-based messaging
- Authorization-gated delivery
- Message persistence and acknowledgment

**Out of Scope:**
- Voice/video streaming (not relevant for agent communication)
- Large file transfer (use Storage layer)
- Message search/indexing (use Memory layer)
- Full Signal Protocol Double Ratchet (unnecessary complexity for agents)
- Human-facing messaging clients (UI/UX)

---

## 1. Functional Requirements

### FR-MSG-001: End-to-End Encryption (ML-KEM + AES-256-GCM)

**Priority:** Critical
**OSS Reference:** Signal Protocol envelope structure

#### Description
All messages between agents MUST be encrypted end-to-end such that only the intended recipient can decrypt the payload. The system MUST use post-quantum key encapsulation (ML-KEM-768) to protect symmetric keys.

#### Acceptance Criteria
- [ ] Sender generates AES-256 symmetric key for each message
- [ ] Payload encrypted with AES-256-GCM using generated key
- [ ] Symmetric key wrapped using recipient's ML-KEM-768 public key
- [ ] No intermediate system can read message content
- [ ] Encryption metadata (nonce, wrapped key) included in envelope
- [ ] Decryption only possible with recipient's ML-KEM-768 private key

#### User Stories
```gherkin
Feature: End-to-End Encryption

  Scenario: Agent sends encrypted message to another agent
    Given Agent A has a message "deploy model v2"
    And Agent B's ML-KEM-768 public key is registered in NHI
    When Agent A sends the message to Agent B
    Then the message payload is encrypted with AES-256-GCM
    And the symmetric key is wrapped with Agent B's ML-KEM public key
    And the encrypted envelope is transmitted
    And only Agent B can decrypt the message
    And the messaging service cannot read the plaintext

  Scenario: Message encryption with post-quantum protection
    Given quantum computers may break classical crypto
    When Agent A encrypts a message for Agent B
    Then ML-KEM-768 is used for key encapsulation
    And the encryption is quantum-resistant
    And forward secrecy is maintained

  Scenario: Failed decryption with wrong recipient key
    Given a message encrypted for Agent B
    When Agent C attempts to decrypt the message
    Then decryption fails with "key mismatch" error
    And the plaintext is never exposed
    And the attempt is logged to audit
```

#### Edge Cases
- **Recipient key rotation:** If recipient rotates keys mid-flight, message should be re-encrypted or queued for retry
- **Corrupted wrapped key:** Decryption fails, sender notified, message requeued with new encryption
- **Missing recipient key:** Message queued until recipient registers public key

#### Performance Requirements
- Encryption throughput: >100,000 messages/second per CPU core
- Key wrapping latency: <1ms per message (ML-KEM-768 operation)
- Decryption latency: <1ms per message

---

### FR-MSG-002: Hybrid Signatures (Ed25519 + ML-DSA)

**Priority:** Critical
**OSS Reference:** Signal Protocol non-repudiation extensions

#### Description
All messages MUST be cryptographically signed by the sender using both classical (Ed25519) and post-quantum (ML-DSA-65) signatures to ensure authenticity, integrity, and non-repudiation.

#### Acceptance Criteria
- [ ] Message envelope signed with Ed25519 (classical)
- [ ] Message envelope signed with ML-DSA-65 (post-quantum)
- [ ] Both signatures included in transmitted envelope
- [ ] Recipient verifies both signatures before trusting message
- [ ] Signatures computed over canonical envelope representation
- [ ] Signing keys obtained from sender's NHI identity
- [ ] Failed signature verification rejects message and logs to audit

#### User Stories
```gherkin
Feature: Hybrid Cryptographic Signatures

  Scenario: Message authenticity verification
    Given Agent A sends a message to Agent B
    When the message is transmitted
    Then the envelope includes Ed25519 signature
    And the envelope includes ML-DSA-65 signature
    And both signatures are computed over the same canonical data
    When Agent B receives the message
    Then Agent B verifies the Ed25519 signature
    And Agent B verifies the ML-DSA-65 signature
    And both signatures are valid
    And Agent B trusts the message is from Agent A

  Scenario: Non-repudiation proof
    Given Agent A sends "authorize payment $10,000" to Agent B
    When the message is signed and delivered
    Then the signatures provide cryptographic proof
    And Agent A cannot deny sending the message
    And the proof is admissible for audit/compliance

  Scenario: Signature forgery prevention
    Given an attacker intercepts a message from Agent A
    When the attacker modifies the payload
    And attempts to re-sign with fake keys
    Then the Ed25519 signature verification fails
    And the ML-DSA signature verification fails
    And Agent B rejects the message
    And the forgery attempt is logged

  Scenario: Quantum-resistant signatures
    Given a future quantum computer may break Ed25519
    When Agent A signs a message
    Then ML-DSA-65 provides quantum-resistant authenticity
    And the message remains trustworthy post-quantum
```

#### Edge Cases
- **Partial signature verification:** If Ed25519 passes but ML-DSA fails (or vice versa), reject message
- **Key rotation during verification:** Recipient must verify against correct public key version (use key ID in envelope)
- **Signature timestamp mismatch:** Reject if signature timestamp differs significantly from message timestamp

#### Performance Requirements
- Combined signature generation: <2ms (Ed25519 + ML-DSA)
- Combined signature verification: <2ms
- Signature size overhead: <3KB per message (ML-DSA-65 is ~2.5KB)

---

### FR-MSG-003: Message Delivery (Priority, Expiration)

**Priority:** High
**OSS Reference:** NATS JetStream durability

#### Description
The system MUST support configurable message delivery with priority levels, time-to-live (TTL) expiration, delivery acknowledgment, and retry logic.

#### Acceptance Criteria
- [ ] Messages can be sent with priority: Low, Normal, High, Critical
- [ ] Critical priority messages bypass normal queuing
- [ ] Messages can specify TTL (time-to-live)
- [ ] Expired messages are not delivered and sender is notified
- [ ] Delivery acknowledgment required for reliable messaging
- [ ] Unacknowledged messages are retried with exponential backoff
- [ ] Maximum retry count configurable (default: 5 attempts)
- [ ] Dead-letter queue for permanently failed messages

#### User Stories
```gherkin
Feature: Message Delivery Management

  Scenario: High-priority message delivery
    Given Agent A sends a critical alert message
    When the message priority is set to "Critical"
    Then the message bypasses normal FIFO queue
    And is delivered before Normal/Low priority messages
    And delivery latency is <10ms p99

  Scenario: Message expiration
    Given Agent A sends a message with TTL=5 seconds
    And the recipient Agent B is offline
    When 5 seconds elapse
    Then the message is marked expired
    And delivery is aborted
    And Agent A receives expiration notification
    And the expired message is logged to audit

  Scenario: Reliable delivery with acknowledgment
    Given Agent A sends a message to Agent B
    When the message is delivered to Agent B
    Then Agent B sends acknowledgment
    And Agent A is notified of successful delivery
    When acknowledgment is not received within timeout
    Then the message is requeued for retry
    And retry attempts increment

  Scenario: Dead-letter queue
    Given a message fails delivery 5 times
    When the maximum retry count is reached
    Then the message is moved to dead-letter queue
    And Agent A is notified of permanent failure
    And operations team is alerted for investigation
```

#### Edge Cases
- **Recipient comes online after expiration:** Message not delivered, sender informed
- **Network partition during delivery:** Message requeued, deduplicated on recipient side
- **Acknowledgment lost but message delivered:** Idempotency key prevents duplicate delivery

#### Performance Requirements
- Message routing latency: <5ms p50, <10ms p99 (same datacenter)
- Queue throughput: >50,000 messages/second per queue
- Acknowledgment roundtrip: <5ms p99

---

### FR-MSG-004: Channel Creation and Management

**Priority:** High
**OSS Reference:** NATS topic-based messaging

#### Description
Agents MUST be able to create, manage, and subscribe to communication channels for both direct 1:1 messaging and topic-based publish/subscribe patterns.

#### Acceptance Criteria
- [ ] Agents can create named topics
- [ ] Topic owner can set access policies: Open, Private, AuthzRequired, Allowlist
- [ ] Agents can publish messages to topics (subject to authorization)
- [ ] Agents can subscribe to topics (subject to authorization)
- [ ] Topic subscriptions are durable (survive agent restarts)
- [ ] Topics support configurable retention (duration-based)
- [ ] Topics support configurable max message size
- [ ] Topics support configurable max subscriber count

#### User Stories
```gherkin
Feature: Channel and Topic Management

  Scenario: Creating a private topic
    Given Agent A is a coordinator agent
    When Agent A creates topic "swarm/coordination"
    And sets policy to "Private"
    Then only Agent A can publish to the topic
    And only Agent A can subscribe to the topic
    And other agents receive "permission denied" when accessing

  Scenario: Publishing to a topic
    Given Agent A owns topic "swarm/coordination"
    And Agent B is allowlisted for the topic
    When Agent B publishes a message to the topic
    Then the message is delivered to all subscribers
    And Agent A receives the message
    And the publish is logged to audit

  Scenario: Topic subscription with authorization
    Given topic "public/announcements" has policy "Open"
    When Agent C subscribes to the topic
    Then the subscription is created
    And Agent C receives all new messages on the topic
    When the topic policy changes to "AuthzRequired"
    Then Agent C's subscription is re-validated
    And unauthorized agents are unsubscribed

  Scenario: Topic message retention
    Given topic "logs/debug" has retention=1 hour
    When messages are published to the topic
    And 1 hour elapses
    Then expired messages are purged
    And storage is reclaimed
    And subscribers cannot retrieve old messages
```

#### Edge Cases
- **Topic name collision:** Namespaced by owner NHI (e.g., `agent-id/topic-name`)
- **Subscriber offline during publish:** Messages buffered according to retention policy
- **Topic deletion with active subscribers:** Subscribers notified, subscriptions terminated gracefully

#### Performance Requirements
- Topic creation latency: <10ms
- Subscribe/unsubscribe latency: <5ms
- Topic fanout: >10,000 subscribers/second per topic
- Concurrent topics per agent: >1,000

---

### FR-MSG-005: Key Rotation

**Priority:** High
**OSS Reference:** Signal Protocol rekeying

#### Description
The system MUST support seamless cryptographic key rotation for both encryption (ML-KEM) and signing (Ed25519 + ML-DSA) keys without message loss or downtime.

#### Acceptance Criteria
- [ ] Agents can rotate encryption keys while preserving inflight messages
- [ ] Agents can rotate signing keys with version tracking
- [ ] Senders automatically use latest recipient public key
- [ ] Recipients maintain old keys for grace period (configurable, default 7 days)
- [ ] Messages encrypted with old keys still decryptable during grace period
- [ ] Key rotation events logged to audit
- [ ] Key rotation triggered manually or on schedule

#### User Stories
```gherkin
Feature: Cryptographic Key Rotation

  Scenario: Scheduled encryption key rotation
    Given Agent A has ML-KEM key pair with rotation policy "every 90 days"
    When 90 days elapse
    Then NHI generates new ML-KEM key pair for Agent A
    And the new public key is published
    And the old private key is retained for 7-day grace period
    And inflight messages encrypted with old key remain decryptable
    And new messages use the new public key

  Scenario: Manual key rotation after compromise
    Given Agent B's signing key is suspected compromised
    When Agent B initiates emergency key rotation
    Then NHI immediately generates new Ed25519 + ML-DSA keys
    And the old keys are marked revoked
    And all future messages use new signing keys
    And historical signatures remain verifiable (key version in signature)
    And the rotation is logged to audit

  Scenario: Graceful key migration
    Given Agent C rotates encryption keys
    And Agent D sends a message during rotation
    When Agent D's message is encrypted with old public key
    Then Agent C can still decrypt using old private key
    And no messages are lost during transition
    When the grace period expires
    Then the old private key is securely deleted
```

#### Edge Cases
- **Key rotation during high message volume:** Queue messages briefly, resume with new keys
- **Multiple concurrent key rotations:** Version each key, track active/grace/revoked states
- **Recipient key unknown after rotation:** Sender fetches latest key from NHI

#### Performance Requirements
- Key rotation completion time: <100ms
- Message delivery disruption: 0 messages lost
- Grace period key lookup: <1ms (cached in memory)

---

### FR-MSG-006: Request/Response Messaging Pattern

**Priority:** Medium
**OSS Reference:** NATS request/reply

#### Description
Agents MUST be able to send a message and synchronously wait for a response from the recipient, with timeout and correlation tracking.

#### Acceptance Criteria
- [ ] `send_and_wait()` API for request/response
- [ ] Automatic correlation ID generation
- [ ] Timeout configurable (default 30 seconds)
- [ ] Response matched to request via correlation ID
- [ ] Timeout returns error to caller
- [ ] Multiple concurrent requests supported

#### User Stories
```gherkin
Feature: Request/Response Messaging

  Scenario: Synchronous request/response
    Given Agent A needs to query Agent B for data
    When Agent A calls send_and_wait(Agent B, "get_status", timeout=5s)
    Then the message includes correlation_id
    And Agent A blocks waiting for response
    When Agent B receives the message
    And Agent B replies with status data
    Then the reply includes the same correlation_id
    And Agent A receives the response
    And the roundtrip completes in <5 seconds

  Scenario: Request timeout
    Given Agent A sends request to Agent B with timeout=2s
    When Agent B is offline
    And 2 seconds elapse
    Then Agent A receives timeout error
    And the request is marked failed
    And Agent A can retry or abort
```

#### Edge Cases
- **Response arrives after timeout:** Discard response, log late arrival
- **Multiple responses to single request:** Accept first, discard duplicates

#### Performance Requirements
- Correlation ID generation: <1µs
- Request/response roundtrip: <20ms p99 (same datacenter)

---

### FR-MSG-007: Authorization-Gated Delivery

**Priority:** Critical
**OSS Reference:** Creto Authorization integration

#### Description
Message delivery MUST be gated by authorization checks to enforce policy-based access control for who can send messages to whom and which topics agents can publish/subscribe to.

#### Acceptance Criteria
- [ ] Authorization check before message delivery
- [ ] Policy question: `can agent A send message to agent B?`
- [ ] Policy question: `can agent A publish to topic T?`
- [ ] Policy question: `can agent A subscribe to topic T?`
- [ ] Denied messages are not delivered and sender is notified
- [ ] Authorization check completes in <1ms
- [ ] Rate limiting enforced via authorization policies

#### User Stories
```gherkin
Feature: Authorization-Gated Delivery

  Scenario: Authorized message delivery
    Given Agent A wants to send message to Agent B
    When Authorization checks if Agent A can send to Agent B
    And the policy returns ALLOW
    Then the message is encrypted and delivered
    And the delivery is logged to audit

  Scenario: Unauthorized delivery attempt
    Given Agent C wants to send message to Agent D
    When Authorization checks if Agent C can send to Agent D
    And the policy returns DENY (reason: "not in same team")
    Then the message is rejected before encryption
    And Agent C receives "permission denied" error
    And the denial is logged to audit

  Scenario: Rate limiting via policy
    Given Agent E is rate-limited to 10 messages/minute
    When Agent E sends 11 messages in 1 minute
    Then the first 10 messages are delivered
    And the 11th message is denied with "rate limit exceeded"
    And Agent E receives retry-after duration

  Scenario: Topic publish authorization
    Given topic "confidential/finance" requires role "finance-agent"
    When Agent F (no finance role) publishes to the topic
    Then Authorization returns DENY
    And the message is rejected
    When Agent G (has finance role) publishes
    Then Authorization returns ALLOW
    And the message is delivered to subscribers
```

#### Edge Cases
- **Authorization service unavailable:** Fail-closed (deny delivery), queue for retry
- **Policy update during delivery:** Use policy version at delivery time

#### Performance Requirements
- Authorization check latency: <1ms (target 168ns via inline integration)
- Denied message processing: <100µs

---

## 2. Non-Functional Requirements

### NFR-MSG-001: Encryption Performance (>100K msg/sec)

**Priority:** High
**Category:** Performance

#### Description
The messaging system MUST achieve encryption throughput exceeding 100,000 messages per second per CPU core to support high-scale agent communication.

#### Acceptance Criteria
- [ ] AES-256-GCM encryption: >100K msg/sec per core
- [ ] ML-KEM-768 key wrapping: >1K wraps/sec per core
- [ ] Signature generation (Ed25519 + ML-DSA): >500 signatures/sec per core
- [ ] Benchmarks included in test suite
- [ ] Performance regression tests in CI/CD

#### Measurement
- Use `cargo bench` with `criterion` crate
- Test with 1KB, 10KB, 100KB payloads
- Measure on reference hardware (AWS c7g.xlarge)

#### Trade-offs
- SIMD acceleration for AES-GCM (x86_64 AES-NI, ARM NEON)
- Batch processing for ML-KEM operations
- Async I/O to avoid blocking on slow operations

---

### NFR-MSG-002: Delivery Authorization (<1ms)

**Priority:** High
**Category:** Performance

#### Description
Authorization checks for message delivery MUST complete in less than 1 millisecond to avoid becoming a bottleneck in the message path.

#### Acceptance Criteria
- [ ] Authorization check latency: <1ms p99
- [ ] Integration with creto-authz inline path (168ns target)
- [ ] Cached policy decisions for repeated checks
- [ ] Fallback to deny if authorization service slow/unavailable

#### Measurement
- Benchmark `DeliveryPolicy.can_deliver()` latency
- Monitor p50, p99, p999 latencies in production
- Alert if p99 exceeds 1ms

---

### NFR-MSG-003: Key Rotation (<100ms)

**Priority:** Medium
**Category:** Performance

#### Description
Cryptographic key rotation operations MUST complete in less than 100 milliseconds to minimize disruption to message delivery.

#### Acceptance Criteria
- [ ] Key pair generation: <50ms
- [ ] Key publication to NHI: <30ms
- [ ] Key cache invalidation: <10ms
- [ ] Total rotation time: <100ms
- [ ] Zero message loss during rotation

#### Measurement
- End-to-end rotation latency benchmark
- Track rotation success rate (target: 100%)

---

### NFR-MSG-004: End-to-End Latency (<10ms p99)

**Priority:** High
**Category:** Performance

#### Description
Message end-to-end latency (send to receive) MUST be less than 10ms at p99 for agents in the same datacenter.

#### Acceptance Criteria
- [ ] Same-datacenter latency: <5ms p50, <10ms p99
- [ ] Cross-datacenter latency: <50ms p99
- [ ] Latency measurement included in message envelope
- [ ] Monitoring and alerting on latency SLOs

#### Measurement
- Timestamp at send, timestamp at receive
- Report latency distribution in metrics

---

### NFR-MSG-005: Availability (99.99%)

**Priority:** High
**Category:** Reliability

#### Description
The messaging service MUST maintain 99.99% availability (52 minutes downtime/year).

#### Acceptance Criteria
- [ ] Multi-region deployment
- [ ] Automatic failover on component failure
- [ ] Health checks and circuit breakers
- [ ] Graceful degradation (queue messages during outages)
- [ ] Uptime monitoring and SLO tracking

---

### NFR-MSG-006: Scalability (1M agents)

**Priority:** High
**Category:** Scalability

#### Description
The system MUST scale horizontally to support 1 million concurrent agents with linear resource scaling.

#### Acceptance Criteria
- [ ] Horizontal scaling of message routers
- [ ] Partition topics across multiple brokers
- [ ] Shard message queues by recipient hash
- [ ] Load testing with 1M simulated agents
- [ ] Resource usage growth is linear (not exponential)

---

### NFR-MSG-007: Message Durability (Zero Loss)

**Priority:** Critical
**Category:** Reliability

#### Description
Acknowledged messages MUST NOT be lost under any failure scenario (disk, network, process crash).

#### Acceptance Criteria
- [ ] Messages persisted to durable storage before acknowledgment
- [ ] Write-ahead log for in-flight messages
- [ ] Replication factor: 3 (quorum-based writes)
- [ ] Recovery from disk/process failure
- [ ] Chaos testing (simulate failures, verify zero loss)

---

### NFR-MSG-008: Security (Post-Quantum Ready)

**Priority:** Critical
**Category:** Security

#### Description
All cryptographic primitives MUST be post-quantum resistant to protect against future quantum attacks.

#### Acceptance Criteria
- [ ] ML-KEM-768 for key encapsulation
- [ ] ML-DSA-65 for digital signatures
- [ ] Hybrid mode with Ed25519 for defense-in-depth
- [ ] Crypto-agility: algorithm swappable without protocol changes
- [ ] FIPS 203/204 compliance (when standardized)

---

### NFR-MSG-009: Auditability (100% Coverage)

**Priority:** High
**Category:** Compliance

#### Description
All messaging operations (send, receive, delivery denials, key rotations) MUST be logged to the immutable audit trail.

#### Acceptance Criteria
- [ ] Message sent event logged (sender, recipient, message ID, timestamp)
- [ ] Message received event logged
- [ ] Delivery denial logged (reason, policy ID)
- [ ] Key rotation logged (agent, old key ID, new key ID)
- [ ] Audit records are immutable (Merkle-anchored)
- [ ] Audit query performance: <100ms for recent events

---

### NFR-MSG-010: Compliance (HIPAA, FedRAMP)

**Priority:** High
**Category:** Compliance

#### Description
The messaging system MUST meet encryption and audit requirements for HIPAA and FedRAMP.

#### Acceptance Criteria
- [ ] HIPAA: PHI encrypted in transit (AES-256) ✓
- [ ] HIPAA: Access audit trail ✓
- [ ] FedRAMP SC-8: Transmission confidentiality ✓
- [ ] FedRAMP SC-13: Cryptographic protection (FIPS 140-2 modules)
- [ ] FedRAMP AU-2: Auditable events ✓

---

## 3. User Stories

### US-MSG-001: Agent-to-Agent Direct Messaging

**As** an AI agent
**I want** to send encrypted messages to another agent
**So that** I can coordinate actions securely without eavesdropping

**Acceptance Criteria:**
- Agent can address message to another agent by NHI
- Message is E2E encrypted (only recipient can read)
- Message is signed (recipient can verify sender)
- Delivery confirmation is received

---

### US-MSG-002: Topic-Based Broadcasting

**As** a coordinator agent
**I want** to publish updates to a topic that multiple agents subscribe to
**So that** I can efficiently broadcast coordination instructions

**Acceptance Criteria:**
- Coordinator creates topic with access policy
- Subscriber agents can subscribe to the topic
- Published messages are delivered to all subscribers
- Topic access is enforced via authorization

---

### US-MSG-003: Request/Response for Queries

**As** an agent
**I want** to send a request and synchronously wait for a response
**So that** I can query another agent's state without polling

**Acceptance Criteria:**
- Agent can send request with correlation ID
- Agent blocks until response or timeout
- Response is matched to request automatically
- Timeout returns error to caller

---

### US-MSG-004: Secure Key Rotation

**As** a security-conscious agent
**I want** to rotate my encryption keys periodically
**So that** I limit the blast radius of a key compromise

**Acceptance Criteria:**
- Agent can rotate keys on schedule or manually
- Inflight messages encrypted with old key remain decryptable
- New messages use new key automatically
- Key rotation completes without message loss

---

### US-MSG-005: Priority Message Delivery

**As** a monitoring agent
**I want** to send critical alerts with high priority
**So that** they are delivered before normal messages

**Acceptance Criteria:**
- Agent can set message priority (Critical, High, Normal, Low)
- Critical messages bypass normal queue
- Delivery latency for critical messages <10ms p99

---

### US-MSG-006: Authorization-Based Access Control

**As** a compliance agent
**I want** message delivery to be gated by authorization policies
**So that** agents can only communicate according to defined rules

**Acceptance Criteria:**
- Authorization checked before delivery
- Unauthorized messages are denied with reason
- Denials are logged to audit
- Policies can enforce rate limits

---

## 4. Edge Cases and Scenarios

### Scenario 1: Network Partition

**Given:** Agent A sends message to Agent B
**When:** Network partition occurs mid-delivery
**Then:**
- Message is queued for retry
- Exponential backoff applied (1s, 2s, 4s, 8s, 16s)
- After 5 retries, message moved to dead-letter queue
- Agent A is notified of failure
- Audit logged

### Scenario 2: Recipient Key Rotation During Message Encryption

**Given:** Agent A starts encrypting message for Agent B
**When:** Agent B rotates encryption key before message is sent
**Then:**
- Sender re-fetches latest public key from NHI
- Message is re-encrypted with new key
- Delivery proceeds normally
- Old key is not used

### Scenario 3: Signature Verification Failure

**Given:** Message arrives at Agent B
**When:** Ed25519 signature is valid but ML-DSA signature is invalid
**Then:**
- Message is rejected (both signatures must be valid)
- Rejection logged to audit (potential forgery attempt)
- Sender notified of verification failure
- Message not delivered

### Scenario 4: Topic Subscription While Offline

**Given:** Agent C is offline
**When:** Agent C's subscription is created to topic "updates"
**Then:**
- Subscription is marked "pending" until Agent C comes online
- Messages published while offline are buffered (per retention policy)
- When Agent C comes online, buffered messages delivered
- Expired messages (beyond retention) are not delivered

### Scenario 5: Concurrent Message Delivery to Same Agent

**Given:** 10,000 agents send messages to Agent D simultaneously
**When:** Messages arrive at routing layer
**Then:**
- Messages are queued in priority order (Critical first)
- Agent D's receiver processes messages sequentially
- No messages are lost
- Delivery rate matches Agent D's consumption rate
- Backpressure applied to senders if queue full

---

## 5. OSS Reference Patterns

### From Signal Protocol

**X3DH Key Agreement (Adapted for NHI):**
- Original: Phone number → prekey bundle lookup
- Creto: Agent NHI → ML-KEM public key lookup via NHI service

**Envelope Structure:**
- Original: `encrypted_payload || wrapped_key || sender_identity`
- Creto: `encrypted_payload || wrapped_key_ml_kem || sig_ed25519 || sig_ml_dsa || sender_nhi`

**Forward Secrecy Concepts:**
- Original: Double Ratchet with per-message keys
- Creto: Simplified (no per-message ratchet, but support key rotation)

**Post-Quantum Extensions (PQXDH):**
- Original: ML-KEM for X3DH
- Creto: ML-KEM-768 for key wrapping, ML-DSA-65 for signatures

### From NATS

**Request/Reply Pattern:**
- Correlation ID for request/response matching
- Timeout handling on sender side

**Topic-Based Pub/Sub:**
- Hierarchical topic names (`swarm/coordination/agent-123`)
- Subscriber groups for load balancing (future enhancement)

**JetStream Durability:**
- Message persistence with acknowledgment
- Retention policies (time-based, message count)
- Dead-letter queues for failed messages

---

## 6. Compliance Mapping

| Regulation | Requirement | Messaging Compliance |
|------------|-------------|---------------------|
| **HIPAA 164.312(e)(1)** | Transmission security | E2E encryption (AES-256-GCM) ✓ |
| **HIPAA 164.312(e)(2)(i)** | Integrity controls | Signatures (Ed25519 + ML-DSA) ✓ |
| **HIPAA 164.312(e)(2)(ii)** | Encryption | ML-KEM-768 key wrapping ✓ |
| **FedRAMP SC-8** | Transmission confidentiality | E2E encryption ✓ |
| **FedRAMP SC-8(1)** | Cryptographic protection | FIPS 140-2 crypto modules ✓ |
| **FedRAMP SC-13** | Cryptographic key management | NHI key management, rotation ✓ |
| **FedRAMP AU-2** | Audit events | All operations logged to immutable audit ✓ |
| **PCI DSS 4.1** | Encryption of cardholder data | AES-256 encryption ✓ |
| **GDPR Art 32** | Security of processing | Encryption, pseudonymization ✓ |

---

## 7. Dependencies

### Platform Layer
- **creto-nhi:** Agent identity, key management, signing/decryption delegation
- **creto-crypto:** ML-KEM-768, ML-DSA-65, Ed25519, AES-256-GCM implementations
- **creto-consensus:** Timestamp ordering for message sequencing
- **creto-audit:** Immutable audit trail for all operations

### Security Layer
- **creto-authz:** Authorization checks for delivery gating (168ns inline path)

### External
- **Storage backend:** Persistent message queue (RocksDB, PostgreSQL)
- **Networking:** Transport layer (QUIC, TCP/TLS)

---

## 8. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Encryption throughput** | >100K msg/sec | Benchmark `MessageSender.send()` |
| **Delivery latency p99** | <10ms | End-to-end timestamp delta |
| **Authorization check** | <1ms | `DeliveryPolicy.can_deliver()` latency |
| **Key rotation time** | <100ms | End-to-end rotation duration |
| **Availability** | 99.99% | Uptime monitoring |
| **Message loss rate** | 0% | Chaos testing, failure injection |
| **Signature verification success** | 100% | Verify all received messages |

---

## 9. Open Questions

1. **Should we implement full Double Ratchet for per-message forward secrecy?**
   - Tradeoff: Complexity vs. security benefit for agent use case
   - Decision: Defer to v2, focus on key rotation for v1

2. **What is the default message retention policy for topics?**
   - Proposal: 7 days for standard topics, 24 hours for ephemeral
   - Configurable per-topic

3. **Should we support message recall/deletion after delivery?**
   - Use case: Revoke accidentally sent credentials
   - Challenge: E2E encryption makes server-side deletion impossible
   - Proposal: Sender can request recipient to delete, but not enforceable

4. **How long should offline agents buffer messages?**
   - Proposal: Up to retention period, then expire with sender notification
   - Configurable per-agent

5. **Should topics support ordering guarantees?**
   - FIFO ordering within single publisher?
   - Total ordering across all publishers (expensive)?
   - Proposal: Per-publisher FIFO, no cross-publisher ordering

6. **Cross-organization agent messaging?**
   - Challenge: NHI namespace, key distribution
   - Proposal: Federated identity model (future work)

---

## 10. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team - Messaging Lead Agent | Initial requirements extraction from SDD 01, Product SDD, and Signal Protocol analysis |
