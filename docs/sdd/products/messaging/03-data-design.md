---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# Messaging Data Design

## Table of Contents

1. [Storage Strategy](#storage-strategy)
2. [Database Schemas](#database-schemas)
3. [Data Models](#data-models)
4. [Encryption at Rest](#encryption-at-rest)
5. [Retention Policies](#retention-policies)
6. [Data Flow](#data-flow)
7. [Migration Strategy](#migration-strategy)
8. [Performance Optimization](#performance-optimization)

---

## Storage Strategy

### Core Principles

1. **Never store plaintext message content**
   - Only encrypted envelopes stored
   - Plaintext never persisted to disk
   - Decryption happens in-memory only

2. **Minimize storage duration**
   - Messages deleted after delivery acknowledgment
   - Retention only for undelivered messages (up to TTL)
   - Audit logs separate from message storage

3. **Partition for scale**
   - Shard by recipient NHI hash
   - Separate hot (active) and cold (archived) data
   - Use consistent hashing for partition assignment

4. **Encrypt everything at rest**
   - Database-level encryption (TDE)
   - Application-level encryption for sensitive fields
   - Hardware security modules (HSM) for keys

### Storage Layers

```
┌─────────────────────────────────────────────────────────────┐
│            HOT STORAGE (Fast, In-Memory)                    │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Redis Cluster                                          │ │
│  │  - Prekey bundles (public keys)                        │ │
│  │  - Active session states                               │ │
│  │  - Pending message envelopes (queue)                   │ │
│  │  TTL: 7 days max                                       │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│           WARM STORAGE (Persistent, ACID)                   │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ PostgreSQL (Primary)                                   │ │
│  │  - Ratchet state (encrypted)                           │ │
│  │  - Channel metadata                                    │ │
│  │  - Delivery receipts                                   │ │
│  │  - Key rotation history                                │ │
│  │  Retention: 30 days                                    │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│           COLD STORAGE (Archived, Immutable)                │
│  ┌────────────────────────────────────────────────────────┐ │
│  │ Object Storage (S3)                                    │ │
│  │  - Audit logs (immutable)                              │ │
│  │  - Compliance archives                                 │ │
│  │  - Historical analytics                                │ │
│  │  Retention: 7 years (compliance)                       │ │
│  └────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

---

## Database Schemas

### 1. Message Envelopes (Hot Storage - Redis)

**Purpose:** Store encrypted message envelopes awaiting delivery.

**Data structure:**
```rust
// Redis key: "envelope:{recipient_nhi}:{message_id}"
// Redis type: Hash
// TTL: envelope.ttl_seconds or 604800 (7 days)

pub struct MessageEnvelope {
    // Identity
    pub message_id: Uuid,              // UUID v4
    pub sender_nhi: String,            // Sender's NHI (agent-123)
    pub recipient_nhi: String,         // Recipient's NHI
    pub channel_id: Option<Uuid>,      // For topic-based messaging

    // Timestamps
    pub created_at: i64,               // Unix timestamp (milliseconds)
    pub expires_at: i64,               // created_at + ttl_seconds

    // Encrypted payload
    pub encrypted_payload: Vec<u8>,    // AES-256-GCM(plaintext)
    pub nonce: [u8; 12],               // AES-GCM nonce
    pub auth_tag: [u8; 16],            // GCM authentication tag

    // Key encapsulation
    pub wrapped_key: Vec<u8>,          // ML-KEM-768(AES key) ~1568 bytes
    pub key_id: Uuid,                  // Recipient's public key ID

    // Signatures
    pub signature_ed25519: [u8; 64],   // Ed25519 signature
    pub signature_ml_dsa: Vec<u8>,     // ML-DSA-65 signature ~3293 bytes

    // Metadata
    pub priority: u8,                  // 0=Low, 1=Normal, 2=High, 3=Critical
    pub ttl_seconds: u32,              // Time-to-live
    pub correlation_id: Option<Uuid>,  // For request/response
    pub compression: u8,               // 0=None, 1=Gzip, 2=Zstd
}
```

**Redis storage format:**
```redis
HSET envelope:{recipient_nhi}:{message_id}
    message_id          "550e8400-e29b-41d4-a716-446655440000"
    sender_nhi          "agent-alice-01"
    recipient_nhi       "agent-bob-02"
    created_at          "1735171200000"
    expires_at          "1735776000000"
    encrypted_payload   <binary blob>
    nonce               <12 bytes>
    auth_tag            <16 bytes>
    wrapped_key         <1568 bytes>
    key_id              "7c9e6679-7425-40de-944b-e07fc1f90ae7"
    signature_ed25519   <64 bytes>
    signature_ml_dsa    <3293 bytes>
    priority            "1"
    ttl_seconds         "604800"
    correlation_id      NULL
    compression         "0"

EXPIRE envelope:{recipient_nhi}:{message_id} 604800
```

**Indexing:**
```redis
# Priority queue (sorted set)
# Redis key: "queue:{recipient_nhi}:priority:{priority}"
# Score: created_at (for FIFO within priority)
ZADD queue:agent-bob-02:priority:3 1735171200000 "550e8400-e29b-41d4-a716-446655440000"
```

**Partitioning:**
```rust
fn partition_key(recipient_nhi: &str) -> u32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    recipient_nhi.hash(&mut hasher);
    (hasher.finish() % 256) as u32  // 256 partitions
}

// Redis cluster: 256 nodes, each handles hash slot range
// Node 0: slots 0-63    → recipients hash % 256 in [0, 63]
// Node 1: slots 64-127  → recipients hash % 256 in [64, 127]
// ...
```

### 2. Key Bundles (Hot Storage - Redis)

**Purpose:** Store agent public keys for encryption.

**Data structure:**
```rust
pub struct KeyBundle {
    pub agent_nhi: String,
    pub key_id: Uuid,
    pub key_type: KeyType,  // MlKem768, Ed25519, MlDsa65
    pub public_key: Vec<u8>,
    pub created_at: i64,
    pub expires_at: Option<i64>,  // NULL for long-lived keys
    pub status: KeyStatus,  // Active, Grace, Revoked
}

pub enum KeyType {
    MlKem768,    // 1184 bytes
    Ed25519,     // 32 bytes
    MlDsa65,     // ~1952 bytes
}

pub enum KeyStatus {
    Active,      // Current key
    Grace,       // Old key in grace period (7 days)
    Revoked,     // Compromised, do not use
}
```

**Redis storage:**
```redis
# Active key
HSET key:agent-alice-01:ml_kem_768:active
    key_id          "7c9e6679-7425-40de-944b-e07fc1f90ae7"
    public_key      <1184 bytes>
    created_at      "1735171200000"
    expires_at      "1767707200000"  # 1 year
    status          "active"

# Grace period keys (list)
LPUSH key:agent-alice-01:ml_kem_768:grace
    "old-key-id-1:public_key_bytes:expires_at"
    "old-key-id-2:public_key_bytes:expires_at"

# Set TTL on grace list
EXPIRE key:agent-alice-01:ml_kem_768:grace 604800  # 7 days
```

**Cache invalidation:**
```rust
// When key rotates, invalidate cache
pub async fn invalidate_key_cache(agent_nhi: &str) -> Result<()> {
    redis_client.del(format!("key:{}:ml_kem_768:active", agent_nhi)).await?;
    redis_client.del(format!("key:{}:ed25519:active", agent_nhi)).await?;
    redis_client.del(format!("key:{}:ml_dsa_65:active", agent_nhi)).await?;
    Ok(())
}
```

### 3. Ratchet State (Warm Storage - PostgreSQL)

**Purpose:** Store session key rotation state (encrypted at rest).

**Schema:**
```sql
CREATE TABLE ratchet_states (
    session_id              UUID PRIMARY KEY,
    agent_a_nhi             VARCHAR(255) NOT NULL,
    agent_b_nhi             VARCHAR(255) NOT NULL,

    -- Current encryption state
    current_key_id          UUID NOT NULL,
    current_key_version     INTEGER NOT NULL DEFAULT 1,

    -- Rotation schedule
    rotation_policy         VARCHAR(50) NOT NULL,  -- 'hourly', 'daily', 'weekly', 'on_demand'
    rotation_interval_secs  INTEGER,
    last_rotated_at         TIMESTAMP WITH TIME ZONE NOT NULL,
    next_rotation_at        TIMESTAMP WITH TIME ZONE,

    -- Grace period keys (JSON encrypted)
    grace_period_keys       JSONB,  -- [{key_id, expires_at}, ...]

    -- Metadata
    created_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Partitioning
    partition_id            INTEGER NOT NULL,  -- hash(agent_a_nhi) % 256

    -- Indexes
    INDEX idx_ratchet_next_rotation (next_rotation_at) WHERE next_rotation_at IS NOT NULL,
    INDEX idx_ratchet_partition (partition_id, agent_a_nhi),
    CONSTRAINT unique_session UNIQUE (agent_a_nhi, agent_b_nhi)
);

-- Table partitioning by partition_id
CREATE TABLE ratchet_states_p0 PARTITION OF ratchet_states
    FOR VALUES FROM (0) TO (64);
CREATE TABLE ratchet_states_p1 PARTITION OF ratchet_states
    FOR VALUES FROM (64) TO (128);
-- ... (4 partitions total)

-- Encrypt grace_period_keys at application level
-- Structure: [{key_id: UUID, expires_at: timestamp}, ...]
```

**Example row:**
```sql
INSERT INTO ratchet_states (
    session_id,
    agent_a_nhi,
    agent_b_nhi,
    current_key_id,
    current_key_version,
    rotation_policy,
    rotation_interval_secs,
    last_rotated_at,
    next_rotation_at,
    grace_period_keys,
    partition_id
) VALUES (
    '550e8400-e29b-41d4-a716-446655440000',
    'agent-alice-01',
    'agent-bob-02',
    '7c9e6679-7425-40de-944b-e07fc1f90ae7',
    5,  -- 5th rotation
    'daily',
    86400,  -- 24 hours
    '2025-12-25 10:00:00+00',
    '2025-12-26 10:00:00+00',
    '[
        {"key_id": "old-key-1", "expires_at": "2025-12-27 10:00:00+00"},
        {"key_id": "old-key-2", "expires_at": "2025-12-28 10:00:00+00"}
    ]'::jsonb,
    42  -- hash("agent-alice-01") % 256
);
```

**Encryption at rest:**
```rust
// Application-level encryption for grace_period_keys
pub async fn store_ratchet_state(state: &RatchetState) -> Result<()> {
    let grace_keys_json = serde_json::to_string(&state.grace_period_keys)?;

    // Encrypt with database encryption key (from HSM)
    let encrypted_grace_keys = encrypt_field(&grace_keys_json).await?;

    sqlx::query!(
        "INSERT INTO ratchet_states (session_id, grace_period_keys, ...)
         VALUES ($1, $2, ...)",
        state.session_id,
        encrypted_grace_keys,
        // ...
    ).execute(&db_pool).await?;

    Ok(())
}
```

### 4. Channel Metadata (Warm Storage - PostgreSQL)

**Purpose:** Store topic/channel configuration and subscriptions.

**Schema:**
```sql
CREATE TABLE channels (
    channel_id              UUID PRIMARY KEY,
    channel_name            VARCHAR(255) NOT NULL,
    owner_nhi               VARCHAR(255) NOT NULL,

    -- Policy
    policy_type             VARCHAR(50) NOT NULL,  -- 'open', 'private', 'authz_required', 'allowlist'
    allowlist_nhis          TEXT[],  -- For allowlist policy

    -- Retention
    retention_seconds       INTEGER NOT NULL DEFAULT 604800,  -- 7 days
    max_message_size_bytes  INTEGER NOT NULL DEFAULT 1048576,  -- 1MB
    max_subscribers         INTEGER NOT NULL DEFAULT 10000,

    -- Statistics
    total_messages          BIGINT NOT NULL DEFAULT 0,
    total_subscribers       INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    created_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    INDEX idx_channel_owner (owner_nhi),
    INDEX idx_channel_name (channel_name),
    CONSTRAINT unique_channel_name_owner UNIQUE (owner_nhi, channel_name)
);

CREATE TABLE channel_subscriptions (
    subscription_id         UUID PRIMARY KEY,
    channel_id              UUID NOT NULL REFERENCES channels(channel_id) ON DELETE CASCADE,
    subscriber_nhi          VARCHAR(255) NOT NULL,

    -- Subscription state
    status                  VARCHAR(50) NOT NULL DEFAULT 'active',  -- 'active', 'paused', 'deleted'
    offset                  BIGINT NOT NULL DEFAULT 0,  -- Last consumed message offset

    -- Timestamps
    subscribed_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_consumed_at        TIMESTAMP WITH TIME ZONE,

    INDEX idx_subscription_channel (channel_id),
    INDEX idx_subscription_subscriber (subscriber_nhi),
    CONSTRAINT unique_subscription UNIQUE (channel_id, subscriber_nhi)
);
```

**Example:**
```sql
-- Create channel
INSERT INTO channels (channel_id, channel_name, owner_nhi, policy_type)
VALUES (
    'c9a3a2e0-1234-5678-9abc-def012345678',
    'swarm-coordination',
    'agent-coordinator-01',
    'authz_required'
);

-- Subscribe
INSERT INTO channel_subscriptions (subscription_id, channel_id, subscriber_nhi)
VALUES (
    'd1b4c3f1-2345-6789-abcd-ef0123456789',
    'c9a3a2e0-1234-5678-9abc-def012345678',
    'agent-worker-01'
);
```

### 5. Delivery Receipts (Warm Storage - PostgreSQL)

**Purpose:** Track message delivery acknowledgments for audit.

**Schema:**
```sql
CREATE TABLE delivery_receipts (
    receipt_id              UUID PRIMARY KEY,
    message_id              UUID NOT NULL,
    sender_nhi              VARCHAR(255) NOT NULL,
    recipient_nhi           VARCHAR(255) NOT NULL,
    channel_id              UUID,

    -- Delivery status
    status                  VARCHAR(50) NOT NULL,  -- 'pending', 'delivered', 'expired', 'failed'
    delivery_attempts       INTEGER NOT NULL DEFAULT 0,
    last_attempt_at         TIMESTAMP WITH TIME ZONE,

    -- Timestamps
    sent_at                 TIMESTAMP WITH TIME ZONE NOT NULL,
    delivered_at            TIMESTAMP WITH TIME ZONE,
    acknowledged_at         TIMESTAMP WITH TIME ZONE,

    -- Failure details
    failure_reason          TEXT,

    -- Partitioning
    partition_id            INTEGER NOT NULL,  -- hash(recipient_nhi) % 256

    INDEX idx_receipt_message (message_id),
    INDEX idx_receipt_recipient (recipient_nhi, sent_at),
    INDEX idx_receipt_status (status, sent_at)
);

-- Partition by month (time-series data)
CREATE TABLE delivery_receipts_2025_12 PARTITION OF delivery_receipts
    FOR VALUES FROM ('2025-12-01') TO ('2026-01-01');
-- Auto-create partitions via cron job
```

**Lifecycle:**
```
1. Message sent → INSERT (status='pending')
2. Message delivered → UPDATE (status='delivered', delivered_at=NOW())
3. Acknowledgment received → UPDATE (acknowledged_at=NOW())
4. Retention expires (30 days) → Archive to S3, DELETE from Postgres
```

---

## Data Models

### Rust Data Structures

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub version: u8,
    pub message_id: Uuid,
    pub sender_nhi: String,
    pub recipient_nhi: String,
    pub channel_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,

    // Encrypted content
    pub encrypted_payload: Vec<u8>,
    pub nonce: [u8; 12],
    pub auth_tag: [u8; 16],

    // Key encapsulation
    pub wrapped_key: Vec<u8>,
    pub key_id: Uuid,

    // Signatures
    pub signature_ed25519: [u8; 64],
    pub signature_ml_dsa: Vec<u8>,

    // Metadata
    pub priority: MessagePriority,
    pub ttl_seconds: u32,
    pub correlation_id: Option<Uuid>,
    pub compression: CompressionType,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum MessagePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[repr(u8)]
pub enum CompressionType {
    None = 0,
    Gzip = 1,
    Zstd = 2,
}

impl MessageEnvelope {
    /// Canonical representation for signing (deterministic binary format)
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.version);
        bytes.extend_from_slice(self.message_id.as_bytes());
        bytes.extend_from_slice(self.sender_nhi.as_bytes());
        bytes.extend_from_slice(self.recipient_nhi.as_bytes());
        bytes.extend_from_slice(&self.created_at.timestamp_millis().to_le_bytes());
        bytes.extend_from_slice(&self.encrypted_payload);
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(&self.wrapped_key);

        bytes
    }

    /// Wire format size estimation
    pub fn wire_size(&self) -> usize {
        1 +  // version
        16 + // message_id (UUID)
        self.sender_nhi.len() +
        self.recipient_nhi.len() +
        8 +  // created_at (i64)
        8 +  // expires_at (i64)
        self.encrypted_payload.len() +
        12 + // nonce
        16 + // auth_tag
        self.wrapped_key.len() +  // ~1568 bytes (ML-KEM-768)
        16 + // key_id (UUID)
        64 + // signature_ed25519
        self.signature_ml_dsa.len() +  // ~3293 bytes (ML-DSA-65)
        1 +  // priority
        4 +  // ttl_seconds
        16 + // correlation_id (optional)
        1    // compression
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatchetState {
    pub session_id: Uuid,
    pub agent_a: String,
    pub agent_b: String,
    pub current_key_id: Uuid,
    pub current_key_version: u32,
    pub rotation_policy: RotationPolicy,
    pub last_rotated_at: DateTime<Utc>,
    pub next_rotation_at: Option<DateTime<Utc>>,
    pub grace_period_keys: Vec<GracePeriodKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RotationPolicy {
    Hourly,
    Daily,
    Weekly,
    OnDemand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracePeriodKey {
    pub key_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    pub owner: String,
    pub policy: ChannelPolicy,
    pub retention_seconds: u32,
    pub max_message_size: u32,
    pub max_subscribers: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelPolicy {
    Open,
    Private,
    AuthzRequired,
    Allowlist(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    pub receipt_id: Uuid,
    pub message_id: Uuid,
    pub sender: String,
    pub recipient: String,
    pub status: DeliveryStatus,
    pub delivery_attempts: u32,
    pub sent_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Expired,
    Failed,
}
```

---

## Encryption at Rest

### Database-Level Encryption (Transparent Data Encryption)

**PostgreSQL:**
```sql
-- Enable TDE (Transparent Data Encryption)
-- Encrypts all data at the block level

-- Using pgcrypto extension for column-level encryption
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Encrypt sensitive column
UPDATE ratchet_states
SET grace_period_keys = pgp_sym_encrypt(
    grace_period_keys::text,
    current_setting('app.encryption_key')
);

-- Decrypt on read
SELECT
    session_id,
    pgp_sym_decrypt(grace_period_keys::bytea, current_setting('app.encryption_key'))::jsonb AS grace_keys
FROM ratchet_states;
```

**Redis:**
```bash
# Redis Encryption at Rest (RDB snapshots)
# Use LUKS (Linux Unified Key Setup) for block device encryption

cryptsetup luksFormat /dev/sdb  # Encrypt Redis data volume
cryptsetup luksOpen /dev/sdb redis_encrypted
mkfs.ext4 /dev/mapper/redis_encrypted
mount /dev/mapper/redis_encrypted /var/lib/redis
```

### Application-Level Encryption

**Field-level encryption for grace_period_keys:**

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct FieldEncryptor {
    cipher: Aes256Gcm,
}

impl FieldEncryptor {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(Key::from_slice(key));
        Self { cipher }
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<Vec<u8>, EncryptionError> {
        let nonce = generate_nonce();  // Random 12 bytes
        let ciphertext = self.cipher.encrypt(&nonce, plaintext.as_bytes())?;

        // Format: nonce || ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    pub fn decrypt(&self, encrypted: &[u8]) -> Result<String, EncryptionError> {
        let (nonce, ciphertext) = encrypted.split_at(12);
        let plaintext = self.cipher.decrypt(Nonce::from_slice(nonce), ciphertext)?;

        Ok(String::from_utf8(plaintext)?)
    }
}

// Usage
let encryptor = FieldEncryptor::new(&db_encryption_key);
let encrypted_grace_keys = encryptor.encrypt(&serde_json::to_string(&grace_keys)?)?;

// Store to database
sqlx::query!(
    "INSERT INTO ratchet_states (grace_period_keys, ...) VALUES ($1, ...)",
    encrypted_grace_keys
).execute(&pool).await?;
```

**Key management:**
```rust
// Database encryption key stored in HSM (via creto-nhi)
pub async fn get_db_encryption_key() -> Result<[u8; 32]> {
    let nhi_client = NhiClient::new();
    nhi_client.get_encryption_key("db_field_encryption_v1").await
}
```

---

## Retention Policies

### Message Envelopes

**Policy:**
- **Delivered messages:** Delete immediately after acknowledgment
- **Undelivered messages:** Retain until TTL expires (default: 7 days)
- **Expired messages:** Delete, notify sender

**Implementation:**
```rust
pub async fn cleanup_expired_envelopes(redis: &RedisClient) -> Result<u64> {
    let now = Utc::now().timestamp_millis();
    let mut deleted = 0;

    // Scan all envelope keys
    let mut cursor = "0".to_string();
    loop {
        let (new_cursor, keys): (String, Vec<String>) = redis
            .scan(cursor.clone(), "envelope:*", 1000)
            .await?;

        for key in keys {
            let expires_at: i64 = redis.hget(&key, "expires_at").await?;

            if expires_at < now {
                // Message expired
                let sender: String = redis.hget(&key, "sender_nhi").await?;
                let message_id: String = redis.hget(&key, "message_id").await?;

                // Notify sender of expiration
                audit_client.log(AuditEvent {
                    event_type: "message_expired",
                    actor: sender.clone(),
                    message_id: message_id.clone(),
                    ..Default::default()
                }).await?;

                // Delete envelope
                redis.del(&key).await?;
                deleted += 1;
            }
        }

        cursor = new_cursor;
        if cursor == "0" {
            break;
        }
    }

    Ok(deleted)
}

// Run cleanup every hour
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(3600));
    loop {
        interval.tick().await;
        let _ = cleanup_expired_envelopes(&redis_client).await;
    }
});
```

### Delivery Receipts

**Policy:**
- **Active receipts:** Retain 30 days
- **Archived receipts:** Move to S3 after 30 days, retain 7 years

**Implementation:**
```rust
pub async fn archive_old_receipts(db: &PgPool, s3: &S3Client) -> Result<()> {
    let cutoff = Utc::now() - Duration::days(30);

    // Fetch old receipts
    let receipts = sqlx::query_as!(
        DeliveryReceipt,
        "SELECT * FROM delivery_receipts WHERE sent_at < $1 LIMIT 10000",
        cutoff
    ).fetch_all(db).await?;

    if receipts.is_empty() {
        return Ok(());
    }

    // Serialize to Parquet for efficient storage
    let parquet_file = serialize_to_parquet(&receipts)?;

    // Upload to S3
    let key = format!(
        "delivery_receipts/{}/{}.parquet",
        cutoff.format("%Y/%m"),
        Uuid::new_v4()
    );
    s3.put_object(bucket, &key, parquet_file).await?;

    // Delete from Postgres
    let receipt_ids: Vec<Uuid> = receipts.iter().map(|r| r.receipt_id).collect();
    sqlx::query!(
        "DELETE FROM delivery_receipts WHERE receipt_id = ANY($1)",
        &receipt_ids
    ).execute(db).await?;

    Ok(())
}
```

### Ratchet State

**Policy:**
- **Active sessions:** Retain indefinitely (while agents communicate)
- **Inactive sessions (>90 days):** Archive to S3, delete from Postgres

```rust
pub async fn cleanup_inactive_sessions(db: &PgPool) -> Result<()> {
    let cutoff = Utc::now() - Duration::days(90);

    sqlx::query!(
        "DELETE FROM ratchet_states
         WHERE last_rotated_at < $1
         AND NOT EXISTS (
             SELECT 1 FROM delivery_receipts dr
             WHERE dr.sender_nhi = ratchet_states.agent_a_nhi
             AND dr.recipient_nhi = ratchet_states.agent_b_nhi
             AND dr.sent_at > $1
         )",
        cutoff
    ).execute(db).await?;

    Ok(())
}
```

---

## Data Flow

### Send Message Data Flow

```
1. Agent A calls send_message()
   ↓
2. MessageSender encrypts payload
   ↓
3. Envelope written to Redis
   Key: envelope:{recipient_nhi}:{message_id}
   TTL: envelope.ttl_seconds
   ↓
4. Envelope added to priority queue
   Redis ZADD queue:{recipient_nhi}:priority:{priority}
   ↓
5. Delivery receipt created (status=pending)
   PostgreSQL INSERT INTO delivery_receipts
   ↓
6. Agent B polls receive_messages()
   ↓
7. Envelope fetched from Redis queue
   Redis ZRANGE queue:{recipient_nhi}:priority:3 0 0
   ↓
8. MessageReceiver decrypts envelope
   ↓
9. Agent B acknowledges message
   ↓
10. Envelope deleted from Redis
    Redis DEL envelope:{recipient_nhi}:{message_id}
    ↓
11. Delivery receipt updated (status=delivered)
    PostgreSQL UPDATE delivery_receipts
    ↓
12. After 30 days, receipt archived to S3
```

### Key Rotation Data Flow

```
1. RotationScheduler triggers rotation
   ↓
2. Fetch RatchetState from PostgreSQL
   SELECT * FROM ratchet_states WHERE next_rotation_at < NOW()
   ↓
3. NHI generates new ML-KEM keypair
   ↓
4. Update RatchetState:
   - Move current_key_id to grace_period_keys
   - Set new current_key_id
   - Update last_rotated_at, next_rotation_at
   ↓
5. Write updated state to PostgreSQL
   UPDATE ratchet_states
   ↓
6. Invalidate key cache in Redis
   DEL key:{agent_nhi}:ml_kem_768:active
   ↓
7. Audit log rotation event
   PostgreSQL INSERT INTO audit_events
   ↓
8. After 7 days, grace period keys expire
   Cleanup grace_period_keys array
```

---

## Migration Strategy

### Schema Versioning

**Approach:** Use PostgreSQL schema migrations with version tracking.

```sql
CREATE TABLE schema_migrations (
    version             INTEGER PRIMARY KEY,
    description         TEXT NOT NULL,
    applied_at          TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Migration 001: Initial schema
INSERT INTO schema_migrations (version, description)
VALUES (1, 'Initial messaging schema');

-- Migration 002: Add compression support
ALTER TABLE message_envelopes ADD COLUMN compression INTEGER DEFAULT 0;
INSERT INTO schema_migrations (version, description)
VALUES (2, 'Add compression support');
```

**Migration tools:**
- **sqlx-cli** (Rust): `sqlx migrate run`
- **Flyway** (JVM): `flyway migrate`

### Zero-Downtime Migrations

**Strategy:**
1. **Backward-compatible changes first:**
   - Add new columns (nullable)
   - Add new tables
   - Dual-write to old and new schemas

2. **Deploy application code that supports both schemas**

3. **Backfill data**

4. **Remove old schema (after validation)**

**Example: Adding compression field**

```rust
// Phase 1: Add nullable column
ALTER TABLE message_envelopes ADD COLUMN compression INTEGER;

// Phase 2: Deploy code that writes compression
pub fn store_envelope(envelope: &MessageEnvelope) {
    redis.hset("envelope:...", "compression", envelope.compression);
}

// Phase 3: Backfill existing envelopes
UPDATE message_envelopes SET compression = 0 WHERE compression IS NULL;

// Phase 4: Make column NOT NULL
ALTER TABLE message_envelopes ALTER COLUMN compression SET NOT NULL;
```

---

## Performance Optimization

### Indexing Strategy

**PostgreSQL:**
```sql
-- Ratchet state: Find sessions due for rotation
CREATE INDEX idx_ratchet_next_rotation
ON ratchet_states (next_rotation_at)
WHERE next_rotation_at IS NOT NULL;

-- Delivery receipts: Query by recipient and time
CREATE INDEX idx_receipt_recipient_time
ON delivery_receipts (recipient_nhi, sent_at DESC);

-- Channel subscriptions: Find subscribers for a channel
CREATE INDEX idx_subscription_channel
ON channel_subscriptions (channel_id)
WHERE status = 'active';
```

**Redis:**
```redis
# Priority queues already use sorted sets (O(log N) operations)
# No additional indexing needed
```

### Query Optimization

**Batch fetching:**
```rust
// Instead of N queries:
for recipient in recipients {
    let messages = fetch_messages(recipient).await?;
}

// Use single query with IN clause:
let messages = sqlx::query!(
    "SELECT * FROM message_envelopes WHERE recipient_nhi = ANY($1)",
    &recipients
).fetch_all(&pool).await?;
```

**Connection pooling:**
```rust
// PostgreSQL connection pool
let pool = PgPoolOptions::new()
    .max_connections(100)
    .connect(&database_url)
    .await?;

// Redis connection pool (r2d2)
let redis_pool = Pool::builder()
    .max_size(50)
    .build(RedisConnectionManager::new(redis_url)?)?;
```

### Caching Strategy

**Public keys (hot path):**
```rust
// Cache ML-KEM public keys in Redis (1 hour TTL)
pub async fn get_public_key_cached(
    agent_nhi: &str,
    key_type: KeyType,
) -> Result<Vec<u8>> {
    let cache_key = format!("pubkey:{}:{:?}", agent_nhi, key_type);

    // Try cache first
    if let Some(cached) = redis.get(&cache_key).await? {
        return Ok(cached);
    }

    // Cache miss: Fetch from NHI
    let pubkey = nhi_client.get_public_key(agent_nhi, key_type).await?;

    // Store in cache (1 hour TTL)
    redis.setex(&cache_key, 3600, &pubkey).await?;

    Ok(pubkey)
}
```

**Delivery authorization decisions:**
```rust
// Cache AuthZ decisions (5 minutes TTL)
pub async fn check_delivery_cached(
    sender: &str,
    recipient: &str,
) -> Result<Decision> {
    let cache_key = format!("authz:{}:{}", sender, recipient);

    if let Some(cached) = redis.get(&cache_key).await? {
        return Ok(serde_json::from_str(&cached)?);
    }

    let decision = authz_client.check(sender, Action::SendMessage, recipient).await?;

    // Cache ALLOW decisions only (avoid caching denials)
    if matches!(decision, Decision::Allow) {
        redis.setex(&cache_key, 300, &serde_json::to_string(&decision)?).await?;
    }

    Ok(decision)
}
```

### Partitioning

**Redis partitioning (256 nodes):**
- Distributes load across Redis Cluster
- Each node handles ~1/256 of recipient space
- Linear scaling with more nodes

**PostgreSQL partitioning (by time for receipts):**
```sql
-- Partition delivery_receipts by month
CREATE TABLE delivery_receipts_2025_12 PARTITION OF delivery_receipts
    FOR VALUES FROM ('2025-12-01') TO ('2026-01-01');

-- Auto-create future partitions
CREATE OR REPLACE FUNCTION create_monthly_partition()
RETURNS void AS $$
DECLARE
    start_date DATE;
    end_date DATE;
    partition_name TEXT;
BEGIN
    start_date := date_trunc('month', CURRENT_DATE + INTERVAL '1 month');
    end_date := start_date + INTERVAL '1 month';
    partition_name := 'delivery_receipts_' || to_char(start_date, 'YYYY_MM');

    EXECUTE format(
        'CREATE TABLE IF NOT EXISTS %I PARTITION OF delivery_receipts
         FOR VALUES FROM (%L) TO (%L)',
        partition_name,
        start_date,
        end_date
    );
END;
$$ LANGUAGE plpgsql;

-- Schedule monthly partition creation
SELECT cron.schedule('create-partition', '0 0 1 * *', 'SELECT create_monthly_partition()');
```

---

## Summary

### Key Design Decisions

1. **Never store plaintext:** Only encrypted envelopes persisted
2. **Immediate deletion:** Messages deleted after delivery acknowledgment
3. **Partitioning:** 256 partitions for horizontal scaling
4. **Encryption at rest:** TDE + application-level field encryption
5. **Retention:** 7 days (messages), 30 days (receipts), 7 years (audit)
6. **Caching:** Public keys (1h), AuthZ decisions (5m)

### Storage Estimates

**Per message:**
- Envelope size: ~6KB (1KB payload + 1.5KB ML-KEM + 3.3KB ML-DSA)
- Delivery receipt: ~200 bytes

**Scale estimates (1M agents, 1M msg/sec):**
- Active envelopes (7 day retention): 6KB × 604800 msg × 1M agents = 3.6 PB (unrealistic, most delivered within seconds)
- Realistic (99% delivered within 1 hour): 6KB × 3600 msg × 1M agents = 21.6 TB
- Delivery receipts (30 days): 200B × 2.6B msg = 520 GB
- Archived receipts (7 years): Compressed Parquet ~50 GB/year = 350 GB

### Next Steps

1. **API design** (see 04-api-design.md)
2. **Security threat modeling** (see 05-security.md)
3. **Performance benchmarking** (see 06-testing.md)
4. **Operational procedures** (see 07-runbook.md)
