# Creto Enablement Layer - 20-Week Implementation Timeline

## Executive Summary

This timeline orchestrates a parallel SPARC-based implementation across 4 core products (Metering, Oversight, Runtime, Messaging) with 4 investor-facing demos, culminating in a production-ready v1.0.0 release.

**Key Milestones**:
- **Week 4**: All contracts finalized, CI/CD operational
- **Week 8**: Metering MVP (85% coverage)
- **Week 10**: Oversight MVP (85% coverage)
- **Week 12**: Runtime MVP (80% coverage)
- **Week 14**: Messaging MVP (90% coverage)
- **Week 16**: All 4 demos alpha-ready
- **Week 20**: v1.0.0 Release

---

## 1. 20-Week Master Timeline

### **Week 1: SPARC-P Kickoff - Skeleton Generation**

**Phase**: Planning & Preparation
**Team**: 1 Planner + 4 Coders
**Parallel Streams**: 4 (one per product)

#### Objectives
Generate compilable code skeletons with zero implementations but complete API surfaces.

#### Tasks
- [ ] Initialize Cargo workspace with 5 crates:
  - `creto-metering`
  - `creto-oversight`
  - `creto-runtime`
  - `creto-messaging`
  - `creto-common`
- [ ] Generate Rust structs from SDD specifications:
  - Metering: `UsageEvent`, `AggregationEngine`, `PricingModel`
  - Oversight: `Request`, `StateMachine`, `ApprovalEngine`
  - Runtime: `Sandbox`, `WarmPool`, `SecretInjector`
  - Messaging: `X3DH`, `DoubleRatchet`, `MessageEnvelope`
- [ ] Define all trait signatures (no implementations):
  - `EventIngestion`, `PolicyEnforcement`, `SecureMessaging`, etc.
- [ ] Setup CI/CD pipeline skeleton:
  - GitHub Actions workflows for `cargo check`
  - Placeholder jobs for tests, lints, benchmarks

#### Deliverables
✅ **Exit Criteria**:
- `cargo check` passes for all 5 crates
- All public APIs defined in `.rs` files (functions return `todo!()`)
- Dependency graph validated (no circular deps)

#### Risks
⚠️ **Blocker**: Ambiguous SDD specifications → Mitigation: Daily sync with architect

---

### **Week 2: SPARC-P Complete - Contract Finalization**

**Phase**: Planning & Preparation
**Team**: 1 Architect + 3 Coders + 1 Security Engineer

#### Objectives
Solidify all external contracts (policies, schemas, APIs) to enable parallel development.

#### Tasks
- [ ] **Cedar Policy Templates** (`policies/`):
  - Metering quota enforcement policies
  - Oversight approval quorum policies
  - Runtime network egress policies
  - Messaging NHI authorization policies
- [ ] **SQL Migration Scripts** (`migrations/`):
  - PostgreSQL schemas for all 4 products
  - Idempotency constraints (e.g., `UNIQUE(transaction_id)`)
  - Audit log tables with JSONB fields
- [ ] **gRPC Proto Definitions** (`proto/`):
  - Inter-product communication contracts
  - Request/response message types
  - Streaming interfaces (e.g., real-time metering events)
- [ ] **OpenAPI Specifications** (`openapi/`):
  - REST API contracts for external clients
  - Webhook payload schemas (Oversight notifications)
- [ ] **Mock Implementations**:
  - AuthZ mock (Cedar evaluation simulator)
  - NHI mock (key bundle provider)
  - Vault mock (secret storage)

#### Deliverables
✅ **Exit Criteria**:
- All `.proto`, `.cedar`, `.sql`, `.yaml` files validated
- Mock test harness passing (`cargo test --all-features`)
- Contract documentation generated (protoc-gen-doc)

#### Risks
⚠️ **Dependency Conflicts**: Cedar policy language version mismatches → Use locked toolchain

---

### **Week 3: SPARC-A Infrastructure - Build & Test Automation**

**Phase**: Architecture
**Team**: 1 DevOps Engineer + 2 Coders + 1 QA Lead

#### Objectives
Establish automated quality gates for continuous validation.

#### Tasks
- [ ] **GitHub Actions Workflows** (`.github/workflows/`):
  - `ci.yml`: Run `cargo test`, `clippy`, `fmt` on every PR
  - `coverage.yml`: Generate coverage reports with `tarpaulin` (upload to Codecov)
  - `benchmark.yml`: Run `criterion` benchmarks on main branch merges
  - `docs.yml`: Build and deploy `rustdoc` to GitHub Pages
- [ ] **Code Coverage Tooling**:
  - Configure `cargo-tarpaulin` with 85% threshold for metering/oversight
  - Exclude generated code (e.g., `proto/` builds) from coverage
- [ ] **Benchmark Framework**:
  - Setup `criterion` for critical paths:
    - Metering event ingestion throughput
    - Oversight state machine transitions
    - Runtime sandbox startup latency
    - Messaging encryption/decryption ops
- [ ] **Integration Test Harness**:
  - Docker Compose setup for PostgreSQL, Cedar AuthZ, mock NHI
  - Testcontainers integration for isolated tests
- [ ] **Documentation Generation**:
  - `cargo doc --no-deps --open` setup
  - README templates with usage examples

#### Deliverables
✅ **Exit Criteria**:
- CI/CD pipeline green on main branch
- Coverage reports accessible at codecov.io
- Benchmark baselines recorded

#### Risks
⚠️ **Flaky Tests**: Network-dependent integration tests → Use `testcontainers` with retries

---

### **Week 4: SPARC-A Integration - Cross-Crate Orchestration**

**Phase**: Architecture
**Team**: 1 System Architect + 3 Backend Engineers

#### Objectives
Enable seamless inter-product communication and shared infrastructure.

#### Tasks
- [ ] **Dependency Injection Setup**:
  - Implement `creto-common::DI` registry for service locators
  - Use trait objects for runtime polymorphism (e.g., `Box<dyn EventIngestion>`)
- [ ] **Configuration Management**:
  - `config.toml` schema with environment overrides
  - Use `figment` crate for layered config (default → file → env vars)
- [ ] **Logging/Tracing Infrastructure**:
  - `tracing-subscriber` with JSON output for production
  - Structured logs with `request_id`, `product`, `operation` fields
  - OpenTelemetry exporter for distributed tracing
- [ ] **Error Handling Patterns**:
  - Define `creto-common::Error` enum with `thiserror`
  - Propagate errors across boundaries (e.g., gRPC status codes)
- [ ] **Feature Flag System**:
  - `creto-common::features` module for runtime toggles
  - Use LaunchDarkly SDK for A/B testing demos

#### Deliverables
✅ **Exit Criteria**:
- Integration tests scaffold passing (`tests/integration/`)
- All crates can import `creto-common` without circular deps
- Config hot-reload working in dev mode

#### Risks
⚠️ **Config Drift**: Prod/dev config mismatches → Use schema validation (JSON Schema)

---

## **Weeks 5-8: SPARC-R1 Metering Implementation (TDD)**

**Phase**: Refinement (Test-Driven Development)
**Team**: 1 Tech Lead + 4 Backend Engineers + 1 QA Engineer
**Target Coverage**: 85%

### **Week 5: Event Ingestion Pipeline**

#### Objectives
Accept, validate, and persist usage events with idempotency guarantees.

#### Tasks
- [ ] **Event Ingestion API** (TDD):
  1. Write test: `test_ingest_duplicates_idempotent()`
  2. Implement: `EventIngester::ingest(event: UsageEvent) -> Result<(), MeteringError>`
  3. Assert: Duplicate `transaction_id` returns `Ok(())` without double-counting
- [ ] **Event Schema Validation**:
  - Use `serde_valid` for JSON schema enforcement
  - Test invalid events (missing fields, wrong types)
- [ ] **Postgres Storage**:
  - Insert into `usage_events` table with `ON CONFLICT (transaction_id) DO NOTHING`
  - Unit test: Verify row count unchanged on duplicate

#### Unit Tests (17 tests)
```rust
#[test] fn test_ingest_valid_event() { ... }
#[test] fn test_ingest_duplicate_transaction_id() { ... }
#[test] fn test_ingest_invalid_schema() { ... }
#[test] fn test_ingest_all_event_types() { ... } // 13 event types from SDD
```

#### Deliverables
✅ **Exit Criteria**:
- 100% of ingestion tests passing
- 90% coverage for `event_ingester.rs`

---

### **Week 6: Aggregation Engine & Pricing**

#### Objectives
Calculate usage totals and apply pricing models.

#### Tasks
- [ ] **Aggregation Functions** (TDD):
  - `COUNT(event_type)`: Test with 1M events, verify correctness
  - `SUM(quantity)`: Test decimal precision (no floating-point errors)
  - `MAX(timestamp)`: Test across time zones
  - `UNIQUE(agent_id)`: Test with 100K unique agents
- [ ] **Pricing Model Integration**:
  - `PricingEngine::calculate(usage: &Aggregation) -> Money`
  - Test tiered pricing (e.g., $0.01/request for first 10K, $0.005 after)
  - Property test: Total cost never negative

#### Property Tests (using `proptest`)
```rust
proptest! {
    #[test]
    fn aggregation_monotonic(events in prop::collection::vec(any::<UsageEvent>(), 0..1000)) {
        let total = aggregate(&events);
        assert!(total >= 0);
    }
}
```

#### Deliverables
✅ **Exit Criteria**:
- All aggregation types tested with 10K+ events
- 85% coverage for `aggregation.rs`

---

### **Week 7: Quota Enforcement Integration**

#### Objectives
Block operations when quotas exceeded; integrate with AuthZ.

#### Tasks
- [ ] **Real-Time Budget Tracking**:
  - Maintain in-memory cache of current usage (Redis-backed)
  - Test race conditions: Concurrent requests at quota boundary
- [ ] **AuthZ Policy Integration**:
  - Call Cedar `isAuthorized()` with current usage context
  - Test policy: `DENY IF usage > quota`
- [ ] **Integration Tests**:
  - Scenario: Agent exceeds monthly quota → Next request returns `403 Forbidden`
  - Scenario: Quota resets at month boundary → Requests allowed again

#### Integration Tests (5 scenarios)
```rust
#[tokio::test]
async fn test_quota_exceeded_blocks_request() {
    // Setup: Agent with 100 request quota
    // Action: Send 101 requests
    // Assert: 101st returns MeteringError::QuotaExceeded
}
```

#### Deliverables
✅ **Exit Criteria**:
- Integration tests passing with mock AuthZ
- 80% coverage for `quota_enforcer.rs`

---

### **Week 8: Invoice Generation & Cross-Product Attribution**

#### Objectives
Generate accurate invoices; attribute costs across products.

#### Tasks
- [ ] **Invoice Generator**:
  - Aggregate usage across all products for a billing period
  - Test: Multi-product invoice includes Metering + Runtime costs
- [ ] **Cross-Product Attribution**:
  - Track which agent used which products (via `agent_id` foreign key)
  - Test: Agent uses 3 products → Invoice line items for each
- [ ] **Coverage Gate**:
  - Run `cargo tarpaulin` → Fail CI if <85%
  - Add missing tests for error paths

#### Deliverables
✅ **Exit Criteria**:
- **Metering MVP**: All acceptance tests passing
- **85% test coverage** (enforced by CI)
- Invoice generation working end-to-end

---

## **Weeks 7-10: SPARC-R1 Oversight Implementation (TDD)**

**Phase**: Refinement (Overlaps with Metering Week 7-8)
**Team**: 1 Tech Lead + 3 Backend Engineers + 1 Workflow Specialist
**Target Coverage**: 85%

### **Week 7: State Machine Core**

#### Objectives
Implement request lifecycle state transitions.

#### Tasks
- [ ] **State Machine** (TDD):
  - States: `Pending → Approved/Rejected/Escalated → Completed`
  - Test invalid transitions (e.g., `Completed → Pending` should fail)
- [ ] **Request Storage**:
  - Postgres table with JSONB `context` field
  - Test: Store/retrieve complex context (1MB JSON)

#### Unit Tests (12 tests)
```rust
#[test] fn test_state_transition_pending_to_approved() { ... }
#[test] fn test_invalid_transition_rejected_to_approved() { ... }
```

#### Deliverables
✅ **Exit Criteria**:
- All state transitions tested
- 90% coverage for `state_machine.rs`

---

### **Week 8: Approval Workflow Engine**

#### Objectives
Calculate quorum, route to approvers, enforce timeouts.

#### Tasks
- [ ] **Quorum Calculation** (TDD):
  - Test: 3-of-5 approvers with weights → Needs 60% weighted approval
- [ ] **Timeout Scheduling**:
  - Use `tokio::time::sleep` for async timeouts
  - Test: Request auto-escalates after 24 hours

#### Deliverables
✅ **Exit Criteria**:
- Quorum logic tested with 20 scenarios
- 85% coverage for `approval_engine.rs`

---

### **Week 9: Channel Adapters & Policy Triggers**

#### Objectives
Notify approvers via Slack/email; trigger oversight on policy violations.

#### Tasks
- [ ] **Channel Adapters**:
  - Slack: Send message with approve/reject buttons
  - Email: Send link to approval dashboard
  - Test: Mock Slack API, verify payload
- [ ] **Policy Trigger Integration**:
  - AuthZ policy: `ESCALATE IF amount > $10,000`
  - Test: High-value request automatically creates oversight request

#### Integration Tests (3 channels × 2 actions = 6 tests)
```rust
#[tokio::test]
async fn test_slack_approval_button_click() { ... }
```

#### Deliverables
✅ **Exit Criteria**:
- All 3 channels tested
- 80% coverage for `channels.rs`

---

### **Week 10: Memory Context & Durability**

#### Objectives
Integrate Claude memory for context; ensure checkpoint/resume works.

#### Tasks
- [ ] **Memory Integration**:
  - Store conversation context in Claude memory
  - Test: Resume request after system restart with full context
- [ ] **Checkpoint/Restore**:
  - Persist state machine snapshots to Postgres
  - Test: Server crash → Resume from last checkpoint

#### Deliverables
✅ **Exit Criteria**:
- **Oversight MVP**: End-to-end approval flow working
- **85% coverage gate** passed

---

## **Weeks 9-12: SPARC-R1 Runtime Implementation (TDD)**

**Phase**: Refinement (Overlaps with Oversight Week 9-10)
**Team**: 1 Tech Lead + 4 Systems Engineers + 1 Security Engineer
**Target Coverage**: 80%

### **Week 9: Sandbox Lifecycle Management**

#### Objectives
Create, start, stop, and destroy gVisor sandboxes.

#### Tasks
- [ ] **Sandbox Controller** (TDD):
  - Test: Create sandbox → Status is `Created`
  - Test: Start sandbox → Process runs, returns PID
  - Test: Stop sandbox → Process killed within 5s
- [ ] **gVisor Runtime Adapter**:
  - Use `runsc` CLI wrapper
  - Test: Invalid config → Returns error

#### Unit Tests (10 tests)
```rust
#[test] fn test_create_sandbox_with_valid_config() { ... }
#[test] fn test_start_sandbox_returns_pid() { ... }
```

#### Deliverables
✅ **Exit Criteria**:
- Sandbox lifecycle tested
- 85% coverage for `sandbox_controller.rs`

---

### **Week 10: Warm Pool & Network Enforcement**

#### Objectives
Pre-warm sandboxes for low latency; enforce egress policies.

#### Tasks
- [ ] **Warm Pool Engine**:
  - Maintain pool of 10 pre-started sandboxes
  - Test: Pool replenishes after sandbox allocation
- [ ] **Network Egress Enforcement**:
  - AuthZ policy: `ALLOW egress ONLY TO approved_domains`
  - Test: Request to unapproved domain → Blocked by iptables

#### Integration Tests (5 scenarios)
```rust
#[tokio::test]
async fn test_warm_pool_allocation_latency() {
    // Assert: <100ms to get warm sandbox
}
```

#### Deliverables
✅ **Exit Criteria**:
- Warm pool working with <100ms latency
- 80% coverage for `warm_pool.rs`

---

### **Week 11: Secret Injection & Checkpointing**

#### Objectives
Inject Vault secrets; support CRIU checkpointing.

#### Tasks
- [ ] **Secret Injection**:
  - Fetch secret from Vault, inject as env var
  - Test: Secret rotation → New sandboxes get updated secret
- [ ] **Checkpoint/Restore**:
  - Use CRIU to snapshot running sandbox
  - Test: Restore sandbox → Resumes from exact state

#### Deliverables
✅ **Exit Criteria**:
- Secret injection tested with 3 providers (Vault, AWS Secrets Manager, mock)
- 75% coverage for `checkpoint.rs`

---

### **Week 12: Inference Abstraction & Multi-Runtime**

#### Objectives
Abstract inference layer; support Kata Containers.

#### Tasks
- [ ] **Inference Layer**:
  - Trait: `InferenceProvider { fn invoke(&self, input: &str) -> Result<String> }`
  - Implementations: Claude API, OpenAI API, mock
  - Test: Switch providers without code changes
- [ ] **Multi-Runtime Support**:
  - Add Kata Containers runtime adapter
  - Test: Same workload runs on gVisor and Kata

#### Deliverables
✅ **Exit Criteria**:
- **Runtime MVP**: Sandboxes running Claude inference
- **80% coverage gate** passed

---

## **Weeks 11-14: SPARC-R1 Messaging Implementation (TDD)**

**Phase**: Refinement (Overlaps with Runtime Week 11-12)
**Team**: 1 Cryptography Lead + 3 Backend Engineers + 1 Security Auditor
**Target Coverage**: 90% (crypto code requires higher coverage)

### **Week 11: X3DH Key Agreement**

#### Objectives
Implement X3DH protocol for initial key exchange.

#### Tasks
- [ ] **NHI Key Bundle Management**:
  - Generate identity key, signed prekey, one-time prekeys
  - Test: Bundle serialization/deserialization
- [ ] **X3DH Handshake** (TDD):
  - Test: Alice and Bob derive shared secret
  - Property test: Shared secrets always match

#### Crypto Property Tests (using `proptest`)
```rust
proptest! {
    #[test]
    fn x3dh_shared_secret_symmetric(
        alice_keys in arb_keypair(),
        bob_keys in arb_keypair()
    ) {
        let secret_alice = x3dh_alice(&alice_keys, &bob_keys.public);
        let secret_bob = x3dh_bob(&bob_keys, &alice_keys.public);
        assert_eq!(secret_alice, secret_bob);
    }
}
```

#### Deliverables
✅ **Exit Criteria**:
- X3DH interoperability tested with Signal's test vectors
- 95% coverage for `x3dh.rs`

---

### **Week 12: Double Ratchet & Message Envelopes**

#### Objectives
Implement Double Ratchet for forward secrecy.

#### Tasks
- [ ] **Double Ratchet** (TDD):
  - Test: Chain key derivation produces unique keys
  - Test: Out-of-order message delivery → Decrypts correctly
- [ ] **Message Envelope Format**:
  - Protobuf schema with encrypted payload + header
  - Test: Malformed envelope → Rejected

#### Unit Tests (15 tests)
```rust
#[test] fn test_ratchet_forward_secrecy() { ... }
#[test] fn test_out_of_order_messages() { ... }
```

#### Deliverables
✅ **Exit Criteria**:
- Ratchet tested with 1000-message conversation
- 90% coverage for `double_ratchet.rs`

---

### **Week 13: PQXDH Post-Quantum & Group Messaging**

#### Objectives
Add post-quantum resistance; support group chats.

#### Tasks
- [ ] **PQXDH Extension**:
  - Integrate Kyber768 KEM
  - Test: Hybrid X3DH + PQXDH handshake
- [ ] **Group Messaging (Sender Keys)**:
  - Distribute sender keys to group members
  - Test: 100-member group → All receive messages

#### Deliverables
✅ **Exit Criteria**:
- PQXDH tested against NIST PQ vectors
- 85% coverage for `pqxdh.rs`

---

### **Week 14: Message Delivery & AuthZ Integration**

#### Objectives
Deliver messages reliably; enforce access policies.

#### Tasks
- [ ] **Message Delivery System**:
  - Queue undelivered messages (PostgreSQL SKIP LOCKED)
  - Test: Recipient offline → Message queued, delivered on reconnect
- [ ] **Authorization Integration**:
  - Cedar policy: `ALLOW send IF sender.role == "agent" AND recipient.consent == true`
  - Test: Unauthorized send → Rejected

#### Deliverables
✅ **Exit Criteria**:
- **Messaging MVP**: End-to-end encrypted messaging working
- **90% coverage gate** passed (highest due to security sensitivity)

---

## **Weeks 12-16: SPARC-R2 Demo Development (Parallel Track)**

**Phase**: Refinement (Investor-Facing Demonstrations)
**Team**: 1 Product Manager + 2 Full-Stack Engineers + 1 Designer

### **Week 12-13: Trading Demo - "Autonomous Portfolio Manager"**

#### Objectives
Showcase Metering quota enforcement and Oversight approval flows.

#### Features
- [ ] **Agent Provisioning UI**: Dashboard to create trading agent with budget ($10K/month)
- [ ] **Real-Time Trading Dashboard**:
  - Live chart of portfolio value
  - Metered API calls (TradingView, Alpaca)
  - Budget burn-down meter
- [ ] **Oversight Trigger Flow**:
  - Agent attempts $50K trade → Oversight request created
  - Slack notification to approver
  - Approve/reject buttons update state in real-time

#### Technical Stack
- Frontend: React + TailwindCSS
- Backend: creto-metering + creto-oversight APIs
- Mock Trading API (simulated market data)

#### Deliverables
✅ **Exit Criteria**:
- Demo video: 3-minute walkthrough
- Working prototype deployed to staging

---

### **Week 13-14: Travel Demo - "Multi-Agent Trip Planner"**

#### Objectives
Demonstrate Runtime sandbox isolation and Messaging coordination.

#### Features
- [ ] **Multi-Agent Coordination**:
  - 3 agents: FlightBooker, HotelFinder, ActivityPlanner
  - Encrypted messaging between agents (using creto-messaging)
- [ ] **Budget Allocation UI**:
  - Set total trip budget ($5K)
  - Agents negotiate allocation via messages
- [ ] **Parallel Booking Visualization**:
  - Real-time log of agent actions in isolated sandboxes
  - Show network egress restrictions (only to approved APIs)

#### Deliverables
✅ **Exit Criteria**:
- Demo shows 3 sandboxes running concurrently
- Message encryption visualized (show key rotation)

---

### **Week 14-15: Healthcare Demo - "Clinical Decision Support"**

#### Objectives
Highlight Oversight physician approval and HIPAA audit logging.

#### Features
- [ ] **Diagnostic Workflow**:
  - Agent analyzes patient symptoms → Suggests diagnosis
  - Requires physician approval before sharing with patient
- [ ] **Physician Approval Dashboard**:
  - Email notification with case details
  - One-click approve/reject with rationale field
- [ ] **HIPAA Audit Viewer**:
  - Immutable log of all agent actions
  - Filter by patient ID, agent ID, timestamp

#### Deliverables
✅ **Exit Criteria**:
- Demo complies with HIPAA audit requirements (simulated)
- Physician approval latency <5 seconds

---

### **Week 15-16: PSA Demo - "Autonomous Engagement Manager"**

#### Objectives
Showcase cross-product integration (all 4 products in one workflow).

#### Features
- [ ] **Engagement Setup**:
  - Agent creates SOW with scope, budget, timeline
  - Metering: Track hours spent on engagement
- [ ] **Document Analysis UI**:
  - Agent reads contracts in sandbox (Runtime)
  - Flags risky clauses → Oversight request to partner
- [ ] **Partner Sign-Off Flow**:
  - Encrypted message to partner (Messaging)
  - Partner approves via secure link
  - Invoice generated (Metering)

#### Deliverables
✅ **Exit Criteria**:
- **All 4 demos alpha-ready**
- Polished UI/UX for investor presentation

---

## **Week 17-18: SPARC-C Integration Phase**

**Phase**: Completion (Full-Stack Integration)
**Team**: 1 Tech Lead + 6 Engineers (2 per stream) + 1 QA Lead

### **Week 17: E2E Integration Testing**

#### Objectives
Validate all products work together in realistic scenarios.

#### Tasks
- [ ] **Cross-Product Workflows**:
  - Scenario: Agent exceeds quota → Oversight request → Approved → Quota increased → Agent continues
  - Scenario: Agent in sandbox sends encrypted message → Recipient decrypts → Metering logs both operations
- [ ] **Performance Benchmarks**:
  - Target: 10,000 metered events/second
  - Target: <200ms oversight request creation
  - Target: <500ms sandbox cold start
  - Target: <50ms message encryption
- [ ] **Security Audit**:
  - External pen test of all APIs
  - Cedar policy review by security team
  - Crypto implementation audit (X3DH, Double Ratchet)

#### Deliverables
✅ **Exit Criteria**:
- E2E tests passing for all 4 demos
- Performance benchmarks meet targets
- Zero critical security findings

---

### **Week 18: Production Readiness**

#### Objectives
Harden infrastructure for production deployment.

#### Tasks
- [ ] **Monitoring & Alerting**:
  - Prometheus metrics for all products
  - Grafana dashboards (quotas, approval latency, sandbox uptime)
  - PagerDuty alerts for SLA violations
- [ ] **Database Migrations**:
  - Test migrations on production-sized datasets (10M rows)
  - Zero-downtime migration strategy
- [ ] **Disaster Recovery**:
  - Backup/restore procedures for Postgres
  - Sandbox state recovery (CRIU checkpoints in S3)
- [ ] **Load Testing**:
  - Use `k6` to simulate 1000 concurrent agents
  - Test autoscaling (Kubernetes HPA)

#### Deliverables
✅ **Exit Criteria**:
- Production environment passes load tests
- DR runbook documented and tested

---

## **Week 19-20: SPARC-C Polish & Release**

**Phase**: Completion (Investor Presentation Readiness)
**Team**: 1 Product Manager + 4 Engineers + 1 Technical Writer + 1 QA

### **Week 19: Demo Polish & Rehearsal**

#### Objectives
Refine demos for maximum impact; prepare investor pitch.

#### Tasks
- [ ] **UI/UX Polish**:
  - Designer review of all 4 demos
  - Fix visual bugs, improve animations
  - Add "wow" moments (e.g., real-time quota countdown)
- [ ] **Demo Rehearsal**:
  - Practice 10-minute pitch 5 times
  - Record video demos as backup
  - Prepare Q&A for technical deep-dives
- [ ] **Documentation Finalization**:
  - API reference docs (OpenAPI + rustdoc)
  - Deployment guide (Kubernetes manifests)
  - Security whitepaper (crypto protocols)

#### Deliverables
✅ **Exit Criteria**:
- All demos run flawlessly (no live coding)
- Pitch deck with embedded demo videos

---

### **Week 20: v1.0.0 Release**

#### Objectives
Tag v1.0.0, deploy to production, announce release.

#### Tasks
- [ ] **Release Checklist**:
  - [ ] All CI/CD checks passing
  - [ ] Coverage gates enforced (85%+ for Metering/Oversight)
  - [ ] Security audit sign-off
  - [ ] Legal review of open-source licenses
  - [ ] Changelog generated (`git cliff`)
- [ ] **Production Deployment**:
  - Deploy to AWS EKS (multi-region)
  - Smoke tests in production
  - Gradual rollout (10% → 50% → 100% traffic)
- [ ] **Release Announcement**:
  - Blog post on company website
  - Social media campaign (#CretoLaunch)
  - Email to beta customers

#### Deliverables
✅ **Exit Criteria**:
- **v1.0.0 tagged** and available on GitHub
- **Production deployment successful** (99.9% uptime in first 48h)
- **Investor presentation delivered**

---

## 2. Gantt Chart

```
Week:     1   2   3   4   5   6   7   8   9  10  11  12  13  14  15  16  17  18  19  20
          │   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │   │
SPARC-P:  ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
SPARC-A:  ░░░░░░░░████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
Metering: ░░░░░░░░░░░░░░░░████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
Oversight:░░░░░░░░░░░░░░░░░░░░░░░░████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
Runtime:  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░
Messaging:░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████████████████░░░░░░░░░░░░░░░░░░░░
Demo1:    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████████░░░░░░░░░░░░░░░░░░░░░░░░
Demo2:    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████████░░░░░░░░░░░░░░░░░░░░
Demo3:    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████████░░░░░░░░░░░░░░░░
Demo4:    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████████░░░░░░░░░░░░
SPARC-C:  ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░████████████

Legend:
████ = Active development
░░░░ = Maintenance/support

Critical Path: SPARC-P → SPARC-A → Metering → Oversight → Runtime → Messaging → SPARC-C
Parallel Tracks: Demos (Weeks 12-16), Security Audit (Week 17)
```

---

## 3. Weekly Checkpoints (Friday EOD Criteria)

| Week | Must Be True | Verification Method |
|------|--------------|---------------------|
| **1** | `cargo check` passes for all crates | CI/CD green |
| **2** | All contracts compilable, mock tests passing | `cargo test --all-features` |
| **3** | CI/CD pipeline operational, coverage >50% | Codecov dashboard |
| **4** | Integration tests passing, DI working | `cargo test --test integration` |
| **5** | Metering ingestion tested (17 tests green) | `cargo test metering::ingestion` |
| **6** | Aggregation engine handles 10K events | Benchmark: <100ms for 10K |
| **7** | Quota enforcement blocks overages | Integration test passing |
| **8** | Metering MVP complete, 85% coverage | Coverage badge green |
| **9** | Oversight state machine tested | 12 unit tests passing |
| **10** | Oversight MVP complete, 85% coverage | Coverage badge green |
| **11** | Runtime sandboxes start in <500ms | Benchmark passing |
| **12** | Runtime MVP complete, 80% coverage | Coverage badge green |
| **13** | Messaging X3DH + Ratchet working | Interop test with Signal vectors |
| **14** | Messaging MVP complete, 90% coverage | Coverage badge green |
| **15** | Demo 3 (Healthcare) alpha deployed | Staging environment live |
| **16** | All 4 demos rehearsed 3 times | PM approval |
| **17** | E2E tests passing, no P0 bugs | QA sign-off |
| **18** | Load tests passing (1000 agents) | k6 dashboard green |
| **19** | Demo polish complete, docs finalized | Technical writer approval |
| **20** | v1.0.0 released, production stable | 99.9% uptime in 48h |

---

## 4. Resource Allocation

### **Staffing by Phase**

| Week | Phase | Engineers | Specialists | Total FTE |
|------|-------|-----------|-------------|-----------|
| **1-2** | SPARC-P | 4 Coders | 1 Planner, 1 Architect | 6 |
| **3-4** | SPARC-A | 3 Backend | 1 DevOps, 1 QA | 5 |
| **5-8** | Metering R1 | 4 Backend | 1 Tech Lead, 1 QA | 6 |
| **7-10** | Oversight R1 | 3 Backend | 1 Tech Lead, 1 Workflow | 5 |
| **9-12** | Runtime R1 | 4 Systems | 1 Tech Lead, 1 Security | 6 |
| **11-14** | Messaging R1 | 3 Backend | 1 Crypto Lead, 1 Security | 5 |
| **12-16** | Demos | 2 Full-Stack | 1 PM, 1 Designer | 4 |
| **17-18** | Integration | 6 Backend | 1 Tech Lead, 1 QA, 1 Security | 9 |
| **19-20** | Polish | 4 Backend | 1 PM, 1 Tech Writer, 1 QA | 7 |

### **Peak Team Size**: Week 17 (9 FTEs)
### **Average Team Size**: 6 FTEs

---

## 5. Risk Register & Mitigation

| Week | Risk | Impact | Probability | Mitigation |
|------|------|--------|-------------|------------|
| **1-2** | SDD ambiguity delays skeleton | High | Medium | Daily architect sync, freeze specs Week 2 |
| **3** | Flaky integration tests block CI | Medium | High | Use `testcontainers`, retry logic |
| **5-8** | Metering coverage <85% | High | Medium | TDD enforcement, pair programming |
| **9** | Oversight timeout logic fails | Medium | Medium | Property testing with `tokio::time::pause` |
| **11** | gVisor compatibility issues | High | Low | Fallback to Kata Containers |
| **13** | PQXDH crypto bug in prod | Critical | Low | External audit, fuzzing tests |
| **15** | Demo UI bugs during rehearsal | Medium | High | QA review 3 days before rehearsal |
| **17** | E2E tests fail at scale | High | Medium | Load testing from Week 12 |
| **19** | Investor demo WiFi failure | High | Low | Pre-record backup videos |
| **20** | Production deployment rollback | Critical | Medium | Blue-green deployment, canary rollout |

---

## 6. Success Metrics (OKRs)

### **Objective**: Deliver production-ready v1.0.0 in 20 weeks

| Key Result | Target | Tracking |
|------------|--------|----------|
| Test coverage (Metering/Oversight) | ≥85% | Codecov dashboard |
| Test coverage (Runtime) | ≥80% | Codecov dashboard |
| Test coverage (Messaging) | ≥90% | Codecov dashboard |
| Metering event ingestion throughput | 10,000/sec | Criterion benchmarks |
| Oversight request latency (p95) | <200ms | Prometheus metrics |
| Runtime sandbox cold start (p95) | <500ms | Criterion benchmarks |
| Messaging encryption latency (p95) | <50ms | Criterion benchmarks |
| Production uptime (Week 20) | ≥99.9% | Datadog APM |
| Demo success rate (4 demos) | 100% | Rehearsal checklist |
| Investor NPS | ≥9/10 | Post-demo survey |

---

## 7. Tools & Infrastructure

| Category | Tool | Purpose |
|----------|------|---------|
| **Version Control** | GitHub | Code hosting, PR reviews |
| **CI/CD** | GitHub Actions | Automated testing, deployment |
| **Coverage** | `cargo-tarpaulin` + Codecov | Enforce coverage gates |
| **Benchmarking** | `criterion` | Performance regression detection |
| **Testing** | `proptest`, `testcontainers` | Property tests, integration tests |
| **Tracing** | OpenTelemetry + Jaeger | Distributed tracing |
| **Monitoring** | Prometheus + Grafana | Real-time metrics |
| **Alerting** | PagerDuty | Incident management |
| **Documentation** | `rustdoc`, OpenAPI, mdBook | API docs, guides |
| **Load Testing** | `k6` | Simulate 1000 agents |
| **Security** | `cargo-audit`, Snyk | Dependency vulnerability scanning |
| **Deployment** | Kubernetes (EKS) | Container orchestration |
| **Secrets** | HashiCorp Vault | Secrets management |

---

## 8. Communication Plan

### **Weekly Cadence**
- **Monday 9am**: Sprint planning (set week goals)
- **Wednesday 2pm**: Mid-week sync (blockers, dependencies)
- **Friday 4pm**: Checkpoint review (verify exit criteria)

### **Daily Standups** (async via Slack)
- What I did yesterday
- What I'm doing today
- Blockers (tagged with @tech-lead)

### **Demo Reviews**
- **Week 13, 14, 15, 16 (Fridays)**: Demo rehearsals with PM
- **Week 19 (Friday)**: Final investor rehearsal

### **Escalation Path**
1. Engineer → Tech Lead (resolve in 2 hours)
2. Tech Lead → Engineering Manager (resolve in 24 hours)
3. Engineering Manager → CTO (critical path blocker)

---

## 9. Appendix: SPARC Phase Definitions

| Phase | Abbreviation | Focus | Duration |
|-------|--------------|-------|----------|
| **Planning** | SPARC-P | Skeleton generation, contract definition | Week 1-2 |
| **Architecture** | SPARC-A | Infrastructure, CI/CD, integration setup | Week 3-4 |
| **Refinement R1** | SPARC-R1 | TDD implementation of core products | Week 5-14 |
| **Refinement R2** | SPARC-R2 | Demo development (parallel track) | Week 12-16 |
| **Completion** | SPARC-C | Integration, polish, release | Week 17-20 |

---

## 10. Next Actions

**Immediate (Week 0 - Pre-Kickoff)**:
1. [ ] Finalize SDD (resolve all TODOs)
2. [ ] Hire 2 additional backend engineers
3. [ ] Provision AWS EKS cluster (staging + production)
4. [ ] Setup GitHub organization and repositories
5. [ ] Create Slack channels (#metering, #oversight, #runtime, #messaging, #demos)
6. [ ] Schedule Week 1 kickoff meeting

**Week 1 Day 1**:
1. [ ] All-hands kickoff presentation (share this timeline)
2. [ ] Assign engineers to products (2 per product)
3. [ ] Initialize Cargo workspace (`cargo init --workspace`)
4. [ ] Create first PR (README + project structure)

---

**Prepared by**: Timeline Planner Agent
**Date**: 2025-12-26
**Version**: 1.0.0
**Next Review**: Week 4 (verify SPARC-A completion)
