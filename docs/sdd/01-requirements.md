---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
---

# SDD-01: Requirements & OSS Pattern Analysis

## Purpose

This document defines the functional and non-functional requirements for the Enablement Layer, including detailed analysis of OSS reference implementations, their limitations, and the Creto extensions that address the "attribution gap."

## Scope

**In Scope:**
- Design philosophy and precedents
- OSS pattern extraction (Lago, HumanLayer, Agent Sandbox, Signal)
- Gap analysis and Creto differentiation
- Rust trait definitions per product
- Crate architecture
- Compliance mapping

**Out of Scope:**
- Implementation details (see product SDDs)
- Deployment architecture (see 07-deployment-design.md)

---

## 1. Introduction

### 1.1 Design Philosophy

This document follows the same design pattern established for Creto's Security Layer products: identify leading open-source implementations, extract their proven patterns and data models, then rebuild in Rust with Sovereign platform primitives (NHI, Crypto-Agility, Consensus, Audit) integrated from the foundation.

**Precedents:**

| Product | Pattern Source | Creto Rebuild |
|---------|----------------|---------------|
| Authorization by Creto | Cerbos/Zanzibar | 168ns latency, 4-agent intelligence, NHI-native |
| Memory by Creto (RuVector) | pgvector | Row-level vector security, embedding provenance |
| Storage by Creto | S3/MinIO | Crypto-agile encryption, semantic classification |

### 1.2 Enablement Layer Products

| Product | OSS Reference | Creto Differentiation |
|---------|---------------|----------------------|
| **Metering** | Lago, OpenMeter | NHI-attributed events, inline quota enforcement via Authorization (168ns path) |
| **Oversight** | HumanLayer, LangGraph | Policy-triggered via Authorization, cryptographically signed approvals, immutable audit |
| **Runtime** | Agent Sandbox, gVisor | NHI-bound sandboxes, egress via Authorization, attestation linked to identity |
| **Messaging** | Signal Protocol, NATS | E2E with NHI keys, PQC-ready (ML-KEM), Authorization-gated delivery |

### 1.3 The Attribution Gap

Every OSS project in this space answers "what happened" but not "which agent did it, who authorized that agent, and can you prove it cryptographically?" This is the gap Creto fills.

| Dimension | OSS Approach | Creto Approach |
|-----------|--------------|----------------|
| **Identity** | API key, user_id string | Agent NHI + full delegation chain |
| **Attribution** | "Customer X used 100 tokens" | "Agent A (spawned by Agent B, delegated from Human C) used 100 tokens" |
| **Cryptography** | Hardcoded algorithms | Crypto-agile, PQC migration path |
| **Audit** | Application logs | Immutable, Merkle-anchored, legally admissible |
| **Authorization** | Separate system, bolted on | Inline (168ns), same policy language |

---

## 2. Metering by Creto

### 2.1 OSS Reference: Lago

**Repository:** getlago/lago (10K+ stars, AGPL-3.0)
**Throughput:** 15,000 events/second
**Pedigree:** Qonto team (built billing 0→tens of millions MRR), YC-backed

#### 2.1.1 Lago Event Model

Lago's event structure is the foundation of usage-based billing. Every billable action sends an event with these fields:

| Field | Lago Definition |
|-------|-----------------|
| `transaction_id` | Unique identifier for idempotency and deduplication |
| `external_subscription_id` | Links event to customer subscription for billing |
| `code` | Billable metric code (e.g., "api_calls", "storage_gb") |
| `timestamp` | UNIX timestamp (seconds or milliseconds) |
| `precise_total_amount_cents` | For dynamic pricing models |
| `properties` | Custom dimensions: `{"tokens": 1500, "model": "gpt-4"}` |

#### 2.1.2 Lago Aggregation Types

| Type | Calculation | Use Case |
|------|-------------|----------|
| COUNT | Number of events | API calls, transactions |
| SUM | Sum of property value | Tokens, bytes, compute-ms |
| UNIQUE COUNT | Distinct values of property | Active users, unique IPs |
| MAX | Maximum property value | Peak connections, max memory |

#### 2.1.3 Lago Pricing Models

- **Flat Fee:** $X per billing period
- **Per Unit:** $X per metric unit (linear)
- **Tiered (Graduated):** Different rates per tier, each tier billed at its rate
- **Tiered (Volume):** All units billed at the tier reached
- **Package:** $X for Y units, overage charged separately
- **Prepaid Credits:** Wallet drawdown with auto-top-up

### 2.2 Lago Limitations (What's Missing)

| Gap | Impact |
|-----|--------|
| No agent identity model | Can't answer "which agent instance consumed this?" |
| No delegation chain | Can't trace billing to root human principal |
| Quota is post-hoc | Check happens after consumption, not inline with authorization |
| No cryptographic audit | Billing disputes require trust, not proof |
| Hardcoded crypto | No PQC migration path |

### 2.3 Creto Rebuild: BillableEvent

Creto extends Lago's event model with NHI integration:

| Field | Lago | Creto Extension |
|-------|------|-----------------|
| `transaction_id` | String UUID | Same (idempotency key) |
| `external_subscription_id` | String customer ID | **`agent_nhi: AgentIdentity`** (cryptographic) |
| (none) | — | **`delegation_chain: Vec<AgentIdentity>`** |
| `code` | Metric code string | Same (`event_type`) |
| `timestamp` | UNIX timestamp | **Consensus-ordered timestamp** |
| `properties` | JSON object | Same (typed `Properties`) |
| (none) | — | **`signature: Signature`** (ML-DSA) |

### 2.4 Creto Integration: Inline Quota Enforcement

Unlike Lago (post-hoc), Creto enforces quotas inline during Authorization:

1. Agent requests action → Authorization service (168ns policy check)
2. Authorization calls `QuotaEnforcer` inline (~10µs additional)
3. `QuotaEnforcer` checks: bloom filter (in-memory) → Redis (fallback)
4. If over quota: `DENY` with `reason="quota_exceeded"`, `suggested_action="upgrade_plan"`
5. If within quota: `ALLOW` + reserve provisional usage
6. On action completion: finalize `BillableEvent` → Immutable Audit

**Total latency:** 168ns (policy) + 10µs (quota) = **sub-20µs authorization with quota**

### 2.5 Rust Interface: creto-metering

| Trait | Methods & Purpose |
|-------|-------------------|
| `BillableEvent` | `agent_nhi()`, `delegation_chain()`, `event_type()`, `properties()`, `timestamp()`, `idempotency_key()` |
| `MetricAggregator` | `aggregate(events, window) → AggregatedMetric`. Implements SUM, COUNT, UNIQUE_COUNT, MAX |
| `QuotaEnforcer` | `check_quota(agent, action) → QuotaDecision`. Called inline from Authorization |
| `PriceCalculator` | `calculate(metric, price_model) → Amount`. Supports all Lago pricing models |
| `InvoiceGenerator` | `generate(customer, period) → Invoice`. Period close, line items, credits |

---

## 3. Oversight by Creto

### 3.1 OSS Reference: HumanLayer

**Repository:** humanlayer/humanlayer (8K+ stars, OSS)
**Pattern:** `@require_approval` decorator intercepts tool calls
**Channels:** Slack, Email, Discord, webhook

#### 3.1.1 HumanLayer Core Pattern

HumanLayer wraps "dangerous" functions with an approval gate:

```python
@hl.require_approval()
def send_email(to: str, subject: str, body: str):
    # Only executes after human clicks "Approve"
```

When the LLM calls this tool, HumanLayer intercepts, sends notification to human, pauses execution until response.

#### 3.1.2 HumanLayer Workflow

1. Define which tools require approval (decorator or config)
2. Agent calls tool → HumanLayer intercepts
3. Notification sent via configured channel (Slack bot, email)
4. Human sees: "Agent wants to send this email. Approve/Deny?"
5. Human clicks Approve → tool executes
6. Human clicks Deny → tool call fails with feedback to LLM

#### 3.1.3 HumanLayer Features

| Feature | Description |
|---------|-------------|
| `require_approval` | Decorator to gate specific tool calls |
| `human_as_tool` | Let agent ask human for advice/input (not just approval) |
| Omnichannel | Route to Slack, email, Discord, webhook |
| Granular routing | Route to specific teams or individuals |
| Timeouts | Auto-deny or escalate after timeout |
| Audit trail | Log all approvals/denials |

### 3.2 Also Referenced: Agent Control Plane (ACP)

HumanLayer's ACP project provides Kubernetes-native agent orchestration with HITL:

- **Task CRD:** Kubernetes resource for agent tasks
- **ContactChannel CRD:** Define how to reach humans (Slack, email)
- **Durable execution:** Tasks survive pod restarts
- **MCP support:** Model Context Protocol integration

### 3.3 HumanLayer Limitations (What's Missing)

| Gap | Impact |
|-----|--------|
| Approval at tool layer, not policy layer | Must annotate every tool; can't express "all actions > $10K need approval" |
| No cryptographic signatures | "Alice approved" is a log entry, not a signed attestation |
| No agent identity context | Approver doesn't see delegation chain (who spawned this agent?) |
| No Memory integration | Approver doesn't see why agent is doing this (context from Memory) |
| Application-layer audit | Logs can be tampered; not Merkle-anchored |

### 3.4 Creto Rebuild: Policy-Triggered Oversight

Creto integrates HITL at the Authorization policy layer, not the tool layer:

1. Same policy language that defines "can agent do X" also defines "must human approve X"
2. Agent requests action → Authorization evaluates policy
3. Policy returns: `ALLOW`, `DENY`, or `REQUIRES_OVERSIGHT`
4. `REQUIRES_OVERSIGHT` → Oversight service creates `OversightRequest`
5. `OversightRequest` includes: agent NHI, delegation chain, action, Memory context (why)
6. Human approves → signed with approver's key → logged to Immutable Audit

### 3.5 Creto Extension: OversightRequest

| Field | HumanLayer | Creto Extension |
|-------|------------|-----------------|
| `request_id` | UUID | Same |
| `tool_name` | String function name | **`pending_action: Action`** (structured) |
| (none) | — | **`agent_nhi: AgentIdentity`** |
| (none) | — | **`delegation_chain: Vec<AgentIdentity>`** |
| (none) | — | **`policy_trigger: PolicyId`** (which rule required this) |
| (none) | — | **`context: MemoryContext`** (why agent is acting) |
| `approved_by` | String email | **`approver: HumanIdentity + Signature`** (ML-DSA) |

### 3.6 Rust Interface: creto-oversight

| Trait | Methods & Purpose |
|-------|-------------------|
| `OversightPolicy` | `requires_approval(ctx) → ApprovalRequirement`. Evaluated by Authorization |
| `OversightRequest` | State machine: Pending → Approved/Denied/Escalated/TimedOut |
| `NotificationChannel` | `send(request) → Result`. Slack, Email, Teams, webhook implementations |
| `EscalationChain` | `next_level()`, `timeout_action()`. Configurable per policy |
| `ApprovalSigner` | `sign(approval) → SignedApproval`. Uses Crypto-Agility layer |
| `CheckpointStore` | `checkpoint()`, `resume()`. Survives process restart (like ACP) |

---

## 4. Runtime by Creto

### 4.1 OSS Reference: Agent Sandbox

**Repository:** kubernetes-sigs/agent-sandbox (CNCF, Apache-2.0)
**Announced:** KubeCon NA 2025 (November)
**Backends:** gVisor (default), Kata Containers
**Pedigree:** Google Cloud (GKE Sandbox integration)

#### 4.1.1 Agent Sandbox CRDs

| CRD | Purpose |
|-----|---------|
| `Sandbox` | Core resource: declarative API for isolated, stateful, singleton container |
| `SandboxTemplate` | Blueprint: base image, resource limits, security policies |
| `SandboxClaim` | Request: transactional resource to request an execution environment |
| `SandboxWarmPool` | Pool: pre-warmed sandboxes for sub-second allocation |

#### 4.1.2 Agent Sandbox Architecture

From the Google Open Source Blog announcement:

- **Strong Isolation:** gVisor (user-space kernel, syscall interception) or Kata (lightweight VM)
- **Warm Pools:** Pre-warmed pods reduce cold start to <1 second (90% improvement)
- **Stable Identity:** Each sandbox has stable hostname and network identity
- **Persistent Storage:** Survives restarts
- **Python SDK:** Context manager for sandbox lifecycle (`with sandbox.create() as sb: ...`)

#### 4.1.3 GKE-Specific Features

- **Pod Snapshots:** Checkpoint/restore running pods (CPU and GPU workloads)
- **Managed gVisor:** Optimized for AI agent workloads in GKE Sandbox

### 4.2 Also Referenced: gVisor & Kata Containers

| Aspect | gVisor | Kata Containers |
|--------|--------|-----------------|
| Isolation model | User-space kernel (intercept syscalls) | Lightweight VM (hardware virtualization) |
| Security boundary | Process-level (Sentry) | VM-level (KVM/QEMU) |
| Cold start | Fast (<100ms with warm pool) | Slower (~500ms) |
| Syscall coverage | ~200 syscalls | Full Linux kernel |
| Best for | High-throughput, interactive | Regulated, highest security |

### 4.3 Agent Sandbox Limitations (What's Missing)

| Gap | Impact |
|-----|--------|
| No NHI binding | Sandbox has "stable identity" but not cryptographic agent identity |
| No attestation to identity | Can't prove "this code running in this sandbox belongs to this agent" |
| No Authorization integration | Network policy is static K8s NetworkPolicy, not dynamic AuthZ |
| No secrets via NHI delegation | Secrets are K8s Secrets, not NHI-delegated leases |

### 4.4 Creto Rebuild: NHI-Bound Sandboxes

Creto extends Agent Sandbox with NHI integration at spawn time:

| Field | Agent Sandbox | Creto Extension |
|-------|---------------|-----------------|
| `image` | OCI image ref | Same |
| `metadata.name` | K8s stable identity | **`agent_nhi: AgentIdentity`** (cryptographic) |
| `runtimeClassName` | gvisor or kata-qemu | Same (`attestation_platform`) |
| `resources` | K8s resource limits | Same (`resource_limits`) |
| (NetworkPolicy) | Static K8s NetworkPolicy | **`network_policy`: dynamic via Authorization** |
| (K8s Secrets) | Static secret mount | **`secrets: Vec<SecretRef>`** (NHI-delegated leases) |
| (none) | — | **`attestation: Attestation`** (signed proof) |

### 4.5 Creto Integration: Egress via Authorization

Unlike static K8s NetworkPolicy, Creto enforces egress dynamically:

1. Sandbox attempts connection to `api.example.com:443`
2. Runtime intercepts at network layer
3. Authorization check: can `agent_nhi` access `api.example.com`?
4. Policy evaluated (same 168ns path, same policy language)
5. `ALLOW` → connection proceeds, logged to Audit
6. `DENY` → connection blocked, logged with reason

### 4.6 Creto Extension: Attestation

Cryptographic proof linking sandbox to agent identity:

| Field | Description |
|-------|-------------|
| `sandbox_id` | Unique identifier for this sandbox instance |
| `agent_nhi` | Bound agent identity (from spawn) |
| `image_hash` | SHA-256 of container image layers |
| `config_hash` | SHA-256 of sandbox configuration |
| `platform` | gVisor, Kata, SGX, SEV |
| `timestamp` | Consensus-ordered attestation time |
| `signature` | Platform-signed (verifiable by third party) |

### 4.7 Rust Interface: creto-runtime

| Trait | Methods & Purpose |
|-------|-------------------|
| `SandboxRuntime` | `spawn(spec) → Handle`, `terminate()`, `exec()`, `checkpoint()`, `restore()` |
| `Attestor` | `attest(handle) → Attestation`. Platform-specific (gVisor, Kata, SGX) |
| `WarmPoolManager` | `claim(pool, nhi) → Handle`, `release()`. Sub-second allocation |
| `NetworkEnforcer` | `check_egress(dest) → calls Authorization`. Intercepts at network layer |
| `SecretInjector` | `inject(handle, refs)`. Injects NHI-delegated secrets into sandbox |

---

## 5. Messaging by Creto

### 5.1 OSS Reference: Signal Protocol

**Specification:** signal.org/docs/specifications/doubleratchet
**Adopted by:** WhatsApp, Facebook Messenger, Google Messages (RCS)
**Properties:** Forward secrecy, post-compromise security, deniability

#### 5.1.1 Signal Protocol Components

| Component | Purpose |
|-----------|---------|
| X3DH | Extended Triple Diffie-Hellman: Initial key agreement (asynchronous) |
| Double Ratchet | Ongoing key renewal: DH ratchet + KDF ratchet |
| Prekeys | One-time ephemeral public keys uploaded to server for async |
| Curve25519 | Elliptic curve for DH key exchange |
| AES-256-GCM | Symmetric encryption for message content |
| HMAC-SHA256 | Key derivation function (KDF) |

#### 5.1.2 Double Ratchet Algorithm

The core innovation of Signal Protocol:

1. **Symmetric Ratchet (KDF Chain):** Derives new message key for every message. Earlier keys cannot be calculated from later.
2. **DH Ratchet:** Parties send new DH public values attached to messages. Mixes fresh DH into derived keys.
3. **Result:** Compromise of current key doesn't expose past (forward secrecy) or future (post-compromise security) messages.

#### 5.1.3 Signal Protocol Security Properties

- **Confidentiality:** Only sender and recipient can read content
- **Integrity:** Tampering detected via AEAD
- **Authentication:** Parties verified via public key fingerprints
- **Forward secrecy:** Past messages protected even if keys compromised
- **Post-compromise security:** Recovery after key compromise ("self-healing")
- **Deniability:** Cannot cryptographically prove who sent a message

### 5.2 Signal Protocol Extensions: PQC

As of October 2025, Signal introduced post-quantum extensions:

- **PQXDH:** Post-Quantum Extended Diffie-Hellman (replaces X3DH)
- **Triple Ratchet:** Double Ratchet + Sparse Post-Quantum Ratchet (SPQR) in parallel
- **ML-KEM (CRYSTALS-Kyber):** Post-quantum KEM for key encapsulation

**Hybrid approach:** Attacker must break both EC and PQC to compromise.

### 5.3 Also Referenced: NATS

For message patterns (not encryption):

- **Request/Reply:** Synchronous message exchange with correlation ID
- **Pub/Sub:** Topic-based messaging with subscriber management
- **Queue Groups:** Load balancing across subscribers
- **JetStream:** Durable messaging with replay

### 5.4 Signal Protocol Limitations for Agents

| Gap | Impact |
|-----|--------|
| Human-centric identity | Designed for phone numbers/accounts, not agent NHI |
| No delegation model | Can't express "Agent B can message on behalf of Agent A" |
| No policy-gated delivery | If you have address, you can send (no Authorization check) |
| Deniability undesirable | For agents, we want non-repudiation (audit trail) |
| No group efficiency | Signal's group protocol is pairwise (inefficient for agent swarms) |

### 5.5 Creto Rebuild: NHI-Native E2E Messaging

Creto adapts Signal Protocol patterns for agent communication:

| Aspect | Signal Protocol | Creto Approach |
|--------|-----------------|----------------|
| Identity | Phone number / account | Agent NHI (cryptographic) |
| Key agreement | X3DH / PQXDH | NHI public key (ML-KEM-768) |
| Encryption | AES-256-GCM | Same (via Crypto-Agility layer) |
| Signatures | Optional (deniability) | **Required** (Ed25519 + ML-DSA) |
| Delivery | Anyone can send | **Authorization-gated** |
| Audit | None (privacy) | **Metadata to Immutable Audit** |

### 5.6 Creto Message Envelope

| Field | Description |
|-------|-------------|
| `message_id` | Unique identifier for tracking and deduplication |
| `sender_nhi` | Sending agent's NHI (authenticated) |
| `recipient_nhi` | Receiving agent's NHI (or topic for pub/sub) |
| `encrypted_payload` | AES-256-GCM encrypted message content |
| `wrapped_key` | Symmetric key wrapped with recipient's ML-KEM-768 public key |
| `signature` | Ed25519 + ML-DSA signature over envelope (non-repudiation) |
| `timestamp` | Consensus-ordered send time |

### 5.7 Creto Integration: Authorization-Gated Delivery

Before delivery, Authorization checks:

1. Can `sender_nhi` send messages at all?
2. Can `sender_nhi` send to `recipient_nhi` specifically?
3. Is `sender_nhi` rate-limited for this channel?
4. If pub/sub: can `sender_nhi` publish to this topic?

If any check fails: Message rejected before delivery. Sender receives `DENY` with reason.

### 5.8 Rust Interface: creto-messaging

| Trait | Methods & Purpose |
|-------|-------------------|
| `MessageSender` | `send()`, `send_and_wait()`, `publish()`. Encrypts + signs + delivers |
| `MessageReceiver` | `receive()`, `subscribe()`, `acknowledge()`. Decrypts + verifies |
| `MessageEncryptor` | `encrypt()`, `decrypt()`. Uses Crypto-Agility layer (ML-KEM-768) |
| `MessageSigner` | `sign()`, `verify()`. Uses Crypto-Agility layer (Ed25519 + ML-DSA) |
| `DeliveryPolicy` | `can_deliver() → calls Authorization` (168ns) |

---

## 6. Crate Architecture

### 6.1 Monorepo Structure

The Enablement Layer is organized as a Cargo workspace monorepo (`creto-enablement/`):

```
creto-enablement/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── creto-metering/           # Lago patterns → NHI attribution
│   │   ├── src/
│   │   │   ├── event.rs          # BillableEvent trait
│   │   │   ├── aggregation.rs    # Metric aggregations
│   │   │   ├── pricing.rs        # Price models
│   │   │   ├── quota.rs          # Inline enforcement
│   │   │   └── invoice.rs
│   │
│   ├── creto-oversight/          # HumanLayer patterns → policy-triggered
│   │   ├── src/
│   │   │   ├── policy.rs         # OversightPolicy trait
│   │   │   ├── request.rs        # OversightRequest state machine
│   │   │   ├── channel.rs        # NotificationChannel trait
│   │   │   ├── escalation.rs
│   │   │   └── checkpoint.rs
│   │
│   ├── creto-runtime/            # Agent Sandbox patterns → NHI-bound
│   │   ├── src/
│   │   │   ├── sandbox.rs        # SandboxRuntime trait
│   │   │   ├── attestation.rs    # Platform attestation
│   │   │   ├── warmpool.rs       # Pre-warmed pools
│   │   │   ├── network.rs        # Egress via AuthZ
│   │   │   └── backend/
│   │   │       ├── gvisor.rs
│   │   │       └── kata.rs
│   │
│   ├── creto-messaging/          # Signal patterns → NHI keys
│   │   ├── src/
│   │   │   ├── envelope.rs       # E2E encrypted messages
│   │   │   ├── sender.rs
│   │   │   ├── receiver.rs
│   │   │   └── delivery.rs       # AuthZ-gated
│   │
│   └── creto-enablement-common/  # Shared types
```

### 6.2 Dependency Graph

Each Enablement crate depends on Platform and Security layers:

| Enablement Crate | Depends On | Integration Point |
|------------------|------------|-------------------|
| creto-metering | creto-nhi | `AgentIdentity` in `BillableEvent` |
| creto-metering | creto-authz | `QuotaEnforcer` called inline from AuthZ |
| creto-oversight | creto-authz | `OversightPolicy` evaluated by AuthZ |
| creto-oversight | creto-crypto | `ApprovalSigner` for signed approvals |
| creto-runtime | creto-nhi | `AgentIdentity` bound to sandbox at spawn |
| creto-runtime | creto-authz | `NetworkEnforcer` calls AuthZ for egress |
| creto-messaging | creto-crypto | ML-KEM-768 encryption, ML-DSA signatures |
| creto-messaging | creto-authz | `DeliveryPolicy` calls AuthZ before delivery |
| (all) | creto-audit | All operations logged to Immutable Audit |

---

## 7. Build vs. Integrate Strategy

### 7.1 Decision Framework

| Product | Strategy | OSS Accelerator | Effort |
|---------|----------|-----------------|--------|
| Metering | Extract patterns, rebuild in Rust | Lago event model, aggregations | 3-4 months |
| Oversight | Extract patterns, integrate with AuthZ | HumanLayer SDK patterns | 2-3 months |
| Runtime | Build on standard, extend with NHI | Agent Sandbox CRDs + gVisor | 4-6 months |
| Messaging | Adapt Signal patterns for agents | Double Ratchet concepts | 3-4 months |

### 7.2 What We Take from OSS

- **Lago:** Event schema, aggregation types, pricing models, invoice structure
- **HumanLayer:** Decorator pattern, channel abstraction, escalation chains, checkpoint/resume
- **Agent Sandbox:** CRD structure, warm pool pattern, runtime abstraction
- **Signal Protocol:** E2E encryption flow, key wrapping pattern (adapted for PQC)

### 7.3 What We Add (Creto Differentiators)

| Differentiator | Value |
|----------------|-------|
| **NHI Attribution** | Every event/request/sandbox/message tied to cryptographic agent identity |
| **Delegation Chain** | Full lineage to root human principal (accountability) |
| **Inline Authorization** | 168ns policy check with quota/oversight/egress in same path |
| **Crypto-Agility** | PQC-ready (ML-KEM-768, ML-DSA), zero-downtime algorithm migration |
| **Immutable Audit** | Merkle-anchored, legally admissible compliance trail |
| **Rust Performance** | Memory-safe, zero-copy, SIMD where applicable |

### 7.4 Same Playbook as Security Layer

This approach mirrors what worked for Authorization/Memory/Storage:

| Product | OSS Pattern Source | Creto Rebuild |
|---------|-------------------|---------------|
| Authorization | Cerbos policy model, Zanzibar ReBAC | 168ns, 4-agent intelligence, NHI-native |
| Memory | pgvector (Postgres extension) | RuVector: row-level security, provenance |
| Storage | S3 API, MinIO | Crypto-agile, semantic classification |
| Metering | Lago event model | NHI attribution, inline quota |
| Oversight | HumanLayer SDK | Policy-triggered, signed approvals |
| Runtime | Agent Sandbox + gVisor | NHI-bound, egress via AuthZ |
| Messaging | Signal Protocol | NHI keys, PQC, AuthZ-gated |

---

## 8. Compliance Mapping

| Regulation | Metering | Oversight | Runtime | Messaging |
|------------|----------|-----------|---------|-----------|
| **EU AI Act Art 14** | — | Human oversight ✓ | — | — |
| **SOX 404** | Billing audit ✓ | Dual control ✓ | — | — |
| **FedRAMP SC-7** | — | — | Boundary ✓ | — |
| **FedRAMP SC-8** | — | — | — | Transit enc ✓ |
| **HIPAA** | Usage audit ✓ | Min necessary ✓ | Isolation ✓ | PHI enc ✓ |
| **PCI DSS** | — | — | Segment ✓ | Transit enc ✓ |

---

## 9. Open Questions

1. Should Metering support real-time streaming aggregation or batch-only?
2. What's the default timeout for Oversight requests?
3. Should Runtime support SGX/TDX attestation in v1?
4. Should Messaging implement full Double Ratchet for per-message forward secrecy?
5. How do we handle cross-organization agent communication?

---

## 10. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
