---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
parent_sdd: docs/sdd/products/oversight/01-requirements.md
---

# SDD-OVS-04: Oversight API Design

## 1. API Overview

### 1.1 API Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     API LAYER                               │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────┐  ┌──────────────────┐  ┌────────────┐ │
│  │  gRPC Server    │  │  REST Gateway    │  │  Webhooks  │ │
│  │  (internal)     │  │  (external)      │  │  (inbound) │ │
│  │  Port: 50051    │  │  Port: 8080      │  │            │ │
│  └────────┬────────┘  └────────┬─────────┘  └──────┬─────┘ │
│           │                    │                    │       │
│           └────────────────────┴────────────────────┘       │
│                              │                              │
└──────────────────────────────┼──────────────────────────────┘
                               │
                               v
                    ┌──────────────────┐
                    │ OversightService │
                    │ (Business Logic) │
                    └──────────────────┘
```

### 1.2 API Protocols

| Protocol | Purpose | Clients | Authentication |
|----------|---------|---------|----------------|
| **gRPC** | Internal service-to-service (Authorization, Memory, Audit) | Creto services | mTLS + JWT |
| **REST** | External API for approvers and integrations | Web UI, mobile apps, external systems | API keys + OAuth 2.0 |
| **Webhook** | Inbound callbacks from channels (Slack, ServiceNow) | Slack, email providers, ServiceNow | HMAC signature verification |

---

## 2. gRPC API Design

### 2.1 Protocol Buffer Definitions

```protobuf
syntax = "proto3";

package creto.oversight.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/duration.proto";

// Oversight service definition
service OversightService {
    // Create oversight request (called by Authorization service)
    rpc CreateRequest(CreateRequestRequest) returns (CreateRequestResponse);

    // Submit approval/denial response
    rpc SubmitResponse(SubmitResponseRequest) returns (SubmitResponseResponse);

    // Query request status
    rpc GetRequest(GetRequestRequest) returns (GetRequestResponse);

    // List requests (for dashboards)
    rpc ListRequests(ListRequestsRequest) returns (ListRequestsResponse);

    // Cancel pending request
    rpc CancelRequest(CancelRequestRequest) returns (CancelRequestResponse);

    // Stream request state changes (for real-time updates)
    rpc WatchRequest(WatchRequestRequest) returns (stream RequestStateChange);
}

// ========== CreateRequest ==========

message CreateRequestRequest {
    // Agent identity
    string agent_nhi = 1;
    repeated string delegation_chain = 2;

    // Action context
    string action = 3;  // JSON-serialized Action
    string resource = 4;  // JSON-serialized Resource
    string policy_id = 5;

    // Request context
    string action_description = 6;
    string reasoning = 7;
    repeated RiskFactor risk_factors = 8;
    ImpactAssessment impact = 9;

    // Oversight requirement
    OversightRequirement requirement = 10;
}

message OversightRequirement {
    repeated string approvers = 1;
    EscalationChain escalation_chain = 2;
    QuorumConfig quorum = 3;
}

message EscalationChain {
    repeated EscalationTier tiers = 1;
    TimeoutAction final_action = 2;
}

message EscalationTier {
    repeated string approvers = 1;
    google.protobuf.Duration timeout = 2;
    repeated string channels = 3;  // "SLACK", "EMAIL", "WEBHOOK"
    QuorumConfig quorum = 4;
}

enum TimeoutAction {
    AUTO_DENY = 0;
    AUTO_APPROVE = 1;
    BLOCK_INDEFINITELY = 2;
}

message QuorumConfig {
    QuorumType type = 1;
    int32 threshold = 2;  // For THRESHOLD type
}

enum QuorumType {
    ANY = 0;
    ALL = 1;
    THRESHOLD = 2;
}

message RiskFactor {
    string category = 1;  // "Financial", "Compliance", "Data", "Security"
    string severity = 2;  // "Low", "Medium", "High", "Critical"
    string description = 3;
}

message ImpactAssessment {
    repeated string affected_resources = 1;
    bool reversible = 2;
    double estimated_cost = 3;
}

message CreateRequestResponse {
    string request_id = 1;
    RequestState state = 2;
    google.protobuf.Timestamp created_at = 3;
}

enum RequestState {
    PENDING = 0;
    APPROVED = 1;
    DENIED = 2;
    ESCALATED = 3;
    TIMEOUT = 4;
    CANCELLED = 5;
}

// ========== SubmitResponse ==========

message SubmitResponseRequest {
    string request_id = 1;

    // Approver identity
    ApproverIdentity approver = 2;

    // Decision
    ApprovalDecision decision = 3;
    string reason = 4;  // Optional justification
    string question = 5;  // For REQUEST_MORE_INFO

    // Cryptographic signature
    Signature signature = 6;

    // Channel metadata
    string channel_type = 7;  // "SLACK", "EMAIL", "WEBHOOK"
    map<string, string> channel_metadata = 8;
}

message ApproverIdentity {
    string subject = 1;  // Email or NHI identifier
    string name = 2;
    bytes public_key = 3;  // ML-DSA or Ed25519 public key
}

enum ApprovalDecision {
    APPROVE = 0;
    DENY = 1;
    REQUEST_MORE_INFO = 2;
}

message Signature {
    string algorithm = 1;  // "ML-DSA-65", "Ed25519"
    bytes value = 2;  // Signature bytes
}

message SubmitResponseResponse {
    RequestState new_state = 1;
    bool quorum_met = 2;
    string override_token = 3;  // For Authorization service (if approved)
}

// ========== GetRequest ==========

message GetRequestRequest {
    string request_id = 1;
}

message GetRequestResponse {
    string request_id = 1;
    string agent_nhi = 2;
    repeated string delegation_chain = 3;
    string action_description = 4;
    string reasoning = 5;
    repeated RiskFactor risk_factors = 6;
    RequestState state = 7;
    int32 tier_index = 8;
    repeated ApprovalResponse responses = 9;
    google.protobuf.Timestamp created_at = 10;
    google.protobuf.Timestamp updated_at = 11;
}

message ApprovalResponse {
    ApproverIdentity approver = 1;
    ApprovalDecision decision = 2;
    string reason = 3;
    Signature signature = 4;
    google.protobuf.Timestamp responded_at = 5;
}

// ========== ListRequests ==========

message ListRequestsRequest {
    // Filters
    repeated RequestState states = 1;
    string agent_nhi = 2;
    string approver_subject = 3;

    // Pagination
    int32 page_size = 4;
    string page_token = 5;
}

message ListRequestsResponse {
    repeated GetRequestResponse requests = 1;
    string next_page_token = 2;
    int32 total_count = 3;
}

// ========== CancelRequest ==========

message CancelRequestRequest {
    string request_id = 1;
    string reason = 2;
}

message CancelRequestResponse {
    RequestState new_state = 1;
}

// ========== WatchRequest ==========

message WatchRequestRequest {
    string request_id = 1;
}

message RequestStateChange {
    string request_id = 1;
    RequestState old_state = 2;
    RequestState new_state = 3;
    google.protobuf.Timestamp timestamp = 4;
}
```

### 2.2 gRPC Service Implementation

```rust
use tonic::{Request, Response, Status};
use oversight_proto::oversight_service_server::OversightService;

pub struct OversightServiceImpl {
    request_manager: Arc<RequestManager>,
    policy_engine: Arc<PolicyEngine>,
    response_handler: Arc<ResponseHandler>,
}

#[tonic::async_trait]
impl OversightService for OversightServiceImpl {
    async fn create_request(
        &self,
        request: Request<CreateRequestRequest>,
    ) -> Result<Response<CreateRequestResponse>, Status> {
        let req = request.into_inner();

        // Verify caller is authorized (Authorization service)
        self.verify_caller_authorized(&request.metadata()).await
            .map_err(|e| Status::unauthenticated(e.to_string()))?;

        // Create oversight request
        let request_id = self.request_manager.create_request(OversightRequest {
            agent_nhi: AgentNhi::from_str(&req.agent_nhi)
                .map_err(|e| Status::invalid_argument(e.to_string()))?,
            delegation_chain: req.delegation_chain.iter()
                .map(|nhi| AgentNhi::from_str(nhi))
                .collect::<Result<Vec<_>, _>>()
                .map_err(|e| Status::invalid_argument(e.to_string()))?,
            action: serde_json::from_str(&req.action)
                .map_err(|e| Status::invalid_argument(e.to_string()))?,
            resource: serde_json::from_str(&req.resource)
                .map_err(|e| Status::invalid_argument(e.to_string()))?,
            policy_id: req.policy_id,
            action_description: req.action_description,
            reasoning: req.reasoning,
            risk_factors: req.risk_factors.into_iter().map(|rf| RiskFactor {
                category: rf.category,
                severity: rf.severity,
                description: rf.description,
            }).collect(),
            requirement: req.requirement.ok_or_else(|| Status::invalid_argument("requirement required"))?.into(),
        }).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateRequestResponse {
            request_id: request_id.to_string(),
            state: RequestState::Pending as i32,
            created_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
        }))
    }

    async fn submit_response(
        &self,
        request: Request<SubmitResponseRequest>,
    ) -> Result<Response<SubmitResponseResponse>, Status> {
        let req = request.into_inner();

        // Parse request ID
        let request_id = RequestId::from_str(&req.request_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        // Build approval response
        let response = ApprovalResponse {
            approver: ApproverIdentity {
                subject: req.approver.as_ref().unwrap().subject.clone(),
                name: req.approver.as_ref().unwrap().name.clone(),
                public_key: req.approver.as_ref().unwrap().public_key.clone(),
            },
            decision: match req.decision {
                0 => ApprovalDecision::Approve,
                1 => ApprovalDecision::Deny,
                2 => ApprovalDecision::RequestMoreInfo { question: req.question },
                _ => return Err(Status::invalid_argument("invalid decision")),
            },
            reason: req.reason,
            signature: Signature {
                algorithm: req.signature.as_ref().unwrap().algorithm.clone(),
                value: req.signature.as_ref().unwrap().value.clone(),
            },
            channel_type: req.channel_type.parse()
                .map_err(|e| Status::invalid_argument(format!("invalid channel_type: {}", e)))?,
            channel_metadata: req.channel_metadata,
            responded_at: Timestamp::now(),
        };

        // Submit response
        let outcome = self.response_handler.submit_response(request_id, response).await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SubmitResponseResponse {
            new_state: outcome.new_state as i32,
            quorum_met: outcome.quorum_met,
            override_token: outcome.override_token,
        }))
    }

    async fn get_request(
        &self,
        request: Request<GetRequestRequest>,
    ) -> Result<Response<GetRequestResponse>, Status> {
        let req = request.into_inner();

        let request_id = RequestId::from_str(&req.request_id)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let state = self.request_manager.get_request(request_id).await
            .map_err(|e| match e {
                Error::RequestNotFound => Status::not_found("request not found"),
                _ => Status::internal(e.to_string()),
            })?;

        Ok(Response::new(GetRequestResponse {
            request_id: state.request_id.to_string(),
            agent_nhi: state.agent_nhi.to_string(),
            delegation_chain: state.delegation_chain.iter().map(|nhi| nhi.to_string()).collect(),
            action_description: state.action_description,
            reasoning: state.reasoning,
            risk_factors: state.risk_factors.into_iter().map(|rf| RiskFactor {
                category: rf.category,
                severity: rf.severity,
                description: rf.description,
            }).collect(),
            state: state.state as i32,
            tier_index: state.tier_index as i32,
            responses: state.responses.into_iter().map(|r| /* convert */).collect(),
            created_at: Some(state.created_at.into()),
            updated_at: Some(state.updated_at.into()),
        }))
    }
}
```

### 2.3 gRPC Error Handling

**Error Codes:**
- `UNAUTHENTICATED` (16): Caller not authorized
- `INVALID_ARGUMENT` (3): Invalid request parameters
- `NOT_FOUND` (5): Request ID not found
- `ALREADY_EXISTS` (6): Duplicate response from approver
- `FAILED_PRECONDITION` (9): Request in terminal state, cannot accept responses
- `INTERNAL` (13): Internal server error

**Error Details:**
```protobuf
message ErrorDetails {
    string error_code = 1;  // "SIGNATURE_VERIFICATION_FAILED", "QUORUM_ALREADY_MET"
    string message = 2;
    map<string, string> metadata = 3;
}
```

---

## 3. REST API Design

### 3.1 REST Endpoint Definitions

**Base URL:** `https://oversight.company.com/api/v1`

| Method | Endpoint | Purpose | Auth |
|--------|----------|---------|------|
| POST | `/requests` | Create oversight request | Service account |
| GET | `/requests/{id}` | Get request details | Approver or agent |
| POST | `/requests/{id}/approve` | Submit approval | Approver |
| POST | `/requests/{id}/deny` | Submit denial | Approver |
| POST | `/requests/{id}/cancel` | Cancel request | Agent |
| GET | `/requests` | List requests (dashboard) | Approver |
| POST | `/webhook/slack` | Slack webhook callback | HMAC signature |
| POST | `/webhook/email` | Email approval link callback | One-time token |
| POST | `/webhook/response` | Generic webhook callback | HMAC signature |

### 3.2 OpenAPI Specification

```yaml
openapi: 3.0.0
info:
  title: Creto Oversight API
  version: 1.0.0
  description: Human-in-the-Loop approval orchestration API

servers:
  - url: https://oversight.company.com/api/v1
    description: Production server

security:
  - ApiKeyAuth: []
  - OAuth2: [oversight:read, oversight:write]

paths:
  /requests:
    post:
      summary: Create oversight request
      operationId: createRequest
      tags: [Requests]
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateRequestRequest'
      responses:
        '201':
          description: Request created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/CreateRequestResponse'
        '400':
          $ref: '#/components/responses/BadRequest'
        '401':
          $ref: '#/components/responses/Unauthorized'
        '500':
          $ref: '#/components/responses/InternalError'

    get:
      summary: List requests
      operationId: listRequests
      tags: [Requests]
      parameters:
        - name: state
          in: query
          schema:
            type: array
            items:
              type: string
              enum: [PENDING, APPROVED, DENIED, ESCALATED, TIMEOUT, CANCELLED]
        - name: agent_nhi
          in: query
          schema:
            type: string
        - name: approver
          in: query
          schema:
            type: string
        - name: page_size
          in: query
          schema:
            type: integer
            default: 20
        - name: page_token
          in: query
          schema:
            type: string
      responses:
        '200':
          description: List of requests
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/ListRequestsResponse'

  /requests/{request_id}:
    get:
      summary: Get request details
      operationId: getRequest
      tags: [Requests]
      parameters:
        - name: request_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Request details
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OversightRequest'
        '404':
          $ref: '#/components/responses/NotFound'

  /requests/{request_id}/approve:
    post:
      summary: Approve request
      operationId: approveRequest
      tags: [Responses]
      parameters:
        - name: request_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [approver, signature]
              properties:
                approver:
                  $ref: '#/components/schemas/ApproverIdentity'
                reason:
                  type: string
                  description: Optional justification for approval
                signature:
                  $ref: '#/components/schemas/Signature'
      responses:
        '200':
          description: Approval recorded
          content:
            application/json:
              schema:
                type: object
                properties:
                  new_state:
                    type: string
                    enum: [PENDING, APPROVED]
                  quorum_met:
                    type: boolean
                  override_token:
                    type: string
                    description: Authorization override token (if approved)
        '400':
          $ref: '#/components/responses/BadRequest'
        '409':
          description: Duplicate response or request already resolved

  /requests/{request_id}/deny:
    post:
      summary: Deny request
      operationId: denyRequest
      tags: [Responses]
      parameters:
        - name: request_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              required: [approver, reason, signature]
              properties:
                approver:
                  $ref: '#/components/schemas/ApproverIdentity'
                reason:
                  type: string
                  description: Reason for denial
                signature:
                  $ref: '#/components/schemas/Signature'
      responses:
        '200':
          description: Denial recorded
          content:
            application/json:
              schema:
                type: object
                properties:
                  new_state:
                    type: string
                    enum: [DENIED]

  /requests/{request_id}/cancel:
    post:
      summary: Cancel request
      operationId: cancelRequest
      tags: [Requests]
      parameters:
        - name: request_id
          in: path
          required: true
          schema:
            type: string
            format: uuid
      requestBody:
        content:
          application/json:
            schema:
              type: object
              properties:
                reason:
                  type: string
      responses:
        '200':
          description: Request cancelled

  /webhook/slack:
    post:
      summary: Slack interactive message webhook
      operationId: slackWebhook
      tags: [Webhooks]
      security: []  # Verified via Slack signing secret
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                type:
                  type: string
                  enum: [block_actions]
                user:
                  type: object
                  properties:
                    id:
                      type: string
                    email:
                      type: string
                actions:
                  type: array
                  items:
                    type: object
                    properties:
                      action_id:
                        type: string
                        enum: [approve, deny, more_info]
                      value:
                        type: string  # request_id
      responses:
        '200':
          description: Webhook processed

  /webhook/email:
    get:
      summary: Email approval link callback
      operationId: emailApprovalLink
      tags: [Webhooks]
      security: []  # Verified via one-time token
      parameters:
        - name: token
          in: query
          required: true
          schema:
            type: string
      responses:
        '200':
          description: HTML page confirming approval
          content:
            text/html:
              schema:
                type: string

components:
  schemas:
    CreateRequestRequest:
      type: object
      required: [agent_nhi, delegation_chain, action, resource, policy_id, requirement]
      properties:
        agent_nhi:
          type: string
        delegation_chain:
          type: array
          items:
            type: string
        action:
          type: string
          description: JSON-serialized Action
        resource:
          type: object
          description: Resource metadata
        policy_id:
          type: string
        action_description:
          type: string
        reasoning:
          type: string
        risk_factors:
          type: array
          items:
            $ref: '#/components/schemas/RiskFactor'
        requirement:
          $ref: '#/components/schemas/OversightRequirement'

    OversightRequirement:
      type: object
      properties:
        escalation_chain:
          $ref: '#/components/schemas/EscalationChain'

    EscalationChain:
      type: object
      properties:
        tiers:
          type: array
          items:
            $ref: '#/components/schemas/EscalationTier'
        final_action:
          type: string
          enum: [AUTO_DENY, AUTO_APPROVE, BLOCK_INDEFINITELY]

    EscalationTier:
      type: object
      properties:
        approvers:
          type: array
          items:
            type: string
        timeout:
          type: string
          format: duration
        channels:
          type: array
          items:
            type: string
            enum: [SLACK, EMAIL, WEBHOOK]
        quorum:
          $ref: '#/components/schemas/QuorumConfig'

    QuorumConfig:
      type: object
      properties:
        type:
          type: string
          enum: [ANY, ALL, THRESHOLD]
        threshold:
          type: integer

    RiskFactor:
      type: object
      properties:
        category:
          type: string
        severity:
          type: string
          enum: [Low, Medium, High, Critical]
        description:
          type: string

    ApproverIdentity:
      type: object
      required: [subject, public_key]
      properties:
        subject:
          type: string
        name:
          type: string
        public_key:
          type: string
          format: byte

    Signature:
      type: object
      required: [algorithm, value]
      properties:
        algorithm:
          type: string
          enum: [ML-DSA-65, Ed25519]
        value:
          type: string
          format: byte

    OversightRequest:
      type: object
      properties:
        request_id:
          type: string
          format: uuid
        agent_nhi:
          type: string
        delegation_chain:
          type: array
          items:
            type: string
        action_description:
          type: string
        reasoning:
          type: string
        risk_factors:
          type: array
          items:
            $ref: '#/components/schemas/RiskFactor'
        state:
          type: string
          enum: [PENDING, APPROVED, DENIED, ESCALATED, TIMEOUT, CANCELLED]
        tier_index:
          type: integer
        responses:
          type: array
          items:
            $ref: '#/components/schemas/ApprovalResponse'
        created_at:
          type: string
          format: date-time
        updated_at:
          type: string
          format: date-time

    ApprovalResponse:
      type: object
      properties:
        approver:
          $ref: '#/components/schemas/ApproverIdentity'
        decision:
          type: string
          enum: [APPROVE, DENY, REQUEST_MORE_INFO]
        reason:
          type: string
        signature:
          $ref: '#/components/schemas/Signature'
        responded_at:
          type: string
          format: date-time

    ListRequestsResponse:
      type: object
      properties:
        requests:
          type: array
          items:
            $ref: '#/components/schemas/OversightRequest'
        next_page_token:
          type: string
        total_count:
          type: integer

  responses:
    BadRequest:
      description: Invalid request parameters
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string
              details:
                type: object

    Unauthorized:
      description: Authentication required or failed
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string

    NotFound:
      description: Resource not found
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string

    InternalError:
      description: Internal server error
      content:
        application/json:
          schema:
            type: object
            properties:
              error:
                type: string
              request_id:
                type: string

  securitySchemes:
    ApiKeyAuth:
      type: apiKey
      in: header
      name: X-API-Key
    OAuth2:
      type: oauth2
      flows:
        authorizationCode:
          authorizationUrl: https://auth.company.com/oauth/authorize
          tokenUrl: https://auth.company.com/oauth/token
          scopes:
            oversight:read: Read oversight requests
            oversight:write: Create and respond to oversight requests
```

---

## 4. Webhook Integration Specifications

### 4.1 Slack Webhook

**Inbound: Interactive Message Response**

Slack POSTs to `/api/v1/webhook/slack` when approver clicks button:

```json
{
  "type": "block_actions",
  "user": {
    "id": "U123456",
    "username": "alice",
    "email": "alice@company.com"
  },
  "actions": [
    {
      "action_id": "approve",
      "block_id": "approval_buttons",
      "value": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
    }
  ],
  "message": {
    "ts": "1703520000.123456"
  },
  "channel": {
    "id": "C123456"
  }
}
```

**Verification:**
```rust
// Verify Slack signing secret
let timestamp = headers.get("X-Slack-Request-Timestamp").unwrap();
let signature = headers.get("X-Slack-Signature").unwrap();

let signing_base = format!("v0:{}:{}", timestamp, request_body);
let expected_signature = format!("v0={}", hmac_sha256(SLACK_SIGNING_SECRET, &signing_base));

if signature != expected_signature {
    return Err(Error::InvalidSignature);
}
```

### 4.2 Email Webhook

**One-Time Approval Link:**

```
https://oversight.company.com/api/v1/webhook/email?token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Token Payload:**
```json
{
  "request_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "approver_subject": "cfo@company.com",
  "decision": "APPROVE",
  "exp": 1703606400,  // Expiration (matches tier timeout)
  "nonce": "random-nonce"  // One-time use
}
```

**Token Verification:**
```rust
let claims: ApprovalTokenClaims = jsonwebtoken::decode(
    &token,
    &DecodingKey::from_secret(EMAIL_TOKEN_SECRET),
    &Validation::default(),
)?;

// Check expiration
if claims.exp < Timestamp::now().as_secs() {
    return Err(Error::TokenExpired);
}

// Check one-time use (nonce tracking in Redis)
if self.redis.exists(format!("nonce:{}", claims.nonce)).await? {
    return Err(Error::TokenAlreadyUsed);
}
self.redis.set_ex(format!("nonce:{}", claims.nonce), "1", claims.exp - Timestamp::now().as_secs()).await?;
```

### 4.3 ServiceNow Webhook

**Outbound: Create Approval Ticket**

POST to `https://company.service-now.com/api/oversight/requests`:

```json
{
  "request_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "agent_nhi": "agent:payment-bot-v3@company.creto",
  "delegation_chain": [
    "agent:payment-bot-v3@company.creto",
    "agent:accounts-payable@company.creto",
    "human:alice@company.com"
  ],
  "action": "Transfer $50,000 to vendor invoice #INV-2024-1234",
  "reasoning": "Invoice approved in AP system. Due date: 2024-12-31.",
  "risk_factors": [
    {
      "category": "Financial",
      "severity": "High",
      "description": "Large transaction exceeding $10K threshold"
    }
  ],
  "approver": "cfo@company.com",
  "callback_url": "https://oversight.company.com/api/v1/webhook/response",
  "expires_at": "2024-12-25T16:00:00Z"
}
```

**HMAC Signature:**
```
X-Oversight-Signature: sha256=abc123...
```

**Inbound: Approval Response**

ServiceNow POSTs to `/api/v1/webhook/response`:

```json
{
  "request_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "ticket_id": "CHG0012345",
  "decision": "APPROVE",
  "approver": {
    "subject": "cfo@company.com",
    "name": "Alice CFO",
    "public_key": "..."
  },
  "reason": "Invoice verified, payment authorized per contract",
  "signature": {
    "algorithm": "ML-DSA-65",
    "value": "..."
  },
  "timestamp": "2024-12-25T14:30:00Z"
}
```

---

## 5. Authentication and Authorization

### 5.1 API Key Authentication

**For Service-to-Service:**
```
Authorization: Bearer sk_live_abc123...
```

**Validation:**
```rust
let api_key = headers.get("Authorization")
    .and_then(|h| h.to_str().ok())
    .and_then(|h| h.strip_prefix("Bearer "))
    .ok_or(Error::MissingApiKey)?;

let client_identity = self.api_key_store.validate(api_key).await?;

// Check client has permission to create requests
if !client_identity.scopes.contains("oversight:write") {
    return Err(Error::InsufficientPermissions);
}
```

### 5.2 OAuth 2.0 (For External Integrations)

**Authorization Flow:**
1. Client redirects to: `https://auth.company.com/oauth/authorize?client_id=...&scope=oversight:read+oversight:write`
2. User logs in and grants consent
3. Redirect to: `https://client.com/callback?code=abc123`
4. Client exchanges code for access token: `POST /oauth/token`
5. Client uses access token: `Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`

**Token Validation:**
```rust
let token = headers.get("Authorization")
    .and_then(|h| h.to_str().ok())
    .and_then(|h| h.strip_prefix("Bearer "))
    .ok_or(Error::MissingToken)?;

let claims: TokenClaims = jsonwebtoken::decode(
    token,
    &DecodingKey::from_rsa_pem(OAUTH_PUBLIC_KEY)?,
    &Validation::default(),
)?;

// Check scopes
if !claims.scope.split(' ').any(|s| s == "oversight:write") {
    return Err(Error::InsufficientScopes);
}
```

### 5.3 mTLS (For gRPC)

**Mutual TLS Configuration:**
```rust
let tls_config = ServerTlsConfig::new()
    .identity(Identity::from_pem(SERVER_CERT, SERVER_KEY))
    .client_ca_root(Certificate::from_pem(CLIENT_CA));

Server::builder()
    .tls_config(tls_config)?
    .add_service(OversightServiceServer::new(service))
    .serve(addr)
    .await?;
```

**Client Certificate Verification:**
```rust
// Extract client identity from mTLS certificate
let client_cert = request.peer_certs()
    .and_then(|certs| certs.first())
    .ok_or(Status::unauthenticated("client certificate required"))?;

let client_identity = self.nhi_registry.verify_certificate(client_cert).await
    .map_err(|e| Status::unauthenticated(e.to_string()))?;

// Check client is authorized service (Authorization, Memory, Audit)
if !AUTHORIZED_SERVICES.contains(&client_identity.service_name) {
    return Err(Status::permission_denied("service not authorized"));
}
```

---

## 6. Rate Limiting

### 6.1 Rate Limit Configuration

| Client Type | Limit | Window | Burst |
|------------|-------|--------|-------|
| Service account (Authorization) | 10,000 req/s | 1 second | 20,000 |
| Service account (other) | 1,000 req/s | 1 second | 2,000 |
| OAuth client | 100 req/s | 1 second | 200 |
| Webhook callback | 1,000 req/s | 1 second | 2,000 |

### 6.2 Rate Limit Implementation

**Token Bucket Algorithm (Redis):**
```rust
pub async fn check_rate_limit(
    &self,
    client_id: &str,
    limit: u64,
    window: Duration,
) -> Result<bool> {
    let key = format!("ratelimit:{}:{}", client_id, window.as_secs());
    let current_count: u64 = self.redis.incr(&key, 1).await?;

    if current_count == 1 {
        self.redis.expire(&key, window.as_secs() as usize).await?;
    }

    if current_count > limit {
        return Ok(false);  // Rate limit exceeded
    }

    Ok(true)
}
```

**Headers:**
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 987
X-RateLimit-Reset: 1703520060
```

---

## 7. API Versioning

### 7.1 Versioning Strategy

**URL-Based Versioning:**
- `/api/v1/requests` (current)
- `/api/v2/requests` (future)

**Deprecation Policy:**
- Support 2 major versions concurrently
- 6-month deprecation notice
- Sunset header: `Sunset: Sat, 01 Jun 2025 00:00:00 GMT`

### 7.2 Breaking vs Non-Breaking Changes

**Non-Breaking (Allowed):**
- Add new optional fields
- Add new endpoints
- Add new enum values (with fallback)

**Breaking (Requires New Version):**
- Remove or rename fields
- Change field types
- Remove endpoints
- Change authentication mechanism

---

**END OF DOCUMENT**
