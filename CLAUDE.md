# Claude Code Playbook: creto-enablement

## 🎯 CRITICAL: SDD-FIRST METHODOLOGY

**This project follows a Software Design Document (SDD) first approach.**

> **Design before code. Document before implement. Specify before build.**

No code should be written until the corresponding SDD section is complete and approved.

---

## Project Overview

**creto-enablement** is a Rust monorepo containing the Enablement Layer products for the Creto Sovereign platform. These products provide orchestration and governance capabilities for AI agents.

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

## Design Philosophy

**Pattern: Extract from OSS → Rebuild with Sovereign Primitives**

Each product follows this approach:
1. Identify leading OSS implementation (Lago, HumanLayer, Agent Sandbox, Signal)
2. Extract proven patterns, data models, APIs
3. Rebuild in Rust with NHI/Crypto-Agility/Consensus/Audit integrated from foundation
4. Connect to Authorization service (168ns path) for inline policy enforcement

---

## 📋 SDD Document Structure

All design documents live in `/docs/sdd/`:

```
docs/sdd/
├── 00-overview.md           # Executive summary, vision, scope
├── 01-requirements.md       # Functional & non-functional requirements
├── 02-architecture.md       # System architecture & component design
├── 03-data-design.md        # Data models, schemas, storage strategy
├── 04-api-design.md         # API contracts, endpoints, interfaces
├── 05-security-design.md    # Security model, auth, encryption
├── 06-integration-design.md # External systems, third-party services
├── 07-deployment-design.md  # Infrastructure, CI/CD, environments
├── 08-testing-strategy.md   # Test plans, coverage requirements
├── 09-implementation-plan.md # Phased rollout, milestones, timeline
│
├── products/                # Per-product SDDs
│   ├── metering.md          # creto-metering design
│   ├── oversight.md         # creto-oversight design
│   ├── runtime.md           # creto-runtime design
│   └── messaging.md         # creto-messaging design
```

---

## Repository Structure

```
creto-enablement/
├── Cargo.toml                      # Workspace root
├── CLAUDE.md                       # This file
├── README.md
├── docs/
│   ├── sdd/                        # Software Design Documents
│   │   └── products/               # Per-product SDDs
│   ├── decisions/                  # Architecture Decision Records
│   └── diagrams/                   # Mermaid/ASCII diagrams
├── crates/
│   ├── creto-metering/             # Usage-based billing (Lago patterns)
│   ├── creto-oversight/            # Human-in-the-loop (HumanLayer patterns)
│   ├── creto-runtime/              # Sandboxed execution (Agent Sandbox patterns)
│   ├── creto-messaging/            # Secure agent messaging (Signal patterns)
│   └── creto-enablement-common/    # Shared types
├── examples/                       # Usage examples
├── tests/                          # Integration tests
└── benches/                        # Performance benchmarks
```

---

## OSS Pattern References

When implementing features, refer to these OSS projects for proven patterns:

### Metering (Lago Patterns)
- **Repo**: getlago/lago
- **Docs**: https://getlago.com/docs/guide/events/ingesting-usage
- **Key patterns**:
  - Event schema: `transaction_id`, `external_subscription_id`, `code`, `timestamp`, `properties`
  - Aggregation types: `COUNT`, `SUM`, `UNIQUE_COUNT`, `MAX`
  - Pricing models: flat fee, per-unit, tiered (graduated/volume), package, prepaid credits
  - Idempotency via `transaction_id`

### Oversight (HumanLayer Patterns)
- **Repo**: humanlayer/humanlayer
- **Docs**: https://humanlayer.vercel.app/
- **Key patterns**:
  - `@require_approval()` decorator pattern
  - `human_as_tool()` for agent-initiated human contact
  - Channel abstraction (Slack, email, webhook)
  - Escalation chains with timeout handling
  - Checkpoint/resume for durability

### Runtime (Agent Sandbox Patterns)
- **Repo**: kubernetes-sigs/agent-sandbox
- **Docs**: https://agent-sandbox.sigs.k8s.io/
- **Key patterns**:
  - CRDs: `Sandbox`, `SandboxTemplate`, `SandboxClaim`, `SandboxWarmPool`
  - Warm pool for sub-second allocation
  - Runtime abstraction (gVisor vs Kata)
  - Python SDK context manager pattern

### Messaging (Signal Protocol Patterns)
- **Spec**: https://signal.org/docs/specifications/doubleratchet/
- **Key patterns**:
  - X3DH for initial key agreement (adapt for NHI keys)
  - Double Ratchet for forward secrecy
  - Envelope: encrypted payload + wrapped key + signature
  - PQXDH/Triple Ratchet for PQC (ML-KEM integration)

---

## Integration Points

### With Authorization (creto-authz)

All four products integrate with the Authorization service:

```rust
// Metering: QuotaEnforcer called inline
// Oversight: Policy returns REQUIRES_OVERSIGHT
// Runtime: Network egress checked via AuthZ
// Messaging: Delivery gated by AuthZ
```

### With NHI (creto-nhi)

Every data structure includes agent identity:

```rust
use creto_nhi::{AgentIdentity, DelegationChain};

pub struct SomeCretoStruct {
    pub agent_nhi: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,
}
```

### With Crypto-Agility (creto-crypto)

Use platform crypto primitives, never hardcode algorithms.

### With Audit (creto-audit)

All operations log to immutable audit trail.

---

## Performance Targets

| Operation | Target | Notes |
|-----------|--------|-------|
| Quota check | <10µs | In-memory bloom filter + Redis fallback |
| Oversight state transition | <1ms | State machine update |
| Warm pool claim | <100ms | Pre-warmed sandbox binding |
| Cold sandbox spawn | <2s | gVisor, <5s Kata |
| Message encryption | >100K msg/s | AES-256-GCM + ML-KEM wrap |
| Message delivery auth | <1ms | AuthZ check (168ns) + routing |

---

## SDD Development Workflow

### Phase 1: Discovery & Requirements
1. Define problem statement and target users
2. Document functional requirements (user stories, use cases)
3. Document non-functional requirements (performance, scale, security)
4. Identify constraints and dependencies

### Phase 2: Architecture & Design
1. Design system architecture (components, boundaries, interactions)
2. Define data models and storage strategy
3. Specify API contracts and interfaces
4. Document security model and threat analysis

### Phase 3: Implementation Planning
1. Break down into implementation phases
2. Define milestones and success criteria
3. Establish testing strategy
4. Plan deployment and rollout

### Phase 4: Implementation (Code)
- **Only after SDD approval**
- Code follows the documented design
- Deviations require SDD updates first

---

## SDD Status Tracking

Use frontmatter in each SDD file:

```yaml
---
status: draft | review | approved | implemented
author: [name]
created: YYYY-MM-DD
updated: YYYY-MM-DD
reviewers: [names]
---
```

---

## Development Commands

```bash
# Build
cargo build --workspace

# Test
cargo test --workspace

# Specific crate
cargo test -p creto-metering

# Benchmarks
cargo bench -p creto-metering -- quota
cargo bench -p creto-messaging -- encryption
cargo bench -p creto-runtime -- warmpool
```

---

## Feature Flags

```toml
# creto-runtime: gvisor (default), kata, sgx
# creto-oversight: slack (default), email (default), teams, servicenow
# creto-metering: stripe
```

---

**Remember: The SDD is the source of truth. Code is the implementation of that truth.**
