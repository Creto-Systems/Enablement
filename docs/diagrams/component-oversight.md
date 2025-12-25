# Oversight Product - Component Diagram

## Overview

This component diagram details the internal architecture of the Oversight product, which manages approval workflows, policy enforcement, and human-in-the-loop controls for AI agent operations. It shows the state machine for approval requests, multi-channel notification routing, and durability guarantees.

## Purpose

- Detail components within the Oversight product container
- Show the approval request state machine (PENDING → APPROVED/REJECTED/ESCALATED/TIMEOUT)
- Illustrate multi-channel notification routing (Slack, Email, Webhook)
- Visualize policy evaluation engine using CEL expressions
- Document durability and retry mechanisms

## Diagram

```mermaid
graph TB
    subgraph External["External Systems"]
        Agent[AI Agent]
        Human[Human Approver]
        Slack[Slack API]
        Email[Email Service]
        Webhook[Webhook Endpoint]
        Audit[Audit Logger]
    end

    subgraph APIGateway["API Gateway Layer"]
        LB[Load Balancer]
        Gateway[API Gateway<br/>Kong/Envoy]
    end

    subgraph OversightProduct["Oversight Product Container"]

        subgraph RequestManagement["Request Management Layer"]
            RequestAPI[Request Manager API<br/>gRPC Service]
            StateMachine[Approval State Machine<br/>PENDING→APPROVED/REJECTED/<br/>ESCALATED/TIMEOUT]
            TimeoutScheduler[Timeout Scheduler<br/>Cron Jobs]
            EscalationEngine[Escalation Engine<br/>Rule-based Routing]
        end

        subgraph PolicyEvaluation["Policy Evaluation Layer"]
            PolicyAPI[Policy Engine API<br/>gRPC Service]
            CELEvaluator[CEL Expression Evaluator<br/>google/cel-go]
            PolicyCache[Policy Cache<br/>In-Memory LRU]
            PolicyVersioning[Policy Version Manager<br/>Immutable Snapshots]
        end

        subgraph NotificationRouting["Notification Layer"]
            ChannelRouter[Channel Router<br/>Multi-Channel Dispatcher]
            SlackNotifier[Slack Notifier<br/>Webhook + Interactive Buttons]
            EmailNotifier[Email Notifier<br/>SMTP + HTML Templates]
            WebhookNotifier[Webhook Notifier<br/>HTTP POST + Signatures]
            RetryQueue[Retry Queue<br/>Exponential Backoff]
        end

        subgraph Durability["Durability & Persistence Layer"]
            DurabilityManager[Durability Manager<br/>Transactional Writes]
            StateStore[State Store Writer<br/>PostgreSQL + WAL]
            SnapshotService[Snapshot Service<br/>S3 Policy Backups]
            RecoveryService[Recovery Service<br/>Crash Recovery]
        end

        subgraph DataStores["Data Stores"]
            PostgresDB[(PostgreSQL<br/>oversight_requests<br/>oversight_policies)]
            RedisCache[(Redis Cluster<br/>Pending Requests<br/>5-min TTL)]
            S3Bucket[(S3 Bucket<br/>policy-snapshots<br/>Versioned)]
        end
    end

    %% External → API Gateway
    Agent -->|POST /oversight/requests<br/>Approval Request| LB
    Agent -->|GET /oversight/requests/{id}<br/>Check Status| LB
    Human -->|POST /oversight/approve<br/>Slack Button Click| Slack
    Slack -->|Webhook: Button Click| Gateway
    LB -->|Route| Gateway

    %% API Gateway → Oversight Components
    Gateway -->|gRPC: CreateRequest| RequestAPI
    Gateway -->|gRPC: EvaluatePolicy| PolicyAPI
    Gateway -->|gRPC: HandleCallback| RequestAPI

    %% Request Management Flow
    RequestAPI -->|1. Initialize State| StateMachine
    StateMachine -->|State: PENDING| PolicyAPI
    PolicyAPI -->|2. Evaluate Policy| CELEvaluator

    %% Policy Evaluation
    CELEvaluator -->|Check Cache| PolicyCache
    PolicyCache -->|Cache Miss| PolicyVersioning
    PolicyVersioning -->|Load Policy| PostgresDB
    CELEvaluator -->|Decision: AUTO_APPROVE| StateMachine
    CELEvaluator -->|Decision: REQUIRES_APPROVAL| ChannelRouter
    CELEvaluator -->|Decision: AUTO_REJECT| StateMachine

    %% State Transitions
    StateMachine -->|State: APPROVED| DurabilityManager
    StateMachine -->|State: REJECTED| DurabilityManager
    StateMachine -->|State: TIMEOUT| TimeoutScheduler
    StateMachine -->|State: ESCALATED| EscalationEngine

    %% Notification Routing
    ChannelRouter -->|Route by Priority| SlackNotifier
    ChannelRouter -->|Route by Priority| EmailNotifier
    ChannelRouter -->|Route by Priority| WebhookNotifier

    SlackNotifier -->|POST /chat.postMessage| Slack
    EmailNotifier -->|SMTP Send| Email
    WebhookNotifier -->|HTTP POST + HMAC| Webhook

    %% Retry Logic
    SlackNotifier -->|Failure| RetryQueue
    EmailNotifier -->|Failure| RetryQueue
    WebhookNotifier -->|Failure| RetryQueue
    RetryQueue -->|Retry with Backoff| ChannelRouter

    %% Timeout Handling
    TimeoutScheduler -->|Every 1 min| PostgresDB
    TimeoutScheduler -->|Find Expired| StateMachine
    StateMachine -->|Auto-Reject/Escalate| EscalationEngine

    %% Escalation
    EscalationEngine -->|Escalate to Manager| ChannelRouter
    EscalationEngine -->|Escalate to On-Call| ChannelRouter

    %% Durability & Persistence
    DurabilityManager -->|Transactional Write| StateStore
    StateStore -->|INSERT/UPDATE| PostgresDB
    StateStore -->|Cache Update| RedisCache
    PolicyVersioning -->|Snapshot Policy| SnapshotService
    SnapshotService -->|Upload JSON| S3Bucket

    %% Recovery
    RecoveryService -->|On Startup| PostgresDB
    RecoveryService -->|Recover In-Flight| StateMachine

    %% Audit Logging
    RequestAPI -.->|Log: Request Created| Audit
    StateMachine -.->|Log: State Transition| Audit
    CELEvaluator -.->|Log: Policy Decision| Audit
    ChannelRouter -.->|Log: Notification Sent| Audit

    %% Cross-Product Integration
    RequestAPI -.->|Query Metering<br/>Cost Data| Gateway
    StateMachine -.->|Unblock Runtime<br/>Sandbox Execution| Gateway

    classDef api fill:#e1f5ff,stroke:#0066cc,stroke-width:2px
    classDef cache fill:#fff4e1,stroke:#ff9900,stroke-width:2px
    classDef db fill:#f0f0f0,stroke:#333,stroke-width:2px
    classDef processor fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px
    classDef external fill:#ffe1e1,stroke:#cc0000,stroke-width:2px

    class RequestAPI,PolicyAPI api
    class PolicyCache,RedisCache cache
    class PostgresDB,S3Bucket db
    class StateMachine,CELEvaluator processor
    class Slack,Email,Webhook external
```

## Component Inventory

### Request Management Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Request Manager API** | Rust/gRPC | Creates approval requests, handles callbacks, queries status |
| **Approval State Machine** | Rust FSM | Manages state transitions: PENDING→APPROVED/REJECTED/ESCALATED/TIMEOUT |
| **Timeout Scheduler** | Cron (every 1 min) | Finds expired requests, triggers auto-reject or escalation |
| **Escalation Engine** | Rule-based Router | Routes escalated requests to managers, on-call engineers |

**State Machine Transitions:**
```
PENDING → APPROVED (human approval OR auto-approve policy)
PENDING → REJECTED (human rejection OR auto-reject policy)
PENDING → TIMEOUT (timeout exceeded without response)
PENDING → ESCALATED (timeout → escalation rule)
ESCALATED → APPROVED/REJECTED (escalation target responds)
```

### Policy Evaluation Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Policy Engine API** | Rust/gRPC | Evaluates policies against requests using CEL |
| **CEL Expression Evaluator** | google/cel-go | Executes CEL expressions with request context |
| **Policy Cache** | In-Memory LRU | Caches compiled CEL policies (10,000 entries, 1-hour TTL) |
| **Policy Version Manager** | Immutable Snapshots | Manages policy versions, rollback capability |

**Policy Evaluation Flow:**
1. Receive request metadata (agent_id, resource, action, cost, risk_score)
2. Load applicable policies from cache or PostgreSQL
3. Compile CEL expression (if not cached)
4. Evaluate expression with request context
5. Return decision: `AUTO_APPROVE`, `REQUIRES_APPROVAL`, `AUTO_REJECT`

### Notification Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Channel Router** | Multi-Channel Dispatcher | Routes notifications based on priority, approver preferences |
| **Slack Notifier** | Slack SDK + Webhooks | Posts messages with interactive buttons (Approve/Reject) |
| **Email Notifier** | SMTP Client | Sends HTML emails with approval links |
| **Webhook Notifier** | HTTP Client | POSTs to custom webhook URLs with HMAC signatures |
| **Retry Queue** | Exponential Backoff | Retries failed notifications (3 attempts: 10s, 30s, 90s) |

**Channel Selection Logic:**
- **High priority**: Slack + Email simultaneously
- **Medium priority**: Slack only
- **Low priority**: Email only
- **Custom**: Webhook to integration platform

### Durability & Persistence Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Durability Manager** | Transactional Coordinator | Ensures state updates are atomic and durable |
| **State Store Writer** | PostgreSQL + WAL | Writes approval state with WAL-based durability |
| **Snapshot Service** | S3 Uploader | Backs up policy snapshots for auditing and rollback |
| **Recovery Service** | Crash Recovery | Recovers in-flight requests on service restart |

**Durability Guarantees:**
- **Atomicity**: State transitions are single PostgreSQL transactions
- **Durability**: WAL flushed to disk before acknowledging
- **Recovery**: On startup, scan `oversight_requests` for in-flight requests, resume state machine

## Data Flow Details

### Flow 1: Create Approval Request

```
Agent → API Gateway → Request Manager API
  ↓
1. Validate delegation chain (AuthZ)
2. Initialize state machine: PENDING
3. Persist to PostgreSQL: oversight_requests table
4. Evaluate policy via Policy Engine API
  ↓
Policy Engine:
  - Load applicable policies (cache or DB)
  - Compile CEL expression
  - Evaluate with context: {agent_id, resource, action, cost, metadata}
  - Return decision: AUTO_APPROVE | REQUIRES_APPROVAL | AUTO_REJECT
  ↓
If AUTO_APPROVE:
  - Transition state: PENDING → APPROVED
  - Return response to agent
If AUTO_REJECT:
  - Transition state: PENDING → REJECTED
  - Return error to agent
If REQUIRES_APPROVAL:
  - Route to Channel Router
  - Send notifications (Slack/Email/Webhook)
  - Return request_id to agent
```

**Request Schema:**
```protobuf
message ApprovalRequest {
  string request_id = 1;         // UUID v7
  string agent_id = 2;           // Agent public key hash
  string resource = 3;           // "sandbox_create", "llm_inference", etc.
  string action = 4;             // "execute", "read", "write"
  double estimated_cost = 5;     // USD
  int32 risk_score = 6;          // 0-100
  map<string, string> metadata = 7;
  google.protobuf.Timestamp created_at = 8;
  google.protobuf.Timestamp expires_at = 9;  // Timeout deadline
}
```

### Flow 2: Policy Evaluation (CEL)

**Example Policy:**
```cel
// Auto-approve low-cost, low-risk requests
request.estimated_cost <= 10.0 &&
request.risk_score <= 30 &&
agent.tier == "trusted" &&
time.now() - agent.last_violation > duration("7d")
  ? "AUTO_APPROVE"
  : "REQUIRES_APPROVAL"
```

**Evaluation Context:**
```json
{
  "request": {
    "agent_id": "agent_abc123",
    "resource": "sandbox_create",
    "action": "execute",
    "estimated_cost": 5.50,
    "risk_score": 20,
    "metadata": {"sandbox_type": "gvisor"}
  },
  "agent": {
    "tier": "trusted",
    "monthly_spend": 1200.00,
    "last_violation": "2025-11-15T10:00:00Z"
  },
  "time": {
    "now": "2025-12-25T14:30:00Z"
  }
}
```

**Policy Decision Logic:**
```rust
pub enum PolicyDecision {
    AutoApprove,          // Execute immediately
    RequiresApproval,     // Send to human
    AutoReject,           // Deny immediately
}

impl PolicyEngine {
    pub fn evaluate(&self, request: &ApprovalRequest) -> PolicyDecision {
        let context = build_context(request);
        let policy = self.load_policy(request.resource);
        let result = self.cel_evaluator.eval(policy.expression, context);

        match result {
            "AUTO_APPROVE" => PolicyDecision::AutoApprove,
            "AUTO_REJECT" => PolicyDecision::AutoReject,
            _ => PolicyDecision::RequiresApproval,
        }
    }
}
```

### Flow 3: Multi-Channel Notification

```
Channel Router (receives REQUIRES_APPROVAL decision)
  ↓
1. Determine channels based on priority:
   - High priority: Slack + Email
   - Medium: Slack only
   - Low: Email only
  ↓
2. Slack Notifier:
   - POST to Slack API: /chat.postMessage
   - Include interactive buttons: [Approve] [Reject]
   - Embed request metadata in button callback_id
   - On failure: Add to Retry Queue
  ↓
3. Email Notifier:
   - Render HTML template with request details
   - Include approval link: https://oversight.example.com/approve/{request_id}?token={jwt}
   - Send via SMTP
   - On failure: Add to Retry Queue
  ↓
4. Retry Queue (if notification fails):
   - Attempt 1: Wait 10s, retry
   - Attempt 2: Wait 30s, retry
   - Attempt 3: Wait 90s, retry
   - After 3 failures: Log error, trigger escalation
```

**Slack Message Payload:**
```json
{
  "channel": "#approvals",
  "text": "Approval required for agent_abc123",
  "blocks": [
    {
      "type": "section",
      "text": {
        "type": "mrkdwn",
        "text": "*Resource:* sandbox_create\n*Cost:* $5.50\n*Risk:* 20/100"
      }
    },
    {
      "type": "actions",
      "elements": [
        {
          "type": "button",
          "text": {"type": "plain_text", "text": "Approve"},
          "style": "primary",
          "value": "approve:request_xyz789"
        },
        {
          "type": "button",
          "text": {"type": "plain_text", "text": "Reject"},
          "style": "danger",
          "value": "reject:request_xyz789"
        }
      ]
    }
  ]
}
```

### Flow 4: Approval Callback (Slack Button Click)

```
Human clicks [Approve] button in Slack
  ↓
Slack sends webhook to API Gateway
  ↓
Request Manager API receives callback:
  - Extract request_id from callback_id
  - Validate HMAC signature from Slack
  - Authenticate approver (OAuth token)
  ↓
State Machine transition:
  PENDING → APPROVED
  ↓
Durability Manager:
  - BEGIN TRANSACTION
  - UPDATE oversight_requests SET state='APPROVED', approver_id='user123', approved_at=NOW()
  - COMMIT (WAL flush)
  ↓
Notify agent (via webhook or polling)
  ↓
Audit log: Record approval with approver identity
```

### Flow 5: Timeout & Escalation

```
Timeout Scheduler (cron every 1 minute)
  ↓
Query PostgreSQL:
  SELECT * FROM oversight_requests
  WHERE state='PENDING' AND expires_at < NOW()
  ↓
For each expired request:
  - Evaluate escalation policy:
    - If escalation_enabled: Transition to ESCALATED
    - Else: Transition to TIMEOUT (auto-reject)
  ↓
If ESCALATED:
  - Escalation Engine routes to next approver tier
  - Update expires_at (extend by escalation_timeout)
  - Send new notifications to escalation targets
  ↓
If TIMEOUT:
  - Transition state: PENDING → REJECTED
  - Notify agent of timeout
  - Audit log: Record timeout reason
```

**Escalation Policy Example:**
```yaml
escalation_policy:
  - level: 1
    approvers: ["engineer_alice", "engineer_bob"]
    timeout: 5m
  - level: 2
    approvers: ["manager_charlie"]
    timeout: 15m
  - level: 3
    approvers: ["oncall_pagerduty"]
    timeout: 30m
  - final_action: AUTO_REJECT
```

## Implementation Considerations

### State Machine Durability

**PostgreSQL Schema:**
```sql
CREATE TABLE oversight_requests (
  request_id UUID PRIMARY KEY,
  agent_id TEXT NOT NULL,
  resource TEXT NOT NULL,
  action TEXT NOT NULL,
  state TEXT NOT NULL,  -- PENDING, APPROVED, REJECTED, ESCALATED, TIMEOUT
  estimated_cost NUMERIC(10, 2),
  risk_score INT,
  metadata JSONB,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  expires_at TIMESTAMPTZ NOT NULL,
  approver_id TEXT,
  approved_at TIMESTAMPTZ,
  rejected_reason TEXT,
  escalation_level INT DEFAULT 0,
  INDEX idx_state_expires (state, expires_at),
  INDEX idx_agent_id (agent_id)
);
```

**WAL Configuration:**
- `synchronous_commit = on`: Flush WAL before returning
- `wal_level = replica`: Enable replication and PITR
- `archive_mode = on`: Archive WAL to S3 for recovery

### Policy Versioning

**Immutable Policy Snapshots:**
- Each policy change creates new version
- Old versions retained for audit trail
- Requests reference specific policy version (immutable evaluation)

```sql
CREATE TABLE oversight_policies (
  policy_id UUID PRIMARY KEY,
  version INT NOT NULL,
  resource TEXT NOT NULL,
  cel_expression TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  created_by TEXT NOT NULL,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  UNIQUE (resource, version)
);
```

### Performance Optimization

**CEL Policy Caching:**
- Compile CEL expressions once, cache in memory
- LRU cache: 10,000 entries (covers 99% of policies)
- Cache invalidation: On policy update, flush affected entries

**Redis Caching for Pending Requests:**
- Key pattern: `oversight:pending:{agent_id}`
- Value: List of request_ids
- TTL: 5 minutes (matches typical approval time)
- Use case: Fast lookup for "Does this agent have pending approvals?"

### Scalability

**Horizontal Scaling:**
- Request Manager API: 6 replicas (HPA on request rate)
- Policy Engine API: 4 replicas (CPU-bound CEL evaluation)
- Timeout Scheduler: Single leader election (Kubernetes leader election)

**Database Sharding:**
- Partition `oversight_requests` by `created_at` (weekly partitions)
- Partition `oversight_policies` by `resource` (hash partitioning)

### Resilience

**Notification Retry Policy:**
- Max retries: 3
- Backoff: Exponential (10s, 30s, 90s)
- Dead letter queue: After 3 failures, escalate to on-call

**Circuit Breakers:**
- Slack API: Open after 5 failures, half-open after 60s
- Email SMTP: Open after 3 failures, half-open after 30s

**Recovery on Crash:**
- On startup, `RecoveryService` queries PostgreSQL for in-flight requests
- Resume state machine from last persisted state
- Resend notifications if acknowledgment not received

### Security

**AuthZ Integration:**
- Every approval request validates delegation chain
- Approvers authenticated via OAuth (Slack, Google Workspace)
- Audit trail records approver identity for compliance

**Webhook HMAC Signatures:**
- All webhook callbacks include HMAC-SHA256 signature
- Secret key shared with webhook endpoint
- Prevents replay attacks and tampering

## Integration Points

### Cross-Product Triggers

**Metering → Oversight:**
- **Cost threshold alerts**: When agent exceeds 80% of budget, create approval request for budget increase
- **Quota exhaustion**: Require approval before granting additional quota

**Oversight → Runtime:**
- **Approval gates**: Runtime checks Oversight before executing high-risk sandboxes
- **Unblock execution**: On approval, Runtime receives webhook to proceed with execution

**Oversight → Messaging:**
- **Encrypted notifications**: Use Messaging product for E2E encrypted approval requests (future)

### External Integrations

**Slack API:**
- Endpoints: `/chat.postMessage`, `/oauth.v2.access`
- Interactive buttons with callback webhooks
- OAuth for approver authentication

**Email Service:**
- SMTP server for transactional emails
- HTML templates with approval links (JWT-signed)
- SPF/DKIM for deliverability

**Custom Webhooks:**
- HTTP POST with JSON payload
- HMAC-SHA256 signature in header
- Retry on 5xx errors

## Related Diagrams

- [C4 Context Diagram](./c4-context.md) - System-level context
- [C4 Container Diagram](./c4-container.md) - Container-level architecture
- [Data Flow Matrix](./data-flow-matrix.md) - Cross-product data flows

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-25 | Initial Oversight component diagram for Issue #62 |
