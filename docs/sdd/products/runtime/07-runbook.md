---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
github_issue: "#TBD"
oss_reference: kubernetes-sigs/agent-sandbox
---

# RTM-07: Runtime Operations Runbook

## Table of Contents
1. [Service Overview](#service-overview)
2. [Operational Procedures](#operational-procedures)
3. [Warm Pool Management](#warm-pool-management)
4. [Incident Response](#incident-response)
5. [Performance Tuning](#performance-tuning)
6. [Monitoring and Alerts](#monitoring-and-alerts)
7. [Troubleshooting](#troubleshooting)
8. [Maintenance Windows](#maintenance-windows)

---

## Service Overview

### Architecture Components

```
┌──────────────────────────────────────────────────────────┐
│             Runtime Host (Production)                    │
│  ┌────────────────────────────────────────────────────┐  │
│  │  Application (Orchestrator)                        │  │
│  │  - Embeds creto-runtime library                    │  │
│  │  - Manages sandboxes                               │  │
│  └────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────┐  │
│  │  creto-runtime Library                             │  │
│  │  - SandboxManager                                  │  │
│  │  - WarmPoolManager                                 │  │
│  │  - EgressController                                │  │
│  │  - AttestationService                              │  │
│  └────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────┐  │
│  │  containerd + Runtime Backends                     │  │
│  │  - runsc (gVisor)                                  │  │
│  │  - kata-runtime (Kata Containers)                  │  │
│  └────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────┘
         │                      │
         ↓                      ↓
  ┌──────────────┐      ┌──────────────┐
  │     etcd     │      │  S3/Blob     │
  │(pool state)  │      │(checkpoints) │
  └──────────────┘      └──────────────┘
```

### Key Metrics

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| **Cold Spawn Latency (gVisor)** | <2s (p99) | >3s |
| **Cold Spawn Latency (Kata)** | <5s (p99) | >7s |
| **Warm Pool Claim Latency** | <100ms (p99) | >200ms |
| **Egress Check Latency** | <1ms (p99) | >5ms |
| **Concurrent Sandboxes** | ≥1000 | <500 |
| **Pool Ready Count** | ≥min_ready | <min_ready |
| **Service Availability** | 99.9% | <99% |

### Dependencies

| Service | Purpose | Impact if Down |
|---------|---------|----------------|
| **containerd** | OCI runtime management | Cannot spawn sandboxes |
| **etcd** | Pool state storage | Pool stats unavailable, claim fails |
| **creto-nhi** | Agent identity | Cannot bind NHI to sandbox |
| **creto-authz** | Policy enforcement | RequireAuthz egress blocked |
| **creto-audit** | Event logging | No audit trail (continue operation) |
| **S3/Blob** | Checkpoint storage | Cannot checkpoint/restore |

---

## Operational Procedures

### Service Startup

**Prerequisites**:
- containerd running (`systemctl status containerd`)
- etcd cluster healthy (`etcdctl endpoint health`)
- gVisor runsc installed (`which runsc`)
- Kata runtime installed (optional, `which kata-runtime`)

**Procedure**:
```bash
# 1. Verify dependencies
systemctl status containerd
etcdctl --endpoints=http://localhost:2379 endpoint health

# 2. Start runtime service (if separate process)
systemctl start creto-runtime

# 3. Verify service health
curl http://localhost:8080/health
# Expected: {"status": "healthy", "components": {"containerd": "ok", "etcd": "ok"}}

# 4. Check warm pools
curl http://localhost:8080/v1/warm-pools
# Expected: List of pools with statistics

# 5. Verify sandbox spawn
creto-runtime-cli spawn --image python:3.11 --runtime gvisor
# Expected: Sandbox spawned in <2s
```

---

### Service Shutdown

**Graceful Shutdown (Maintenance)**:
```bash
# 1. Drain warm pools (prevent new claims)
creto-runtime-cli pool drain --all

# 2. Wait for active sandboxes to finish (or set timeout)
while [ $(creto-runtime-cli list --status Running | wc -l) -gt 0 ]; do
  echo "Waiting for $(creto-runtime-cli list --status Running | wc -l) sandboxes to finish..."
  sleep 10
done

# 3. Checkpoint long-running sandboxes (optional)
for sandbox in $(creto-runtime-cli list --status Running); do
  creto-runtime-cli checkpoint $sandbox
done

# 4. Stop service
systemctl stop creto-runtime

# 5. Verify all sandboxes terminated
creto-runtime-cli list
# Expected: Empty list
```

**Emergency Shutdown**:
```bash
# 1. Force terminate all sandboxes
creto-runtime-cli terminate --all --force

# 2. Stop service
systemctl stop creto-runtime
```

---

### Runtime Backend Upgrade

**gVisor Upgrade** (Low Risk):
```bash
# 1. Download new runsc binary
wget https://storage.googleapis.com/gvisor/releases/release/20241215/runsc
chmod +x runsc

# 2. Verify version
./runsc --version
# runsc version release-20241215.0

# 3. Test with canary sandbox
creto-runtime-cli spawn --image alpine --runtime gvisor --runsc-path ./runsc
# Expected: Successful spawn

# 4. Replace system runsc
sudo mv /usr/local/bin/runsc /usr/local/bin/runsc.old
sudo mv ./runsc /usr/local/bin/runsc

# 5. Restart containerd (picks up new runsc)
sudo systemctl restart containerd

# 6. Verify existing sandboxes unaffected
creto-runtime-cli list
# Expected: All sandboxes still Running

# 7. Monitor for errors
journalctl -u containerd -f | grep -i error
```

**Kata Containers Upgrade** (Medium Risk):
```bash
# 1. Drain Kata sandboxes
creto-runtime-cli drain --runtime kata

# 2. Install new Kata version
sudo apt-get update
sudo apt-get install kata-containers=3.3.0

# 3. Verify version
kata-runtime --version
# kata-runtime : 3.3.0

# 4. Test with canary
creto-runtime-cli spawn --image alpine --runtime kata

# 5. Re-enable Kata runtime
creto-runtime-cli enable --runtime kata
```

---

## Warm Pool Management

### Pool Health Monitoring

```bash
# Check pool statistics
creto-runtime-cli pool stats agent-executor-pool

# Output:
# Pool: agent-executor-pool
# Ready: 12/20
# Claimed: 8
# Warming: 2
# Claims/min: 15.3
# Avg claim latency: 82ms
# P99 claim latency: 95ms
# Oldest sandbox age: 45m
```

**Alerts**:
- **Low Ready Count**: `ready_count < min_ready`
  - Action: Check warming errors, scale up pool
- **High Claim Latency**: `p99_claim_latency > 200ms`
  - Action: Increase pool size, investigate NHI binding delay
- **Old Sandboxes**: `oldest_sandbox_age > max_age`
  - Action: Verify eviction logic, manually evict

---

### Pool Scaling

**Manual Scale Up**:
```bash
# Increase max_ready for pool
creto-runtime-cli pool update agent-executor-pool \
  --min-ready 20 \
  --max-ready 50

# Verify scaling
watch 'creto-runtime-cli pool stats agent-executor-pool'
```

**Manual Scale Down**:
```bash
# Decrease max_ready (excess sandboxes evicted gradually)
creto-runtime-cli pool update agent-executor-pool \
  --min-ready 5 \
  --max-ready 10

# Force eviction (aggressive)
creto-runtime-cli pool evict agent-executor-pool --count 10
```

**Auto-Scaling Tuning**:
```bash
# Adjust headroom factor (higher = more ready sandboxes)
creto-runtime-cli pool update agent-executor-pool \
  --auto-scaling-headroom 2.0  # 2x claims/min

# Disable auto-scaling
creto-runtime-cli pool update agent-executor-pool \
  --auto-scaling-enabled false
```

---

### Pool Recovery

**Scenario: Pool Drained (0 Ready)**

```bash
# 1. Check pool configuration
creto-runtime-cli pool get agent-executor-pool

# 2. Inspect warming errors
creto-runtime-cli pool logs agent-executor-pool --phase Warming
# Common errors:
# - Image pull failure (registry unreachable)
# - Insufficient resources (CPU/memory exhausted)
# - Backend unavailable (containerd down)

# 3. Fix root cause (e.g., restart containerd)
sudo systemctl restart containerd

# 4. Manually spawn sandboxes to replenish
for i in {1..10}; do
  creto-runtime-cli pool spawn agent-executor-pool &
done

# 5. Monitor recovery
watch 'creto-runtime-cli pool stats agent-executor-pool'
```

---

## Incident Response

### Sandbox Escape Detected

**Severity**: CRITICAL

**Detection**:
- Alerting system flags anomalous syscalls
- Security scan detects unauthorized host access
- Audit log shows sandbox accessing `/proc/1/`

**Response**:
```bash
# 1. ISOLATE: Terminate affected sandbox immediately
creto-runtime-cli terminate <sandbox-id> --force

# 2. CONTAIN: Disable runtime backend temporarily
systemctl stop containerd

# 3. INVESTIGATE: Collect forensic data
# - Sandbox logs
journalctl -u containerd | grep <sandbox-id> > sandbox-escape-forensics.log

# - Attestation record
creto-runtime-cli attestation get <sandbox-id> > attestation.json

# - Checkpoint (if available)
aws s3 cp s3://checkpoints/<checkpoint-id>/ ./checkpoint-forensics/ --recursive

# 4. PATCH: Update gVisor/Kata if CVE identified
# (See Runtime Backend Upgrade)

# 5. NOTIFY: Alert security team and users
# - Send incident report
# - Revoke compromised credentials

# 6. RESUME: Restart service after patch
systemctl start containerd
```

---

### Network Exfiltration Attempt

**Severity**: HIGH

**Detection**:
- Audit log shows repeated egress denials
- AuthZ service flags suspicious destination patterns

**Response**:
```bash
# 1. IDENTIFY: Find sandbox making exfiltration attempts
creto-runtime-cli audit search --event EgressDenied --destination <attacker-domain>
# Output: sandbox-id, agent-nhi, destination, timestamp

# 2. TERMINATE: Kill affected sandbox
creto-runtime-cli terminate <sandbox-id>

# 3. ANALYZE: Review sandbox configuration
creto-runtime-cli get <sandbox-id> --json > sandbox-config.json

# Check if NetworkPolicy misconfigured (unintended Allow rule)
jq '.spec.networkPolicy' sandbox-config.json

# 4. BLOCK: Update NetworkPolicy to explicitly deny attacker domain
creto-runtime-cli network-policy add-rule --destination <attacker-domain> --action Deny

# 5. AUDIT: Check for lateral movement
creto-runtime-cli audit search --agent-nhi <compromised-agent> --event EgressAttempt
```

---

### Resource Exhaustion (DoS)

**Severity**: MEDIUM

**Detection**:
- Host CPU/memory usage at 100%
- Spawn failures due to insufficient resources
- Alerting system flags high resource usage

**Response**:
```bash
# 1. IDENTIFY: Find resource-hogging sandboxes
creto-runtime-cli list --sort-by cpu_usage --descending
# or
creto-runtime-cli list --sort-by memory_usage --descending

# 2. TERMINATE: Kill top resource consumers
for sandbox in $(creto-runtime-cli list --cpu-usage '>80%' --limit 5); do
  creto-runtime-cli terminate $sandbox
done

# 3. VERIFY: Check host resources freed
top -b -n 1 | grep creto-runtime
free -h

# 4. INVESTIGATE: Check if resource limits enforced
creto-runtime-cli get <sandbox-id> --json | jq '.spec.resourceLimits'

# 5. PREVENT: Lower resource limits for future sandboxes
# Update SandboxTemplate or default limits
```

---

### Warm Pool Exhaustion

**Severity**: LOW-MEDIUM

**Detection**:
- Claim latency >200ms (falling back to cold spawn)
- Pool statistics show `ready_count = 0`

**Response**:
```bash
# 1. IMMEDIATE: Increase pool size
creto-runtime-cli pool update <pool-id> --min-ready 20 --max-ready 50

# 2. DIAGNOSE: Check why sandboxes not warming
creto-runtime-cli pool logs <pool-id> --phase Warming
# Common issues:
# - Image pull throttled (rate limit)
# - containerd unresponsive
# - etcd write failures

# 3. WORKAROUND: Allow cold spawns temporarily
# (Clients automatically fall back, no action needed)

# 4. MONITOR: Verify pool replenishing
watch 'creto-runtime-cli pool stats <pool-id>'

# 5. LONG-TERM: Adjust auto-scaling parameters
creto-runtime-cli pool update <pool-id> --auto-scaling-headroom 2.0
```

---

## Performance Tuning

### Optimizing Cold Spawn Latency

**1. Image Pull Optimization**:
```bash
# Use digest pinning (skip tag resolution)
# Before: "python:3.11" (requires tag lookup)
# After: "python@sha256:abcd1234..." (direct pull)

# Pre-pull common images on hosts
for image in python:3.11 node:18 golang:1.21; do
  ctr image pull docker.io/library/$image
done
```

**2. gVisor Platform Tuning**:
```yaml
# /etc/containerd/config.toml
[plugins."io.containerd.runtime.v1.linux"]
  runtime = "runsc"
  runtime_root = "/run/containerd/runsc"
  shim = "containerd-shim-runsc-v1"

[plugins."io.containerd.runtime.v1.linux".options]
  # Use kvm platform (faster than ptrace)
  platform = "kvm"

  # Disable unnecessary features
  file_access = "shared"  # Faster than "exclusive"
  overlay = true
```

**3. Kata Tuning**:
```toml
# /etc/kata-containers/configuration.toml
[hypervisor.qemu]
  # Use Cloud Hypervisor (faster boot than QEMU)
  path = "/usr/bin/cloud-hypervisor"

  # Reduce VM boot time
  kernel_params = "quiet console=hvc0 agent.log=debug"

  # Smaller initrd
  initrd = "/usr/share/kata-containers/kata-containers-initrd-mini.img"
```

---

### Optimizing Claim Latency

**1. NHI Binding Optimization**:
```rust
// Cache NHI public keys to avoid repeated fetches
pub struct NhiCache {
    cache: Arc<DashMap<AgentIdentity, PublicKey>>,
}

impl NhiCache {
    pub async fn get_or_fetch(&self, agent_nhi: &AgentIdentity) -> Result<PublicKey, Error> {
        if let Some(pubkey) = self.cache.get(agent_nhi) {
            return Ok(pubkey.clone());
        }

        let pubkey = self.nhi_client.fetch_public_key(agent_nhi).await?;
        self.cache.insert(agent_nhi.clone(), pubkey.clone());
        Ok(pubkey)
    }
}
```

**2. Secret Injection Optimization**:
```rust
// Batch secret fetches
pub async fn fetch_secrets_batch(
    &self,
    refs: &[SecretRef],
) -> Result<Vec<ResolvedSecret>, Error> {
    // Parallel fetch (vs sequential)
    let futures = refs.iter().map(|r| self.fetch_secret(r));
    futures::future::try_join_all(futures).await
}
```

**3. Pool State Storage**:
```bash
# Use local etcd (low latency)
# Before: Remote etcd cluster (10ms roundtrip)
# After: Local etcd on same host (<1ms)

# Or use in-memory cache with async etcd sync
```

---

### Optimizing Egress Check Latency

**1. Policy Cache**:
```rust
// Bloom filter for fast negative lookups
pub struct PolicyCache {
    bloom: BloomFilter,
    redis: RedisClient,
}

impl PolicyCache {
    pub async fn check(&self, dest: &str) -> Option<EgressDecision> {
        // Fast path: Bloom filter (no network call)
        if !self.bloom.contains(dest) {
            return Some(EgressDecision::Deny);  // Definitely not allowed
        }

        // Slow path: Redis lookup
        self.redis.get(dest).await.ok()
    }
}
```

**2. AuthZ Client Connection Pooling**:
```rust
// Reuse HTTP/2 connections to creto-authz
let authz_client = AuthzClient::new_with_pool(
    "http://authz-service:8080",
    PoolConfig {
        max_idle_per_host: 10,
        keep_alive: Duration::from_secs(90),
    },
);
```

---

## Monitoring and Alerts

### Key Metrics to Monitor

**Prometheus Metrics**:
```prometheus
# Spawn latency histogram
creto_runtime_spawn_latency_seconds{backend="gvisor"} # p50, p99
creto_runtime_spawn_latency_seconds{backend="kata"}

# Claim latency histogram
creto_runtime_pool_claim_latency_seconds{pool_id="agent-executor-pool"} # p50, p99

# Egress check latency
creto_runtime_egress_check_latency_seconds # p99

# Sandbox count gauge
creto_runtime_sandboxes_total{status="Ready"}
creto_runtime_sandboxes_total{status="Running"}

# Pool statistics
creto_runtime_pool_ready_count{pool_id="agent-executor-pool"}
creto_runtime_pool_claimed_count{pool_id="agent-executor-pool"}
creto_runtime_pool_claims_per_minute{pool_id="agent-executor-pool"}

# Egress enforcement
creto_runtime_egress_attempts_total{decision="Allow"}
creto_runtime_egress_attempts_total{decision="Deny"}
```

---

### Alerting Rules

```yaml
# alerts.yml
groups:
  - name: creto-runtime
    interval: 30s
    rules:
      # High spawn latency
      - alert: HighSpawnLatency
        expr: histogram_quantile(0.99, creto_runtime_spawn_latency_seconds{backend="gvisor"}) > 3
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "gVisor spawn latency p99 >3s"
          description: "Spawn latency {{ $value }}s exceeds target 2s"

      # Pool exhaustion
      - alert: WarmPoolExhausted
        expr: creto_runtime_pool_ready_count < creto_runtime_pool_min_ready
        for: 2m
        labels:
          severity: critical
        annotations:
          summary: "Warm pool {{ $labels.pool_id }} exhausted"
          description: "Ready count {{ $value }} below min_ready"

      # High egress denials
      - alert: HighEgressDenials
        expr: rate(creto_runtime_egress_attempts_total{decision="Deny"}[5m]) > 10
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High egress denial rate"
          description: "{{ $value }} egress denials per second"

      # Sandbox escape attempt
      - alert: SandboxEscapeAttempt
        expr: increase(creto_runtime_syscall_violations_total[1m]) > 0
        labels:
          severity: critical
        annotations:
          summary: "Sandbox escape attempt detected"
          description: "Anomalous syscalls detected in sandbox {{ $labels.sandbox_id }}"
```

---

## Troubleshooting

### Common Issues

#### Issue: Sandbox Spawn Fails with "Image Not Found"

**Symptoms**:
```
Error: SpawnError::ImageNotFound("python:3.11")
```

**Diagnosis**:
```bash
# Check containerd can pull image
ctr image pull docker.io/library/python:3.11

# Check registry authentication
cat ~/.docker/config.json

# Test network connectivity to registry
curl -I https://registry-1.docker.io/v2/
```

**Resolution**:
```bash
# Pre-pull image
ctr image pull docker.io/library/python:3.11

# Or use digest pinning
creto-runtime-cli spawn --image python@sha256:abcd1234...
```

---

#### Issue: Warm Pool Claim Timeout

**Symptoms**:
```
Error: ClaimError::Timeout (waited 30s, no ready sandboxes)
```

**Diagnosis**:
```bash
# Check pool statistics
creto-runtime-cli pool stats <pool-id>
# Output: ready_count = 0, warming_count = 10

# Check warming errors
creto-runtime-cli pool logs <pool-id> --phase Warming
# Common: containerd timeout, etcd write failure
```

**Resolution**:
```bash
# Increase pool size
creto-runtime-cli pool update <pool-id> --max-ready 50

# Restart containerd if unresponsive
sudo systemctl restart containerd

# Fallback: Use cold spawn
# (Clients automatically fall back)
```

---

#### Issue: Egress Connection Refused (Unexpected)

**Symptoms**:
Agent code reports `ECONNREFUSED` for allowed destination.

**Diagnosis**:
```bash
# Check NetworkPolicy
creto-runtime-cli get <sandbox-id> --json | jq '.spec.networkPolicy'

# Verify destination matches rule
creto-runtime-cli network-policy test --destination api.anthropic.com

# Check audit log for denials
creto-runtime-cli audit search --sandbox-id <sandbox-id> --event EgressDenied
```

**Resolution**:
```bash
# Add missing egress rule
creto-runtime-cli network-policy add-rule \
  --destination api.anthropic.com \
  --action RequireAuthz

# Or update NetworkPolicy in SandboxSpec
```

---

#### Issue: Attestation Verification Failed

**Symptoms**:
```
Error: VerificationResult::Invalid { reason: "Ed25519 signature invalid" }
```

**Diagnosis**:
```bash
# Check signing key rotation
creto-runtime-cli keys list
# Output: active key, deprecated keys

# Verify attestation timestamp
creto-runtime-cli attestation get <sandbox-id> --json | jq '.validUntil'

# Check if attestation expired
date -u
```

**Resolution**:
```bash
# If key rotated, verify using old key
creto-runtime-cli attestation verify <attestation> --key-version v1

# If expired, regenerate attestation
# (Only possible if sandbox still running)

# If tampered, investigate security incident
```

---

## Maintenance Windows

### Planned Maintenance Checklist

**Pre-Maintenance (T-24 hours)**:
- [ ] Announce maintenance window to users
- [ ] Scale up warm pools (buffer for downtime)
- [ ] Checkpoint long-running sandboxes
- [ ] Verify backup restoration procedures
- [ ] Prepare rollback plan

**During Maintenance**:
- [ ] Drain warm pools
- [ ] Wait for sandboxes to finish (or force terminate)
- [ ] Perform upgrade/maintenance
- [ ] Verify service health
- [ ] Restore warm pools
- [ ] Test canary sandboxes

**Post-Maintenance (T+2 hours)**:
- [ ] Monitor error rates
- [ ] Check spawn/claim latencies
- [ ] Verify audit logging
- [ ] Review attestation signatures
- [ ] Send all-clear notification

---

### Rollback Procedures

**Scenario: Upgrade Causes Spawn Failures**

```bash
# 1. Revert to previous runsc/kata version
sudo mv /usr/local/bin/runsc /usr/local/bin/runsc.new
sudo mv /usr/local/bin/runsc.old /usr/local/bin/runsc

# 2. Restart containerd
sudo systemctl restart containerd

# 3. Verify rollback successful
creto-runtime-cli spawn --image alpine --runtime gvisor

# 4. Notify users of rollback
echo "Rollback complete, service restored"
```

---

## Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2025-12-25 | 0.1 | Claude | Initial runbook |
