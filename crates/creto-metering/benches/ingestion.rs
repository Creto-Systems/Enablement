use creto_metering::{
    DedupConfig, Deduplicator, EventValidator, UsageEvent, UsageEventType, ValidationConfig,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;

/// Helper function to create a sample usage event
fn create_sample_event(index: u64) -> UsageEvent {
    UsageEvent {
        transaction_id: format!("txn_{}", index),
        timestamp: chrono::Utc::now(),
        organization_id: creto_common::OrganizationId::new(),
        agent_id: creto_common::AgentId::new(),
        external_subscription_id: Some(format!("sub_{}", index % 100)),
        event_type: UsageEventType::ApiCall,
        code: "api_calls".to_string(),
        quantity: 1,
        properties: serde_json::json!({
            "endpoint": "/api/v1/data",
            "method": "GET",
            "status_code": 200
        }),
        delegation_depth: 0,
    }
}

/// Benchmark: Event Creation Throughput
/// Tests raw event instantiation performance
fn bench_event_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_creation");
    group.throughput(Throughput::Elements(1000));
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("create_1000_events", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let event = black_box(create_sample_event(i));
                black_box(event);
            }
        });
    });

    group.finish();
}

/// Benchmark: Event Validation Throughput
/// Tests validation logic performance with various configurations
fn bench_event_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("event_validation");
    group.throughput(Throughput::Elements(1000));
    group.measurement_time(Duration::from_secs(10));

    // Strict validation config
    let strict_config = ValidationConfig {
        max_transaction_id_length: 255,
        max_quantity: 1_000_000,
        max_future_hours: 1,
        max_past_days: 7,
        max_properties_bytes: 65536,
        max_delegation_depth: 3,
        max_external_subscription_id_length: 255,
        collect_all_errors: false,
    };

    // Lenient validation config
    let lenient_config = ValidationConfig {
        max_transaction_id_length: 512,
        max_quantity: 10_000_000,
        max_future_hours: 24,
        max_past_days: 30,
        max_properties_bytes: 262144,
        max_delegation_depth: 10,
        max_external_subscription_id_length: 512,
        collect_all_errors: true,
    };

    group.bench_function("validate_strict_1000_events", |b| {
        let validator = EventValidator::new(strict_config.clone());
        let events: Vec<UsageEvent> = (0..1000).map(create_sample_event).collect();

        b.iter(|| {
            for event in &events {
                let result = black_box(validator.validate(event));
                black_box(result);
            }
        });
    });

    group.bench_function("validate_lenient_1000_events", |b| {
        let validator = EventValidator::new(lenient_config.clone());
        let events: Vec<UsageEvent> = (0..1000).map(create_sample_event).collect();

        b.iter(|| {
            for event in &events {
                let result = black_box(validator.validate(event));
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark: Deduplication Check Throughput (Local Cache Only)
/// Tests in-memory deduplication performance
fn bench_dedup_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("deduplication");
    group.throughput(Throughput::Elements(1000));
    group.measurement_time(Duration::from_secs(10));

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let config = DedupConfig {
        ttl_seconds: 300,
        key_prefix: "test:".to_string(),
        use_local_fallback: true,
        local_cache_max_size: 10000,
    };

    group.bench_function("dedup_check_1000_unique", |b| {
        let events: Vec<UsageEvent> = (0..1000).map(create_sample_event).collect();

        b.iter(|| {
            let deduplicator = Deduplicator::local_only(config.clone());
            runtime.block_on(async {
                for event in &events {
                    let result =
                        black_box(deduplicator.check_and_mark(&event.transaction_id).await);
                    black_box(result);
                }
            });
        });
    });

    group.bench_function("dedup_check_1000_duplicates", |b| {
        let event = create_sample_event(0);

        b.iter(|| {
            let deduplicator = Deduplicator::local_only(config.clone());
            runtime.block_on(async {
                // Pre-populate with the event
                let _ = deduplicator.check_and_mark(&event.transaction_id).await;

                for _ in 0..1000 {
                    let result =
                        black_box(deduplicator.check_and_mark(&event.transaction_id).await);
                    black_box(result);
                }
            });
        });
    });

    group.bench_function("dedup_check_1000_mixed", |b| {
        let events: Vec<UsageEvent> = (0..1000).map(create_sample_event).collect();

        b.iter(|| {
            let deduplicator = Deduplicator::local_only(config.clone());
            runtime.block_on(async {
                // Pre-populate 50% of events
                for (i, event) in events.iter().enumerate() {
                    if i % 2 == 0 {
                        let _ = deduplicator.check_and_mark(&event.transaction_id).await;
                    }
                }

                for event in &events {
                    let result =
                        black_box(deduplicator.check_and_mark(&event.transaction_id).await);
                    black_box(result);
                }
            });
        });
    });

    group.finish();
}

/// Benchmark: Batch Processing Simulation
/// Tests combined validation + deduplication throughput (target: >10K events/sec)
fn bench_batch_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_processing");
    group.measurement_time(Duration::from_secs(15));

    let runtime = tokio::runtime::Runtime::new().unwrap();

    // Test different batch sizes
    for batch_size in [100, 500, 1000, 5000, 10000].iter() {
        group.throughput(Throughput::Elements(*batch_size as u64));

        group.bench_function(format!("process_{}_events", batch_size), |b| {
            let validation_config = ValidationConfig {
                max_transaction_id_length: 255,
                max_quantity: 1_000_000,
                max_future_hours: 1,
                max_past_days: 7,
                max_properties_bytes: 65536,
                max_delegation_depth: 3,
                max_external_subscription_id_length: 255,
                collect_all_errors: false,
            };

            let dedup_config = DedupConfig {
                ttl_seconds: 300,
                key_prefix: "bench:".to_string(),
                use_local_fallback: true,
                local_cache_max_size: 10000,
            };

            let validator = EventValidator::new(validation_config);
            let events: Vec<UsageEvent> =
                (0..*batch_size as u64).map(create_sample_event).collect();

            b.iter(|| {
                let deduplicator = Deduplicator::local_only(dedup_config.clone());
                runtime.block_on(async {
                    let mut processed = 0;
                    let mut validated = 0;
                    let mut duplicates = 0;

                    for event in &events {
                        processed += 1;

                        // Validation
                        if validator.validate(event).is_ok() {
                            validated += 1;

                            // Deduplication
                            match deduplicator.check_and_mark(&event.transaction_id).await {
                                Ok(result) if result.is_duplicate() => {
                                    duplicates += 1;
                                }
                                _ => {}
                            }
                        }
                    }

                    black_box((processed, validated, duplicates));
                });
            });
        });
    }

    group.finish();
}

/// Benchmark: High-Throughput Stress Test
/// Tests sustained >10K events/sec processing
fn bench_high_throughput_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("high_throughput_stress");
    group.throughput(Throughput::Elements(10000));
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(50);

    let runtime = tokio::runtime::Runtime::new().unwrap();

    group.bench_function("sustained_10k_events_per_sec", |b| {
        let validation_config = ValidationConfig {
            max_transaction_id_length: 255,
            max_quantity: 1_000_000,
            max_future_hours: 1,
            max_past_days: 7,
            max_properties_bytes: 65536,
            max_delegation_depth: 3,
            max_external_subscription_id_length: 255,
            collect_all_errors: false,
        };

        let dedup_config = DedupConfig {
            ttl_seconds: 300,
            key_prefix: "stress:".to_string(),
            use_local_fallback: true,
            local_cache_max_size: 20000,
        };

        let validator = EventValidator::new(validation_config);

        // Pre-generate 10K events
        let events: Vec<UsageEvent> = (0..10000).map(create_sample_event).collect();

        b.iter(|| {
            let deduplicator = Deduplicator::local_only(dedup_config.clone());
            runtime.block_on(async {
                let mut successful = 0;

                for event in &events {
                    if validator.validate(event).is_ok() {
                        match deduplicator.check_and_mark(&event.transaction_id).await {
                            Ok(result) if result.is_new() => {
                                successful += 1;
                            }
                            _ => {}
                        }
                    }
                }

                black_box(successful);
            });
        });
    });

    group.bench_function("sustained_20k_events_per_sec", |b| {
        let validation_config = ValidationConfig {
            max_transaction_id_length: 512,
            max_quantity: 10_000_000,
            max_future_hours: 24,
            max_past_days: 30,
            max_properties_bytes: 262144,
            max_delegation_depth: 5,
            max_external_subscription_id_length: 512,
            collect_all_errors: true,
        };

        let dedup_config = DedupConfig {
            ttl_seconds: 300,
            key_prefix: "stress20k:".to_string(),
            use_local_fallback: true,
            local_cache_max_size: 30000,
        };

        let validator = EventValidator::new(validation_config);

        // Pre-generate 20K events
        let events: Vec<UsageEvent> = (0..20000).map(create_sample_event).collect();

        b.iter(|| {
            let deduplicator = Deduplicator::local_only(dedup_config.clone());
            runtime.block_on(async {
                let mut successful = 0;

                for event in &events {
                    if validator.validate(event).is_ok() {
                        match deduplicator.check_and_mark(&event.transaction_id).await {
                            Ok(result) if result.is_new() => {
                                successful += 1;
                            }
                            _ => {}
                        }
                    }
                }

                black_box(successful);
            });
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_event_creation,
    bench_event_validation,
    bench_dedup_check,
    bench_batch_processing,
    bench_high_throughput_stress
);
criterion_main!(benches);
