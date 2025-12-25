# Runtime Product - Component Diagram

## Overview

This component diagram details the internal architecture of the Runtime product, which provides secure sandboxed execution environments for AI agents. It shows the warm pool lifecycle, runtime adapter abstraction for multiple backend technologies (gVisor, Kata Containers), and egress control mechanisms.

## Purpose

- Detail components within the Runtime product container
- Show warm pool lifecycle: Idle → Claimed → Running → Recycled/Destroyed
- Illustrate runtime adapter abstraction layer (gVisor, Kata, Firecracker)
- Visualize egress control and network policy enforcement
- Document sandbox resource management and quota enforcement

## Diagram

```mermaid
graph TB
    subgraph External["External Systems"]
        Agent[AI Agent]
        ExternalAPI[External APIs<br/>LLMs, Databases, etc.]
        Audit[Audit Logger]
    end

    subgraph APIGateway["API Gateway Layer"]
        LB[Load Balancer]
        Gateway[API Gateway<br/>Kong/Envoy]
    end

    subgraph RuntimeProduct["Runtime Product Container"]

        subgraph SandboxManagement["Sandbox Management Layer"]
            SandboxAPI[Sandbox Controller API<br/>gRPC Service]
            LifecycleManager[Lifecycle Manager<br/>Create/Start/Stop/Destroy]
            ResourceManager[Resource Manager<br/>CPU/Memory/Disk Quotas]
            MetricsCollector[Metrics Collector<br/>Resource Usage Tracking]
        end

        subgraph WarmPool["Warm Pool Layer"]
            WarmPoolAPI[Warm Pool Manager<br/>Background Service]
            PoolOptimizer[Pool Optimizer<br/>Auto-Scaling Logic]
            StateTracker[State Tracker<br/>Idle/Claimed/Running]
            EvictionPolicy[Eviction Policy<br/>LRU + Age-based]
        end

        subgraph RuntimeAbstraction["Runtime Adapter Layer"]
            AdapterAPI[Runtime Adapter API<br/>gRPC Service]
            GVisorAdapter[gVisor Adapter<br/>runsc Integration]
            KataAdapter[Kata Adapter<br/>kata-runtime Integration]
            FirecrackerAdapter[Firecracker Adapter<br/>firecracker-containerd]
            BackendSelector[Backend Selector<br/>Agent-based Routing]
        end

        subgraph NetworkControl["Egress Control Layer"]
            EgressAPI[Egress Controller API<br/>gRPC Service]
            AllowlistEngine[Allowlist Engine<br/>Domain/IP Filtering]
            NetworkPolicy[Network Policy Manager<br/>iptables/eBPF Rules]
            TrafficMonitor[Traffic Monitor<br/>Bandwidth Tracking]
        end

        subgraph ImageManagement["Image & Storage Layer"]
            ImageRegistry[Image Registry<br/>OCI Image Storage]
            ImageBuilder[Image Builder<br/>Dockerfile → OCI Image]
            SnapshotService[Snapshot Service<br/>Pre-warmed Images]
            VolumeManager[Volume Manager<br/>Persistent Storage]
        end

        subgraph DataStores["Data Stores"]
            PostgresDB[(PostgreSQL<br/>runtime_sandboxes<br/>runtime_images)]
            RedisCache[(Redis Cluster<br/>Warm Pool Inventory<br/>Resource Quotas)]
            S3Bucket[(S3 Bucket<br/>Sandbox Images<br/>Logs)]
        end
    end

    %% External → API Gateway
    Agent -->|POST /runtime/sandboxes<br/>Create Sandbox| LB
    Agent -->|POST /runtime/sandboxes/{id}/execute<br/>Run Code| LB
    Agent -->|DELETE /runtime/sandboxes/{id}<br/>Destroy Sandbox| LB
    LB -->|Route| Gateway

    %% API Gateway → Runtime Components
    Gateway -->|gRPC: CreateSandbox| SandboxAPI
    Gateway -->|gRPC: ExecuteCode| SandboxAPI
    Gateway -->|gRPC: DestroySandbox| SandboxAPI
    Gateway -->|gRPC: ConfigureEgress| EgressAPI

    %% Sandbox Creation Flow
    SandboxAPI -->|1. Check Quota| ResourceManager
    ResourceManager -->|Query Quota| RedisCache
    ResourceManager -->|Quota OK| WarmPoolAPI

    WarmPoolAPI -->|2. Claim Warm Sandbox| StateTracker
    StateTracker -->|Check Inventory| RedisCache
    StateTracker -->|Available → Claimed| LifecycleManager
    StateTracker -->|None Available → Create New| AdapterAPI

    %% Runtime Adapter Flow
    AdapterAPI -->|3. Select Backend| BackendSelector
    BackendSelector -->|Route by Agent Tier| GVisorAdapter
    BackendSelector -->|Route by Agent Tier| KataAdapter
    BackendSelector -->|Route by Agent Tier| FirecrackerAdapter

    GVisorAdapter -->|Load Image| ImageRegistry
    KataAdapter -->|Load Image| ImageRegistry
    FirecrackerAdapter -->|Load Snapshot| SnapshotService

    ImageRegistry -->|Pull OCI Image| S3Bucket
    SnapshotService -->|Load Pre-warmed Image| S3Bucket

    GVisorAdapter -->|Create runsc Container| LifecycleManager
    KataAdapter -->|Create Kata VM| LifecycleManager
    FirecrackerAdapter -->|Create microVM| LifecycleManager

    %% Lifecycle Management
    LifecycleManager -->|4. Start Sandbox| AdapterAPI
    LifecycleManager -->|5. Configure Network| EgressAPI
    LifecycleManager -->|6. Track State| StateTracker
    LifecycleManager -->|7. Persist Metadata| PostgresDB

    %% Egress Control
    EgressAPI -->|Load Allowlist| AllowlistEngine
    AllowlistEngine -->|Query Policies| PostgresDB
    AllowlistEngine -->|Generate Rules| NetworkPolicy
    NetworkPolicy -->|Apply iptables| AdapterAPI
    TrafficMonitor -->|Track Bandwidth| MetricsCollector

    %% Code Execution
    SandboxAPI -->|Execute in Sandbox| AdapterAPI
    AdapterAPI -->|Run via Backend| GVisorAdapter
    GVisorAdapter -->|Enforce Resource Limits| ResourceManager
    ResourceManager -->|Monitor CPU/Memory| MetricsCollector
    MetricsCollector -->|Report Usage| PostgresDB
    MetricsCollector -.->|Send Events to Metering| Gateway

    %% Warm Pool Management
    WarmPoolAPI -->|Background: Pre-warm| PoolOptimizer
    PoolOptimizer -->|Calculate Pool Size| StateTracker
    PoolOptimizer -->|Create Sandboxes| AdapterAPI
    StateTracker -->|Update Inventory| RedisCache

    EvictionPolicy -->|Evict Idle Sandboxes| LifecycleManager
    LifecycleManager -->|Destroy Evicted| AdapterAPI

    %% Recycling
    SandboxAPI -->|On Completion| LifecycleManager
    LifecycleManager -->|Reset State| AdapterAPI
    AdapterAPI -->|Recycle to Pool| StateTracker
    StateTracker -->|Mark as Idle| RedisCache

    %% Image Building
    ImageBuilder -->|Build from Dockerfile| ImageRegistry
    ImageRegistry -->|Push to S3| S3Bucket
    SnapshotService -->|Pre-warm Image| ImageBuilder

    %% Volume Management
    VolumeManager -->|Attach Persistent Volume| AdapterAPI
    VolumeManager -->|Snapshot Volume| S3Bucket

    %% Audit Logging
    SandboxAPI -.->|Log: Sandbox Created| Audit
    LifecycleManager -.->|Log: State Transition| Audit
    EgressAPI -.->|Log: Egress Rule Applied| Audit
    MetricsCollector -.->|Log: Resource Violation| Audit

    %% Cross-Product Integration
    ResourceManager -.->|Query Metering Quotas| Gateway
    SandboxAPI -.->|Check Oversight Approval| Gateway
    MetricsCollector -.->|Report Usage to Metering| Gateway

    classDef api fill:#e1f5ff,stroke:#0066cc,stroke-width:2px
    classDef cache fill:#fff4e1,stroke:#ff9900,stroke-width:2px
    classDef db fill:#f0f0f0,stroke:#333,stroke-width:2px
    classDef processor fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px
    classDef backend fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px

    class SandboxAPI,EgressAPI api
    class RedisCache cache
    class PostgresDB,S3Bucket db
    class LifecycleManager,PoolOptimizer processor
    class GVisorAdapter,KataAdapter,FirecrackerAdapter backend
```

## Component Inventory

### Sandbox Management Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Sandbox Controller API** | Go/gRPC | Orchestrates sandbox lifecycle, handles client requests |
| **Lifecycle Manager** | Go/State Machine | Manages create/start/stop/destroy transitions |
| **Resource Manager** | Go/cgroups | Enforces CPU, memory, disk quotas using cgroups v2 |
| **Metrics Collector** | Prometheus Client | Tracks resource usage, reports to Metering product |

**Lifecycle States:**
```
IDLE → CLAIMED → STARTING → RUNNING → STOPPING → STOPPED → RECYCLING → IDLE
                                                           → DESTROYING → DESTROYED
```

### Warm Pool Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Warm Pool Manager** | Go/Background Service | Maintains pool of pre-warmed sandboxes for fast startup |
| **Pool Optimizer** | Go/Auto-Scaler | Calculates optimal pool size based on demand patterns |
| **State Tracker** | Go/Redis Client | Tracks sandbox states (Idle/Claimed/Running) in Redis |
| **Eviction Policy** | LRU + Age-based | Evicts idle sandboxes after 5 minutes or on memory pressure |

**Pool Sizing Algorithm:**
```
target_pool_size = max(
  baseline_pool_size,  // e.g., 10 sandboxes
  avg_requests_per_minute * 0.8,  // 80% of average load
  p95_requests_per_minute * 0.5   // 50% of peak load
)
```

### Runtime Adapter Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Runtime Adapter API** | Go/gRPC | Abstraction layer over multiple sandbox backends |
| **gVisor Adapter** | runsc (Go) | Lightweight container sandboxing using gVisor |
| **Kata Adapter** | kata-runtime (Rust) | VM-based isolation using Kata Containers |
| **Firecracker Adapter** | firecracker (Rust) | microVM isolation using AWS Firecracker |
| **Backend Selector** | Rule-based Router | Routes agents to appropriate backend based on tier |

**Backend Selection Logic:**
```
if agent.tier == "basic":
    backend = gVisor  // Lightweight, <50ms startup
elif agent.tier == "standard":
    backend = Kata    // Strong isolation, <200ms startup
elif agent.tier == "premium":
    backend = Firecracker  // Ultra-secure, <150ms startup
```

### Egress Control Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Egress Controller API** | Go/gRPC | Configures network egress policies for sandboxes |
| **Allowlist Engine** | Go/Domain Matcher | Filters allowed domains/IPs using regex + CIDR matching |
| **Network Policy Manager** | iptables/eBPF | Applies network rules using iptables or eBPF programs |
| **Traffic Monitor** | Go/Prometheus | Tracks egress bandwidth, reports to Metering |

**Allowlist Example:**
```yaml
egress_policy:
  allow:
    - "*.openai.com"          # OpenAI API
    - "*.anthropic.com"       # Anthropic API
    - "10.0.0.0/8"            # Private network
  deny:
    - "*"                     # Default deny all
  rate_limit:
    bandwidth: 100MB/s        # Max egress bandwidth
```

### Image & Storage Layer

| Component | Technology | Responsibilities |
|-----------|-----------|------------------|
| **Image Registry** | OCI Distribution | Stores sandbox images in OCI format |
| **Image Builder** | Dockerfile → OCI | Builds custom sandbox images from Dockerfiles |
| **Snapshot Service** | S3-backed Storage | Stores pre-warmed snapshots for fast cloning |
| **Volume Manager** | CSI Driver | Manages persistent volumes attached to sandboxes |

**Image Layers:**
```
Base Layer: Ubuntu 22.04 (500 MB)
Runtime Layer: Python 3.11 + Node.js 20 (200 MB)
Agent Layer: Custom agent code (50 MB)
Total: 750 MB (compressed: 300 MB)
```

## Data Flow Details

### Flow 1: Create Sandbox (Warm Pool Hit)

```
Agent → API Gateway → Sandbox Controller API
  ↓
1. Validate delegation chain (AuthZ)
2. Check quota (Resource Manager → Redis)
3. Query warm pool (Warm Pool Manager → Redis)
  ↓
Warm Pool Manager:
  - Find idle sandbox matching agent requirements
  - Claim sandbox: State = IDLE → CLAIMED
  - Return sandbox_id
  ↓
Lifecycle Manager:
  - Configure sandbox for agent (inject code, env vars)
  - Apply egress policy (Egress Controller)
  - Transition state: CLAIMED → STARTING → RUNNING
  ↓
Persist to PostgreSQL:
  INSERT INTO runtime_sandboxes (sandbox_id, agent_id, state, backend, created_at)
  ↓
Return sandbox_id to agent (total time: <50ms)
```

### Flow 2: Create Sandbox (Cold Start)

```
Agent → Sandbox Controller API
  ↓
1. Check quota
2. Query warm pool → No idle sandboxes available
  ↓
Backend Selector:
  - Determine backend: gVisor (basic), Kata (standard), Firecracker (premium)
  ↓
Runtime Adapter (e.g., gVisor):
  1. Load image from S3: s3://sandbox-images/python-3.11.tar.gz
  2. Extract image layers to local cache
  3. Create runsc container:
     runsc create --bundle /var/lib/runsc/sandbox_xyz789 \
                  --config config.json \
                  sandbox_xyz789
  4. Configure cgroups: CPU (2 cores), Memory (4 GB), Disk (10 GB)
  5. Apply network policy: iptables -A OUTPUT -d openai.com -j ACCEPT
  ↓
Lifecycle Manager:
  - Start container: runsc start sandbox_xyz789
  - Transition state: STARTING → RUNNING
  ↓
Return sandbox_id to agent (total time: ~2-5s cold start)
```

### Flow 3: Execute Code in Sandbox

```
Agent → Sandbox Controller API → Runtime Adapter
  ↓
1. Validate sandbox ownership (agent_id matches)
2. Check sandbox state (must be RUNNING)
  ↓
Runtime Adapter (gVisor example):
  - Execute code via runsc:
    runsc exec sandbox_xyz789 /bin/python3 /tmp/agent_code.py
  - Stream stdout/stderr back to agent
  - Monitor resource usage (CPU, memory)
  ↓
Metrics Collector:
  - Track CPU seconds: 1.2s
  - Track memory peak: 500 MB
  - Report to Metering:
    POST /metering/events {
      resource_type: "sandbox_cpu_seconds",
      quantity: 1.2,
      agent_id: "agent_abc123"
    }
  ↓
Return execution result to agent
```

### Flow 4: Warm Pool Recycling

```
Agent completes sandbox execution
  ↓
Lifecycle Manager:
  - Stop sandbox (graceful shutdown)
  - Reset state: Clear agent code, env vars, network rules
  - Transition: RUNNING → STOPPING → STOPPED → RECYCLING
  ↓
Runtime Adapter:
  - Clean sandbox filesystem: rm -rf /tmp/*
  - Reset cgroups limits
  - Flush network policies
  ↓
State Tracker:
  - Mark sandbox as IDLE in Redis
  - Add to warm pool inventory
  ↓
Eviction Policy (background job):
  - Check idle time: If > 5 minutes, destroy sandbox
  - Check memory pressure: If pool > 100 sandboxes, evict LRU
  ↓
If evicted:
  - Lifecycle Manager → Destroy sandbox
  - Remove from Redis inventory
```

### Flow 5: Egress Control

```
Egress Controller API receives policy configuration
  ↓
1. Parse allowlist rules (domains, IPs, CIDR blocks)
2. Resolve domains to IP addresses (DNS lookup)
3. Generate iptables rules:
   iptables -A OUTPUT -d 13.224.0.0/14 -j ACCEPT  # openai.com CIDR
   iptables -A OUTPUT -d 0.0.0.0/0 -j DROP        # Deny all else
4. Apply rules to sandbox network namespace:
   nsenter --net=/var/run/netns/sandbox_xyz789 iptables-restore < rules.txt
  ↓
Traffic Monitor:
  - Monitor egress bytes using iptables counters
  - Report to Metering every 60 seconds:
    POST /metering/events {
      resource_type: "sandbox_egress_bytes",
      quantity: 52428800,  // 50 MB
      agent_id: "agent_abc123"
    }
```

## Implementation Considerations

### Performance Optimization

**Warm Pool Benefits:**
- **Startup time**: 50ms (warm) vs. 2-5s (cold)
- **Cost**: Pre-warmed sandboxes cost ~$0.001/hour idle
- **Hit rate**: Target 90% warm pool hit rate during business hours

**Image Caching:**
- Local cache: SSD-backed `/var/lib/runtime/cache` (1 TB)
- Eviction: LRU, keep 20 most popular images
- Pre-pull: Daily cron job pulls top 10 images

**Resource Limits (cgroups v2):**
```
CPU: 2 cores (200% CPU shares)
Memory: 4 GB hard limit, 3 GB soft limit (OOM kill at 4 GB)
Disk I/O: 100 MB/s read, 50 MB/s write (blkio throttling)
Network: 100 MB/s egress (tc qdisc)
```

### Scalability

**Horizontal Scaling:**
- Sandbox Controller API: 12 replicas (HPA on sandbox count)
- Runtime Adapter: 8 nodes (Kubernetes node pool, 100 sandboxes/node)
- Warm Pool Manager: Single leader (leader election)

**Vertical Scaling:**
- Node size: c6i.4xlarge (16 vCPU, 32 GB RAM)
- Max sandboxes per node: 100 (assuming 2 cores, 4 GB per sandbox)
- Cluster capacity: 800 sandboxes (8 nodes × 100)

**Auto-Scaling Triggers:**
- Warm pool depletion: If idle count < 10, scale up nodes
- High CPU: If node CPU > 80%, scale up
- Low utilization: If node CPU < 20% for 10 min, scale down

### Resilience

**Sandbox Crash Recovery:**
- Health checks: Every 30 seconds, ping sandbox via gRPC
- On crash: Transition to FAILED state, notify agent
- Auto-destroy: Failed sandboxes destroyed after 5 minutes

**Node Failure:**
- Sandboxes on failed node marked LOST
- Warm pool replenished on remaining nodes
- Agents notified to retry sandbox creation

**Backend Fallback:**
- If gVisor fails, fallback to Kata
- If Kata unavailable, queue request for retry
- Circuit breaker: Stop routing to failed backend for 5 minutes

### Security

**Isolation Levels:**
- **gVisor**: Application kernel in userspace (ptrace or KVM)
- **Kata**: Full VM isolation with lightweight kernel
- **Firecracker**: microVM with minimized attack surface

**Network Policies:**
- Default deny all egress
- Allowlist enforced at kernel level (iptables/eBPF)
- DNS resolution blocked (prevent DNS tunneling)

**Image Security:**
- Images scanned for vulnerabilities (Trivy)
- Base images pinned to specific SHAs (no `latest` tag)
- Signed images using Notary (Docker Content Trust)

**AuthZ Integration:**
- Every sandbox creation validates delegation chain
- Agent identity embedded in sandbox metadata
- Audit trail records all sandbox lifecycle events

### Monitoring & Alerting

**Key Metrics:**
- `runtime_sandboxes_total{state="running"}` (gauge)
- `runtime_warmpool_size{state="idle"}` (gauge)
- `runtime_sandbox_create_duration_seconds` (histogram)
- `runtime_sandbox_startup_type{type="warm|cold"}` (counter)
- `runtime_egress_bytes_total` (counter)

**Alerts:**
- Warm pool depleted: Page on-call
- Cold start p99 > 10s: Warning
- Sandbox creation error rate > 5%: Critical
- Node resource exhaustion: Page on-call

## Integration Points

### Cross-Product Dependencies

**Runtime → Metering:**
- **Resource usage reporting**: CPU seconds, memory GB-hours, egress bytes
- **Quota checks**: Before creating sandbox, check Metering quota

**Runtime → Oversight:**
- **Approval gates**: High-risk sandboxes require Oversight approval
- **Unblock execution**: On approval, proceed with sandbox creation

**Runtime → Messaging:**
- **Sandbox-to-sandbox communication**: Use Messaging for encrypted IPC (future)

### External Dependencies

**Image Registry (S3):**
- Stores OCI images and snapshots
- Lifecycle policy: Delete images unused for 90 days
- Versioning: Enabled for rollback capability

**Metrics Backend (Prometheus):**
- Scrapes `/metrics` endpoint every 15 seconds
- Retention: 15 days local, 1 year remote (Thanos)

## Related Diagrams

- [C4 Context Diagram](./c4-context.md) - System-level context
- [C4 Container Diagram](./c4-container.md) - Container-level architecture
- [Data Flow Matrix](./data-flow-matrix.md) - Cross-product data flows

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-12-25 | Initial Runtime component diagram for Issue #63 |
