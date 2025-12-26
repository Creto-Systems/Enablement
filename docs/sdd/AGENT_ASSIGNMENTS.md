# SPARC Agent Assignment Matrix
## Creto Enablement Layer Implementation

**Document Version:** 1.0
**Created:** 2025-12-26
**Coordinator:** SPARC Orchestration Agent
**Project Duration:** 20 weeks (140 days)

---

## 1. Swarm Topology

### Hierarchical Swarm Structure

```
                    ┌─────────────────────────────┐
                    │     QUEEN AGENT             │
                    │     sparc-coord             │
                    │     (SPARC Orchestrator)    │
                    │     Phase Gate Controller   │
                    └──────────────┬──────────────┘
                                   │
        ┏━━━━━━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━━━━━━━┓
        ┃                          ┃                           ┃
        ▼                          ▼                           ▼
  ┌─────────────┐          ┌──────────────┐          ┌─────────────┐
  │ PRODUCT     │          │ DEMO         │          │ INFRA       │
  │ HIVE        │          │ HIVE         │          │ HIVE        │
  │ (4 products)│          │ (4 demos)    │          │ (DevOps)    │
  └──────┬──────┘          └──────┬───────┘          └──────┬──────┘
         │                        │                         │
    ┌────┴────┐              ┌────┴────┐              ┌─────┴──────┐
    ▼    ▼    ▼              ▼    ▼    ▼              ▼     ▼      ▼
  [12 agents]              [8 agents]              [3 agents]

  Metering  Oversight      Trading  Travel       Architect
  Runtime   Messaging      Health   PSA          CI/CD
                                                  Security
```

### Topology Characteristics
- **Type:** Hierarchical with mesh communication at peer level
- **Depth:** 3 levels (Queen → Hive → Workers)
- **Width:** 23 concurrent agents maximum
- **Communication:** Memory-coordinated with hooks
- **Fault Tolerance:** 2 backup agents per critical path

---

## 2. Agent Types and Capabilities

### Master Agent Registry

| Agent Type | Primary Capabilities | Tool Access | Concurrent Limit | Memory Namespace |
|------------|---------------------|-------------|------------------|------------------|
| **sparc-coord** | Phase orchestration, quality gates, swarm coordination | ALL | 1 | `swarm/queen` |
| **planner** | Task decomposition, dependency mapping, resource allocation | Read, TodoWrite, Bash | 1 | `swarm/planning` |
| **coder** | Rust implementation, core logic, data structures | Read, Write, Edit, Bash, Grep | 4 | `swarm/product/{name}` |
| **tester** | TDD, unit tests, integration tests, coverage | Read, Write, Edit, Bash | 4 | `swarm/test/{product}` |
| **reviewer** | Code review, security audit, quality assurance | Read, Grep, Bash | 2 | `swarm/review` |
| **backend-dev** | API implementation, demo backends, integration | Read, Write, Edit, Bash, Grep | 4 | `swarm/demo/{name}` |
| **system-architect** | System design, integration architecture, contracts | Read, Write, Edit | 1 | `swarm/architecture` |
| **cicd-engineer** | Pipeline setup, automation, deployment | Read, Write, Bash | 1 | `swarm/infra/cicd` |
| **code-analyzer** | Performance analysis, optimization, profiling | Read, Grep, Bash | 1 | `swarm/analysis` |
| **api-docs** | Documentation generation, API specs, guides | Read, Write, Grep | 1 | `swarm/docs` |

### Agent Specialization Matrix

```
CAPABILITY HEAT MAP (1=Low, 5=High)

Agent Type       │ Rust │ Testing │ Security │ APIs │ Integration │ Docs
─────────────────┼──────┼─────────┼──────────┼──────┼─────────────┼──────
sparc-coord      │  3   │    4    │    4     │  3   │      5      │  4
planner          │  2   │    3    │    2     │  3   │      4      │  5
coder            │  5   │    3    │    3     │  3   │      3      │  2
tester           │  3   │    5    │    4     │  3   │      4      │  3
reviewer         │  4   │    4    │    5     │  4   │      3      │  3
backend-dev      │  3   │    3    │    3     │  5   │      4      │  3
system-architect │  3   │    3    │    4     │  4   │      5      │  4
cicd-engineer    │  2   │    4    │    4     │  3   │      5      │  3
```

---

## 3. Product-to-Agent Mapping

### 3.1 creto-metering (Usage Tracking & Billing)

**Agent Assignments:**

| Agent ID | Type | Role | Files Owned | Phase Active |
|----------|------|------|-------------|--------------|
| `coder-metering-1` | coder | Core implementation | `metering/src/lib.rs`, `metering/src/tracker.rs` | P, R1, R2 |
| `coder-metering-2` | coder | Storage layer | `metering/src/storage.rs`, `metering/src/db/` | R1, R2 |
| `tester-metering-1` | tester | Unit tests | `metering/tests/unit/` | R1, R2 |
| `tester-metering-2` | tester | Integration tests | `metering/tests/integration/` | R2, C |
| `reviewer-security` | reviewer | Security audit | All metering files | R2, C |

**Handoff Chain:**
```
coder-metering-1 → coder-metering-2 → tester-metering-1 → tester-metering-2 → reviewer-security
```

**Memory Keys:**
- State: `swarm/product/metering/state`
- API Contract: `swarm/product/metering/api`
- Test Coverage: `swarm/product/metering/coverage`

---

### 3.2 creto-oversight (Observability & Monitoring)

**Agent Assignments:**

| Agent ID | Type | Role | Files Owned | Phase Active |
|----------|------|------|-------------|--------------|
| `coder-oversight-1` | coder | Telemetry core | `oversight/src/lib.rs`, `oversight/src/telemetry.rs` | P, R1, R2 |
| `coder-oversight-2` | coder | Metrics & logs | `oversight/src/metrics.rs`, `oversight/src/logging.rs` | R1, R2 |
| `tester-oversight-1` | tester | Unit tests | `oversight/tests/unit/` | R1, R2 |
| `tester-oversight-2` | tester | Integration tests | `oversight/tests/integration/` | R2, C |
| `reviewer-security` | reviewer | Security audit | All oversight files | R2, C |

**Handoff Chain:**
```
coder-oversight-1 → coder-oversight-2 → tester-oversight-1 → tester-oversight-2 → reviewer-security
```

**Memory Keys:**
- State: `swarm/product/oversight/state`
- Metrics Schema: `swarm/product/oversight/metrics-schema`
- Test Coverage: `swarm/product/oversight/coverage`

---

### 3.3 creto-runtime (Safe Execution Environment)

**Agent Assignments:**

| Agent ID | Type | Role | Files Owned | Phase Active |
|----------|------|------|-------------|--------------|
| `coder-runtime-1` | coder | Runtime core | `runtime/src/lib.rs`, `runtime/src/executor.rs` | P, R1, R2 |
| `coder-runtime-2` | coder | Security sandbox | `runtime/src/sandbox.rs`, `runtime/src/isolation.rs` | R1, R2 |
| `tester-runtime-1` | tester | Unit tests | `runtime/tests/unit/` | R1, R2 |
| `tester-runtime-2` | tester | Integration tests | `runtime/tests/integration/` | R2, C |
| `reviewer-security` | reviewer | Security audit | All runtime files | R2, C |

**Handoff Chain:**
```
coder-runtime-1 → coder-runtime-2 → tester-runtime-1 → tester-runtime-2 → reviewer-security
```

**Memory Keys:**
- State: `swarm/product/runtime/state`
- Sandbox Config: `swarm/product/runtime/sandbox-config`
- Security Audit: `swarm/product/runtime/security-audit`

---

### 3.4 creto-messaging (Event Bus & Communication)

**Agent Assignments:**

| Agent ID | Type | Role | Files Owned | Phase Active |
|----------|------|------|-------------|--------------|
| `coder-messaging-1` | coder | Message broker | `messaging/src/lib.rs`, `messaging/src/broker.rs` | P, R1, R2 |
| `coder-messaging-2` | coder | Encryption & routing | `messaging/src/crypto.rs`, `messaging/src/router.rs` | R1, R2 |
| `tester-messaging-1` | tester | Unit tests | `messaging/tests/unit/` | R1, R2 |
| `tester-messaging-2` | tester | Integration tests | `messaging/tests/integration/` | R2, C |
| `reviewer-crypto` | reviewer | Cryptography audit | Crypto-related files | R2, C |

**Handoff Chain:**
```
coder-messaging-1 → coder-messaging-2 → tester-messaging-1 → tester-messaging-2 → reviewer-crypto
```

**Memory Keys:**
- State: `swarm/product/messaging/state`
- Crypto Protocol: `swarm/product/messaging/crypto-protocol`
- Message Schema: `swarm/product/messaging/schema`

---

## 4. Demo-to-Agent Mapping

### 4.1 Demo 1: Autonomous Trading Bot

**Purpose:** AI agent makes trades, tracks usage via metering, monitors via oversight

**Agent Assignments:**

| Agent ID | Type | Role | Files Owned | Dependencies |
|----------|------|------|-------------|--------------|
| `backend-dev-trading-1` | backend-dev | Trading logic | `demos/trading/src/bot.rs` | metering, oversight |
| `backend-dev-trading-2` | backend-dev | API integration | `demos/trading/src/api.rs` | messaging |
| `tester-demo-trading` | tester | Demo tests | `demos/trading/tests/` | All trading files |

**Integration Points:**
- **creto-metering:** Track API calls, compute costs
- **creto-oversight:** Monitor bot decisions, log trades
- **creto-messaging:** Event notifications

**Memory Keys:**
- State: `swarm/demo/trading/state`
- Integration Status: `swarm/demo/trading/integration`

---

### 4.2 Demo 2: AI Travel Coordination Fleet

**Purpose:** Multi-agent travel planning with cost tracking and coordination

**Agent Assignments:**

| Agent ID | Type | Role | Files Owned | Dependencies |
|----------|------|------|-------------|--------------|
| `backend-dev-travel-1` | backend-dev | Fleet coordinator | `demos/travel/src/coordinator.rs` | runtime, messaging |
| `backend-dev-travel-2` | backend-dev | Agent spawning | `demos/travel/src/spawner.rs` | runtime |
| `tester-demo-travel` | tester | Demo tests | `demos/travel/tests/` | All travel files |

**Integration Points:**
- **creto-runtime:** Safe agent execution
- **creto-messaging:** Inter-agent communication
- **creto-metering:** Track coordination costs

**Memory Keys:**
- State: `swarm/demo/travel/state`
- Agent Registry: `swarm/demo/travel/agents`

---

### 4.3 Demo 3: Healthcare AI Diagnostics

**Purpose:** Privacy-preserving AI diagnostics with compliance monitoring

**Agent Assignments:**

| Agent ID | Type | Role | Files Owned | Dependencies |
|----------|------|------|-------------|--------------|
| `backend-dev-healthcare-1` | backend-dev | Diagnostic engine | `demos/healthcare/src/diagnostics.rs` | runtime, oversight |
| `backend-dev-healthcare-2` | backend-dev | Privacy layer | `demos/healthcare/src/privacy.rs` | messaging |
| `tester-demo-healthcare` | tester | Demo tests | `demos/healthcare/tests/` | All healthcare files |

**Integration Points:**
- **creto-runtime:** Sandboxed execution for privacy
- **creto-oversight:** Compliance monitoring
- **creto-messaging:** Encrypted communication

**Memory Keys:**
- State: `swarm/demo/healthcare/state`
- Compliance Status: `swarm/demo/healthcare/compliance`

---

### 4.4 Demo 4: Professional Services Automation (PSA)

**Purpose:** Multi-tenant AI services with usage-based billing

**Agent Assignments:**

| Agent ID | Type | Role | Files Owned | Dependencies |
|----------|------|------|-------------|--------------|
| `backend-dev-psa-1` | backend-dev | Service orchestrator | `demos/psa/src/orchestrator.rs` | All products |
| `backend-dev-psa-2` | backend-dev | Billing integration | `demos/psa/src/billing.rs` | metering |
| `tester-demo-psa` | tester | Demo tests | `demos/psa/tests/` | All PSA files |

**Integration Points:**
- **All 4 Products:** Demonstrates full integration
- **creto-metering:** Multi-tenant billing
- **creto-oversight:** Service health monitoring

**Memory Keys:**
- State: `swarm/demo/psa/state`
- Tenant Registry: `swarm/demo/psa/tenants`

---

## 5. Infrastructure Hive Mapping

### Infrastructure Agent Assignments

| Agent ID | Type | Role | Files Owned | Phase Active |
|----------|------|------|-------------|--------------|
| `system-architect-1` | system-architect | Overall architecture | `docs/architecture/`, `ARCHITECTURE.md` | A, R1, C |
| `cicd-engineer-1` | cicd-engineer | CI/CD pipelines | `.github/workflows/`, `scripts/ci/` | A, R2, C |
| `code-analyzer-1` | code-analyzer | Performance profiling | `scripts/bench/`, performance reports | R2, C |
| `api-docs-1` | api-docs | Documentation | `docs/api/`, README files | R2, C |

**Memory Keys:**
- Architecture Decisions: `swarm/infra/architecture/decisions`
- CI/CD Status: `swarm/infra/cicd/status`
- Performance Baselines: `swarm/infra/perf/baselines`

---

## 6. Phase-Specific Agent Activation

### Phase P: Preparation (Weeks 1-2, Days 1-14)

**Active Agents:** 6 total

| Agent | Focus | Deliverables |
|-------|-------|--------------|
| sparc-coord | Phase orchestration | Phase gate criteria |
| planner | Task decomposition | 140-day task breakdown |
| coder-metering-1 | Skeleton generation | `metering/src/lib.rs` skeleton |
| coder-oversight-1 | Skeleton generation | `oversight/src/lib.rs` skeleton |
| coder-runtime-1 | Skeleton generation | `runtime/src/lib.rs` skeleton |
| coder-messaging-1 | Skeleton generation | `messaging/src/lib.rs` skeleton |

**Concurrent Execution Pattern:**
```javascript
[Single Message - Phase P Setup]:
  Task("Planner", "Decompose 140-day plan into daily tasks", "planner")
  Task("Metering Skeleton", "Generate creto-metering skeleton with TDD stubs", "coder")
  Task("Oversight Skeleton", "Generate creto-oversight skeleton with TDD stubs", "coder")
  Task("Runtime Skeleton", "Generate creto-runtime skeleton with TDD stubs", "coder")
  Task("Messaging Skeleton", "Generate creto-messaging skeleton with TDD stubs", "coder")
```

---

### Phase A: Architecture (Weeks 3-4, Days 15-28)

**Active Agents:** 4 total

| Agent | Focus | Deliverables |
|-------|-------|--------------|
| sparc-coord | Quality gate enforcement | Architecture review |
| system-architect-1 | System design | Architecture diagrams, API contracts |
| cicd-engineer-1 | Infrastructure setup | CI/CD pipelines, Docker configs |
| planner | Refinement planning | R1/R2 task allocation |

**Concurrent Execution Pattern:**
```javascript
[Single Message - Phase A Setup]:
  Task("System Architect", "Design integration architecture and API contracts", "system-architect")
  Task("CI/CD Engineer", "Setup GitHub Actions, Docker, and deployment pipelines", "cicd-engineer")
  Task("Planner", "Refine R1/R2 task allocation based on architecture", "planner")
```

---

### Phase R1: Core Refinement (Weeks 5-10, Days 29-70)

**Active Agents:** 12 total (4 coders + 4 testers + 2 reviewers + 2 support)

| Agent | Focus | Concurrent Work |
|-------|-------|-----------------|
| sparc-coord | Progress tracking | Phase monitoring |
| coder-metering-1 | Core logic | Parallel with others |
| coder-oversight-1 | Core logic | Parallel with others |
| coder-runtime-1 | Core logic | Parallel with others |
| coder-messaging-1 | Core logic | Parallel with others |
| tester-metering-1 | Unit tests | Follows coder-metering-1 |
| tester-oversight-1 | Unit tests | Follows coder-oversight-1 |
| tester-runtime-1 | Unit tests | Follows coder-runtime-1 |
| tester-messaging-1 | Unit tests | Follows coder-messaging-1 |
| reviewer-security | Security review | Periodic checkpoints |
| code-analyzer-1 | Performance profiling | Periodic analysis |
| api-docs-1 | API documentation | Follows implementation |

**Concurrent Execution Pattern:**
```javascript
[Single Message - R1 Sprint 1]:
  // Week 5-6: Metering implementation
  Task("Metering Core", "Implement usage tracking with TDD", "coder")
  Task("Metering Tests", "Write unit tests for tracker", "tester")

[Single Message - R1 Sprint 2]:
  // Week 7-8: Oversight implementation
  Task("Oversight Core", "Implement telemetry and metrics", "coder")
  Task("Oversight Tests", "Write unit tests for telemetry", "tester")

[Single Message - R1 Sprint 3]:
  // Week 9-10: Runtime + Messaging implementation
  Task("Runtime Core", "Implement safe execution environment", "coder")
  Task("Runtime Tests", "Write unit tests for executor", "tester")
  Task("Messaging Core", "Implement message broker", "coder")
  Task("Messaging Tests", "Write unit tests for broker", "tester")
```

---

### Phase R2: Demo Refinement (Weeks 11-16, Days 71-112)

**Active Agents:** 14 total (4 backend-devs + 4 demo testers + 4 product coders + 2 reviewers)

| Agent | Focus | Demo Assignment |
|-------|-------|-----------------|
| sparc-coord | Demo coordination | Cross-demo orchestration |
| backend-dev-trading-1 | Trading bot | Demo 1 |
| backend-dev-trading-2 | Trading API | Demo 1 |
| backend-dev-travel-1 | Travel coordinator | Demo 2 |
| backend-dev-travel-2 | Fleet spawning | Demo 2 |
| backend-dev-healthcare-1 | Diagnostics | Demo 3 |
| backend-dev-healthcare-2 | Privacy layer | Demo 3 |
| backend-dev-psa-1 | Service orchestration | Demo 4 |
| backend-dev-psa-2 | Billing integration | Demo 4 |
| tester-demo-trading | Trading tests | Demo 1 |
| tester-demo-travel | Travel tests | Demo 2 |
| tester-demo-healthcare | Healthcare tests | Demo 3 |
| tester-demo-psa | PSA tests | Demo 4 |
| reviewer-security | Security audit | All demos |

**Concurrent Execution Pattern:**
```javascript
[Single Message - R2 Week 11-12]:
  // All demos start in parallel
  Task("Trading Bot", "Build autonomous trading demo with metering", "backend-dev")
  Task("Travel Fleet", "Build multi-agent travel coordination", "backend-dev")
  Task("Healthcare AI", "Build privacy-preserving diagnostics", "backend-dev")
  Task("PSA Platform", "Build multi-tenant service platform", "backend-dev")
```

---

### Phase C: Completion (Weeks 17-20, Days 113-140)

**Active Agents:** 23 total (ALL agents active)

| Agent | Focus | Final Deliverables |
|-------|-------|-------------------|
| sparc-coord | Integration oversight | Final quality gates |
| All product coders | Bug fixes, polish | Production-ready code |
| All testers | E2E testing | Full integration tests |
| All backend-devs | Demo polish | Production demos |
| reviewer-security | Final audit | Security sign-off |
| reviewer-crypto | Crypto audit | Cryptography sign-off |
| system-architect-1 | Integration validation | Architecture sign-off |
| cicd-engineer-1 | Deployment automation | Production pipelines |
| code-analyzer-1 | Performance validation | Performance report |
| api-docs-1 | Documentation finalization | Complete documentation |

**Concurrent Execution Pattern:**
```javascript
[Single Message - Phase C Integration]:
  // All agents work in parallel on final integration
  Task("Metering Polish", "Final bug fixes and optimization", "coder")
  Task("Oversight Polish", "Final bug fixes and optimization", "coder")
  Task("Runtime Polish", "Final bug fixes and optimization", "coder")
  Task("Messaging Polish", "Final bug fixes and optimization", "coder")
  Task("E2E Tests", "Complete end-to-end integration tests", "tester")
  Task("Security Audit", "Final security review of all components", "reviewer")
  Task("Performance Validation", "Benchmark all products and demos", "code-analyzer")
  Task("Documentation", "Finalize all API docs and guides", "api-docs")
```

---

## 7. Communication Protocol

### 7.1 Memory-Based State Sharing

**Memory Key Hierarchy:**
```
swarm/
├── queen/                          # SPARC coordinator state
│   ├── current_phase               # P, A, R1, R2, C
│   ├── phase_progress              # % completion
│   └── quality_gates               # Gate status
│
├── planning/
│   ├── tasks                       # TodoWrite output
│   ├── dependencies                # Task dependency graph
│   └── resource_allocation         # Agent assignments
│
├── product/
│   ├── metering/
│   │   ├── state                   # Implementation state
│   │   ├── api                     # API contract
│   │   ├── coverage                # Test coverage %
│   │   └── issues                  # Open issues
│   ├── oversight/
│   ├── runtime/
│   └── messaging/
│
├── demo/
│   ├── trading/
│   │   ├── state                   # Demo implementation state
│   │   ├── integration             # Integration status
│   │   └── test_results            # Test pass/fail
│   ├── travel/
│   ├── healthcare/
│   └── psa/
│
└── infra/
    ├── architecture/
    │   └── decisions               # ADRs
    ├── cicd/
    │   └── status                  # Pipeline health
    └── perf/
        └── baselines               # Performance metrics
```

### 7.2 Memory Access Patterns

**Before Starting Work:**
```bash
# Agent reads shared state
npx claude-flow@alpha memory retrieve --key "swarm/product/metering/state"
npx claude-flow@alpha memory retrieve --key "swarm/product/metering/api"
```

**During Work:**
```bash
# Agent updates progress
npx claude-flow@alpha memory store --key "swarm/product/metering/state" \
  --value '{"status": "in_progress", "completion": 45, "agent": "coder-metering-1"}'
```

**After Completing Work:**
```bash
# Agent hands off to next agent
npx claude-flow@alpha memory store --key "swarm/product/metering/handoff" \
  --value '{"from": "coder-metering-1", "to": "tester-metering-1", "status": "ready"}'
```

### 7.3 Handoff Procedures

**Standard Handoff Protocol:**

1. **Completion Signal:**
   - Agent stores completion status in memory
   - Agent runs post-task hook
   - Agent updates handoff key

2. **Validation:**
   - Next agent checks handoff key
   - Next agent validates prerequisites
   - Next agent runs pre-task hook

3. **Acknowledgment:**
   - Next agent updates state to "in_progress"
   - Next agent acknowledges handoff in memory

**Example Handoff: Coder → Tester**

```bash
# Coder completes work
npx claude-flow@alpha memory store --key "swarm/product/metering/handoff" \
  --value '{
    "from": "coder-metering-1",
    "to": "tester-metering-1",
    "status": "ready",
    "files": ["metering/src/lib.rs", "metering/src/tracker.rs"],
    "test_requirements": "Unit tests for tracker, 80% coverage minimum"
  }'

# Tester receives handoff
npx claude-flow@alpha memory retrieve --key "swarm/product/metering/handoff"

# Tester acknowledges
npx claude-flow@alpha memory store --key "swarm/product/metering/state" \
  --value '{"status": "testing", "agent": "tester-metering-1"}'
```

### 7.4 Conflict Resolution

**Conflict Types and Resolution:**

| Conflict Type | Detection | Resolution | Resolver |
|--------------|-----------|------------|----------|
| **File Lock** | Two agents editing same file | First-write-wins, second agent rebases | sparc-coord |
| **API Contract Change** | Breaking change detected | Vote among affected agents | system-architect-1 |
| **Test Failure** | Test fails after merge | Rollback change, fix required | reviewer-security |
| **Performance Regression** | Benchmark below baseline | Investigation required, blocking | code-analyzer-1 |
| **Dependency Cycle** | Circular dependency detected | Architecture review | system-architect-1 |

**Escalation Path:**
```
Agent → Peer Review → Hive Lead → sparc-coord → Manual Intervention
```

---

## 8. Hooks Integration

### 8.1 Pre-Task Hooks (Every Agent)

**Hook Execution:**
```bash
npx claude-flow@alpha hooks pre-task \
  --agent "{agent_id}" \
  --task "{task_id}" \
  --description "{task_description}"
```

**Hook Actions:**
- Validate prerequisites in memory
- Check phase gates (P → A → R1 → R2 → C)
- Load previous context from memory
- Auto-assign based on file type
- Prepare resources (create dirs, etc.)

**Example:**
```bash
# coder-metering-1 starting work
npx claude-flow@alpha hooks pre-task \
  --agent "coder-metering-1" \
  --task "implement-usage-tracker" \
  --description "Implement core usage tracking logic in metering/src/tracker.rs"

# Hook output:
# ✓ Phase gate check: R1 active
# ✓ Prerequisites met: metering skeleton exists
# ✓ Context loaded: API contract from memory
# ✓ Resources ready: metering/src/ directory exists
```

---

### 8.2 Post-Edit Hooks (After File Changes)

**Hook Execution:**
```bash
npx claude-flow@alpha hooks post-edit \
  --file "{file_path}" \
  --memory-key "swarm/{product}/{phase}/{artifact}"
```

**Hook Actions:**
- Auto-format code (rustfmt)
- Run linter (clippy)
- Store file metadata in memory
- Update test coverage metrics
- Trigger dependent agent notifications

**Example:**
```bash
# After editing metering/src/tracker.rs
npx claude-flow@alpha hooks post-edit \
  --file "metering/src/tracker.rs" \
  --memory-key "swarm/product/metering/tracker-impl"

# Hook output:
# ✓ Formatted with rustfmt
# ✓ Clippy warnings: 0
# ✓ Metadata stored in memory
# ✓ Notified: tester-metering-1 (tests needed)
```

---

### 8.3 Post-Task Hooks (After Task Completion)

**Hook Execution:**
```bash
npx claude-flow@alpha hooks post-task \
  --task-id "{task_id}" \
  --status "{success|failure}" \
  --metrics '{"files_changed": X, "lines_added": Y, "tests_written": Z}'
```

**Hook Actions:**
- Train neural patterns from success/failure
- Update task status in memory
- Generate handoff to next agent
- Export metrics for phase tracking
- Update quality gate progress

**Example:**
```bash
# coder-metering-1 completes task
npx claude-flow@alpha hooks post-task \
  --task-id "implement-usage-tracker" \
  --status "success" \
  --metrics '{"files_changed": 2, "lines_added": 450, "tests_written": 0}'

# Hook output:
# ✓ Neural pattern trained: rust-implementation
# ✓ Task marked complete in memory
# ✓ Handoff created for tester-metering-1
# ✓ Phase R1 progress: 12% → 18%
```

---

### 8.4 Session Management Hooks

**Session Start:**
```bash
npx claude-flow@alpha hooks session-restore \
  --session-id "swarm-{phase}-{date}"
```

**Session End:**
```bash
npx claude-flow@alpha hooks session-end \
  --session-id "swarm-{phase}-{date}" \
  --export-metrics true
```

**Hook Actions:**
- Load/save swarm state
- Persist agent assignments
- Export daily metrics
- Generate progress summary

---

## 9. Quality Gates and Phase Transitions

### 9.1 Phase Gate Criteria

| Phase Transition | Quality Gate | Validation Method | Gate Keeper |
|------------------|--------------|-------------------|-------------|
| **P → A** | All skeletons generated, TDD stubs in place | File existence check, compile test | sparc-coord |
| **A → R1** | Architecture approved, CI/CD operational | Architecture review, pipeline test | system-architect-1 |
| **R1 → R2** | All products pass tests, 80% coverage | Test suite run, coverage report | reviewer-security |
| **R2 → C** | All demos functional, integration tests pass | E2E test suite | sparc-coord |
| **C → Done** | All quality gates passed, documentation complete | Final audit checklist | sparc-coord |

### 9.2 Quality Gate Validation Script

```bash
#!/bin/bash
# quality-gate-check.sh

PHASE=$1  # P, A, R1, R2, C

case $PHASE in
  "P")
    echo "Checking Phase P → A transition..."
    # Check skeletons exist
    [[ -f "metering/src/lib.rs" ]] || exit 1
    [[ -f "oversight/src/lib.rs" ]] || exit 1
    [[ -f "runtime/src/lib.rs" ]] || exit 1
    [[ -f "messaging/src/lib.rs" ]] || exit 1
    # Check compile
    cargo check || exit 1
    echo "✓ Phase P complete. Ready for A."
    ;;

  "A")
    echo "Checking Phase A → R1 transition..."
    # Check architecture docs
    [[ -f "docs/architecture/SYSTEM_DESIGN.md" ]] || exit 1
    # Check CI/CD
    [[ -f ".github/workflows/ci.yml" ]] || exit 1
    # Test pipeline
    gh workflow run ci.yml || exit 1
    echo "✓ Phase A complete. Ready for R1."
    ;;

  "R1")
    echo "Checking Phase R1 → R2 transition..."
    # Run all tests
    cargo test --all || exit 1
    # Check coverage
    COVERAGE=$(cargo tarpaulin --out Stdout | grep "Coverage" | awk '{print $2}' | sed 's/%//')
    [[ $COVERAGE -ge 80 ]] || exit 1
    echo "✓ Phase R1 complete. Coverage: $COVERAGE%. Ready for R2."
    ;;

  "R2")
    echo "Checking Phase R2 → C transition..."
    # Run demo tests
    cargo test --package demo-trading || exit 1
    cargo test --package demo-travel || exit 1
    cargo test --package demo-healthcare || exit 1
    cargo test --package demo-psa || exit 1
    # Run integration tests
    cargo test --test integration || exit 1
    echo "✓ Phase R2 complete. Ready for C."
    ;;

  "C")
    echo "Checking Phase C → Done transition..."
    # Final audit
    cargo clippy -- -D warnings || exit 1
    cargo audit || exit 1
    # Documentation check
    [[ -f "docs/api/API_REFERENCE.md" ]] || exit 1
    # Performance validation
    cargo bench || exit 1
    echo "✓ Phase C complete. DONE!"
    ;;
esac
```

---

## 10. Agent Spawn Templates

### 10.1 Product Development Agent Spawn

```javascript
// Template for spawning product coder + tester pair

[Single Message - Product Agent Spawn]:
  Task("Product Coder", `
    You are coder-{product}-1.

    TASK: Implement {product} core functionality using TDD.

    FILES: {product}/src/lib.rs, {product}/src/{module}.rs

    HOOKS:
    1. Pre-task: npx claude-flow@alpha hooks pre-task --agent "coder-{product}-1" --task "core-impl"
    2. Post-edit: npx claude-flow@alpha hooks post-edit --file "{product}/src/{module}.rs"
    3. Post-task: npx claude-flow@alpha hooks post-task --task-id "core-impl" --status "success"

    MEMORY:
    - Read API contract: swarm/product/{product}/api
    - Store implementation state: swarm/product/{product}/state
    - Create handoff: swarm/product/{product}/handoff

    HANDOFF: When complete, hand off to tester-{product}-1
  `, "coder"),

  Task("Product Tester", `
    You are tester-{product}-1.

    TASK: Write unit tests for {product} with 80%+ coverage.

    FILES: {product}/tests/unit/*.rs

    HOOKS:
    1. Pre-task: npx claude-flow@alpha hooks pre-task --agent "tester-{product}-1" --task "unit-tests"
    2. Post-edit: npx claude-flow@alpha hooks post-edit --file "{product}/tests/unit/{module}_test.rs"
    3. Post-task: npx claude-flow@alpha hooks post-task --task-id "unit-tests" --status "success"

    MEMORY:
    - Wait for handoff: swarm/product/{product}/handoff
    - Store coverage: swarm/product/{product}/coverage

    HANDOFF: When complete, hand off to reviewer-security
  `, "tester")
```

---

### 10.2 Demo Development Agent Spawn

```javascript
// Template for spawning demo backend developer

[Single Message - Demo Agent Spawn]:
  Task("Demo Backend Developer", `
    You are backend-dev-{demo}-1.

    TASK: Build {demo} demo integrating all 4 Creto products.

    FILES: demos/{demo}/src/*.rs

    DEPENDENCIES:
    - creto-metering (usage tracking)
    - creto-oversight (monitoring)
    - creto-runtime (execution)
    - creto-messaging (communication)

    HOOKS:
    1. Pre-task: npx claude-flow@alpha hooks pre-task --agent "backend-dev-{demo}-1" --task "demo-impl"
    2. Post-edit: npx claude-flow@alpha hooks post-edit --file "demos/{demo}/src/{module}.rs"
    3. Post-task: npx claude-flow@alpha hooks post-task --task-id "demo-impl" --status "success"

    MEMORY:
    - Read product APIs: swarm/product/*/api
    - Store demo state: swarm/demo/{demo}/state
    - Store integration status: swarm/demo/{demo}/integration

    HANDOFF: When complete, hand off to tester-demo-{demo}
  `, "backend-dev")
```

---

### 10.3 Infrastructure Agent Spawn

```javascript
// Template for spawning infrastructure agents

[Single Message - Infrastructure Agent Spawn]:
  Task("System Architect", `
    You are system-architect-1.

    TASK: Design overall system architecture and integration contracts.

    FILES: docs/architecture/*.md

    HOOKS:
    1. Pre-task: npx claude-flow@alpha hooks pre-task --agent "system-architect-1" --task "architecture"
    2. Post-edit: npx claude-flow@alpha hooks post-edit --file "docs/architecture/SYSTEM_DESIGN.md"
    3. Post-task: npx claude-flow@alpha hooks post-task --task-id "architecture" --status "success"

    MEMORY:
    - Store architectural decisions: swarm/infra/architecture/decisions
    - Store API contracts: swarm/product/*/api

    DELIVERABLES:
    - System architecture diagram
    - API contracts for all 4 products
    - Integration patterns
  `, "system-architect"),

  Task("CI/CD Engineer", `
    You are cicd-engineer-1.

    TASK: Setup GitHub Actions CI/CD pipelines, Docker configs, and deployment automation.

    FILES: .github/workflows/*.yml, Dockerfile, scripts/ci/*.sh

    HOOKS:
    1. Pre-task: npx claude-flow@alpha hooks pre-task --agent "cicd-engineer-1" --task "cicd-setup"
    2. Post-edit: npx claude-flow@alpha hooks post-edit --file ".github/workflows/ci.yml"
    3. Post-task: npx claude-flow@alpha hooks post-task --task-id "cicd-setup" --status "success"

    MEMORY:
    - Store pipeline status: swarm/infra/cicd/status

    DELIVERABLES:
    - GitHub Actions workflows (test, build, deploy)
    - Docker multi-stage builds
    - Deployment scripts
  `, "cicd-engineer")
```

---

## 11. Monitoring and Metrics

### 11.1 Agent Performance Metrics

**Tracked Metrics:**
- Task completion time
- File edit count
- Test coverage contribution
- Code quality (clippy warnings)
- Memory usage (keys stored/retrieved)
- Handoff latency (time between completion and next agent start)

**Storage:**
```bash
npx claude-flow@alpha memory store --key "swarm/metrics/agent/{agent_id}" \
  --value '{
    "tasks_completed": 12,
    "avg_completion_time_hours": 4.2,
    "files_edited": 45,
    "tests_written": 89,
    "coverage_contribution": 12.5,
    "clippy_warnings": 3
  }'
```

---

### 11.2 Phase Progress Tracking

**Daily Progress Update:**
```bash
# Run at end of each day
npx claude-flow@alpha memory store --key "swarm/queen/phase_progress" \
  --value '{
    "phase": "R1",
    "day": 42,
    "completion_percentage": 65,
    "tasks_completed_today": 8,
    "blockers": ["Performance regression in metering"],
    "next_milestone": "R1 → R2 transition (Day 70)"
  }'
```

---

### 11.3 Quality Gate Dashboard

**Gate Status:**
```bash
npx claude-flow@alpha memory retrieve --key "swarm/queen/quality_gates"

# Output:
{
  "P_to_A": {
    "status": "passed",
    "date": "2025-01-09",
    "criteria": ["skeletons", "compile"],
    "passed": true
  },
  "A_to_R1": {
    "status": "passed",
    "date": "2025-01-23",
    "criteria": ["architecture", "cicd"],
    "passed": true
  },
  "R1_to_R2": {
    "status": "in_progress",
    "target_date": "2025-03-06",
    "criteria": ["tests", "coverage_80"],
    "passed": false,
    "progress": {
      "tests": true,
      "coverage_80": false,
      "current_coverage": 72
    }
  }
}
```

---

## 12. Failure Recovery and Rollback

### 12.1 Agent Failure Recovery

**Failure Detection:**
- Agent doesn't update memory for 4 hours
- Post-task hook reports failure status
- Quality gate validation fails

**Recovery Actions:**

1. **Automatic Retry:**
   ```bash
   # sparc-coord detects failure
   npx claude-flow@alpha memory retrieve --key "swarm/product/metering/state"
   # State shows "in_progress" for 4+ hours

   # Spawn backup agent
   Task("Backup Coder", "Resume work on metering tracker. Previous agent timed out.", "coder")
   ```

2. **Rollback:**
   ```bash
   # If changes broke tests
   git checkout HEAD~1 metering/src/tracker.rs

   # Update memory
   npx claude-flow@alpha memory store --key "swarm/product/metering/state" \
     --value '{"status": "rolled_back", "reason": "test_failure"}'
   ```

---

### 12.2 Quality Gate Failure

**Failure Scenarios:**

| Gate | Failure Reason | Recovery Action |
|------|---------------|-----------------|
| P → A | Skeleton missing | Re-spawn coder to generate skeleton |
| A → R1 | CI/CD pipeline broken | cicd-engineer-1 fixes pipeline |
| R1 → R2 | Coverage below 80% | Spawn additional tester agents |
| R2 → C | Demo test failure | backend-dev fixes integration |
| C → Done | Security audit failure | reviewer-security blocks, fix required |

**Recovery Example:**
```bash
# R1 → R2 gate fails due to low coverage (72%)
./quality-gate-check.sh R1
# Exit code: 1 (failure)

# sparc-coord spawns additional testers
Task("Extra Tester 1", "Add tests to metering until 80% coverage", "tester")
Task("Extra Tester 2", "Add tests to oversight until 80% coverage", "tester")

# Retry gate check
./quality-gate-check.sh R1
# Exit code: 0 (success)
```

---

## 13. Success Criteria

### 13.1 Per-Phase Success Criteria

**Phase P (Preparation):**
- ✅ All 4 product skeletons generated
- ✅ TDD stubs in place (failing tests)
- ✅ Cargo workspace compiles
- ✅ 140-day task breakdown complete

**Phase A (Architecture):**
- ✅ System architecture documented
- ✅ API contracts defined for all products
- ✅ CI/CD pipelines operational
- ✅ Docker configs ready

**Phase R1 (Core Refinement):**
- ✅ All 4 products implement core features
- ✅ Unit tests pass
- ✅ 80%+ test coverage
- ✅ Clippy warnings resolved

**Phase R2 (Demo Refinement):**
- ✅ All 4 demos functional
- ✅ Integration tests pass
- ✅ Performance benchmarks meet baselines
- ✅ Security audit passed

**Phase C (Completion):**
- ✅ E2E tests pass
- ✅ Documentation complete
- ✅ Deployment pipelines tested
- ✅ Final quality audit passed

---

### 13.2 Overall Project Success Criteria

**Functional:**
- All 4 products production-ready
- All 4 demos working end-to-end
- Full integration tested

**Quality:**
- 80%+ test coverage
- 0 critical security issues
- Performance meets baselines
- 0 clippy warnings

**Documentation:**
- API reference complete
- Architecture documented
- Deployment guides ready
- Demo READMEs complete

**Process:**
- All phase gates passed
- All handoffs completed
- All agents reported success
- Metrics tracked and exported

---

## 14. Appendices

### Appendix A: Agent Quick Reference

| Agent ID | Type | Primary Role | Memory Namespace |
|----------|------|--------------|------------------|
| sparc-coord | Orchestrator | Phase gates | swarm/queen |
| planner | Planner | Task breakdown | swarm/planning |
| coder-metering-1 | Coder | Metering core | swarm/product/metering |
| coder-oversight-1 | Coder | Oversight core | swarm/product/oversight |
| coder-runtime-1 | Coder | Runtime core | swarm/product/runtime |
| coder-messaging-1 | Coder | Messaging core | swarm/product/messaging |
| tester-metering-1 | Tester | Metering tests | swarm/test/metering |
| tester-oversight-1 | Tester | Oversight tests | swarm/test/oversight |
| tester-runtime-1 | Tester | Runtime tests | swarm/test/runtime |
| tester-messaging-1 | Tester | Messaging tests | swarm/test/messaging |
| backend-dev-trading-1 | Backend Dev | Trading demo | swarm/demo/trading |
| backend-dev-travel-1 | Backend Dev | Travel demo | swarm/demo/travel |
| backend-dev-healthcare-1 | Backend Dev | Healthcare demo | swarm/demo/healthcare |
| backend-dev-psa-1 | Backend Dev | PSA demo | swarm/demo/psa |
| reviewer-security | Reviewer | Security audit | swarm/review |
| system-architect-1 | Architect | System design | swarm/infra/architecture |
| cicd-engineer-1 | DevOps | CI/CD pipelines | swarm/infra/cicd |
| code-analyzer-1 | Analyzer | Performance | swarm/infra/perf |
| api-docs-1 | Documenter | API docs | swarm/docs |

---

### Appendix B: Memory Key Reference

```
swarm/
├── queen/
│   ├── current_phase
│   ├── phase_progress
│   └── quality_gates
├── planning/
│   ├── tasks
│   ├── dependencies
│   └── resource_allocation
├── product/
│   ├── metering/
│   │   ├── state
│   │   ├── api
│   │   ├── coverage
│   │   └── handoff
│   ├── oversight/
│   ├── runtime/
│   └── messaging/
├── demo/
│   ├── trading/
│   ├── travel/
│   ├── healthcare/
│   └── psa/
├── infra/
│   ├── architecture/
│   ├── cicd/
│   └── perf/
└── metrics/
    └── agent/{agent_id}
```

---

### Appendix C: Hooks Command Reference

```bash
# Pre-task hook
npx claude-flow@alpha hooks pre-task \
  --agent "{agent_id}" \
  --task "{task_id}" \
  --description "{task_description}"

# Post-edit hook
npx claude-flow@alpha hooks post-edit \
  --file "{file_path}" \
  --memory-key "swarm/{namespace}/{key}"

# Post-task hook
npx claude-flow@alpha hooks post-task \
  --task-id "{task_id}" \
  --status "{success|failure}" \
  --metrics '{json}'

# Session restore
npx claude-flow@alpha hooks session-restore \
  --session-id "swarm-{phase}-{date}"

# Session end
npx claude-flow@alpha hooks session-end \
  --session-id "swarm-{phase}-{date}" \
  --export-metrics true

# Notify
npx claude-flow@alpha hooks notify \
  --message "{notification_message}"
```

---

### Appendix D: Concurrent Execution Cheat Sheet

**✅ CORRECT - Batch everything in single message:**
```javascript
[Single Message]:
  Task("Agent 1", "...", "type")
  Task("Agent 2", "...", "type")
  Task("Agent 3", "...", "type")
  TodoWrite { todos: [...10 todos...] }
  Write "file1.rs"
  Write "file2.rs"
  Bash "command1 && command2"
```

**❌ WRONG - Multiple messages:**
```javascript
Message 1: Task("Agent 1")
Message 2: Task("Agent 2")
Message 3: TodoWrite
Message 4: Write "file"
```

---

## END OF DOCUMENT

**Next Steps:**
1. SPARC coordinator (you) initializes swarm topology
2. Spawn agents according to current phase (P, A, R1, R2, C)
3. Monitor quality gates and phase transitions
4. Coordinate handoffs via memory
5. Ensure hooks are executed by all agents

**Remember:**
- **Claude Code's Task tool spawns ALL agents that do actual work**
- **MCP tools ONLY coordinate (optional for complex tasks)**
- **Batch ALL operations in single messages**
- **Use hooks for coordination**
- **Store state in memory**
- **Follow phase gates strictly**
