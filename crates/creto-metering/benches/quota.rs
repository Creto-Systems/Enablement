//! Quota enforcement benchmarks.
//!
//! Verifies the <10µs p99 latency target for quota checks.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use creto_metering::{BloomConfig, Quota, QuotaBloomFilter, QuotaEnforcer, QuotaKey, QuotaPeriod};
use std::time::Duration;

/// Benchmark: Bloom Filter Operations (<1µs target)
fn bench_bloom_filter(c: &mut Criterion) {
    let mut group = c.benchmark_group("bloom_filter");
    group.throughput(Throughput::Elements(1000));
    group.measurement_time(Duration::from_secs(10));

    let config = BloomConfig::default();
    let filter = QuotaBloomFilter::new(config);

    // Pre-populate with 1000 keys
    for i in 0..1000u64 {
        let key = QuotaKey::new(
            &format!("org_{}", i),
            &format!("agent_{}", i),
            "api_calls",
            "daily",
        );
        filter.insert(key.as_str());
    }

    group.bench_function("bloom_check_1000_existing", |b| {
        let keys: Vec<QuotaKey> = (0..1000u64)
            .map(|i| {
                QuotaKey::new(
                    &format!("org_{}", i),
                    &format!("agent_{}", i),
                    "api_calls",
                    "daily",
                )
            })
            .collect();

        b.iter(|| {
            for key in &keys {
                let result = black_box(filter.might_contain(key.as_str()));
                black_box(result);
            }
        });
    });

    group.bench_function("bloom_check_1000_missing", |b| {
        let keys: Vec<QuotaKey> = (1000..2000u64)
            .map(|i| {
                QuotaKey::new(
                    &format!("org_{}", i),
                    &format!("agent_{}", i),
                    "api_calls",
                    "daily",
                )
            })
            .collect();

        b.iter(|| {
            for key in &keys {
                let result = black_box(filter.might_contain(key.as_str()));
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark: QuotaEnforcer Check (<10µs target)
fn bench_enforcer_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("enforcer_check");
    group.throughput(Throughput::Elements(1000));
    group.measurement_time(Duration::from_secs(10));

    let enforcer = QuotaEnforcer::with_defaults();

    // Register 100 quotas
    for i in 0..100u64 {
        let org_id = creto_common::OrganizationId::new();
        let mut quota = Quota::new(org_id, format!("metric_{}", i), 1_000_000, QuotaPeriod::Daily);
        quota.agent_id = Some(creto_common::AgentId::new());
        enforcer.register_quota(&quota);
    }

    let org_id = creto_common::OrganizationId::new();
    let agent_id = creto_common::AgentId::new();

    group.bench_function("check_no_quota_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result =
                    black_box(enforcer.check(&org_id, &agent_id, "nonexistent", 1));
                black_box(result);
            }
        });
    });

    // Register a quota for this org/agent
    let mut test_quota = Quota::new(org_id.clone(), "api_calls", 1_000_000, QuotaPeriod::Daily);
    test_quota.agent_id = Some(agent_id.clone());
    enforcer.register_quota(&test_quota);

    group.bench_function("check_registered_quota_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result =
                    black_box(enforcer.check(&org_id, &agent_id, "api_calls", 1));
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark: Quota Check Latency (p99 target)
fn bench_quota_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("quota_latency");
    group.sample_size(1000);
    group.measurement_time(Duration::from_secs(15));

    let enforcer = QuotaEnforcer::with_defaults();
    let org_id = creto_common::OrganizationId::new();
    let agent_id = creto_common::AgentId::new();

    // Register quota
    let mut quota = Quota::new(org_id.clone(), "tokens", 10_000_000, QuotaPeriod::Daily);
    quota.agent_id = Some(agent_id.clone());
    enforcer.register_quota(&quota);

    group.bench_function("single_check_latency", |b| {
        b.iter(|| black_box(enforcer.check(&org_id, &agent_id, "tokens", 100)))
    });

    group.finish();
}

/// Benchmark: Reservation Operations
fn bench_reservation(c: &mut Criterion) {
    let mut group = c.benchmark_group("reservation");
    group.throughput(Throughput::Elements(100));
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("reserve_and_release_100", |b| {
        let enforcer = QuotaEnforcer::with_defaults();
        let org_id = creto_common::OrganizationId::new();
        let agent_id = creto_common::AgentId::new();

        let mut quota = Quota::new(org_id.clone(), "compute", 1_000_000, QuotaPeriod::Daily);
        quota.agent_id = Some(agent_id.clone());
        enforcer.register_quota(&quota);

        b.iter(|| {
            for i in 0..100 {
                let res_id = enforcer
                    .reserve(&org_id, &agent_id, "compute", 10, 60)
                    .unwrap();
                black_box(res_id);

                if i % 2 == 0 {
                    let _ = black_box(enforcer.release_reservation(res_id));
                } else {
                    let _ = black_box(enforcer.commit_reservation(res_id, 8));
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_bloom_filter,
    bench_enforcer_check,
    bench_quota_latency,
    bench_reservation
);
criterion_main!(benches);
