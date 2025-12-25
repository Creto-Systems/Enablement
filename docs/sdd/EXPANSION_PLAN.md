# SDD Expansion Plan: Enablement Layer

## Executive Summary

This plan expands the Enablement Layer documentation from 13 high-level SDDs to a comprehensive **67-document** SDD suite with full traceability, validation gates, and cross-product alignment.

---

## 1. Complete SDD Inventory

### 1.1 Core Platform SDDs (10 docs) - ✅ EXISTS

| ID | Document | Status |
|----|----------|--------|
| CORE-00 | `00-overview.md` | ✅ Complete |
| CORE-01 | `01-requirements.md` | ✅ Complete |
| CORE-02 | `02-architecture.md` | ✅ Complete |
| CORE-03 | `03-data-design.md` | ✅ Complete |
| CORE-04 | `04-api-design.md` | ✅ Complete |
| CORE-05 | `05-security-design.md` | ✅ Complete |
| CORE-06 | `06-integration-design.md` | ✅ Complete |
| CORE-07 | `07-deployment-design.md` | ✅ Complete |
| CORE-08 | `08-testing-strategy.md` | ✅ Complete |
| CORE-09 | `09-implementation-plan.md` | ✅ Complete |

### 1.2 Per-Product SDDs (4 products × 8 docs = 32 docs) - 🔴 NEW

#### Metering Product (8 docs)

| ID | Document | Description |
|----|----------|-------------|
| MTR-00 | `products/metering/00-overview.md` | Product vision, scope, success metrics |
| MTR-01 | `products/metering/01-requirements.md` | Detailed functional/non-functional requirements |
| MTR-02 | `products/metering/02-architecture.md` | Component design, internal data flows |
| MTR-03 | `products/metering/03-data-design.md` | Schemas, indexes, partitioning strategy |
| MTR-04 | `products/metering/04-api-design.md` | Detailed API contracts, SDK methods |
| MTR-05 | `products/metering/05-security.md` | Product-specific threat model, controls |
| MTR-06 | `products/metering/06-testing.md` | Test cases, benchmarks, coverage |
| MTR-07 | `products/metering/07-runbook.md` | Operational procedures, troubleshooting |

#### Oversight Product (8 docs)

| ID | Document | Description |
|----|----------|-------------|
| OVS-00 | `products/oversight/00-overview.md` | Product vision, scope, success metrics |
| OVS-01 | `products/oversight/01-requirements.md` | Detailed functional/non-functional requirements |
| OVS-02 | `products/oversight/02-architecture.md` | Component design, state machine details |
| OVS-03 | `products/oversight/03-data-design.md` | Schemas, policy storage, request lifecycle |
| OVS-04 | `products/oversight/04-api-design.md` | Detailed API contracts, webhook specs |
| OVS-05 | `products/oversight/05-security.md` | Approval verification, channel security |
| OVS-06 | `products/oversight/06-testing.md` | State machine tests, integration tests |
| OVS-07 | `products/oversight/07-runbook.md` | Escalation handling, timeout procedures |

#### Runtime Product (8 docs)

| ID | Document | Description |
|----|----------|-------------|
| RTM-00 | `products/runtime/00-overview.md` | Product vision, scope, success metrics |
| RTM-01 | `products/runtime/01-requirements.md` | Sandbox requirements, isolation levels |
| RTM-02 | `products/runtime/02-architecture.md` | Controller design, warm pool internals |
| RTM-03 | `products/runtime/03-data-design.md` | Sandbox state, attestation records |
| RTM-04 | `products/runtime/04-api-design.md` | CRD specs, SDK methods |
| RTM-05 | `products/runtime/05-security.md` | Isolation guarantees, egress controls |
| RTM-06 | `products/runtime/06-testing.md` | Sandbox lifecycle tests, escape tests |
| RTM-07 | `products/runtime/07-runbook.md` | Pool management, incident response |

#### Messaging Product (8 docs)

| ID | Document | Description |
|----|----------|-------------|
| MSG-00 | `products/messaging/00-overview.md` | Product vision, scope, success metrics |
| MSG-01 | `products/messaging/01-requirements.md` | E2E encryption requirements, delivery SLAs |
| MSG-02 | `products/messaging/02-architecture.md` | Encryption pipeline, routing design |
| MSG-03 | `products/messaging/03-data-design.md` | Message storage, key management |
| MSG-04 | `products/messaging/04-api-design.md` | Messaging API, channel specs |
| MSG-05 | `products/messaging/05-security.md` | Cryptographic protocol, key rotation |
| MSG-06 | `products/messaging/06-testing.md` | Crypto tests, delivery tests |
| MSG-07 | `products/messaging/07-runbook.md` | Key compromise procedures, delivery debugging |

### 1.3 Architecture Decision Records (10 docs) - 🔴 NEW

| ID | Document | Decision Topic |
|----|----------|----------------|
| ADR-001 | `decisions/ADR-001-hybrid-signatures.md` | Ed25519 + ML-DSA hybrid approach |
| ADR-002 | `decisions/ADR-002-storage-strategy.md` | PostgreSQL vs alternatives |
| ADR-003 | `decisions/ADR-003-warm-pool-design.md` | Pre-warming vs on-demand |
| ADR-004 | `decisions/ADR-004-message-encryption.md` | ML-KEM + AES-256-GCM choice |
| ADR-005 | `decisions/ADR-005-quota-enforcement.md` | Bloom filter + Redis design |
| ADR-006 | `decisions/ADR-006-oversight-channels.md` | Slack/Email/Webhook priority |
| ADR-007 | `decisions/ADR-007-sandbox-runtime.md` | gVisor vs Kata selection |
| ADR-008 | `decisions/ADR-008-api-versioning.md` | gRPC versioning strategy |
| ADR-009 | `decisions/ADR-009-observability-stack.md` | Prometheus/Grafana/Loki choice |
| ADR-010 | `decisions/ADR-010-ci-cd-pipeline.md` | GitHub Actions + ArgoCD |

### 1.4 API Specifications (8 docs) - 🔴 NEW

| ID | Document | Description |
|----|----------|-------------|
| API-01 | `api/proto/metering.proto` | Metering gRPC service definition |
| API-02 | `api/proto/oversight.proto` | Oversight gRPC service definition |
| API-03 | `api/proto/runtime.proto` | Runtime gRPC service definition |
| API-04 | `api/proto/messaging.proto` | Messaging gRPC service definition |
| API-05 | `api/openapi/metering.yaml` | Metering REST API (OpenAPI 3.1) |
| API-06 | `api/openapi/oversight.yaml` | Oversight REST API (OpenAPI 3.1) |
| API-07 | `api/openapi/runtime.yaml` | Runtime REST API (OpenAPI 3.1) |
| API-08 | `api/openapi/messaging.yaml` | Messaging REST API (OpenAPI 3.1) |

### 1.5 Diagrams (7 docs) - 🔴 NEW

| ID | Document | Description |
|----|----------|-------------|
| DGM-01 | `diagrams/c4-context.md` | C4 System Context diagram |
| DGM-02 | `diagrams/c4-container.md` | C4 Container diagram |
| DGM-03 | `diagrams/component-metering.md` | Metering component diagram |
| DGM-04 | `diagrams/component-oversight.md` | Oversight component diagram |
| DGM-05 | `diagrams/component-runtime.md` | Runtime component diagram |
| DGM-06 | `diagrams/component-messaging.md` | Messaging component diagram |
| DGM-07 | `diagrams/data-flow-matrix.md` | Cross-product data flows |

---

## 2. Document Totals

| Category | Count | Status |
|----------|-------|--------|
| Core Platform SDDs | 10 | ✅ Complete |
| Per-Product SDDs | 32 | 🔴 New (4 exist as stubs) |
| ADRs | 10 | 🔴 New |
| API Specifications | 8 | 🔴 New |
| Diagrams | 7 | 🔴 New |
| **TOTAL** | **67** | **10 complete, 57 new** |

---

## 3. Hive Mind Orchestration Plan

### 3.1 Agent Roles & Skills

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         HIVE MIND AGENT TOPOLOGY                            │
│                                                                             │
│                           ┌─────────────────┐                               │
│                           │  QUEEN AGENT    │                               │
│                           │  (Coordinator)  │                               │
│                           │                 │                               │
│                           │ • Consensus     │                               │
│                           │ • Validation    │                               │
│                           │ • Alignment     │                               │
│                           └────────┬────────┘                               │
│                                    │                                        │
│            ┌───────────┬───────────┼───────────┬───────────┐               │
│            │           │           │           │           │               │
│            ▼           ▼           ▼           ▼           ▼               │
│   ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐
│   │  ARCHITECT  │ │  METERING   │ │  OVERSIGHT  │ │   RUNTIME   │ │ MESSAGING │
│   │   SWARM     │ │   SWARM     │ │   SWARM     │ │   SWARM     │ │   SWARM   │
│   │             │ │             │ │             │ │             │ │           │
│   │ • System    │ │ • Billing   │ │ • HITL      │ │ • Sandbox   │ │ • Crypto  │
│   │ • Security  │ │ • Quota     │ │ • Approval  │ │ • Isolation │ │ • E2E     │
│   │ • API       │ │ • Events    │ │ • Channels  │ │ • Warm Pool │ │ • Delivery│
│   └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └──────┬──────┘ └─────┬─────┘
│          │               │               │               │              │     │
│          └───────────────┴───────────────┴───────────────┴──────────────┘     │
│                                          │                                    │
│                                          ▼                                    │
│                              ┌─────────────────────┐                          │
│                              │   VALIDATOR SWARM   │                          │
│                              │                     │                          │
│                              │ • Cross-reference   │                          │
│                              │ • Consistency       │                          │
│                              │ • Security review   │                          │
│                              │ • API contracts     │                          │
│                              └─────────────────────┘                          │
│                                                                               │
└───────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Agent Specifications

| Agent Type | Count | Skills | Responsibilities |
|------------|-------|--------|------------------|
| **Queen Coordinator** | 1 | `queen-coordinator`, `collective-intelligence-coordinator` | Orchestration, consensus, conflict resolution |
| **Architect Swarm** | 3 | `system-architect`, `specification`, `architecture` | Core SDDs, cross-cutting design |
| **Metering Swarm** | 2 | `coder`, `analyst`, `backend-dev` | Metering product SDDs |
| **Oversight Swarm** | 2 | `coder`, `analyst`, `backend-dev` | Oversight product SDDs |
| **Runtime Swarm** | 2 | `coder`, `analyst`, `backend-dev` | Runtime product SDDs |
| **Messaging Swarm** | 2 | `security-manager`, `coder`, `analyst` | Messaging product SDDs |
| **Validator Swarm** | 3 | `reviewer`, `code-analyzer`, `tester` | Validation, consistency, alignment |
| **TOTAL** | **15** | | |

### 3.3 Consensus Mechanisms

```
┌─────────────────────────────────────────────────────────────────────┐
│                      CONSENSUS CHECKPOINTS                          │
│                                                                     │
│  PHASE 1: Requirements Alignment                                    │
│  ─────────────────────────────────                                  │
│  • All product swarms review CORE-01 (requirements)                 │
│  • Each swarm produces product-specific requirements                │
│  • CONSENSUS GATE: Requirements traceability matrix approved        │
│                                                                     │
│  PHASE 2: Architecture Consensus                                    │
│  ─────────────────────────────────                                  │
│  • Architect swarm proposes cross-product architecture              │
│  • Product swarms validate against their requirements               │
│  • CONSENSUS GATE: Interface contracts agreed                       │
│                                                                     │
│  PHASE 3: Parallel Product Development                              │
│  ─────────────────────────────────────                              │
│  • Each product swarm works independently                           │
│  • Shared memory for cross-references                               │
│  • VALIDATION GATE: Per-product internal consistency                │
│                                                                     │
│  PHASE 4: Cross-Product Validation                                  │
│  ─────────────────────────────────                                  │
│  • Validator swarm checks all 67 documents                          │
│  • Cross-reference verification                                     │
│  • CONSENSUS GATE: Full alignment confirmed                         │
│                                                                     │
│  PHASE 5: Final Review                                              │
│  ─────────────────────────────────                                  │
│  • Queen coordinator final approval                                 │
│  • All swarms sign off                                              │
│  • RELEASE GATE: SDD suite approved for implementation              │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. Validation Framework

### 4.1 Validation Rules

| Rule ID | Rule | Check Type |
|---------|------|------------|
| VAL-001 | Every requirement has traceability to architecture | Cross-reference |
| VAL-002 | Every API endpoint documented in both proto and OpenAPI | Consistency |
| VAL-003 | Security controls cover all threat model entries | Coverage |
| VAL-004 | Data models match between SDDs and API specs | Schema validation |
| VAL-005 | Performance targets consistent across documents | Value check |
| VAL-006 | Terminology consistent (glossary adherence) | Text analysis |
| VAL-007 | ADRs referenced where decisions are mentioned | Link validation |
| VAL-008 | Diagrams match textual descriptions | Visual-text alignment |
| VAL-009 | Test cases cover all requirements | Traceability |
| VAL-010 | Runbooks cover all operational scenarios | Completeness |

### 4.2 Validation Matrix

```
                    ┌─────┬─────┬─────┬─────┬─────┐
                    │CORE │ MTR │ OVS │ RTM │ MSG │
        ┌───────────┼─────┼─────┼─────┼─────┼─────┤
        │ CORE      │  ●  │  ◐  │  ◐  │  ◐  │  ◐  │
        ├───────────┼─────┼─────┼─────┼─────┼─────┤
        │ Metering  │  ◐  │  ●  │  ○  │  ○  │  ○  │
        ├───────────┼─────┼─────┼─────┼─────┼─────┤
        │ Oversight │  ◐  │  ◐  │  ●  │  ○  │  ○  │
        ├───────────┼─────┼─────┼─────┼─────┼─────┤
        │ Runtime   │  ◐  │  ○  │  ○  │  ●  │  ◐  │
        ├───────────┼─────┼─────┼─────┼─────┼─────┤
        │ Messaging │  ◐  │  ○  │  ○  │  ◐  │  ●  │
        └───────────┴─────┴─────┴─────┴─────┴─────┘

        ● = Full validation (internal consistency)
        ◐ = Cross-validation (interface alignment)
        ○ = Spot check (terminology, references)
```

### 4.3 Validation Checkpoints

| Checkpoint | Trigger | Validator | Blocking |
|------------|---------|-----------|----------|
| **V1: Requirements** | After Phase 1 | Architect Swarm | Yes |
| **V2: Architecture** | After Phase 2 | All Product Swarms | Yes |
| **V3: Product Internal** | Per product completion | Validator Swarm | No (flag issues) |
| **V4: Cross-Product** | After all products | Validator Swarm | Yes |
| **V5: Final** | Before release | Queen + All Swarms | Yes |

---

## 5. Execution Timeline

### 5.1 Phase Schedule

```
Week 1 ──────────────────────────────────────────────────────────────────
│ Day 1-2: Setup & Issue Creation
│ Day 3-5: Phase 1 - Requirements Alignment
│
Week 2 ──────────────────────────────────────────────────────────────────
│ Day 1-3: Phase 2 - Architecture Consensus
│ Day 4-5: Phase 3 Start - Product SDDs (parallel)
│
Week 3 ──────────────────────────────────────────────────────────────────
│ Day 1-5: Phase 3 Continue - Product SDDs (parallel)
│         • Metering: MTR-00 through MTR-07
│         • Oversight: OVS-00 through OVS-07
│         • Runtime: RTM-00 through RTM-07
│         • Messaging: MSG-00 through MSG-07
│
Week 4 ──────────────────────────────────────────────────────────────────
│ Day 1-2: Phase 3 Complete - ADRs & API Specs
│ Day 3-4: Phase 4 - Cross-Product Validation
│ Day 5: Phase 5 - Final Review & Approval
```

### 5.2 Parallel Execution Model

```
┌────────────────────────────────────────────────────────────────────────┐
│                    PARALLEL EXECUTION LANES                            │
│                                                                        │
│  Lane 1 (Architect)  ████████░░░░░░░░░░░░░░ ADRs, Diagrams             │
│  Lane 2 (Metering)   ░░░░████████████░░░░░░ MTR-00 → MTR-07            │
│  Lane 3 (Oversight)  ░░░░████████████░░░░░░ OVS-00 → OVS-07            │
│  Lane 4 (Runtime)    ░░░░████████████░░░░░░ RTM-00 → RTM-07            │
│  Lane 5 (Messaging)  ░░░░████████████░░░░░░ MSG-00 → MSG-07            │
│  Lane 6 (API Specs)  ░░░░░░░░████████░░░░░░ Proto + OpenAPI            │
│  Lane 7 (Validator)  ░░░░░░░░░░░░░░████████ V3, V4, V5                 │
│                      ─────────────────────────────────────             │
│                      W1    W2    W3    W4                              │
│                                                                        │
│  Legend: █ = Active work  ░ = Waiting/Dependencies                     │
└────────────────────────────────────────────────────────────────────────┘
```

---

## 6. GitHub Issues Structure

### 6.1 Issue Hierarchy

```
Epic: SDD Expansion
├── Milestone: Phase 1 - Requirements
│   ├── Issue: [CORE] Finalize requirements traceability
│   ├── Issue: [MTR] Metering requirements extraction
│   ├── Issue: [OVS] Oversight requirements extraction
│   ├── Issue: [RTM] Runtime requirements extraction
│   └── Issue: [MSG] Messaging requirements extraction
│
├── Milestone: Phase 2 - Architecture
│   ├── Issue: [ARCH] Cross-product interface contracts
│   ├── Issue: [ARCH] Dependency graph validation
│   └── Issue: [ARCH] Integration point specifications
│
├── Milestone: Phase 3 - Product SDDs
│   ├── Issue: [MTR-00] Metering Overview
│   ├── Issue: [MTR-01] Metering Requirements
│   ├── ... (32 product SDD issues)
│   └── Issue: [MSG-07] Messaging Runbook
│
├── Milestone: Phase 4 - Cross-Cutting
│   ├── Issue: [ADR-001] Hybrid Signatures Decision
│   ├── ... (10 ADR issues)
│   ├── Issue: [API-01] Metering Proto Spec
│   ├── ... (8 API spec issues)
│   ├── Issue: [DGM-01] C4 Context Diagram
│   └── ... (7 diagram issues)
│
└── Milestone: Phase 5 - Validation
    ├── Issue: [VAL] Cross-reference validation
    ├── Issue: [VAL] Terminology consistency
    ├── Issue: [VAL] Security coverage verification
    └── Issue: [VAL] Final sign-off
```

### 6.2 Issue Labels

| Label | Color | Purpose |
|-------|-------|---------|
| `sdd` | Blue | All SDD-related issues |
| `core` | Purple | Core platform SDDs |
| `metering` | Green | Metering product |
| `oversight` | Orange | Oversight product |
| `runtime` | Red | Runtime product |
| `messaging` | Cyan | Messaging product |
| `adr` | Yellow | Architecture decisions |
| `api-spec` | Pink | API specifications |
| `diagram` | Gray | Visual documentation |
| `validation` | Black | Validation tasks |
| `consensus-required` | Gold | Needs multi-swarm agreement |
| `blocking` | Dark Red | Blocks other work |

### 6.3 Issue Count Summary

| Category | Issues |
|----------|--------|
| Phase 1 - Requirements | 5 |
| Phase 2 - Architecture | 3 |
| Phase 3 - Product SDDs | 32 |
| Phase 3 - ADRs | 10 |
| Phase 3 - API Specs | 8 |
| Phase 3 - Diagrams | 7 |
| Phase 5 - Validation | 4 |
| **TOTAL ISSUES** | **69** |

---

## 7. Memory & Coordination Strategy

### 7.1 Shared Memory Namespaces

```
memory/
├── sdd/
│   ├── requirements/          # Extracted requirements per product
│   ├── interfaces/            # Agreed interface contracts
│   ├── terminology/           # Glossary and term definitions
│   ├── decisions/             # ADR summaries for quick reference
│   └── validation/            # Validation results and issues
│
├── swarm/
│   ├── metering/              # Metering swarm working memory
│   ├── oversight/             # Oversight swarm working memory
│   ├── runtime/               # Runtime swarm working memory
│   ├── messaging/             # Messaging swarm working memory
│   └── validator/             # Validator swarm findings
│
└── consensus/
    ├── checkpoints/           # Consensus gate results
    ├── conflicts/             # Unresolved conflicts
    └── resolutions/           # Resolved conflicts with rationale
```

### 7.2 Cross-Swarm Communication

| Event | Producer | Consumers | Channel |
|-------|----------|-----------|---------|
| Requirement extracted | Product swarm | Architect, Validator | `sdd/requirements/{product}` |
| Interface proposed | Architect | All product swarms | `sdd/interfaces/{interface}` |
| Conflict detected | Validator | Queen, affected swarms | `consensus/conflicts/{id}` |
| Consensus reached | Queen | All swarms | `consensus/checkpoints/{phase}` |
| Validation complete | Validator | Queen | `sdd/validation/{phase}` |

---

## 8. Risk Mitigation

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Swarm divergence | Medium | High | Frequent consensus checkpoints |
| Terminology drift | High | Medium | Shared glossary, validator checks |
| Interface conflicts | Medium | High | Early architecture consensus |
| Scope creep | Medium | Medium | Fixed document templates |
| Validation bottleneck | Low | High | Parallel validation lanes |

---

## 9. Success Criteria

| Criterion | Metric | Target |
|-----------|--------|--------|
| Document completion | Issues closed | 69/69 |
| Validation pass rate | Rules passed | ≥95% |
| Cross-reference integrity | Broken links | 0 |
| Consensus gates passed | Gates cleared | 5/5 |
| Timeline adherence | Days elapsed | ≤20 |

---

## 10. Approval

**Plan Status:** PENDING REVIEW

**Required Approvals:**
- [ ] User review and approval
- [ ] Resource allocation confirmed
- [ ] Timeline accepted

**Next Steps (after approval):**
1. Create all 69 GitHub issues with proper labels and milestones
2. Initialize hive mind swarm topology
3. Begin Phase 1 execution

---

*This plan was generated for review. Execution will begin upon approval.*
