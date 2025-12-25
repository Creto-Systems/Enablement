---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
oss_reference: humanlayer/humanlayer
---

# Product SDD: creto-oversight

## Purpose

Human-in-the-loop (HITL) approval workflows for sensitive AI agent actions. Enables organizations to require human approval before agents execute high-risk operations, with configurable policies, multi-channel notifications, and escalation chains.

## Scope

**In Scope:**
- OversightPolicy definition and evaluation
- OversightRequest state machine
- Multi-channel notifications (Slack, email, webhook)
- Escalation chains with timeouts
- Checkpoint/resume for durability
- NHI/delegation chain context

**Out of Scope:**
- Policy authoring UI (separate service)
- Channel-specific apps (Slack bot, etc.)
- Human identity management (delegated to IdP)

---

## 1. OSS Reference: HumanLayer

**Repository:** https://github.com/humanlayer/humanlayer

**Key Patterns to Extract:**
- `@require_approval()` decorator pattern
- `human_as_tool()` for agent-initiated contact
- Channel abstraction
- Escalation chain configuration
- Checkpoint/resume durability

**Differences from HumanLayer:**
- Policy evaluation via Authorization service (not inline decorator)
- NHI integration (agent identity in approval context)
- Memory integration (agent reasoning context)
- Crypto-signed approvals (non-repudiation)
- Consensus-ordered state transitions

---

## 2. Core Traits

### 2.1 OversightPolicy

```rust
/// Evaluated by Authorization service to determine if HITL required
pub trait OversightPolicy: Send + Sync {
    /// Evaluate whether action requires approval
    fn requires_approval(&self, ctx: &ActionContext) -> ApprovalRequirement;
}

pub struct ActionContext {
    pub agent_nhi: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,
    pub action: Action,
    pub resource: Resource,
    pub memory_context: Option<MemoryContext>,  // Why agent is acting
    pub risk_score: Option<f64>,                 // ML-computed risk
}

pub enum ApprovalRequirement {
    /// No approval needed
    None,

    /// Approval required from specified approvers
    Required {
        approvers: Vec<ApproverId>,
        timeout: Duration,
        escalation: Option<EscalationChain>,
        quorum: ApprovalQuorum,
    },

    /// Conditional approval based on runtime evaluation
    Conditional {
        condition: Box<dyn PolicyCondition>,
        if_true: Box<ApprovalRequirement>,
        if_false: Box<ApprovalRequirement>,
    },
}

pub enum ApprovalQuorum {
    /// Any single approver
    Any,
    /// All approvers must approve
    All,
    /// N of M approvers
    Threshold { required: usize },
}
```

### 2.2 OversightRequest

```rust
/// State machine for pending approvals
pub struct OversightRequest {
    pub id: RequestId,

    // Identity context
    pub agent_nhi: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,

    // Action details
    pub pending_action: Action,
    pub resource: Resource,
    pub context: RequestContext,

    // Policy that triggered this
    pub policy_id: PolicyId,
    pub policy_trigger_reason: String,

    // Approval requirements
    pub required_approvers: Vec<ApproverId>,
    pub quorum: ApprovalQuorum,
    pub escalation: Option<EscalationChain>,

    // State
    pub state: OversightState,
    pub responses: Vec<ApprovalResponse>,

    // Timing
    pub created_at: Timestamp,
    pub expires_at: Timestamp,
    pub state_changed_at: Timestamp,
}

pub enum OversightState {
    /// Awaiting human response
    Pending,

    /// Approved by human(s)
    Approved {
        by: Vec<ApprovalResponse>,
        final_at: Timestamp,
    },

    /// Denied by human
    Denied {
        by: ApprovalResponse,
        reason: String,
    },

    /// Escalated to next tier
    Escalated {
        to: EscalationTier,
        at: Timestamp,
        previous_tier: EscalationTier,
    },

    /// Timed out without response
    TimedOut {
        action: TimeoutAction,
        at: Timestamp,
    },

    /// Cancelled by agent or system
    Cancelled {
        reason: String,
        at: Timestamp,
    },
}

pub struct ApprovalResponse {
    pub approver: HumanIdentity,
    pub decision: ApprovalDecision,
    pub reason: Option<String>,
    pub timestamp: Timestamp,
    pub signature: Signature,  // Non-repudiation
    pub channel: ChannelId,
}

pub enum ApprovalDecision {
    Approve,
    Deny,
    RequestMoreInfo { question: String },
}
```

### 2.3 NotificationChannel

```rust
/// Channel for human notifications
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Channel identifier
    fn channel_id(&self) -> &ChannelId;

    /// Send approval request notification
    async fn send(&self, request: &OversightRequest) -> Result<NotificationId, ChannelError>;

    /// Poll for response (for channels that don't push)
    async fn poll_response(&self, notification_id: &NotificationId) -> Option<ApprovalResponse>;

    /// Send reminder
    async fn send_reminder(&self, request: &OversightRequest) -> Result<(), ChannelError>;

    /// Send resolution notification
    async fn send_resolution(&self, request: &OversightRequest) -> Result<(), ChannelError>;
}

// Implementations
pub struct SlackChannel { ... }
pub struct EmailChannel { ... }
pub struct WebhookChannel { ... }
```

### 2.4 EscalationChain

```rust
pub struct EscalationChain {
    pub tiers: Vec<EscalationTier>,
    pub final_action: TimeoutAction,
}

pub struct EscalationTier {
    pub tier_id: TierId,
    pub approvers: Vec<ApproverId>,
    pub timeout: Duration,
    pub channels: Vec<ChannelId>,
}

pub enum TimeoutAction {
    /// Automatically deny after final timeout
    AutoDeny,
    /// Automatically approve after final timeout
    AutoApprove,
    /// Keep pending indefinitely (agent blocks)
    BlockIndefinitely,
    /// Execute fallback action
    Fallback { action: Action },
}
```

---

## 3. State Machine

```
                    ┌─────────────────────┐
                    │      PENDING        │
                    └──────────┬──────────┘
                               │
         ┌─────────────────────┼─────────────────────┐
         │                     │                     │
         ▼                     ▼                     ▼
┌─────────────────┐   ┌─────────────────┐   ┌─────────────────┐
│    APPROVED     │   │     DENIED      │   │    ESCALATED    │
│  (quorum met)   │   │  (any denial)   │   │  (tier timeout) │
└─────────────────┘   └─────────────────┘   └────────┬────────┘
                                                     │
                                                     ▼
                                            Back to PENDING
                                            (new tier)
                                                     │
                                                     ▼
                                            ┌─────────────────┐
                                            │   TIMED_OUT     │
                                            │ (final timeout) │
                                            └─────────────────┘
```

### State Transitions

| From | To | Trigger |
|------|-----|---------|
| Pending | Approved | Quorum of approvals received |
| Pending | Denied | Any denial received |
| Pending | Escalated | Tier timeout without quorum |
| Escalated | Pending | New tier, reset approvers |
| Pending | TimedOut | Final tier timeout |
| Any | Cancelled | Agent/system cancellation |

---

## 4. Data Models

### 4.1 Request Context

```rust
pub struct RequestContext {
    /// Human-readable description of what agent wants to do
    pub action_description: String,

    /// Why agent is taking this action (from Memory)
    pub reasoning: Option<String>,

    /// Relevant memory snippets for context
    pub memory_context: Vec<MemorySnippet>,

    /// Risk indicators
    pub risk_factors: Vec<RiskFactor>,

    /// Impact assessment
    pub impact: ImpactAssessment,
}

pub struct RiskFactor {
    pub category: RiskCategory,
    pub description: String,
    pub severity: Severity,
}

pub enum RiskCategory {
    Financial,
    DataAccess,
    ExternalCommunication,
    SystemModification,
    Compliance,
    Custom(String),
}

pub struct ImpactAssessment {
    pub affected_resources: Vec<Resource>,
    pub reversible: bool,
    pub estimated_cost: Option<Money>,
}
```

### 4.2 Checkpoint (Durability)

```rust
pub struct OversightCheckpoint {
    pub request_id: RequestId,
    pub state: OversightState,
    pub responses: Vec<ApprovalResponse>,
    pub notifications_sent: Vec<NotificationRecord>,
    pub escalation_history: Vec<EscalationEvent>,
    pub checkpoint_at: Timestamp,
    pub version: u64,  // Optimistic concurrency
}

// Enables resume after crash
impl OversightService {
    pub async fn resume_pending(&self) -> Result<Vec<OversightRequest>, Error> {
        let checkpoints = self.checkpoint_store.load_pending().await?;
        // Reconstruct state machines from checkpoints
    }
}
```

---

## 5. Architecture

### 5.1 Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      creto-oversight                        │
│                                                             │
│  ┌─────────────────┐         ┌─────────────────────────┐   │
│  │  Policy Engine  │────────►│  Request State Machine  │   │
│  │                 │         │                         │   │
│  └────────┬────────┘         └────────────┬────────────┘   │
│           │                               │                │
│           │                               ▼                │
│           │                  ┌─────────────────────────┐   │
│           │                  │   Checkpoint Store      │   │
│           │                  │   (durability)          │   │
│           │                  └─────────────────────────┘   │
│           │                               │                │
│           ▼                               ▼                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Notification Router                     │   │
│  └─────────┬──────────────┬──────────────┬─────────────┘   │
│            │              │              │                  │
│            ▼              ▼              ▼                  │
│       ┌────────┐    ┌────────┐    ┌──────────┐             │
│       │ Slack  │    │ Email  │    │ Webhook  │             │
│       └────────┘    └────────┘    └──────────┘             │
└─────────────────────────────────────────────────────────────┘
         │                    │
         ▼                    ▼
    ┌─────────┐          ┌─────────┐
    │  AuthZ  │          │  Memory │
    │(policy) │          │(context)│
    └─────────┘          └─────────┘
```

### 5.2 Request Lifecycle

```
Agent Action
     │
     ▼
┌─────────────────┐
│ Authorization   │
│ Check           │
└────────┬────────┘
         │ RequiresOversight
         ▼
┌─────────────────┐
│ Create Request  │
│ + Checkpoint    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐      ┌─────────────────┐
│ Fetch Memory    │─────►│ Build Context   │
│ Context         │      │                 │
└─────────────────┘      └────────┬────────┘
                                  │
                                  ▼
                         ┌─────────────────┐
                         │ Send to         │
                         │ Channels        │
                         └────────┬────────┘
                                  │
                    ┌─────────────┴─────────────┐
                    ▼                           ▼
             ┌─────────────┐             ┌─────────────┐
             │ Wait for    │             │ Escalation  │
             │ Response    │             │ Timer       │
             └──────┬──────┘             └──────┬──────┘
                    │                           │
                    └─────────────┬─────────────┘
                                  │
                                  ▼
                         ┌─────────────────┐
                         │ Update State    │
                         │ + Checkpoint    │
                         └────────┬────────┘
                                  │
                         ┌────────┴────────┐
                         ▼                 ▼
                   ┌──────────┐      ┌──────────┐
                   │ Approved │      │ Denied   │
                   │ Resume   │      │ Abort    │
                   │ Action   │      │ Action   │
                   └──────────┘      └──────────┘
```

---

## 6. API Design

### 6.1 Request Management

```rust
impl OversightService {
    /// Create new oversight request (called by Authorization)
    pub async fn create_request(
        &self,
        agent: &AgentIdentity,
        delegation_chain: &[AgentIdentity],
        action: &Action,
        policy_id: &PolicyId,
    ) -> Result<OversightRequest, Error>;

    /// Wait for resolution (blocking)
    pub async fn wait_for_resolution(
        &self,
        request_id: &RequestId,
        timeout: Duration,
    ) -> Result<OversightState, Error>;

    /// Cancel pending request
    pub async fn cancel_request(
        &self,
        request_id: &RequestId,
        reason: &str,
    ) -> Result<(), Error>;
}
```

### 6.2 Response Handling

```rust
impl OversightService {
    /// Submit approval response (called by channel handlers)
    pub async fn submit_response(
        &self,
        request_id: &RequestId,
        response: ApprovalResponse,
    ) -> Result<OversightState, Error>;

    /// Get request details (for UI/channel rendering)
    pub async fn get_request(
        &self,
        request_id: &RequestId,
    ) -> Result<OversightRequest, Error>;
}
```

### 6.3 Policy Configuration

```rust
impl OversightService {
    /// Register oversight policy
    pub async fn register_policy(
        &self,
        policy: Box<dyn OversightPolicy>,
    ) -> Result<PolicyId, Error>;

    /// Configure escalation chain for policy
    pub async fn set_escalation(
        &self,
        policy_id: &PolicyId,
        chain: EscalationChain,
    ) -> Result<(), Error>;
}
```

---

## 7. Integration Points

### 7.1 Authorization Integration

```rust
// Authorization returns RequiresOversight when policy triggers
match authz.check(request).await? {
    Decision::RequiresOversight { policy_id } => {
        let request = oversight.create_request(
            &agent_nhi,
            &delegation_chain,
            &action,
            &policy_id,
        ).await?;

        // Agent blocks waiting for human
        let state = oversight.wait_for_resolution(&request.id, timeout).await?;

        match state {
            OversightState::Approved { .. } => proceed_with_action(),
            OversightState::Denied { reason, .. } => abort_with_reason(reason),
            _ => handle_other_states(),
        }
    }
}
```

### 7.2 Memory Integration

```rust
// Fetch reasoning context from Memory
let memory_context = memory.query(MemoryQuery {
    agent_nhi: &agent_nhi,
    relevance_to: &action.description(),
    limit: 5,
}).await?;

// Include in request context
let context = RequestContext {
    action_description: action.human_description(),
    reasoning: memory_context.get_reasoning(),
    memory_context: memory_context.snippets,
    ..
};
```

### 7.3 Audit Integration

```rust
// All state transitions are audited
audit.log(AuditRecord {
    who: approver.identity,
    what: "oversight_approved",
    resource: format!("request://{}", request_id),
    outcome: Outcome::Success,
    signature: Some(response.signature),
    ..
}).await?;
```

---

## 8. Performance Requirements

| Metric | Target | Notes |
|--------|--------|-------|
| Request creation | <10ms | Including checkpoint |
| State transition | <1ms | State machine update |
| Notification send | <500ms | Per channel |
| Resume from checkpoint | <100ms | After crash recovery |

---

## 9. Open Questions

1. What's the default timeout for oversight requests?
2. Should we support "approve with modifications"?
3. How do we handle approver unavailability (vacation, etc.)?
4. Should there be approval delegation (A approves on behalf of B)?
5. What's the retention policy for completed requests?

---

## 10. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
