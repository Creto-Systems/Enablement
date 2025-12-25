---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
oss_reference: kubernetes-sigs/agent-sandbox
---

# Product SDD: creto-runtime

## Purpose

Sandboxed execution environments for AI agents with cryptographic attestation. Enables organizations to run untrusted agent code in isolated containers with verifiable security boundaries, NHI-bound identity, and policy-enforced network egress.

## Scope

**In Scope:**
- SandboxSpec definition and lifecycle
- Warm pool management for fast allocation
- Multiple backends (gVisor, Kata Containers)
- Platform attestation (cryptographic proof)
- Network egress enforcement via Authorization
- NHI-delegated secret injection
- Checkpoint/restore for migration

**Out of Scope:**
- Kubernetes operator (separate component)
- Image registry management
- GPU/accelerator passthrough (v2)
- Confidential computing (SGX/TDX) (v2)

---

## 1. OSS Reference: Agent Sandbox

**Repository:** https://github.com/kubernetes-sigs/agent-sandbox

**Key Patterns to Extract:**
- CRDs: `Sandbox`, `SandboxTemplate`, `SandboxClaim`, `SandboxWarmPool`
- Warm pool for sub-second allocation
- Runtime abstraction (pluggable backends)
- Python SDK context manager pattern

**Differences from Agent Sandbox:**
- NHI binding at spawn (not claim)
- Cryptographic attestation of sandbox config
- Network egress via Authorization (not NetworkPolicy)
- Secret injection via NHI delegation
- Non-Kubernetes native (library, not operator)

---

## 2. Core Traits

### 2.1 SandboxSpec

```rust
/// Configuration for a sandboxed execution environment
pub struct SandboxSpec {
    // Image
    pub image: ImageRef,
    pub image_pull_policy: ImagePullPolicy,

    // Identity binding
    pub agent_nhi: AgentIdentity,           // Bound at spawn
    pub delegation_chain: Vec<AgentIdentity>,

    // Security
    pub attestation_policy: AttestationPolicy,
    pub syscall_filter: SyscallFilter,
    pub capabilities: Vec<Capability>,
    pub read_only_root: bool,

    // Resources
    pub resource_limits: ResourceLimits,
    pub storage_limits: StorageLimits,

    // Network
    pub network_policy: NetworkPolicy,       // Enforced via AuthZ
    pub dns_config: Option<DnsConfig>,

    // Secrets (NHI-delegated)
    pub secrets: Vec<SecretRef>,
    pub env_vars: HashMap<String, String>,

    // Lifecycle
    pub ttl: Duration,
    pub idle_timeout: Option<Duration>,
}

pub struct ResourceLimits {
    pub cpu_millicores: u32,        // 1000 = 1 CPU
    pub memory_bytes: u64,
    pub pids_limit: Option<u32>,
}

pub struct StorageLimits {
    pub root_fs_size: u64,
    pub tmp_size: u64,
    pub volumes: Vec<VolumeMount>,
}
```

### 2.2 Attestation

```rust
/// Cryptographic proof linking sandbox to identity and configuration
pub struct Attestation {
    pub sandbox_id: SandboxId,
    pub agent_nhi: AgentIdentity,

    // Configuration hashes
    pub image_hash: Hash,           // OCI image digest
    pub config_hash: Hash,          // Canonical hash of SandboxSpec
    pub init_hash: Hash,            // Hash of initial filesystem state

    // Platform proof
    pub platform: AttestationPlatform,
    pub platform_evidence: Vec<u8>,  // TEE quote, etc.

    // Timing
    pub created_at: Timestamp,
    pub valid_until: Timestamp,

    // Signature
    pub signature: Signature,        // Runtime signs attestation
}

pub enum AttestationPlatform {
    GVisor { version: String },
    Kata { version: String, hypervisor: String },
    SGX { mrenclave: [u8; 32], mrsigner: [u8; 32] },  // Future
    SEV { measurement: [u8; 48] },                     // Future
    None,  // Development mode only
}

pub struct AttestationPolicy {
    pub require_attestation: bool,
    pub allowed_platforms: Vec<AttestationPlatform>,
    pub max_attestation_age: Duration,
}
```

### 2.3 SandboxRuntime

```rust
/// Runtime abstraction for different sandbox backends
#[async_trait]
pub trait SandboxRuntime: Send + Sync {
    /// Get runtime identifier
    fn runtime_id(&self) -> &str;

    /// Spawn a new sandbox from spec
    async fn spawn(&self, spec: &SandboxSpec) -> Result<SandboxHandle, Error>;

    /// Terminate a running sandbox
    async fn terminate(&self, handle: &SandboxHandle) -> Result<(), Error>;

    /// Execute command in sandbox
    async fn exec(
        &self,
        handle: &SandboxHandle,
        cmd: &Command,
    ) -> Result<ExecOutput, Error>;

    /// Get sandbox status
    async fn status(&self, handle: &SandboxHandle) -> Result<SandboxStatus, Error>;

    /// Generate attestation for sandbox
    async fn attest(&self, handle: &SandboxHandle) -> Result<Attestation, Error>;

    /// Checkpoint sandbox state (for migration)
    async fn checkpoint(&self, handle: &SandboxHandle) -> Result<Checkpoint, Error>;

    /// Restore sandbox from checkpoint
    async fn restore(&self, checkpoint: &Checkpoint) -> Result<SandboxHandle, Error>;
}

// Backend implementations
pub struct GVisorRuntime { ... }
pub struct KataRuntime { ... }
```

### 2.4 WarmPoolManager

```rust
/// Manages pre-warmed sandbox pools for fast allocation
#[async_trait]
pub trait WarmPoolManager: Send + Sync {
    /// Claim a pre-warmed sandbox and bind NHI
    async fn claim(
        &self,
        pool_id: &PoolId,
        agent_nhi: &AgentIdentity,
        delegation_chain: &[AgentIdentity],
    ) -> Result<SandboxHandle, Error>;

    /// Release sandbox back to pool (if reusable) or terminate
    async fn release(&self, handle: SandboxHandle) -> Result<(), Error>;

    /// Get pool statistics
    async fn pool_stats(&self, pool_id: &PoolId) -> Result<PoolStats, Error>;
}

pub struct WarmPool {
    pub id: PoolId,
    pub template: SandboxSpec,      // Base spec (no NHI yet)
    pub min_ready: usize,
    pub max_ready: usize,
    pub max_age: Duration,
    pub reusable: bool,             // Can sandboxes be recycled?
}

pub struct PoolStats {
    pub ready: usize,
    pub in_use: usize,
    pub warming: usize,
    pub avg_claim_latency: Duration,
}
```

---

## 3. Data Models

### 3.1 SandboxHandle

```rust
pub struct SandboxHandle {
    pub id: SandboxId,
    pub spec: SandboxSpec,
    pub attestation: Option<Attestation>,
    pub status: SandboxStatus,
    pub created_at: Timestamp,
    pub bound_at: Option<Timestamp>,    // When NHI was bound
    pub runtime: String,                // "gvisor", "kata", etc.
}

pub enum SandboxStatus {
    Creating,
    Warming,                            // In warm pool, no NHI yet
    Ready,                              // NHI bound, ready for exec
    Running { pid: u32 },
    Paused,
    Checkpointed { checkpoint_id: CheckpointId },
    Terminated { exit_code: i32, at: Timestamp },
    Failed { error: String, at: Timestamp },
}
```

### 3.2 Network Policy

```rust
pub struct NetworkPolicy {
    /// Default action for unlisted destinations
    pub default_action: NetworkAction,

    /// Explicit rules (checked via Authorization at runtime)
    pub egress_rules: Vec<EgressRule>,

    /// DNS resolution policy
    pub dns_policy: DnsPolicy,
}

pub struct EgressRule {
    pub destination: EgressDestination,
    pub action: NetworkAction,
    pub authorization_check: bool,  // Check AuthZ for each connection?
}

pub enum EgressDestination {
    Any,
    CidrBlock(IpNetwork),
    Domain(String),                 // *.example.com
    DomainExact(String),            // api.example.com
    Service(ServiceRef),            // Internal service reference
}

pub enum NetworkAction {
    Allow,
    Deny,
    RequireAuthz,  // Check Authorization for each connection
}
```

### 3.3 Secret Injection

```rust
pub struct SecretRef {
    pub name: String,
    pub source: SecretSource,
    pub mount: SecretMount,
}

pub enum SecretSource {
    /// Fetch from NHI delegation
    NhiDelegated {
        secret_id: String,
        delegation_scope: DelegationScope,
    },
    /// Static value (development only)
    Static { value: String },
}

pub enum SecretMount {
    /// Mount as file
    File { path: PathBuf, mode: u32 },
    /// Set as environment variable
    EnvVar { name: String },
}
```

---

## 4. Architecture

### 4.1 Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      creto-runtime                          │
│                                                             │
│  ┌─────────────────┐         ┌─────────────────────────┐   │
│  │  Sandbox        │         │    Warm Pool            │   │
│  │  Manager        │────────►│    Manager              │   │
│  └────────┬────────┘         └────────────┬────────────┘   │
│           │                               │                │
│           │                               ▼                │
│           │                  ┌─────────────────────────┐   │
│           │                  │   Pre-warmed Sandboxes  │   │
│           │                  │   (no NHI bound yet)    │   │
│           │                  └─────────────────────────┘   │
│           │                               │                │
│           ▼                               ▼                │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Runtime Abstraction Layer              │   │
│  └─────────┬──────────────────────────────┬────────────┘   │
│            │                              │                 │
│            ▼                              ▼                 │
│       ┌─────────┐                   ┌─────────┐            │
│       │ gVisor  │                   │  Kata   │            │
│       │ Backend │                   │ Backend │            │
│       └─────────┘                   └─────────┘            │
│                                                             │
│  ┌─────────────────┐         ┌─────────────────────────┐   │
│  │  Attestation    │         │    Secret Injector      │   │
│  │  Generator      │         │    (NHI-delegated)      │   │
│  └─────────────────┘         └─────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
    ┌─────────┐          ┌─────────┐         ┌─────────┐
    │  AuthZ  │          │   NHI   │         │  Audit  │
    │(egress) │          │(secrets)│         │  (log)  │
    └─────────┘          └─────────┘         └─────────┘
```

### 4.2 Spawn Flow

```
Spawn Request
     │
     ▼
┌─────────────────┐
│ Authorization   │
│ (check spawn)   │
└────────┬────────┘
         │ Allow
         ▼
┌─────────────────┐      ┌──────────────┐
│ Check Warm Pool │─────►│ Pool Has     │──── Yes ───┐
│                 │      │ Ready?       │            │
└────────┬────────┘      └──────────────┘            │
         │ No                                        │
         ▼                                           │
┌─────────────────┐                                  │
│ Cold Spawn      │                                  │
│ (2-5s)          │                                  │
└────────┬────────┘                                  │
         │                                           │
         └───────────────────┬───────────────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Bind NHI to     │
                    │ Sandbox         │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Generate        │
                    │ Attestation     │
                    └────────┬────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Fetch & Inject  │◄───── NHI delegation
                    │ Secrets         │
                    └────────┬────────┘
                             │
                             ▼
                        Sandbox Ready
                        (<100ms warm, <5s cold)
```

### 4.3 Egress Enforcement

```
Sandbox → External Network
     │
     ▼
┌─────────────────┐
│ Intercept       │
│ Connection      │
│ (gVisor/Kata)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Policy Check    │
│ (local rules)   │
└────────┬────────┘
         │ RequireAuthz
         ▼
┌─────────────────┐
│ Authorization   │────────────────────────┐
│ Check           │                        │
└────────┬────────┘                        │
         │                                 │
    ┌────┴────┐                           ▼
    ▼         ▼                      ┌─────────┐
 Allow      Deny                     │  Audit  │
    │         │                      │  Log    │
    ▼         ▼                      └─────────┘
 Forward   Drop
 Packet    Packet
```

---

## 5. API Design

### 5.1 Sandbox Lifecycle

```rust
impl RuntimeService {
    /// Spawn a new sandbox
    pub async fn spawn(&self, spec: SandboxSpec) -> Result<SandboxHandle, Error>;

    /// Claim from warm pool (preferred)
    pub async fn claim(
        &self,
        pool_id: &PoolId,
        agent_nhi: &AgentIdentity,
        delegation_chain: &[AgentIdentity],
    ) -> Result<SandboxHandle, Error>;

    /// Execute command in sandbox
    pub async fn exec(
        &self,
        sandbox_id: &SandboxId,
        command: Command,
    ) -> Result<ExecOutput, Error>;

    /// Terminate sandbox
    pub async fn terminate(&self, sandbox_id: &SandboxId) -> Result<(), Error>;

    /// Get attestation
    pub async fn get_attestation(&self, sandbox_id: &SandboxId) -> Result<Attestation, Error>;
}
```

### 5.2 Warm Pool Management

```rust
impl RuntimeService {
    /// Create warm pool
    pub async fn create_pool(&self, pool: WarmPool) -> Result<PoolId, Error>;

    /// Delete warm pool
    pub async fn delete_pool(&self, pool_id: &PoolId) -> Result<(), Error>;

    /// Scale pool
    pub async fn scale_pool(&self, pool_id: &PoolId, min: usize, max: usize) -> Result<(), Error>;

    /// Get pool statistics
    pub async fn pool_stats(&self, pool_id: &PoolId) -> Result<PoolStats, Error>;
}
```

### 5.3 Checkpoint/Restore

```rust
impl RuntimeService {
    /// Checkpoint sandbox state
    pub async fn checkpoint(&self, sandbox_id: &SandboxId) -> Result<CheckpointId, Error>;

    /// Restore from checkpoint
    pub async fn restore(
        &self,
        checkpoint_id: &CheckpointId,
        new_nhi: Option<&AgentIdentity>,  // Re-bind to different agent
    ) -> Result<SandboxHandle, Error>;
}
```

---

## 6. Integration Points

### 6.1 Authorization Integration

```rust
// Spawn authorization
authz.check(CheckRequest {
    agent: agent_nhi,
    action: "sandbox_spawn",
    resource: format!("image://{}", image_ref),
}).await?;

// Egress authorization (per-connection)
authz.check(CheckRequest {
    agent: sandbox.agent_nhi,
    action: "network_egress",
    resource: format!("https://{}:{}", dest_host, dest_port),
}).await?;
```

### 6.2 NHI Integration

```rust
// Bind identity to sandbox
sandbox.bind_identity(&agent_nhi, &delegation_chain).await?;

// Fetch delegated secrets
let secrets = nhi.get_delegated_secrets(
    &agent_nhi,
    &delegation_chain,
    &secret_refs,
).await?;

// Inject secrets into sandbox
sandbox.inject_secrets(secrets).await?;
```

### 6.3 Audit Integration

```rust
// Sandbox lifecycle events
audit.log(AuditRecord {
    who: agent_nhi,
    what: "sandbox_spawned",
    resource: format!("sandbox://{}", sandbox_id),
    outcome: Outcome::Success,
    ..
}).await?;

// Egress events
audit.log(AuditRecord {
    who: sandbox.agent_nhi,
    what: "network_egress",
    resource: format!("https://{}:{}", dest_host, dest_port),
    outcome: Outcome::Success,  // or Denied
    ..
}).await?;
```

---

## 7. Performance Requirements

| Metric | Target | Notes |
|--------|--------|-------|
| Warm pool claim | <100ms p99 | NHI binding + attestation |
| Cold spawn (gVisor) | <2s p99 | Full spawn from image |
| Cold spawn (Kata) | <5s p99 | VM startup overhead |
| Exec latency | <10ms | Command execution overhead |
| Egress authz check | <1ms | Per-connection check |
| Checkpoint size | <100MB | For typical agent workload |

---

## 8. Security Considerations

### 8.1 Threat Model

| Threat | Mitigation |
|--------|------------|
| Sandbox escape | gVisor/Kata isolation, syscall filtering |
| Identity spoofing | NHI binding at spawn, attestation |
| Secret exfiltration | NHI-scoped delegation, short TTL |
| Unauthorized egress | AuthZ-enforced network policy |
| Supply chain attack | Image digest verification, attestation |

### 8.2 Attestation Verification

```rust
// Verifier checks attestation before trusting sandbox output
fn verify_attestation(attestation: &Attestation) -> Result<(), VerificationError> {
    // 1. Verify signature
    verify_signature(&attestation.signature, &attestation)?;

    // 2. Check platform trust
    verify_platform_evidence(&attestation.platform, &attestation.platform_evidence)?;

    // 3. Verify freshness
    ensure!(attestation.valid_until > now(), "Attestation expired");

    // 4. Check config hash matches expected
    verify_config_hash(&attestation.config_hash, expected_config)?;

    Ok(())
}
```

---

## 9. Open Questions

1. Should we support live migration of sandboxes between hosts?
2. What's the warm pool sizing strategy (static vs dynamic)?
3. Should sandboxes be reusable after release (security vs efficiency)?
4. How do we handle sandbox crash recovery?
5. Should we support GPU passthrough in v1?

---

## 10. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
