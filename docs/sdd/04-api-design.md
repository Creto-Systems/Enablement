---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
---

# SDD-04: API Design

## Purpose

This document defines the API contracts for the Enablement Layer, including gRPC service definitions, REST endpoints (where applicable), and SDK interfaces.

## Scope

**In Scope:**
- gRPC service definitions per crate
- Error handling and status codes
- Authentication and authorization headers
- Rate limiting and quotas
- SDK client interfaces

**Out of Scope:**
- Internal crate-to-crate APIs (see 02-architecture.md)
- Platform/Security layer APIs

---

## 1. API Principles

### 1.1 Design Guidelines

| Principle | Implementation |
|-----------|----------------|
| **gRPC-first** | All services expose gRPC; REST via gRPC-gateway where needed |
| **Strongly typed** | Protocol Buffers for all payloads |
| **NHI-native** | All requests include agent identity context |
| **Idempotent** | All mutating operations support idempotency keys |
| **Observable** | All responses include request IDs for tracing |

### 1.2 Common Headers

```protobuf
// Included in all requests
message RequestContext {
    string request_id = 1;          // UUID, for tracing
    string idempotency_key = 2;     // Optional, for retries
    AgentIdentity agent_nhi = 3;    // Caller identity
    repeated AgentIdentity delegation_chain = 4;
    google.protobuf.Timestamp timestamp = 5;
}

// Included in all responses
message ResponseMetadata {
    string request_id = 1;
    google.protobuf.Duration latency = 2;
    string node_id = 3;             // Serving node
}
```

### 1.3 Error Model

```protobuf
message Error {
    ErrorCode code = 1;
    string message = 2;
    map<string, string> details = 3;
    string request_id = 4;
    RetryInfo retry_info = 5;
}

enum ErrorCode {
    ERROR_CODE_UNSPECIFIED = 0;

    // Client errors (4xx equivalent)
    INVALID_ARGUMENT = 1;
    NOT_FOUND = 2;
    ALREADY_EXISTS = 3;
    PERMISSION_DENIED = 4;
    QUOTA_EXCEEDED = 5;
    OVERSIGHT_REQUIRED = 6;
    RATE_LIMITED = 7;

    // Server errors (5xx equivalent)
    INTERNAL = 10;
    UNAVAILABLE = 11;
    TIMEOUT = 12;
}

message RetryInfo {
    google.protobuf.Duration retry_after = 1;
    bool retryable = 2;
}
```

---

## 2. Metering API

### 2.1 gRPC Service Definition

```protobuf
syntax = "proto3";
package creto.metering.v1;

service MeteringService {
    // Event ingestion
    rpc IngestEvent(IngestEventRequest) returns (IngestEventResponse);
    rpc IngestEventBatch(IngestEventBatchRequest) returns (IngestEventBatchResponse);

    // Quota management (called by Authorization inline)
    rpc CheckQuota(CheckQuotaRequest) returns (CheckQuotaResponse);
    rpc ReserveUsage(ReserveUsageRequest) returns (ReserveUsageResponse);
    rpc FinalizeUsage(FinalizeUsageRequest) returns (FinalizeUsageResponse);

    // Usage queries
    rpc GetUsage(GetUsageRequest) returns (GetUsageResponse);
    rpc GetUsageBreakdown(GetUsageBreakdownRequest) returns (GetUsageBreakdownResponse);

    // Invoice management
    rpc GenerateInvoice(GenerateInvoiceRequest) returns (GenerateInvoiceResponse);
    rpc GetInvoice(GetInvoiceRequest) returns (GetInvoiceResponse);
    rpc ListInvoices(ListInvoicesRequest) returns (ListInvoicesResponse);
}
```

### 2.2 Event Ingestion

```protobuf
message IngestEventRequest {
    RequestContext context = 1;
    BillableEvent event = 2;
}

message BillableEvent {
    string idempotency_key = 1;
    string event_type = 2;
    map<string, PropertyValue> properties = 3;
    google.protobuf.Timestamp timestamp = 4;
    string subscription_id = 5;
    bytes signature = 6;            // Optional ML-DSA signature
}

message PropertyValue {
    oneof value {
        string string_value = 1;
        int64 int_value = 2;
        double float_value = 3;
        bool bool_value = 4;
    }
}

message IngestEventResponse {
    ResponseMetadata metadata = 1;
    string event_id = 2;
    QuotaStatus quota_status = 3;   // Current quota state after event
}

message QuotaStatus {
    int64 used = 1;
    int64 limit = 2;
    float percentage = 3;
    google.protobuf.Timestamp resets_at = 4;
}
```

### 2.3 Quota Enforcement

```protobuf
message CheckQuotaRequest {
    RequestContext context = 1;
    string event_type = 2;
    int64 requested_units = 3;      // How many units this action will consume
}

message CheckQuotaResponse {
    ResponseMetadata metadata = 1;
    QuotaDecision decision = 2;
}

message QuotaDecision {
    bool allowed = 1;
    int64 remaining = 2;
    string denial_reason = 3;       // If not allowed
    google.protobuf.Duration retry_after = 4;
    string reservation_id = 5;      // If allowed, use to finalize
}

message FinalizeUsageRequest {
    RequestContext context = 1;
    string reservation_id = 2;
    int64 actual_units = 3;         // May differ from reserved
    bool success = 4;               // Did the action succeed?
}
```

---

## 3. Oversight API

### 3.1 gRPC Service Definition

```protobuf
syntax = "proto3";
package creto.oversight.v1;

service OversightService {
    // Request management (called by Authorization)
    rpc CreateRequest(CreateRequestRequest) returns (CreateRequestResponse);
    rpc GetRequest(GetRequestRequest) returns (GetRequestResponse);
    rpc CancelRequest(CancelRequestRequest) returns (CancelRequestResponse);

    // Blocking wait (for synchronous flows)
    rpc WaitForResolution(WaitForResolutionRequest) returns (WaitForResolutionResponse);

    // Response submission (called by channel handlers)
    rpc SubmitResponse(SubmitResponseRequest) returns (SubmitResponseResponse);

    // Queries
    rpc ListPendingRequests(ListPendingRequestsRequest) returns (ListPendingRequestsResponse);
    rpc GetRequestHistory(GetRequestHistoryRequest) returns (GetRequestHistoryResponse);
}

service NotificationService {
    // Channel management
    rpc RegisterChannel(RegisterChannelRequest) returns (RegisterChannelResponse);
    rpc TestChannel(TestChannelRequest) returns (TestChannelResponse);

    // Webhook receiver (for channel callbacks)
    rpc HandleWebhook(HandleWebhookRequest) returns (HandleWebhookResponse);
}
```

### 3.2 Request Creation

```protobuf
message CreateRequestRequest {
    RequestContext context = 1;
    Action pending_action = 2;
    string resource = 3;
    string policy_id = 4;
    string policy_trigger_reason = 5;
    RequestContextInfo request_context = 6;
}

message Action {
    string action_type = 1;
    string description = 2;
    map<string, string> parameters = 3;
    Money estimated_cost = 4;
    bool reversible = 5;
}

message RequestContextInfo {
    string action_description = 1;
    string reasoning = 2;
    repeated MemorySnippet memory_context = 3;
    repeated RiskFactor risk_factors = 4;
}

message CreateRequestResponse {
    ResponseMetadata metadata = 1;
    string request_id = 2;
    google.protobuf.Timestamp expires_at = 3;
    repeated string notification_channels = 4;
}
```

### 3.3 Response Submission

```protobuf
message SubmitResponseRequest {
    RequestContext context = 1;
    string request_id = 2;
    ApprovalDecision decision = 3;
    string reason = 4;
    string channel_id = 5;
    bytes approver_signature = 6;   // ML-DSA signature
    HumanIdentity approver = 7;
}

enum ApprovalDecision {
    APPROVAL_DECISION_UNSPECIFIED = 0;
    APPROVE = 1;
    DENY = 2;
    ESCALATE = 3;
    REQUEST_INFO = 4;
}

message SubmitResponseResponse {
    ResponseMetadata metadata = 1;
    OversightState new_state = 2;
    bool action_can_proceed = 3;
}

message OversightState {
    oneof state {
        PendingState pending = 1;
        ApprovedState approved = 2;
        DeniedState denied = 3;
        EscalatedState escalated = 4;
        TimedOutState timed_out = 5;
    }
}
```

---

## 4. Runtime API

### 4.1 gRPC Service Definition

```protobuf
syntax = "proto3";
package creto.runtime.v1;

service RuntimeService {
    // Sandbox lifecycle
    rpc SpawnSandbox(SpawnSandboxRequest) returns (SpawnSandboxResponse);
    rpc ClaimFromPool(ClaimFromPoolRequest) returns (ClaimFromPoolResponse);
    rpc TerminateSandbox(TerminateSandboxRequest) returns (TerminateSandboxResponse);

    // Execution
    rpc Exec(ExecRequest) returns (ExecResponse);
    rpc ExecStream(ExecStreamRequest) returns (stream ExecStreamResponse);

    // State management
    rpc CheckpointSandbox(CheckpointSandboxRequest) returns (CheckpointSandboxResponse);
    rpc RestoreSandbox(RestoreSandboxRequest) returns (RestoreSandboxResponse);

    // Attestation
    rpc GetAttestation(GetAttestationRequest) returns (GetAttestationResponse);
    rpc VerifyAttestation(VerifyAttestationRequest) returns (VerifyAttestationResponse);

    // Pool management
    rpc CreatePool(CreatePoolRequest) returns (CreatePoolResponse);
    rpc DeletePool(DeletePoolRequest) returns (DeletePoolResponse);
    rpc GetPoolStats(GetPoolStatsRequest) returns (GetPoolStatsResponse);
}
```

### 4.2 Sandbox Operations

```protobuf
message SpawnSandboxRequest {
    RequestContext context = 1;
    SandboxSpec spec = 2;
}

message SandboxSpec {
    string image = 1;
    ImagePullPolicy pull_policy = 2;
    ResourceLimits resources = 3;
    NetworkPolicy network = 4;
    repeated SecretRef secrets = 5;
    map<string, string> env_vars = 6;
    google.protobuf.Duration ttl = 7;
    AttestationPolicy attestation_policy = 8;
}

message ResourceLimits {
    uint32 cpu_millicores = 1;
    uint64 memory_bytes = 2;
    uint64 storage_bytes = 3;
}

message SpawnSandboxResponse {
    ResponseMetadata metadata = 1;
    string sandbox_id = 2;
    SandboxStatus status = 3;
    Attestation attestation = 4;
}

message ClaimFromPoolRequest {
    RequestContext context = 1;
    string pool_id = 2;
}

message ClaimFromPoolResponse {
    ResponseMetadata metadata = 1;
    string sandbox_id = 2;
    google.protobuf.Duration claim_latency = 3;
    bool was_cold_start = 4;
    Attestation attestation = 5;
}
```

### 4.3 Execution

```protobuf
message ExecRequest {
    RequestContext context = 1;
    string sandbox_id = 2;
    repeated string command = 3;
    map<string, string> env = 4;
    string working_dir = 5;
    google.protobuf.Duration timeout = 6;
}

message ExecResponse {
    ResponseMetadata metadata = 1;
    int32 exit_code = 2;
    bytes stdout = 3;
    bytes stderr = 4;
    google.protobuf.Duration execution_time = 5;
}

// Streaming execution
message ExecStreamRequest {
    RequestContext context = 1;
    string sandbox_id = 2;
    repeated string command = 3;
    bytes stdin = 4;                // Incremental stdin
}

message ExecStreamResponse {
    oneof output {
        bytes stdout = 1;
        bytes stderr = 2;
        int32 exit_code = 3;
    }
}
```

---

## 5. Messaging API

### 5.1 gRPC Service Definition

```protobuf
syntax = "proto3";
package creto.messaging.v1;

service MessagingService {
    // Sending
    rpc Send(SendRequest) returns (SendResponse);
    rpc SendAndWait(SendAndWaitRequest) returns (SendAndWaitResponse);
    rpc Publish(PublishRequest) returns (PublishResponse);

    // Receiving
    rpc Receive(ReceiveRequest) returns (stream MessageEnvelope);
    rpc Acknowledge(AcknowledgeRequest) returns (AcknowledgeResponse);
    rpc Nack(NackRequest) returns (NackResponse);

    // Topics
    rpc CreateTopic(CreateTopicRequest) returns (CreateTopicResponse);
    rpc DeleteTopic(DeleteTopicRequest) returns (DeleteTopicResponse);
    rpc Subscribe(SubscribeRequest) returns (SubscribeResponse);
    rpc Unsubscribe(UnsubscribeRequest) returns (UnsubscribeResponse);

    // Key management (delegates to NHI)
    rpc GetPublicKey(GetPublicKeyRequest) returns (GetPublicKeyResponse);
}
```

### 5.2 Message Operations

```protobuf
message SendRequest {
    RequestContext context = 1;
    AgentIdentity recipient = 2;
    bytes payload = 3;
    SendOptions options = 4;
}

message SendOptions {
    ContentType content_type = 1;
    MessagePriority priority = 2;
    google.protobuf.Duration ttl = 3;
    bool require_ack = 4;
    string correlation_id = 5;
}

message SendResponse {
    ResponseMetadata metadata = 1;
    string message_id = 2;
    DeliveryStatus delivery_status = 3;
}

enum DeliveryStatus {
    DELIVERY_STATUS_UNSPECIFIED = 0;
    QUEUED = 1;
    DELIVERED = 2;
    FAILED = 3;
}

message SendAndWaitRequest {
    RequestContext context = 1;
    AgentIdentity recipient = 2;
    bytes payload = 3;
    google.protobuf.Duration timeout = 4;
    SendOptions options = 5;
}

message SendAndWaitResponse {
    ResponseMetadata metadata = 1;
    string message_id = 2;
    bytes response_payload = 3;
    AgentIdentity responder = 4;
    google.protobuf.Duration round_trip_time = 5;
}
```

### 5.3 Message Envelope (Wire Format)

```protobuf
message MessageEnvelope {
    string message_id = 1;
    string correlation_id = 2;

    AgentIdentity sender = 3;
    MessageRecipient recipient = 4;

    bytes encrypted_payload = 5;
    bytes encryption_nonce = 6;
    bytes wrapped_key = 7;

    bytes signature_ed25519 = 8;
    bytes signature_ml_dsa = 9;

    google.protobuf.Timestamp timestamp = 10;
    google.protobuf.Duration ttl = 11;
    ContentType content_type = 12;
    MessagePriority priority = 13;
}

message MessageRecipient {
    oneof target {
        AgentIdentity agent = 1;
        string topic_id = 2;
        MulticastGroup multicast = 3;
    }
}
```

---

## 6. SDK Interfaces

### 6.1 Rust SDK

```rust
// creto-metering-client
pub struct MeteringClient { /* ... */ }

impl MeteringClient {
    pub async fn ingest_event(&self, event: BillableEvent) -> Result<EventId, Error>;
    pub async fn check_quota(&self, event_type: &str, units: u64) -> Result<QuotaDecision, Error>;
    pub async fn get_usage(&self, period: &BillingPeriod) -> Result<UsageSummary, Error>;
}

// creto-oversight-client
pub struct OversightClient { /* ... */ }

impl OversightClient {
    pub async fn create_request(&self, action: &Action) -> Result<RequestId, Error>;
    pub async fn wait_for_resolution(&self, id: &RequestId, timeout: Duration) -> Result<OversightState, Error>;
    pub async fn submit_response(&self, id: &RequestId, response: ApprovalResponse) -> Result<OversightState, Error>;
}

// creto-runtime-client
pub struct RuntimeClient { /* ... */ }

impl RuntimeClient {
    pub async fn spawn(&self, spec: SandboxSpec) -> Result<SandboxHandle, Error>;
    pub async fn claim(&self, pool_id: &PoolId) -> Result<SandboxHandle, Error>;
    pub async fn exec(&self, sandbox_id: &SandboxId, cmd: &[String]) -> Result<ExecOutput, Error>;
    pub async fn terminate(&self, sandbox_id: &SandboxId) -> Result<(), Error>;
}

// creto-messaging-client
pub struct MessagingClient { /* ... */ }

impl MessagingClient {
    pub async fn send(&self, to: &AgentIdentity, payload: &[u8]) -> Result<MessageId, Error>;
    pub async fn request(&self, to: &AgentIdentity, payload: &[u8], timeout: Duration) -> Result<Response, Error>;
    pub async fn subscribe(&self, topic: &TopicId) -> Result<Subscription, Error>;
    pub async fn receive(&self) -> Result<ReceivedMessage, Error>;
}
```

### 6.2 Python SDK

```python
# creto_metering
class MeteringClient:
    async def ingest_event(self, event: BillableEvent) -> EventId: ...
    async def check_quota(self, event_type: str, units: int) -> QuotaDecision: ...

# creto_oversight
class OversightClient:
    async def create_request(self, action: Action) -> RequestId: ...
    async def wait_for_resolution(self, request_id: RequestId, timeout: timedelta) -> OversightState: ...

# creto_runtime
class RuntimeClient:
    async def spawn(self, spec: SandboxSpec) -> SandboxHandle: ...
    @asynccontextmanager
    async def sandbox(self, spec: SandboxSpec) -> AsyncIterator[SandboxHandle]: ...

# creto_messaging
class MessagingClient:
    async def send(self, to: AgentIdentity, payload: bytes) -> MessageId: ...
    async def request(self, to: AgentIdentity, payload: bytes, timeout: timedelta) -> Response: ...
```

---

## 7. Rate Limiting

### 7.1 Per-Service Limits

| Service | Default Rate | Burst | Scope |
|---------|--------------|-------|-------|
| Metering.IngestEvent | 10,000/s | 50,000 | Per subscription |
| Metering.CheckQuota | 100,000/s | 500,000 | Per agent |
| Oversight.CreateRequest | 100/s | 500 | Per agent |
| Runtime.SpawnSandbox | 10/s | 50 | Per agent |
| Runtime.ClaimFromPool | 1,000/s | 5,000 | Per pool |
| Messaging.Send | 10,000/s | 50,000 | Per sender |

### 7.2 Rate Limit Headers

```protobuf
message RateLimitInfo {
    int64 limit = 1;
    int64 remaining = 2;
    google.protobuf.Timestamp reset_at = 3;
    google.protobuf.Duration retry_after = 4;  // If rate limited
}
```

---

## 8. Versioning

### 8.1 API Versioning Strategy

- **Major versions** in package name: `creto.metering.v1`, `creto.metering.v2`
- **Minor versions** via additive changes only
- **Deprecation**: 6-month notice before removal

### 8.2 Version Negotiation

```protobuf
message VersionInfo {
    string api_version = 1;         // e.g., "v1"
    string server_version = 2;      // e.g., "1.2.3"
    repeated string supported_versions = 3;
}
```

---

## 9. Decisions

| Decision | Rationale |
|----------|-----------|
| gRPC-first | Performance, streaming, strong typing |
| Protocol Buffers | Schema evolution, multi-language |
| Per-agent rate limits | Fair usage, prevent abuse |
| Streaming for exec | Real-time output, long-running commands |

---

## 10. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
