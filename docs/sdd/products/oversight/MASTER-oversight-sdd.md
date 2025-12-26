---
status: approved
author: Architecture Team
created: 2025-12-25
updated: 2025-12-25
reviewers: [Security Team, Compliance Team, Platform Team]
oss_reference: humanlayer/humanlayer, LangGraph interrupt()
version: 1.0.0
---

# MASTER SDD: Creto Oversight (Human-in-the-Loop)

> **This is the single authoritative reference for creto-oversight.**
> All implementation MUST conform to this specification.

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [System Architecture](#2-system-architecture)
3. [Core Components](#3-core-components)
4. [State Machine Specification](#4-state-machine-specification)
5. [API Contracts](#5-api-contracts)
6. [SDK Patterns](#6-sdk-patterns)
7. [Integration Contracts](#7-integration-contracts)
8. [Data Model](#8-data-model)
9. [Error Taxonomy](#9-error-taxonomy)
10. [Edge Cases & Failure Modes](#10-edge-cases--failure-modes)
11. [Sequence Diagrams](#11-sequence-diagrams)
12. [Security Model](#12-security-model)
13. [Performance Specifications](#13-performance-specifications)
14. [Operational Procedures](#14-operational-procedures)

---

## 1. Executive Summary

### 1.1 Purpose

**creto-oversight** provides Human-in-the-Loop (HITL) approval workflows for AI agent actions. It enables organizations to require human approval before agents execute high-risk operations, with full context about the agent's identity, delegation chain, and reasoning.

### 1.2 Differentiators vs. OSS (HumanLayer)

| Dimension | HumanLayer | Creto Oversight |
|-----------|------------|-----------------|
| **Trigger** | `@require_approval()` decorator | Policy-driven via Authorization (168ns) |
| **Identity** | Tool name + args | Agent NHI + full delegation chain |
| **Proof** | Database log | ML-DSA-65 cryptographic signature |
| **Reasoning** | Not available | Memory context integration |
| **Audit** | Application logs | Merkle-anchored immutable audit |
| **Workflow Pause** | Basic checkpoint | LangGraph-style `interrupt()` semantics |

### 1.3 Key Metrics

| Metric | Target | SLA |
|--------|--------|-----|
| Request creation latency (p99) | <10ms | Contractual |
| State transition latency (p99) | <1ms | Contractual |
| Notification delivery (p95) | <5s | Best effort |
| Concurrent pending requests | ≥10,000 | Per instance |
| Throughput | ≥1,000 req/s | Combined create + respond |
| Uptime | 99.9% | Contractual |

---

## 2. System Architecture

### 2.1 Component Topology

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              OVERSIGHT SERVICE                               │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐        │
│  │   PolicyEngine  │    │  RequestManager │    │  ChannelRouter  │        │
│  │                 │    │                 │    │                 │        │
│  │ • Bloom filter  │◄──►│ • State machine │◄──►│ • Slack adapter │        │
│  │ • Policy cache  │    │ • Quorum calc   │    │ • Email adapter │        │
│  │ • AuthZ bridge  │    │ • Timeout sched │    │ • Webhook       │        │
│  └────────┬────────┘    └────────┬────────┘    └────────┬────────┘        │
│           │                      │                      │                  │
│           └──────────────────────┼──────────────────────┘                  │
│                                  │                                         │
│  ┌───────────────────────────────┴───────────────────────────────────────┐ │
│  │                      DurabilityManager                                 │ │
│  │  • PostgreSQL checkpoint  • Redis cache  • Recovery on startup        │ │
│  └───────────────────────────────────────────────────────────────────────┘ │
│                                  │                                         │
└──────────────────────────────────┼─────────────────────────────────────────┘
                                   │
┌──────────────────────────────────┼─────────────────────────────────────────┐
│                          EXTERNAL DEPENDENCIES                              │
│                                  │                                         │
│  ┌────────────────┐  ┌──────────┴─────────┐  ┌─────────────────┐          │
│  │  Authorization │  │      Memory        │  │      Audit      │          │
│  │  (creto-authz) │  │  (creto-memory)    │  │  (creto-audit)  │          │
│  │                │  │                    │  │                 │          │
│  │ • Policy eval  │  │ • Agent reasoning  │  │ • Merkle tree   │          │
│  │ • Override tok │  │ • Context fetch    │  │ • Immutable log │          │
│  └────────────────┘  └────────────────────┘  └─────────────────┘          │
└───────────────────────────────────────────────────────────────────────────┘
```

### 2.2 Deployment Architecture

```yaml
# kubernetes/oversight-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: creto-oversight
  namespace: creto
spec:
  replicas: 3
  selector:
    matchLabels:
      app: creto-oversight
  template:
    metadata:
      labels:
        app: creto-oversight
    spec:
      containers:
      - name: oversight
        image: creto/oversight:v1.0.0
        ports:
        - containerPort: 8080  # gRPC
        - containerPort: 8081  # REST
        - containerPort: 9090  # Metrics
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: oversight-secrets
              key: database-url
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: oversight-secrets
              key: redis-url
        - name: DATABASE_POOL_SIZE
          value: "25"
        livenessProbe:
          httpGet:
            path: /healthz
            port: 9090
          initialDelaySeconds: 10
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 9090
          initialDelaySeconds: 5
          periodSeconds: 5
```

---

## 3. Core Components

### 3.1 PolicyEngine

**Purpose:** Match incoming authorization decisions to oversight policies.

**Interface:**
```rust
/// Policy engine for oversight requirement resolution
pub struct PolicyEngine {
    /// Bloom filter for fast negative matching (99.9% filter rate)
    bloom_filter: BloomFilter,
    /// Policy cache (Redis-backed, 1h TTL)
    policy_cache: Arc<RwLock<PolicyCache>>,
    /// Connection to Authorization service
    authz_client: AuthzClient,
}

impl PolicyEngine {
    /// Check if action requires oversight
    ///
    /// # Performance
    /// - Bloom filter miss: <100µs
    /// - Cache hit: <500µs
    /// - Cache miss (DB lookup): <5ms
    ///
    /// # Returns
    /// - `None` if no oversight required
    /// - `Some(OversightRequirement)` if oversight required
    pub async fn match_policy(
        &self,
        agent_nhi: &AgentNhi,
        action: &Action,
        resource: &Resource,
    ) -> Result<Option<OversightRequirement>, PolicyError>;

    /// Rebuild bloom filter from all policies
    /// Called on startup and policy updates
    pub async fn rebuild_bloom_filter(&self) -> Result<(), PolicyError>;
}

#[derive(Debug, Clone)]
pub struct OversightRequirement {
    /// Policy that triggered oversight
    pub policy_id: PolicyId,
    /// Escalation chain configuration
    pub escalation_chain: EscalationChain,
    /// Approval quorum type
    pub quorum: ApprovalQuorum,
    /// Notification channels for first tier
    pub channels: Vec<ChannelConfig>,
}

#[derive(Debug, Clone)]
pub enum ApprovalQuorum {
    /// First approver to respond decides
    Any,
    /// All approvers must approve
    All,
    /// N of M approvers must approve
    Threshold { required: u32, total: u32 },
}
```

### 3.2 RequestManager

**Purpose:** Manage oversight request lifecycle and state transitions.

**Interface:**
```rust
/// Central request lifecycle manager
pub struct RequestManager {
    /// State store (PostgreSQL)
    state_store: PostgresStateStore,
    /// Timeout scheduler
    timeout_scheduler: TimeoutScheduler,
    /// Notification router
    channel_router: ChannelRouter,
    /// Audit logger
    audit: AuditClient,
}

impl RequestManager {
    /// Create new oversight request
    ///
    /// # Flow
    /// 1. Validate agent NHI and delegation chain
    /// 2. Fetch agent reasoning from Memory service
    /// 3. Create request in PENDING state
    /// 4. Schedule timeout for first tier
    /// 5. Send notifications via channel router
    /// 6. Return request ID for polling
    ///
    /// # Latency Target: <10ms (p99)
    pub async fn create_request(
        &self,
        trigger: OversightTrigger,
        requirement: OversightRequirement,
    ) -> Result<RequestId, RequestError>;

    /// Submit approval/denial response
    ///
    /// # Flow
    /// 1. Verify approver is eligible for current tier
    /// 2. Verify cryptographic signature
    /// 3. Record response
    /// 4. Calculate quorum
    /// 5. Transition state if quorum met/denied
    /// 6. Log to audit trail
    ///
    /// # Latency Target: <1ms (p99)
    pub async fn submit_response(
        &self,
        request_id: RequestId,
        response: ApprovalResponse,
    ) -> Result<StateTransitionOutcome, ResponseError>;

    /// Get current request state
    pub async fn get_request(
        &self,
        request_id: RequestId,
    ) -> Result<RequestState, RequestError>;

    /// Cancel pending request
    pub async fn cancel_request(
        &self,
        request_id: RequestId,
        reason: &str,
    ) -> Result<(), RequestError>;

    /// Handle tier timeout (called by scheduler)
    pub async fn handle_tier_timeout(
        &self,
        request_id: RequestId,
        tier_index: usize,
    ) -> Result<(), RequestError>;
}

#[derive(Debug, Clone)]
pub struct OversightTrigger {
    /// Agent requesting action
    pub agent_nhi: AgentNhi,
    /// Full delegation chain
    pub delegation_chain: Vec<AgentNhi>,
    /// Action being requested
    pub action: Action,
    /// Resource being acted upon
    pub resource: Resource,
    /// Human-readable description
    pub action_description: String,
}
```

### 3.3 ChannelRouter

**Purpose:** Route notifications to approvers via configured channels.

**Interface:**
```rust
/// Multi-channel notification router
pub struct ChannelRouter {
    /// Channel adapters
    channels: HashMap<ChannelType, Box<dyn NotificationChannel>>,
    /// Delivery tracking
    delivery_store: DeliveryStore,
}

/// Channel adapter trait
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Send notification to approver
    async fn send_notification(
        &self,
        request: &OversightRequest,
        context: &ApprovalContext,
        approver: &ApproverIdentity,
    ) -> Result<DeliveryReceipt, ChannelError>;

    /// Handle response from channel
    async fn handle_response(
        &self,
        channel_response: ChannelResponse,
    ) -> Result<ApprovalResponse, ChannelError>;
}

#[derive(Debug, Clone)]
pub enum ChannelType {
    Slack,
    Email,
    Webhook,
    Teams,      // v2
    ServiceNow, // v2
}
```

### 3.4 DurabilityManager

**Purpose:** Ensure request state survives process crashes.

**Interface:**
```rust
/// Checkpoint and recovery manager
pub struct DurabilityManager {
    /// PostgreSQL connection pool
    pool: PgPool,
    /// Redis cache client
    redis: RedisClient,
}

impl DurabilityManager {
    /// Checkpoint request state (called after every state change)
    ///
    /// # Latency Target: <5ms (p99)
    pub async fn save_checkpoint(
        &self,
        state: &RequestState,
    ) -> Result<(), DurabilityError>;

    /// Load checkpoint by request ID
    pub async fn load_checkpoint(
        &self,
        request_id: RequestId,
    ) -> Result<Option<RequestState>, DurabilityError>;

    /// Recover all pending requests on startup
    ///
    /// Called during service initialization to resume
    /// any requests that were in-flight during shutdown/crash
    pub async fn recover_on_startup(&self) -> Result<Vec<RequestState>, DurabilityError>;

    /// Optimistic concurrency save (detects concurrent modifications)
    pub async fn save_checkpoint_with_version(
        &self,
        state: &RequestState,
    ) -> Result<(), DurabilityError>;
}
```

---

## 4. State Machine Specification

### 4.1 State Diagram

```
                              ┌─────────────────────────────┐
                              │                             │
                              │         PENDING             │
                              │    (awaiting response)      │
                              │                             │
                              └──────────────┬──────────────┘
                                             │
            ┌────────────────────────────────┼────────────────────────────────┐
            │                                │                                │
            ▼                                ▼                                ▼
┌───────────────────────┐      ┌───────────────────────┐      ┌───────────────────────┐
│                       │      │                       │      │                       │
│       APPROVED        │      │        DENIED         │      │       ESCALATED       │
│   (quorum achieved)   │      │    (denial received)  │      │    (tier timeout)     │
│                       │      │                       │      │                       │
│ Actions:              │      │ Actions:              │      │ Actions:              │
│ • Issue override token│      │ • Notify agent        │      │ • Notify next tier    │
│ • Notify agent        │      │ • Log denial reason   │      │ • Reset timeout       │
│ • Resume workflow     │      │                       │      │ • Update tier_index   │
│                       │      │                       │      │                       │
└───────────────────────┘      └───────────────────────┘      └───────────┬───────────┘
                                                                          │
                                                                          │ Loop back
                                                                          │ to PENDING
                                                                          │
                              ┌───────────────────────┐                   │
                              │                       │◄──────────────────┘
                              │      TIMED_OUT        │
                              │  (final tier timeout) │
                              │                       │
                              │ Actions:              │
                              │ • Execute final_action│
                              │   (AUTO_DENY/APPROVE/ │
                              │    BLOCK_INDEFINITELY)│
                              │                       │
                              └───────────────────────┘

                              ┌───────────────────────┐
                              │                       │
                              │      CANCELLED        │
                              │   (agent/system)      │
                              │                       │
                              │ Actions:              │
                              │ • Notify agent        │
                              │ • Log cancellation    │
                              │                       │
                              └───────────────────────┘
```

### 4.2 State Transition Table

| Current State | Trigger | Next State | Guard Conditions | Side Effects |
|---------------|---------|------------|------------------|--------------|
| `PENDING` | Approval received | `PENDING` | Quorum NOT met | Record response |
| `PENDING` | Approval received | `APPROVED` | Quorum met | Issue override token, notify agent |
| `PENDING` | Denial received | `DENIED` | Always | Notify agent, log reason |
| `PENDING` | Tier timeout | `ESCALATED` | More tiers exist | Notify next tier, reset timeout |
| `PENDING` | Tier timeout | `TIMED_OUT` | Final tier | Execute `final_action` |
| `PENDING` | Cancel request | `CANCELLED` | Request owner | Notify agent |
| `ESCALATED` | — | `PENDING` | Immediate | New tier active |
| `APPROVED` | Any | N/A | — | Reject (terminal state) |
| `DENIED` | Any | N/A | — | Reject (terminal state) |
| `TIMED_OUT` | Any | N/A | — | Reject (terminal state) |
| `CANCELLED` | Any | N/A | — | Reject (terminal state) |

### 4.3 State Transition Implementation

```rust
impl RequestState {
    /// Apply state transition with validation
    pub fn apply_transition(
        &mut self,
        event: StateEvent,
        clock: &dyn Clock,
    ) -> Result<StateTransitionOutcome, StateError> {
        // Guard: No transitions from terminal states
        if self.state.is_terminal() {
            return Err(StateError::TransitionFromTerminalState {
                current: self.state,
                event,
            });
        }

        match event {
            StateEvent::ResponseReceived { response } => {
                self.handle_response(response, clock)
            }
            StateEvent::TierTimeout { tier_index } => {
                self.handle_tier_timeout(tier_index, clock)
            }
            StateEvent::CancelRequested { reason } => {
                self.handle_cancel(reason, clock)
            }
        }
    }

    fn handle_response(
        &mut self,
        response: ApprovalResponse,
        clock: &dyn Clock,
    ) -> Result<StateTransitionOutcome, StateError> {
        // Guard: Verify approver is eligible for current tier
        if !self.is_approver_eligible(&response.approver) {
            return Err(StateError::ApproverNotEligible {
                approver: response.approver.subject.clone(),
                tier_index: self.tier_index,
            });
        }

        // Guard: Verify not duplicate response
        if self.responses.iter().any(|r| r.approver.subject == response.approver.subject) {
            return Ok(StateTransitionOutcome::Duplicate);
        }

        // Record response
        self.responses.push(response.clone());
        self.updated_at = clock.now();
        self.version += 1;

        // Check for denial (immediate state change)
        if response.decision == ApprovalDecision::Deny {
            self.state = State::Denied;
            return Ok(StateTransitionOutcome::Transitioned {
                new_state: State::Denied,
                reason: "Denial received".to_string(),
            });
        }

        // Calculate quorum
        let approvals = self.responses.iter()
            .filter(|r| r.decision == ApprovalDecision::Approve)
            .count();

        let quorum_met = match &self.quorum {
            ApprovalQuorum::Any => approvals >= 1,
            ApprovalQuorum::All => approvals >= self.current_tier_approvers().len(),
            ApprovalQuorum::Threshold { required, .. } => approvals >= *required as usize,
        };

        if quorum_met {
            self.state = State::Approved;
            Ok(StateTransitionOutcome::Transitioned {
                new_state: State::Approved,
                reason: format!("Quorum met: {} approvals", approvals),
            })
        } else {
            Ok(StateTransitionOutcome::ResponseRecorded {
                approvals_so_far: approvals,
                approvals_needed: self.approvals_needed(),
            })
        }
    }

    fn handle_tier_timeout(
        &mut self,
        tier_index: usize,
        clock: &dyn Clock,
    ) -> Result<StateTransitionOutcome, StateError> {
        // Guard: Verify timeout is for current tier
        if tier_index != self.tier_index {
            return Err(StateError::TimeoutTierMismatch {
                expected: self.tier_index,
                received: tier_index,
            });
        }

        // Check if more tiers exist
        if self.tier_index + 1 < self.escalation_chain.tiers.len() {
            // Escalate to next tier
            self.tier_index += 1;
            self.state = State::Escalated;
            self.updated_at = clock.now();
            self.version += 1;

            Ok(StateTransitionOutcome::Escalated {
                from_tier: tier_index,
                to_tier: self.tier_index,
            })
        } else {
            // Final tier timeout
            self.state = State::TimedOut;
            self.updated_at = clock.now();
            self.version += 1;

            Ok(StateTransitionOutcome::FinalTimeout {
                action: self.escalation_chain.final_action.clone(),
            })
        }
    }
}
```

---

## 5. API Contracts

### 5.1 gRPC Service Definition

```protobuf
syntax = "proto3";

package creto.oversight.v1;

import "google/protobuf/timestamp.proto";
import "google/protobuf/duration.proto";

// Oversight service for human-in-the-loop approvals
service OversightService {
  // Create oversight request (called by Authorization service)
  rpc CreateRequest(CreateRequestRequest) returns (CreateRequestResponse);

  // Submit approval/denial response
  rpc SubmitResponse(SubmitResponseRequest) returns (SubmitResponseResponse);

  // Get request status
  rpc GetRequest(GetRequestRequest) returns (GetRequestResponse);

  // List requests with filters
  rpc ListRequests(ListRequestsRequest) returns (ListRequestsResponse);

  // Cancel pending request
  rpc CancelRequest(CancelRequestRequest) returns (CancelRequestResponse);

  // Stream request updates (for real-time UI)
  rpc WatchRequest(WatchRequestRequest) returns (stream RequestUpdate);

  // Await approval (LangGraph interrupt() semantics)
  rpc AwaitApproval(AwaitApprovalRequest) returns (AwaitApprovalResponse);
}

message CreateRequestRequest {
  // Agent identity
  string agent_nhi = 1;
  // Delegation chain (root to agent)
  repeated string delegation_chain = 2;
  // Action being requested
  Action action = 3;
  // Resource being acted upon
  Resource resource = 4;
  // Human-readable description
  string action_description = 5;
  // Policy that triggered oversight
  string policy_id = 6;
  // Oversight requirement from policy
  OversightRequirement requirement = 7;
  // Optional: Agent reasoning context
  string reasoning = 8;
  // Optional: Risk factors
  repeated RiskFactor risk_factors = 9;
  // Idempotency key (prevents duplicate requests)
  string idempotency_key = 10;
}

message CreateRequestResponse {
  // Unique request identifier
  string request_id = 1;
  // Current state
  RequestState state = 2;
  // Estimated time to first response
  google.protobuf.Duration estimated_response_time = 3;
}

message SubmitResponseRequest {
  // Request being responded to
  string request_id = 1;
  // Approver identity
  ApproverIdentity approver = 2;
  // Decision
  ApprovalDecision decision = 3;
  // Optional reason
  string reason = 4;
  // Cryptographic signature
  Signature signature = 5;
  // Channel response was submitted through
  ChannelType channel = 6;
  // Channel-specific metadata
  map<string, string> channel_metadata = 7;
}

message SubmitResponseResponse {
  // Whether response was accepted
  bool accepted = 1;
  // New state after response
  RequestState new_state = 2;
  // Reason if rejected
  string rejection_reason = 3;
  // Override token (if approved)
  OverrideToken override_token = 4;
}

// LangGraph interrupt() semantics
message AwaitApprovalRequest {
  // Request to wait for
  string request_id = 1;
  // Maximum time to wait
  google.protobuf.Duration timeout = 2;
  // Polling interval
  google.protobuf.Duration poll_interval = 3;
}

message AwaitApprovalResponse {
  // Final state
  RequestState state = 1;
  // Override token if approved
  OverrideToken override_token = 2;
  // All responses received
  repeated ApprovalResponseRecord responses = 3;
  // Time waited
  google.protobuf.Duration elapsed = 4;
}

enum RequestState {
  REQUEST_STATE_UNSPECIFIED = 0;
  REQUEST_STATE_PENDING = 1;
  REQUEST_STATE_APPROVED = 2;
  REQUEST_STATE_DENIED = 3;
  REQUEST_STATE_ESCALATED = 4;
  REQUEST_STATE_TIMED_OUT = 5;
  REQUEST_STATE_CANCELLED = 6;
}

enum ApprovalDecision {
  APPROVAL_DECISION_UNSPECIFIED = 0;
  APPROVAL_DECISION_APPROVE = 1;
  APPROVAL_DECISION_DENY = 2;
  APPROVAL_DECISION_REQUEST_MORE_INFO = 3;
}

message ApproverIdentity {
  // Subject identifier (email, NHI)
  string subject = 1;
  // Display name
  string name = 2;
  // Public key for signature verification
  bytes public_key = 3;
}

message Signature {
  // Algorithm (ML-DSA-65, ML-DSA-87, Ed25519)
  string algorithm = 1;
  // Signature bytes
  bytes value = 2;
}

message OverrideToken {
  // Token value
  string token = 1;
  // Expiration time
  google.protobuf.Timestamp expires_at = 2;
  // Signed by Oversight service
  Signature signature = 3;
}
```

### 5.2 REST API (OpenAPI 3.0)

```yaml
openapi: 3.0.3
info:
  title: Creto Oversight API
  version: 1.0.0
  description: Human-in-the-Loop approval service

paths:
  /api/v1/requests:
    post:
      operationId: createRequest
      summary: Create oversight request
      tags: [Requests]
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateRequestInput'
      responses:
        '201':
          description: Request created
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OversightRequest'
        '400':
          $ref: '#/components/responses/BadRequest'
        '409':
          $ref: '#/components/responses/Conflict'

  /api/v1/requests/{requestId}:
    get:
      operationId: getRequest
      summary: Get request by ID
      tags: [Requests]
      parameters:
        - name: requestId
          in: path
          required: true
          schema:
            type: string
            format: uuid
      responses:
        '200':
          description: Request found
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/OversightRequest'
        '404':
          $ref: '#/components/responses/NotFound'

  /api/v1/requests/{requestId}/responses:
    post:
      operationId: submitResponse
      summary: Submit approval/denial
      tags: [Responses]
      parameters:
        - name: requestId
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
              $ref: '#/components/schemas/SubmitResponseInput'
      responses:
        '200':
          description: Response accepted
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/SubmitResponseOutput'
        '400':
          $ref: '#/components/responses/BadRequest'
        '403':
          $ref: '#/components/responses/Forbidden'
        '404':
          $ref: '#/components/responses/NotFound'
        '409':
          $ref: '#/components/responses/Conflict'

  /api/v1/requests/{requestId}/await:
    post:
      operationId: awaitApproval
      summary: Wait for approval (LangGraph interrupt() semantics)
      description: |
        Blocks until request reaches terminal state or timeout.
        Use for synchronous approval workflows.
      tags: [Requests]
      parameters:
        - name: requestId
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
              $ref: '#/components/schemas/AwaitApprovalInput'
      responses:
        '200':
          description: Request resolved
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/AwaitApprovalOutput'
        '408':
          description: Timeout waiting for approval

components:
  schemas:
    CreateRequestInput:
      type: object
      required:
        - agent_nhi
        - action
        - resource
        - action_description
        - policy_id
        - requirement
      properties:
        agent_nhi:
          type: string
          example: "agent:payment-bot-v3@company.creto"
        delegation_chain:
          type: array
          items:
            type: string
          example: ["agent:payment-bot-v3@company.creto", "human:alice@company.com"]
        action:
          type: string
          example: "TransferFunds"
        resource:
          type: object
          additionalProperties: true
          example:
            amount: 50000
            currency: "USD"
            recipient: "vendor@example.com"
        action_description:
          type: string
          example: "Transfer $50,000 to vendor invoice #INV-2024-1234"
        policy_id:
          type: string
          example: "pol_large_transfer_cfo_approval"
        requirement:
          $ref: '#/components/schemas/OversightRequirement'
        reasoning:
          type: string
          example: "Invoice approved in AP system. Due date: 2024-12-31."
        risk_factors:
          type: array
          items:
            $ref: '#/components/schemas/RiskFactor'
        idempotency_key:
          type: string
          format: uuid

    OversightRequirement:
      type: object
      required:
        - escalation_chain
        - quorum
      properties:
        escalation_chain:
          $ref: '#/components/schemas/EscalationChain'
        quorum:
          $ref: '#/components/schemas/ApprovalQuorum'

    EscalationChain:
      type: object
      required:
        - tiers
        - final_action
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
      required:
        - approvers
        - timeout_seconds
        - channels
      properties:
        tier_id:
          type: string
        approvers:
          type: array
          items:
            type: string
          example: ["cfo@company.com"]
        timeout_seconds:
          type: integer
          minimum: 60
          maximum: 604800
          example: 7200
        channels:
          type: array
          items:
            type: string
            enum: [SLACK, EMAIL, WEBHOOK]
        quorum_override:
          $ref: '#/components/schemas/ApprovalQuorum'

    ApprovalQuorum:
      type: object
      required:
        - type
      properties:
        type:
          type: string
          enum: [ANY, ALL, THRESHOLD]
        required:
          type: integer
          description: Required for THRESHOLD type

    RiskFactor:
      type: object
      properties:
        category:
          type: string
          enum: [FINANCIAL, COMPLIANCE, SECURITY, OPERATIONAL]
        severity:
          type: string
          enum: [LOW, MEDIUM, HIGH, CRITICAL]
        description:
          type: string

    AwaitApprovalInput:
      type: object
      properties:
        timeout_seconds:
          type: integer
          minimum: 1
          maximum: 86400
          default: 7200
          description: Maximum time to wait (default 2 hours)
        poll_interval_seconds:
          type: integer
          minimum: 1
          maximum: 60
          default: 5
          description: Polling interval (default 5 seconds)

    AwaitApprovalOutput:
      type: object
      properties:
        state:
          type: string
          enum: [APPROVED, DENIED, TIMED_OUT, CANCELLED]
        override_token:
          type: string
          description: Present only if approved
        responses:
          type: array
          items:
            $ref: '#/components/schemas/ApprovalResponseRecord'
        elapsed_seconds:
          type: integer
```

---

## 6. SDK Patterns

### 6.1 LangGraph interrupt() Semantics

**This is a P0 gap from OSS alignment report.**

The `.await_approval()` SDK pattern provides LangGraph-style workflow pause/resume:

```python
# Python SDK - LangGraph interrupt() style
from creto_oversight import OversightClient, AwaitConfig

client = OversightClient(endpoint="oversight.company.com:8080")

# Create oversight request
request_id = await client.create_request(
    agent_nhi="agent:payment-bot@company.creto",
    action="TransferFunds",
    resource={"amount": 50000, "currency": "USD"},
    action_description="Transfer $50K to vendor",
    policy_id="pol_large_transfer",
)

# LangGraph interrupt() - blocks until approval or timeout
result = await client.await_approval(
    request_id=request_id,
    config=AwaitConfig(
        timeout=timedelta(hours=2),
        poll_interval=timedelta(seconds=5),
    )
)

# Result contains final state
if result.state == "APPROVED":
    # Use override token to execute action
    await execute_transfer(override_token=result.override_token)
elif result.state == "DENIED":
    # Log denial and abort
    logger.warning(f"Transfer denied: {result.denial_reason}")
elif result.state == "TIMED_OUT":
    # Handle timeout
    logger.warning("Approval timed out")
```

### 6.2 Rust SDK

```rust
use creto_oversight::{OversightClient, AwaitConfig, CreateRequestInput};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = OversightClient::new("oversight.company.com:8080").await?;

    // Create request
    let request_id = client.create_request(CreateRequestInput {
        agent_nhi: "agent:payment-bot@company.creto".to_string(),
        action: "TransferFunds".to_string(),
        resource: serde_json::json!({
            "amount": 50000,
            "currency": "USD"
        }),
        action_description: "Transfer $50K to vendor".to_string(),
        policy_id: "pol_large_transfer".to_string(),
        ..Default::default()
    }).await?;

    // Await approval (LangGraph interrupt() semantics)
    let result = client.await_approval(
        request_id,
        AwaitConfig {
            timeout: Duration::from_secs(7200),      // 2 hours
            poll_interval: Duration::from_secs(5),   // 5 seconds
        }
    ).await?;

    match result.state {
        State::Approved => {
            let token = result.override_token.unwrap();
            execute_transfer_with_token(token).await?;
        }
        State::Denied => {
            eprintln!("Transfer denied: {:?}", result.responses);
        }
        State::TimedOut => {
            eprintln!("Approval timed out after 2 hours");
        }
        _ => unreachable!("await_approval only returns terminal states"),
    }

    Ok(())
}
```

### 6.3 TypeScript SDK

```typescript
import { OversightClient, AwaitConfig } from '@creto/oversight-sdk';

const client = new OversightClient({
  endpoint: 'oversight.company.com:8080',
  credentials: await loadCredentials(),
});

// Create request
const requestId = await client.createRequest({
  agentNhi: 'agent:payment-bot@company.creto',
  action: 'TransferFunds',
  resource: { amount: 50000, currency: 'USD' },
  actionDescription: 'Transfer $50K to vendor',
  policyId: 'pol_large_transfer',
});

// Await approval (LangGraph interrupt() semantics)
const result = await client.awaitApproval(requestId, {
  timeoutMs: 7200000,     // 2 hours
  pollIntervalMs: 5000,   // 5 seconds
});

switch (result.state) {
  case 'APPROVED':
    await executeTransfer(result.overrideToken);
    break;
  case 'DENIED':
    console.warn(`Transfer denied: ${result.denialReason}`);
    break;
  case 'TIMED_OUT':
    console.warn('Approval timed out');
    break;
}
```

### 6.4 Decorator Pattern (Python)

For compatibility with HumanLayer-style patterns:

```python
from creto_oversight import require_oversight, OversightPolicy

@require_oversight(
    policy_id="pol_large_transfer",
    timeout=timedelta(hours=2),
    channels=["slack", "email"],
)
async def transfer_funds(amount: float, recipient: str) -> TransferResult:
    """Transfer funds to recipient.

    This function will pause for human approval before executing
    if amount > $10,000 (per policy pol_large_transfer).
    """
    # This code only executes after approval
    return await bank_api.transfer(amount, recipient)

# Usage - decorator handles oversight transparently
result = await transfer_funds(amount=50000, recipient="vendor@example.com")
```

---

## 7. Integration Contracts

### 7.1 Authorization Service (creto-authz)

**Contract: Policy Evaluation Return**

When Authorization evaluates a policy that requires oversight:

```rust
/// Authorization service returns this when oversight required
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthzDecision {
    /// Action allowed without oversight
    Allow,
    /// Action denied
    Deny { reason: String },
    /// Action requires human approval
    RequiresOversight {
        /// Policy that triggered oversight
        policy_id: PolicyId,
        /// Full oversight requirement
        requirement: OversightRequirement,
    },
}
```

**Contract: Override Token Validation**

After approval, Oversight issues override token. Authorization validates:

```rust
/// Authorization service validates override token
impl AuthzService {
    pub async fn validate_override_token(
        &self,
        token: &OverrideToken,
        action: &Action,
        resource: &Resource,
    ) -> Result<bool, AuthzError> {
        // 1. Verify token signature (Oversight service's key)
        self.crypto.verify_signature(&token.signature)?;

        // 2. Check token not expired
        if token.expires_at < Timestamp::now() {
            return Err(AuthzError::TokenExpired);
        }

        // 3. Check token not already used (one-time use)
        if self.redis.exists(&format!("used_token:{}", token.nonce)).await? {
            return Err(AuthzError::TokenAlreadyUsed);
        }

        // 4. Mark token as used
        self.redis.set_ex(
            &format!("used_token:{}", token.nonce),
            "1",
            token.expires_at.duration_since(Timestamp::now()).as_secs(),
        ).await?;

        Ok(true)
    }
}
```

### 7.2 Memory Service (creto-memory)

**Contract: Reasoning Context Fetch**

Oversight queries Memory for agent reasoning:

```rust
/// Memory service interface for oversight
pub trait MemoryClient {
    /// Fetch agent reasoning context for approval request
    ///
    /// # Parameters
    /// - `agent_nhi`: Agent requesting action
    /// - `action`: Action being requested
    /// - `limit`: Maximum number of context entries
    ///
    /// # Returns
    /// Relevant reasoning entries (goals, recent interactions)
    async fn fetch_reasoning_context(
        &self,
        agent_nhi: &AgentNhi,
        action: &Action,
        limit: usize,
    ) -> Result<Vec<ReasoningEntry>, MemoryError>;
}

#[derive(Debug, Clone)]
pub struct ReasoningEntry {
    /// Entry timestamp
    pub timestamp: Timestamp,
    /// Entry type (goal, interaction, decision)
    pub entry_type: EntryType,
    /// Human-readable content
    pub content: String,
    /// Relevance score (0.0 - 1.0)
    pub relevance: f64,
}
```

### 7.3 Audit Service (creto-audit)

**Contract: State Transition Logging**

Every state transition logged to immutable audit:

```rust
/// Audit event for oversight state transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversightAuditEvent {
    /// Event ID
    pub event_id: AuditEventId,
    /// Request ID
    pub request_id: RequestId,
    /// Event type
    pub event_type: OversightEventType,
    /// Agent NHI
    pub agent_nhi: AgentNhi,
    /// Approver (if response event)
    pub approver_subject: Option<String>,
    /// Previous state
    pub old_state: Option<State>,
    /// New state
    pub new_state: State,
    /// Decision (if response event)
    pub decision: Option<ApprovalDecision>,
    /// Signature (if response event)
    pub signature: Option<Signature>,
    /// Timestamp
    pub timestamp: Timestamp,
}

#[derive(Debug, Clone)]
pub enum OversightEventType {
    RequestCreated,
    NotificationSent { channel: ChannelType },
    ResponseReceived,
    StateTransition,
    TierEscalation,
    RequestCancelled,
    FinalTimeout,
}
```

### 7.4 NHI Registry (creto-nhi)

**Contract: Delegation Chain Validation**

```rust
/// NHI registry interface for oversight
pub trait NhiRegistry {
    /// Validate delegation chain integrity
    ///
    /// # Validation Rules
    /// 1. Each link in chain is valid NHI
    /// 2. Each delegation is signed by delegator
    /// 3. No delegation is revoked
    /// 4. Chain terminates at human principal
    async fn validate_delegation_chain(
        &self,
        chain: &[AgentNhi],
    ) -> Result<DelegationValidation, NhiError>;

    /// Resolve approver to public key
    async fn get_approver_public_key(
        &self,
        approver_subject: &str,
    ) -> Result<PublicKey, NhiError>;
}

#[derive(Debug, Clone)]
pub struct DelegationValidation {
    /// Is chain valid?
    pub valid: bool,
    /// Root human principal
    pub root_principal: String,
    /// Validation errors (if invalid)
    pub errors: Vec<ValidationError>,
}
```

---

## 8. Data Model

### 8.1 PostgreSQL Schema

```sql
-- Core request table
CREATE TABLE oversight_requests (
    request_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Agent identity
    agent_nhi TEXT NOT NULL,
    delegation_chain JSONB NOT NULL,

    -- Action context
    action TEXT NOT NULL,
    resource JSONB NOT NULL,
    policy_id TEXT NOT NULL,

    -- Request context
    action_description TEXT NOT NULL,
    reasoning TEXT,
    risk_factors JSONB,
    impact_assessment JSONB,

    -- Oversight configuration
    escalation_chain JSONB NOT NULL,
    quorum JSONB NOT NULL,

    -- State machine
    state TEXT NOT NULL DEFAULT 'PENDING'
        CHECK (state IN ('PENDING', 'APPROVED', 'DENIED',
                         'ESCALATED', 'TIMED_OUT', 'CANCELLED')),
    tier_index INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,

    -- Optimistic concurrency
    version BIGINT NOT NULL DEFAULT 0,

    -- Idempotency
    idempotency_key UUID UNIQUE
);

-- Response table
CREATE TABLE approval_responses (
    response_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL REFERENCES oversight_requests(request_id)
        ON DELETE CASCADE,

    -- Approver
    approver_subject TEXT NOT NULL,
    approver_name TEXT NOT NULL,
    approver_public_key BYTEA NOT NULL,

    -- Decision
    decision TEXT NOT NULL CHECK (decision IN ('APPROVE', 'DENY', 'REQUEST_MORE_INFO')),
    reason TEXT,
    question TEXT,

    -- Signature
    signature_algorithm TEXT NOT NULL,
    signature_value BYTEA NOT NULL,

    -- Channel
    channel_type TEXT NOT NULL,
    channel_metadata JSONB,

    -- Timestamp
    responded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One response per approver per request
    UNIQUE(request_id, approver_subject)
);

-- Indexes
CREATE INDEX idx_requests_state ON oversight_requests(state)
    WHERE state IN ('PENDING', 'ESCALATED');
CREATE INDEX idx_requests_agent_nhi ON oversight_requests(agent_nhi);
CREATE INDEX idx_requests_created_at ON oversight_requests(created_at);
CREATE INDEX idx_responses_request_id ON approval_responses(request_id);
```

### 8.2 Redis Cache Schema

```
# Key patterns
checkpoint:{request_id}         → JSON RequestState (TTL: 1h)
policy:{policy_hash}            → JSON OversightRequirement (TTL: 1h)
active_requests                 → Set of request_ids
approver:{subject}:pending      → Set of request_ids awaiting this approver
token:used:{nonce}              → "1" (TTL: token expiry)
```

---

## 9. Error Taxonomy

### 9.1 Error Codes

| Code | Name | HTTP | gRPC | Description | Recovery |
|------|------|------|------|-------------|----------|
| `OVS-001` | `RequestNotFound` | 404 | NOT_FOUND | Request ID does not exist | Check request ID |
| `OVS-002` | `RequestAlreadyResolved` | 409 | FAILED_PRECONDITION | Request in terminal state | No action needed |
| `OVS-003` | `ApproverNotEligible` | 403 | PERMISSION_DENIED | Approver not in current tier | Wait for escalation |
| `OVS-004` | `DuplicateResponse` | 409 | ALREADY_EXISTS | Approver already responded | Idempotent, ignore |
| `OVS-005` | `InvalidSignature` | 400 | INVALID_ARGUMENT | Signature verification failed | Resign response |
| `OVS-006` | `SignatureAlgorithmMismatch` | 400 | INVALID_ARGUMENT | Algorithm doesn't match key | Use correct algorithm |
| `OVS-007` | `DelegationChainInvalid` | 400 | INVALID_ARGUMENT | Chain validation failed | Fix delegation |
| `OVS-008` | `PolicyNotFound` | 400 | NOT_FOUND | Policy ID doesn't exist | Check policy ID |
| `OVS-009` | `IdempotencyConflict` | 409 | ALREADY_EXISTS | Different request with same key | Use different key |
| `OVS-010` | `ConcurrentModification` | 409 | ABORTED | Optimistic lock conflict | Retry |
| `OVS-011` | `TimeoutTierMismatch` | 400 | FAILED_PRECONDITION | Timeout for wrong tier | Internal error |
| `OVS-012` | `ChannelDeliveryFailed` | 502 | UNAVAILABLE | All channels failed | Will retry |
| `OVS-013` | `DatabaseUnavailable` | 503 | UNAVAILABLE | PostgreSQL unreachable | Wait and retry |
| `OVS-014` | `CacheUnavailable` | 503 | UNAVAILABLE | Redis unreachable | Falls back to DB |
| `OVS-015` | `AuthorizationUnavailable` | 503 | UNAVAILABLE | AuthZ service down | Wait and retry |
| `OVS-016` | `MemoryUnavailable` | 503 | UNAVAILABLE | Memory service down | Proceed without context |
| `OVS-017` | `AwaitTimeout` | 408 | DEADLINE_EXCEEDED | Await exceeded timeout | Increase timeout or poll |
| `OVS-018` | `RateLimited` | 429 | RESOURCE_EXHAUSTED | Too many requests | Back off |
| `OVS-019` | `InvalidState` | 400 | INVALID_ARGUMENT | State not valid enum | Check state value |
| `OVS-020` | `QuorumConfigInvalid` | 400 | INVALID_ARGUMENT | Invalid quorum config | Fix quorum spec |

### 9.2 Error Response Format

```rust
/// Structured error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OversightError {
    /// Machine-readable error code
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Additional context
    pub details: Option<serde_json::Value>,
    /// Request ID for tracing
    pub request_id: Option<RequestId>,
    /// Trace ID for debugging
    pub trace_id: String,
    /// Timestamp
    pub timestamp: Timestamp,
    /// Suggested recovery action
    pub recovery: Option<String>,
}

// Example error response
{
    "code": "OVS-003",
    "message": "Approver not eligible for current tier",
    "details": {
        "approver_subject": "alice@company.com",
        "current_tier": 1,
        "eligible_approvers": ["cfo@company.com", "ceo@company.com"]
    },
    "request_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
    "trace_id": "trace-xyz-123",
    "timestamp": "2024-12-25T14:30:00Z",
    "recovery": "Wait for escalation to tier with your approval eligibility"
}
```

---

## 10. Edge Cases & Failure Modes

### 10.1 Edge Cases

| # | Scenario | Expected Behavior | Test Coverage |
|---|----------|-------------------|---------------|
| EC-01 | Approver responds twice | Second response ignored (idempotent), return `DuplicateResponse` | Unit test |
| EC-02 | Approval and denial arrive simultaneously | First to be processed wins (serialized via DB transaction) | Integration test |
| EC-03 | Timeout fires after approval received | No-op, request already in terminal state | Unit test |
| EC-04 | All channels fail to deliver | Log error, retry with backoff, escalate if still failing | Integration test |
| EC-05 | Agent NHI revoked during pending request | Validation at response time, deny if revoked | Integration test |
| EC-06 | Approver key rotated during pending request | Accept both old and new key for 30-day grace period | Unit test |
| EC-07 | Request created with expired policy | Reject at creation, return `PolicyNotFound` | Unit test |
| EC-08 | Database connection lost during state transition | Transaction rollback, retry, log error | Integration test |
| EC-09 | Redis unavailable | Fall back to PostgreSQL, performance degradation | Integration test |
| EC-10 | Authorization service unavailable | Queue request, retry when available | Integration test |
| EC-11 | Memory service unavailable | Create request without reasoning context | Integration test |
| EC-12 | Escalation chain has 0 tiers | Reject at creation with validation error | Unit test |
| EC-13 | Quorum threshold > number of approvers | Reject at creation with validation error | Unit test |
| EC-14 | Same approver in multiple tiers | Allowed, can respond in any tier they're eligible | Unit test |
| EC-15 | Clock skew between nodes | Use consensus time from creto-consensus | Integration test |
| EC-16 | Service restart during pending request | Recover from checkpoint, resume timeouts | E2E test |

### 10.2 Failure Modes

| # | Failure | Detection | Mitigation | Recovery |
|---|---------|-----------|------------|----------|
| FM-01 | PostgreSQL primary down | Health check fails | Failover to standby | Automatic via Patroni |
| FM-02 | Redis cluster failure | Connection refused | Fall back to PostgreSQL | Automatic |
| FM-03 | Slack API unavailable | HTTP 5xx | Retry with backoff, failover to email | Automatic |
| FM-04 | Email delivery failure | SMTP error | Retry 3x, log permanent failure | Manual resend |
| FM-05 | Webhook target unavailable | HTTP timeout | Retry with exponential backoff | Automatic |
| FM-06 | Memory leak | OOM killer | Pod restart | Automatic via K8s |
| FM-07 | Timeout scheduler crash | Heartbeat missing | Leader election, takeover | Automatic |
| FM-08 | Signature verification service down | gRPC unavailable | Reject responses until available | Automatic when restored |
| FM-09 | Audit service unavailable | gRPC unavailable | Buffer events locally, flush when available | Automatic |
| FM-10 | Consensus time unavailable | gRPC unavailable | Use local time with warning | Automatic when restored |

### 10.3 Failure Recovery Procedures

**FM-01: PostgreSQL Primary Down**

```bash
# Automatic failover via Patroni (no manual action needed)
# Verify failover completed:
patronictl list

# If manual intervention needed:
patronictl failover --force

# Verify Oversight service reconnected:
kubectl logs -n creto deployment/creto-oversight | grep "database connection"
```

**FM-06: Memory Leak / OOM**

```bash
# K8s will automatically restart pod
# Check restart count:
kubectl get pods -n creto -l app=creto-oversight

# If recurring, capture heap dump before restart:
kubectl exec -n creto $POD -- jemalloc-prof-dump

# Analyze with:
jeprof target/debug/creto-oversight heap_dump
```

---

## 11. Sequence Diagrams

### 11.1 Simple Approval Flow

```
┌─────────┐     ┌──────────────┐     ┌────────────┐     ┌────────────┐     ┌──────────┐
│  Agent  │     │ Authorization│     │  Oversight │     │   Slack    │     │ Approver │
└────┬────┘     └──────┬───────┘     └─────┬──────┘     └─────┬──────┘     └────┬─────┘
     │                 │                    │                   │                 │
     │ TransferFunds   │                    │                   │                 │
     │ $50,000         │                    │                   │                 │
     │────────────────►│                    │                   │                 │
     │                 │                    │                   │                 │
     │                 │ Policy: amount>10K │                   │                 │
     │                 │ requires oversight │                   │                 │
     │                 │                    │                   │                 │
     │  REQUIRES_      │                    │                   │                 │
     │  OVERSIGHT      │                    │                   │                 │
     │◄────────────────│                    │                   │                 │
     │                 │                    │                   │                 │
     │ CreateRequest   │                    │                   │                 │
     │─────────────────────────────────────►│                   │                 │
     │                 │                    │                   │                 │
     │                 │                    │ Post Blocks msg   │                 │
     │                 │                    │──────────────────►│                 │
     │                 │                    │                   │                 │
     │                 │                    │     200 OK        │                 │
     │                 │                    │◄──────────────────│                 │
     │                 │                    │                   │                 │
     │ request_id      │                    │                   │ 🔔 Approval     │
     │◄─────────────────────────────────────│                   │ Required        │
     │                 │                    │                   │────────────────►│
     │                 │                    │                   │                 │
     │ AwaitApproval   │                    │                   │                 │
     │─────────────────────────────────────►│                   │  [Approve]     │
     │                 │                    │                   │◄────────────────│
     │                 │                    │                   │                 │
     │                 │                    │ Interaction       │                 │
     │                 │                    │ callback          │                 │
     │                 │                    │◄──────────────────│                 │
     │                 │                    │                   │                 │
     │                 │                    │ Verify signature  │                 │
     │                 │                    │ Update state      │                 │
     │                 │                    │ Log to audit      │                 │
     │                 │                    │                   │                 │
     │ APPROVED +      │                    │                   │                 │
     │ override_token  │                    │                   │                 │
     │◄─────────────────────────────────────│                   │                 │
     │                 │                    │                   │                 │
     │ TransferFunds   │                    │                   │                 │
     │ + override      │                    │                   │                 │
     │────────────────►│                    │                   │                 │
     │                 │                    │                   │                 │
     │                 │ Validate token     │                   │                 │
     │                 │ Execute action     │                   │                 │
     │                 │                    │                   │                 │
     │    ALLOW        │                    │                   │                 │
     │◄────────────────│                    │                   │                 │
     │                 │                    │                   │                 │
```

### 11.2 Escalation Flow

```
┌─────────┐     ┌────────────┐     ┌────────────┐     ┌────────────┐     ┌──────────┐
│  Agent  │     │  Oversight │     │  Slack T1  │     │ Email T2   │     │   CEO    │
└────┬────┘     └─────┬──────┘     └─────┬──────┘     └─────┬──────┘     └────┬─────┘
     │                │                   │                  │                 │
     │ CreateRequest  │                   │                  │                 │
     │ (2-tier chain) │                   │                  │                 │
     │───────────────►│                   │                  │                 │
     │                │                   │                  │                 │
     │ request_id     │ Tier 1: CFO      │                  │                 │
     │◄───────────────│ 1h timeout       │                  │                 │
     │                │──────────────────►│                  │                 │
     │ AwaitApproval  │                   │                  │                 │
     │───────────────►│                   │                  │                 │
     │                │                   │                  │                 │
     │                │                   │                  │                 │
     │                │                   │                  │                 │
     │                │◄──── 1 hour ─────►│                  │                 │
     │                │    NO RESPONSE    │                  │                 │
     │                │                   │                  │                 │
     │                │ TIER TIMEOUT      │                  │                 │
     │                │ State: ESCALATED  │                  │                 │
     │                │                   │                  │                 │
     │                │ Tier 2: CEO       │                  │                 │
     │                │ 2h timeout        │                  │                 │
     │                │─────────────────────────────────────►│                 │
     │                │                   │                  │  📧 Urgent     │
     │                │                   │                  │  Approval      │
     │                │                   │                  │────────────────►│
     │                │                   │                  │                 │
     │                │                   │                  │  Click approve │
     │                │                   │                  │  link          │
     │                │                   │                  │◄────────────────│
     │                │                   │                  │                 │
     │                │ Email callback    │                  │                 │
     │                │◄─────────────────────────────────────│                 │
     │                │                   │                  │                 │
     │                │ Verify token      │                  │                 │
     │                │ State: APPROVED   │                  │                 │
     │                │                   │                  │                 │
     │ APPROVED       │                   │                  │                 │
     │◄───────────────│                   │                  │                 │
     │                │                   │                  │                 │
```

### 11.3 Quorum (2-of-3) Flow

```
┌─────────┐     ┌────────────┐     ┌──────────┐     ┌──────────┐     ┌──────────┐
│  Agent  │     │  Oversight │     │ Approver1│     │ Approver2│     │ Approver3│
└────┬────┘     └─────┬──────┘     └────┬─────┘     └────┬─────┘     └────┬─────┘
     │                │                  │                │                │
     │ CreateRequest  │                  │                │                │
     │ Quorum: 2-of-3 │                  │                │                │
     │───────────────►│                  │                │                │
     │                │                  │                │                │
     │                │ Notify all 3     │                │                │
     │                │──────────────────►─────────────────►───────────────►
     │                │                  │                │                │
     │ request_id     │                  │                │                │
     │◄───────────────│                  │                │                │
     │                │                  │                │                │
     │ AwaitApproval  │                  │                │                │
     │───────────────►│                  │                │                │
     │                │                  │                │                │
     │                │                  │  [Approve]     │                │
     │                │◄─────────────────│                │                │
     │                │                  │                │                │
     │                │ Quorum: 1/2      │                │                │
     │                │ State: PENDING   │                │                │
     │                │                  │                │                │
     │                │                  │                │  [Approve]     │
     │                │◄───────────────────────────────────│                │
     │                │                  │                │                │
     │                │ Quorum: 2/2 ✓    │                │                │
     │                │ State: APPROVED  │                │                │
     │                │                  │                │                │
     │ APPROVED       │                  │                │                │
     │◄───────────────│                  │                │                │
     │                │                  │                │                │
     │                │                  │                │  [Approve]     │
     │                │◄─────────────────────────────────────────────────── │
     │                │                  │                │   (redundant)  │
     │                │ Already resolved │                │                │
     │                │ Ignore response  │                │                │
     │                │                  │                │                │
```

---

## 12. Security Model

### 12.1 Threat Model

| Threat | Actor | Impact | Mitigation |
|--------|-------|--------|------------|
| Approval bypass | Malicious agent | Execute unauthorized action | Override tokens single-use, 60s expiry |
| Signature forgery | Attacker | Forge approval | ML-DSA-65/87 quantum-resistant signatures |
| Channel impersonation | Attacker | Submit fake approval | HMAC on webhooks, Slack signing verification |
| Replay attack | Attacker | Reuse old approval | Signature binds request_id + timestamp |
| Timeout manipulation | Insider | Force auto-approval | Consensus-ordered time, immutable timeouts |
| SQL injection | Attacker | Data exfiltration | Parameterized queries, input validation |

### 12.2 Cryptographic Signatures

```rust
/// Sign approval response
pub fn sign_approval(
    request_id: &RequestId,
    decision: &ApprovalDecision,
    timestamp: Timestamp,
    private_key: &PrivateKey,
) -> Result<Signature, SignatureError> {
    // Construct canonical message
    let message = format!(
        "{}||{}||{}",
        request_id,
        decision.to_string(),
        timestamp.as_secs()
    );

    // Sign with ML-DSA-65 or Ed25519
    let signature_value = match private_key.algorithm() {
        KeyAlgorithm::MlDsa65 => ml_dsa::sign(private_key, message.as_bytes())?,
        KeyAlgorithm::MlDsa87 => ml_dsa::sign(private_key, message.as_bytes())?,
        KeyAlgorithm::Ed25519 => ed25519::sign(private_key, message.as_bytes())?,
    };

    Ok(Signature {
        algorithm: private_key.algorithm().to_string(),
        value: signature_value,
    })
}
```

### 12.3 Access Control Matrix

| Operation | OversightAdmin | Approver | Agent | ServiceAccount | Auditor |
|-----------|----------------|----------|-------|----------------|---------|
| Create request | ✓ | ✗ | ✗ | ✓ | ✗ |
| Submit response | ✓ | ✓ (eligible) | ✗ | ✗ | ✗ |
| Get request | ✓ | ✓ (assigned) | ✓ (own) | ✓ | ✓ |
| Cancel request | ✓ | ✗ | ✓ (own) | ✗ | ✗ |
| Manage policies | ✓ | ✗ | ✗ | ✗ | ✗ |
| View audit log | ✓ | ✗ | ✗ | ✗ | ✓ |

---

## 13. Performance Specifications

### 13.1 Latency Targets

| Operation | P50 | P95 | P99 | Max |
|-----------|-----|-----|-----|-----|
| Create request | 2ms | 5ms | 10ms | 50ms |
| Submit response | 200µs | 500µs | 1ms | 5ms |
| Get request | 500µs | 1ms | 2ms | 10ms |
| Await approval (poll) | 1ms | 2ms | 5ms | 20ms |
| Bloom filter check | 10µs | 50µs | 100µs | 500µs |
| Checkpoint write | 1ms | 3ms | 5ms | 20ms |

### 13.2 Throughput Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Create requests/sec | 1,000 | Per instance |
| Submit responses/sec | 5,000 | Per instance |
| Concurrent pending | 10,000 | Per instance |
| Notification delivery/sec | 500 | Per channel |

### 13.3 Capacity Planning

```
# Formula: Required instances
instances = (peak_requests_per_sec / 800) + 1  # 80% capacity buffer

# Example: 2,000 peak requests/sec
instances = (2000 / 800) + 1 = 3.5 → 4 instances

# Formula: Database connections
connections = instances * pool_size
connections = 4 * 25 = 100 connections

# Formula: Redis memory
redis_memory = concurrent_requests * avg_checkpoint_size * 2
redis_memory = 10000 * 5KB * 2 = 100MB per instance

# Formula: PostgreSQL storage (per year)
storage = requests_per_day * 365 * avg_row_size * 2 (responses)
storage = 100000 * 365 * 2KB * 2 = 146GB per year
```

---

## 14. Operational Procedures

### 14.1 Health Checks

```yaml
# Liveness probe - basic process health
livenessProbe:
  httpGet:
    path: /healthz
    port: 9090
  initialDelaySeconds: 10
  periodSeconds: 10
  failureThreshold: 3

# Readiness probe - dependency health
readinessProbe:
  httpGet:
    path: /ready
    port: 9090
  initialDelaySeconds: 5
  periodSeconds: 5
  failureThreshold: 2
```

**Health check implementation:**

```rust
/// Readiness check - verifies all dependencies
pub async fn ready_check(&self) -> ReadyStatus {
    let db_ok = self.pool.acquire().await.is_ok();
    let redis_ok = self.redis.ping().await.is_ok();

    ReadyStatus {
        ready: db_ok && redis_ok,
        checks: vec![
            Check { name: "database", healthy: db_ok },
            Check { name: "redis", healthy: redis_ok },
        ],
    }
}
```

### 14.2 Monitoring Dashboard

**Key metrics to monitor:**

```promql
# Request creation rate
rate(oversight_request_created_total[5m])

# State transitions by type
sum by (transition) (rate(oversight_state_transition_total[5m]))

# P99 latency
histogram_quantile(0.99, rate(oversight_request_duration_seconds_bucket[5m]))

# Pending requests (should not grow unbounded)
oversight_pending_requests_total

# Notification delivery success rate
sum(rate(oversight_notification_delivered_total[5m])) /
sum(rate(oversight_notification_sent_total[5m]))

# Error rate
sum(rate(oversight_errors_total[5m])) /
sum(rate(oversight_requests_total[5m]))
```

### 14.3 Alert Definitions

| Alert | Condition | Severity | Runbook |
|-------|-----------|----------|---------|
| HighLatency | p99 > 10ms for 5min | Warning | §14.4.1 |
| ServiceDown | 0 healthy pods for 1min | Critical | §14.4.2 |
| DatabaseDown | DB health check failing | Critical | §14.4.3 |
| HighPendingCount | >8000 pending for 10min | Warning | §14.4.4 |
| NotificationFailures | <98% success for 10min | Warning | §14.4.5 |
| StuckRequests | >10 pending >24h | Warning | §14.4.6 |

### 14.4 Runbook Procedures

#### 14.4.1 High Latency

```bash
# 1. Check database query performance
psql -d creto_oversight -c "
SELECT query, calls, mean_exec_time
FROM pg_stat_statements
ORDER BY mean_exec_time DESC LIMIT 10;
"

# 2. Check Redis latency
redis-cli --latency-history

# 3. Scale if needed
kubectl scale deployment/creto-oversight -n creto --replicas=5
```

#### 14.4.2 Service Down

```bash
# 1. Check pod status
kubectl get pods -n creto -l app=creto-oversight

# 2. Check events
kubectl describe pod -n creto $POD_NAME

# 3. Check logs
kubectl logs -n creto $POD_NAME --tail=100

# 4. Force restart if needed
kubectl rollout restart deployment/creto-oversight -n creto
```

#### 14.4.6 Stuck Requests

```bash
# 1. Query stuck requests
psql -d creto_oversight -c "
SELECT request_id, agent_nhi, created_at
FROM oversight_requests
WHERE state IN ('PENDING', 'ESCALATED')
  AND created_at < NOW() - INTERVAL '24 hours';
"

# 2. Check notification delivery
psql -d creto_oversight -c "
SELECT request_id, channel_type, status, error_message
FROM notification_log
WHERE request_id = '$REQUEST_ID';
"

# 3. Manually escalate if needed
curl -X POST https://oversight.company.com/api/v1/admin/requests/$REQUEST_ID/escalate \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

---

## Appendix A: Glossary

| Term | Definition |
|------|------------|
| **Oversight Request** | A pending human approval for an agent action |
| **Escalation Chain** | Multi-tier approval flow with timeout-based escalation |
| **Quorum** | Approval requirement (any, all, N-of-M) |
| **Override Token** | One-time token issued after approval to bypass policy |
| **NHI** | Non-Human Identity - agent identity credential |
| **Delegation Chain** | Lineage from agent to root human principal |
| **Checkpoint** | Durable state snapshot for crash recovery |

---

## Appendix B: Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-12-25 | Architecture Team | Initial MASTER SDD |

---

**END OF MASTER SDD**
