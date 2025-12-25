---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
---

# SDD-09: Implementation Plan

## Purpose

This document defines the phased implementation plan for the Enablement Layer, including milestones, dependencies, timelines, and success criteria.

## Scope

**In Scope:**
- Implementation phases and milestones
- Dependency mapping
- Resource requirements
- Risk mitigation
- Success criteria

**Out of Scope:**
- Detailed sprint planning
- Individual task assignments
- Budget allocation

---

## 1. Implementation Strategy

### 1.1 Principles

| Principle | Implementation |
|-----------|----------------|
| **Incremental delivery** | Ship working software in each phase |
| **Dependency-first** | Build foundations before products |
| **Vertical slices** | Complete features end-to-end |
| **Integration early** | Connect products as they're built |
| **Production-ready** | Each phase deployable to production |

### 1.2 High-Level Timeline

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         IMPLEMENTATION ROADMAP                           │
│                                                                          │
│  Phase 0      Phase 1       Phase 2       Phase 3       Phase 4         │
│  Foundation   Metering      Oversight     Runtime       Messaging       │
│  ──────────   ──────────    ──────────    ──────────    ──────────      │
│  Weeks 1-2    Weeks 3-6     Weeks 7-10    Weeks 11-14   Weeks 15-18     │
│                                                                          │
│  ┌─────┐      ┌─────┐       ┌─────┐       ┌─────┐       ┌─────┐         │
│  │Setup│─────►│Core │──────►│HITL │──────►│Sandbox│────►│E2E  │         │
│  │Infra│      │Bill │       │Flow │       │Exec  │      │Msg  │         │
│  └─────┘      └─────┘       └─────┘       └─────┘       └─────┘         │
│                                                                          │
│                    Phase 5: Integration & Hardening (Weeks 19-22)       │
│                    Phase 6: Production Release (Weeks 23-24)            │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. Phase 0: Foundation (Weeks 1-2)

### 2.1 Objectives

- Repository setup with workspace structure
- CI/CD pipeline operational
- Development environment documented
- Platform layer integration contracts defined

### 2.2 Deliverables

| Deliverable | Description | Owner |
|-------------|-------------|-------|
| Repository structure | Cargo workspace with all crates stubbed | Architect |
| CI pipeline | Build, test, lint on every PR | DevOps |
| CD pipeline | Automated deployment to dev environment | DevOps |
| Platform contracts | Trait definitions for NHI, AuthZ, Crypto, Audit | Architect |
| Developer docs | Setup guide, contribution guidelines | Tech Lead |

### 2.3 Tasks

```
□ Create Cargo workspace with crate stubs
□ Configure GitHub Actions (build, test, lint)
□ Set up Kubernetes dev cluster
□ Define platform integration traits
□ Create mock implementations of platform services
□ Document development environment setup
□ Establish code review guidelines
□ Set up observability stack (dev)
```

### 2.4 Success Criteria

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace` passes
- [ ] PR workflow with required checks functional
- [ ] Deployment to dev cluster automated
- [ ] Platform trait interfaces documented and reviewed

### 2.5 Dependencies

| Dependency | Type | Status |
|------------|------|--------|
| creto-nhi mock | Internal | Required |
| creto-authz mock | Internal | Required |
| creto-crypto mock | Internal | Required |
| creto-audit mock | Internal | Required |

---

## 3. Phase 1: Metering (Weeks 3-6)

### 3.1 Objectives

- Event ingestion pipeline operational
- Quota enforcement with sub-10µs cache hits
- Basic aggregation working
- Integration with AuthZ for quota policies

### 3.2 Milestone Breakdown

#### Week 3-4: Core Event Pipeline

```
□ BillableEvent schema with validation
□ Event ingestion gRPC service
□ Redis-backed event deduplication
□ Basic PostgreSQL event storage
□ Idempotency key handling
□ Unit tests for event validation
```

#### Week 5: Quota Enforcement

```
□ QuotaEnforcer with bloom filter
□ Redis fallback for cache misses
□ Reservation system for pre-allocation
□ AuthZ integration for quota policies
□ Quota check benchmarks (<10µs target)
```

#### Week 6: Aggregation & Billing

```
□ Aggregation types (COUNT, SUM, UNIQUE_COUNT, MAX)
□ Time-based aggregation windows
□ Pricing model evaluation
□ BillingService interface
□ Integration tests with all components
```

### 3.3 Deliverables

| Deliverable | Acceptance Criteria |
|-------------|---------------------|
| Event ingestion service | >10K events/sec sustained |
| Quota enforcement | P99 <10µs for cache hits |
| Aggregation engine | All Lago aggregation types |
| Metering SDK (Rust) | Ergonomic client library |

### 3.4 Success Criteria

- [ ] Event ingestion benchmark: >10K/sec
- [ ] Quota check benchmark: P99 <10µs (cache hit)
- [ ] Test coverage: ≥80%
- [ ] Integration with mock AuthZ working
- [ ] Deployed to dev environment

---

## 4. Phase 2: Oversight (Weeks 7-10)

### 4.1 Objectives

- Policy definition and matching
- Approval request lifecycle
- Multi-channel notifications (Slack, email)
- Integration with Metering for cost-based policies

### 4.2 Milestone Breakdown

#### Week 7-8: Policy Engine

```
□ OversightPolicy data model
□ Policy matching engine
□ Policy storage (PostgreSQL)
□ Policy CRUD gRPC service
□ Trigger evaluation (cost, action, resource)
□ Unit tests for policy matching
```

#### Week 9: Approval Workflow

```
□ OversightRequest state machine
□ Request creation and tracking
□ Timeout handling
□ Escalation chain
□ Approval/denial processing
□ Integration with audit trail
```

#### Week 10: Notification Channels

```
□ NotificationChannel trait
□ Slack channel implementation
□ Email channel implementation
□ Webhook channel implementation
□ Channel routing logic
□ End-to-end integration tests
```

### 4.3 Deliverables

| Deliverable | Acceptance Criteria |
|-------------|---------------------|
| Policy engine | Match policies in <1ms |
| Approval workflow | Full state machine operational |
| Slack integration | Approval buttons work |
| Email integration | Signed approval links |

### 4.4 Success Criteria

- [ ] Policy matching benchmark: <1ms
- [ ] State machine: All transitions tested
- [ ] Slack notifications: Button interactions work
- [ ] Integration with Metering: Cost-based triggers
- [ ] Test coverage: ≥80%

---

## 5. Phase 3: Runtime (Weeks 11-14)

### 5.1 Objectives

- Sandbox lifecycle management
- Warm pool for fast allocation
- Network egress control
- Attestation integration

### 5.2 Milestone Breakdown

#### Week 11-12: Sandbox Lifecycle

```
□ SandboxSpec data model
□ SandboxController state machine
□ gVisor runtime integration
□ Sandbox creation/deletion
□ Resource limits enforcement
□ Basic sandbox lifecycle tests
```

#### Week 13: Warm Pool

```
□ WarmPoolManager implementation
□ Pre-warming based on demand
□ Sandbox claiming (<100ms target)
□ Pool sizing algorithms
□ Pool health monitoring
□ Warm pool benchmarks
```

#### Week 14: Security Controls

```
□ Network policy integration
□ Egress AuthZ checks
□ Attestation verification
□ NHI binding to sandbox
□ Syscall filtering validation
□ Security integration tests
```

### 5.3 Deliverables

| Deliverable | Acceptance Criteria |
|-------------|---------------------|
| Sandbox controller | gVisor sandboxes work |
| Warm pool | Claim in <100ms |
| Egress control | All egress goes through AuthZ |
| Attestation | Runtime attestation verified |

### 5.4 Success Criteria

- [ ] Cold sandbox spawn: <2s (gVisor)
- [ ] Warm pool claim: <100ms
- [ ] Egress control: No bypass possible
- [ ] Attestation: Verified on every spawn
- [ ] Test coverage: ≥80%

---

## 6. Phase 4: Messaging (Weeks 15-18)

### 6.1 Objectives

- End-to-end encrypted messaging
- Hybrid cryptography (classical + PQ)
- Multi-channel delivery
- AuthZ-gated delivery

### 6.2 Milestone Breakdown

#### Week 15-16: Encryption Layer

```
□ MessageEnvelope data model
□ Hybrid key encapsulation (X25519 + ML-KEM)
□ Symmetric encryption (AES-256-GCM)
□ Hybrid signatures (Ed25519 + ML-DSA)
□ Encryption/decryption service
□ Crypto benchmarks (>100K msg/sec)
```

#### Week 17: Delivery Pipeline

```
□ DeliveryPolicy data model
□ Inbox storage (PostgreSQL)
□ Priority-based delivery
□ Expiration handling
□ Delivery confirmation
□ Delivery pipeline tests
```

#### Week 18: Channel Integration

```
□ AuthZ integration for delivery
□ Agent-to-agent messaging
□ Human-to-agent messaging
□ Broadcast capabilities
□ End-to-end integration tests
```

### 6.3 Deliverables

| Deliverable | Acceptance Criteria |
|-------------|---------------------|
| E2E encryption | Hybrid ML-KEM + AES-256-GCM |
| Message delivery | Priority-based, confirmed |
| AuthZ integration | All delivery gated |
| Messaging SDK | Ergonomic client library |

### 6.4 Success Criteria

- [ ] Encryption benchmark: >100K msg/sec
- [ ] Delivery auth: <1ms (AuthZ check)
- [ ] E2E encryption: Both classical and PQ
- [ ] Test coverage: ≥80%

---

## 7. Phase 5: Integration & Hardening (Weeks 19-22)

### 7.1 Objectives

- Cross-product integration testing
- Performance optimization
- Security hardening
- Documentation completion

### 7.2 Focus Areas

#### Week 19-20: Integration

```
□ Cross-product E2E tests
□ Metering + Oversight integration
□ Runtime + Messaging integration
□ Full workflow tests
□ Load testing (all products)
□ Chaos engineering
```

#### Week 21-22: Hardening

```
□ Performance profiling and optimization
□ Security audit (internal)
□ Fuzz testing on all parsers
□ API documentation completion
□ Runbook creation
□ Incident response procedures
```

### 7.3 Deliverables

| Deliverable | Acceptance Criteria |
|-------------|---------------------|
| E2E test suite | Complete workflow coverage |
| Load test results | All targets met |
| Security report | No critical issues |
| Documentation | Complete API docs, runbooks |

### 7.4 Success Criteria

- [ ] E2E tests: All cross-product workflows
- [ ] Load tests: Sustain 10K events/sec
- [ ] Security: No critical/high vulnerabilities
- [ ] Documentation: 100% API coverage

---

## 8. Phase 6: Production Release (Weeks 23-24)

### 8.1 Objectives

- Production deployment
- Monitoring and alerting operational
- Customer pilot program
- Post-launch stabilization

### 8.2 Activities

```
□ Production infrastructure provisioning
□ Secret management setup
□ Monitoring dashboards deployed
□ Alert rules configured
□ Rollback procedures tested
□ Pilot customer onboarding
□ 24/7 support rotation established
□ Post-launch monitoring (2 weeks)
```

### 8.3 Release Criteria

- [ ] All Phase 5 success criteria met
- [ ] Production infrastructure operational
- [ ] Monitoring/alerting configured
- [ ] Runbooks reviewed and approved
- [ ] Rollback tested successfully
- [ ] Security sign-off obtained

---

## 9. Dependency Graph

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        IMPLEMENTATION DEPENDENCIES                       │
│                                                                          │
│  Platform Layer (External)                                               │
│  ────────────────────────                                                │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐                        │
│  │creto-nhi│ │creto-   │ │creto-   │ │creto-   │                        │
│  │         │ │authz    │ │crypto   │ │audit    │                        │
│  └────┬────┘ └────┬────┘ └────┬────┘ └────┬────┘                        │
│       │           │           │           │                              │
│       └───────────┼───────────┼───────────┘                              │
│                   │           │                                          │
│                   ▼           ▼                                          │
│  ┌────────────────────────────────────────────┐                         │
│  │         creto-enablement-common             │                         │
│  │    (Shared types, platform integration)     │                         │
│  └───────────────────┬────────────────────────┘                         │
│                      │                                                   │
│       ┌──────────────┼──────────────┬──────────────┐                    │
│       │              │              │              │                     │
│       ▼              ▼              ▼              ▼                     │
│  ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐                  │
│  │Metering │   │Oversight│   │ Runtime │   │Messaging│                  │
│  │(Phase 1)│──►│(Phase 2)│   │(Phase 3)│   │(Phase 4)│                  │
│  └─────────┘   └────┬────┘   └────┬────┘   └────┬────┘                  │
│       │             │             │             │                        │
│       └─────────────┴─────────────┴─────────────┘                        │
│                            │                                             │
│                            ▼                                             │
│               ┌────────────────────────┐                                 │
│               │    Integration Tests   │                                 │
│               │       (Phase 5)        │                                 │
│               └────────────────────────┘                                 │
└─────────────────────────────────────────────────────────────────────────┘
```

### 9.1 Critical Path

1. **Platform layer mocks** → Foundation complete
2. **creto-enablement-common** → All products can start
3. **Metering quota enforcement** → Oversight cost triggers
4. **Runtime sandbox lifecycle** → Messaging in sandboxes

### 9.2 External Dependencies

| Dependency | Needed By | Risk | Mitigation |
|------------|-----------|------|------------|
| creto-nhi | Phase 0 | Medium | Mock implementation |
| creto-authz | Phase 1 | High | Mock with defined interface |
| creto-crypto | Phase 1 | Medium | Use ring as fallback |
| Kubernetes | Phase 0 | Low | Use Kind for dev |
| Redis | Phase 1 | Low | Standard deployment |
| PostgreSQL | Phase 1 | Low | Standard deployment |

---

## 10. Risk Management

### 10.1 Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Platform layer delays | Medium | High | Mock interfaces, parallel development |
| gVisor performance issues | Low | Medium | Kata containers as backup |
| PQ crypto integration complexity | Medium | Medium | Start with classical, add PQ incrementally |
| Cross-product integration issues | Medium | Medium | Early integration testing |

### 10.2 Schedule Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Underestimated complexity | Medium | Medium | Buffer in each phase |
| Resource unavailability | Low | High | Cross-training, documentation |
| Scope creep | Medium | Medium | Strict change control |
| External dependency delays | Medium | Medium | Mock implementations |

### 10.3 Risk Response Plan

```
If platform layer delayed > 2 weeks:
  → Extend mock implementations
  → Adjust Phase 5 integration timeline
  → Consider parallel workstreams

If performance targets not met:
  → Add optimization sprint
  → Consider architecture changes
  → Engage performance specialists

If security issues found:
  → Prioritize fixes over features
  → Extend hardening phase
  → Engage security team
```

---

## 11. Resource Requirements

### 11.1 Team Structure

| Role | Count | Phase Allocation |
|------|-------|------------------|
| **Architect** | 1 | All phases |
| **Senior Engineers** | 3 | 1 per product track |
| **Engineers** | 4 | Distributed across tracks |
| **DevOps** | 1 | Phases 0, 5, 6 |
| **Security** | 1 | Phases 0, 5, 6 |
| **Tech Lead** | 1 | All phases |

### 11.2 Infrastructure

| Resource | Purpose | Environment |
|----------|---------|-------------|
| Kubernetes cluster | Deployment | Dev, Staging, Prod |
| PostgreSQL | Data storage | All |
| Redis | Caching | All |
| CI/CD runners | Build/test | GitHub Actions |
| Observability stack | Monitoring | All |

---

## 12. Success Metrics

### 12.1 Delivery Metrics

| Metric | Target |
|--------|--------|
| On-time delivery | ≥90% of milestones |
| Scope completion | ≥95% of committed features |
| Quality | <5 critical bugs post-release |
| Test coverage | ≥80% across all products |

### 12.2 Performance Metrics

| Metric | Target | Phase |
|--------|--------|-------|
| Quota check latency | P99 <10µs | Phase 1 |
| Event ingestion | >10K/sec | Phase 1 |
| Oversight state change | <1ms | Phase 2 |
| Sandbox claim | <100ms | Phase 3 |
| Message encryption | >100K/sec | Phase 4 |

### 12.3 Operational Metrics

| Metric | Target |
|--------|--------|
| Availability | 99.9% |
| MTTR | <30 minutes |
| Deployment frequency | Weekly |
| Change failure rate | <5% |

---

## 13. Decision Log

| Date | Decision | Rationale |
|------|----------|-----------|
| TBD | Phase order (M→O→R→Msg) | Metering is foundation for cost-based oversight |
| TBD | 6-month timeline | Balance between speed and quality |
| TBD | Mock platform layer | Enable parallel development |

---

## 14. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
