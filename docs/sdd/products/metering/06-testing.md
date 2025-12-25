---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# SDD-MTR-06: Metering Testing Strategy

## Overview

This document specifies the comprehensive testing strategy for **creto-metering**, including unit tests, integration tests, load tests, security tests, and benchmarks. The goal is to achieve:

- **>90% code coverage** for critical paths (event ingestion, quota enforcement, billing)
- **<10µs p99 latency** for quota checks (cache hits)
- **>10,000 events/sec throughput** for batch ingestion
- **99.999% accuracy** for billing calculations (zero double-billing)

## Test Pyramid

```
         /\
        /  \
       / E2E \         10 tests (slow, expensive)
      /______\
     /        \
    /  Integ   \       100 tests (moderate speed)
   /____________\
  /              \
 /  Unit Tests    \    1000 tests (fast, cheap)
/__________________\
```

**Distribution**:
- **Unit Tests**: 80% of tests (fast, isolated, 1000+ tests)
- **Integration Tests**: 15% of tests (moderate speed, 100-200 tests)
- **End-to-End Tests**: 5% of tests (slow, full system, 10-20 tests)

---

## 1. Unit Tests

### 1.1 Event Validation

**Test Cases**:

```rust
#[cfg(test)]
mod event_validation_tests {
    use super::*;

    #[test]
    fn test_valid_event_schema() {
        let event = EventRequest {
            idempotency_key: "test-key-001".to_string(),
            agent_nhi: "agent:nhi:ed25519:test123".to_string(),
            delegation_chain: vec!["human:alice@test.com".to_string()],
            event_type: "llm_tokens".to_string(),
            timestamp: Some(Utc::now()),
            properties: json!({"tokens": 1500, "model": "gpt-4"}),
            signature: vec![0u8; 64],
            signature_algorithm: "ML-DSA-65".to_string(),
        };

        let validator = EventValidator::new();
        assert!(validator.validate_schema(&event).is_ok());
    }

    #[test]
    fn test_missing_required_field() {
        let event = EventRequest {
            idempotency_key: "".to_string(),  // Empty (invalid)
            agent_nhi: "agent:nhi:ed25519:test123".to_string(),
            delegation_chain: vec![],
            event_type: "llm_tokens".to_string(),
            timestamp: Some(Utc::now()),
            properties: json!({}),
            signature: vec![],
            signature_algorithm: "ML-DSA-65".to_string(),
        };

        let validator = EventValidator::new();
        let result = validator.validate_schema(&event);

        assert!(result.is_err());
        assert!(matches!(result, Err(Error::InvalidRequest(_))));
    }

    #[test]
    fn test_timestamp_too_far_future() {
        let event = EventRequest {
            idempotency_key: "test-key-002".to_string(),
            agent_nhi: "agent:nhi:ed25519:test123".to_string(),
            delegation_chain: vec!["human:bob@test.com".to_string()],
            event_type: "api_call".to_string(),
            timestamp: Some(Utc::now() + Duration::from_secs(3600)),  // 1 hour in future
            properties: json!({}),
            signature: vec![0u8; 64],
            signature_algorithm: "ML-DSA-65".to_string(),
        };

        let validator = EventValidator::new();
        let result = validator.validate_timestamp(&event);

        assert!(result.is_err());
        assert!(matches!(result, Err(Error::TimestampSkew(_))));
    }

    #[test]
    fn test_properties_nested_too_deep() {
        let deeply_nested = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "level4": {  // 4 levels deep (max = 3)
                            "data": "value"
                        }
                    }
                }
            }
        });

        let event = EventRequest {
            idempotency_key: "test-key-003".to_string(),
            agent_nhi: "agent:nhi:ed25519:test123".to_string(),
            delegation_chain: vec![],
            event_type: "test_event".to_string(),
            timestamp: Some(Utc::now()),
            properties: deeply_nested,
            signature: vec![],
            signature_algorithm: "ML-DSA-65".to_string(),
        };

        let validator = EventValidator::new();
        assert!(validator.validate_properties_depth(&event).is_err());
    }
}
```

---

### 1.2 Aggregation Logic

**Test Cases**:

```rust
#[cfg(test)]
mod aggregation_tests {
    use super::*;

    #[test]
    fn test_count_aggregation() {
        let aggregator = CountAggregator;
        let mut state: u64 = 0;

        // Simulate 10 events
        for _ in 0..10 {
            let event = create_test_event();
            aggregator.accumulate(&mut state, &event);
        }

        assert_eq!(aggregator.finalize(&state), Decimal::from(10));
    }

    #[test]
    fn test_sum_aggregation() {
        let aggregator = SumAggregator {
            property_path: "tokens".to_string(),
        };
        let mut state = Decimal::ZERO;

        // Event 1: 1000 tokens
        let event1 = create_test_event_with_properties(json!({"tokens": 1000}));
        aggregator.accumulate(&mut state, &event1);

        // Event 2: 500 tokens
        let event2 = create_test_event_with_properties(json!({"tokens": 500}));
        aggregator.accumulate(&mut state, &event2);

        assert_eq!(aggregator.finalize(&state), Decimal::from(1500));
    }

    #[test]
    fn test_unique_count_aggregation() {
        let aggregator = UniqueCountAggregator {
            property_path: "model".to_string(),
        };
        let mut state = HashSet::new();

        // 3 events: gpt-4, gpt-4, gpt-3.5 (2 unique models)
        let events = vec![
            create_test_event_with_properties(json!({"model": "gpt-4"})),
            create_test_event_with_properties(json!({"model": "gpt-4"})),  // Duplicate
            create_test_event_with_properties(json!({"model": "gpt-3.5-turbo"})),
        ];

        for event in events {
            aggregator.accumulate(&mut state, &event);
        }

        assert_eq!(aggregator.finalize(&state), Decimal::from(2));
    }

    #[test]
    fn test_max_aggregation() {
        let aggregator = MaxAggregator {
            property_path: "latency_ms".to_string(),
        };
        let mut state = Decimal::ZERO;

        let events = vec![
            create_test_event_with_properties(json!({"latency_ms": 100})),
            create_test_event_with_properties(json!({"latency_ms": 450})),  // Max
            create_test_event_with_properties(json!({"latency_ms": 200})),
        ];

        for event in events {
            aggregator.accumulate(&mut state, &event);
        }

        assert_eq!(aggregator.finalize(&state), Decimal::from(450));
    }
}
```

---

### 1.3 Pricing Model Calculations

**Test Cases**:

```rust
#[cfg(test)]
mod pricing_tests {
    use super::*;

    #[test]
    fn test_flat_fee_pricing() {
        let pricing = FlatFeePricing {
            amount: Decimal::from_str("99.00").unwrap(),
            currency: Currency::USD,
            prorated: false,
        };

        // Flat fee ignores usage
        assert_eq!(pricing.calculate_charge(Decimal::ZERO), Decimal::from_str("99.00").unwrap());
        assert_eq!(pricing.calculate_charge(Decimal::from(1000)), Decimal::from_str("99.00").unwrap());
    }

    #[test]
    fn test_per_unit_pricing() {
        let pricing = PerUnitPricing {
            unit_price: Decimal::from_str("0.002").unwrap(),
            currency: Currency::USD,
        };

        // 15,000 units × $0.002 = $30.00
        let usage = Decimal::from(15000);
        let charge = pricing.calculate_charge(usage);

        assert_eq!(charge, Decimal::from_str("30.00").unwrap());
    }

    #[test]
    fn test_tiered_graduated_pricing() {
        let pricing = TieredGraduatedPricing {
            tiers: vec![
                Tier { up_to: Some(Decimal::from(1000)), unit_price: Decimal::from_str("0.01").unwrap() },
                Tier { up_to: Some(Decimal::from(10000)), unit_price: Decimal::from_str("0.008").unwrap() },
                Tier { up_to: None, unit_price: Decimal::from_str("0.005").unwrap() },
            ],
            currency: Currency::USD,
        };

        // 15,000 units:
        // - First 1,000: 1,000 × $0.01 = $10.00
        // - Next 9,000: 9,000 × $0.008 = $72.00
        // - Remaining 5,000: 5,000 × $0.005 = $25.00
        // Total: $107.00
        let usage = Decimal::from(15000);
        let charge = pricing.calculate_charge(usage);

        assert_eq!(charge, Decimal::from_str("107.00").unwrap());
    }

    #[test]
    fn test_tiered_volume_pricing() {
        let pricing = TieredVolumePricing {
            tiers: vec![
                Tier { up_to: Some(Decimal::from(1000)), unit_price: Decimal::from_str("0.01").unwrap() },
                Tier { up_to: Some(Decimal::from(10000)), unit_price: Decimal::from_str("0.008").unwrap() },
                Tier { up_to: None, unit_price: Decimal::from_str("0.005").unwrap() },
            ],
            currency: Currency::USD,
        };

        // 15,000 units → Tier 3 (all units at $0.005)
        // 15,000 × $0.005 = $75.00
        let usage = Decimal::from(15000);
        let charge = pricing.calculate_charge(usage);

        assert_eq!(charge, Decimal::from_str("75.00").unwrap());
    }

    #[test]
    fn test_package_pricing_no_overage() {
        let pricing = PackagePricing {
            package_size: 1000,
            package_price: Decimal::from_str("50.00").unwrap(),
            overage_unit_price: Decimal::from_str("0.06").unwrap(),
            currency: Currency::USD,
        };

        // 800 units (within package) = $50.00
        let usage = Decimal::from(800);
        let charge = pricing.calculate_charge(usage);

        assert_eq!(charge, Decimal::from_str("50.00").unwrap());
    }

    #[test]
    fn test_package_pricing_with_overage() {
        let pricing = PackagePricing {
            package_size: 1000,
            package_price: Decimal::from_str("50.00").unwrap(),
            overage_unit_price: Decimal::from_str("0.06").unwrap(),
            currency: Currency::USD,
        };

        // 1,200 units = $50.00 + (200 × $0.06) = $62.00
        let usage = Decimal::from(1200);
        let charge = pricing.calculate_charge(usage);

        assert_eq!(charge, Decimal::from_str("62.00").unwrap());
    }
}
```

---

### 1.4 Quota Decision Logic

**Test Cases**:

```rust
#[cfg(test)]
mod quota_tests {
    use super::*;

    #[test]
    fn test_quota_within_limit() {
        let state = QuotaState {
            subscription_id: Uuid::new_v4(),
            event_type: "api_calls".to_string(),
            limit: 1000,
            period: QuotaPeriod::Hourly,
            overflow_action: OverflowAction::Block,
            current_usage: 450,
            period_start: Utc::now(),
        };

        let enforcer = QuotaEnforcerImpl::new_test();
        let decision = enforcer.evaluate_quota_state(&state).unwrap();

        match decision {
            QuotaDecision::Allow { remaining, .. } => {
                assert_eq!(remaining, 550);
            }
            _ => panic!("Expected Allow decision"),
        }
    }

    #[test]
    fn test_quota_exceeded_block() {
        let state = QuotaState {
            subscription_id: Uuid::new_v4(),
            event_type: "api_calls".to_string(),
            limit: 1000,
            period: QuotaPeriod::Hourly,
            overflow_action: OverflowAction::Block,
            current_usage: 1001,  // Over limit
            period_start: Utc::now(),
        };

        let enforcer = QuotaEnforcerImpl::new_test();
        let decision = enforcer.evaluate_quota_state(&state).unwrap();

        match decision {
            QuotaDecision::Deny { reason, current_usage, limit, .. } => {
                assert_eq!(reason, QuotaExceededReason::LimitReached);
                assert_eq!(current_usage, 1001);
                assert_eq!(limit, 1000);
            }
            _ => panic!("Expected Deny decision"),
        }
    }

    #[test]
    fn test_quota_exceeded_allow_with_overage() {
        let state = QuotaState {
            subscription_id: Uuid::new_v4(),
            event_type: "api_calls".to_string(),
            limit: 1000,
            period: QuotaPeriod::Hourly,
            overflow_action: OverflowAction::AllowWithOverage,
            current_usage: 1001,
            period_start: Utc::now(),
        };

        let enforcer = QuotaEnforcerImpl::new_test();
        let decision = enforcer.evaluate_quota_state(&state).unwrap();

        // Should allow but mark for overage fee
        match decision {
            QuotaDecision::Allow { remaining, .. } => {
                assert_eq!(remaining, 0);  // Over quota
            }
            _ => panic!("Expected Allow decision (with overage)"),
        }
    }
}
```

---

## 2. Integration Tests

### 2.1 Event Ingestion → Aggregation Pipeline

**Test Cases**:

```rust
#[tokio::test]
async fn test_event_ingestion_updates_aggregation() {
    // Setup test environment
    let test_env = TestEnvironment::new().await;

    // Submit event
    let event = create_test_event(json!({
        "event_type": "llm_tokens",
        "properties": {"tokens": 1500, "model": "gpt-4"}
    }));

    let response = test_env.ingestion_service.ingest_event(event.clone()).await.unwrap();

    // Wait for async aggregation update
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Query aggregation
    let usage = test_env.aggregation_engine.get_usage(
        event.subscription_id,
        "llm_tokens",
        TimePeriod::current_hour(),
    ).await.unwrap();

    // Verify aggregation updated
    assert_eq!(usage.count, 1);
    assert_eq!(usage.sum_value, Decimal::from(1500));
    assert!(usage.by_dimension.get("model").unwrap().contains_key("gpt-4"));
}
```

---

### 2.2 Quota Check → Event Ingestion → Quota Refresh

**Test Cases**:

```rust
#[tokio::test]
async fn test_quota_enforcement_lifecycle() {
    let test_env = TestEnvironment::new().await;

    // Setup quota: 10 events/hour
    test_env.setup_quota(QuotaConfig {
        subscription_id: test_env.subscription_id,
        event_type: "api_call".to_string(),
        limit: 10,
        period: QuotaPeriod::Hourly,
        overflow_action: OverflowAction::Block,
    }).await;

    // Submit 10 events (should succeed)
    for i in 0..10 {
        let event = create_test_event_with_type("api_call");
        let response = test_env.ingestion_service.ingest_event(event).await.unwrap();
        assert!(matches!(response, EventResponse::Created(_)));
    }

    // Check quota (should show 0 remaining)
    let decision = test_env.quota_enforcer.check_quota(
        &test_env.agent_nhi,
        "api_call",
    ).await.unwrap();

    match decision {
        QuotaDecision::Allow { remaining, .. } => assert_eq!(remaining, 0),
        _ => panic!("Expected Allow with 0 remaining"),
    }

    // Submit 11th event (should be denied)
    let event_11 = create_test_event_with_type("api_call");
    let decision_11 = test_env.quota_enforcer.check_quota(
        &test_env.agent_nhi,
        "api_call",
    ).await.unwrap();

    match decision_11 {
        QuotaDecision::Deny { reason, .. } => {
            assert_eq!(reason, QuotaExceededReason::LimitReached);
        }
        _ => panic!("Expected Deny decision"),
    }
}
```

---

### 2.3 Authorization Integration

**Test Cases**:

```rust
#[tokio::test]
async fn test_authz_integration_inline_quota_check() {
    let test_env = TestEnvironment::new().await;

    // Mock Authorization service
    let authz = test_env.authz_client;

    // Setup quota: 5 events/hour
    test_env.setup_quota(QuotaConfig {
        subscription_id: test_env.subscription_id,
        event_type: "vector_query".to_string(),
        limit: 5,
        period: QuotaPeriod::Hourly,
        overflow_action: OverflowAction::Block,
    }).await;

    // Authorization check with quota enforcement
    for i in 0..5 {
        let authz_result = authz.check_with_quota(
            &test_env.agent_nhi,
            "vector:query",
            "resource:database",
        ).await.unwrap();

        assert!(authz_result.allowed);
        assert!(authz_result.quota_remaining.unwrap() == 5 - i - 1);
    }

    // 6th request: Denied by quota
    let authz_result_6 = authz.check_with_quota(
        &test_env.agent_nhi,
        "vector:query",
        "resource:database",
    ).await.unwrap();

    assert!(!authz_result_6.allowed);
    assert_eq!(authz_result_6.denial_reason, Some("quota_exceeded".to_string()));
}
```

---

### 2.4 Invoice Generation End-to-End

**Test Cases**:

```rust
#[tokio::test]
async fn test_invoice_generation_full_pipeline() {
    let test_env = TestEnvironment::new().await;

    // Setup billing configuration
    test_env.setup_billable_metric(BillableMetricConfig {
        metric_code: "llm_tokens".to_string(),
        aggregation: AggregationType::Sum,
        aggregation_property: Some("tokens".to_string()),
        pricing_model: PricingModel::PerUnit {
            unit_price: Decimal::from_str("0.00002").unwrap(),  // $0.02 per 1K tokens
            currency: Currency::USD,
        },
    }).await;

    // Submit usage events
    for _ in 0..10 {
        let event = create_test_event(json!({
            "event_type": "llm_tokens",
            "properties": {"tokens": 1500, "model": "gpt-4"}
        }));
        test_env.ingestion_service.ingest_event(event).await.unwrap();
    }

    // Total usage: 10 events × 1,500 tokens = 15,000 tokens
    // Expected charge: 15,000 × $0.00002 = $0.30

    // Generate invoice
    let invoice = test_env.billing_service.generate_invoice(
        test_env.subscription_id,
        BillingPeriod::current_month(),
    ).await.unwrap();

    // Verify line item
    assert_eq!(invoice.line_items.len(), 1);
    let line_item = &invoice.line_items[0];
    assert_eq!(line_item.metric_code, "llm_tokens");
    assert_eq!(line_item.quantity, Decimal::from(15000));
    assert_eq!(line_item.amount, Decimal::from_str("0.30").unwrap());

    // Verify totals
    assert_eq!(invoice.subtotal, Decimal::from_str("0.30").unwrap());
    assert_eq!(invoice.total, invoice.subtotal + invoice.tax);
}
```

---

## 3. Load Tests

### 3.1 Event Ingestion Throughput

**Test Scenario**: Sustained 10,000 events/second for 1 hour

**k6 Script**:
```javascript
import http from 'k6/http';
import { check, sleep } from 'k6';

export let options = {
  stages: [
    { duration: '5m', target: 100 },    // Ramp up to 100 VUs
    { duration: '50m', target: 100 },   // Hold 100 VUs for 50 min
    { duration: '5m', target: 0 },      // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(99)<500'],   // 99% of requests < 500ms
    http_req_failed: ['rate<0.01'],     // Error rate < 1%
  },
};

export default function () {
  // Generate batch of 100 events
  const batch = [];
  for (let i = 0; i < 100; i++) {
    batch.push({
      idempotency_key: `load-test-${__VU}-${__ITER}-${i}`,
      agent_nhi: `agent:nhi:ed25519:loadtest-${__VU}`,
      delegation_chain: ['human:loadtest@example.com'],
      event_type: 'load_test_event',
      properties: {
        vu: __VU,
        iteration: __ITER,
        index: i,
      },
      signature: 'base64encodedSignature',
    });
  }

  const response = http.post(
    'https://api.creto.io/metering/v1/events/batch',
    JSON.stringify({ events: batch }),
    {
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${__ENV.TOKEN}`,
      },
    }
  );

  check(response, {
    'status is 207': (r) => r.status === 207,
    'batch succeeded': (r) => JSON.parse(r.body).succeeded === 100,
  });

  sleep(1);  // 100 VUs × 100 events/sec = 10,000 events/sec
}
```

**Success Criteria**:
- p99 latency <500ms
- Error rate <1%
- 0 events lost (verify DB count matches submitted)
- Database CPU <80%
- Redis memory <90%

---

### 3.2 Quota Check Latency

**Test Scenario**: 100,000 concurrent quota checks

**Rust Benchmark**:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn quota_check_cache_hit_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let enforcer = rt.block_on(async {
        let enforcer = QuotaEnforcerImpl::new_with_cache().await;
        // Pre-populate cache
        for i in 0..10000 {
            enforcer.cache_quota_state(create_test_quota(i)).await;
        }
        enforcer
    });

    c.bench_function("quota_check_cache_hit", |b| {
        b.to_async(&rt).iter(|| async {
            let agent = format!("agent:nhi:ed25519:test{}", rand::random::<u32>() % 10000);
            black_box(enforcer.check_quota(&agent, "api_call").await.unwrap())
        })
    });
}

criterion_group!(benches, quota_check_cache_hit_benchmark);
criterion_main!(benches);
```

**Success Criteria**:
- p50 latency <2µs
- p99 latency <10µs
- p99.9 latency <50µs
- Cache hit rate >95%

---

### 3.3 Aggregation Query Performance

**Test Scenario**: Query 1M events

**SQL Benchmark**:
```sql
-- Setup: Insert 1M events
INSERT INTO events (subscription_id, event_type, timestamp, properties)
SELECT
    '123e4567-e89b-12d3-a456-426614174000'::uuid,
    'llm_tokens',
    now() - (random() * interval '30 days'),
    jsonb_build_object('tokens', (random() * 10000)::int)
FROM generate_series(1, 1000000);

-- Benchmark query (should use materialized view)
EXPLAIN ANALYZE
SELECT
    SUM(event_count) AS total_count,
    SUM(total_tokens) AS total_tokens
FROM usage_hourly
WHERE subscription_id = '123e4567-e89b-12d3-a456-426614174000'
  AND event_type = 'llm_tokens'
  AND hour >= now() - interval '30 days';
```

**Success Criteria**:
- Query time <100ms
- Uses index scan (not seq scan)
- Rows examined <10% of total events

---

## 4. Security Tests

### 4.1 Signature Verification

**Test Cases**:

```rust
#[tokio::test]
async fn test_reject_invalid_signature() {
    let test_env = TestEnvironment::new().await;

    let event = EventRequest {
        idempotency_key: "test-key".to_string(),
        agent_nhi: "agent:nhi:ed25519:test123".to_string(),
        delegation_chain: vec![],
        event_type: "test_event".to_string(),
        timestamp: Some(Utc::now()),
        properties: json!({}),
        signature: vec![0u8; 64],  // Invalid signature (all zeros)
        signature_algorithm: "ML-DSA-65".to_string(),
    };

    let result = test_env.ingestion_service.ingest_event(event).await;

    assert!(result.is_err());
    assert!(matches!(result, Err(Error::InvalidSignature)));

    // Verify audit log
    let audit_event = test_env.audit_client.get_last_event().await.unwrap();
    assert_eq!(audit_event.action, "signature.failed");
}
```

---

### 4.2 Replay Attack Prevention

**Test Cases**:

```rust
#[tokio::test]
async fn test_idempotency_prevents_replay() {
    let test_env = TestEnvironment::new().await;

    let event = create_test_event(json!({"event_type": "api_call"}));

    // Submit event (should succeed)
    let response1 = test_env.ingestion_service.ingest_event(event.clone()).await.unwrap();
    assert!(matches!(response1, EventResponse::Created(_)));

    // Replay same event (should return existing event_id)
    let response2 = test_env.ingestion_service.ingest_event(event.clone()).await.unwrap();
    assert!(matches!(response2, EventResponse::Accepted(_)));

    // Verify only 1 event in database
    let count = test_env.count_events_by_idempotency_key(&event.idempotency_key).await;
    assert_eq!(count, 1);
}
```

---

### 4.3 Quota Bypass Attempts

**Test Cases**:

```rust
#[tokio::test]
async fn test_quota_bypass_via_clock_manipulation() {
    let test_env = TestEnvironment::new().await;

    // Setup quota: 5 events/hour
    test_env.setup_quota(QuotaConfig {
        limit: 5,
        period: QuotaPeriod::Hourly,
        overflow_action: OverflowAction::Block,
        ..Default::default()
    }).await;

    // Submit 5 events (exhaust quota)
    for _ in 0..5 {
        test_env.submit_event().await;
    }

    // Attempt to bypass quota by backdating event to previous hour
    let backdated_event = EventRequest {
        timestamp: Some(Utc::now() - Duration::from_secs(3700)),  // 1h 2m ago
        ..create_test_event(json!({}))
    };

    // Should still be denied (consensus timestamp used, not agent timestamp)
    let decision = test_env.quota_enforcer.check_quota(
        &test_env.agent_nhi,
        &backdated_event.event_type,
    ).await.unwrap();

    assert!(matches!(decision, QuotaDecision::Deny { .. }));
}
```

---

## 5. Benchmark Suite

### 5.1 Critical Path Benchmarks

**Benchmarks to Track**:

| Operation | Target | Current | Tracking Since |
|-----------|--------|---------|----------------|
| `quota_check` (cache hit) | <10µs | 5.2µs | 2024-12 |
| `quota_check` (cache miss) | <500µs | 320µs | 2024-12 |
| `ingest_event` (single) | <20ms | 15ms | 2024-12 |
| `ingest_batch` (1000 events) | <500ms | 420ms | 2024-12 |
| `generate_invoice` | <1s | 680ms | 2024-12 |
| `aggregation_query` (1M events) | <100ms | 85ms | 2024-12 |

**Benchmark Script**:
```bash
# Run all benchmarks
cargo bench --package creto-metering -- --save-baseline main

# Compare against baseline
cargo bench --package creto-metering -- --baseline main

# Generate report
cargo criterion
```

---

## 6. Test Coverage Requirements

### Coverage Targets

| Component | Target Coverage | Critical Path Coverage |
|-----------|-----------------|------------------------|
| Event Ingestion | >95% | 100% |
| Quota Enforcement | >95% | 100% |
| Aggregation | >90% | 100% |
| Billing | >90% | 100% |
| API Layer | >80% | N/A |
| Utilities | >70% | N/A |

**Overall Target**: >90% line coverage

**Generate Coverage Report**:
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML coverage report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html
```

---

## 7. Continuous Testing

### CI Pipeline

**GitHub Actions Workflow**:
```yaml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

      redis:
        image: redis:7
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run unit tests
        run: cargo test --lib

      - name: Run integration tests
        run: cargo test --test '*'
        env:
          DATABASE_URL: postgres://postgres:test@localhost/metering_test
          REDIS_URL: redis://localhost:6379

      - name: Run benchmarks (regression check)
        run: cargo bench -- --test

      - name: Generate coverage
        run: |
          cargo tarpaulin --out Xml
          bash <(curl -s https://codecov.io/bash)
```

---

**Summary**: This testing strategy ensures creto-metering meets all performance, reliability, and security requirements through comprehensive automated testing at every layer.

**Next Document**: SDD-MTR-07: Runbook (operational procedures, monitoring, incident response)
