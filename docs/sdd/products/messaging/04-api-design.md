---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# Messaging API Design

## Table of Contents

1. [API Overview](#api-overview)
2. [gRPC Service Definition](#grpc-service-definition)
3. [REST API](#rest-api)
4. [Client SDK](#client-sdk)
5. [Error Handling](#error-handling)
6. [Rate Limiting](#rate-limiting)
7. [Versioning Strategy](#versioning-strategy)
8. [Authentication](#authentication)

---

## API Overview

### Design Principles

1. **gRPC-first for performance:** Binary protocol, streaming support
2. **REST gateway for compatibility:** HTTP/JSON fallback
3. **Idempotent operations:** Safe retries via message_id
4. **Explicit errors:** Structured error codes and messages
5. **Backward compatible:** Protocol buffers field evolution

### API Layers

```
┌─────────────────────────────────────────────────────────┐
│              CLIENT SDK (Rust, Python, Go)              │
│  - High-level API (send, receive, subscribe)           │
│  - Automatic encryption/decryption                      │
│  - Retry logic and backoff                              │
└────────────────────┬────────────────────────────────────┘
                     │
         ┌───────────┴────────────┐
         │                        │
         ▼                        ▼
┌─────────────────┐      ┌─────────────────┐
│   gRPC API      │      │   REST API      │
│ (Primary)       │      │ (HTTP Gateway)  │
│ Port 50051      │      │ Port 8080       │
└────────┬────────┘      └────────┬────────┘
         │                        │
         └───────────┬────────────┘
                     ▼
         ┌─────────────────────┐
         │ EnvelopeProcessor   │
         │ KeyManagementService│
         │ ChannelService      │
         └─────────────────────┘
```

---

## gRPC Service Definition

### Protocol Buffers Schema

**File: `messaging.proto`**

```protobuf
syntax = "proto3";

package creto.messaging.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/empty.proto";

// ============================================================================
// MESSAGING SERVICE
// ============================================================================

service MessagingService {
  // Send encrypted message to recipient
  rpc SendMessage(SendMessageRequest) returns (SendMessageResponse);

  // Send message and wait for response (request/response pattern)
  rpc SendAndWait(SendAndWaitRequest) returns (SendAndWaitResponse);

  // Receive pending messages
  rpc ReceiveMessages(ReceiveMessagesRequest) returns (stream MessageEnvelope);

  // Acknowledge message delivery
  rpc AcknowledgeMessage(AcknowledgeRequest) returns (google.protobuf.Empty);

  // Batch operations
  rpc SendMessageBatch(SendMessageBatchRequest) returns (SendMessageBatchResponse);
}

// ============================================================================
// MESSAGE TYPES
// ============================================================================

message SendMessageRequest {
  // Recipient agent NHI
  string recipient_nhi = 1;

  // Encrypted payload (AES-256-GCM)
  bytes encrypted_payload = 2;

  // AES-GCM nonce (12 bytes)
  bytes nonce = 3;

  // AES-GCM authentication tag (16 bytes)
  bytes auth_tag = 4;

  // Wrapped AES key (ML-KEM-768 encapsulation)
  bytes wrapped_key = 5;

  // Recipient's public key ID
  string key_id = 6;

  // Ed25519 signature (64 bytes)
  bytes signature_ed25519 = 7;

  // ML-DSA-65 signature (~3293 bytes)
  bytes signature_ml_dsa = 8;

  // Message priority
  MessagePriority priority = 9;

  // Time-to-live (seconds)
  optional uint32 ttl_seconds = 10;

  // Correlation ID for request/response
  optional string correlation_id = 11;

  // Channel ID for topic-based messaging
  optional string channel_id = 12;

  // Compression type
  CompressionType compression = 13;

  // Idempotency key (prevents duplicate sends)
  string idempotency_key = 14;
}

message SendMessageResponse {
  // Assigned message ID
  string message_id = 1;

  // Timestamp when message was enqueued
  google.protobuf.Timestamp enqueued_at = 2;

  // Estimated delivery time
  google.protobuf.Timestamp estimated_delivery = 3;
}

message SendAndWaitRequest {
  // Same as SendMessageRequest
  SendMessageRequest request = 1;

  // Timeout for waiting (seconds)
  uint32 timeout_seconds = 2;
}

message SendAndWaitResponse {
  // Original message ID
  string request_message_id = 1;

  // Response message
  MessageEnvelope response = 2;
}

message ReceiveMessagesRequest {
  // Maximum number of messages to receive
  uint32 max_messages = 1;

  // Wait timeout (seconds) - long polling
  optional uint32 wait_timeout_seconds = 2;

  // Filter by priority (optional)
  optional MessagePriority min_priority = 3;

  // Filter by channel (optional)
  optional string channel_id = 4;
}

message AcknowledgeRequest {
  // Message IDs to acknowledge
  repeated string message_ids = 1;
}

message SendMessageBatchRequest {
  repeated SendMessageRequest messages = 1;
}

message SendMessageBatchResponse {
  repeated SendMessageResponse responses = 1;
  repeated Error errors = 2;
}

message MessageEnvelope {
  // Protocol version
  uint32 version = 1;

  // Message ID
  string message_id = 2;

  // Sender agent NHI
  string sender_nhi = 3;

  // Recipient agent NHI
  string recipient_nhi = 4;

  // Channel ID (for topic-based messaging)
  optional string channel_id = 5;

  // Timestamps
  google.protobuf.Timestamp created_at = 6;
  google.protobuf.Timestamp expires_at = 7;

  // Encrypted content
  bytes encrypted_payload = 8;
  bytes nonce = 9;
  bytes auth_tag = 10;

  // Key encapsulation
  bytes wrapped_key = 11;
  string key_id = 12;

  // Signatures
  bytes signature_ed25519 = 13;
  bytes signature_ml_dsa = 14;

  // Metadata
  MessagePriority priority = 15;
  uint32 ttl_seconds = 16;
  optional string correlation_id = 17;
  CompressionType compression = 18;
}

enum MessagePriority {
  LOW = 0;
  NORMAL = 1;
  HIGH = 2;
  CRITICAL = 3;
}

enum CompressionType {
  NONE = 0;
  GZIP = 1;
  ZSTD = 2;
}

// ============================================================================
// KEY MANAGEMENT SERVICE
// ============================================================================

service KeyManagementService {
  // Upload prekey bundle (public keys)
  rpc UploadKeyBundle(UploadKeyBundleRequest) returns (google.protobuf.Empty);

  // Get agent's public key bundle
  rpc GetKeyBundle(GetKeyBundleRequest) returns (KeyBundle);

  // Rotate session key
  rpc RotateSessionKey(RotateSessionKeyRequest) returns (google.protobuf.Empty);

  // Get rotation history
  rpc GetRotationHistory(GetRotationHistoryRequest) returns (RotationHistoryResponse);
}

message UploadKeyBundleRequest {
  // Agent NHI
  string agent_nhi = 1;

  // ML-KEM-768 public key (1184 bytes)
  bytes ml_kem_public_key = 2;

  // Ed25519 public key (32 bytes)
  bytes ed25519_public_key = 3;

  // ML-DSA-65 public key (~1952 bytes)
  bytes ml_dsa_public_key = 4;

  // Expiration time for keys
  google.protobuf.Timestamp expires_at = 5;
}

message GetKeyBundleRequest {
  string agent_nhi = 1;
}

message KeyBundle {
  string agent_nhi = 1;
  string key_id = 2;
  bytes ml_kem_public_key = 3;
  bytes ed25519_public_key = 4;
  bytes ml_dsa_public_key = 5;
  google.protobuf.Timestamp created_at = 6;
  google.protobuf.Timestamp expires_at = 7;
  KeyStatus status = 8;
}

enum KeyStatus {
  ACTIVE = 0;
  GRACE_PERIOD = 1;
  REVOKED = 2;
}

message RotateSessionKeyRequest {
  string session_id = 1;
  bool emergency = 2;  // Skip grace period
}

message GetRotationHistoryRequest {
  string agent_nhi = 1;
  uint32 limit = 2;
}

message RotationHistoryResponse {
  repeated RotationEvent events = 1;
}

message RotationEvent {
  string session_id = 1;
  string old_key_id = 2;
  string new_key_id = 3;
  google.protobuf.Timestamp rotated_at = 4;
  string reason = 5;  // "scheduled", "emergency", "on_demand"
}

// ============================================================================
// CHANNEL SERVICE
// ============================================================================

service ChannelService {
  // Create channel (topic)
  rpc CreateChannel(CreateChannelRequest) returns (CreateChannelResponse);

  // Delete channel
  rpc DeleteChannel(DeleteChannelRequest) returns (google.protobuf.Empty);

  // Subscribe to channel
  rpc Subscribe(SubscribeRequest) returns (google.protobuf.Empty);

  // Unsubscribe from channel
  rpc Unsubscribe(UnsubscribeRequest) returns (google.protobuf.Empty);

  // Publish message to channel
  rpc Publish(PublishRequest) returns (PublishResponse);

  // List channels
  rpc ListChannels(ListChannelsRequest) returns (ListChannelsResponse);

  // Get channel metadata
  rpc GetChannel(GetChannelRequest) returns (Channel);
}

message CreateChannelRequest {
  string channel_name = 1;
  ChannelPolicy policy = 2;
  uint32 retention_seconds = 3;
  uint32 max_subscribers = 4;
  repeated string allowlist_nhis = 5;  // For allowlist policy
}

message CreateChannelResponse {
  string channel_id = 1;
}

message DeleteChannelRequest {
  string channel_id = 1;
}

message SubscribeRequest {
  string channel_id = 1;
}

message UnsubscribeRequest {
  string channel_id = 1;
}

message PublishRequest {
  string channel_id = 1;
  SendMessageRequest message = 2;  // Reuse message structure
}

message PublishResponse {
  string message_id = 1;
  uint32 subscriber_count = 2;
}

message ListChannelsRequest {
  optional string owner_nhi = 1;
  uint32 limit = 2;
  string page_token = 3;
}

message ListChannelsResponse {
  repeated Channel channels = 1;
  string next_page_token = 2;
}

message GetChannelRequest {
  string channel_id = 1;
}

message Channel {
  string channel_id = 1;
  string channel_name = 2;
  string owner_nhi = 3;
  ChannelPolicy policy = 4;
  uint32 retention_seconds = 5;
  uint32 max_subscribers = 6;
  uint32 current_subscribers = 7;
  uint64 total_messages = 8;
  google.protobuf.Timestamp created_at = 9;
}

enum ChannelPolicy {
  OPEN = 0;
  PRIVATE = 1;
  AUTHZ_REQUIRED = 2;
  ALLOWLIST = 3;
}

// ============================================================================
// ERROR TYPES
// ============================================================================

message Error {
  ErrorCode code = 1;
  string message = 2;
  map<string, string> details = 3;
}

enum ErrorCode {
  UNKNOWN = 0;
  INVALID_ARGUMENT = 1;
  NOT_FOUND = 2;
  ALREADY_EXISTS = 3;
  PERMISSION_DENIED = 4;
  UNAUTHENTICATED = 5;
  RESOURCE_EXHAUSTED = 6;
  FAILED_PRECONDITION = 7;
  ABORTED = 8;
  OUT_OF_RANGE = 9;
  UNIMPLEMENTED = 10;
  INTERNAL = 11;
  UNAVAILABLE = 12;
  DEADLINE_EXCEEDED = 13;

  // Custom error codes
  SIGNATURE_VERIFICATION_FAILED = 100;
  ENCRYPTION_FAILED = 101;
  DECRYPTION_FAILED = 102;
  KEY_NOT_FOUND = 103;
  MESSAGE_EXPIRED = 104;
  AUTHORIZATION_DENIED = 105;
  RATE_LIMIT_EXCEEDED = 106;
  INVALID_KEY_FORMAT = 107;
}
```

---

## REST API

### HTTP Gateway (gRPC-Gateway)

**Base URL:** `https://messaging.creto.io/v1`

**Authentication:** Bearer token in `Authorization` header

### Endpoints

#### 1. Send Message

```http
POST /v1/messages
Content-Type: application/json
Authorization: Bearer <token>

{
  "recipient_nhi": "agent-bob-02",
  "encrypted_payload": "<base64>",
  "nonce": "<base64>",
  "auth_tag": "<base64>",
  "wrapped_key": "<base64>",
  "key_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "signature_ed25519": "<base64>",
  "signature_ml_dsa": "<base64>",
  "priority": "NORMAL",
  "ttl_seconds": 604800,
  "idempotency_key": "unique-key-123"
}
```

**Response (201 Created):**
```json
{
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "enqueued_at": "2025-12-25T10:00:00Z",
  "estimated_delivery": "2025-12-25T10:00:02Z"
}
```

**Error Response (403 Forbidden):**
```json
{
  "error": {
    "code": "AUTHORIZATION_DENIED",
    "message": "Sender not authorized to send to recipient",
    "details": {
      "sender": "agent-alice-01",
      "recipient": "agent-bob-02",
      "policy_id": "deny-cross-org"
    }
  }
}
```

#### 2. Receive Messages

```http
GET /v1/messages?max_messages=10&wait_timeout_seconds=30
Authorization: Bearer <token>
```

**Response (200 OK):**
```json
{
  "messages": [
    {
      "message_id": "550e8400-e29b-41d4-a716-446655440000",
      "sender_nhi": "agent-alice-01",
      "recipient_nhi": "agent-bob-02",
      "encrypted_payload": "<base64>",
      "nonce": "<base64>",
      "wrapped_key": "<base64>",
      "key_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
      "signature_ed25519": "<base64>",
      "signature_ml_dsa": "<base64>",
      "created_at": "2025-12-25T10:00:00Z",
      "expires_at": "2026-01-01T10:00:00Z",
      "priority": "NORMAL"
    }
  ]
}
```

#### 3. Acknowledge Messages

```http
POST /v1/messages/acknowledge
Content-Type: application/json
Authorization: Bearer <token>

{
  "message_ids": [
    "550e8400-e29b-41d4-a716-446655440000",
    "661f9511-f39c-52e5-b827-557766551111"
  ]
}
```

**Response (204 No Content)**

#### 4. Upload Key Bundle

```http
POST /v1/keys/bundle
Content-Type: application/json
Authorization: Bearer <token>

{
  "agent_nhi": "agent-alice-01",
  "ml_kem_public_key": "<base64>",
  "ed25519_public_key": "<base64>",
  "ml_dsa_public_key": "<base64>",
  "expires_at": "2026-12-25T10:00:00Z"
}
```

**Response (201 Created)**

#### 5. Get Key Bundle

```http
GET /v1/keys/bundle/agent-bob-02
Authorization: Bearer <token>
```

**Response (200 OK):**
```json
{
  "agent_nhi": "agent-bob-02",
  "key_id": "7c9e6679-7425-40de-944b-e07fc1f90ae7",
  "ml_kem_public_key": "<base64>",
  "ed25519_public_key": "<base64>",
  "ml_dsa_public_key": "<base64>",
  "created_at": "2025-12-25T10:00:00Z",
  "expires_at": "2026-12-25T10:00:00Z",
  "status": "ACTIVE"
}
```

#### 6. Create Channel

```http
POST /v1/channels
Content-Type: application/json
Authorization: Bearer <token>

{
  "channel_name": "swarm-coordination",
  "policy": "AUTHZ_REQUIRED",
  "retention_seconds": 604800,
  "max_subscribers": 1000
}
```

**Response (201 Created):**
```json
{
  "channel_id": "c9a3a2e0-1234-5678-9abc-def012345678"
}
```

#### 7. Subscribe to Channel

```http
POST /v1/channels/{channel_id}/subscribe
Authorization: Bearer <token>
```

**Response (204 No Content)**

#### 8. Publish to Channel

```http
POST /v1/channels/{channel_id}/publish
Content-Type: application/json
Authorization: Bearer <token>

{
  "message": {
    "recipient_nhi": "*",  // Broadcast to all subscribers
    "encrypted_payload": "<base64>",
    // ... same as SendMessage
  }
}
```

**Response (201 Created):**
```json
{
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "subscriber_count": 42
}
```

---

## Client SDK

### Rust SDK

**Installation:**
```toml
[dependencies]
creto-messaging = "0.1.0"
```

**Usage:**

```rust
use creto_messaging::{MessagingClient, SendOptions, MessagePriority};
use creto_nhi::NhiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let nhi_client = NhiClient::new("agent-alice-01").await?;
    let messaging_client = MessagingClient::builder()
        .endpoint("https://messaging.creto.io")
        .nhi_client(nhi_client.clone())
        .build()
        .await?;

    // Send message
    let message_id = messaging_client
        .send(
            "agent-bob-02",
            b"Hello, Bob!",
            SendOptions {
                priority: MessagePriority::Normal,
                ttl_seconds: Some(3600),
                ..Default::default()
            },
        )
        .await?;

    println!("Message sent: {}", message_id);

    // Receive messages
    let messages = messaging_client
        .receive(10, None)
        .await?;

    for msg in messages {
        println!(
            "Received from {}: {}",
            msg.sender,
            String::from_utf8_lossy(&msg.payload)
        );

        // Acknowledge
        messaging_client.acknowledge(&msg.id).await?;
    }

    // Request/response pattern
    let response = messaging_client
        .send_and_wait(
            "agent-bob-02",
            b"get_status",
            SendOptions::default(),
            std::time::Duration::from_secs(5),
        )
        .await?;

    println!("Response: {}", String::from_utf8_lossy(&response.payload));

    Ok(())
}
```

**Channel usage:**

```rust
use creto_messaging::{ChannelPolicy, ChannelClient};

async fn channel_example() -> Result<()> {
    let channel_client = ChannelClient::new(messaging_client);

    // Create channel
    let channel_id = channel_client
        .create("swarm-coordination", ChannelPolicy::AuthzRequired)
        .await?;

    // Subscribe
    channel_client.subscribe(&channel_id).await?;

    // Publish
    channel_client
        .publish(&channel_id, b"Coordination message")
        .await?;

    // Receive from channel
    let messages = channel_client
        .receive_from_channel(&channel_id, 10)
        .await?;

    Ok(())
}
```

### Python SDK

**Installation:**
```bash
pip install creto-messaging
```

**Usage:**

```python
from creto_messaging import MessagingClient, SendOptions, MessagePriority
from creto_nhi import NhiClient
import asyncio

async def main():
    # Initialize client
    nhi_client = NhiClient("agent-alice-01")
    messaging_client = MessagingClient(
        endpoint="https://messaging.creto.io",
        nhi_client=nhi_client
    )

    # Send message
    message_id = await messaging_client.send(
        recipient="agent-bob-02",
        payload=b"Hello, Bob!",
        options=SendOptions(
            priority=MessagePriority.NORMAL,
            ttl_seconds=3600
        )
    )
    print(f"Message sent: {message_id}")

    # Receive messages
    messages = await messaging_client.receive(max_messages=10)
    for msg in messages:
        print(f"Received from {msg.sender}: {msg.payload.decode()}")
        await messaging_client.acknowledge(msg.id)

    # Request/response
    response = await messaging_client.send_and_wait(
        recipient="agent-bob-02",
        payload=b"get_status",
        timeout=5.0
    )
    print(f"Response: {response.payload.decode()}")

asyncio.run(main())
```

### Go SDK

**Installation:**
```bash
go get github.com/creto/messaging-go
```

**Usage:**

```go
package main

import (
    "context"
    "fmt"
    "time"

    messaging "github.com/creto/messaging-go"
    "github.com/creto/nhi-go"
)

func main() {
    ctx := context.Background()

    // Initialize client
    nhiClient, _ := nhi.NewClient("agent-alice-01")
    client, _ := messaging.NewClient(ctx, messaging.Config{
        Endpoint:  "messaging.creto.io:50051",
        NHIClient: nhiClient,
    })
    defer client.Close()

    // Send message
    messageID, err := client.Send(ctx, &messaging.SendRequest{
        Recipient: "agent-bob-02",
        Payload:   []byte("Hello, Bob!"),
        Options: messaging.SendOptions{
            Priority:   messaging.PriorityNormal,
            TTLSeconds: 3600,
        },
    })
    if err != nil {
        panic(err)
    }
    fmt.Printf("Message sent: %s\n", messageID)

    // Receive messages
    messages, err := client.Receive(ctx, &messaging.ReceiveRequest{
        MaxMessages: 10,
    })
    if err != nil {
        panic(err)
    }

    for _, msg := range messages {
        fmt.Printf("Received from %s: %s\n", msg.Sender, string(msg.Payload))
        client.Acknowledge(ctx, msg.ID)
    }

    // Request/response
    response, err := client.SendAndWait(ctx, &messaging.SendAndWaitRequest{
        Recipient: "agent-bob-02",
        Payload:   []byte("get_status"),
        Timeout:   5 * time.Second,
    })
    if err != nil {
        panic(err)
    }
    fmt.Printf("Response: %s\n", string(response.Payload))
}
```

---

## Error Handling

### Error Code Mapping

| gRPC Status | HTTP Status | ErrorCode | Description |
|-------------|-------------|-----------|-------------|
| `INVALID_ARGUMENT` | 400 | `INVALID_ARGUMENT` | Missing/invalid field |
| `NOT_FOUND` | 404 | `NOT_FOUND` | Message/channel not found |
| `ALREADY_EXISTS` | 409 | `ALREADY_EXISTS` | Duplicate channel name |
| `PERMISSION_DENIED` | 403 | `AUTHORIZATION_DENIED` | AuthZ check failed |
| `UNAUTHENTICATED` | 401 | `UNAUTHENTICATED` | Missing/invalid token |
| `RESOURCE_EXHAUSTED` | 429 | `RATE_LIMIT_EXCEEDED` | Too many requests |
| `FAILED_PRECONDITION` | 400 | `FAILED_PRECONDITION` | Invalid state transition |
| `ABORTED` | 409 | `ABORTED` | Conflict (retry) |
| `OUT_OF_RANGE` | 400 | `OUT_OF_RANGE` | Invalid offset/limit |
| `UNIMPLEMENTED` | 501 | `UNIMPLEMENTED` | Feature not available |
| `INTERNAL` | 500 | `INTERNAL` | Server error |
| `UNAVAILABLE` | 503 | `UNAVAILABLE` | Service down |
| `DEADLINE_EXCEEDED` | 504 | `DEADLINE_EXCEEDED` | Timeout |

### Custom Error Codes

| ErrorCode | HTTP Status | Retry? | Description |
|-----------|-------------|--------|-------------|
| `SIGNATURE_VERIFICATION_FAILED` | 400 | No | Invalid signature (potential forgery) |
| `ENCRYPTION_FAILED` | 500 | Yes | Crypto operation failed (transient) |
| `DECRYPTION_FAILED` | 400 | No | Invalid wrapped key/nonce |
| `KEY_NOT_FOUND` | 404 | Yes | Recipient key not published yet |
| `MESSAGE_EXPIRED` | 410 | No | TTL exceeded before delivery |
| `AUTHORIZATION_DENIED` | 403 | No | Policy denied delivery |
| `RATE_LIMIT_EXCEEDED` | 429 | Yes (after delay) | Quota exceeded |
| `INVALID_KEY_FORMAT` | 400 | No | Malformed public key |

### Error Response Format

**gRPC:**
```protobuf
message Error {
  ErrorCode code = 1;
  string message = 2;
  map<string, string> details = 3;
}
```

**REST (JSON):**
```json
{
  "error": {
    "code": "SIGNATURE_VERIFICATION_FAILED",
    "message": "Ed25519 signature verification failed",
    "details": {
      "sender": "agent-alice-01",
      "message_id": "550e8400-e29b-41d4-a716-446655440000",
      "canonical_hash": "sha256:abc123..."
    }
  }
}
```

### SDK Error Handling

**Rust:**
```rust
use creto_messaging::{MessagingError, ErrorCode};

match messaging_client.send(...).await {
    Ok(message_id) => println!("Sent: {}", message_id),
    Err(MessagingError::AuthorizationDenied { sender, recipient, reason }) => {
        eprintln!("Denied: {} -> {} ({})", sender, recipient, reason);
    }
    Err(MessagingError::RateLimitExceeded { retry_after }) => {
        tokio::time::sleep(retry_after).await;
        // Retry
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

**Python:**
```python
from creto_messaging.exceptions import (
    AuthorizationDenied,
    RateLimitExceeded,
    MessagingError
)

try:
    await client.send(...)
except AuthorizationDenied as e:
    print(f"Denied: {e.sender} -> {e.recipient} ({e.reason})")
except RateLimitExceeded as e:
    await asyncio.sleep(e.retry_after)
    # Retry
except MessagingError as e:
    print(f"Error: {e}")
```

---

## Rate Limiting

### Rate Limit Headers (REST)

```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 723
X-RateLimit-Reset: 1735171200
```

**On rate limit exceeded:**
```http
HTTP/1.1 429 Too Many Requests
Retry-After: 60
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1735171200

{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded: 1000 requests per minute",
    "details": {
      "limit": "1000",
      "window": "60",
      "retry_after": "60"
    }
  }
}
```

### gRPC Rate Limiting (Metadata)

**Response metadata:**
```
x-ratelimit-limit: 1000
x-ratelimit-remaining: 723
x-ratelimit-reset: 1735171200
```

**On rate limit exceeded:**
```
Code: RESOURCE_EXHAUSTED
Message: "Rate limit exceeded: 1000 requests per minute"
Metadata:
  x-ratelimit-limit: 1000
  x-ratelimit-remaining: 0
  x-ratelimit-reset: 1735171200
  retry-after: 60
```

### SDK Automatic Retry

**Rust:**
```rust
let client = MessagingClient::builder()
    .endpoint("https://messaging.creto.io")
    .retry_config(RetryConfig {
        max_retries: 3,
        initial_backoff: Duration::from_secs(1),
        max_backoff: Duration::from_secs(60),
        respect_retry_after: true,
    })
    .build()
    .await?;
```

**Python:**
```python
client = MessagingClient(
    endpoint="https://messaging.creto.io",
    retry_config=RetryConfig(
        max_retries=3,
        initial_backoff=1.0,
        max_backoff=60.0,
        respect_retry_after=True
    )
)
```

---

## Versioning Strategy

### Protocol Buffer Versioning

**Field evolution rules:**
1. **Add new fields:** Use optional/repeated (backward compatible)
2. **Deprecate old fields:** Mark as deprecated, maintain for 2 major versions
3. **Never remove fields:** Only mark as reserved
4. **Never change field numbers:** Breaks wire format

**Example migration:**

```protobuf
// v1 (old)
message SendMessageRequest {
  string recipient_nhi = 1;
  bytes encrypted_payload = 2;
  // ...
}

// v2 (add compression)
message SendMessageRequest {
  string recipient_nhi = 1;
  bytes encrypted_payload = 2;
  CompressionType compression = 13;  // New field (optional)
  // ...
}

// v3 (deprecate old field)
message SendMessageRequest {
  string recipient_nhi = 1;
  bytes encrypted_payload = 2 [deprecated = true];  // Use compressed_payload instead
  bytes compressed_payload = 14;
  CompressionType compression = 13;
  // ...
}
```

### API Versioning

**URL versioning (REST):**
```
/v1/messages          (stable)
/v2/messages          (new features)
/v1beta1/messages     (beta, may change)
```

**Package versioning (gRPC):**
```protobuf
package creto.messaging.v1;  // Stable
package creto.messaging.v2;  // New version
```

**Deprecation timeline:**
1. **v1 (current):** Supported indefinitely
2. **v2 (new):** Released, v1 enters maintenance mode
3. **v1 (deprecated):** 6 months after v2 release
4. **v1 (sunset):** 12 months after v2 release (unless critical usage)

---

## Authentication

### NHI Token-Based Auth

**Flow:**
```
1. Agent authenticates with NHI service
   ↓
2. NHI issues JWT token (signed with ML-DSA)
   ↓
3. Agent includes token in gRPC metadata / HTTP header
   ↓
4. Messaging service verifies token with NHI public key
   ↓
5. Extract agent_nhi from token claims
```

**gRPC metadata:**
```rust
let mut request = tonic::Request::new(send_message_request);
request.metadata_mut().insert(
    "authorization",
    format!("Bearer {}", nhi_token).parse().unwrap(),
);

let response = client.send_message(request).await?;
```

**HTTP header:**
```http
POST /v1/messages
Authorization: Bearer eyJhbGciOiJNTC1EU0EtNjUiLCJ0eXAiOiJKV1QifQ...
```

**Token claims:**
```json
{
  "iss": "creto-nhi",
  "sub": "agent-alice-01",
  "aud": "creto-messaging",
  "exp": 1735171200,
  "iat": 1735167600,
  "nhi": "agent-alice-01",
  "capabilities": ["send_message", "receive_message", "create_channel"]
}
```

---

## Summary

### Key API Design Decisions

1. **gRPC-first:** Binary protocol for performance, streaming support
2. **REST gateway:** HTTP/JSON for compatibility
3. **Protocol buffers:** Versioned schemas, field evolution
4. **Structured errors:** ErrorCode enum + details map
5. **Rate limiting:** Per-agent quotas, Retry-After headers
6. **NHI authentication:** JWT tokens signed with ML-DSA
7. **Idempotency:** Idempotency keys for safe retries

### Performance Targets

| Operation | Latency (p99) | Throughput |
|-----------|---------------|------------|
| `SendMessage` | <5ms | >100K req/sec |
| `ReceiveMessages` | <10ms | >50K req/sec |
| `UploadKeyBundle` | <20ms | >10K req/sec |
| `CreateChannel` | <50ms | >1K req/sec |
| `Publish` | <15ms | >20K req/sec |

### Next Steps

1. **Security deep dive** (see 05-security.md)
2. **Test strategy** (see 06-testing.md)
3. **Operational runbooks** (see 07-runbook.md)
