---
status: draft
author: Claude
created: 2025-12-25
updated: 2025-12-25
reviewers: []
github_issue: "#TBD"
oss_reference: kubernetes-sigs/agent-sandbox
---

# RTM-03: Runtime Data Design

## Table of Contents
1. [CRD Schemas](#crd-schemas)
2. [Rust Data Models](#rust-data-models)
3. [State Management](#state-management)
4. [Storage Strategy](#storage-strategy)
5. [Attestation Records](#attestation-records)
6. [Audit Events](#audit-events)

---

## CRD Schemas

### Sandbox CRD

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: sandboxes.runtime.creto.ai
spec:
  group: runtime.creto.ai
  versions:
    - name: v1alpha1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              required: [image, agentNhi, runtimeClass]
              properties:
                # Image configuration
                image:
                  type: string
                  description: "OCI image reference (e.g., docker.io/library/python:3.11)"

                # Identity binding
                agentNhi:
                  type: object
                  required: [publicKey, algorithm]
                  properties:
                    publicKey:
                      type: string
                      description: "Agent public key (base64-encoded)"
                    algorithm:
                      type: string
                      enum: [Ed25519, ML-DSA-65]
                    delegationChain:
                      type: array
                      items:
                        type: object
                        properties:
                          publicKey:
                            type: string
                          algorithm:
                            type: string

                # Runtime configuration
                runtimeClass:
                  type: string
                  enum: [gvisor, kata]
                  default: gvisor

                # Resource limits
                resources:
                  type: object
                  properties:
                    cpuMillicores:
                      type: integer
                      minimum: 100
                      maximum: 64000
                      default: 1000
                    memoryBytes:
                      type: integer
                      minimum: 134217728  # 128MB
                      maximum: 137438953472  # 128GB
                      default: 2147483648  # 2GB
                    diskBytes:
                      type: integer
                      minimum: 1073741824  # 1GB
                      maximum: 1099511627776  # 1TB
                      default: 10737418240  # 10GB

                # Network policy
                networkPolicy:
                  type: object
                  properties:
                    defaultAction:
                      type: string
                      enum: [Allow, Deny]
                      default: Deny
                    egressRules:
                      type: array
                      items:
                        type: object
                        required: [destination, action]
                        properties:
                          destination:
                            type: object
                            oneOf:
                              - required: [cidrBlock]
                              - required: [domain]
                              - required: [serviceRef]
                            properties:
                              cidrBlock:
                                type: string
                                pattern: '^([0-9]{1,3}\.){3}[0-9]{1,3}/[0-9]{1,2}$'
                              domain:
                                type: string
                              serviceRef:
                                type: object
                                properties:
                                  name:
                                    type: string
                                  namespace:
                                    type: string
                          action:
                            type: string
                            enum: [Allow, Deny, RequireAuthz]
                    dnsPolicy:
                      type: object
                      properties:
                        allowedResolvers:
                          type: array
                          items:
                            type: string
                        blockedDomains:
                          type: array
                          items:
                            type: string

                # Secrets
                secrets:
                  type: array
                  items:
                    type: object
                    required: [name, source, mount]
                    properties:
                      name:
                        type: string
                      source:
                        type: object
                        oneOf:
                          - required: [nhiDelegated]
                          - required: [k8sSecret]
                        properties:
                          nhiDelegated:
                            type: object
                            required: [secretId, delegationScope]
                            properties:
                              secretId:
                                type: string
                              delegationScope:
                                type: object
                                properties:
                                  resource:
                                    type: string
                                  actions:
                                    type: array
                                    items:
                                      type: string
                                  ttl:
                                    type: integer
                          k8sSecret:
                            type: object
                            properties:
                              name:
                                type: string
                              key:
                                type: string
                      mount:
                        type: object
                        oneOf:
                          - required: [envVar]
                          - required: [file]
                        properties:
                          envVar:
                            type: object
                            properties:
                              name:
                                type: string
                          file:
                            type: object
                            properties:
                              path:
                                type: string
                              mode:
                                type: integer

                # Lifecycle
                ttl:
                  type: integer
                  description: "Time-to-live in seconds"
                  minimum: 60
                  maximum: 86400  # 24 hours

                idleTimeout:
                  type: integer
                  description: "Idle timeout in seconds"
                  minimum: 60

            status:
              type: object
              properties:
                phase:
                  type: string
                  enum: [Creating, Ready, Running, Checkpointing, Checkpointed, Terminating, Terminated, Failed]

                conditions:
                  type: array
                  items:
                    type: object
                    properties:
                      type:
                        type: string
                      status:
                        type: string
                        enum: [True, False, Unknown]
                      lastTransitionTime:
                        type: string
                        format: date-time
                      reason:
                        type: string
                      message:
                        type: string

                sandboxId:
                  type: string

                attestation:
                  type: object
                  properties:
                    hash:
                      type: string
                    signature:
                      type: string
                    createdAt:
                      type: string
                      format: date-time
                    validUntil:
                      type: string
                      format: date-time

                resourceUsage:
                  type: object
                  properties:
                    cpuUsage:
                      type: string
                    memoryUsage:
                      type: string
                    diskUsage:
                      type: string

                createdAt:
                  type: string
                  format: date-time
```

---

### SandboxTemplate CRD

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: sandboxtemplates.runtime.creto.ai
spec:
  group: runtime.creto.ai
  versions:
    - name: v1alpha1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              description: "Template for Sandbox specification (agentNhi omitted, bound at claim)"
              properties:
                image:
                  type: string
                runtimeClass:
                  type: string
                  enum: [gvisor, kata]
                resources:
                  type: object
                  # Same as Sandbox.spec.resources
                networkPolicy:
                  type: object
                  # Same as Sandbox.spec.networkPolicy
                secrets:
                  type: array
                  # Same as Sandbox.spec.secrets (NHI binding done at claim)
```

---

### SandboxWarmPool CRD

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: sandboxwarmpools.runtime.creto.ai
spec:
  group: runtime.creto.ai
  versions:
    - name: v1alpha1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              required: [template, minReady, maxReady]
              properties:
                template:
                  type: object
                  description: "Reference to SandboxTemplate or inline spec"
                  oneOf:
                    - required: [templateRef]
                    - required: [inline]
                  properties:
                    templateRef:
                      type: object
                      properties:
                        name:
                          type: string
                        namespace:
                          type: string
                    inline:
                      type: object
                      # Same as SandboxTemplate.spec

                minReady:
                  type: integer
                  minimum: 1
                  maximum: 100
                  default: 5

                maxReady:
                  type: integer
                  minimum: 1
                  maximum: 100
                  default: 20

                maxAge:
                  type: integer
                  description: "Maximum sandbox age in pool (seconds)"
                  default: 3600  # 1 hour

                reusable:
                  type: boolean
                  description: "Whether sandboxes can be reused after release"
                  default: true

                autoScaling:
                  type: object
                  properties:
                    enabled:
                      type: boolean
                      default: true
                    headroomFactor:
                      type: number
                      minimum: 1.0
                      maximum: 3.0
                      default: 1.5

            status:
              type: object
              properties:
                readyCount:
                  type: integer
                claimedCount:
                  type: integer
                warmingCount:
                  type: integer

                statistics:
                  type: object
                  properties:
                    claimsPerMinute:
                      type: number
                    avgClaimLatencyMs:
                      type: number
                    p99ClaimLatencyMs:
                      type: number

                conditions:
                  type: array
                  items:
                    type: object
```

---

### SandboxClaim CRD

```yaml
apiVersion: apiextensions.k8s.io/v1
kind: CustomResourceDefinition
metadata:
  name: sandboxclaims.runtime.creto.ai
spec:
  group: runtime.creto.ai
  versions:
    - name: v1alpha1
      served: true
      storage: true
      schema:
        openAPIV3Schema:
          type: object
          properties:
            spec:
              type: object
              required: [poolRef, agentNhi]
              properties:
                poolRef:
                  type: object
                  required: [name]
                  properties:
                    name:
                      type: string
                    namespace:
                      type: string

                agentNhi:
                  type: object
                  # Same as Sandbox.spec.agentNhi

                ttlOverride:
                  type: integer
                  description: "Override pool's default TTL"

            status:
              type: object
              properties:
                phase:
                  type: string
                  enum: [Pending, Claimed, Released, Failed]

                sandboxRef:
                  type: object
                  properties:
                    name:
                      type: string
                    namespace:
                      type: string

                claimLatencyMs:
                  type: number

                claimedAt:
                  type: string
                  format: date-time
```

---

## Rust Data Models

### Core Sandbox Types

```rust
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Unique sandbox identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SandboxId(String);

impl SandboxId {
    pub fn new() -> Self {
        Self(format!("sb-{}", uuid::Uuid::new_v4()))
    }
}

/// Sandbox specification (user input)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSpec {
    /// OCI image reference
    pub image: ImageRef,

    /// Agent identity (cryptographic)
    pub agent_nhi: AgentIdentity,

    /// Delegation chain (agent -> ... -> human)
    pub delegation_chain: Vec<AgentIdentity>,

    /// Runtime backend selection
    pub runtime_class: RuntimeClass,

    /// Resource limits
    pub resource_limits: ResourceLimits,

    /// Network egress policy
    pub network_policy: NetworkPolicy,

    /// NHI-delegated secrets
    pub secrets: Vec<SecretRef>,

    /// Time-to-live (automatic termination)
    pub ttl: Option<Duration>,

    /// Idle timeout (terminate if no exec for duration)
    pub idle_timeout: Option<Duration>,
}

/// OCI image reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageRef {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest: Option<String>,  // SHA-256 digest
}

impl From<&str> for ImageRef {
    fn from(s: &str) -> Self {
        // Parse "docker.io/library/python:3.11"
        // or "gcr.io/project/image@sha256:abcd1234"
        todo!("Parse image reference")
    }
}

/// Agent identity (NHI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub public_key: Vec<u8>,
    pub algorithm: SignatureAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    Ed25519,
    MlDsa65,  // FIPS 204 ML-DSA
}

/// Runtime backend selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RuntimeClass {
    GVisor,
    Kata,
}

/// Resource limits (cgroups)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// CPU allocation in millicores (1000 = 1 CPU)
    pub cpu_millicores: u32,

    /// Memory limit in bytes
    pub memory_bytes: u64,

    /// Disk limit in bytes
    pub disk_bytes: u64,

    /// PID limit
    pub pid_limit: u32,

    /// Network bandwidth limit (bytes/sec)
    pub network_bandwidth_bps: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            cpu_millicores: 1000,  // 1 CPU
            memory_bytes: 2 * 1024 * 1024 * 1024,  // 2GB
            disk_bytes: 10 * 1024 * 1024 * 1024,  // 10GB
            pid_limit: 1024,
            network_bandwidth_bps: None,
        }
    }
}
```

---

### Network Policy Types

```rust
/// Network egress policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Default action for unmatched destinations
    pub default_action: NetworkAction,

    /// Egress rules (evaluated in order)
    pub egress_rules: Vec<EgressRule>,

    /// DNS policy
    pub dns_policy: Option<DnsPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkAction {
    /// Allow connection
    Allow,

    /// Deny connection
    Deny,

    /// Require Authorization service check
    RequireAuthz,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EgressRule {
    /// Destination matcher
    pub destination: EgressDestination,

    /// Action to take
    pub action: NetworkAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EgressDestination {
    /// CIDR block (e.g., "10.0.0.0/8")
    CidrBlock(ipnetwork::IpNetwork),

    /// Domain pattern (e.g., "*.anthropic.com")
    Domain(String),

    /// Exact domain match
    DomainExact(String),

    /// Kubernetes service reference
    ServiceRef { name: String, namespace: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsPolicy {
    /// Allowed DNS resolvers
    pub allowed_resolvers: Vec<std::net::IpAddr>,

    /// Blocked domain patterns
    pub blocked_domains: Vec<String>,

    /// Domains requiring AuthZ check
    pub require_authz_domains: Vec<String>,
}
```

---

### Secret Types

```rust
/// Secret reference in sandbox spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretRef {
    /// Secret name (for reference)
    pub name: String,

    /// Secret source
    pub source: SecretSource,

    /// How to mount the secret
    pub mount: SecretMount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretSource {
    /// NHI-delegated secret from delegation service
    NhiDelegated {
        secret_id: String,
        delegation_scope: DelegationScope,
    },

    /// Kubernetes secret (for compatibility)
    K8sSecret {
        name: String,
        key: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationScope {
    /// Resource the secret grants access to
    pub resource: String,

    /// Allowed actions
    pub actions: Vec<String>,

    /// Time-to-live
    pub ttl: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretMount {
    /// Environment variable
    EnvVar { name: String },

    /// File on filesystem
    File { path: String, mode: u32 },
}

/// Resolved secret (plaintext, in-memory only)
#[derive(Clone)]
pub struct ResolvedSecret {
    pub name: String,
    pub value: Vec<u8>,
    pub mount: SecretMount,
}

impl Drop for ResolvedSecret {
    fn drop(&mut self) {
        // Zero memory on drop
        for byte in &mut self.value {
            *byte = 0;
        }
    }
}
```

---

### Sandbox Handle and Status

```rust
/// Handle to running sandbox (returned from spawn/claim)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxHandle {
    pub id: SandboxId,
    pub agent_nhi: AgentIdentity,
    pub attestation: Attestation,
    pub status: SandboxStatus,
    pub created_at: Timestamp,
}

/// Sandbox status (lifecycle phase)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxStatus {
    Creating,
    Ready,
    Running { command: String },
    Checkpointing,
    Checkpointed { checkpoint_id: CheckpointId },
    Terminating,
    Terminated { reason: TerminationReason },
    Failed { error: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TerminationReason {
    UserRequested,
    TtlExpired,
    IdleTimeout,
    ResourceExhaustion,
    OomKilled,
    CrashLoopBackoff,
}

/// Detailed sandbox information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxInfo {
    pub id: SandboxId,
    pub spec: SandboxSpec,
    pub status: SandboxStatus,
    pub resource_usage: ResourceUsage,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_usage: f64,  // Percentage
    pub memory_bytes: u64,
    pub disk_bytes: u64,
    pub network_rx_bytes: u64,
    pub network_tx_bytes: u64,
}
```

---

### Warm Pool Types

```rust
/// Unique pool identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PoolId(String);

/// Warm pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarmPoolConfig {
    pub id: PoolId,

    /// Template for sandboxes in pool (no NHI binding)
    pub template: SandboxSpec,

    /// Minimum ready sandboxes
    pub min_ready: usize,

    /// Maximum ready sandboxes
    pub max_ready: usize,

    /// Maximum age before eviction
    pub max_age: Duration,

    /// Whether sandboxes are reusable
    pub reusable: bool,

    /// Auto-scaling configuration
    pub auto_scaling: Option<AutoScalingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoScalingConfig {
    pub enabled: bool,

    /// Headroom factor (e.g., 1.5 = 50% headroom)
    pub headroom_factor: f64,

    /// Scaling interval
    pub interval: Duration,
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStatistics {
    pub ready_count: usize,
    pub claimed_count: usize,
    pub warming_count: usize,

    /// Claims per minute (rolling average)
    pub claims_per_minute: f64,

    /// Claim latency statistics
    pub avg_claim_latency: Duration,
    pub p50_claim_latency: Duration,
    pub p99_claim_latency: Duration,

    /// Pool age
    pub oldest_sandbox_age: Duration,
}

/// Sandbox state in pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolSandboxState {
    Warming { started_at: Timestamp },
    Ready { ready_at: Timestamp },
    Claimed { claimed_at: Timestamp, agent_nhi: AgentIdentity },
    Evicting { started_at: Timestamp },
}
```

---

## State Management

### Pool State Storage (etcd)

```rust
use etcd_client::{Client as EtcdClient, GetOptions, PutOptions};

pub struct PoolStateStore {
    etcd: Arc<EtcdClient>,
}

impl PoolStateStore {
    /// Store pool configuration
    pub async fn store_pool_config(&self, config: &WarmPoolConfig) -> Result<(), Error> {
        let key = format!("/warmpool/{}/config", config.id.0);
        let value = serde_json::to_vec(config)?;

        self.etcd.put(key, value, None).await?;
        Ok(())
    }

    /// Get pool configuration
    pub async fn get_pool_config(&self, pool_id: &PoolId) -> Result<WarmPoolConfig, Error> {
        let key = format!("/warmpool/{}/config", pool_id.0);
        let resp = self.etcd.get(key, None).await?;

        let kv = resp.kvs().first().ok_or(Error::PoolNotFound)?;
        let config = serde_json::from_slice(kv.value())?;
        Ok(config)
    }

    /// Add sandbox to pool
    pub async fn add_sandbox(
        &self,
        pool_id: &PoolId,
        sandbox_id: &SandboxId,
        state: PoolSandboxState,
    ) -> Result<(), Error> {
        let key = format!("/warmpool/{}/sandboxes/{}", pool_id.0, sandbox_id.0);
        let value = serde_json::to_vec(&state)?;

        self.etcd.put(key, value, None).await?;
        Ok(())
    }

    /// Pop ready sandbox from pool (atomic)
    pub async fn pop_ready(&self, pool_id: &PoolId) -> Result<SandboxId, Error> {
        // Use etcd transaction for atomic pop
        let prefix = format!("/warmpool/{}/sandboxes/", pool_id.0);

        loop {
            // 1. List ready sandboxes
            let resp = self.etcd.get(prefix.clone(), Some(GetOptions::new().with_prefix())).await?;

            for kv in resp.kvs() {
                let state: PoolSandboxState = serde_json::from_slice(kv.value())?;

                if matches!(state, PoolSandboxState::Ready { .. }) {
                    let sandbox_id = SandboxId(kv.key_str()?.split('/').last().unwrap().to_string());

                    // 2. Try to claim it (CAS)
                    let new_state = PoolSandboxState::Claimed {
                        claimed_at: Timestamp::now(),
                        agent_nhi: AgentIdentity::default(),  // Updated by caller
                    };

                    let txn = self.etcd.txn()
                        .when(vec![Compare::value(kv.key(), CompareOp::Equal, kv.value())])
                        .and_then(vec![TxnOp::put(
                            kv.key(),
                            serde_json::to_vec(&new_state)?,
                            None,
                        )])
                        .or_else(vec![])
                        .await?;

                    if txn.succeeded() {
                        return Ok(sandbox_id);
                    }
                    // CAS failed, retry
                }
            }

            // No ready sandboxes found
            return Err(Error::NoReadySandboxes);
        }
    }

    /// Update pool statistics
    pub async fn update_stats(
        &self,
        pool_id: &PoolId,
        stats: &PoolStatistics,
    ) -> Result<(), Error> {
        let key = format!("/warmpool/{}/stats", pool_id.0);
        let value = serde_json::to_vec(stats)?;

        self.etcd.put(key, value, None).await?;
        Ok(())
    }
}
```

---

### Sandbox Registry (In-Memory + etcd)

```rust
use dashmap::DashMap;

pub struct SandboxRegistry {
    /// In-memory cache for fast lookups
    local_cache: Arc<DashMap<SandboxId, SandboxInfo>>,

    /// Durable storage (etcd)
    etcd: Arc<EtcdClient>,
}

impl SandboxRegistry {
    /// Register new sandbox
    pub async fn register(&self, info: SandboxInfo) -> Result<(), Error> {
        // 1. Store in etcd
        let key = format!("/sandboxes/{}", info.id.0);
        let value = serde_json::to_vec(&info)?;
        self.etcd.put(key, value, None).await?;

        // 2. Cache locally
        self.local_cache.insert(info.id.clone(), info);

        Ok(())
    }

    /// Get sandbox info
    pub async fn get(&self, id: &SandboxId) -> Result<SandboxInfo, Error> {
        // 1. Check local cache
        if let Some(info) = self.local_cache.get(id) {
            return Ok(info.clone());
        }

        // 2. Fetch from etcd
        let key = format!("/sandboxes/{}", id.0);
        let resp = self.etcd.get(key, None).await?;

        let kv = resp.kvs().first().ok_or(Error::SandboxNotFound)?;
        let info: SandboxInfo = serde_json::from_slice(kv.value())?;

        // 3. Update cache
        self.local_cache.insert(id.clone(), info.clone());

        Ok(info)
    }

    /// Update sandbox status
    pub async fn update_status(
        &self,
        id: &SandboxId,
        status: SandboxStatus,
    ) -> Result<(), Error> {
        let mut info = self.get(id).await?;
        info.status = status;
        info.updated_at = Timestamp::now();

        self.register(info).await
    }

    /// Remove sandbox
    pub async fn remove(&self, id: &SandboxId) -> Result<(), Error> {
        // 1. Remove from etcd
        let key = format!("/sandboxes/{}", id.0);
        self.etcd.delete(key, None).await?;

        // 2. Remove from cache
        self.local_cache.remove(id);

        Ok(())
    }

    /// List all sandboxes
    pub async fn list(&self) -> Result<Vec<SandboxInfo>, Error> {
        let resp = self.etcd.get("/sandboxes/", Some(GetOptions::new().with_prefix())).await?;

        let mut infos = Vec::new();
        for kv in resp.kvs() {
            let info: SandboxInfo = serde_json::from_slice(kv.value())?;
            infos.push(info);
        }

        Ok(infos)
    }
}
```

---

## Storage Strategy

### Checkpoint Storage (S3/Blob)

```rust
use aws_sdk_s3::Client as S3Client;

pub struct CheckpointStorage {
    s3: Arc<S3Client>,
    bucket: String,
}

impl CheckpointStorage {
    /// Upload checkpoint to durable storage
    pub async fn upload_checkpoint(
        &self,
        checkpoint_id: &CheckpointId,
        data: &CheckpointData,
    ) -> Result<(), Error> {
        // 1. Upload memory dump
        self.s3.put_object()
            .bucket(&self.bucket)
            .key(format!("checkpoints/{}/memory.img", checkpoint_id.0))
            .body(data.memory_dump.clone().into())
            .send()
            .await?;

        // 2. Upload CPU state
        self.s3.put_object()
            .bucket(&self.bucket)
            .key(format!("checkpoints/{}/cpu-state.json", checkpoint_id.0))
            .body(serde_json::to_vec(&data.cpu_state)?.into())
            .send()
            .await?;

        // 3. Upload filesystem delta
        self.s3.put_object()
            .bucket(&self.bucket)
            .key(format!("checkpoints/{}/fs-delta.tar.gz", checkpoint_id.0))
            .body(data.fs_delta.clone().into())
            .send()
            .await?;

        Ok(())
    }

    /// Upload checkpoint metadata
    pub async fn upload_metadata(
        &self,
        checkpoint_id: &CheckpointId,
        metadata: &CheckpointMetadata,
    ) -> Result<(), Error> {
        self.s3.put_object()
            .bucket(&self.bucket)
            .key(format!("checkpoints/{}/metadata.json", checkpoint_id.0))
            .body(serde_json::to_vec(metadata)?.into())
            .send()
            .await?;

        Ok(())
    }

    /// Download checkpoint from storage
    pub async fn download_checkpoint(
        &self,
        checkpoint_id: &CheckpointId,
    ) -> Result<CheckpointData, Error> {
        // 1. Download memory dump
        let memory_dump = self.s3.get_object()
            .bucket(&self.bucket)
            .key(format!("checkpoints/{}/memory.img", checkpoint_id.0))
            .send()
            .await?
            .body
            .collect()
            .await?
            .into_bytes()
            .to_vec();

        // 2. Download CPU state
        let cpu_state_bytes = self.s3.get_object()
            .bucket(&self.bucket)
            .key(format!("checkpoints/{}/cpu-state.json", checkpoint_id.0))
            .send()
            .await?
            .body
            .collect()
            .await?
            .into_bytes();

        let cpu_state = serde_json::from_slice(&cpu_state_bytes)?;

        // 3. Download filesystem delta
        let fs_delta = self.s3.get_object()
            .bucket(&self.bucket)
            .key(format!("checkpoints/{}/fs-delta.tar.gz", checkpoint_id.0))
            .send()
            .await?
            .body
            .collect()
            .await?
            .into_bytes()
            .to_vec();

        Ok(CheckpointData {
            memory_dump,
            cpu_state,
            fs_delta,
        })
    }
}
```

---

## Attestation Records

### Attestation Structure

```rust
use sha2::{Sha256, Digest};

/// Unique checkpoint identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CheckpointId(String);

/// Cryptographic attestation of sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attestation {
    /// Sandbox ID
    pub sandbox_id: SandboxId,

    /// Agent identity (bound to sandbox)
    pub agent_nhi: AgentIdentity,

    /// Delegation chain (agent -> ... -> human)
    pub delegation_chain: Vec<AgentIdentity>,

    /// Configuration hashes
    pub image_hash: Hash,       // SHA-256 of OCI image
    pub config_hash: Hash,      // SHA-256 of SandboxSpec
    pub init_hash: Hash,        // SHA-256 of initial filesystem

    /// Platform information
    pub platform: AttestationPlatform,
    pub platform_evidence: Vec<u8>,

    /// Temporal validity
    pub created_at: Timestamp,
    pub valid_until: Timestamp,

    /// Hybrid signature (Ed25519 + ML-DSA)
    pub signature: Signature,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttestationPlatform {
    GVisor { version: String },
    Kata { version: String, hypervisor: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hash([u8; 32]);  // SHA-256

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub ed25519: Vec<u8>,
    pub ml_dsa: Option<Vec<u8>>,  // ML-DSA-65 signature (post-quantum)
}

impl Attestation {
    /// Compute canonical byte representation for signing
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Deterministic serialization (no signature field)
        bytes.extend_from_slice(self.sandbox_id.0.as_bytes());
        bytes.extend_from_slice(&self.agent_nhi.public_key);
        // ... (serialize all fields except signature)

        bytes
    }

    /// Compute hash of attestation
    pub fn hash(&self) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(&self.canonical_bytes());
        Hash(hasher.finalize().into())
    }
}

/// Attestation verification result
#[derive(Debug, Clone)]
pub enum VerificationResult {
    Valid {
        agent_nhi: AgentIdentity,
        delegation_chain: Vec<AgentIdentity>,
        platform: AttestationPlatform,
    },
    Invalid { reason: String },
    Expired,
}
```

---

## Audit Events

### Event Types

```rust
/// Audit events logged to creto-audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    SandboxSpawned {
        sandbox_id: SandboxId,
        agent_nhi: AgentIdentity,
        attestation_hash: Hash,
        runtime_class: RuntimeClass,
    },

    SandboxTerminated {
        sandbox_id: SandboxId,
        reason: TerminationReason,
        duration: Duration,
    },

    PoolClaimed {
        pool_id: PoolId,
        sandbox_id: SandboxId,
        agent_nhi: AgentIdentity,
        claim_latency: Duration,
    },

    PoolReleased {
        pool_id: PoolId,
        sandbox_id: SandboxId,
        reusable: bool,
    },

    EgressAttempt {
        sandbox_id: SandboxId,
        destination: EgressDestination,
        decision: EgressDecision,
        latency: Duration,
    },

    SecretInjected {
        sandbox_id: SandboxId,
        secret_name: String,
        source: String,  // "nhi-delegated" or "k8s-secret"
    },

    AttestationGenerated {
        sandbox_id: SandboxId,
        attestation_hash: Hash,
    },

    SandboxCheckpointed {
        sandbox_id: SandboxId,
        checkpoint_id: CheckpointId,
        size_bytes: u64,
    },

    SandboxRestored {
        checkpoint_id: CheckpointId,
        sandbox_id: SandboxId,
        agent_nhi: AgentIdentity,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EgressDecision {
    Allow,
    Deny,
}
```

---

### Timestamp Type

```rust
use chrono::{DateTime, Utc};

/// Timestamp (UTC, millisecond precision)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    pub fn now() -> Self {
        Self(Utc::now())
    }

    pub fn unix_millis(&self) -> i64 {
        self.0.timestamp_millis()
    }
}

impl std::ops::Add<Duration> for Timestamp {
    type Output = Self;

    fn add(self, duration: Duration) -> Self {
        Self(self.0 + chrono::Duration::from_std(duration).unwrap())
    }
}
```

---

## Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2025-12-25 | 0.1 | Claude | Initial data design |
