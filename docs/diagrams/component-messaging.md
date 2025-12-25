# Messaging Product - Component Diagram

## Overview

This component diagram details the internal architecture of the Messaging product, which provides end-to-end encrypted communication between AI agents using the Double Ratchet algorithm (Signal Protocol). It shows key agreement (X3DH), ratchet state management, message encryption/decryption, and delivery routing.

## Purpose

- Detail components within the Messaging product container
- Show X3DH key agreement protocol for initial key exchange
- Illustrate Double Ratchet state machine for forward secrecy
- Visualize message encryption/decryption flow (ChaCha20-Poly1305)
- Document offline message queuing and delivery routing

## Diagram

```mermaid
graph TB
    subgraph External["External Systems"]
        SenderAgent[Sender Agent]
        RecipientAgent[Recipient Agent]
        Audit[Audit Logger]
    end

    subgraph APIGateway["API Gateway Layer"]
        LB[Load Balancer]
        Gateway[API Gateway<br/>Kong/Envoy]
    end

    subgraph MessagingProduct["Messaging Product Container"]

        subgraph KeyManagement["Key Agreement Layer"]
            KeyAgreementAPI[Key Agreement Service<br/>gRPC Service]
            X3DHProtocol[X3DH Protocol Handler<br/>Extended Triple Diffie-Hellman]
            PrekeyManager[Prekey Manager<br/>One-Time Prekey Generation]
            IdentityKeyStore[Identity Key Store<br/>Ed25519 Public Keys]
        end

        subgraph RatchetEngine["Double Ratchet Layer"]
            RatchetAPI[Ratchet Engine API<br/>gRPC Service]
            StateMachine[Ratchet State Machine<br/>Send/Receive Chain Keys]
            KDFChain[KDF Chain<br/>HKDF-SHA256]
            MessageKeyCache[Message Key Cache<br/>Out-of-Order Delivery]
        end

        subgraph EncryptionLayer["Encryption/Decryption Layer"]
            EnvelopeAPI[Envelope Processor API<br/>gRPC Service]
            Encryptor[Message Encryptor<br/>ChaCha20-Poly1305]
            Decryptor[Message Decryptor<br/>ChaCha20-Poly1305]
            NonceGenerator[Nonce Generator<br/>96-bit Random]
        end

        subgraph DeliveryRouting["Delivery Layer"]
            DeliveryAPI[Delivery Router API<br/>gRPC Service]
            MessageQueue[Message Queue<br/>Offline Message Storage]
            DeliveryAckTracker[Delivery Ack Tracker<br/>Receipt Tracking]
            PushNotifier[Push Notifier<br/>FCM/APNs Integration]
        end

        subgraph DataStores["Data Stores"]
            PostgresDB[(PostgreSQL<br/>messaging_prekeys<br/>messaging_sessions<br/>messaging_queue)]
            RedisCache[(Redis Cluster<br/>Ratchet State Cache<br/>Message Key Cache)]
            KafkaTopic[(Kafka Topic<br/>message.delivered.v1<br/>30-day retention)]
        end
    end

    %% External → API Gateway
    SenderAgent -->|POST /messaging/prekeys/{recipient}<br/>Fetch Prekey Bundle| LB
    SenderAgent -->|POST /messaging/send<br/>Encrypted Message| LB
    RecipientAgent -->|GET /messaging/receive<br/>Poll Messages| LB
    RecipientAgent -->|POST /messaging/upload-prekeys<br/>Publish Prekeys| LB
    LB -->|Route| Gateway

    %% API Gateway → Messaging Components
    Gateway -->|gRPC: GetPrekeyBundle| KeyAgreementAPI
    Gateway -->|gRPC: UploadPrekeys| KeyAgreementAPI
    Gateway -->|gRPC: SendMessage| EnvelopeAPI
    Gateway -->|gRPC: ReceiveMessages| DeliveryAPI

    %% Key Agreement Flow (X3DH)
    KeyAgreementAPI -->|1. Fetch Recipient Prekey Bundle| PrekeyManager
    PrekeyManager -->|Query Prekeys| PostgresDB
    PrekeyManager -->|Load Identity Key| IdentityKeyStore
    IdentityKeyStore -->|Read Public Key| PostgresDB

    X3DHProtocol -->|2. Perform X3DH| PrekeyManager
    X3DHProtocol -->|3. Derive Shared Secret<br/>ECDH(IK_sender, SPK_recipient)| KDFChain
    X3DHProtocol -->|4. Initialize Ratchet State| RatchetAPI

    %% Upload Prekeys
    KeyAgreementAPI -->|Upload Prekeys| PrekeyManager
    PrekeyManager -->|Store Prekeys| PostgresDB
    PrekeyManager -->|Cache Active Prekeys| RedisCache

    %% Message Sending Flow
    EnvelopeAPI -->|1. Get Ratchet State| RatchetAPI
    RatchetAPI -->|Load from Cache| RedisCache
    RedisCache -->|Cache Miss| PostgresDB

    RatchetAPI -->|2. Advance Send Chain| StateMachine
    StateMachine -->|3. Derive Message Key| KDFChain
    KDFChain -->|HKDF-SHA256| MessageKeyCache

    EnvelopeAPI -->|4. Encrypt Message| Encryptor
    NonceGenerator -->|Generate Nonce| Encryptor
    Encryptor -->|ChaCha20-Poly1305| EnvelopeAPI

    EnvelopeAPI -->|5. Persist Ratchet State| RatchetAPI
    RatchetAPI -->|Update State| PostgresDB
    RatchetAPI -->|Update Cache| RedisCache

    EnvelopeAPI -->|6. Route to Delivery| DeliveryAPI

    %% Message Delivery Flow
    DeliveryAPI -->|7. Check Recipient Online| MessageQueue
    MessageQueue -->|Online → Direct Deliver| RecipientAgent
    MessageQueue -->|Offline → Queue| PostgresDB
    MessageQueue -->|Trigger Push Notification| PushNotifier

    PushNotifier -->|FCM/APNs| RecipientAgent

    DeliveryAPI -->|8. Track Delivery| DeliveryAckTracker
    DeliveryAckTracker -->|Publish Event| KafkaTopic

    %% Message Receiving Flow
    RecipientAgent -->|Poll Messages| DeliveryAPI
    DeliveryAPI -->|Fetch Queued Messages| PostgresDB
    DeliveryAPI -->|Return Encrypted Envelope| RecipientAgent

    RecipientAgent -->|Decrypt Locally| Decryptor
    Decryptor -->|Get Ratchet State| RatchetAPI
    RatchetAPI -->|Advance Receive Chain| StateMachine
    StateMachine -->|Derive Message Key| KDFChain
    Decryptor -->|ChaCha20-Poly1305 Decrypt| RecipientAgent

    RecipientAgent -->|Acknowledge Receipt| DeliveryAPI
    DeliveryAPI -->|Delete from Queue| PostgresDB
    DeliveryAPI -->|Update Ack Tracker| DeliveryAckTracker

    %% Out-of-Order Delivery
    StateMachine -->|Cache Skipped Keys| MessageKeyCache
    MessageKeyCache -->|Store in Redis| RedisCache
    MessageKeyCache -->|Max 100 Keys| RedisCache

    %% Audit Logging
    KeyAgreementAPI -.->|Log: Key Exchange| Audit
    EnvelopeAPI -.->|Log: Message Sent (Metadata Only)| Audit
    DeliveryAPI -.->|Log: Message Delivered| Audit

    %% Cross-Product Integration
    KeyAgreementAPI -.->|Fetch Agent Identity| Gateway
    EnvelopeAPI -.->|Report Message Volume to Metering| Gateway

    classDef api fill:#e1f5ff,stroke:#0066cc,stroke-width:2px
    classDef cache fill:#fff4e1,stroke:#ff9900,stroke-width:2px
    classDef db fill:#f0f0f0,stroke:#333,stroke-width:2px
    classDef processor fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px
    classDef crypto fill:#fff3e0,stroke:#ef6c00,stroke-width:2px

    class KeyAgreementAPI,EnvelopeAPI,DeliveryAPI api
    class RedisCache,MessageKeyCache cache
    class PostgresDB,KafkaTopic db
    class StateMachine,KDFChain processor
    class Encryptor,Decryptor,X3DHProtocol crypto
```

## Component Inventory

### Key Agreement Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Key Agreement Service** | Rust/gRPC | Orchestrates X3DH protocol, manages prekey bundles |
| **X3DH Protocol Handler** | libsignal-protocol | Implements Extended Triple Diffie-Hellman key exchange |
| **Prekey Manager** | PostgreSQL Client | Generates, stores, and rotates one-time prekeys |
| **Identity Key Store** | PostgreSQL Client | Stores agents' long-term Ed25519 identity keys |

**X3DH Protocol:**
- **IK (Identity Key)**: Long-term Ed25519 keypair per agent
- **SPK (Signed Prekey)**: Medium-term key rotated weekly
- **OPK (One-Time Prekey)**: Single-use keys for each session
- **Ephemeral Key**: Temporary key for each key exchange

### Double Ratchet Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Ratchet Engine API** | Rust/gRPC | Manages Double Ratchet state for active sessions |
| **Ratchet State Machine** | libsignal-protocol | Advances send/receive chains, derives keys |
| **KDF Chain** | HKDF-SHA256 | Key derivation using HMAC-based Extract-and-Expand |
| **Message Key Cache** | Redis (LRU) | Caches out-of-order message keys (max 100 per session) |

**Ratchet State:**
```rust
struct RatchetState {
    root_key: [u8; 32],           // 256-bit root key
    send_chain_key: [u8; 32],     // Send chain key
    receive_chain_key: [u8; 32],  // Receive chain key
    send_counter: u32,            // Message number for send chain
    receive_counter: u32,         // Message number for receive chain
    dh_keypair: DHKeyPair,        // Current DH ratchet keypair
    dh_remote_public: [u8; 32],   // Remote DH public key
    skipped_keys: HashMap<(u32, u32), [u8; 32]>,  // Out-of-order keys
}
```

### Encryption/Decryption Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Envelope Processor API** | Rust/gRPC | Encrypts outgoing messages, decrypts incoming messages |
| **Message Encryptor** | ChaCha20-Poly1305 | AEAD encryption (256-bit key, 96-bit nonce) |
| **Message Decryptor** | ChaCha20-Poly1305 | AEAD decryption with authentication tag verification |
| **Nonce Generator** | CSPRNG | Generates cryptographically secure 96-bit nonces |

**Message Envelope:**
```protobuf
message EncryptedEnvelope {
  string sender_id = 1;           // Sender agent public key hash
  string recipient_id = 2;        // Recipient agent public key hash
  bytes ciphertext = 3;           // ChaCha20-Poly1305 encrypted payload
  bytes nonce = 4;                // 96-bit nonce
  bytes auth_tag = 5;             // 128-bit Poly1305 authentication tag
  uint32 send_counter = 6;        // Ratchet send counter
  bytes dh_public_key = 7;        // Sender's current DH public key
  google.protobuf.Timestamp timestamp = 8;
}
```

### Delivery Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Delivery Router API** | Rust/gRPC | Routes messages to online/offline recipients |
| **Message Queue** | PostgreSQL + Redis | Stores offline messages until recipient polls |
| **Delivery Ack Tracker** | Kafka Producer | Tracks message delivery confirmations |
| **Push Notifier** | FCM/APNs Client | Sends push notifications to wake up offline agents |

**Delivery Strategies:**
- **Online recipient**: Direct gRPC streaming delivery
- **Offline recipient**: Queue in PostgreSQL, send push notification
- **Max queue size**: 10,000 messages per recipient
- **TTL**: 30 days (auto-delete after expiration)

## Data Flow Details

### Flow 1: Initial Key Exchange (X3DH)

```
Sender Agent wants to initiate conversation with Recipient Agent
  ↓
1. Sender → Key Agreement API: GET /messaging/prekeys/{recipient_id}
  ↓
Key Agreement Service:
  - Query Prekey Manager for recipient's prekey bundle
  - Prekey Manager → PostgreSQL: Fetch identity key, signed prekey, one-time prekey
  - Mark one-time prekey as USED (single-use)
  - Return bundle: {IK_recipient, SPK_recipient, OPK_recipient, SPK_signature}
  ↓
2. Sender (local computation):
  - Generate ephemeral keypair: EK_sender
  - Perform X3DH:
    DH1 = ECDH(IK_sender, SPK_recipient)
    DH2 = ECDH(EK_sender, IK_recipient)
    DH3 = ECDH(EK_sender, SPK_recipient)
    DH4 = ECDH(EK_sender, OPK_recipient)  // If OPK available
  - Derive shared secret:
    shared_secret = KDF(DH1 || DH2 || DH3 || DH4)
  - Initialize Double Ratchet state:
    (root_key, send_chain_key) = HKDF(shared_secret)
  ↓
3. Sender → Ratchet Engine API: InitializeSession
  - Persist ratchet state to PostgreSQL
  - Cache state in Redis (TTL: 1 hour)
  ↓
Now ready to send encrypted messages
```

**Database Schema (Prekeys):**
```sql
CREATE TABLE messaging_prekeys (
  prekey_id UUID PRIMARY KEY,
  agent_id TEXT NOT NULL,
  key_type TEXT NOT NULL,  -- 'identity', 'signed_prekey', 'one_time_prekey'
  public_key BYTEA NOT NULL,
  private_key_encrypted BYTEA,  -- Encrypted with agent's master key
  signature BYTEA,              -- SPK signature by IK
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  used_at TIMESTAMPTZ,          -- NULL for unused OPKs
  INDEX idx_agent_key_type (agent_id, key_type, used_at)
);
```

### Flow 2: Send Encrypted Message

```
Sender Agent → Envelope Processor API: SendMessage
  ↓
1. Envelope Processor → Ratchet Engine: GetRatchetState(session_id)
  - Ratchet Engine → Redis: HGET ratchet:{session_id}
  - If miss, load from PostgreSQL
  ↓
2. Ratchet State Machine: Advance Send Chain
  - Derive message key:
    (send_chain_key, message_key) = HKDF(send_chain_key, "message_key")
  - Increment send_counter
  ↓
3. Message Encryptor:
  - Generate nonce: nonce = CSPRNG(96 bits)
  - Encrypt plaintext:
    (ciphertext, auth_tag) = ChaCha20-Poly1305.Encrypt(
      key=message_key,
      nonce=nonce,
      plaintext=message,
      associated_data=(sender_id || recipient_id || send_counter)
    )
  ↓
4. Build Envelope:
  envelope = {
    sender_id, recipient_id, ciphertext, nonce, auth_tag,
    send_counter, dh_public_key, timestamp
  }
  ↓
5. Ratchet Engine: Persist Updated State
  - PostgreSQL: UPDATE messaging_sessions SET ratchet_state=...
  - Redis: HSET ratchet:{session_id} (overwrite cache)
  ↓
6. Envelope Processor → Delivery Router: RouteMessage(envelope)
  ↓
Delivery Router:
  - Check if recipient online (WebSocket connection active)
  - If online: Stream envelope directly via gRPC
  - If offline:
    - PostgreSQL: INSERT INTO messaging_queue
    - Push Notifier → FCM/APNs: Send notification
  ↓
7. Delivery Ack Tracker:
  - Kafka: Publish message.delivered.v1 event
  - Audit Logger: Log metadata (sender_id, recipient_id, timestamp, size)
```

### Flow 3: Receive and Decrypt Message

```
Recipient Agent → Delivery Router API: ReceiveMessages
  ↓
1. Delivery Router → PostgreSQL:
  SELECT * FROM messaging_queue
  WHERE recipient_id='agent_xyz789' AND delivered_at IS NULL
  ORDER BY created_at ASC
  LIMIT 100
  ↓
2. Return encrypted envelopes to recipient
  ↓
Recipient Agent (local computation):
  For each envelope:
    3. Ratchet Engine API: GetRatchetState(session_id)
    4. Ratchet State Machine: Advance Receive Chain
      - Check if send_counter matches receive_counter
      - If mismatch (out-of-order): Cache skipped keys in Redis
      - Derive message key:
        (receive_chain_key, message_key) = HKDF(receive_chain_key, "message_key")
      - Increment receive_counter
    5. Message Decryptor:
      plaintext = ChaCha20-Poly1305.Decrypt(
        key=message_key,
        nonce=envelope.nonce,
        ciphertext=envelope.ciphertext,
        associated_data=(sender_id || recipient_id || send_counter)
      )
      - Verify auth_tag (throws if tampered)
    6. Ratchet Engine: Persist Updated State
  ↓
Recipient Agent → Delivery Router API: AcknowledgeReceipt(message_ids)
  ↓
Delivery Router:
  - PostgreSQL: UPDATE messaging_queue SET delivered_at=NOW()
  - Kafka: Publish message.acknowledged.v1 event
```

### Flow 4: Out-of-Order Message Handling

```
Scenario: Recipient receives messages with send_counter [1, 3, 2]
  ↓
Message 1 (send_counter=1):
  - receive_counter=1 (matches)
  - Derive message_key_1, decrypt, increment receive_counter to 2
  ↓
Message 3 (send_counter=3):
  - receive_counter=2 (mismatch! Expected 2, got 3)
  - Derive and cache skipped key:
    message_key_2 = HKDF(receive_chain_key, "message_key")
    Cache in Redis: HSET skipped_keys:{session_id} 2 message_key_2
  - Advance to send_counter=3:
    message_key_3 = HKDF(receive_chain_key, "message_key")
    receive_counter = 3
  - Decrypt message 3 with message_key_3
  ↓
Message 2 (send_counter=2):
  - receive_counter=3 (already past this)
  - Check skipped keys cache:
    message_key_2 = HGET skipped_keys:{session_id} 2
  - Decrypt message 2 with cached message_key_2
  - Delete from cache: HDEL skipped_keys:{session_id} 2
```

**Max Skipped Keys:**
- Redis cache limit: 100 skipped keys per session
- Eviction policy: If > 100, delete oldest skipped keys
- Prevents DoS attack (malicious sender flooding with high counters)

### Flow 5: Prekey Rotation

```
Background job (cron every 7 days):
  ↓
For each agent:
  1. Generate new signed prekey (SPK):
    (spk_private, spk_public) = Ed25519.GenerateKeypair()
    signature = Ed25519.Sign(identity_key_private, spk_public)
  2. Upload to Prekey Manager:
    - PostgreSQL: INSERT INTO messaging_prekeys (SPK)
    - Mark old SPK as DEPRECATED (keep for 30 days for in-flight sessions)
  3. Generate 100 new one-time prekeys (OPKs):
    For i in 0..100:
      (opk_private, opk_public) = X25519.GenerateKeypair()
      PostgreSQL: INSERT INTO messaging_prekeys (OPK)
  4. Delete exhausted OPKs (used_at IS NOT NULL AND created_at < NOW() - 30 days)
```

## Implementation Considerations

### Performance Optimization

**Ratchet State Caching:**
- **Redis cache**: Hot sessions cached with 1-hour TTL
- **Cache hit rate**: Target 95% (most sessions active within 1 hour)
- **Cache key pattern**: `ratchet:{sender_id}:{recipient_id}`

**Prekey Bundle Caching:**
- **Redis cache**: Active prekey bundles cached for 5 minutes
- **Cache invalidation**: On prekey rotation, flush affected keys
- **Pre-warming**: Background job pre-caches top 1000 agents' bundles

**Message Key Derivation:**
- **HKDF optimization**: Use SIMD-accelerated HMAC-SHA256
- **Batch derivation**: Derive multiple message keys in parallel for out-of-order handling

### Scalability

**Horizontal Scaling:**
- Key Agreement Service: 4 replicas (stateless)
- Ratchet Engine: 6 replicas (stateless, state in Redis/PostgreSQL)
- Envelope Processor: 8 replicas (CPU-bound encryption)
- Delivery Router: 10 replicas (I/O-bound database queries)

**Database Sharding:**
- **Prekeys table**: Partition by `agent_id` hash (16 shards)
- **Sessions table**: Partition by `(sender_id, recipient_id)` hash (32 shards)
- **Message queue**: Partition by `recipient_id` hash (64 shards)

**Message Queue Limits:**
- **Max queue size per recipient**: 10,000 messages
- **TTL**: 30 days (auto-delete)
- **Overflow policy**: FIFO eviction (delete oldest messages)

### Resilience

**Ratchet State Recovery:**
- On cache miss, load from PostgreSQL
- On database failure, return error (cannot derive keys without state)
- Circuit breaker: If PostgreSQL down, queue writes for retry

**Delivery Retries:**
- Push notifications: 3 retries (exponential backoff: 10s, 30s, 90s)
- On failure, mark message as UNDELIVERABLE
- Human intervention: Ops can manually retry or delete

**Key Rotation Failures:**
- If SPK rotation fails, keep old SPK active
- Alert on-call: "Prekey rotation failed for agent_xyz789"
- Manual intervention: Re-run rotation job

### Security

**Forward Secrecy:**
- **Double Ratchet**: New message key for every message
- **Key deletion**: Message keys deleted after use (not stored)
- **Compromise recovery**: If DH ratchet key compromised, new key negotiated on next message

**Post-Compromise Security:**
- **DH ratchet**: Periodic DH key rotation (every 100 messages or 1 hour)
- **Healing property**: Attacker who steals old keys cannot decrypt future messages

**Metadata Protection:**
- **Audit logs**: Only metadata logged (sender_id, recipient_id, timestamp, size)
- **No plaintext**: Message content never logged or stored unencrypted
- **Sealed sender (future)**: Hide sender identity from server

**Key Storage:**
- **Private keys encrypted**: Agent private keys encrypted with agent's master key
- **Hardware security**: Master keys stored in HSM (future)

### Monitoring & Alerting

**Key Metrics:**
- `messaging_prekey_bundles_fetched_total` (counter)
- `messaging_messages_sent_total` (counter)
- `messaging_messages_queued{state="online|offline"}` (counter)
- `messaging_ratchet_state_cache_hit_ratio` (gauge)
- `messaging_encryption_duration_seconds` (histogram)
- `messaging_delivery_latency_seconds` (histogram)

**Alerts:**
- Ratchet state cache hit rate < 90%: Warning
- Encryption p99 > 10ms: Warning
- Delivery failure rate > 5%: Critical
- Prekey exhaustion: Page on-call (agent has < 10 OPKs remaining)

## Integration Points

### Cross-Product Dependencies

**Messaging → NHI (Platform Layer):**
- Fetch agent identity keys (Ed25519 public keys)
- Validate agent ownership before prekey upload

**Messaging → AuthZ (Security Layer):**
- Validate delegation chains for API requests
- Ensure sender is authorized to message recipient

**Messaging → Metering:**
- Report message volume: Number of messages sent, total bytes
- Report storage usage: Queued message count, bytes

**Messaging → Audit:**
- Log key exchange events (X3DH initiated)
- Log message delivery metadata (no plaintext)
- Log prekey rotation events

### External Integrations

**Push Notifications:**
- **FCM (Firebase Cloud Messaging)**: Android, Web
- **APNs (Apple Push Notification service)**: iOS, macOS
- Payload: Silent notification to wake app, no message content

**Kafka (Audit Trail):**
- Topic: `message.delivered.v1`
- Retention: 30 days
- Use case: Forensics, compliance audits

## Related Diagrams

- [C4 Context Diagram](./c4-context.md) - System-level context
- [C4 Container Diagram](./c4-container.md) - Container-level architecture
- [Data Flow Matrix](./data-flow-matrix.md) - Cross-product data flows

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-25 | Initial Messaging component diagram for Issue #64 |
