---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
oss_reference: signal-protocol
---

# Product SDD: creto-messaging

## Purpose

Secure end-to-end encrypted communication between AI agents. Enables agents to exchange messages with confidentiality (only recipient can read), authenticity (verifiable sender), and non-repudiation (signed messages), using post-quantum cryptography.

## Scope

**In Scope:**
- MessageEnvelope format (E2E encrypted)
- ML-KEM-768 key wrapping (post-quantum)
- Ed25519 + ML-DSA hybrid signatures
- Request/response messaging pattern
- Pub/sub topic-based messaging
- Authorization-gated delivery
- Message persistence and acknowledgment

**Out of Scope:**
- Full Signal Protocol (Double Ratchet complexity not needed for agent-to-agent)
- Voice/video streaming
- Large file transfer (use Storage layer)
- Message search/indexing (use Memory layer)

---

## 1. OSS Reference: Signal Protocol

**Specification:** https://signal.org/docs/specifications/

**Key Patterns to Extract:**
- X3DH key agreement (adapted for NHI keys)
- Envelope structure (encrypted payload + wrapped key)
- Non-repudiation via signatures
- Forward secrecy concepts

**Differences from Signal:**
- Simplified key management (NHI provides keys)
- No Double Ratchet (agents don't need per-message forward secrecy)
- Post-quantum from day one (ML-KEM-768, ML-DSA)
- Authorization-gated delivery
- Topic-based pub/sub (not just 1:1)

---

## 2. Core Traits

### 2.1 MessageEnvelope

```rust
/// E2E encrypted message envelope
pub struct MessageEnvelope {
    // Identification
    pub message_id: MessageId,
    pub correlation_id: Option<MessageId>,  // For request/response

    // Routing
    pub sender_nhi: AgentIdentity,
    pub recipient: MessageRecipient,

    // Encrypted content
    pub encrypted_payload: Vec<u8>,          // AES-256-GCM ciphertext
    pub encryption_nonce: [u8; 12],          // GCM nonce
    pub wrapped_key: Vec<u8>,                // ML-KEM-768 wrapped symmetric key

    // Signatures (non-repudiation)
    pub signature_ed25519: [u8; 64],         // Ed25519 signature
    pub signature_ml_dsa: Vec<u8>,           // ML-DSA-65 signature (PQ)

    // Metadata
    pub timestamp: Timestamp,
    pub ttl: Option<Duration>,
    pub content_type: ContentType,
    pub priority: MessagePriority,
}

pub enum MessageRecipient {
    /// Direct to agent
    Agent(AgentIdentity),
    /// Broadcast to topic subscribers
    Topic(TopicId),
    /// Multicast to specific agents
    Multicast(Vec<AgentIdentity>),
}

pub enum ContentType {
    /// Arbitrary bytes
    Binary,
    /// JSON payload
    Json,
    /// Protocol buffers
    Protobuf { schema: String },
    /// Reference to Storage object
    StorageRef { storage_id: StorageId, hash: Hash },
}

pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,  // Bypass normal queuing
}
```

### 2.2 MessageSender

```rust
/// Sending messages to other agents
#[async_trait]
pub trait MessageSender: Send + Sync {
    /// Send message to recipient (fire-and-forget)
    async fn send(
        &self,
        to: &MessageRecipient,
        payload: &[u8],
        options: SendOptions,
    ) -> Result<MessageId, Error>;

    /// Send and wait for reply (request/response)
    async fn send_and_wait(
        &self,
        to: &AgentIdentity,
        payload: &[u8],
        timeout: Duration,
        options: SendOptions,
    ) -> Result<Response, Error>;

    /// Publish to topic (pub/sub)
    async fn publish(
        &self,
        topic: &TopicId,
        payload: &[u8],
        options: SendOptions,
    ) -> Result<MessageId, Error>;
}

pub struct SendOptions {
    pub content_type: ContentType,
    pub priority: MessagePriority,
    pub ttl: Option<Duration>,
    pub require_ack: bool,
    pub correlation_id: Option<MessageId>,
}

pub struct Response {
    pub message_id: MessageId,
    pub payload: Vec<u8>,
    pub sender: AgentIdentity,
    pub latency: Duration,
}
```

### 2.3 MessageReceiver

```rust
/// Receiving messages
#[async_trait]
pub trait MessageReceiver: Send + Sync {
    /// Receive next message (blocking)
    async fn receive(&self) -> Result<ReceivedMessage, Error>;

    /// Receive with timeout
    async fn receive_timeout(&self, timeout: Duration) -> Result<Option<ReceivedMessage>, Error>;

    /// Subscribe to topic
    async fn subscribe(&self, topic: &TopicId) -> Result<Subscription, Error>;

    /// Unsubscribe from topic
    async fn unsubscribe(&self, subscription: Subscription) -> Result<(), Error>;

    /// Acknowledge message receipt
    async fn acknowledge(&self, message_id: &MessageId) -> Result<(), Error>;

    /// Negative acknowledge (request redelivery)
    async fn nack(&self, message_id: &MessageId, reason: &str) -> Result<(), Error>;
}

pub struct ReceivedMessage {
    pub envelope: MessageEnvelope,
    pub decrypted_payload: Vec<u8>,
    pub verified: SignatureVerification,
    pub received_at: Timestamp,
}

pub struct SignatureVerification {
    pub ed25519_valid: bool,
    pub ml_dsa_valid: bool,
    pub sender_verified: bool,  // NHI lookup confirmed sender
}

pub struct Subscription {
    pub id: SubscriptionId,
    pub topic: TopicId,
    pub filter: Option<MessageFilter>,
}
```

### 2.4 DeliveryPolicy

```rust
/// Authorization-gated delivery
#[async_trait]
pub trait DeliveryPolicy: Send + Sync {
    /// Check if sender can deliver to recipient
    async fn can_deliver(
        &self,
        sender: &AgentIdentity,
        recipient: &MessageRecipient,
    ) -> Result<DeliveryDecision, Error>;
}

pub enum DeliveryDecision {
    Allow,
    Deny { reason: String },
    RequireOversight { policy_id: PolicyId },
    RateLimit { retry_after: Duration },
}
```

---

## 3. Cryptographic Design

### 3.1 Encryption Flow

```
Sender                                              Recipient
  │                                                     │
  │ 1. Generate symmetric key                           │
  │    K = random(256 bits)                             │
  │                                                     │
  │ 2. Encrypt payload                                  │
  │    C = AES-256-GCM(K, nonce, payload)               │
  │                                                     │
  │ 3. Wrap symmetric key with recipient's ML-KEM      │
  │    WK = ML-KEM-768.Encapsulate(recipient.pk)        │
  │                                                     │
  │ 4. Sign envelope (Ed25519 + ML-DSA)                │
  │    sig_ed = Ed25519.Sign(sender.sk, envelope)       │
  │    sig_pq = ML-DSA-65.Sign(sender.sk_pq, envelope)  │
  │                                                     │
  │ 5. Transmit envelope                                │
  │ ─────────────────────────────────────────────────► │
  │                                                     │
  │                      6. Verify signatures           │
  │                         Ed25519.Verify(sender.pk)   │
  │                         ML-DSA.Verify(sender.pk_pq) │
  │                                                     │
  │                      7. Unwrap symmetric key        │
  │                         K = ML-KEM.Decapsulate(WK)  │
  │                                                     │
  │                      8. Decrypt payload             │
  │                         payload = AES-GCM.Dec(K, C) │
  │                                                     │
```

### 3.2 Key Management

```rust
/// Keys are managed by NHI, not messaging
pub struct AgentKeys {
    // Obtained from NHI service
    pub identity: AgentIdentity,

    // Encryption (ML-KEM-768)
    pub encryption_public: MlKemPublicKey,
    // Private key held by NHI, operations delegated

    // Signing (Ed25519 + ML-DSA hybrid)
    pub signing_public_ed25519: Ed25519PublicKey,
    pub signing_public_ml_dsa: MlDsaPublicKey,
    // Private keys held by NHI
}

// Messaging requests crypto operations from NHI
impl MessagingService {
    async fn encrypt_for(&self, recipient: &AgentIdentity, payload: &[u8]) -> Result<EncryptedPayload, Error> {
        // 1. Get recipient's public key from NHI
        let recipient_keys = self.nhi.get_public_keys(recipient).await?;

        // 2. Generate symmetric key locally
        let symmetric_key = generate_aes_key();

        // 3. Encrypt payload locally
        let ciphertext = aes_gcm_encrypt(&symmetric_key, payload)?;

        // 4. Wrap key with recipient's ML-KEM public key
        let wrapped_key = ml_kem_encapsulate(&recipient_keys.encryption_public)?;

        Ok(EncryptedPayload { ciphertext, wrapped_key })
    }

    async fn sign(&self, envelope: &MessageEnvelope) -> Result<Signatures, Error> {
        // Delegate signing to NHI (private keys never leave NHI)
        let data = envelope.canonical_bytes();

        let sig_ed = self.nhi.sign_ed25519(&self.identity, &data).await?;
        let sig_pq = self.nhi.sign_ml_dsa(&self.identity, &data).await?;

        Ok(Signatures { ed25519: sig_ed, ml_dsa: sig_pq })
    }
}
```

---

## 4. Data Models

### 4.1 Message Storage

```rust
pub struct StoredMessage {
    pub message_id: MessageId,
    pub envelope: MessageEnvelope,

    // Delivery tracking
    pub status: MessageStatus,
    pub delivery_attempts: u32,
    pub last_attempt: Option<Timestamp>,
    pub delivered_at: Option<Timestamp>,
    pub acknowledged_at: Option<Timestamp>,

    // Retention
    pub expires_at: Option<Timestamp>,
    pub retained_until: Timestamp,
}

pub enum MessageStatus {
    Queued,
    Delivering,
    Delivered,
    Acknowledged,
    Failed { error: String },
    Expired,
}
```

### 4.2 Topic

```rust
pub struct Topic {
    pub id: TopicId,
    pub name: String,

    // Access control
    pub owner_nhi: AgentIdentity,
    pub publish_policy: TopicPolicy,
    pub subscribe_policy: TopicPolicy,

    // Configuration
    pub retention: Duration,
    pub max_message_size: usize,
    pub max_subscribers: Option<usize>,
}

pub enum TopicPolicy {
    /// Anyone can publish/subscribe
    Open,
    /// Only owner can publish/subscribe
    Private,
    /// Authorization check required
    AuthzRequired,
    /// Explicit allowlist
    Allowlist(Vec<AgentIdentity>),
}
```

---

## 5. Architecture

### 5.1 Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                     creto-messaging                         │
│                                                             │
│  ┌─────────────────┐         ┌─────────────────────────┐   │
│  │  Message        │         │    Message              │   │
│  │  Sender         │         │    Receiver             │   │
│  └────────┬────────┘         └────────────┬────────────┘   │
│           │                               │                │
│           ▼                               ▼                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Encryption Engine                       │   │
│  │  (AES-256-GCM + ML-KEM-768 + Ed25519 + ML-DSA)       │   │
│  └─────────────────────────────────────────────────────┘   │
│           │                               │                │
│           ▼                               ▼                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Delivery Router                         │   │
│  │  (direct | topic | multicast)                        │   │
│  └─────────────────────────────────────────────────────┘   │
│           │                               │                │
│           ▼                               ▼                │
│  ┌─────────────────┐         ┌─────────────────────────┐   │
│  │  Message Queue  │         │    Topic Manager        │   │
│  │  (persistence)  │         │    (pub/sub)            │   │
│  └─────────────────┘         └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
    ┌─────────┐          ┌─────────┐         ┌─────────┐
    │  AuthZ  │          │   NHI   │         │  Audit  │
    │(deliver)│          │ (keys)  │         │  (log)  │
    └─────────┘          └─────────┘         └─────────┘
```

### 5.2 Send Flow

```
Agent A wants to send to Agent B
         │
         ▼
┌─────────────────┐
│ Authorization   │
│ (can_deliver?)  │
└────────┬────────┘
         │ Allow
         ▼
┌─────────────────┐
│ Get B's Public  │◄───── NHI lookup
│ Keys            │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Generate Symkey │
│ + Encrypt       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Wrap Key with   │
│ ML-KEM-768      │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Sign Envelope   │◄───── NHI (Ed25519 + ML-DSA)
│                 │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Queue for       │
│ Delivery        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Deliver to B    │────────────────────────┐
│                 │                        │
└─────────────────┘                        ▼
                                     ┌─────────┐
                                     │  Audit  │
                                     └─────────┘
```

### 5.3 Receive Flow

```
Message arrives for Agent B
         │
         ▼
┌─────────────────┐
│ Verify          │
│ Signatures      │
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
 Valid    Invalid
    │         │
    │         ▼
    │      Reject
    │      + Audit
    │
    ▼
┌─────────────────┐
│ Unwrap Key      │◄───── NHI (ML-KEM decapsulate)
│ (ML-KEM)        │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Decrypt Payload │
│ (AES-256-GCM)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Deliver to      │
│ Receiver        │
└────────┬────────┘
         │
         ▼
    Wait for ACK
         │
         ▼
┌─────────────────┐
│ Mark Delivered  │
│ + Audit         │
└─────────────────┘
```

---

## 6. API Design

### 6.1 Sending Messages

```rust
impl MessagingService {
    /// Send message to agent
    pub async fn send(
        &self,
        to: &AgentIdentity,
        payload: &[u8],
        options: SendOptions,
    ) -> Result<MessageId, Error>;

    /// Send and wait for response
    pub async fn request(
        &self,
        to: &AgentIdentity,
        payload: &[u8],
        timeout: Duration,
    ) -> Result<Response, Error>;

    /// Publish to topic
    pub async fn publish(
        &self,
        topic: &TopicId,
        payload: &[u8],
        options: SendOptions,
    ) -> Result<MessageId, Error>;
}
```

### 6.2 Receiving Messages

```rust
impl MessagingService {
    /// Receive next message
    pub async fn receive(&self) -> Result<ReceivedMessage, Error>;

    /// Receive from specific topic
    pub async fn receive_from_topic(
        &self,
        topic: &TopicId,
    ) -> Result<ReceivedMessage, Error>;

    /// Acknowledge receipt
    pub async fn ack(&self, message_id: &MessageId) -> Result<(), Error>;

    /// Reply to message
    pub async fn reply(
        &self,
        to_message: &MessageId,
        payload: &[u8],
    ) -> Result<MessageId, Error>;
}
```

### 6.3 Topic Management

```rust
impl MessagingService {
    /// Create topic
    pub async fn create_topic(&self, config: TopicConfig) -> Result<TopicId, Error>;

    /// Subscribe to topic
    pub async fn subscribe(&self, topic: &TopicId) -> Result<Subscription, Error>;

    /// Unsubscribe
    pub async fn unsubscribe(&self, subscription: Subscription) -> Result<(), Error>;

    /// List subscriptions
    pub async fn list_subscriptions(&self) -> Result<Vec<Subscription>, Error>;
}
```

---

## 7. Integration Points

### 7.1 Authorization Integration

```rust
// Delivery authorization
let decision = authz.check(CheckRequest {
    agent: sender_nhi,
    action: "send_message",
    resource: format!("agent://{}", recipient_nhi),
}).await?;

// Topic publish authorization
let decision = authz.check(CheckRequest {
    agent: sender_nhi,
    action: "publish",
    resource: format!("topic://{}", topic_id),
}).await?;
```

### 7.2 NHI Integration

```rust
// Key lookup
let recipient_keys = nhi.get_public_keys(&recipient_nhi).await?;

// Signing delegation
let signature = nhi.sign(&sender_nhi, &data).await?;

// Decryption delegation
let symmetric_key = nhi.ml_kem_decapsulate(&recipient_nhi, &wrapped_key).await?;
```

### 7.3 Audit Integration

```rust
// Message sent
audit.log(AuditRecord {
    who: sender_nhi,
    what: "message_sent",
    resource: format!("message://{}", message_id),
    outcome: Outcome::Success,
    ..
}).await?;

// Message received
audit.log(AuditRecord {
    who: recipient_nhi,
    what: "message_received",
    resource: format!("message://{}", message_id),
    outcome: Outcome::Success,
    ..
}).await?;
```

---

## 8. Performance Requirements

| Metric | Target | Notes |
|--------|--------|-------|
| Message encryption | >100K msg/s | Per sender core |
| Key wrapping (ML-KEM) | <1ms | Per message |
| Signature generation | <2ms | Ed25519 + ML-DSA combined |
| End-to-end latency | <10ms p99 | Same datacenter |
| Delivery auth check | <1ms | AuthZ integration |
| Topic fanout | >10K subscribers/s | Per topic |

---

## 9. Security Considerations

### 9.1 Threat Model

| Threat | Mitigation |
|--------|------------|
| Eavesdropping | E2E encryption (AES-256-GCM) |
| MITM key substitution | NHI-verified public keys |
| Message forgery | Hybrid signatures (Ed25519 + ML-DSA) |
| Quantum attacks | ML-KEM-768, ML-DSA (post-quantum) |
| Replay attacks | Message IDs, timestamps, TTL |
| Unauthorized delivery | AuthZ-gated delivery |

### 9.2 Non-Repudiation

```rust
// Signatures provide non-repudiation
// Both classical (Ed25519) and post-quantum (ML-DSA) signatures
// are included for defense in depth

pub struct NonRepudiationProof {
    pub message_id: MessageId,
    pub sender_nhi: AgentIdentity,
    pub message_hash: Hash,
    pub signature_ed25519: [u8; 64],
    pub signature_ml_dsa: Vec<u8>,
    pub timestamp: Timestamp,
}

// Can be used as evidence that sender sent this message
impl NonRepudiationProof {
    pub fn verify(&self, sender_public_keys: &AgentKeys) -> Result<(), Error> {
        verify_ed25519(&self.signature_ed25519, &self.message_hash, &sender_public_keys.signing_public_ed25519)?;
        verify_ml_dsa(&self.signature_ml_dsa, &self.message_hash, &sender_public_keys.signing_public_ml_dsa)?;
        Ok(())
    }
}
```

---

## 10. Open Questions

1. Should we implement full Double Ratchet for per-message forward secrecy?
2. What's the message retention policy?
3. Should we support message recall/delete?
4. How do we handle offline agents (store-and-forward duration)?
5. Should topics support message ordering guarantees?

---

## 11. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
