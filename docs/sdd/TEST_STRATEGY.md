# Test Strategy - Creto Enablement Layer

## Executive Summary

This document defines the comprehensive testing strategy for the Creto Enablement Layer, emphasizing Test-Driven Development (TDD), property-based testing for cryptographic operations, and chaos engineering for distributed scenarios. Our approach ensures reliability, security, and performance across all four core crates.

---

## 1. Testing Philosophy

### Core Principles

**Test-Driven Development (TDD)**
- Tests MUST be written BEFORE implementation code
- Red-Green-Refactor cycle for all new features
- Tests serve as executable specifications
- Regression prevention through comprehensive coverage

**London School TDD**
- Mock external dependencies to isolate units
- Interaction-based testing for component boundaries
- Fast test execution through minimized external calls
- Clear separation between unit and integration tests

**Property-Based Testing**
- Mandatory for all cryptographic operations
- Use `proptest` for invariant verification
- Generate random inputs to discover edge cases
- Validate mathematical properties (e.g., encrypt → decrypt = identity)

**Chaos Engineering**
- Simulate network partitions, latency, and failures
- Test distributed consensus under adversarial conditions
- Verify graceful degradation and recovery
- Use `chaos-mesh` patterns for distributed testing

---

## 2. Test Pyramid

```
                    ┌───────────────┐
                    │     E2E       │  10% (4 demo flows)
                    │   End-to-End  │  - Trading workflow
                    │     Tests     │  - Travel booking
                    │               │  - Healthcare diagnosis
                    │               │  - PSA engagement
                    ├───────────────┤
                    │ Integration   │  30% (cross-product)
                    │    Tests      │  - Inter-crate APIs
                    │               │  - External system mocks
                    │               │  - State transitions
                    ├───────────────┤
                    │     Unit      │  60% (per-crate)
                    │    Tests      │  - Pure functions
                    │               │  - Error conditions
                    │               │  - Edge cases
                    └───────────────┘
```

### Distribution Rationale

- **60% Unit Tests**: Fast feedback, isolated failures, comprehensive coverage
- **30% Integration Tests**: Cross-component contracts, realistic scenarios
- **10% E2E Tests**: Critical paths, user-facing workflows, smoke tests

---

## 3. Coverage Requirements

### Per-Crate Targets

| Crate | Line Coverage | Branch Coverage | Critical Paths | Priority Functions |
|-------|---------------|-----------------|----------------|-------------------|
| **creto-metering** | 85% | 75% | `quota_check`, `event_ingest` | `record_event`, `enforce_quota`, `aggregate_metrics` |
| **creto-oversight** | 85% | 75% | `state_machine`, `approval_flow` | `create_approval_request`, `transition_state`, `apply_policy` |
| **creto-runtime** | 80% | 70% | `sandbox_spawn`, `egress_check` | `warm_pool_claim`, `inject_secrets`, `monitor_resource` |
| **creto-messaging** | 90% | 80% | `encrypt`, `decrypt`, `ratchet` | `double_ratchet_step`, `derive_keys`, `verify_signature` |

### Coverage Calculation

```bash
# Generate coverage reports
cargo tarpaulin --workspace --out Html --out Xml \
  --exclude-files "tests/*" \
  --engine llvm

# Enforce minimum thresholds
cargo tarpaulin --fail-under 85 --workspace
```

### Critical Path Definition

Critical paths are functions that:
1. Handle security-sensitive operations (auth, crypto, secrets)
2. Affect billing/metering accuracy
3. Control state transitions in approval workflows
4. Manage resource allocation/deallocation

---

## 4. Test Categories

### 4.1 Unit Tests (Per-Crate)

**Scope**: Test individual functions/methods in isolation

**Requirements**:
- All public API functions MUST have unit tests
- All error conditions MUST be tested
- Edge cases from SDD MUST be covered

**Example Structure**:

```rust
// creto-metering/src/quota.rs
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_quota_check_under_limit() {
        let mut mock_store = MockQuotaStore::new();
        mock_store.expect_get_usage()
            .with(eq("agent-123"))
            .returning(|_| Ok(50));

        let checker = QuotaChecker::new(mock_store);
        assert!(checker.check("agent-123", 100).unwrap());
    }

    #[test]
    fn test_quota_check_over_limit() {
        let mut mock_store = MockQuotaStore::new();
        mock_store.expect_get_usage()
            .with(eq("agent-123"))
            .returning(|_| Ok(150));

        let checker = QuotaChecker::new(mock_store);
        assert!(!checker.check("agent-123", 100).unwrap());
    }

    #[test]
    fn test_quota_check_store_error() {
        let mut mock_store = MockQuotaStore::new();
        mock_store.expect_get_usage()
            .returning(|_| Err(StoreError::Unavailable));

        let checker = QuotaChecker::new(mock_store);
        assert!(matches!(
            checker.check("agent-123", 100),
            Err(QuotaError::StoreError(_))
        ));
    }
}
```

### 4.2 Integration Tests (Cross-Crate)

**Scope**: Test interactions between multiple crates

**Key Integration Points**:

1. **Metering + AuthZ**: Quota enforcement flow
   ```rust
   #[test]
   fn test_quota_enforcement_blocks_unauthorized() {
       let metering = MeteringService::new();
       let authz = AuthZService::new();

       // Set quota limit
       metering.set_quota("agent-123", 100);
       metering.record_usage("agent-123", 100);

       // Attempt operation over quota
       let result = authz.authorize("agent-123", Operation::ApiCall);
       assert!(matches!(result, Err(AuthError::QuotaExceeded)));
   }
   ```

2. **Oversight + Memory**: Context fetching for approvals
   ```rust
   #[test]
   fn test_approval_request_fetches_context() {
       let oversight = OversightService::new();
       let memory = MemoryService::new();

       memory.store("context-123", json!({"risk": "high"}));

       let request = oversight.create_approval_request(
           "workflow-456",
           "context-123"
       );

       assert_eq!(request.context["risk"], "high");
   }
   ```

3. **Runtime + Vault**: Secret injection into sandboxes
   ```rust
   #[test]
   fn test_sandbox_receives_secrets() {
       let runtime = RuntimeService::new();
       let vault = VaultService::new();

       vault.store("api-key", "secret-123");

       let sandbox = runtime.spawn_sandbox(SandboxConfig {
           secrets: vec!["api-key"],
           ..Default::default()
       });

       assert_eq!(sandbox.env_var("API_KEY"), Some("secret-123"));
   }
   ```

4. **Messaging + NHI**: Key exchange protocol
   ```rust
   #[test]
   fn test_agent_key_exchange() {
       let messaging = MessagingService::new();
       let nhi = NHIService::new();

       let agent_a = nhi.create_identity("agent-a");
       let agent_b = nhi.create_identity("agent-b");

       let session = messaging.establish_session(&agent_a, &agent_b);

       // Verify shared secret established
       assert!(session.can_encrypt());
       assert!(session.can_decrypt());
   }
   ```

### 4.3 End-to-End Tests (Demo Flows)

**Scope**: Complete user workflows across all crates

**1. Trading Workflow**
```rust
#[tokio::test]
async fn test_trading_workflow_end_to_end() {
    // Setup: Initialize all services
    let system = CretoSystem::new().await;

    // Step 1: Agent receives trade signal
    let order = system.submit_order(Order {
        symbol: "AAPL",
        quantity: 100,
        action: TradeAction::Buy,
    }).await.unwrap();

    // Step 2: Authorization check (quota, permissions)
    assert!(system.authz.check_permission(&order).await.is_ok());

    // Step 3: Execute trade (runtime sandbox)
    let result = system.runtime.execute_trade(&order).await.unwrap();

    // Step 4: Audit trail recorded
    let audit = system.oversight.get_audit_trail(&order.id).await.unwrap();
    assert_eq!(audit.events.len(), 3); // submit, authorize, execute

    // Verify metering
    let usage = system.metering.get_usage(&order.agent_id).await.unwrap();
    assert!(usage > 0);
}
```

**2. Travel Booking**
```rust
#[tokio::test]
async fn test_travel_booking_parallel_agents() {
    let system = CretoSystem::new().await;

    let request = TravelRequest {
        destination: "Paris",
        dates: DateRange::new("2025-06-01", "2025-06-07"),
        budget: 5000,
    };

    // Parallel booking agents
    let (flight, hotel, car) = tokio::join!(
        system.agents.book_flight(&request),
        system.agents.book_hotel(&request),
        system.agents.book_car(&request),
    );

    // Approval workflow
    let itinerary = Itinerary::combine(flight?, hotel?, car?);
    let approval = system.oversight.request_approval(itinerary).await.unwrap();

    assert_eq!(approval.state, ApprovalState::Pending);

    // Human approves
    system.oversight.approve(approval.id, "user-123").await.unwrap();

    // Confirm bookings
    let confirmed = system.agents.confirm_bookings(&itinerary).await.unwrap();
    assert!(confirmed);
}
```

**3. Healthcare Diagnosis**
```rust
#[tokio::test]
async fn test_healthcare_diagnosis_workflow() {
    let system = CretoSystem::new().await;

    let patient = PatientData {
        id: "patient-456",
        symptoms: vec!["fever", "cough", "fatigue"],
        history: load_medical_history(),
    };

    // Agent analyzes symptoms
    let diagnosis = system.agents.diagnose(&patient).await.unwrap();

    // Requires physician approval for prescription
    if diagnosis.requires_prescription {
        let approval = system.oversight.request_approval(
            ApprovalRequest::new(diagnosis.clone())
                .require_role("physician")
        ).await.unwrap();

        // Physician reviews and approves
        system.oversight.approve(approval.id, "dr-smith").await.unwrap();
    }

    // Document decision
    let audit = system.oversight.get_audit_trail(&patient.id).await.unwrap();
    assert!(audit.contains_approval());
}
```

**4. PSA Engagement Analysis**
```rust
#[tokio::test]
async fn test_psa_engagement_workflow() {
    let system = CretoSystem::new().await;

    let engagement = Engagement {
        client: "acme-corp",
        project: "digital-transformation",
        duration_weeks: 12,
    };

    // Agent analyzes scope
    let analysis = system.agents.analyze_engagement(&engagement).await.unwrap();

    // Review by senior consultant
    let review = system.oversight.request_review(analysis).await.unwrap();
    system.oversight.complete_review(review.id, ReviewDecision::Approved).await.unwrap();

    // Sign-off creates binding contract
    let contract = system.agents.generate_contract(&engagement).await.unwrap();
    let signed = system.oversight.sign_off(contract.id, "partner-123").await.unwrap();

    assert!(signed.is_binding());
}
```

---

## 5. Performance Tests

### Benchmarking Strategy

Use `criterion` for all performance benchmarks with statistical rigor.

### Target Latencies

| Test | Target | Percentile | Tool | Rationale |
|------|--------|------------|------|-----------|
| **Quota check latency** | <10µs | p99 | criterion | Hot path, called on every operation |
| **Event ingestion throughput** | >10K/s | sustained | criterion | High-volume telemetry |
| **Oversight state transition** | <1ms | p95 | criterion | User-facing approval latency |
| **Message encryption** | >100K msg/s | sustained | criterion | Agent-to-agent communication |
| **Warm pool claim** | <100ms | p99 | criterion | Sandbox startup time |

### Benchmark Implementation

```rust
// benches/quota_check.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_quota_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("quota_check");

    for num_agents in [10, 100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_agents),
            num_agents,
            |b, &num_agents| {
                let checker = setup_quota_checker(num_agents);
                b.iter(|| {
                    checker.check(black_box("agent-123"), black_box(100))
                });
            }
        );
    }

    group.finish();
}

criterion_group!(benches, bench_quota_check);
criterion_main!(benches);
```

### Performance Regression Detection

```yaml
# .github/workflows/benchmark.yml
- name: Run benchmarks
  run: cargo bench --workspace -- --save-baseline main

- name: Compare to baseline
  run: cargo bench --workspace -- --baseline main --load-baseline main

- name: Fail on regression
  run: |
    if grep "Performance has regressed" bench_results.txt; then
      exit 1
    fi
```

---

## 6. Security Tests

### 6.1 Fuzzing

**Tool**: `cargo-fuzz` (libFuzzer integration)

**Targets**:
- All parsers (JSON, CBOR, Protobuf)
- Cryptographic primitives
- State machine transitions
- Input validation functions

```rust
// fuzz/fuzz_targets/quota_parser.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use creto_metering::parse_quota_request;

fuzz_target!(|data: &[u8]| {
    // Should never panic or crash
    let _ = parse_quota_request(data);
});
```

**CI Integration**:
```bash
# Run fuzzing for 10 minutes per target
cargo fuzz run quota_parser -- -max_total_time=600
cargo fuzz run approval_parser -- -max_total_time=600
```

### 6.2 Property-Based Testing

**Tool**: `proptest`

**Cryptographic Properties**:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encrypt_decrypt_roundtrip(plaintext in any::<Vec<u8>>()) {
        let key = generate_key();
        let ciphertext = encrypt(&key, &plaintext).unwrap();
        let decrypted = decrypt(&key, &ciphertext).unwrap();

        prop_assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_signature_verification(message in any::<Vec<u8>>()) {
        let keypair = generate_keypair();
        let signature = sign(&keypair.secret, &message).unwrap();

        prop_assert!(verify(&keypair.public, &message, &signature));
    }

    #[test]
    fn test_quota_commutative(
        events in prop::collection::vec(any::<Event>(), 1..100)
    ) {
        let mut quota_a = QuotaTracker::new(1000);
        let mut quota_b = QuotaTracker::new(1000);

        // Apply events in order
        for event in &events {
            quota_a.apply(event);
        }

        // Apply events in reverse
        for event in events.iter().rev() {
            quota_b.apply(event);
        }

        // Result should be the same (commutative)
        prop_assert_eq!(quota_a.remaining(), quota_b.remaining());
    }
}
```

### 6.3 Mutation Testing

**Tool**: `cargo-mutants`

**Purpose**: Verify test suite kills all mutations

```bash
# Generate mutations and run tests
cargo mutants --workspace

# Require 90% mutation kill rate
cargo mutants --check --minimum-test-score 90
```

### 6.4 Dependency Auditing

**Tool**: `cargo-audit`

```bash
# Check for known vulnerabilities
cargo audit

# Fail CI on vulnerabilities
cargo audit --deny warnings
```

**Automated Updates**:
```yaml
# .github/workflows/audit.yml
- name: Security audit
  run: |
    cargo audit --json > audit.json
    if [ $(jq '.vulnerabilities.count' audit.json) -gt 0 ]; then
      echo "Vulnerabilities found!"
      exit 1
    fi
```

---

## 7. Mock Definitions

### 7.1 External Dependency Mocks

For each external crate (`creto-authz`, `creto-vault`, `creto-memory`, `creto-nhi`), define:

**Mock Trait Implementation**:
```rust
// tests/mocks/authz.rs
use mockall::mock;
use creto_authz::{AuthZService, Permission, Decision};

mock! {
    pub AuthZService {}

    impl AuthZService for AuthZService {
        fn check_permission(&self, subject: &str, resource: &str, action: &str)
            -> Result<Decision, AuthZError>;

        fn grant_permission(&self, subject: &str, resource: &str, action: &str)
            -> Result<(), AuthZError>;

        fn revoke_permission(&self, subject: &str, resource: &str, action: &str)
            -> Result<(), AuthZError>;
    }
}
```

**Test Fixtures**:
```rust
// tests/fixtures/mod.rs
pub struct TestFixtures;

impl TestFixtures {
    pub fn default_quota() -> QuotaConfig {
        QuotaConfig {
            limit: 1000,
            period: Duration::from_secs(3600),
            burst: 100,
        }
    }

    pub fn sample_agent() -> Agent {
        Agent {
            id: "test-agent-123".into(),
            name: "Test Agent".into(),
            capabilities: vec!["read", "write"],
        }
    }

    pub fn approval_request() -> ApprovalRequest {
        ApprovalRequest {
            id: "approval-456".into(),
            workflow_id: "workflow-789".into(),
            state: ApprovalState::Pending,
            created_at: SystemTime::now(),
        }
    }
}
```

**Error Simulation**:
```rust
// tests/mocks/error_injector.rs
pub struct ErrorInjector {
    failure_rate: f64,
}

impl ErrorInjector {
    pub fn new(failure_rate: f64) -> Self {
        Self { failure_rate }
    }

    pub fn maybe_fail<T, E>(&self, ok: T, err: E) -> Result<T, E> {
        if rand::random::<f64>() < self.failure_rate {
            Err(err)
        } else {
            Ok(ok)
        }
    }
}

// Usage in tests
#[test]
fn test_retry_on_transient_failure() {
    let injector = ErrorInjector::new(0.5); // 50% failure rate
    let mut mock_store = MockQuotaStore::new();

    mock_store.expect_get_usage()
        .returning(move |_| {
            injector.maybe_fail(100, StoreError::Timeout)
        });

    // Should retry and eventually succeed
    let result = retry_with_backoff(|| mock_store.get_usage("agent-123"));
    assert!(result.is_ok());
}
```

---

## 8. CI/CD Integration

### 8.1 GitHub Actions Workflow

```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  pull_request:
  push:
    branches: [main, develop]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  format:
    name: Code Formatting
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  lint:
    name: Clippy Lints
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo clippy --workspace --all-targets -- -D warnings

  test:
    name: Unit Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace --lib --bins

  integration:
    name: Integration Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --workspace --features integration --test '*'

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: |
          cargo tarpaulin --workspace \
            --out Xml \
            --exclude-files "tests/*" \
            --engine llvm \
            --fail-under 85
      - name: Upload to codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./cobertura.xml

  benchmarks:
    name: Performance Benchmarks
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo bench --workspace --no-run  # Compile but don't run

  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-audit
      - run: cargo audit --deny warnings

  fuzz:
    name: Fuzzing
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo install cargo-fuzz
      - run: |
          for target in fuzz/fuzz_targets/*; do
            cargo +nightly fuzz run $(basename $target .rs) -- -max_total_time=300
          done
```

### 8.2 Pre-commit Hooks

```bash
# .git/hooks/pre-commit
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --workspace --all-targets -- -D warnings

# Unit tests
cargo test --workspace --lib

echo "All checks passed!"
```

---

## 9. Quality Gates

### 9.1 Pull Request Requirements

| Gate | Criteria | Enforcement | Blocking |
|------|----------|-------------|----------|
| **Formatting** | `cargo fmt --check` passes | CI required | Yes |
| **Linting** | 0 clippy warnings (`-D warnings`) | CI required | Yes |
| **Unit Tests** | All tests pass | CI required | Yes |
| **Integration Tests** | All tests pass | CI required | Yes |
| **Code Coverage** | ≥85% line coverage | CI required | Yes |
| **Branch Coverage** | ≥75% branch coverage | CI required | Yes |
| **Security Audit** | 0 known vulnerabilities | CI required | Yes |
| **Performance** | Benchmarks compile | CI required | No |
| **Documentation** | Public APIs documented | Manual review | No |

### 9.2 Release Requirements

| Gate | Criteria | Enforcement | Blocking |
|------|----------|-------------|----------|
| **All PR Gates** | Must pass all PR requirements | CI + Manual | Yes |
| **E2E Tests** | All 4 demo flows pass | CI required | Yes |
| **Benchmarks** | Meet latency/throughput targets | Manual review | Yes |
| **Load Testing** | Sustained load for 1 hour | Manual test | Yes |
| **Chaos Testing** | Survives network partitions | Manual test | Yes |
| **Documentation** | README, CHANGELOG updated | Manual review | Yes |
| **Version Bump** | Semantic versioning applied | Manual review | Yes |

### 9.3 Coverage Enforcement

```toml
# .tarpaulin.toml
[tarpaulin]
command = "test"
workspace = true
exclude-files = ["tests/*", "benches/*", "examples/*"]
engine = "llvm"
fail-under = 85
branch = true
```

### 9.4 Continuous Monitoring

```yaml
# Post-merge monitoring
- name: Track coverage trends
  run: |
    COVERAGE=$(cargo tarpaulin --out Json | jq '.coverage')
    echo "coverage=$COVERAGE" >> $GITHUB_ENV

- name: Alert on degradation
  if: env.coverage < 85
  run: |
    echo "::error::Coverage dropped below 85%"
    exit 1
```

---

## 10. Testing Tools & Dependencies

### 10.1 Core Testing Crates

```toml
# Cargo.toml [dev-dependencies]
[dev-dependencies]
# Testing frameworks
tokio-test = "0.4"
proptest = "1.4"
quickcheck = "1.0"

# Mocking
mockall = "0.12"
wiremock = "0.6"

# Assertions
assert_matches = "1.5"
pretty_assertions = "1.4"

# Benchmarking
criterion = { version = "0.5", features = ["html_reports"] }

# Coverage
tarpaulin = "0.27"

# Fuzzing (requires nightly)
cargo-fuzz = "0.11"

# Security
cargo-audit = "0.18"
cargo-mutants = "23.11"
```

### 10.2 Testing Utilities

```rust
// tests/common/mod.rs
pub mod setup {
    use tempfile::TempDir;

    pub fn test_database() -> (Database, TempDir) {
        let dir = TempDir::new().unwrap();
        let db = Database::open(dir.path()).unwrap();
        (db, dir)
    }

    pub fn test_runtime() -> Runtime {
        Runtime::new().unwrap()
    }
}

pub mod assertions {
    pub fn assert_within_tolerance(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() < tolerance,
            "Expected {} ± {}, got {}",
            expected, tolerance, actual
        );
    }
}
```

---

## 11. Chaos Testing Scenarios

### 11.1 Network Partition

```rust
#[tokio::test]
async fn test_consensus_under_partition() {
    let mut cluster = TestCluster::new(5).await;

    // Partition cluster into two groups
    cluster.partition(&[0, 1], &[2, 3, 4]).await;

    // Attempt to reach consensus
    let result = cluster.nodes[0].propose_value("test").await;

    // Should timeout or return no quorum
    assert!(matches!(result, Err(ConsensusError::NoQuorum)));

    // Heal partition
    cluster.heal_partition().await;

    // Should now succeed
    let result = cluster.nodes[0].propose_value("test").await;
    assert!(result.is_ok());
}
```

### 11.2 Byzantine Failures

```rust
#[tokio::test]
async fn test_byzantine_node_detection() {
    let mut cluster = TestCluster::new(7).await;

    // Make 2 nodes Byzantine (< 1/3)
    cluster.nodes[1].set_byzantine(true);
    cluster.nodes[2].set_byzantine(true);

    // Honest nodes should detect and exclude them
    let result = cluster.reach_consensus("value").await.unwrap();

    assert_eq!(result.participating_nodes, 5);
    assert!(!result.participants.contains(&1));
    assert!(!result.participants.contains(&2));
}
```

---

## 12. Test Data Management

### 12.1 Test Fixtures Repository

```
tests/
├── fixtures/
│   ├── agents.json       # Sample agent configurations
│   ├── quotas.json       # Quota test cases
│   ├── approvals.json    # Approval workflows
│   └── messages.json     # Encrypted message samples
```

### 12.2 Fixture Loading

```rust
// tests/common/fixtures.rs
use serde::de::DeserializeOwned;

pub fn load_fixture<T: DeserializeOwned>(name: &str) -> T {
    let path = format!("tests/fixtures/{}.json", name);
    let content = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&content).unwrap()
}

// Usage
let agents: Vec<Agent> = load_fixture("agents");
```

---

## 13. Conclusion

This test strategy ensures the Creto Enablement Layer achieves:

✅ **High Reliability**: 85%+ coverage, property-based testing
✅ **Security Assurance**: Fuzzing, mutation testing, dependency audits
✅ **Performance Validation**: Sub-millisecond latencies, >10K ops/sec throughput
✅ **Chaos Resilience**: Survives network partitions and Byzantine failures
✅ **Continuous Quality**: Automated CI/CD gates, coverage tracking

**Next Steps**:
1. Implement mock trait definitions for all external dependencies
2. Write initial unit tests for critical paths (quota_check, state_machine)
3. Set up CI pipeline with coverage enforcement
4. Create E2E test harness for demo workflows
5. Configure fuzzing targets for all parsers

---

**Document Version**: 1.0
**Last Updated**: 2025-12-26
**Status**: ✅ Complete - Ready for Implementation
