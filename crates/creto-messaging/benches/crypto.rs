//! Cryptographic benchmarks for messaging.
//!
//! Verifies the >100K msg/s target for encryption and key operations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use creto_common::AgentId;
use creto_messaging::{
    DoubleRatchet, KeyBundle, Session,
    topic::{TopicConfig, TopicManager},
    x3dh::X3DH,
};
use std::time::Duration;

/// Benchmark: Key Bundle Generation
fn bench_key_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_generation");
    group.throughput(Throughput::Elements(100));
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("generate_key_bundle_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let agent_id = AgentId::new();
                let bundle = black_box(KeyBundle::new(agent_id));
                black_box(bundle);
            }
        });
    });

    group.finish();
}

/// Benchmark: X3DH Key Agreement
fn bench_x3dh(c: &mut Criterion) {
    let mut group = c.benchmark_group("x3dh");
    group.throughput(Throughput::Elements(100));
    group.measurement_time(Duration::from_secs(10));

    let alice_id = AgentId::new();
    let bob_id = AgentId::new();
    let alice_bundle = KeyBundle::new(alice_id);
    let bob_bundle = KeyBundle::new(bob_id);

    group.bench_function("x3dh_initiate_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let result = black_box(X3DH::initiate(&alice_bundle, &bob_bundle));
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark: Double Ratchet Encryption (>100K msg/s target)
fn bench_double_ratchet(c: &mut Criterion) {
    let mut group = c.benchmark_group("double_ratchet");
    group.throughput(Throughput::Elements(1000));
    group.measurement_time(Duration::from_secs(15));

    // Setup keys for the ratchet
    let shared_secret = [0u8; 32];
    let their_signed_prekey = [1u8; 32]; // Dummy prekey

    // Small message (128 bytes)
    let small_message = vec![0u8; 128];

    group.bench_function("encrypt_small_1000", |b| {
        b.iter(|| {
            let mut ratchet = DoubleRatchet::new_sender(&shared_secret, &their_signed_prekey);
            for _ in 0..1000 {
                let result = black_box(ratchet.encrypt(&small_message));
                black_box(result);
            }
        });
    });

    // Medium message (1KB)
    let medium_message = vec![0u8; 1024];

    group.bench_function("encrypt_medium_1000", |b| {
        b.iter(|| {
            let mut ratchet = DoubleRatchet::new_sender(&shared_secret, &their_signed_prekey);
            for _ in 0..1000 {
                let result = black_box(ratchet.encrypt(&medium_message));
                black_box(result);
            }
        });
    });

    // Large message (64KB)
    let large_message = vec![0u8; 64 * 1024];

    group.bench_function("encrypt_large_100", |b| {
        b.iter(|| {
            let mut ratchet = DoubleRatchet::new_sender(&shared_secret, &their_signed_prekey);
            for _ in 0..100 {
                let result = black_box(ratchet.encrypt(&large_message));
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark: Session Operations
fn bench_session(c: &mut Criterion) {
    let mut group = c.benchmark_group("session");
    group.throughput(Throughput::Elements(100));
    group.measurement_time(Duration::from_secs(10));

    let alice_id = AgentId::new();
    let bob_id = AgentId::new();
    let alice_bundle = KeyBundle::new(alice_id);
    let bob_bundle = KeyBundle::new(bob_id);

    group.bench_function("establish_session_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let x3dh_result = X3DH::initiate(&alice_bundle, &bob_bundle).unwrap();
                let session = black_box(Session::new_initiator(alice_id, bob_id, &x3dh_result));
                black_box(session);
            }
        });
    });

    // Create session for encrypt/decrypt benchmark
    let x3dh_result = X3DH::initiate(&alice_bundle, &bob_bundle).unwrap();
    let message = b"Hello, this is a test message for benchmarking!";

    group.bench_function("session_encrypt_1000", |b| {
        b.iter(|| {
            let mut session = Session::new_initiator(alice_id, bob_id, &x3dh_result);
            for _ in 0..1000 {
                let result = black_box(session.encrypt(message));
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark: Topic Operations
fn bench_topics(c: &mut Criterion) {
    let mut group = c.benchmark_group("topics");
    group.throughput(Throughput::Elements(100));
    group.measurement_time(Duration::from_secs(10));

    let owner_id = AgentId::new();

    group.bench_function("create_topic_100", |b| {
        b.iter(|| {
            let mut manager = TopicManager::new();
            for i in 0..100 {
                let config = TopicConfig::new(format!("topic-{}", i), owner_id);
                let result = black_box(manager.create_topic(config));
                black_box(result);
            }
        });
    });

    group.bench_function("subscribe_100", |b| {
        b.iter(|| {
            let mut manager = TopicManager::new();
            let config = TopicConfig::new("sub-topic".to_string(), owner_id);
            let topic_id = manager.create_topic(config).unwrap();

            for _ in 0..100 {
                let subscriber_id = AgentId::new();
                let result = black_box(manager.subscribe(topic_id, subscriber_id, None));
                black_box(result);
            }
        });
    });

    // Setup subscribers for publish benchmark
    let mut pub_manager = TopicManager::new();
    let pub_config = TopicConfig::new("pub-topic".to_string(), owner_id);
    let pub_topic_id = pub_manager.create_topic(pub_config).unwrap();

    for _ in 0..10 {
        let subscriber_id = AgentId::new();
        let _ = pub_manager.subscribe(pub_topic_id, subscriber_id, None);
    }

    let message = b"Benchmark message payload";
    let metadata = std::collections::HashMap::new();

    group.bench_function("publish_to_10_subscribers_100", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let result = black_box(pub_manager.publish(pub_topic_id, owner_id, message, metadata.clone()));
                black_box(result);
            }
        });
    });

    group.finish();
}

/// Benchmark: Encryption Throughput (MB/s)
fn bench_encryption_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption_throughput");
    group.throughput(Throughput::Bytes(1024 * 1024)); // 1MB
    group.measurement_time(Duration::from_secs(15));

    let shared_secret = [0u8; 32];
    let their_signed_prekey = [1u8; 32];
    let one_mb = vec![0u8; 1024 * 1024];

    group.bench_function("encrypt_1mb", |b| {
        b.iter(|| {
            let mut ratchet = DoubleRatchet::new_sender(&shared_secret, &their_signed_prekey);
            let result = black_box(ratchet.encrypt(&one_mb));
            black_box(result);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_key_generation,
    bench_x3dh,
    bench_double_ratchet,
    bench_session,
    bench_topics,
    bench_encryption_throughput
);
criterion_main!(benches);
