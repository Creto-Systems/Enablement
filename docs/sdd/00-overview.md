---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
---

# SDD-00: Enablement Layer Overview

## Purpose

This document provides an executive summary of the Enablement Layer for the Creto Sovereign platform. It defines the vision, scope, and high-level architecture of the four products that comprise this layer.

## Scope

**In Scope:**
- creto-metering: Usage-based billing and quota enforcement
- creto-oversight: Human-in-the-loop approval workflows
- creto-runtime: Sandboxed agent execution environments
- creto-messaging: Secure end-to-end agent communication

**Out of Scope:**
- Platform Layer implementation (creto-nhi, creto-crypto, creto-consensus, creto-audit)
- Security Layer implementation (creto-authz, creto-memory, creto-storage)
- Client SDKs and UI components

---

## 1. Vision

The Enablement Layer provides orchestration and governance capabilities for AI agents operating within the Creto Sovereign platform. It enables organizations to:

1. **Meter and bill** agent usage with sub-10µs quota enforcement
2. **Require human approval** for sensitive agent actions
3. **Sandbox agent execution** with cryptographic attestation
4. **Secure agent communication** with post-quantum encryption

---

## 2. Problem Statement

AI agents operating autonomously require governance infrastructure that:

- Tracks resource consumption and enforces billing/quota limits inline
- Enables human oversight without blocking agent workflows unnecessarily
- Isolates agent code execution with verifiable security boundaries
- Provides secure, authenticated, encrypted inter-agent messaging

Existing solutions are fragmented, lack identity integration, and don't provide the performance characteristics required for production AI workloads.

---

## 3. Target Users

| User Type | Needs |
|-----------|-------|
| **Platform Operators** | Deploy, configure, and monitor Enablement services |
| **Agent Developers** | Integrate metering, oversight, runtime, messaging into agents |
| **Security Teams** | Audit agent actions, verify sandbox attestation |
| **Finance Teams** | Access billing data, configure pricing models |
| **Human Approvers** | Receive approval requests, respond via Slack/email |

---

## 4. Layer Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    ENABLEMENT LAYER (This Repo)                 │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌─────────┐ │
│  │   Metering   │ │   Oversight  │ │   Runtime    │ │Messaging│ │
│  │  (billing)   │ │    (HITL)    │ │  (sandbox)   │ │  (E2E)  │ │
│  └──────────────┘ └──────────────┘ └──────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              ↓ depends on
┌─────────────────────────────────────────────────────────────────┐
│                    SECURITY LAYER (External)                    │
│      creto-authz  │  creto-memory  │  creto-storage             │
└─────────────────────────────────────────────────────────────────┘
                              ↓ depends on
┌─────────────────────────────────────────────────────────────────┐
│                    PLATFORM LAYER (External)                    │
│   creto-nhi  │  creto-crypto  │  creto-consensus  │  creto-audit│
└─────────────────────────────────────────────────────────────────┘
```

---

## 5. Product Summaries

### 5.1 Metering (creto-metering)

**Purpose:** Usage-based billing and inline quota enforcement for AI agents.

**OSS Reference:** Lago (getlago/lago)

**Key Capabilities:**
- BillableEvent ingestion with idempotency
- Aggregation: COUNT, SUM, UNIQUE_COUNT, MAX
- Pricing models: flat, per-unit, tiered, package, prepaid credits
- Inline quota enforcement (<10µs via Authorization integration)
- Invoice generation with delegation chain attribution

### 5.2 Oversight (creto-oversight)

**Purpose:** Human-in-the-loop approval workflows for sensitive agent actions.

**OSS Reference:** HumanLayer (humanlayer/humanlayer)

**Key Capabilities:**
- OversightPolicy evaluation via Authorization
- OversightRequest state machine (Pending → Approved/Denied/Escalated/TimedOut)
- Multi-channel notifications (Slack, email, webhook)
- Escalation chains with configurable timeouts
- Checkpoint/resume for durability

### 5.3 Runtime (creto-runtime)

**Purpose:** Sandboxed execution environments for AI agents with cryptographic attestation.

**OSS Reference:** Agent Sandbox (kubernetes-sigs/agent-sandbox)

**Key Capabilities:**
- SandboxSpec bound to agent NHI at spawn
- Warm pool for <100ms claim latency
- Multiple backends: gVisor, Kata Containers
- Platform attestation (cryptographic proof of sandbox config)
- Network egress enforcement via Authorization
- NHI-delegated secret injection

### 5.4 Messaging (creto-messaging)

**Purpose:** Secure end-to-end encrypted communication between agents.

**OSS Reference:** Signal Protocol

**Key Capabilities:**
- MessageEnvelope with E2E encryption (AES-256-GCM)
- ML-KEM-768 key wrapping (post-quantum)
- Ed25519 + ML-DSA hybrid signatures (non-repudiation)
- Request/response and pub/sub patterns
- Authorization-gated delivery

---

## 6. Design Philosophy

**Pattern: Extract from OSS → Rebuild with Sovereign Primitives**

1. Identify leading OSS implementation
2. Extract proven patterns, data models, APIs
3. Rebuild in Rust with NHI/Crypto-Agility/Consensus/Audit integrated from foundation
4. Connect to Authorization service (168ns path) for inline policy enforcement

---

## 7. Cross-Cutting Concerns

### 7.1 Identity (NHI Integration)

Every operation includes:
- `agent_nhi`: The agent performing the action
- `delegation_chain`: Full chain to root human principal

### 7.2 Authorization

All operations check Authorization service:
- Metering: Quota enforcement inline
- Oversight: Policy triggers REQUIRES_OVERSIGHT decision
- Runtime: Network egress policies
- Messaging: Delivery authorization

### 7.3 Audit

All significant operations logged to immutable audit:
- Who (agent NHI + delegation chain)
- What (action type)
- Resource (target of action)
- Outcome (success/failure)
- Cryptographic signature

### 7.4 Crypto-Agility

No hardcoded algorithms. All crypto via creto-crypto:
- Encryption: AES-256-GCM (symmetric)
- Key wrapping: ML-KEM-768 (post-quantum)
- Signing: Ed25519 + ML-DSA (hybrid)
- Hashing: Algorithm-agile

---

## 8. Success Metrics

| Metric | Target |
|--------|--------|
| Quota check latency | <10µs (p99) |
| Oversight state transition | <1ms (p99) |
| Warm pool claim latency | <100ms (p99) |
| Cold sandbox spawn | <2s gVisor, <5s Kata |
| Message encryption throughput | >100K msg/s |
| Message delivery auth | <1ms (p99) |

---

## 9. Open Questions

1. Should metering support real-time aggregation or batch-only?
2. What's the default timeout for oversight requests?
3. Should runtime support SGX/TDX attestation in v1?
4. Should messaging support multi-recipient broadcasts?

---

## 10. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
