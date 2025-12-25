---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
github_issue: "#TBD"
oss_reference: kubernetes-sigs/agent-sandbox
---

# RTM-00: Runtime Product Overview

## Executive Summary

**creto-runtime** is the sandboxed execution environment for AI agents within the Creto Enablement Layer. It provides secure, isolated runtime environments with cryptographic attestation, dynamic network policy enforcement, and sub-100ms allocation from warm pools.

The product extracts proven patterns from **kubernetes-sigs/agent-sandbox** and rebuilds them in Rust with Sovereign primitives (NHI, Crypto-Agility, Authorization) integrated from the foundation.

## Vision

Enable **verifiable, isolated agent execution** where:
- Every sandbox is cryptographically bound to an agent's NHI
- Network egress is enforced by Authorization service (168ns policy evaluation)
- Platform attestations provide legal evidence of "who ran what, where"
- Warm pools enable <100ms allocation for interactive agent experiences
- Checkpoint/restore enables zero-downtime migration

## Problem Statement

Existing container orchestration platforms (Kubernetes, Docker, Nomad) are designed for **long-lived microservices**, not **short-lived agent tasks**. They lack:

1. **Cryptographic Identity Binding**: No integration between container identity and agent NHI
2. **Dynamic Network Policy**: Static NetworkPolicies cannot adapt based on runtime Authorization decisions
3. **Fast Allocation**: Cold starts (5-30s) unacceptable for interactive agents
4. **Attestation**: No cryptographic proof of execution environment
5. **NHI-Delegated Secrets**: Rely on static K8s Secrets, not dynamic delegation

**creto-runtime** solves these by building a **lightweight, agent-first execution layer** on top of proven isolation technologies (gVisor, Kata Containers).

## Scope

### In Scope (v1.0)

- **Sandbox Lifecycle**: Create, monitor, execute, terminate sandboxed environments
- **Warm Pool Management**: Pre-warmed sandboxes for <100ms allocation
- **Runtime Backends**: gVisor (user-space kernel), Kata Containers (VM isolation)
- **Network Egress Control**: Authorization-enforced egress policies with default deny
- **Platform Attestation**: Cryptographic proof linking sandbox to agent NHI and configuration
- **NHI-Delegated Secrets**: Dynamic secret injection via delegation service
- **Checkpoint/Restore**: Migration support for maintenance and fault tolerance

### Out of Scope (Deferred to v2+)

- **Kubernetes Operator**: Runtime is a library; operator is separate component
- **Image Registry Management**: Use existing OCI registries (containerd integration)
- **GPU/Accelerator Passthrough**: Focus on CPU workloads first
- **Confidential Computing**: SGX/TDX attestation deferred (software attestation sufficient)
- **Live Migration**: Checkpoint/restore adequate for v1

## Key Features

### 1. Agent Sandbox Pattern (OSS)

Extracted from **kubernetes-sigs/agent-sandbox**, Google's OSS project for ephemeral agent execution:

- **CRDs**: `Sandbox`, `SandboxTemplate`, `SandboxClaim`, `SandboxWarmPool`
- **Isolation**: gVisor (user-space kernel) or Kata (lightweight VMs)
- **Performance**: <2s cold start (gVisor), <100ms warm claim
- **Python SDK**: Context manager pattern for agent-friendly API

**Creto Extension**: Integrate NHI, Authorization, Crypto-Agility, Audit from foundation.

### 2. Warm Pool Architecture

Pre-warm sandboxes without NHI binding, bind identity on claim:

```
┌────────────────────────────────────────────────────┐
│             WarmPoolManager                        │
│  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐  │
│  │ Ready  │  │ Ready  │  │ Ready  │  │Warming │  │
│  │ (no NHI)  │ (no NHI)  │ (no NHI)  │        │  │
│  └────────┘  └────────┘  └────────┘  └────────┘  │
└────────────────────────────────────────────────────┘
         ↓ claim(agent_nhi) in <100ms
┌────────────────────────────────────────────────────┐
│  SandboxHandle { id, agent_nhi, attestation }     │
└────────────────────────────────────────────────────┘
```

**Benefits**:
- Interactive latency: <100ms vs 2-5s cold start
- Cost efficiency: Reusable pools amortize spawn cost
- Auto-scaling: Pool size adapts to claim rate

### 3. Cryptographic Attestation

Every sandbox spawn generates a platform attestation:

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
    pub platform: AttestationPlatform,  // gVisor | Kata
    pub platform_evidence: Vec<u8>,     // Runtime version, measurements

    // Timing
    pub created_at: Timestamp,
    pub valid_until: Timestamp,

    // Signature (Ed25519 + ML-DSA hybrid)
    pub signature: Signature,
}
```

**Use Cases**:
- **Compliance Audits**: Legal evidence of "who ran what, where"
- **Third-Party Verification**: Independent attestation validation without runtime access
- **Immutable Audit Trail**: Anchored in Merkle tree, consensus-replicated

### 4. Authorization-Enforced Network Egress

Integrate with **creto-authz** (168ns policy evaluation) for dynamic network policy:

```yaml
network_policy:
  default_action: Deny
  egress_rules:
    # Allow internal services
    - destination: "10.0.0.0/8"
      action: Allow

    # Require runtime AuthZ check for external APIs
    - destination: "*.anthropic.com"
      action: RequireAuthz
      authz_context:
        resource: "api.anthropic.com"
        action: "http.post"
        agent_nhi: <bound at claim>
```

**Enforcement Points**:
- **gVisor**: Netstack intercepts syscalls before network layer
- **Kata**: iptables + nftables in guest VM
- **Latency**: <1ms total (168ns policy + network overhead)

### 5. NHI-Delegated Secret Injection

Fetch secrets via **creto-nhi** delegation service at spawn/claim time:

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

**Benefits**:
- No static credentials in images or config
- Automatic rotation on lease renewal
- Audit trail of secret access (logged, never plaintext)

### 6. Checkpoint/Restore for Migration

Leverage **CRIU** (Checkpoint/Restore In Userspace) for migration:

```rust
// Before maintenance: checkpoint sandbox
let checkpoint_id = runtime.checkpoint(&handle.id).await?;
runtime.terminate(&handle).await?;

// After maintenance: restore on different host
let restored_handle = runtime.restore(&checkpoint_id, None).await?;
// Agent continues execution from checkpoint point
```

**Characteristics**:
- **Checkpoint Size**: <100MB for typical agent workload
- **Restore Latency**: <1s (faster than cold start)
- **Migration**: Cross-host if same runtime backend
- **Retention**: 24 hours default, configurable

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                      Client Application                         │
│  (Platform Orchestrator, Agent Controller, Swarm Manager)       │
└───────────────────────────┬─────────────────────────────────────┘
                            │ SandboxRuntime trait
                            ↓
┌─────────────────────────────────────────────────────────────────┐
│                     creto-runtime Library                       │
│  ┌───────────────┐  ┌──────────────┐  ┌──────────────────────┐ │
│  │SandboxManager │  │WarmPoolManager│  │ AttestationService  │ │
│  │(lifecycle)    │  │(<100ms claim)│  │ (sign, verify)      │ │
│  └───────┬───────┘  └──────┬───────┘  └──────────┬───────────┘ │
│          │                  │                     │              │
│  ┌───────┴──────────────────┴─────────────────────┴───────────┐ │
│  │           RuntimeAdapter (Backend Abstraction)             │ │
│  │   ┌──────────────┐          ┌──────────────────────┐       │ │
│  │   │GVisorRuntime │          │ KataRuntime          │       │ │
│  │   │(<2s cold)    │          │ (<5s cold, highest   │       │ │
│  │   │              │          │  security)           │       │ │
│  │   └──────────────┘          └──────────────────────┘       │ │
│  └────────────────────────────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │            EgressController                                │ │
│  │  (NetworkPolicy eval + creto-authz call)                   │ │
│  └────────────────────────────────────────────────────────────┘ │
└───────────────────────┬────────────────────────────────────────┘
                        │ integrates with
         ┌──────────────┼──────────────┬─────────────────┐
         ↓              ↓              ↓                 ↓
  ┌────────────┐ ┌────────────┐ ┌────────────┐  ┌──────────────┐
  │ creto-nhi  │ │creto-authz │ │creto-crypto│  │ creto-audit  │
  │(agent ID)  │ │(168ns eval)│ │(sign/verify)│  │(event log)   │
  └────────────┘ └────────────┘ └────────────┘  └──────────────┘
```

## Component Responsibilities

### SandboxManager
- Lifecycle: Create, start, stop, terminate
- Command execution with I/O capture
- Status querying and monitoring
- TTL enforcement and cleanup

### WarmPoolManager
- Maintain pools of pre-warmed sandboxes
- Auto-scale based on claim rate
- Evict sandboxes older than `max_age`
- NHI binding on claim (<100ms)
- Pool statistics and metrics

### RuntimeAdapter
- Unified `SandboxRuntime` trait
- Backend selection (gVisor vs Kata)
- Runtime-specific configuration
- Capability querying
- Fallback to available runtime

### EgressController
- Network policy evaluation
- Authorization service integration
- Connection interception (gVisor netstack, Kata iptables)
- Egress logging to audit
- DNS policy enforcement

### AttestationService
- Generate platform attestations
- Sign with runtime signing key (Ed25519 + ML-DSA)
- Include platform evidence (gVisor version, Kata measurements)
- Provide verification function for third parties
- Anchor attestations in audit log

### NetworkPolicyManager
- Parse NetworkPolicy configuration
- Evaluate egress rules (CIDR, domain, service refs)
- Cache policy decisions (bloom filter + Redis)
- Rate limiting per sandbox

## Data Flow: Sandbox Spawn

```
1. Client calls runtime.spawn(&spec)
   ↓
2. SandboxManager validates spec (AuthZ check: spawn permission)
   ↓
3. Select runtime backend (gVisor or Kata based on spec.runtime_class)
   ↓
4. RuntimeAdapter.spawn()
   - Pull OCI image (if not cached)
   - Create isolated environment (namespace, cgroups)
   - Bind agent NHI
   - Inject NHI-delegated secrets
   - Apply network policy
   ↓
5. AttestationService.generate()
   - Hash image, config, initial FS
   - Collect platform evidence
   - Sign with runtime key
   ↓
6. Log lifecycle event to creto-audit
   ↓
7. Return SandboxHandle { id, agent_nhi, attestation, status: Ready }
```

## Data Flow: Warm Pool Claim

```
1. Client calls runtime.claim(&pool_id, &agent_nhi, &delegation_chain)
   ↓
2. WarmPoolManager selects ready sandbox from pool (<1ms)
   ↓
3. Bind NHI to sandbox
   - Update sandbox metadata
   - Inject NHI-delegated secrets
   ↓
4. AttestationService.generate() (<50ms)
   ↓
5. Mark sandbox as in-use in pool state
   ↓
6. Log claim event to creto-audit
   ↓
7. Return SandboxHandle (<100ms total)
```

## Data Flow: Network Egress Check

```
1. Agent code makes outbound connection (e.g., curl https://api.anthropic.com)
   ↓
2. Runtime intercepts connection (gVisor netstack or Kata iptables)
   ↓
3. EgressController.check(&destination, &sandbox_id)
   - Parse NetworkPolicy egress rules
   - Match destination against rules
   ↓
4. If action = Allow: pass through
   If action = Deny: reject with ECONNREFUSED
   If action = RequireAuthz:
     - Call creto-authz with context (agent NHI, destination, action)
     - Wait for decision (168ns policy eval + network ~1ms)
     - Allow or deny based on AuthZ response
   ↓
5. Log egress attempt to creto-audit (allowed or denied)
   ↓
6. Return connection to agent or error
```

## Performance Characteristics

| Operation | Target | Measured |
|-----------|--------|----------|
| **Cold Spawn (gVisor)** | <2s (p99) | TBD |
| **Cold Spawn (Kata)** | <5s (p99) | TBD |
| **Warm Pool Claim** | <100ms (p99) | TBD |
| **Attestation Generation** | <50ms | TBD |
| **Egress Check** | <1ms (p99) | TBD |
| **Secret Injection** | <100ms | TBD |
| **Checkpoint** | <10s for 1GB sandbox | TBD |
| **Restore** | <1s warm, <5s cold | TBD |

## Security Model

### Threat Model

| Threat | Mitigation |
|--------|------------|
| **Sandbox Escape** | gVisor user-space kernel (200 syscalls), Kata VM isolation |
| **Network Exfiltration** | Default deny egress, AuthZ enforcement, audit logging |
| **Resource Exhaustion** | cgroups limits (CPU, memory, PID, network) |
| **Credential Theft** | NHI-delegated secrets (no static keys), TTL enforcement |
| **Attestation Forgery** | Hybrid Ed25519 + ML-DSA signature, platform evidence |
| **Man-in-the-Middle** | TLS for all external connections, AuthZ checks |

### Isolation Boundaries

```
┌─────────────────────────────────────────────────────────────┐
│                         Host OS                             │
│  ┌───────────────────────────────────────────────────────┐  │
│  │               creto-runtime Service                   │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐   │  │
│  │  │  Sandbox A  │  │  Sandbox B  │  │  Sandbox C  │   │  │
│  │  │             │  │             │  │             │   │  │
│  │  │  Agent NHI  │  │  Agent NHI  │  │  Agent NHI  │   │  │
│  │  │    (A)      │  │    (B)      │  │    (C)      │   │  │
│  │  │             │  │             │  │             │   │  │
│  │  │  gVisor or  │  │  gVisor or  │  │  gVisor or  │   │  │
│  │  │    Kata     │  │    Kata     │  │    Kata     │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────┘   │  │
│  │       ↑                  ↑                  ↑          │  │
│  │       └──────────────────┴──────────────────┘          │  │
│  │              EgressController (AuthZ)                 │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

- **PID Isolation**: Each sandbox has own PID namespace
- **Network Isolation**: Separate network namespace, egress enforced
- **Filesystem Isolation**: Read-only root, writable volumes with limits
- **CPU/Memory Isolation**: cgroups enforce hard limits

## Integration with Creto Platform

### creto-nhi (Agent Identity)
- Bind agent NHI at spawn/claim time
- Include delegation chain in attestation
- Verify NHI signature before spawn

### creto-authz (Authorization)
- Check spawn permission before creating sandbox
- Enforce network egress policy (168ns eval)
- Gate secret access via delegation check

### creto-crypto (Crypto-Agility)
- Sign attestations with hybrid Ed25519 + ML-DSA
- Hash image layers with SHA-256
- Support key rotation with 30-day overlap

### creto-audit (Audit Trail)
- Log lifecycle events (spawn, claim, terminate)
- Log egress attempts (allowed and denied)
- Log secret injections (no plaintext)
- Anchor attestations in Merkle tree

### creto-consensus (Replication)
- Replicate warm pool state across nodes
- Checkpoint durability (≥3 replicas)
- Leader election for pool management

## Deployment Model

**creto-runtime is a library, not a service.**

Clients (orchestrators, controllers) embed the library and call the `SandboxRuntime` trait:

```rust
use creto_runtime::{SandboxManager, SandboxSpec, RuntimeBackend};

#[tokio::main]
async fn main() {
    let manager = SandboxManager::new(RuntimeBackend::GVisor).await?;

    let spec = SandboxSpec {
        image: "agent-executor:v1".into(),
        agent_nhi: agent_identity,
        // ...
    };

    let handle = manager.spawn(&spec).await?;
    // Use sandbox...
}
```

**Optional Kubernetes Operator** (separate component):
- CRDs: `Sandbox`, `SandboxWarmPool`, `SandboxClaim`, `SandboxTemplate`
- Controller watches CRDs, calls creto-runtime library
- Enables declarative sandbox management via K8s

## Success Metrics

| Metric | Target | Business Impact |
|--------|--------|-----------------|
| **Warm Pool Claim Latency** | <100ms (p99) | Interactive agent UX |
| **Cold Spawn Latency (gVisor)** | <2s (p99) | Fast task startup |
| **Concurrent Sandboxes** | ≥1000 per instance | High-throughput swarms |
| **Sandbox Escape Rate** | 0 successful | Security compliance |
| **Attestation Verification** | 100% valid | Audit trustworthiness |
| **Egress Check Latency** | <1ms (p99) | Minimal overhead |

## Open Questions

1. **Warm Pool Sizing Strategy**
   - Static (fixed min/max) or dynamic (auto-scale)?
   - Proposal: Static for v1, auto-scale in v1.1

2. **Sandbox Reusability**
   - Reuse sandboxes after release or single-use?
   - Security: State leakage concern
   - Performance: Avoid pool churn
   - Proposal: Support both modes (configurable `reusable` flag)

3. **GPU Passthrough**
   - Kata supports GPU passthrough, gVisor does not
   - Use case: ML inference in sandbox
   - Proposal: Defer to v2, focus on CPU workloads first

4. **Confidential Computing**
   - SGX/TDX for hardware-rooted attestation?
   - Complexity: Platform-specific, limited availability
   - Proposal: Defer to v2, software attestation sufficient for v1

5. **Live Migration**
   - Migrate without checkpoint/restore downtime?
   - Complexity: CRIU integration, network state preservation
   - Proposal: Checkpoint/restore adequate for v1, defer live migration to v2

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| **gVisor Syscall Coverage** | Medium | High | Provide Kata fallback, document unsupported syscalls |
| **Warm Pool Cold Starts** | Low | Medium | Monitor pool statistics, alert on low ready count |
| **Attestation Key Compromise** | Low | Critical | Key rotation, HSM storage for production |
| **Network Policy Bypass** | Low | Critical | Security audit, penetration testing |
| **Resource Exhaustion** | Medium | Medium | Hard limits, monitoring, auto-scaling |

## References

1. **Agent Sandbox Specification**
   https://agent-sandbox.sigs.k8s.io/

2. **Google OSS Blog: Agent Sandbox Announcement**
   https://opensource.googleblog.com/2024/11/agent-sandbox.html

3. **gVisor Documentation**
   https://gvisor.dev/docs/

4. **Kata Containers Architecture**
   https://github.com/kata-containers/kata-containers/blob/main/docs/design/architecture/README.md

5. **CRIU (Checkpoint/Restore In Userspace)**
   https://criu.org/

6. **Kubernetes RuntimeClass**
   https://kubernetes.io/docs/concepts/containers/runtime-class/

## Glossary

- **Agent NHI**: Non-Human Identity cryptographically tied to an AI agent
- **Attestation**: Cryptographic proof linking sandbox execution to identity and configuration
- **Cold Start**: Spawning sandbox from OCI image (2-5s)
- **Delegation Chain**: Lineage from agent to root human principal
- **Egress**: Outbound network traffic from sandbox
- **gVisor**: User-space kernel providing container isolation (~200 syscalls)
- **Kata Containers**: Lightweight VM runtime for container isolation (full kernel)
- **Warm Pool**: Pre-warmed sandboxes ready for fast allocation (<100ms)
- **RuntimeClass**: Kubernetes abstraction for container runtime backend selection

## Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2025-12-25 | 0.1 | Claude | Initial draft |

---

**Next Steps**:
1. Review and approve this overview
2. Proceed to detailed architecture design (02-architecture.md)
3. Define data models and schemas (03-data-design.md)
4. Specify API contracts (04-api-design.md)
