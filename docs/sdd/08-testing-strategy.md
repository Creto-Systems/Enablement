---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
---

# SDD-08: Testing Strategy

## Purpose

This document defines the testing strategy for the Enablement Layer, including test levels, coverage requirements, test infrastructure, and quality gates.

## Scope

**In Scope:**
- Test levels and types
- Coverage requirements
- Test infrastructure
- Quality gates for CI/CD
- Performance testing
- Security testing

**Out of Scope:**
- Platform layer testing
- User acceptance testing procedures
- Compliance certification testing

---

## 1. Testing Philosophy

### 1.1 Principles

| Principle | Implementation |
|-----------|----------------|
| **Test pyramid** | More unit tests, fewer E2E tests |
| **Shift left** | Test early, test often |
| **Automation first** | Manual testing only for exploratory |
| **Fast feedback** | Tests complete in <10 minutes |
| **Deterministic** | No flaky tests in CI |
| **Isolated** | Tests don't depend on external services |

### 1.2 Test Pyramid

```
                    ┌─────────────┐
                    │   E2E (5%)  │  Full system, multi-product
                    ├─────────────┤
                 ┌──┴─────────────┴──┐
                 │ Integration (20%)  │  Cross-crate, external deps
                 ├────────────────────┤
              ┌──┴────────────────────┴──┐
              │      Unit Tests (75%)     │  Single function/module
              └───────────────────────────┘
```

---

## 2. Test Levels

### 2.1 Unit Tests

**Scope:** Single function, module, or struct in isolation.

**Location:** `crates/*/src/**/*.rs` (inline `#[cfg(test)]` modules)

**Characteristics:**
- No I/O (mocked)
- No network calls
- No database access
- Execute in <1ms each
- Run in parallel

**Example:**

```rust
// crates/creto-metering/src/aggregation.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sum_aggregator_sums_values() {
        let mut agg = SumAggregator::new();
        agg.add(10.0);
        agg.add(20.0);
        agg.add(5.5);

        assert_eq!(agg.result(), 35.5);
    }

    #[test]
    fn unique_count_aggregator_deduplicates() {
        let mut agg = UniqueCountAggregator::new();
        agg.add("user_1");
        agg.add("user_2");
        agg.add("user_1"); // Duplicate

        assert_eq!(agg.result(), 2);
    }

    #[test]
    fn tiered_pricing_applies_correct_tier() {
        let tiers = vec![
            Tier { up_to: 100, unit_amount: 10 },
            Tier { up_to: 1000, unit_amount: 5 },
            Tier { up_to: u64::MAX, unit_amount: 1 },
        ];
        let pricer = TieredPricer::new(tiers);

        assert_eq!(pricer.calculate(50), 500);    // 50 * 10
        assert_eq!(pricer.calculate(150), 1250);  // 100*10 + 50*5
    }
}
```

### 2.2 Integration Tests

**Scope:** Cross-crate interactions, external dependencies (database, cache).

**Location:** `crates/*/tests/*.rs` and `tests/integration/*.rs`

**Characteristics:**
- Use real database (containerized)
- Use real Redis (containerized)
- Mock external services (NHI, AuthZ)
- Execute in <30s each
- Run sequentially per test file

**Example:**

```rust
// crates/creto-metering/tests/event_ingestion_test.rs

use creto_metering::{EventIngester, BillableEvent};
use testcontainers::{clients::Cli, images::redis::Redis};

#[tokio::test]
async fn test_event_ingestion_with_redis() {
    let docker = Cli::default();
    let redis_node = docker.run(Redis::default());
    let redis_url = format!("redis://localhost:{}", redis_node.get_host_port_ipv4(6379));

    let ingester = EventIngester::new(&redis_url).await.unwrap();

    let event = BillableEvent {
        idempotency_key: "test-123".to_string(),
        agent_nhi: test_agent_identity(),
        event_type: "api_call".to_string(),
        timestamp: Timestamp::now(),
        properties: HashMap::new(),
    };

    let result = ingester.ingest(event.clone()).await;
    assert!(result.is_ok());

    // Verify idempotency - second ingest should return same ID
    let result2 = ingester.ingest(event).await;
    assert_eq!(result.unwrap().event_id, result2.unwrap().event_id);
}
```

### 2.3 End-to-End Tests

**Scope:** Full system behavior across multiple products.

**Location:** `tests/e2e/*.rs`

**Characteristics:**
- Full Kubernetes deployment (Kind)
- Real service-to-service calls
- Test complete workflows
- Execute in <5 minutes
- Run in CI on main branch

**Example:**

```rust
// tests/e2e/metering_oversight_flow.rs

#[tokio::test]
async fn high_cost_operation_triggers_oversight() {
    let cluster = TestCluster::deploy().await;

    // Configure oversight policy for expensive operations
    cluster.oversight.create_policy(OversightPolicy {
        trigger: Trigger::CostExceeds { threshold: Money::usd(100) },
        approvers: vec!["human@example.com".into()],
        timeout: Duration::minutes(30),
    }).await.unwrap();

    // Agent attempts expensive operation
    let result = cluster.metering.record_event(BillableEvent {
        event_type: "expensive_computation".to_string(),
        properties: props!{ "cost" => "150.00" },
        ..default_event()
    }).await;

    // Should be blocked pending oversight
    assert!(matches!(result, Err(Error::OversightRequired { request_id })));

    // Verify oversight request was created
    let requests = cluster.oversight.list_pending().await.unwrap();
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].status, OversightStatus::Pending);
}
```

---

## 3. Test Types

### 3.1 Functional Tests

| Product | Key Test Scenarios |
|---------|-------------------|
| **Metering** | Event ingestion, aggregation, quota enforcement, billing calculation |
| **Oversight** | Policy matching, approval flow, timeout handling, escalation |
| **Runtime** | Sandbox lifecycle, warm pool, network policy, attestation |
| **Messaging** | Encryption/decryption, delivery, channel routing |

### 3.2 Property-Based Tests

Use `proptest` for invariant verification:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn quota_never_goes_negative(
        initial in 0u64..1000,
        operations in prop::collection::vec(0u64..100, 0..50)
    ) {
        let mut quota = QuotaTracker::with_limit(initial);

        for amount in operations {
            let _ = quota.consume(amount);
            prop_assert!(quota.remaining() >= 0);
        }
    }

    #[test]
    fn message_encryption_roundtrip(
        plaintext in prop::collection::vec(any::<u8>(), 0..10000)
    ) {
        let (pk, sk) = generate_keypair();
        let encrypted = encrypt(&plaintext, &pk).unwrap();
        let decrypted = decrypt(&encrypted, &sk).unwrap();

        prop_assert_eq!(plaintext, decrypted);
    }
}
```

### 3.3 Fuzz Tests

Use `cargo-fuzz` for security-critical code:

```rust
// fuzz/fuzz_targets/event_parsing.rs

#![no_main]
use libfuzzer_sys::fuzz_target;
use creto_metering::BillableEvent;

fuzz_target!(|data: &[u8]| {
    // Should never panic or crash
    let _ = BillableEvent::try_from_bytes(data);
});
```

Fuzz targets:
- Event parsing
- Message envelope parsing
- Policy expression evaluation
- Signature verification

### 3.4 Contract Tests

Verify API compatibility with consumers:

```rust
// tests/contract/metering_api_test.rs

use pact_consumer::prelude::*;

#[tokio::test]
async fn metering_event_contract() {
    let pact = PactBuilder::new("agent-sdk", "creto-metering")
        .interaction("ingest event", "", |mut i| {
            i.given("metering service is available");
            i.request.method("POST");
            i.request.path("/v1/events");
            i.request.json_body(json!({
                "idempotency_key": like!("uuid-string"),
                "event_type": like!("api_call"),
                "timestamp": like!("2024-01-01T00:00:00Z"),
                "properties": {}
            }));
            i.response.status(201);
            i.response.json_body(json!({
                "event_id": like!("evt_123"),
                "received_at": like!("2024-01-01T00:00:00Z")
            }));
            i
        })
        .build();

    pact.verify().await;
}
```

---

## 4. Coverage Requirements

### 4.1 Coverage Targets

| Level | Target | Measurement |
|-------|--------|-------------|
| **Line coverage** | ≥80% | `cargo llvm-cov` |
| **Branch coverage** | ≥70% | `cargo llvm-cov` |
| **Critical paths** | 100% | Manual review |

### 4.2 Critical Paths (100% Required)

- Quota enforcement logic
- Authorization checks
- Signature verification
- Encryption/decryption
- Policy evaluation
- State machine transitions

### 4.3 Coverage Enforcement

```yaml
# .github/workflows/coverage.yml
- name: Check coverage
  run: |
    cargo llvm-cov --workspace --lcov --output-path lcov.info
    COVERAGE=$(cargo llvm-cov report --summary-only | grep "Total" | awk '{print $NF}' | tr -d '%')
    if (( $(echo "$COVERAGE < 80" | bc -l) )); then
      echo "Coverage $COVERAGE% is below 80% threshold"
      exit 1
    fi
```

---

## 5. Performance Testing

### 5.1 Benchmark Suite

```rust
// benches/metering_bench.rs

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn quota_check_benchmark(c: &mut Criterion) {
    let enforcer = QuotaEnforcer::new();
    let agent = test_agent();

    c.bench_function("quota_check_allow", |b| {
        b.iter(|| enforcer.check(&agent, "api_call"))
    });
}

fn event_ingestion_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let ingester = rt.block_on(EventIngester::new("redis://localhost:6379")).unwrap();

    let mut group = c.benchmark_group("event_ingestion");
    for batch_size in [1, 10, 100, 1000] {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            &batch_size,
            |b, &size| {
                let events: Vec<_> = (0..size).map(|_| random_event()).collect();
                b.iter(|| rt.block_on(ingester.ingest_batch(&events)))
            },
        );
    }
    group.finish();
}

criterion_group!(benches, quota_check_benchmark, event_ingestion_benchmark);
criterion_main!(benches);
```

### 5.2 Performance Targets

| Operation | P50 | P99 | P99.9 | Notes |
|-----------|-----|-----|-------|-------|
| Quota check (cache hit) | 5µs | 10µs | 50µs | In-memory bloom filter |
| Quota check (cache miss) | 100µs | 500µs | 1ms | Redis lookup |
| Event ingestion | 50µs | 200µs | 1ms | Single event |
| Oversight state change | 100µs | 500µs | 2ms | PostgreSQL write |
| Message encryption | 10µs | 50µs | 200µs | AES-256-GCM |
| Message delivery auth | 100µs | 500µs | 2ms | AuthZ check + routing |

### 5.3 Load Testing

```yaml
# k6 load test configuration
import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '2m', target: 100 },   // Ramp up
    { duration: '5m', target: 100 },   // Sustain
    { duration: '2m', target: 500 },   // Stress
    { duration: '5m', target: 500 },   // Sustain
    { duration: '2m', target: 0 },     // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<200', 'p(99)<500'],
    http_req_failed: ['rate<0.01'],
  },
};

export default function () {
  const res = http.post('http://metering:50051/v1/events', JSON.stringify({
    idempotency_key: `k6-${__VU}-${__ITER}`,
    event_type: 'api_call',
    timestamp: new Date().toISOString(),
    properties: {},
  }));

  check(res, {
    'status is 201': (r) => r.status === 201,
    'response time < 200ms': (r) => r.timings.duration < 200,
  });

  sleep(0.1);
}
```

---

## 6. Security Testing

### 6.1 Static Analysis

```yaml
# CI security checks
- name: Security audit
  run: |
    cargo audit
    cargo deny check

- name: SAST
  uses: github/codeql-action/analyze@v2
  with:
    languages: rust
```

### 6.2 Dynamic Testing

| Test Type | Tool | Frequency |
|-----------|------|-----------|
| Dependency vulnerabilities | `cargo audit` | Every commit |
| License compliance | `cargo deny` | Every commit |
| SAST | CodeQL | Every PR |
| Container scanning | Trivy | Every image build |
| Penetration testing | External vendor | Quarterly |

### 6.3 Cryptographic Testing

```rust
#[test]
fn hybrid_signature_requires_both() {
    let (pk, sk) = generate_hybrid_keypair();
    let message = b"test message";

    // Sign with both algorithms
    let signature = hybrid_sign(message, &sk);

    // Verification requires both signatures to be valid
    assert!(hybrid_verify(message, &signature, &pk).is_ok());

    // Corrupting Ed25519 signature should fail
    let mut corrupted = signature.clone();
    corrupted.ed25519[0] ^= 0xFF;
    assert!(hybrid_verify(message, &corrupted, &pk).is_err());

    // Corrupting ML-DSA signature should fail
    let mut corrupted = signature.clone();
    corrupted.ml_dsa[0] ^= 0xFF;
    assert!(hybrid_verify(message, &corrupted, &pk).is_err());
}
```

---

## 7. Test Infrastructure

### 7.1 Local Development

```bash
# Run all unit tests
cargo test --workspace

# Run with coverage
cargo llvm-cov --workspace

# Run integration tests (requires Docker)
docker-compose -f docker/test-deps.yml up -d
cargo test --workspace --features integration

# Run benchmarks
cargo bench -p creto-metering
```

### 7.2 CI Environment

```yaml
# .github/workflows/test.yml
jobs:
  test:
    runs-on: ubuntu-latest
    services:
      redis:
        image: redis:7
        ports: [6379:6379]
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: test
          POSTGRES_DB: creto_test
        ports: [5432:5432]
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Unit tests
        run: cargo test --workspace

      - name: Integration tests
        run: cargo test --workspace --features integration
        env:
          DATABASE_URL: postgres://postgres:test@localhost:5432/creto_test
          REDIS_URL: redis://localhost:6379

      - name: Coverage
        run: cargo llvm-cov --workspace --lcov --output-path lcov.info

      - uses: codecov/codecov-action@v3
```

### 7.3 Test Data Management

```rust
// tests/fixtures/mod.rs

pub fn test_agent_identity() -> AgentIdentity {
    AgentIdentity {
        nhi: "nhi://test.creto.io/agents/test-agent".parse().unwrap(),
        delegation_chain: vec![],
    }
}

pub fn test_billable_event() -> BillableEvent {
    BillableEvent {
        idempotency_key: uuid::Uuid::new_v4().to_string(),
        agent_nhi: test_agent_identity(),
        event_type: "test_event".to_string(),
        timestamp: Timestamp::now(),
        properties: HashMap::new(),
    }
}

pub fn test_oversight_policy() -> OversightPolicy {
    OversightPolicy {
        id: PolicyId::new(),
        name: "Test Policy".to_string(),
        trigger: Trigger::Always,
        approvers: vec![Approver::user("test@example.com")],
        timeout: Duration::minutes(5),
        default_action: DefaultAction::Deny,
    }
}
```

---

## 8. Quality Gates

### 8.1 PR Requirements

| Gate | Requirement | Blocking |
|------|-------------|----------|
| **Unit tests** | All pass | Yes |
| **Integration tests** | All pass | Yes |
| **Coverage** | ≥80% overall | Yes |
| **Linting** | No warnings | Yes |
| **Security audit** | No critical/high | Yes |
| **Format** | `cargo fmt --check` | Yes |

### 8.2 Release Requirements

| Gate | Requirement |
|------|-------------|
| All PR gates pass | Yes |
| E2E tests pass | Yes |
| Performance benchmarks within thresholds | Yes |
| Security scan (Trivy) clean | Yes |
| Documentation updated | Yes |

### 8.3 CI Badge Status

```markdown
[![Tests](https://github.com/creto-systems/enablement/actions/workflows/test.yml/badge.svg)](...)
[![Coverage](https://codecov.io/gh/creto-systems/enablement/branch/main/graph/badge.svg)](...)
[![Security](https://snyk.io/test/github/creto-systems/enablement/badge.svg)](...)
```

---

## 9. Test Documentation

### 9.1 Test Naming Convention

```rust
// Pattern: <subject>_<scenario>_<expected_outcome>

#[test]
fn quota_enforcer_when_limit_exceeded_returns_error() { ... }

#[test]
fn oversight_policy_with_timeout_auto_denies_after_expiry() { ... }

#[test]
fn message_encryption_with_ml_kem_produces_valid_ciphertext() { ... }
```

### 9.2 Test Documentation

```rust
/// Tests that quota enforcement correctly blocks operations when limit is exceeded.
///
/// # Scenario
/// 1. Create quota tracker with limit of 100
/// 2. Consume 80 units (should succeed)
/// 3. Attempt to consume 30 more units (should fail)
/// 4. Verify remaining quota is still 20
///
/// # Acceptance Criteria
/// - QuotaExceeded error returned when limit would be exceeded
/// - Partial consumption not applied on failure
#[test]
fn quota_enforcement_blocks_when_limit_exceeded() {
    // ...
}
```

---

## 10. Decisions

| Decision | Rationale |
|----------|-----------|
| 80% coverage minimum | Balance between coverage and maintenance burden |
| Property-based testing | Catch edge cases unit tests miss |
| Fuzz testing for parsers | Security-critical input handling |
| Testcontainers over mocks | Higher confidence in integration behavior |
| k6 for load testing | Developer-friendly, scriptable |

---

## 11. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
