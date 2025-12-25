---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
github_issue: "#TBD"
oss_reference: kubernetes-sigs/agent-sandbox
---

# RTM-04: Runtime API Design

## Table of Contents
1. [Rust Library API](#rust-library-api)
2. [gRPC API](#grpc-api)
3. [REST HTTP API](#rest-http-api)
4. [Python SDK](#python-sdk)
5. [CLI Interface](#cli-interface)
6. [Error Handling](#error-handling)

---

## Rust Library API

### SandboxRuntime Trait (Core Abstraction)

```rust
use async_trait::async_trait;

/// Core runtime trait implemented by applications embedding creto-runtime
#[async_trait]
pub trait SandboxRuntime: Send + Sync {
    /// Spawn new sandbox from specification
    ///
    /// # Arguments
    /// * `spec` - Sandbox specification (image, NHI, resources, network policy)
    ///
    /// # Returns
    /// * `SandboxHandle` - Handle to spawned sandbox with attestation
    ///
    /// # Errors
    /// * `SpawnError::ImageNotFound` - OCI image not found in registry
    /// * `SpawnError::AuthorizationDenied` - Agent lacks spawn permission
    /// * `SpawnError::ResourceExhausted` - Insufficient resources
    /// * `SpawnError::InvalidSpec` - Invalid configuration
    async fn spawn(&self, spec: &SandboxSpec) -> Result<SandboxHandle, SpawnError>;

    /// Terminate running sandbox
    ///
    /// # Arguments
    /// * `id` - Sandbox identifier
    ///
    /// # Errors
    /// * `TerminateError::NotFound` - Sandbox not found
    /// * `TerminateError::AlreadyTerminated` - Sandbox already terminated
    async fn terminate(&self, id: &SandboxId) -> Result<(), TerminateError>;

    /// Execute command in sandbox
    ///
    /// # Arguments
    /// * `id` - Sandbox identifier
    /// * `cmd` - Command to execute with arguments
    ///
    /// # Returns
    /// * `ExecOutput` - Stdout, stderr, exit code
    ///
    /// # Errors
    /// * `ExecError::SandboxNotReady` - Sandbox not in Ready state
    /// * `ExecError::CommandFailed` - Command execution failed
    /// * `ExecError::Timeout` - Command exceeded timeout
    async fn exec(&self, id: &SandboxId, cmd: Command) -> Result<ExecOutput, ExecError>;

    /// Query sandbox status
    ///
    /// # Arguments
    /// * `id` - Sandbox identifier
    ///
    /// # Returns
    /// * `SandboxStatus` - Current status (Creating, Ready, Running, etc.)
    async fn status(&self, id: &SandboxId) -> Result<SandboxStatus, StatusError>;

    /// List all sandboxes
    ///
    /// # Returns
    /// * `Vec<SandboxInfo>` - List of sandboxes with metadata
    async fn list(&self) -> Result<Vec<SandboxInfo>, ListError>;

    /// Create checkpoint of sandbox
    ///
    /// # Arguments
    /// * `id` - Sandbox identifier
    ///
    /// # Returns
    /// * `CheckpointId` - Identifier for checkpoint (for restore)
    ///
    /// # Errors
    /// * `CheckpointError::NotSupported` - Runtime backend doesn't support checkpoint
    /// * `CheckpointError::FailedToCapture` - CRIU failed
    async fn checkpoint(&self, id: &SandboxId) -> Result<CheckpointId, CheckpointError>;

    /// Restore sandbox from checkpoint
    ///
    /// # Arguments
    /// * `checkpoint_id` - Checkpoint identifier
    /// * `nhi_override` - Optional new agent NHI (defaults to original)
    ///
    /// # Returns
    /// * `SandboxHandle` - Handle to restored sandbox
    async fn restore(
        &self,
        checkpoint_id: &CheckpointId,
        nhi_override: Option<AgentIdentity>,
    ) -> Result<SandboxHandle, RestoreError>;
}
```

---

### WarmPool API

```rust
/// Warm pool manager for fast sandbox allocation
#[async_trait]
pub trait WarmPoolManager: Send + Sync {
    /// Create warm pool from configuration
    ///
    /// # Arguments
    /// * `config` - Pool configuration (template, min/max size, reusability)
    ///
    /// # Returns
    /// * `PoolId` - Pool identifier
    async fn create_pool(&self, config: WarmPoolConfig) -> Result<PoolId, CreatePoolError>;

    /// Claim sandbox from pool (bind NHI)
    ///
    /// # Arguments
    /// * `pool_id` - Pool identifier
    /// * `agent_nhi` - Agent identity to bind to claimed sandbox
    /// * `delegation_chain` - Delegation chain from agent to human
    ///
    /// # Returns
    /// * `SandboxHandle` - Handle to claimed sandbox (target: <100ms)
    ///
    /// # Errors
    /// * `ClaimError::NoReadySandboxes` - Pool has no ready sandboxes
    /// * `ClaimError::PoolNotFound` - Pool doesn't exist
    async fn claim(
        &self,
        pool_id: &PoolId,
        agent_nhi: &AgentIdentity,
        delegation_chain: &[AgentIdentity],
    ) -> Result<SandboxHandle, ClaimError>;

    /// Release sandbox back to pool
    ///
    /// # Arguments
    /// * `sandbox_id` - Sandbox identifier
    /// * `reusable` - Whether sandbox can be reused (if pool is reusable)
    ///
    /// # Behavior
    /// * If `reusable=true` and pool is reusable: Unbind NHI, reset state, return to pool
    /// * If `reusable=false` or pool is single-use: Terminate sandbox
    async fn release(&self, sandbox_id: &SandboxId, reusable: bool) -> Result<(), ReleaseError>;

    /// Get pool statistics
    ///
    /// # Arguments
    /// * `pool_id` - Pool identifier
    ///
    /// # Returns
    /// * `PoolStatistics` - Ready count, claim rate, latencies
    async fn pool_stats(&self, pool_id: &PoolId) -> Result<PoolStatistics, StatsError>;

    /// Destroy pool (terminate all sandboxes)
    ///
    /// # Arguments
    /// * `pool_id` - Pool identifier
    async fn destroy_pool(&self, pool_id: &PoolId) -> Result<(), DestroyPoolError>;
}
```

---

### AttestationService API

```rust
/// Attestation generation and verification
#[async_trait]
pub trait AttestationService: Send + Sync {
    /// Generate attestation for sandbox
    ///
    /// # Arguments
    /// * `handle` - Raw sandbox handle from runtime backend
    /// * `spec` - Original sandbox specification
    ///
    /// # Returns
    /// * `Attestation` - Signed attestation with platform evidence
    async fn generate(
        &self,
        handle: &RawSandboxHandle,
        spec: &SandboxSpec,
    ) -> Result<Attestation, AttestationError>;

    /// Verify attestation signature and claims
    ///
    /// # Arguments
    /// * `attestation` - Attestation to verify
    ///
    /// # Returns
    /// * `VerificationResult` - Valid, Invalid, or Expired
    async fn verify(&self, attestation: &Attestation) -> Result<VerificationResult, VerifyError>;

    /// Get attestation for sandbox
    ///
    /// # Arguments
    /// * `sandbox_id` - Sandbox identifier
    ///
    /// # Returns
    /// * `Attestation` - Attestation generated at spawn/claim time
    async fn get_attestation(&self, sandbox_id: &SandboxId) -> Result<Attestation, GetError>;
}
```

---

### Command Execution Types

```rust
/// Command to execute in sandbox
#[derive(Debug, Clone)]
pub struct Command {
    /// Program to execute
    pub program: String,

    /// Arguments
    pub args: Vec<String>,

    /// Environment variables
    pub env: HashMap<String, String>,

    /// Working directory
    pub cwd: Option<String>,

    /// Stdin data
    pub stdin: Option<Vec<u8>>,

    /// Timeout
    pub timeout: Option<Duration>,
}

impl Command {
    pub fn new(program: impl Into<String>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: HashMap::new(),
            cwd: None,
            stdin: None,
            timeout: None,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn env(mut self, key: impl Into<String>, val: impl Into<String>) -> Self {
        self.env.insert(key.into(), val.into());
        self
    }

    pub fn cwd(mut self, cwd: impl Into<String>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub fn stdin(mut self, stdin: Vec<u8>) -> Self {
        self.stdin = Some(stdin);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Output from command execution
#[derive(Debug, Clone)]
pub struct ExecOutput {
    /// Standard output
    pub stdout: Vec<u8>,

    /// Standard error
    pub stderr: Vec<u8>,

    /// Exit code
    pub exit_code: i32,

    /// Execution duration
    pub duration: Duration,
}

impl ExecOutput {
    pub fn stdout_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.stdout)
    }

    pub fn stderr_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.stderr)
    }

    pub fn success(&self) -> bool {
        self.exit_code == 0
    }
}
```

---

### Usage Examples

```rust
use creto_runtime::{SandboxManager, SandboxSpec, RuntimeBackend};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize runtime with gVisor backend
    let runtime = SandboxManager::new(RuntimeBackend::GVisor).await?;

    // Define sandbox specification
    let spec = SandboxSpec {
        image: "docker.io/library/python:3.11".into(),
        agent_nhi: get_agent_identity().await?,
        delegation_chain: vec![get_human_principal().await?],
        runtime_class: RuntimeClass::GVisor,
        resource_limits: ResourceLimits {
            cpu_millicores: 2000,  // 2 CPUs
            memory_bytes: 4 * 1024 * 1024 * 1024,  // 4GB
            ..Default::default()
        },
        network_policy: NetworkPolicy {
            default_action: NetworkAction::Deny,
            egress_rules: vec![
                EgressRule {
                    destination: EgressDestination::Domain("*.anthropic.com".into()),
                    action: NetworkAction::RequireAuthz,
                },
            ],
            dns_policy: Some(DnsPolicy {
                allowed_resolvers: vec!["8.8.8.8".parse()?],
                blocked_domains: vec![],
                require_authz_domains: vec![],
            }),
        },
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
                mount: SecretMount::EnvVar {
                    name: "ANTHROPIC_API_KEY".into(),
                },
            },
        ],
        ttl: Some(Duration::from_secs(3600)),
        idle_timeout: Some(Duration::from_secs(300)),
    };

    // Spawn sandbox
    let handle = runtime.spawn(&spec).await?;
    println!("Spawned sandbox: {}", handle.id.0);

    // Execute Python script
    let output = runtime.exec(
        &handle.id,
        Command::new("python3")
            .arg("-c")
            .arg("import os; print(os.environ['ANTHROPIC_API_KEY'][:10])")
            .timeout(Duration::from_secs(10)),
    ).await?;

    println!("Output: {}", output.stdout_str()?);

    // Terminate sandbox
    runtime.terminate(&handle.id).await?;

    Ok(())
}
```

**Warm Pool Example**:
```rust
use creto_runtime::{WarmPoolManager, WarmPoolConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool_manager = WarmPoolManager::new().await?;

    // Create warm pool
    let pool_config = WarmPoolConfig {
        id: PoolId::from("agent-executor-pool"),
        template: base_sandbox_spec(),
        min_ready: 10,
        max_ready: 50,
        max_age: Duration::from_secs(3600),
        reusable: true,
        auto_scaling: Some(AutoScalingConfig {
            enabled: true,
            headroom_factor: 1.5,
            interval: Duration::from_secs(10),
        }),
    };

    let pool_id = pool_manager.create_pool(pool_config).await?;

    // Fast claim (target: <100ms)
    let start = Instant::now();
    let handle = pool_manager.claim(
        &pool_id,
        &agent_nhi,
        &delegation_chain,
    ).await?;
    let latency = start.elapsed();
    println!("Claim latency: {:?}", latency);

    // Use sandbox...
    runtime.exec(&handle.id, Command::new("python3").arg("agent.py")).await?;

    // Release back to pool (reusable)
    pool_manager.release(&handle.id, true).await?;

    Ok(())
}
```

---

## gRPC API

### Service Definition (protobuf)

```protobuf
syntax = "proto3";

package creto.runtime.v1;

service SandboxService {
  // Spawn new sandbox
  rpc Spawn(SpawnRequest) returns (SpawnResponse);

  // Terminate sandbox
  rpc Terminate(TerminateRequest) returns (TerminateResponse);

  // Execute command in sandbox
  rpc Exec(ExecRequest) returns (stream ExecOutput);

  // Query sandbox status
  rpc GetStatus(GetStatusRequest) returns (GetStatusResponse);

  // List sandboxes
  rpc ListSandboxes(ListSandboxesRequest) returns (ListSandboxesResponse);

  // Create checkpoint
  rpc Checkpoint(CheckpointRequest) returns (CheckpointResponse);

  // Restore from checkpoint
  rpc Restore(RestoreRequest) returns (RestoreResponse);
}

service WarmPoolService {
  // Create warm pool
  rpc CreatePool(CreatePoolRequest) returns (CreatePoolResponse);

  // Claim sandbox from pool
  rpc Claim(ClaimRequest) returns (ClaimResponse);

  // Release sandbox to pool
  rpc Release(ReleaseRequest) returns (ReleaseResponse);

  // Get pool statistics
  rpc GetPoolStats(GetPoolStatsRequest) returns (GetPoolStatsResponse);

  // Destroy pool
  rpc DestroyPool(DestroyPoolRequest) returns (DestroyPoolResponse);
}

// Messages

message SpawnRequest {
  SandboxSpec spec = 1;
}

message SpawnResponse {
  string sandbox_id = 1;
  Attestation attestation = 2;
  SandboxStatus status = 3;
}

message SandboxSpec {
  string image = 1;
  AgentIdentity agent_nhi = 2;
  repeated AgentIdentity delegation_chain = 3;
  RuntimeClass runtime_class = 4;
  ResourceLimits resource_limits = 5;
  NetworkPolicy network_policy = 6;
  repeated SecretRef secrets = 7;
  optional uint64 ttl_seconds = 8;
  optional uint64 idle_timeout_seconds = 9;
}

message AgentIdentity {
  bytes public_key = 1;
  SignatureAlgorithm algorithm = 2;
}

enum SignatureAlgorithm {
  ED25519 = 0;
  ML_DSA_65 = 1;
}

enum RuntimeClass {
  GVISOR = 0;
  KATA = 1;
}

message ResourceLimits {
  uint32 cpu_millicores = 1;
  uint64 memory_bytes = 2;
  uint64 disk_bytes = 3;
  uint32 pid_limit = 4;
  optional uint64 network_bandwidth_bps = 5;
}

message NetworkPolicy {
  NetworkAction default_action = 1;
  repeated EgressRule egress_rules = 2;
  optional DnsPolicy dns_policy = 3;
}

enum NetworkAction {
  ALLOW = 0;
  DENY = 1;
  REQUIRE_AUTHZ = 2;
}

message EgressRule {
  EgressDestination destination = 1;
  NetworkAction action = 2;
}

message EgressDestination {
  oneof destination {
    string cidr_block = 1;
    string domain = 2;
    ServiceRef service_ref = 3;
  }
}

message ServiceRef {
  string name = 1;
  string namespace = 2;
}

message DnsPolicy {
  repeated string allowed_resolvers = 1;
  repeated string blocked_domains = 2;
  repeated string require_authz_domains = 3;
}

message SecretRef {
  string name = 1;
  SecretSource source = 2;
  SecretMount mount = 3;
}

message SecretSource {
  oneof source {
    NhiDelegated nhi_delegated = 1;
    K8sSecret k8s_secret = 2;
  }
}

message NhiDelegated {
  string secret_id = 1;
  DelegationScope delegation_scope = 2;
}

message DelegationScope {
  string resource = 1;
  repeated string actions = 2;
  uint64 ttl_seconds = 3;
}

message K8sSecret {
  string name = 1;
  string key = 2;
}

message SecretMount {
  oneof mount {
    EnvVar env_var = 1;
    File file = 2;
  }
}

message EnvVar {
  string name = 1;
}

message File {
  string path = 1;
  uint32 mode = 2;
}

message Attestation {
  string sandbox_id = 1;
  AgentIdentity agent_nhi = 2;
  repeated AgentIdentity delegation_chain = 3;
  bytes image_hash = 4;
  bytes config_hash = 5;
  bytes init_hash = 6;
  AttestationPlatform platform = 7;
  bytes platform_evidence = 8;
  uint64 created_at_unix_ms = 9;
  uint64 valid_until_unix_ms = 10;
  Signature signature = 11;
}

message AttestationPlatform {
  oneof platform {
    GVisorPlatform gvisor = 1;
    KataPlatform kata = 2;
  }
}

message GVisorPlatform {
  string version = 1;
}

message KataPlatform {
  string version = 1;
  string hypervisor = 2;
}

message Signature {
  bytes ed25519 = 1;
  optional bytes ml_dsa = 2;
}

enum SandboxStatus {
  CREATING = 0;
  READY = 1;
  RUNNING = 2;
  CHECKPOINTING = 3;
  CHECKPOINTED = 4;
  TERMINATING = 5;
  TERMINATED = 6;
  FAILED = 7;
}

message ExecRequest {
  string sandbox_id = 1;
  Command command = 2;
}

message Command {
  string program = 1;
  repeated string args = 2;
  map<string, string> env = 3;
  optional string cwd = 4;
  optional bytes stdin = 5;
  optional uint64 timeout_seconds = 6;
}

message ExecOutput {
  oneof output {
    bytes stdout = 1;
    bytes stderr = 2;
    int32 exit_code = 3;
  }
}

// Warm pool messages

message CreatePoolRequest {
  WarmPoolConfig config = 1;
}

message CreatePoolResponse {
  string pool_id = 1;
}

message WarmPoolConfig {
  string pool_id = 1;
  SandboxSpec template = 2;
  uint32 min_ready = 3;
  uint32 max_ready = 4;
  uint64 max_age_seconds = 5;
  bool reusable = 6;
  optional AutoScalingConfig auto_scaling = 7;
}

message AutoScalingConfig {
  bool enabled = 1;
  double headroom_factor = 2;
  uint64 interval_seconds = 3;
}

message ClaimRequest {
  string pool_id = 1;
  AgentIdentity agent_nhi = 2;
  repeated AgentIdentity delegation_chain = 3;
}

message ClaimResponse {
  string sandbox_id = 1;
  Attestation attestation = 2;
  uint64 claim_latency_ms = 3;
}

message ReleaseRequest {
  string sandbox_id = 1;
  bool reusable = 2;
}

message ReleaseResponse {}

message GetPoolStatsRequest {
  string pool_id = 1;
}

message GetPoolStatsResponse {
  PoolStatistics stats = 1;
}

message PoolStatistics {
  uint32 ready_count = 1;
  uint32 claimed_count = 2;
  uint32 warming_count = 3;
  double claims_per_minute = 4;
  uint64 avg_claim_latency_ms = 5;
  uint64 p99_claim_latency_ms = 6;
}
```

---

## REST HTTP API

### Endpoints

#### POST /v1/sandboxes
Spawn new sandbox.

**Request**:
```json
{
  "spec": {
    "image": "docker.io/library/python:3.11",
    "agentNhi": {
      "publicKey": "base64-encoded-key",
      "algorithm": "Ed25519"
    },
    "delegationChain": [],
    "runtimeClass": "gvisor",
    "resources": {
      "cpuMillicores": 2000,
      "memoryBytes": 4294967296
    },
    "networkPolicy": {
      "defaultAction": "Deny",
      "egressRules": [
        {
          "destination": { "domain": "*.anthropic.com" },
          "action": "RequireAuthz"
        }
      ]
    },
    "secrets": [],
    "ttlSeconds": 3600
  }
}
```

**Response** (201 Created):
```json
{
  "sandboxId": "sb-abc123",
  "attestation": {
    "sandboxId": "sb-abc123",
    "agentNhi": { "publicKey": "...", "algorithm": "Ed25519" },
    "imageHash": "sha256:abcd1234...",
    "configHash": "sha256:efgh5678...",
    "createdAtUnixMs": 1703548800000,
    "validUntilUnixMs": 1703552400000,
    "signature": {
      "ed25519": "base64-encoded-signature"
    }
  },
  "status": "Ready",
  "createdAt": "2024-12-25T12:00:00Z"
}
```

---

#### GET /v1/sandboxes/{sandboxId}/status
Get sandbox status.

**Response** (200 OK):
```json
{
  "sandboxId": "sb-abc123",
  "status": "Ready",
  "resourceUsage": {
    "cpuUsage": 12.5,
    "memoryBytes": 524288000,
    "diskBytes": 1073741824,
    "networkRxBytes": 1048576,
    "networkTxBytes": 2097152
  },
  "createdAt": "2024-12-25T12:00:00Z",
  "updatedAt": "2024-12-25T12:05:00Z"
}
```

---

#### POST /v1/sandboxes/{sandboxId}/exec
Execute command in sandbox.

**Request**:
```json
{
  "command": {
    "program": "python3",
    "args": ["-c", "print('Hello, World!')"],
    "env": {},
    "timeoutSeconds": 30
  }
}
```

**Response** (200 OK):
```json
{
  "stdout": "SGVsbG8sIFdvcmxkIQo=",  // base64
  "stderr": "",
  "exitCode": 0,
  "durationMs": 125
}
```

---

#### DELETE /v1/sandboxes/{sandboxId}
Terminate sandbox.

**Response** (204 No Content)

---

#### POST /v1/sandboxes/{sandboxId}/checkpoint
Create checkpoint.

**Response** (200 OK):
```json
{
  "checkpointId": "ckpt-xyz789",
  "sizeBytes": 104857600
}
```

---

#### POST /v1/checkpoints/{checkpointId}/restore
Restore from checkpoint.

**Request**:
```json
{
  "nhiOverride": {
    "publicKey": "base64-encoded-key",
    "algorithm": "Ed25519"
  }
}
```

**Response** (201 Created):
```json
{
  "sandboxId": "sb-def456",
  "attestation": { /* ... */ },
  "status": "Ready"
}
```

---

#### POST /v1/warm-pools
Create warm pool.

**Request**:
```json
{
  "config": {
    "poolId": "agent-executor-pool",
    "template": { /* SandboxSpec without agentNhi */ },
    "minReady": 10,
    "maxReady": 50,
    "maxAgeSeconds": 3600,
    "reusable": true,
    "autoScaling": {
      "enabled": true,
      "headroomFactor": 1.5,
      "intervalSeconds": 10
    }
  }
}
```

**Response** (201 Created):
```json
{
  "poolId": "agent-executor-pool"
}
```

---

#### POST /v1/warm-pools/{poolId}/claim
Claim sandbox from pool.

**Request**:
```json
{
  "agentNhi": {
    "publicKey": "base64-encoded-key",
    "algorithm": "Ed25519"
  },
  "delegationChain": []
}
```

**Response** (200 OK):
```json
{
  "sandboxId": "sb-ghi789",
  "attestation": { /* ... */ },
  "claimLatencyMs": 87
}
```

---

#### POST /v1/warm-pools/{poolId}/release/{sandboxId}
Release sandbox back to pool.

**Request**:
```json
{
  "reusable": true
}
```

**Response** (204 No Content)

---

#### GET /v1/warm-pools/{poolId}/stats
Get pool statistics.

**Response** (200 OK):
```json
{
  "readyCount": 12,
  "claimedCount": 8,
  "warmingCount": 2,
  "claimsPerMinute": 15.3,
  "avgClaimLatencyMs": 82,
  "p99ClaimLatencyMs": 95
}
```

---

## Python SDK

### Installation
```bash
pip install creto-runtime-sdk
```

### Context Manager Pattern

```python
from creto_runtime import SandboxManager, SandboxSpec, RuntimeClass
from creto_runtime.types import ResourceLimits, NetworkPolicy, NetworkAction, EgressRule

# Initialize runtime
runtime = SandboxManager(backend=RuntimeClass.GVISOR)

# Context manager pattern (auto-terminate on exit)
with runtime.spawn(
    SandboxSpec(
        image="docker.io/library/python:3.11",
        agent_nhi=get_agent_identity(),
        runtime_class=RuntimeClass.GVISOR,
        resources=ResourceLimits(
            cpu_millicores=2000,
            memory_bytes=4 * 1024**3,
        ),
        network_policy=NetworkPolicy(
            default_action=NetworkAction.DENY,
            egress_rules=[
                EgressRule(
                    destination={"domain": "*.anthropic.com"},
                    action=NetworkAction.REQUIRE_AUTHZ,
                ),
            ],
        ),
        ttl_seconds=3600,
    )
) as sandbox:
    # Execute commands
    output = sandbox.exec("python3", "-c", "print('Hello')")
    print(output.stdout.decode())

    # Sandbox automatically terminated on context exit
```

### Async API

```python
import asyncio
from creto_runtime import AsyncSandboxManager

async def main():
    runtime = AsyncSandboxManager(backend=RuntimeClass.GVISOR)

    # Spawn sandbox
    handle = await runtime.spawn(spec)

    # Execute command
    output = await runtime.exec(
        handle.id,
        program="python3",
        args=["-c", "import os; print(os.environ['API_KEY'][:10])"],
        timeout=30,
    )

    print(f"Exit code: {output.exit_code}")
    print(f"Output: {output.stdout.decode()}")

    # Terminate
    await runtime.terminate(handle.id)

asyncio.run(main())
```

### Warm Pool API

```python
from creto_runtime import WarmPoolManager, WarmPoolConfig

pool_manager = WarmPoolManager()

# Create pool
pool_id = pool_manager.create_pool(
    WarmPoolConfig(
        pool_id="agent-executor-pool",
        template=base_spec,
        min_ready=10,
        max_ready=50,
        reusable=True,
    )
)

# Claim sandbox (fast: <100ms)
handle = pool_manager.claim(pool_id, agent_nhi, delegation_chain)

# Use sandbox
output = runtime.exec(handle.id, "python3", "agent.py")

# Release back to pool
pool_manager.release(handle.id, reusable=True)
```

---

## CLI Interface

### Installation
```bash
cargo install creto-runtime-cli
```

### Commands

#### Spawn Sandbox
```bash
creto-runtime spawn \
  --image python:3.11 \
  --runtime gvisor \
  --cpu 2000 \
  --memory 4G \
  --network-policy deny-all \
  --egress-allow "*.anthropic.com:RequireAuthz" \
  --secret nhi:api-key-anthropic:ANTHROPIC_API_KEY \
  --ttl 3600

# Output:
# Sandbox spawned: sb-abc123
# Status: Ready
# Attestation hash: sha256:abcd1234...
```

#### Execute Command
```bash
creto-runtime exec sb-abc123 -- python3 -c "print('Hello')"

# Output:
# Hello
# Exit code: 0
```

#### Query Status
```bash
creto-runtime status sb-abc123

# Output:
# Sandbox: sb-abc123
# Status: Ready
# CPU usage: 12.5%
# Memory: 500MB / 4GB
# Created: 2024-12-25 12:00:00 UTC
```

#### Terminate
```bash
creto-runtime terminate sb-abc123

# Output:
# Sandbox sb-abc123 terminated
```

#### Create Warm Pool
```bash
creto-runtime pool create \
  --id agent-executor-pool \
  --template base-spec.yaml \
  --min-ready 10 \
  --max-ready 50 \
  --reusable

# Output:
# Warm pool created: agent-executor-pool
```

#### Claim from Pool
```bash
creto-runtime pool claim agent-executor-pool \
  --agent-nhi @agent-key.pub

# Output:
# Claimed sandbox: sb-def456
# Claim latency: 87ms
```

#### Pool Stats
```bash
creto-runtime pool stats agent-executor-pool

# Output:
# Pool: agent-executor-pool
# Ready: 12
# Claimed: 8
# Warming: 2
# Claims/min: 15.3
# Avg claim latency: 82ms
# P99 claim latency: 95ms
```

---

## Error Handling

### Error Types

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SpawnError {
    #[error("Image not found: {0}")]
    ImageNotFound(String),

    #[error("Authorization denied: {0}")]
    AuthorizationDenied(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Invalid specification: {0}")]
    InvalidSpec(String),

    #[error("Runtime backend unavailable: {0}")]
    BackendUnavailable(String),

    #[error("NHI service error: {0}")]
    NhiError(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum ExecError {
    #[error("Sandbox not ready: {0}")]
    SandboxNotReady(String),

    #[error("Sandbox not found: {0}")]
    SandboxNotFound(String),

    #[error("Command failed: exit code {0}")]
    CommandFailed(i32),

    #[error("Command timeout after {0:?}")]
    Timeout(Duration),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ClaimError {
    #[error("No ready sandboxes in pool: {0}")]
    NoReadySandboxes(String),

    #[error("Pool not found: {0}")]
    PoolNotFound(String),

    #[error("Claim timeout")]
    Timeout,

    #[error("NHI binding failed: {0}")]
    NhiBindingFailed(String),
}
```

### Error Responses (HTTP)

```json
{
  "error": {
    "code": "SANDBOX_NOT_FOUND",
    "message": "Sandbox sb-abc123 not found",
    "details": {
      "sandboxId": "sb-abc123"
    }
  }
}
```

**Error Codes**:
- `IMAGE_NOT_FOUND` - OCI image not found (404)
- `AUTHORIZATION_DENIED` - Agent lacks permission (403)
- `RESOURCE_EXHAUSTED` - Insufficient resources (503)
- `INVALID_SPEC` - Invalid configuration (400)
- `SANDBOX_NOT_FOUND` - Sandbox doesn't exist (404)
- `SANDBOX_NOT_READY` - Sandbox not in Ready state (409)
- `POOL_NOT_FOUND` - Pool doesn't exist (404)
- `NO_READY_SANDBOXES` - Pool has no ready sandboxes (503)
- `INTERNAL_ERROR` - Unexpected error (500)

---

## Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2025-12-25 | 0.1 | Claude | Initial API design |
