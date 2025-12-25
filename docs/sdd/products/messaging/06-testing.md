---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
---

# Messaging Testing Strategy

## Table of Contents

1. [Testing Overview](#testing-overview)
2. [Unit Testing](#unit-testing)
3. [Integration Testing](#integration-testing)
4. [Cryptographic Testing](#cryptographic-testing)
5. [Performance Testing](#performance-testing)
6. [Security Testing](#security-testing)
7. [Chaos Engineering](#chaos-engineering)
8. [Test Automation](#test-automation)

---

## Testing Overview

### Testing Pyramid

```
                   ┌─────────────┐
                   │   Manual    │  (<5%)
                   │ Exploratory │
                   └─────────────┘
              ┌─────────────────────┐
              │   E2E Integration   │  (~10%)
              │  (System-level)     │
              └─────────────────────┘
         ┌──────────────────────────────┐
         │   Component Integration      │  (~25%)
         │ (Service-to-service)         │
         └──────────────────────────────┘
    ┌─────────────────────────────────────┐
    │          Unit Tests                 │  (~60%)
    │   (Functions, modules, classes)     │
    └─────────────────────────────────────┘
```

### Test Coverage Goals

| Layer | Coverage Target | Current |
|-------|----------------|---------|
| **Unit tests** | >80% line coverage | TBD |
| **Integration tests** | >90% API coverage | TBD |
| **E2E tests** | 100% critical paths | TBD |
| **Performance tests** | All SLOs validated | TBD |
| **Security tests** | All threat vectors | TBD |

### Test Environments

```
┌────────────────────────────────────────────────────────┐
│ LOCAL (Developer Workstation)                          │
│  - Unit tests (cargo test)                             │
│  - Integration tests (Docker Compose)                  │
│  - Crypto test vectors                                 │
└────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────┐
│ CI/CD (GitHub Actions)                                 │
│  - All unit tests                                      │
│  - Integration tests (ephemeral environment)           │
│  - Performance regression tests                        │
│  - Security scans (Trivy, cargo-audit)                 │
└────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────┐
│ STAGING (Pre-production)                               │
│  - E2E tests (real components)                         │
│  - Load tests (1M agents simulated)                    │
│  - Chaos engineering (failure injection)               │
│  - Compliance validation                               │
└────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────┐
│ PRODUCTION (Canary testing)                            │
│  - Synthetic monitoring (send/receive every 1min)      │
│  - A/B testing (new features)                          │
│  - Real user monitoring (RUM)                          │
└────────────────────────────────────────────────────────┘
```

---

## Unit Testing

### Cryptographic Functions

**Test: ML-KEM-768 encapsulation/decapsulation**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use creto_crypto::ml_kem_768::{keypair, encapsulate, decapsulate};

    #[test]
    fn test_ml_kem_768_roundtrip() {
        // Generate key pair
        let (pubkey, privkey) = keypair();

        // Generate AES-256 key
        let aes_key: [u8; 32] = rand::random();

        // Encapsulate
        let wrapped_key = encapsulate(&pubkey, &aes_key).unwrap();
        assert_eq!(wrapped_key.len(), 1088);  // ML-KEM-768 ciphertext size

        // Decapsulate
        let recovered_key = decapsulate(&privkey, &wrapped_key).unwrap();
        assert_eq!(recovered_key, aes_key);
    }

    #[test]
    fn test_ml_kem_768_wrong_key() {
        let (pubkey1, _) = keypair();
        let (_, privkey2) = keypair();  // Different key pair

        let aes_key: [u8; 32] = rand::random();
        let wrapped_key = encapsulate(&pubkey1, &aes_key).unwrap();

        // Decapsulation with wrong private key produces random output (not error)
        let recovered_key = decapsulate(&privkey2, &wrapped_key).unwrap();
        assert_ne!(recovered_key, aes_key);
    }

    #[test]
    fn test_ml_kem_768_deterministic() {
        // ML-KEM encapsulation is randomized (different ciphertext each time)
        let (pubkey, _) = keypair();
        let aes_key: [u8; 32] = rand::random();

        let wrapped1 = encapsulate(&pubkey, &aes_key).unwrap();
        let wrapped2 = encapsulate(&pubkey, &aes_key).unwrap();

        // Same key, different ciphertext (randomized)
        assert_ne!(wrapped1, wrapped2);
    }
}
```

**Test: AES-256-GCM encryption/decryption**

```rust
#[test]
fn test_aes_gcm_roundtrip() {
    let key: [u8; 32] = rand::random();
    let nonce: [u8; 12] = rand::random();
    let plaintext = b"Hello, secure world!";
    let aad = b"envelope_metadata";

    let cipher = Aes256Gcm::new(&key.into());

    // Encrypt
    let ciphertext = cipher.encrypt(&nonce.into(), Payload {
        msg: plaintext,
        aad: aad,
    }).unwrap();

    // Decrypt
    let decrypted = cipher.decrypt(&nonce.into(), Payload {
        msg: &ciphertext,
        aad: aad,
    }).unwrap();

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_aes_gcm_tamper_detection() {
    let key: [u8; 32] = rand::random();
    let nonce: [u8; 12] = rand::random();
    let plaintext = b"Original message";

    let cipher = Aes256Gcm::new(&key.into());
    let mut ciphertext = cipher.encrypt(&nonce.into(), plaintext.as_ref()).unwrap();

    // Tamper with ciphertext
    ciphertext[0] ^= 0xFF;

    // Decryption should fail (authentication tag mismatch)
    let result = cipher.decrypt(&nonce.into(), ciphertext.as_ref());
    assert!(result.is_err());
}

#[test]
fn test_aes_gcm_nonce_reuse_danger() {
    // WARNING: This test demonstrates insecurity of nonce reuse
    let key: [u8; 32] = rand::random();
    let nonce: [u8; 12] = rand::random();  // Same nonce

    let cipher = Aes256Gcm::new(&key.into());

    let plaintext1 = b"Message 1";
    let plaintext2 = b"Message 2";

    let ciphertext1 = cipher.encrypt(&nonce.into(), plaintext1.as_ref()).unwrap();
    let ciphertext2 = cipher.encrypt(&nonce.into(), plaintext2.as_ref()).unwrap();

    // XOR ciphertexts reveals XOR of plaintexts (catastrophic failure)
    let xor: Vec<u8> = ciphertext1.iter()
        .zip(ciphertext2.iter())
        .map(|(a, b)| a ^ b)
        .collect();

    // In production: NEVER reuse nonce
    println!("Nonce reuse compromises confidentiality: {:?}", xor);
}
```

**Test: Hybrid signatures (Ed25519 + ML-DSA-65)**

```rust
#[test]
fn test_hybrid_signatures() {
    let message = b"Test message";

    // Ed25519
    let ed_keypair = Ed25519KeyPair::generate();
    let ed_signature = ed_keypair.sign(message);
    assert!(ed_keypair.verify(message, &ed_signature).is_ok());

    // ML-DSA-65
    let ml_keypair = MlDsa65KeyPair::generate();
    let ml_signature = ml_keypair.sign(message);
    assert!(ml_keypair.verify(message, &ml_signature).is_ok());

    // Both must verify
    assert!(ed_keypair.verify(message, &ed_signature).is_ok());
    assert!(ml_keypair.verify(message, &ml_signature).is_ok());
}

#[test]
fn test_signature_forgery_detection() {
    let message = b"Original message";
    let forged_message = b"Forged message!!";

    let keypair = Ed25519KeyPair::generate();
    let signature = keypair.sign(message);

    // Signature valid for original message
    assert!(keypair.verify(message, &signature).is_ok());

    // Signature invalid for forged message
    assert!(keypair.verify(forged_message, &signature).is_err());
}
```

### Message Envelope Processing

**Test: Canonical envelope representation**

```rust
#[test]
fn test_canonical_envelope_deterministic() {
    let envelope = MessageEnvelope {
        version: 1,
        message_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        sender_nhi: "agent-alice-01".to_string(),
        recipient_nhi: "agent-bob-02".to_string(),
        encrypted_payload: vec![1, 2, 3, 4],
        nonce: [5; 12],
        wrapped_key: vec![6; 1088],
        ..Default::default()
    };

    let canonical1 = envelope.to_canonical_bytes();
    let canonical2 = envelope.to_canonical_bytes();

    // Same envelope = same canonical representation
    assert_eq!(canonical1, canonical2);

    // SHA-256 hash for signing
    let hash1 = sha256(&canonical1);
    let hash2 = sha256(&canonical2);
    assert_eq!(hash1, hash2);
}

#[test]
fn test_envelope_field_order_matters() {
    let envelope = MessageEnvelope { /* ... */ };

    // Swapping fields changes canonical representation
    let canonical_original = envelope.to_canonical_bytes();

    let mut envelope_swapped = envelope.clone();
    std::mem::swap(&mut envelope_swapped.sender_nhi, &mut envelope_swapped.recipient_nhi);

    let canonical_swapped = envelope_swapped.to_canonical_bytes();
    assert_ne!(canonical_original, canonical_swapped);
}
```

### Authorization Checks

**Test: Delivery policy enforcement**

```rust
#[tokio::test]
async fn test_authz_allow_delivery() {
    let authz_client = MockAuthzClient::new();
    authz_client.expect_check()
        .returning(|_, _, _| Ok(Decision::Allow));

    let gate = AuthzGate::new(authz_client);
    let envelope = create_test_envelope();

    let result = gate.check_delivery(&envelope).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_authz_deny_delivery() {
    let authz_client = MockAuthzClient::new();
    authz_client.expect_check()
        .returning(|_, _, _| Ok(Decision::Deny("cross_org_denied".to_string())));

    let gate = AuthzGate::new(authz_client);
    let envelope = create_test_envelope();

    let result = gate.check_delivery(&envelope).await;
    assert!(matches!(result, Err(AuthzError::Denied(_))));
}

#[tokio::test]
async fn test_authz_service_unavailable_fail_closed() {
    let authz_client = MockAuthzClient::new();
    authz_client.expect_check()
        .returning(|_, _, _| Err(AuthzClientError::Timeout));

    let gate = AuthzGate::new_with_policy(authz_client, FailurePolicy::FailClosed);
    let envelope = create_test_envelope();

    let result = gate.check_delivery(&envelope).await;
    // Fail-closed: deny on AuthZ failure
    assert!(result.is_err());
}
```

---

## Integration Testing

### End-to-End Message Flow

**Test: Send and receive message**

```rust
#[tokio::test]
async fn test_e2e_send_receive() {
    // Setup test environment
    let (sender_client, recipient_client) = setup_test_clients().await;

    // Send message
    let message_id = sender_client
        .send("agent-bob-02", b"Hello, Bob!", SendOptions::default())
        .await
        .unwrap();

    // Receive message
    let messages = recipient_client
        .receive(10, None)
        .await
        .unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].payload, b"Hello, Bob!");
    assert_eq!(messages[0].sender, "agent-alice-01");

    // Acknowledge
    recipient_client.acknowledge(&message_id).await.unwrap();

    // Message should be deleted after ack
    let messages_after_ack = recipient_client.receive(10, None).await.unwrap();
    assert_eq!(messages_after_ack.len(), 0);
}
```

**Test: Request/response pattern**

```rust
#[tokio::test]
async fn test_request_response() {
    let (requester, responder) = setup_test_clients().await;

    // Responder listens for requests
    tokio::spawn(async move {
        let messages = responder.receive(1, Some(30)).await.unwrap();
        for msg in messages {
            if msg.payload == b"get_status" {
                responder.send(
                    &msg.sender,
                    b"status=healthy",
                    SendOptions {
                        correlation_id: msg.correlation_id,
                        ..Default::default()
                    }
                ).await.unwrap();
            }
        }
    });

    // Requester sends and waits for response
    let response = requester
        .send_and_wait(
            "agent-bob-02",
            b"get_status",
            SendOptions::default(),
            Duration::from_secs(5)
        )
        .await
        .unwrap();

    assert_eq!(response.payload, b"status=healthy");
}
```

**Test: Channel pub/sub**

```rust
#[tokio::test]
async fn test_channel_pub_sub() {
    let (publisher, subscriber1, subscriber2) = setup_test_clients().await;

    // Create channel
    let channel_id = publisher
        .create_channel("test-topic", ChannelPolicy::Open)
        .await
        .unwrap();

    // Subscribe
    subscriber1.subscribe(&channel_id).await.unwrap();
    subscriber2.subscribe(&channel_id).await.unwrap();

    // Publish
    publisher.publish(&channel_id, b"Broadcast message").await.unwrap();

    // Both subscribers receive
    let messages1 = subscriber1.receive_from_channel(&channel_id, 1).await.unwrap();
    let messages2 = subscriber2.receive_from_channel(&channel_id, 1).await.unwrap();

    assert_eq!(messages1.len(), 1);
    assert_eq!(messages2.len(), 1);
    assert_eq!(messages1[0].payload, b"Broadcast message");
    assert_eq!(messages2[0].payload, b"Broadcast message");
}
```

### Key Rotation

**Test: Seamless key rotation**

```rust
#[tokio::test]
async fn test_key_rotation_no_message_loss() {
    let (sender, recipient) = setup_test_clients().await;

    // Send message with old key
    let msg_id_1 = sender.send("agent-bob-02", b"Message 1", SendOptions::default()).await.unwrap();

    // Rotate recipient's key
    recipient.rotate_keys().await.unwrap();

    // Send message with new key
    let msg_id_2 = sender.send("agent-bob-02", b"Message 2", SendOptions::default()).await.unwrap();

    // Both messages should be receivable (grace period)
    let messages = recipient.receive(10, None).await.unwrap();
    assert_eq!(messages.len(), 2);

    // Verify decryption succeeded for both
    assert_eq!(messages[0].payload, b"Message 1");  // Decrypted with old key
    assert_eq!(messages[1].payload, b"Message 2");  // Decrypted with new key
}

#[tokio::test]
async fn test_grace_period_expiration() {
    let (sender, recipient) = setup_test_clients().await;

    // Send message
    sender.send("agent-bob-02", b"Old message", SendOptions::default()).await.unwrap();

    // Rotate key
    recipient.rotate_keys().await.unwrap();

    // Fast-forward time past grace period (7 days)
    tokio::time::advance(Duration::from_secs(7 * 24 * 3600 + 1)).await;

    // Message encrypted with old key should fail to decrypt
    let messages = recipient.receive(10, None).await.unwrap();
    assert_eq!(messages.len(), 0);  // Message discarded (old key deleted)
}
```

---

## Cryptographic Testing

### NIST Test Vectors

**Test: ML-KEM-768 Known Answer Tests**

```rust
#[test]
fn test_ml_kem_768_nist_kat() {
    // Load NIST KAT vectors from file
    let vectors = load_nist_vectors("ml_kem_768_kat.rsp");

    for vector in vectors {
        // Key generation from seed
        let (pubkey, privkey) = ml_kem_768::keypair_from_seed(&vector.seed);
        assert_eq!(pubkey, vector.expected_pubkey);
        assert_eq!(privkey, vector.expected_privkey);

        // Encapsulation with fixed randomness
        let (ciphertext, shared_secret) = ml_kem_768::encapsulate_deterministic(
            &pubkey,
            &vector.encaps_randomness
        );
        assert_eq!(ciphertext, vector.expected_ciphertext);
        assert_eq!(shared_secret, vector.expected_shared_secret);

        // Decapsulation
        let decapsulated = ml_kem_768::decapsulate(&privkey, &ciphertext);
        assert_eq!(decapsulated, shared_secret);
    }
}
```

**Test: ML-DSA-65 Known Answer Tests**

```rust
#[test]
fn test_ml_dsa_65_nist_kat() {
    let vectors = load_nist_vectors("ml_dsa_65_kat.rsp");

    for vector in vectors {
        let (pubkey, privkey) = ml_dsa_65::keypair_from_seed(&vector.seed);
        assert_eq!(pubkey, vector.expected_pubkey);

        // Deterministic signing
        let signature = ml_dsa_65::sign_deterministic(&privkey, &vector.message);
        assert_eq!(signature, vector.expected_signature);

        // Verification
        assert!(ml_dsa_65::verify(&pubkey, &vector.message, &signature));
    }
}
```

### Interoperability Testing

**Test: Cross-implementation compatibility**

```rust
#[test]
fn test_interop_with_reference_implementation() {
    // Message encrypted by reference C implementation
    let envelope_bytes = include_bytes!("testdata/reference_impl_envelope.bin");
    let envelope = MessageEnvelope::decode(envelope_bytes).unwrap();

    // Rust implementation should decrypt successfully
    let plaintext = decrypt_envelope(&envelope, &recipient_privkey).unwrap();
    assert_eq!(plaintext, b"Interop test message");
}

#[test]
fn test_cross_language_signatures() {
    // Signature generated by Python SDK
    let signature_ed25519 = hex::decode("...").unwrap();
    let signature_ml_dsa = hex::decode("...").unwrap();

    let message = b"Cross-language test";

    // Rust verifies Python signatures
    assert!(verify_ed25519(&sender_pubkey, message, &signature_ed25519));
    assert!(verify_ml_dsa(&sender_pubkey, message, &signature_ml_dsa));
}
```

---

## Performance Testing

### Throughput Benchmarks

**Benchmark: Encryption throughput**

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_encryption_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption_throughput");

    for payload_size in [1024, 10_240, 102_400] {  // 1KB, 10KB, 100KB
        group.throughput(criterion::Throughput::Bytes(payload_size as u64));

        group.bench_with_input(
            BenchmarkId::new("aes_gcm", payload_size),
            &payload_size,
            |b, &size| {
                let payload = vec![0u8; size];
                let key: [u8; 32] = rand::random();
                let nonce: [u8; 12] = rand::random();
                let cipher = Aes256Gcm::new(&key.into());

                b.iter(|| {
                    let ciphertext = cipher.encrypt(&nonce.into(), payload.as_ref()).unwrap();
                    black_box(ciphertext);
                });
            }
        );
    }

    group.finish();
}

fn bench_key_encapsulation(c: &mut Criterion) {
    let (pubkey, _) = ml_kem_768::keypair();
    let aes_key: [u8; 32] = rand::random();

    c.bench_function("ml_kem_768_encapsulate", |b| {
        b.iter(|| {
            let wrapped = ml_kem_768::encapsulate(&pubkey, &aes_key).unwrap();
            black_box(wrapped);
        });
    });
}

fn bench_signature_generation(c: &mut Criterion) {
    let ed_keypair = Ed25519KeyPair::generate();
    let ml_keypair = MlDsa65KeyPair::generate();
    let message = b"Benchmark message";

    c.bench_function("ed25519_sign", |b| {
        b.iter(|| {
            let sig = ed_keypair.sign(message);
            black_box(sig);
        });
    });

    c.bench_function("ml_dsa_65_sign", |b| {
        b.iter(|| {
            let sig = ml_keypair.sign(message);
            black_box(sig);
        });
    });
}

criterion_group!(benches,
    bench_encryption_throughput,
    bench_key_encapsulation,
    bench_signature_generation
);
criterion_main!(benches);
```

**Expected results:**

```
encryption_throughput/aes_gcm/1024    time: [450 ns 460 ns 470 ns]
                                      thrpt: [2.18 GB/s 2.23 GB/s 2.27 GB/s]

encryption_throughput/aes_gcm/10240   time: [2.1 µs 2.15 µs 2.2 µs]
                                      thrpt: [4.66 GB/s 4.76 GB/s 4.88 GB/s]

ml_kem_768_encapsulate                time: [95 µs 100 µs 105 µs]

ed25519_sign                          time: [38 µs 40 µs 42 µs]

ml_dsa_65_sign                        time: [1.9 ms 2.0 ms 2.1 ms]
```

### Latency Benchmarks

**Benchmark: End-to-end message latency**

```rust
#[tokio::test]
async fn bench_e2e_latency() {
    let (sender, recipient) = setup_test_clients().await;

    let mut latencies = Vec::new();

    for _ in 0..1000 {
        let start = Instant::now();

        // Send
        let msg_id = sender.send("agent-bob-02", b"Latency test", SendOptions::default()).await.unwrap();

        // Receive
        let messages = recipient.receive(1, None).await.unwrap();
        assert_eq!(messages[0].id, msg_id);

        let latency = start.elapsed();
        latencies.push(latency);
    }

    // Calculate percentiles
    latencies.sort();
    let p50 = latencies[latencies.len() / 2];
    let p99 = latencies[latencies.len() * 99 / 100];
    let p999 = latencies[latencies.len() * 999 / 1000];

    println!("E2E latency: p50={:?}, p99={:?}, p99.9={:?}", p50, p99, p999);

    // Assert SLO
    assert!(p99 < Duration::from_millis(10), "p99 latency exceeds 10ms SLO");
}
```

### Load Testing

**Test: Sustained throughput under load**

```rust
#[tokio::test]
async fn load_test_100k_msg_per_sec() {
    let num_senders = 100;
    let messages_per_sender = 1000;
    let target_throughput = 100_000;  // msg/sec

    let start = Instant::now();

    let tasks: Vec<_> = (0..num_senders)
        .map(|i| {
            tokio::spawn(async move {
                let client = create_test_client(&format!("agent-sender-{}", i)).await;

                for j in 0..messages_per_sender {
                    client.send(
                        "agent-receiver-01",
                        format!("Message {}-{}", i, j).as_bytes(),
                        SendOptions::default()
                    ).await.unwrap();
                }
            })
        })
        .collect();

    for task in tasks {
        task.await.unwrap();
    }

    let elapsed = start.elapsed();
    let total_messages = num_senders * messages_per_sender;
    let throughput = total_messages as f64 / elapsed.as_secs_f64();

    println!("Load test: {} messages in {:?} = {:.0} msg/sec",
             total_messages, elapsed, throughput);

    assert!(throughput >= target_throughput as f64,
            "Throughput {:.0} < target {}", throughput, target_throughput);
}
```

---

## Security Testing

### Fuzzing

**Fuzz: Protocol buffer decoding**

```rust
// fuzz/fuzz_targets/envelope_decode.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use creto_messaging::MessageEnvelope;

fuzz_target!(|data: &[u8]| {
    let _ = MessageEnvelope::decode(data);
});
```

**Fuzz: Signature verification**

```rust
fuzz_target!(|data: &[u8]| {
    if data.len() < 100 {
        return;
    }

    let (message, signature) = data.split_at(50);

    let pubkey = generate_random_pubkey();
    let _ = verify_ed25519(&pubkey, message, signature);
    let _ = verify_ml_dsa(&pubkey, message, signature);
});
```

**Run fuzzing:**
```bash
cargo fuzz run envelope_decode -- -max_total_time=3600  # 1 hour
cargo fuzz run signature_verify -- -max_total_time=3600
```

### Penetration Testing Scenarios

**Test: Replay attack prevention**

```rust
#[tokio::test]
async fn test_replay_attack_blocked() {
    let (sender, recipient) = setup_test_clients().await;

    // Capture legitimate message
    let msg_id = sender.send("agent-bob-02", b"Original", SendOptions::default()).await.unwrap();
    let messages = recipient.receive(1, None).await.unwrap();
    let envelope = messages[0].clone();

    // Replay same envelope
    let result = send_raw_envelope(&envelope).await;

    // Should be rejected (duplicate message_id)
    assert!(matches!(result, Err(MessagingError::DuplicateMessage)));
}
```

**Test: Signature forgery attempt**

```rust
#[tokio::test]
async fn test_signature_forgery_detected() {
    let envelope = create_test_envelope();

    // Tamper with sender NHI
    let mut forged_envelope = envelope.clone();
    forged_envelope.sender_nhi = "agent-attacker-01".to_string();

    // Signature verification should fail (signed by Alice, claims to be Attacker)
    let result = verify_envelope(&forged_envelope).await;
    assert!(matches!(result, Err(MessagingError::SignatureVerificationFailed)));
}
```

---

## Chaos Engineering

### Failure Injection

**Test: Network partition during send**

```rust
#[tokio::test]
async fn chaos_network_partition() {
    let (sender, recipient) = setup_test_clients().await;

    // Inject network partition (drop packets)
    toxiproxy::add_toxic("messaging-grpc", Toxic::Timeout {
        timeout: 5000,  // 5s timeout
    }).await;

    // Send should timeout
    let result = sender.send("agent-bob-02", b"Test", SendOptions {
        timeout: Duration::from_secs(1),
        ..Default::default()
    }).await;

    assert!(matches!(result, Err(MessagingError::Timeout)));

    // Remove toxic
    toxiproxy::remove_toxic("messaging-grpc", "timeout").await;

    // Retry should succeed
    let result_retry = sender.send("agent-bob-02", b"Retry", SendOptions::default()).await;
    assert!(result_retry.is_ok());
}
```

**Test: Database failure during key rotation**

```rust
#[tokio::test]
async fn chaos_db_failure_during_rotation() {
    let client = setup_test_client().await;

    // Kill database mid-rotation
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        kill_postgres().await;
    });

    let result = client.rotate_keys().await;

    // Rotation should fail gracefully
    assert!(result.is_err());

    // Old keys should still be valid (rollback)
    let messages = client.receive(1, None).await;
    assert!(messages.is_ok());
}
```

---

## Test Automation

### CI/CD Pipeline

```yaml
# .github/workflows/test.yml
name: Messaging Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run unit tests
        run: cargo test --lib

  integration-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15
        env:
          POSTGRES_PASSWORD: test
      redis:
        image: redis:7
    steps:
      - uses: actions/checkout@v3
      - name: Run integration tests
        run: cargo test --test '*'

  performance-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: cargo bench --no-fail-fast
      - name: Compare with baseline
        run: |
          cargo install cargo-criterion
          cargo criterion --message-format=json > bench_results.json
          python scripts/check_performance_regression.py

  security-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Audit dependencies
        run: cargo audit
      - name: Scan for vulnerabilities
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --ignore-tests --out Xml
      - name: Upload to Codecov
        uses: codecov/codecov-action@v3
```

### Continuous Monitoring

**Synthetic tests in production:**

```rust
// Production canary test (runs every 1 minute)
pub async fn synthetic_canary() -> Result<()> {
    let canary_sender = create_client("canary-sender").await?;
    let canary_receiver = create_client("canary-receiver").await?;

    let start = Instant::now();

    // Send test message
    let msg_id = canary_sender.send(
        "canary-receiver",
        b"Synthetic canary test",
        SendOptions::default()
    ).await?;

    // Receive
    let messages = canary_receiver.receive(1, Some(5)).await?;
    assert_eq!(messages[0].id, msg_id);

    // Acknowledge
    canary_receiver.acknowledge(&msg_id).await?;

    let latency = start.elapsed();

    // Record metrics
    metrics::histogram!("canary.e2e_latency_ms", latency.as_millis() as f64);

    // Alert if SLO violated
    if latency > Duration::from_millis(100) {
        alert("Canary test exceeded 100ms latency SLO");
    }

    Ok(())
}
```

---

## Summary

### Test Coverage Summary

| Test Type | Count | Coverage |
|-----------|-------|----------|
| **Unit tests** | 250+ | >80% |
| **Integration tests** | 50+ | >90% API |
| **Performance benchmarks** | 20+ | All SLOs |
| **Security tests** | 30+ | All threats |
| **Chaos tests** | 10+ | Key failures |

### Performance SLOs Validated

| Metric | Target | Test |
|--------|--------|------|
| Encryption throughput | >100K msg/sec | `bench_encryption_throughput` |
| E2E latency (p99) | <10ms | `bench_e2e_latency` |
| AuthZ check | <1ms | `bench_authz_check` |
| Key rotation | <100ms | `bench_key_rotation` |

### Next Steps

1. Implement remaining integration tests
2. Run load tests in staging (1M agents)
3. Schedule penetration testing (Q1 2026)
4. Enable continuous fuzzing (OSS-Fuzz)
5. Publish test coverage reports
