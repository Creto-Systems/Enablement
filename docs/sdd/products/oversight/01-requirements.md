---
status: draft
author: Oversight Lead Agent
created: 2024-12-25
updated: 2024-12-25
reviewers: []
oss_reference: humanlayer/humanlayer
parent_sdd: docs/sdd/products/oversight.md
issue: "#3 [OVS] Extract Oversight product requirements"
---

# SDD-OVS-01: Oversight Requirements Specification

## Purpose

This document defines the functional and non-functional requirements for **creto-oversight**, the Human-in-the-Loop (HITL) approval system for AI agent actions. It extracts proven patterns from HumanLayer and extends them with Creto's Sovereign platform primitives (NHI, Crypto-Agility, Authorization, Audit).

## Scope

**In Scope:**
- Policy-triggered approval requirements
- Multi-stage escalation chains
- Multi-channel notification delivery (Slack, email, webhook)
- Request lifecycle state management
- Delegation chain verification and context
- Cryptographically signed approvals
- Durable checkpoint/resume for process survival

**Out of Scope:**
- Policy authoring UI (separate service)
- Channel-specific applications (Slack bot implementation)
- Human identity provider integration (delegated to IdP)
- Agent action execution (delegated to agent runtime)

---

## 1. Introduction

### 1.1 Background

AI agents increasingly perform actions with significant business impact: financial transactions, data access, external communications, and system modifications. Organizations need the ability to require human approval before agents execute high-risk operations, with full context about **why** the agent is acting and **who** delegated authority to that agent.

**The Oversight Gap in OSS:**

| Dimension | HumanLayer | Creto Oversight |
|-----------|------------|-----------------|
| **Trigger mechanism** | Decorator on tool functions | Policy-driven via Authorization service |
| **Identity context** | Tool name + args | Agent NHI + full delegation chain |
| **Approval proof** | Database log entry | Cryptographically signed attestation (ML-DSA) |
| **Agent reasoning** | Not available | Memory context: why agent is acting |
| **Audit trail** | Application logs | Merkle-anchored immutable audit |
| **Policy granularity** | Per-function annotation | Unified policy language (amount, resource, risk) |

### 1.2 Design Principles

1. **Policy-First:** Approval requirements defined in Authorization policy, not code annotations
2. **Context-Rich:** Approvers see delegation chain, agent reasoning, and impact assessment
3. **Non-Repudiation:** Approvals are cryptographically signed, legally binding
4. **Durable:** Request state survives process crashes (checkpoint/resume)
5. **Omnichannel:** Route to Slack, email, Teams, webhook, ServiceNow based on policy
6. **Auditable:** All state transitions logged to immutable audit trail

### 1.3 Key Terms

- **Oversight Policy:** Authorization policy rule that returns `REQUIRES_OVERSIGHT` instead of `ALLOW/DENY`
- **Oversight Request:** State machine representing pending human approval
- **Approver:** Human identity authorized to approve/deny specific request types
- **Escalation Chain:** Multi-tier approval flow with timeout-based escalation
- **Quorum:** Approval requirement (any, all, N-of-M approvers)
- **Checkpoint:** Durable state snapshot for crash recovery

---

## 2. Functional Requirements

### 2.1 Policy Definition & Evaluation

**FR-OVS-001: Policy-Triggered Oversight**

The system SHALL integrate with the Authorization service to trigger oversight based on policy evaluation, not code annotations.

**Acceptance Criteria:**
- Authorization service evaluates policy for agent action
- Policy can return `Decision::RequiresOversight { policy_id, requirement }`
- Requirement specifies approvers, timeout, escalation chain, and quorum
- Same policy language used for authorization and oversight (unified semantics)

**User Story:**
> As a compliance officer, I want to define "any transaction over $10,000 requires CFO approval" in one policy, so that I don't need to annotate every payment function.

**Example Policy:**
```cedar
permit(
  principal,
  action == Action::"transfer_funds",
  resource
)
when {
  resource.amount <= 10000
};

requires_oversight(
  principal,
  action == Action::"transfer_funds",
  resource
)
when {
  resource.amount > 10000
}
with {
  approvers: ["cfo@company.com"],
  timeout: duration("2h"),
  escalation: EscalationChain::CFO_to_CEO,
  quorum: ApprovalQuorum::Any
};
```

**FR-OVS-002: Approver Specification**

The system SHALL support flexible approver specification by:
- Individual human identity (email, IdP subject)
- Role-based (any member of "finance_approvers" group)
- Dynamic (derived from resource ownership, delegation chain)

**Acceptance Criteria:**
- Policy can specify approvers by identifier, role, or expression
- System resolves to concrete list of eligible approvers
- Notification routed to all eligible approvers
- First to respond satisfies `ApprovalQuorum::Any`

**User Story:**
> As a policy author, I want to specify "any member of the finance team can approve" rather than listing individual emails, so policies don't break when team membership changes.

**FR-OVS-003: Timeout & Escalation Configuration**

The system SHALL support multi-tier escalation chains with configurable timeouts.

**Acceptance Criteria:**
- Policy specifies escalation chain with ≥1 tiers
- Each tier has: approvers, timeout duration, notification channels
- On tier timeout without approval, request escalates to next tier
- Final tier timeout triggers configurable action: `AutoDeny`, `AutoApprove`, `BlockIndefinitely`

**User Story:**
> As a risk manager, I want requests to escalate from team lead (1h timeout) → department head (2h) → VP (4h) → auto-deny, so high-risk actions don't stall indefinitely.

**Example Escalation Chain:**
```rust
EscalationChain {
    tiers: vec![
        EscalationTier {
            tier_id: "team_lead",
            approvers: vec!["lead@company.com"],
            timeout: Duration::from_secs(3600),  // 1 hour
            channels: vec!["slack::#finance-approvals"],
        },
        EscalationTier {
            tier_id: "dept_head",
            approvers: vec!["head@company.com"],
            timeout: Duration::from_secs(7200),  // 2 hours
            channels: vec!["slack::#exec-alerts", "email"],
        },
        EscalationTier {
            tier_id: "vp_finance",
            approvers: vec!["vp@company.com"],
            timeout: Duration::from_secs(14400), // 4 hours
            channels: vec!["email", "sms"],
        },
    ],
    final_action: TimeoutAction::AutoDeny,
}
```

**FR-OVS-004: Quorum Requirements**

The system SHALL support multiple quorum models for approvals.

**Acceptance Criteria:**
- `ApprovalQuorum::Any` — first approver to respond resolves request
- `ApprovalQuorum::All` — all approvers must approve
- `ApprovalQuorum::Threshold { required: N }` — N of M approvers must approve
- Single denial immediately transitions to `Denied` state (no override)

**User Story:**
> As a security architect, I want high-risk actions to require 2-of-3 security team approvals, so no single person can unilaterally authorize critical changes.

### 2.2 Request Lifecycle Management

**FR-OVS-005: Request State Machine**

The system SHALL implement a deterministic state machine for oversight requests.

**States:**
- `Pending` — awaiting human response
- `Approved` — quorum of approvals received
- `Denied` — any approver denied
- `Escalated` — tier timeout, moved to next tier
- `TimedOut` — final tier timeout, action taken per policy
- `Cancelled` — agent or system cancelled request

**State Transitions:**

| From | To | Trigger | Side Effects |
|------|-----|---------|--------------|
| Pending | Approved | Quorum met | Resume agent action, audit log, notify agent |
| Pending | Denied | Any denial | Abort agent action, audit log, notify agent |
| Pending | Escalated | Tier timeout | Notify next tier approvers, reset timer |
| Escalated | Pending | — | New tier, awaiting responses |
| Pending | TimedOut | Final timeout | Execute `TimeoutAction`, audit log |
| Any | Cancelled | Agent/system cancel | Abort action, audit log |

**Acceptance Criteria:**
- State transitions are atomic (transactional)
- Each transition checkpointed for durability
- Concurrent responses handled correctly (quorum counted atomically)
- Invalid transitions rejected with error

**User Story:**
> As an agent developer, I need deterministic behavior: if request is approved, action proceeds; if denied, action aborts; no ambiguous states.

**FR-OVS-006: Request Context Enrichment**

The system SHALL provide rich context to approvers for informed decision-making.

**Request Context Fields:**
- **Agent identity:** NHI identifier with human-readable name
- **Delegation chain:** Full lineage to root human principal
- **Action description:** Human-readable "what agent wants to do"
- **Resource details:** Affected resources with metadata
- **Reasoning:** Why agent is taking this action (from Memory)
- **Risk factors:** Categorized risk indicators (financial, data, compliance)
- **Impact assessment:** Reversibility, estimated cost, affected systems

**Acceptance Criteria:**
- Context fetched from Memory service (agent reasoning history)
- Risk factors computed from resource metadata and action type
- Impact assessment includes financial estimate if available
- Approver UI renders context in scannable format

**User Story:**
> As an approver, I need to see not just "Agent wants to transfer $50K" but **why** it's doing that, **who** delegated authority, and what the **risks** are, so I can make an informed decision in seconds.

**Example Context:**
```json
{
  "request_id": "req_abc123",
  "agent_nhi": "agent:payment-bot-v3@company.creto",
  "delegation_chain": [
    "agent:payment-bot-v3@company.creto",
    "agent:accounts-payable@company.creto",
    "human:alice@company.com"
  ],
  "action_description": "Transfer $50,000 to vendor invoice #INV-2024-1234",
  "resource": {
    "type": "bank_transfer",
    "amount": 50000.00,
    "currency": "USD",
    "recipient": "vendor@example.com",
    "invoice_id": "INV-2024-1234"
  },
  "reasoning": "Invoice approved in AP system. Vendor contract requires payment within 30 days. Due date: 2024-12-31.",
  "risk_factors": [
    {
      "category": "Financial",
      "description": "Large transaction exceeding normal $10K threshold",
      "severity": "High"
    },
    {
      "category": "Compliance",
      "description": "Payment to international vendor requires SOX dual control",
      "severity": "Medium"
    }
  ],
  "impact": {
    "affected_resources": ["bank_account:operating"],
    "reversible": false,
    "estimated_cost": 50000.00
  }
}
```

**FR-OVS-007: Delegation Chain Verification**

The system SHALL verify and display the full delegation chain for every oversight request.

**Acceptance Criteria:**
- Delegation chain extracted from agent NHI credentials
- Each link in chain verified against NHI registry
- Chain displayed to approver with human-readable names
- Invalid or revoked delegation causes request rejection

**User Story:**
> As an approver, I need to see that this payment bot was spawned by the AP agent, which was delegated by Alice (CFO), so I can verify legitimate authority chain.

### 2.3 Multi-Channel Notifications

**FR-OVS-008: Channel Abstraction**

The system SHALL support pluggable notification channels via trait abstraction.

**Channels (v1):**
- **Slack:** Blocks-based message with approve/deny buttons
- **Email:** HTML email with secure approval link
- **Webhook:** POST to external system (ServiceNow, Jira)

**Acceptance Criteria:**
- Channel selected based on policy configuration
- Multiple channels per request (parallel delivery)
- Channel failures don't block request creation
- Retry logic for transient failures

**User Story:**
> As an operations lead, I want oversight requests sent to both Slack (for fast response) and email (for record-keeping), so I have multiple ways to respond.

**FR-OVS-009: Slack Integration**

The system SHALL deliver interactive approval requests to Slack channels or DMs.

**Acceptance Criteria:**
- Message uses Blocks API for rich formatting
- Context rendered in scannable format (delegation chain, reasoning, risks)
- Approve/Deny buttons trigger instant response
- Response links back to request ID
- Approval signature embedded in response payload

**User Story:**
> As a busy executive, I want to approve requests from Slack mobile app with one tap, with all context visible inline.

**Example Slack Message:**
```
🔔 **Approval Required**

**Agent:** payment-bot-v3
**Delegated by:** Alice (CFO)
**Action:** Transfer $50,000 to vendor invoice #INV-2024-1234

**Reasoning:** Invoice approved in AP system. Due date: 2024-12-31.

**Risk:** High financial impact, SOX dual control required.

[Approve] [Deny]
```

**FR-OVS-010: Email Integration**

The system SHALL deliver approval requests via email with secure response links.

**Acceptance Criteria:**
- HTML email with rich formatting
- One-time-use approval link with cryptographic token
- Link expiration matches request timeout
- Response recorded with approver email verification

**User Story:**
> As a compliance officer, I want email notifications for high-risk approvals that I can forward to legal for review before responding.

**FR-OVS-011: Webhook Integration**

The system SHALL support webhook delivery to external systems.

**Acceptance Criteria:**
- POST request to configured URL with request context
- Webhook endpoint can respond with approval/denial
- Supports HMAC signature verification
- Retry with exponential backoff on failure

**User Story:**
> As a ServiceNow admin, I want oversight requests to create approval tickets in our ITSM system, with automatic callback when ticket is resolved.

**FR-OVS-012: Notification Delivery Guarantees**

The system SHALL ensure notifications are delivered at least once per tier.

**Acceptance Criteria:**
- Delivery tracked per channel with status
- Retry failed deliveries (transient errors)
- Escalate if all channels fail after 3 retries
- Audit log includes delivery attempts and outcomes

**User Story:**
> As a system operator, I need guarantee that approvers are notified, even if Slack is down, by falling back to email.

### 2.4 Approval Response Handling

**FR-OVS-013: Response Submission**

The system SHALL accept approval responses from multiple channels.

**Acceptance Criteria:**
- Response includes: request ID, approver identity, decision, optional reason
- Approver identity verified against eligible approvers list
- Duplicate responses from same approver idempotently handled
- Response timestamp recorded from consensus-ordered time

**User Story:**
> As an approver, I want to respond from any channel (Slack, email, webhook) and have it count toward the approval quorum.

**FR-OVS-014: Cryptographic Approval Signatures**

The system SHALL require cryptographic signatures on all approval responses.

**Acceptance Criteria:**
- Approver signs response with private key (ML-DSA or Ed25519)
- Signature verified before state transition
- Signature includes: request ID, decision, timestamp, approver identity
- Signature stored in audit trail for non-repudiation

**User Story:**
> As a legal counsel, I need cryptographic proof that CFO approved this $1M transfer, admissible as evidence in dispute resolution.

**Example Signed Approval:**
```rust
ApprovalResponse {
    approver: HumanIdentity {
        subject: "cfo@company.com",
        name: "Alice CFO",
        public_key: [...]
    },
    decision: ApprovalDecision::Approve,
    reason: Some("Invoice verified, payment authorized per contract"),
    timestamp: 1703520000,
    signature: Signature {
        algorithm: "ML-DSA-65",
        value: [...]  // Signs: request_id || decision || timestamp
    },
    channel: ChannelId::Slack("C123456"),
}
```

**FR-OVS-015: Request for More Information**

The system SHALL support approvers requesting clarification before deciding.

**Acceptance Criteria:**
- Approver can submit `RequestMoreInfo { question }` decision
- Question sent to agent (if interactive) or escalated to next tier
- Request remains in `Pending` state
- Timeout clock continues (not paused)

**User Story:**
> As an approver, I want to ask the agent "Why this vendor?" and see its reasoning before approving, without denying the request.

### 2.5 Durability & Fault Tolerance

**FR-OVS-016: Checkpoint & Resume**

The system SHALL checkpoint request state to survive process crashes.

**Acceptance Criteria:**
- State checkpointed after every transition
- Checkpoint includes: state, responses, notifications sent, escalation history
- On startup, resume all `Pending` and `Escalated` requests
- Idempotent resume (no duplicate notifications)

**User Story:**
> As a platform engineer, I need oversight requests to survive pod restarts, so approvals in-flight aren't lost during deployments.

**FR-OVS-017: Timeout Management**

The system SHALL enforce timeout deadlines with millisecond precision.

**Acceptance Criteria:**
- Timeout tracked per tier with monotonic clock
- Timeout triggers escalation or final action
- Timeout persists across restarts (stored in checkpoint)
- Timeout expiration logged to audit

**User Story:**
> As a risk manager, I need strict enforcement of "2 hour approval window" so requests don't linger indefinitely.

---

## 3. Non-Functional Requirements

### 3.1 Performance

**NFR-OVS-001: Request Creation Latency**

The system SHALL create oversight requests in <10ms (p99).

**Measurement:** Time from Authorization returning `RequiresOversight` to request persisted in checkpoint store.

**Rationale:** Agent blocking on approval shouldn't add significant latency to authorization path.

**NFR-OVS-002: State Transition Latency**

The system SHALL process state transitions in <1ms (p99).

**Measurement:** Time from response submission to state update persisted.

**Rationale:** Multiple approvers responding concurrently shouldn't create backlog.

**NFR-OVS-003: Notification Delivery Latency**

The system SHALL deliver notifications in <5 seconds (p95).

**Measurement:** Time from request creation to notification visible in channel (Slack, email inbox).

**Rationale:** Fast human response requires fast notification delivery.

**NFR-OVS-004: Checkpoint Write Latency**

The system SHALL write checkpoints in <5ms (p99).

**Measurement:** Time to persist checkpoint to durable storage.

**Rationale:** Checkpointing on hot path (state transitions) cannot add significant latency.

### 3.2 Scalability

**NFR-OVS-005: Concurrent Requests**

The system SHALL handle ≥10,000 concurrent pending requests per instance.

**Measurement:** Number of `Pending` or `Escalated` requests in memory without degradation.

**Rationale:** Large organizations with many agents may have thousands of approvals in-flight.

**NFR-OVS-006: Throughput**

The system SHALL process ≥1,000 requests/second (creation + state transitions).

**Measurement:** Combined rate of `create_request()` and `submit_response()` calls.

**Rationale:** Approval-heavy workflows (e.g., financial services) generate high request volume.

### 3.3 Reliability

**NFR-OVS-007: Uptime**

The system SHALL achieve 99.9% uptime (3 nines).

**Measurement:** Percentage of time system accepts requests and processes responses.

**Rationale:** Downtime blocks agents from performing critical actions.

**NFR-OVS-008: Data Durability**

The system SHALL guarantee zero data loss for checkpointed requests.

**Measurement:** All `Pending` requests recoverable after process crash.

**Rationale:** Lost approval requests mean agents stuck indefinitely.

### 3.4 Security

**NFR-OVS-009: Authorization Integration**

The system SHALL enforce authorization for all operations (create, respond, query).

**Measurement:** All API calls check caller's authorization via `creto-authz`.

**Rationale:** Only authorized approvers should submit responses.

**NFR-OVS-010: Signature Verification**

The system SHALL cryptographically verify all approval signatures before state transition.

**Measurement:** 100% of responses verified with public key cryptography.

**Rationale:** Prevent forged approvals.

**NFR-OVS-011: Audit Completeness**

The system SHALL log all state transitions to immutable audit trail.

**Measurement:** 100% of transitions have corresponding audit record.

**Rationale:** Compliance requires complete approval history.

### 3.5 Observability

**NFR-OVS-012: Metrics**

The system SHALL export Prometheus metrics for:
- Request creation rate
- State transition counts by type
- Timeout/escalation rates
- Notification delivery success/failure rates
- Response latency (p50, p95, p99)

**NFR-OVS-013: Tracing**

The system SHALL integrate with OpenTelemetry for distributed tracing.

**Measurement:** Every request has trace ID linking Authorization → Oversight → Notification → Response.

**Rationale:** Debug approval delays and failures.

---

## 4. User Stories

### 4.1 Core Approval Flow

**US-OVS-001: Simple Approval**
> **As** a finance agent,
> **I want** to request CFO approval for payments over $10K,
> **So that** high-value transactions have human oversight.

**Acceptance:**
- Agent calls payment API
- Authorization returns `RequiresOversight`
- CFO receives Slack notification
- CFO clicks "Approve"
- Payment proceeds

**US-OVS-002: Escalation Chain**
> **As** a compliance officer,
> **I want** requests to escalate from team lead → department head → VP,
> **So that** unresponsive approvers don't block critical actions.

**Acceptance:**
- Team lead doesn't respond in 1 hour
- Request escalates to department head
- Department head approves in 30 minutes
- Action proceeds

**US-OVS-003: Multi-Approver Quorum**
> **As** a security architect,
> **I want** 2-of-3 security team approvals for privileged access,
> **So that** no single person can authorize sensitive actions.

**Acceptance:**
- Agent requests database access
- Notifications sent to 3 security team members
- First 2 to approve meet quorum
- Access granted

### 4.2 Context & Reasoning

**US-OVS-004: Delegation Chain Visibility**
> **As** an approver,
> **I want** to see which human delegated authority to this agent,
> **So that** I can verify legitimate authorization.

**Acceptance:**
- Approval request shows: Agent A → spawned by Agent B → delegated by Alice
- Approver verifies Alice is authorized to delegate
- Approver approves based on trust in Alice

**US-OVS-005: Agent Reasoning Context**
> **As** an approver,
> **I want** to see why the agent is taking this action,
> **So that** I can make informed decisions without asking follow-up questions.

**Acceptance:**
- Approval request includes Memory context: "Invoice approved in AP system, due date 2024-12-31"
- Approver sees reasoning inline
- Approver approves based on context

### 4.3 Multi-Channel Delivery

**US-OVS-006: Slack + Email Delivery**
> **As** an executive,
> **I want** approval requests in both Slack and email,
> **So that** I can respond from any device.

**Acceptance:**
- Request sent to Slack channel and email
- Executive responds via Slack mobile app
- Email marked as resolved

**US-OVS-007: ServiceNow Integration**
> **As** a ITSM manager,
> **I want** approval requests to create ServiceNow tickets,
> **So that** approvals follow our change management process.

**Acceptance:**
- Oversight request triggers webhook to ServiceNow
- Ticket created with approval context
- Ticket resolution sends approval to Creto
- Agent action proceeds

### 4.4 Audit & Compliance

**US-OVS-008: Cryptographic Approval Proof**
> **As** legal counsel,
> **I want** cryptographic proof of who approved what,
> **So that** we have admissible evidence in disputes.

**Acceptance:**
- Approval response includes ML-DSA signature
- Signature verified and stored in audit trail
- Third party can verify signature with approver's public key

**US-OVS-009: Complete Audit Trail**
> **As** a compliance auditor,
> **I want** complete history of all approval requests and responses,
> **So that** I can demonstrate regulatory compliance.

**Acceptance:**
- Audit log includes: request created, notifications sent, responses received, state transitions
- Log is immutable (Merkle-anchored)
- Log queryable by request ID, agent, approver, date range

---

## 5. Integration Requirements

### 5.1 Authorization Service

**INT-OVS-001:** Authorization evaluates policy and returns `Decision::RequiresOversight { policy_id, requirement }`.

**INT-OVS-002:** Oversight service resolves `requirement` into concrete `OversightRequest` with approvers, timeout, escalation.

**INT-OVS-003:** On approval, Oversight calls Authorization with `override_token` to bypass policy and allow action.

### 5.2 Memory Service

**INT-OVS-004:** Oversight queries Memory for agent reasoning context related to pending action.

**INT-OVS-005:** Memory returns relevant snippets (last N interactions, goal context).

### 5.3 Audit Service

**INT-OVS-006:** All state transitions logged to Audit with:
- Request ID
- Agent NHI
- Approver identity (if response)
- Previous state → new state
- Timestamp
- Signature (if approval/denial)

### 5.4 NHI Registry

**INT-OVS-007:** Oversight verifies agent NHI against registry.

**INT-OVS-008:** Oversight validates delegation chain integrity.

**INT-OVS-009:** Oversight resolves approver identities (email → NHI).

### 5.5 Notification Channels

**INT-OVS-010:** Slack: POST to Slack API with Blocks payload.

**INT-OVS-011:** Email: SMTP send with HTML template.

**INT-OVS-012:** Webhook: POST to configured URL with HMAC signature.

---

## 6. Data Requirements

### 6.1 Request Storage

**DATA-OVS-001:** Store oversight requests with:
- Request ID (UUID)
- Agent NHI
- Delegation chain
- Pending action
- Policy ID
- State (Pending, Approved, Denied, etc.)
- Responses (array)
- Created/updated timestamps

**DATA-OVS-002:** Index by:
- Request ID (primary key)
- Agent NHI (query requests by agent)
- State (query pending requests)
- Created timestamp (query by time range)

### 6.2 Checkpoint Storage

**DATA-OVS-003:** Checkpoint format includes:
- Request ID
- State
- Responses
- Notifications sent
- Escalation history
- Checkpoint timestamp
- Version (optimistic concurrency control)

**DATA-OVS-004:** Checkpoint storage requirements:
- Durable (survive crashes)
- Low-latency writes (<5ms)
- Queryable (load pending on startup)

### 6.3 Policy Storage

**DATA-OVS-005:** Store oversight policies with:
- Policy ID
- Trigger conditions
- Approver specification
- Escalation chain
- Timeout configuration

---

## 7. Constraints

### 7.1 Technical Constraints

**CONST-OVS-001:** Must integrate with existing Authorization service (168ns policy evaluation path).

**CONST-OVS-002:** Must use Crypto-Agility layer for signatures (ML-DSA or Ed25519).

**CONST-OVS-003:** Must checkpoint to PostgreSQL or Redis (existing infrastructure).

**CONST-OVS-004:** Slack integration limited to Blocks API (no custom UI).

### 7.2 Business Constraints

**CONST-OVS-005:** Must support disconnected approvers (email for on-call, traveling execs).

**CONST-OVS-006:** Must handle timezone differences (approver in different region).

### 7.3 Compliance Constraints

**CONST-OVS-007:** EU AI Act Article 14 requires human oversight for high-risk AI systems.

**CONST-OVS-008:** SOX 404 requires dual control (2-person approval) for financial transactions.

**CONST-OVS-009:** HIPAA requires minimum necessary access (oversight for PHI access).

---

## 8. Success Metrics

### 8.1 Functional Success

**METRIC-OVS-001:** 100% of high-risk actions require oversight (no policy bypass).

**METRIC-OVS-002:** 95% of approval requests resolved within first tier (no escalation).

**METRIC-OVS-003:** <1% of requests timeout (approvers responsive).

### 8.2 Performance Success

**METRIC-OVS-004:** p99 request creation latency <10ms.

**METRIC-OVS-005:** p99 notification delivery latency <5s.

**METRIC-OVS-006:** p99 state transition latency <1ms.

### 8.3 Reliability Success

**METRIC-OVS-007:** 99.9% uptime.

**METRIC-OVS-008:** Zero data loss (all requests recoverable).

**METRIC-OVS-009:** <0.1% notification delivery failures.

---

## 9. Open Questions

**Q-OVS-001:** What is the default timeout for oversight requests?
- **Proposed:** 2 hours for first tier, configurable per policy.

**Q-OVS-002:** Should approvers be able to delegate approval authority?
- **Proposed:** v1 = no delegation, v2 = time-bound delegation with audit.

**Q-OVS-003:** How do we handle approver unavailability (vacation, PTO)?
- **Proposed:** Escalation chain includes backup approvers per tier.

**Q-OVS-004:** Should "approve with modifications" be supported?
- **Proposed:** v1 = binary approve/deny, v2 = conditional approval (e.g., "approve if amount reduced to $40K").

**Q-OVS-005:** What is the retention policy for completed requests?
- **Proposed:** 7 years for financial actions (SOX), 3 years for others, configurable.

**Q-OVS-006:** Should agents be able to cancel pending requests?
- **Proposed:** Yes, with audit trail. Use case: agent realizes request was erroneous.

**Q-OVS-007:** How do we prevent approval fatigue (too many requests)?
- **Proposed:** Risk scoring, batch approvals for low-risk, analytics dashboard for policy tuning.

---

## 10. Out of Scope (Future Work)

**OOS-OVS-001:** Approval delegation (approver authorizes deputy to approve on their behalf).

**OOS-OVS-002:** Batch approvals (approve multiple similar requests at once).

**OOS-OVS-003:** Conditional approvals ("approve if amount reduced to X").

**OOS-OVS-004:** Mobile-native apps (dedicated iOS/Android approval apps).

**OOS-OVS-005:** Risk scoring ML model (predict which requests need oversight).

**OOS-OVS-006:** Approval analytics dashboard (policy effectiveness, response times).

---

## 11. References

### 11.1 OSS Reference

- **HumanLayer:** https://github.com/humanlayer/humanlayer
- **HumanLayer Docs:** https://humanlayer.vercel.app/
- **Agent Control Plane (ACP):** https://github.com/humanlayer/agent-control-plane

### 11.2 Internal References

- **Parent SDD:** `/docs/sdd/products/oversight.md`
- **Requirements Analysis:** `/docs/sdd/01-requirements.md` (Section 3: Oversight by Creto)
- **Architecture:** `/docs/sdd/02-architecture.md` (TBD)

### 11.3 Compliance References

- **EU AI Act Article 14:** Human oversight requirements
- **SOX 404:** Internal controls over financial reporting
- **HIPAA Minimum Necessary:** 45 CFR 164.502(b)

---

## 12. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Oversight Lead Agent | Initial requirements extraction from HumanLayer patterns |

---

## Appendix A: HumanLayer Pattern Extraction

### A.1 Core Patterns Extracted

| HumanLayer Feature | Creto Adaptation | Rationale |
|-------------------|------------------|-----------|
| `@require_approval()` decorator | Policy-triggered via Authorization | Unified policy language, not code annotation |
| `human_as_tool()` | `RequestMoreInfo` decision | Approver can ask agent for clarification |
| Channel abstraction | `NotificationChannel` trait | Support Slack, email, webhook, extensible |
| Escalation chains | `EscalationChain` with tiers | Multi-tier approval with timeout |
| Checkpoint/resume | `OversightCheckpoint` | Durability across process restarts |

### A.2 Creto Extensions Beyond HumanLayer

| Extension | Value |
|-----------|-------|
| **NHI Integration** | Agent identity + delegation chain in approval context |
| **Memory Integration** | Agent reasoning context from Memory service |
| **Cryptographic Signatures** | ML-DSA signed approvals for non-repudiation |
| **Immutable Audit** | Merkle-anchored audit trail (legally admissible) |
| **Policy-Driven** | Authorization policy returns `RequiresOversight` (not code annotation) |

---

## Appendix B: Example Policy (Cedar)

```cedar
// Allow small transfers without oversight
permit(
  principal,
  action == Action::"transfer_funds",
  resource
)
when {
  resource.amount <= 10000
};

// Require oversight for large transfers
requires_oversight(
  principal,
  action == Action::"transfer_funds",
  resource
)
when {
  resource.amount > 10000 && resource.amount <= 100000
}
with {
  approvers: ["cfo@company.com"],
  timeout: duration("2h"),
  quorum: ApprovalQuorum::Any,
  channels: ["slack::#finance-approvals", "email"]
};

// Require escalation chain for very large transfers
requires_oversight(
  principal,
  action == Action::"transfer_funds",
  resource
)
when {
  resource.amount > 100000
}
with {
  escalation: EscalationChain {
    tiers: [
      {
        approvers: ["cfo@company.com"],
        timeout: duration("1h"),
        channels: ["slack::#exec-alerts"]
      },
      {
        approvers: ["ceo@company.com"],
        timeout: duration("2h"),
        channels: ["email", "sms"]
      }
    ],
    final_action: TimeoutAction::AutoDeny
  },
  quorum: ApprovalQuorum::All
};
```

---

## Appendix C: State Machine Diagram

```
                         ┌─────────────────────┐
                         │      PENDING        │
                         │  (awaiting human)   │
                         └──────────┬──────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              │                     │                     │
              ▼                     ▼                     ▼
    ┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
    │    APPROVED     │   │     DENIED      │   │    ESCALATED    │
    │  (quorum met)   │   │  (any denial)   │   │  (tier timeout) │
    │                 │   │                 │   │                 │
    │ → Resume action │   │ → Abort action  │   │ → Notify next   │
    └─────────────────┘   └─────────────────┘   └────────┬────────┘
                                                          │
                                                          ▼
                                                 ┌─────────────────┐
                                                 │   PENDING       │
                                                 │  (new tier)     │
                                                 └────────┬────────┘
                                                          │
                                    ┌─────────────────────┴──────────┐
                                    ▼                                ▼
                           ┌─────────────────┐            ┌──────────────────┐
                           │   TIMED_OUT     │            │    CANCELLED     │
                           │ (final timeout) │            │ (agent/system)   │
                           │                 │            │                  │
                           │ → Execute       │            │ → Abort action   │
                           │   TimeoutAction │            │                  │
                           └─────────────────┘            └──────────────────┘
```

---

**END OF DOCUMENT**
