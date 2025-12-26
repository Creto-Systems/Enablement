//! Sandbox and warm pool benchmarks.
//!
//! Verifies the <100ms warm pool claim target and resource limit checks.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use creto_common::{AgentId, OrganizationId};
use creto_runtime::{
    PoolConfig, ResourceLimits, ResourceUsage, Sandbox, SandboxConfig, SandboxState, WarmPool,
};
use std::time::Duration;

/// Benchmark: Sandbox Creation
fn bench_sandbox_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("sandbox_creation");
    group.throughput(Throughput::Elements(100));
    group.measurement_time(Duration::from_secs(10));

    let org_id = OrganizationId::new();
    let agent_id = AgentId::new();
    let config = SandboxConfig::default();

    group.bench_function("create_sandbox_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let sandbox = black_box(Sandbox::new(org_id, agent_id, config.clone()));
                black_box(sandbox);
            }
        });
    });

    // With custom config
    let custom_config = SandboxConfig {
        runtime: "python3.11".to_string(),
        limits: ResourceLimits::generous(),
        ..SandboxConfig::default()
    };

    group.bench_function("create_sandbox_custom_config_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let sandbox = black_box(Sandbox::new(org_id, agent_id, custom_config.clone()));
                black_box(sandbox);
            }
        });
    });

    group.finish();
}

/// Benchmark: Warm Pool Operations (<100ms target)
fn bench_warm_pool(c: &mut Criterion) {
    let mut group = c.benchmark_group("warm_pool");
    group.throughput(Throughput::Elements(100));
    group.measurement_time(Duration::from_secs(10));

    let org_id = OrganizationId::new();
    let agent_id = AgentId::new();

    group.bench_function("pool_add_100", |b| {
        b.iter(|| {
            let pool = WarmPool::new(PoolConfig::default());
            for _ in 0..100 {
                let sandbox = Sandbox::new(org_id, agent_id, SandboxConfig::default());
                let result = black_box(pool.add(sandbox));
                black_box(result);
            }
        });
    });

    group.bench_function("pool_acquire_100", |b| {
        let pool = WarmPool::new(PoolConfig::default());
        // Pre-fill pool
        for _ in 0..200 {
            let sandbox = Sandbox::new(org_id, agent_id, SandboxConfig::default());
            let _ = pool.add(sandbox);
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                for _ in 0..100 {
                    let result = black_box(pool.acquire("default").await);
                    black_box(result);
                }
            });
        });
    });

    group.bench_function("pool_add_and_acquire_50", |b| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            rt.block_on(async {
                let pool = WarmPool::new(PoolConfig::default());

                // Add 50
                for _ in 0..50 {
                    let sandbox = Sandbox::new(org_id, agent_id, SandboxConfig::default());
                    let _ = pool.add(sandbox);
                }

                // Acquire 50
                for _ in 0..50 {
                    let result = black_box(pool.acquire("default").await);
                    black_box(result);
                }
            });
        });
    });

    group.finish();
}

/// Benchmark: Resource Limit Checks
fn bench_resource_checks(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_checks");
    group.throughput(Throughput::Elements(1000));
    group.measurement_time(Duration::from_secs(10));

    let limits = ResourceLimits::default();

    // ResourceUsage with all fields
    let usage_within = ResourceUsage {
        memory_bytes: 256 * 1024 * 1024, // 256MB
        peak_memory_bytes: 300 * 1024 * 1024, // 300MB peak
        cpu_time_ms: 30_000,              // 30s
        wall_time_ms: 100_000,            // 100s
        disk_bytes: 500 * 1024 * 1024,   // 500MB
        process_count: 16,
        open_file_count: 128,
        network_bytes_sent: 0,
        network_bytes_received: 0,
        connection_count: 10,
    };

    let usage_exceeds = ResourceUsage {
        memory_bytes: 1024 * 1024 * 1024, // 1GB - exceeds default 512MB
        peak_memory_bytes: 1024 * 1024 * 1024,
        cpu_time_ms: 30_000,
        wall_time_ms: 100_000,
        disk_bytes: 500 * 1024 * 1024,
        process_count: 16,
        open_file_count: 128,
        network_bytes_sent: 0,
        network_bytes_received: 0,
        connection_count: 10,
    };

    group.bench_function("check_within_limits_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = black_box(usage_within.exceeds(&limits));
                black_box(result);
            }
        });
    });

    group.bench_function("check_exceeds_limits_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let result = black_box(usage_exceeds.exceeds(&limits));
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark: ResourceLimits Creation
fn bench_resource_limits(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_limits");
    group.throughput(Throughput::Elements(1000));
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("create_default_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let limits = black_box(ResourceLimits::default());
                black_box(limits);
            }
        });
    });

    group.bench_function("create_minimal_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let limits = black_box(ResourceLimits::minimal());
                black_box(limits);
            }
        });
    });

    group.bench_function("create_generous_1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let limits = black_box(ResourceLimits::generous());
                black_box(limits);
            }
        });
    });

    group.finish();
}

/// Benchmark: Sandbox State Transitions
fn bench_state_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("sandbox_state");
    group.throughput(Throughput::Elements(100));
    group.measurement_time(Duration::from_secs(10));

    let org_id = OrganizationId::new();
    let agent_id = AgentId::new();

    group.bench_function("state_transitions_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let mut sandbox = Sandbox::new(org_id, agent_id, SandboxConfig::default());

                // Created -> Running
                sandbox.state = SandboxState::Running;
                black_box(sandbox.state);

                // Running -> Paused
                sandbox.state = SandboxState::Paused;
                black_box(sandbox.state);

                // Paused -> Running
                sandbox.state = SandboxState::Running;
                black_box(sandbox.state);

                // Running -> Terminated
                sandbox.state = SandboxState::Terminated;
                black_box(sandbox.state);
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_sandbox_creation,
    bench_warm_pool,
    bench_resource_checks,
    bench_resource_limits,
    bench_state_transitions
);
criterion_main!(benches);
