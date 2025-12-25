---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# SDD-MTR-04: Metering API Design

## Overview

This document specifies the API interfaces for **creto-metering**, including:

1. **gRPC API**: High-performance event ingestion and quota checks (internal)
2. **REST API**: HTTP/JSON for external integrations and dashboards
3. **Library API**: Rust trait interfaces for in-process integration

All APIs follow consistent authentication (via creto-authz), error handling, and versioning patterns.

## API Principles

1. **Idempotency**: All mutation operations (POST, PUT) are idempotent via `idempotency-key`
2. **Versioning**: URL-based versioning (`/v1/events`, `/v2/events`)
3. **Error handling**: Consistent error response format with machine-readable codes
4. **Rate limiting**: 1000 requests/minute per subscription (configurable)
5. **Pagination**: Cursor-based pagination for list endpoints

## gRPC API (Internal, High-Performance)

### Proto Definitions

```protobuf
syntax = "proto3";

package creto.metering.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/struct.proto";

// ============================================================================
// Event Ingestion Service
// ============================================================================

service EventIngestionService {
  // Ingest a single billable event
  rpc IngestEvent(IngestEventRequest) returns (IngestEventResponse);

  // Ingest batch of events (up to 1000)
  rpc IngestBatch(IngestBatchRequest) returns (IngestBatchResponse);

  // Get event by ID (for audit/dispute resolution)
  rpc GetEvent(GetEventRequest) returns (Event);

  // Query events (paginated)
  rpc QueryEvents(QueryEventsRequest) returns (QueryEventsResponse);
}

message IngestEventRequest {
  // Deduplication key (client-generated UUID)
  string idempotency_key = 1;

  // Agent identity (cryptographic NHI)
  string agent_nhi = 2;

  // Delegation chain (from leaf agent to root human)
  repeated string delegation_chain = 3;

  // Event classification
  string event_type = 4;

  // Consensus-ordered timestamp (optional, server assigns if missing)
  google.protobuf.Timestamp timestamp = 5;

  // Flexible properties (JSON)
  google.protobuf.Struct properties = 6;

  // Cryptographic signature (ML-DSA-65)
  bytes signature = 7;
  string signature_algorithm = 8; // Default: "ML-DSA-65"
}

message IngestEventResponse {
  // Unique event ID (server-assigned)
  string event_id = 1;

  // Response status
  EventStatus status = 2;

  enum EventStatus {
    CREATED = 0;    // 201: New event created
    ACCEPTED = 1;   // 202: Duplicate (idempotent)
    CONFLICT = 2;   // 409: Same key, different data
  }

  // For CONFLICT: hash of existing event
  string existing_hash = 3;
}

message IngestBatchRequest {
  repeated IngestEventRequest events = 1;
}

message IngestBatchResponse {
  string batch_id = 1;
  int32 total = 2;
  int32 succeeded = 3;
  int32 failed = 4;

  repeated BatchItemResult results = 5;
}

message BatchItemResult {
  string idempotency_key = 1;

  oneof result {
    IngestEventResponse success = 2;
    ErrorDetail error = 3;
  }
}

message GetEventRequest {
  string event_id = 1;
}

message Event {
  string event_id = 1;
  string idempotency_key = 2;
  string agent_nhi = 3;
  repeated string delegation_chain = 4;
  string subscription_id = 5; // Resolved server-side
  string event_type = 6;
  google.protobuf.Timestamp timestamp = 7;
  google.protobuf.Struct properties = 8;
  bytes signature = 9;
  string audit_log_id = 10; // Reference to creto-audit
  google.protobuf.Timestamp created_at = 11;
}

message QueryEventsRequest {
  string subscription_id = 1;
  string event_type = 2; // Optional filter
  google.protobuf.Timestamp start_time = 3;
  google.protobuf.Timestamp end_time = 4;
  int32 page_size = 5; // Default: 100, max: 1000
  string page_token = 6; // Cursor for pagination
}

message QueryEventsResponse {
  repeated Event events = 1;
  string next_page_token = 2;
  int32 total_count = 3;
}

// ============================================================================
// Quota Enforcement Service
// ============================================================================

service QuotaEnforcementService {
  // Check if agent has quota (called from Authorization)
  rpc CheckQuota(CheckQuotaRequest) returns (CheckQuotaResponse);

  // Reserve quota for upcoming action
  rpc ReserveQuota(ReserveQuotaRequest) returns (ReserveQuotaResponse);

  // Commit or rollback reservation
  rpc CommitReservation(CommitReservationRequest) returns (CommitReservationResponse);
  rpc RollbackReservation(RollbackReservationRequest) returns (RollbackReservationResponse);

  // Get current quota usage
  rpc GetQuotaUsage(GetQuotaUsageRequest) returns (GetQuotaUsageResponse);
}

message CheckQuotaRequest {
  string agent_nhi = 1;
  string event_type = 2;
}

message CheckQuotaResponse {
  QuotaDecision decision = 1;

  message QuotaDecision {
    bool allowed = 1;

    // If allowed
    int64 remaining = 2;
    google.protobuf.Timestamp next_reset = 3;

    // If denied
    QuotaExceededReason reason = 4;
    google.protobuf.Duration retry_after = 5;
    int64 current_usage = 6;
    int64 limit = 7;
  }

  enum QuotaExceededReason {
    LIMIT_REACHED = 0;
    SUBSCRIPTION_SUSPENDED = 1;
    QUOTA_NOT_CONFIGURED = 2;
  }
}

message ReserveQuotaRequest {
  string agent_nhi = 1;
  string event_type = 2;
  int64 quantity = 3; // Default: 1
}

message ReserveQuotaResponse {
  string reservation_id = 1;
  google.protobuf.Timestamp expires_at = 2; // 5 minutes
}

message CommitReservationRequest {
  string reservation_id = 1;
}

message CommitReservationResponse {
  bool committed = 1;
}

message RollbackReservationRequest {
  string reservation_id = 1;
}

message RollbackReservationResponse {
  bool rolled_back = 1;
}

message GetQuotaUsageRequest {
  string subscription_id = 1;
  string event_type = 2;
  QuotaPeriod period = 3;

  enum QuotaPeriod {
    HOURLY = 0;
    DAILY = 1;
    MONTHLY = 2;
    TOTAL = 3;
  }
}

message GetQuotaUsageResponse {
  int64 current_usage = 1;
  int64 limit = 2;
  int64 remaining = 3;
  google.protobuf.Timestamp period_start = 4;
  google.protobuf.Timestamp period_end = 5;
}

// ============================================================================
// Aggregation Service
// ============================================================================

service AggregationService {
  // Get usage for billing period
  rpc GetUsage(GetUsageRequest) returns (GetUsageResponse);

  // Get cost attribution breakdown
  rpc GetAttribution(GetAttributionRequest) returns (GetAttributionResponse);
}

message GetUsageRequest {
  string subscription_id = 1;
  string event_type = 2;
  google.protobuf.Timestamp period_start = 3;
  google.protobuf.Timestamp period_end = 4;
}

message GetUsageResponse {
  string event_type = 1;
  int64 count = 2; // COUNT(*)
  double sum_value = 3; // SUM(property)
  int64 unique_count = 4; // COUNT(DISTINCT property)
  double max_value = 5; // MAX(property)

  // Breakdown by dimensions
  map<string, DimensionUsage> by_dimension = 6;
}

message DimensionUsage {
  int64 count = 1;
  double sum_value = 2;
}

message GetAttributionRequest {
  string subscription_id = 1;
  google.protobuf.Timestamp period_start = 2;
  google.protobuf.Timestamp period_end = 3;
}

message GetAttributionResponse {
  map<string, double> by_agent = 1; // agent_nhi -> cost
  map<string, DimensionAttribution> by_dimension = 2; // dimension -> {value -> cost}
}

message DimensionAttribution {
  map<string, double> costs = 1; // value -> cost
}

// ============================================================================
// Billing Service
// ============================================================================

service BillingService {
  // Generate invoice for billing period
  rpc GenerateInvoice(GenerateInvoiceRequest) returns (Invoice);

  // Finalize invoice (mark as issued, trigger payment)
  rpc FinalizeInvoice(FinalizeInvoiceRequest) returns (FinalizeInvoiceResponse);

  // Get invoice by ID
  rpc GetInvoice(GetInvoiceRequest) returns (Invoice);

  // List invoices for subscription
  rpc ListInvoices(ListInvoicesRequest) returns (ListInvoicesResponse);
}

message GenerateInvoiceRequest {
  string subscription_id = 1;
  google.protobuf.Timestamp period_start = 2;
  google.protobuf.Timestamp period_end = 3;
}

message Invoice {
  string invoice_id = 1;
  string subscription_id = 2;
  google.protobuf.Timestamp period_start = 3;
  google.protobuf.Timestamp period_end = 4;

  repeated LineItem line_items = 5;

  double subtotal = 6;
  double tax = 7;
  double total = 8;
  string currency = 9; // ISO 4217 (e.g., "USD")

  google.protobuf.Struct attribution = 10; // Cost breakdown

  InvoiceStatus status = 11;
  google.protobuf.Timestamp issued_at = 12;
  google.protobuf.Timestamp paid_at = 13;

  string stripe_invoice_id = 14;
  string stripe_payment_intent_id = 15;

  enum InvoiceStatus {
    DRAFT = 0;
    ISSUED = 1;
    PAID = 2;
    VOID = 3;
    DISPUTED = 4;
  }
}

message LineItem {
  string description = 1;
  string metric_code = 2;
  double quantity = 3;
  double unit_price = 4;
  double amount = 5;
}

message FinalizeInvoiceRequest {
  string invoice_id = 1;
}

message FinalizeInvoiceResponse {
  bool finalized = 1;
  string stripe_invoice_url = 2;
}

message GetInvoiceRequest {
  string invoice_id = 1;
}

message ListInvoicesRequest {
  string subscription_id = 1;
  int32 page_size = 2;
  string page_token = 3;
}

message ListInvoicesResponse {
  repeated Invoice invoices = 1;
  string next_page_token = 2;
}

// ============================================================================
// Common Error Type
// ============================================================================

message ErrorDetail {
  string code = 1; // Machine-readable error code
  string message = 2; // Human-readable message
  map<string, string> metadata = 3; // Additional context
}
```

### gRPC Server Implementation (Rust)

```rust
use tonic::{Request, Response, Status};
use creto_metering_proto::v1::{
    event_ingestion_service_server::EventIngestionService,
    IngestEventRequest, IngestEventResponse, IngestBatchRequest, IngestBatchResponse,
};

pub struct EventIngestionServiceImpl {
    service: Arc<crate::EventIngestionService>,
}

#[tonic::async_trait]
impl EventIngestionService for EventIngestionServiceImpl {
    async fn ingest_event(
        &self,
        request: Request<IngestEventRequest>,
    ) -> Result<Response<IngestEventResponse>, Status> {
        let req = request.into_inner();

        // Convert proto to domain model
        let event_request = crate::EventRequest::try_from(req)
            .map_err(|e| Status::invalid_argument(format!("Invalid request: {}", e)))?;

        // Call service layer
        match self.service.ingest_event(event_request).await {
            Ok(response) => Ok(Response::new(response.into())),
            Err(e) => Err(Status::from_error(e)),
        }
    }

    async fn ingest_batch(
        &self,
        request: Request<IngestBatchRequest>,
    ) -> Result<Response<IngestBatchResponse>, Status> {
        let req = request.into_inner();

        if req.events.len() > 1000 {
            return Err(Status::invalid_argument("Batch size exceeds 1000 events"));
        }

        // Convert and process
        let batch_request = crate::BatchRequest::try_from(req)
            .map_err(|e| Status::invalid_argument(format!("Invalid batch: {}", e)))?;

        match self.service.ingest_batch(batch_request).await {
            Ok(response) => Ok(Response::new(response.into())),
            Err(e) => Err(Status::from_error(e)),
        }
    }
}
```

---

## REST API (External, HTTP/JSON)

### Base URL

- **Production**: `https://api.creto.io/metering/v1`
- **Staging**: `https://api-staging.creto.io/metering/v1`

### Authentication

All requests require Bearer token from Authorization service:

```http
Authorization: Bearer <jwt_token>
```

**Token Claims**:
```json
{
  "sub": "agent:nhi:ed25519:abc123...",
  "iss": "creto-authz",
  "aud": "creto-metering",
  "exp": 1735142400,
  "scope": "events:write quotas:read"
}
```

### Common Headers

```http
Content-Type: application/json
X-Request-ID: <uuid>        # For tracing
Idempotency-Key: <uuid>     # For POST/PUT requests
```

---

### POST /v1/events

**Purpose**: Ingest a single billable event.

**Request**:
```json
{
  "idempotency_key": "550e8400-e29b-41d4-a716-446655440000",
  "agent_nhi": "agent:nhi:ed25519:embed-worker-42",
  "delegation_chain": [
    "agent:nhi:ed25519:scheduler-001",
    "human:ops-team@example.com"
  ],
  "event_type": "llm_tokens",
  "timestamp": "2024-12-25T10:30:00Z", // Optional
  "properties": {
    "tokens": 1500,
    "model": "gpt-4",
    "prompt_tokens": 1000,
    "completion_tokens": 500,
    "region": "us-east-1"
  },
  "signature": "base64encodedMLDSASignature...",
  "signature_algorithm": "ML-DSA-65"
}
```

**Response (201 Created)**:
```json
{
  "event_id": "evt_2024_12_25_abc123",
  "status": "created",
  "timestamp": "2024-12-25T10:30:00.123Z"
}
```

**Response (202 Accepted - Duplicate)**:
```json
{
  "event_id": "evt_2024_12_25_abc123",
  "status": "accepted",
  "message": "Event already exists (idempotent)"
}
```

**Response (409 Conflict)**:
```json
{
  "error": {
    "code": "IDEMPOTENCY_CONFLICT",
    "message": "Event with same idempotency_key has different data",
    "metadata": {
      "existing_hash": "sha256:abcdef...",
      "submitted_hash": "sha256:123456..."
    }
  }
}
```

**cURL Example**:
```bash
curl -X POST https://api.creto.io/metering/v1/events \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -H "Idempotency-Key: 550e8400-e29b-41d4-a716-446655440000" \
  -d '{
    "agent_nhi": "agent:nhi:ed25519:embed-worker-42",
    "delegation_chain": ["agent:scheduler", "human:alice"],
    "event_type": "llm_tokens",
    "properties": {
      "tokens": 1500,
      "model": "gpt-4"
    },
    "signature": "base64..."
  }'
```

---

### POST /v1/events/batch

**Purpose**: Ingest batch of events (up to 1000).

**Request**:
```json
{
  "events": [
    {
      "idempotency_key": "event-1",
      "agent_nhi": "agent:worker-1",
      "delegation_chain": ["human:alice"],
      "event_type": "api_call",
      "properties": {"method": "POST"},
      "signature": "base64..."
    },
    {
      "idempotency_key": "event-2",
      "agent_nhi": "agent:worker-2",
      "delegation_chain": ["human:bob"],
      "event_type": "api_call",
      "properties": {"method": "GET"},
      "signature": "base64..."
    }
  ]
}
```

**Response (207 Multi-Status)**:
```json
{
  "batch_id": "batch_abc123",
  "total": 2,
  "succeeded": 1,
  "failed": 1,
  "results": [
    {
      "idempotency_key": "event-1",
      "status": "created",
      "event_id": "evt_001"
    },
    {
      "idempotency_key": "event-2",
      "status": "failed",
      "error": {
        "code": "INVALID_SIGNATURE",
        "message": "ML-DSA signature verification failed"
      }
    }
  ]
}
```

---

### GET /v1/events/:event_id

**Purpose**: Retrieve event by ID (for audit/dispute resolution).

**Response (200 OK)**:
```json
{
  "event_id": "evt_2024_12_25_abc123",
  "idempotency_key": "550e8400-e29b-41d4-a716-446655440000",
  "agent_nhi": "agent:nhi:ed25519:embed-worker-42",
  "delegation_chain": ["agent:scheduler", "human:alice"],
  "subscription_id": "sub_abc123",
  "event_type": "llm_tokens",
  "timestamp": "2024-12-25T10:30:00Z",
  "properties": {
    "tokens": 1500,
    "model": "gpt-4"
  },
  "signature": "base64...",
  "audit_log_id": "audit_xyz789",
  "created_at": "2024-12-25T10:30:00.123Z"
}
```

---

### GET /v1/usage/:subscription_id

**Purpose**: Get current usage for subscription.

**Query Parameters**:
- `event_type` (required): Metric code (e.g., `llm_tokens`)
- `period_start` (optional): ISO8601 timestamp
- `period_end` (optional): ISO8601 timestamp

**Response (200 OK)**:
```json
{
  "subscription_id": "sub_abc123",
  "event_type": "llm_tokens",
  "period": {
    "start": "2024-12-01T00:00:00Z",
    "end": "2024-12-31T23:59:59Z"
  },
  "usage": {
    "count": 15000,
    "sum_tokens": 2500000,
    "unique_models": ["gpt-4", "gpt-3.5-turbo"],
    "max_latency_ms": 450
  },
  "by_dimension": {
    "model": {
      "gpt-4": {
        "count": 10000,
        "sum_tokens": 2000000
      },
      "gpt-3.5-turbo": {
        "count": 5000,
        "sum_tokens": 500000
      }
    },
    "region": {
      "us-east-1": {
        "count": 12000,
        "sum_tokens": 2000000
      },
      "eu-west-1": {
        "count": 3000,
        "sum_tokens": 500000
      }
    }
  }
}
```

---

### GET /v1/quotas/:agent_id

**Purpose**: Get quota status for agent (called by agent or dashboard).

**Query Parameters**:
- `event_type` (required): e.g., `api_calls`

**Response (200 OK)**:
```json
{
  "agent_id": "agent:nhi:ed25519:worker-1",
  "subscription_id": "sub_abc123",
  "event_type": "api_calls",
  "quota": {
    "limit": 1000,
    "current_usage": 450,
    "remaining": 550,
    "period": "hourly",
    "period_start": "2024-12-25T10:00:00Z",
    "period_end": "2024-12-25T11:00:00Z",
    "overflow_action": "block"
  },
  "next_reset": "2024-12-25T11:00:00Z"
}
```

**Response (429 Too Many Requests - Quota Exceeded)**:
```json
{
  "error": {
    "code": "QUOTA_EXCEEDED",
    "message": "Hourly quota of 1000 api_calls exceeded",
    "metadata": {
      "limit": 1000,
      "current_usage": 1001,
      "period": "hourly",
      "retry_after": 1800
    }
  }
}
```

---

### POST /v1/invoices

**Purpose**: Generate invoice for subscription (admin/billing service).

**Request**:
```json
{
  "subscription_id": "sub_abc123",
  "period_start": "2024-12-01T00:00:00Z",
  "period_end": "2024-12-31T23:59:59Z"
}
```

**Response (201 Created)**:
```json
{
  "invoice_id": "inv_2024_12_001",
  "subscription_id": "sub_abc123",
  "period": {
    "start": "2024-12-01T00:00:00Z",
    "end": "2024-12-31T23:59:59Z"
  },
  "line_items": [
    {
      "description": "LLM Tokens (Tiered Graduated Pricing)",
      "metric_code": "llm_tokens",
      "quantity": 2500000,
      "unit_price": null,
      "amount": 12750.00
    },
    {
      "description": "API Calls (Per-Unit Pricing)",
      "metric_code": "api_calls",
      "quantity": 15000,
      "unit_price": 0.002,
      "amount": 30.00
    }
  ],
  "subtotal": 12780.00,
  "tax": 1150.20,
  "total": 13930.20,
  "currency": "USD",
  "attribution": {
    "by_agent": {
      "agent:worker-1": 8000.00,
      "agent:worker-2": 4780.00
    },
    "by_dimension": {
      "model": {
        "gpt-4": 10000.00,
        "gpt-3.5-turbo": 2780.00
      }
    }
  },
  "status": "draft",
  "created_at": "2024-12-31T23:59:59Z"
}
```

---

### GET /v1/invoices/:invoice_id

**Purpose**: Retrieve invoice details.

**Response**: Same as POST /v1/invoices (201 response)

---

### POST /v1/invoices/:invoice_id/finalize

**Purpose**: Mark invoice as issued and trigger payment.

**Response (200 OK)**:
```json
{
  "invoice_id": "inv_2024_12_001",
  "status": "issued",
  "stripe_invoice_url": "https://invoice.stripe.com/i/acct_xxx/inv_yyy",
  "issued_at": "2025-01-01T00:00:00Z"
}
```

---

### GET /v1/attribution/:subscription_id

**Purpose**: Get cost attribution breakdown.

**Query Parameters**:
- `period_start` (required)
- `period_end` (required)

**Response (200 OK)**:
```json
{
  "subscription_id": "sub_abc123",
  "period": {
    "start": "2024-12-01T00:00:00Z",
    "end": "2024-12-31T23:59:59Z"
  },
  "by_agent": {
    "agent:nhi:ed25519:worker-1": {
      "cost": 8000.00,
      "usage": {
        "llm_tokens": 1600000,
        "api_calls": 10000
      }
    },
    "agent:nhi:ed25519:worker-2": {
      "cost": 4780.00,
      "usage": {
        "llm_tokens": 900000,
        "api_calls": 5000
      }
    }
  },
  "by_dimension": {
    "model": {
      "gpt-4": 10000.00,
      "gpt-3.5-turbo": 2780.00
    },
    "region": {
      "us-east-1": 9000.00,
      "eu-west-1": 3780.00
    }
  },
  "total_cost": 12780.00
}
```

---

## Error Response Format

All errors follow consistent JSON structure:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "metadata": {
      "field": "Additional context"
    },
    "request_id": "req_abc123",
    "timestamp": "2024-12-25T10:30:00Z"
  }
}
```

### Error Codes

| HTTP Status | Error Code | Description |
|-------------|------------|-------------|
| 400 | `INVALID_REQUEST` | Malformed request body |
| 400 | `INVALID_SIGNATURE` | ML-DSA signature verification failed |
| 400 | `TIMESTAMP_SKEW` | Event timestamp >10min from server time |
| 401 | `UNAUTHORIZED` | Missing or invalid Bearer token |
| 403 | `FORBIDDEN` | Insufficient permissions |
| 404 | `NOT_FOUND` | Resource not found |
| 409 | `IDEMPOTENCY_CONFLICT` | Same idempotency_key, different data |
| 413 | `PAYLOAD_TOO_LARGE` | Batch size >1000 events |
| 429 | `QUOTA_EXCEEDED` | Spending limit reached |
| 429 | `RATE_LIMIT_EXCEEDED` | API rate limit exceeded |
| 500 | `INTERNAL_ERROR` | Unexpected server error |
| 503 | `SERVICE_UNAVAILABLE` | Database/cache temporarily unavailable |

---

## Rust Library API (In-Process Integration)

### Trait Definitions

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Core Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRequest {
    pub idempotency_key: String,
    pub agent_nhi: String,
    pub delegation_chain: Vec<String>,
    pub event_type: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub properties: serde_json::Value,
    pub signature: Vec<u8>,
    pub signature_algorithm: String,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub event_id: Uuid,
    pub idempotency_key: String,
    pub agent_nhi: String,
    pub delegation_chain: Vec<String>,
    pub subscription_id: Uuid,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub properties: serde_json::Value,
    pub signature: Vec<u8>,
    pub audit_log_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub enum EventResponse {
    Created(Uuid),    // 201
    Accepted(Uuid),   // 202 (idempotent)
}

#[derive(Debug, Clone)]
pub enum QuotaDecision {
    Allow {
        remaining: u64,
        next_reset: Option<DateTime<Utc>>,
    },
    Deny {
        reason: QuotaExceededReason,
        retry_after: Option<chrono::Duration>,
        current_usage: u64,
        limit: u64,
    },
}

#[derive(Debug, Clone)]
pub enum QuotaExceededReason {
    LimitReached,
    SubscriptionSuspended,
    QuotaNotConfigured,
}

// ============================================================================
// Service Traits
// ============================================================================

#[async_trait]
pub trait EventIngestionService: Send + Sync {
    async fn ingest_event(&self, req: EventRequest) -> Result<EventResponse, Error>;
    async fn ingest_batch(&self, reqs: Vec<EventRequest>) -> Result<BatchResponse, Error>;
    async fn get_event(&self, event_id: Uuid) -> Result<Event, Error>;
}

#[async_trait]
pub trait QuotaEnforcer: Send + Sync {
    async fn check_quota(&self, agent: &str, event_type: &str) -> Result<QuotaDecision, Error>;
    async fn reserve_quota(&self, agent: &str, event_type: &str, quantity: u64) -> Result<Uuid, Error>;
    async fn commit_reservation(&self, reservation_id: Uuid) -> Result<(), Error>;
    async fn rollback_reservation(&self, reservation_id: Uuid) -> Result<(), Error>;
}

#[async_trait]
pub trait AggregationEngine: Send + Sync {
    async fn get_usage(
        &self,
        subscription_id: Uuid,
        event_type: &str,
        period: TimePeriod,
    ) -> Result<AggregatedUsage, Error>;

    async fn get_attribution(
        &self,
        subscription_id: Uuid,
        period: TimePeriod,
    ) -> Result<AttributionReport, Error>;
}

#[async_trait]
pub trait BillingService: Send + Sync {
    async fn generate_invoice(
        &self,
        subscription_id: Uuid,
        period: BillingPeriod,
    ) -> Result<Invoice, Error>;

    async fn finalize_invoice(&self, invoice_id: Uuid) -> Result<(), Error>;
    async fn get_invoice(&self, invoice_id: Uuid) -> Result<Invoice, Error>;
}

// ============================================================================
// Supporting Types
// ============================================================================

#[derive(Debug, Clone)]
pub struct TimePeriod {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct AggregatedUsage {
    pub event_type: String,
    pub count: u64,
    pub sum_value: Decimal,
    pub unique_count: usize,
    pub max_value: Decimal,
    pub by_dimension: HashMap<String, DimensionUsage>,
}

#[derive(Debug, Clone)]
pub struct AttributionReport {
    pub by_agent: HashMap<String, Decimal>,
    pub by_dimension: HashMap<String, HashMap<String, Decimal>>,
}

#[derive(Debug, Clone)]
pub struct Invoice {
    pub invoice_id: Uuid,
    pub subscription_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub line_items: Vec<LineItem>,
    pub subtotal: Decimal,
    pub tax: Decimal,
    pub total: Decimal,
    pub currency: String,
    pub attribution: AttributionReport,
    pub status: InvoiceStatus,
    pub stripe_invoice_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct LineItem {
    pub description: String,
    pub metric_code: String,
    pub quantity: Decimal,
    pub unit_price: Option<Decimal>,
    pub amount: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InvoiceStatus {
    Draft,
    Issued,
    Paid,
    Void,
    Disputed,
}

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Idempotency conflict: {existing_hash}")]
    IdempotencyConflict { existing_hash: String },

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Timestamp skew: {0}")]
    TimestampSkew(String),

    #[error("Quota exceeded: {reason:?}")]
    QuotaExceeded { reason: QuotaExceededReason },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Internal error: {0}")]
    Internal(String),
}
```

---

## Usage Examples

### Example 1: Agent Submitting Event (Rust)

```rust
use creto_metering::{EventIngestionService, EventRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize metering client
    let metering = creto_metering::Client::new("https://api.creto.io/metering/v1")?;

    // Agent performs work (e.g., LLM call)
    let llm_response = call_gpt4("What is the capital of France?").await?;

    // Submit billable event
    let event = EventRequest {
        idempotency_key: format!("llm-call-{}", Uuid::new_v4()),
        agent_nhi: "agent:nhi:ed25519:my-agent".to_string(),
        delegation_chain: vec!["human:alice@example.com".to_string()],
        event_type: "llm_tokens".to_string(),
        timestamp: None, // Server assigns
        properties: serde_json::json!({
            "tokens": llm_response.usage.total_tokens,
            "model": "gpt-4",
            "prompt_tokens": llm_response.usage.prompt_tokens,
            "completion_tokens": llm_response.usage.completion_tokens,
        }),
        signature: sign_event_with_nhi_key(&event_data).await?,
        signature_algorithm: "ML-DSA-65".to_string(),
    };

    match metering.ingest_event(event).await? {
        EventResponse::Created(event_id) => {
            println!("Event created: {}", event_id);
        }
        EventResponse::Accepted(event_id) => {
            println!("Event already exists (idempotent): {}", event_id);
        }
    }

    Ok(())
}
```

---

### Example 2: Authorization Service Checking Quota (Rust)

```rust
use creto_metering::{QuotaEnforcer, QuotaDecision};

async fn check_authorization_with_quota(
    authz: &AuthorizationService,
    metering: &dyn QuotaEnforcer,
    agent: &str,
    action: &str,
    resource: &str,
) -> Result<bool, Error> {
    // Step 1: Check authorization policy
    let policy_result = authz.check(agent, action, resource).await?;

    if !policy_result.allowed {
        return Ok(false); // Denied by policy
    }

    // Step 2: Check quota (inline)
    match metering.check_quota(agent, action).await? {
        QuotaDecision::Allow { remaining, .. } => {
            println!("Quota OK, {} remaining", remaining);
            Ok(true)
        }
        QuotaDecision::Deny { reason, retry_after, .. } => {
            println!("Quota exceeded: {:?}, retry after {:?}", reason, retry_after);
            Ok(false) // Deny due to quota
        }
    }
}
```

---

### Example 3: Generating Invoice (Rust)

```rust
use creto_metering::BillingService;
use chrono::{Utc, TimeZone};

async fn generate_monthly_invoice(
    billing: &dyn BillingService,
    subscription_id: Uuid,
) -> Result<(), Error> {
    // Calculate billing period (last month)
    let now = Utc::now();
    let period_start = Utc.ymd(now.year(), now.month() - 1, 1).and_hms(0, 0, 0);
    let period_end = Utc.ymd(now.year(), now.month(), 1).and_hms(0, 0, 0);

    // Generate invoice
    let invoice = billing.generate_invoice(
        subscription_id,
        BillingPeriod { start: period_start, end: period_end },
    ).await?;

    println!("Invoice generated: {}", invoice.invoice_id);
    println!("Total: {} {}", invoice.total, invoice.currency);

    // Finalize and send to customer
    billing.finalize_invoice(invoice.invoice_id).await?;

    Ok(())
}
```

---

## Rate Limiting

### Per-Subscription Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| `POST /v1/events` | 1,000 req/min | Rolling 1 minute |
| `POST /v1/events/batch` | 100 req/min | Rolling 1 minute |
| `GET /v1/usage/*` | 100 req/min | Rolling 1 minute |
| `GET /v1/quotas/*` | 1,000 req/min | Rolling 1 minute |
| `POST /v1/invoices` | 10 req/hour | Rolling 1 hour |

**Rate Limit Headers**:
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 450
X-RateLimit-Reset: 1735142460
```

**429 Response**:
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Too many requests",
    "metadata": {
      "limit": 1000,
      "window": "1 minute",
      "retry_after": 30
    }
  }
}
```

---

## Webhook Events (Future Phase)

### Event Types

| Event | Trigger | Payload |
|-------|---------|---------|
| `event.created` | New event ingested | Event object |
| `quota.exceeded` | Quota limit reached | Quota + usage details |
| `invoice.generated` | Invoice created | Invoice object |
| `invoice.paid` | Stripe payment confirmed | Invoice + payment details |

**Webhook Payload Example**:
```json
{
  "event_type": "quota.exceeded",
  "timestamp": "2024-12-25T10:30:00Z",
  "data": {
    "subscription_id": "sub_abc123",
    "agent_nhi": "agent:worker-1",
    "event_type": "api_calls",
    "quota": {
      "limit": 1000,
      "current_usage": 1001,
      "period": "hourly"
    }
  }
}
```

---

**Next Document**: SDD-MTR-05: Security Design (threat model, attack mitigations)
