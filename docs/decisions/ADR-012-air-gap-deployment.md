---
status: accepted
date: 2025-12-25
deciders: [Architecture Team, Security Team, Infrastructure Team, Compliance Team]
consulted: [Operations Team, Federal Compliance Office, CISO]
informed: [Product Management, Customer Success, Engineering]
---

# ADR-012: Air-Gapped Deployment Patterns

## Context

Certain deployments of the Enablement Layer must operate in fully air-gapped environments with **zero external network connectivity**. These deployments serve some of our most security-sensitive customers and use cases.

### Target Environments

1. **Classified Government (IL5/IL6)**
   - Sensitive Compartmented Information Facilities (SCIFs)
   - SIPRNet and higher classification networks
   - DoD classified research facilities
   - Intelligence community processing centers

2. **Financial Services**
   - Trading floor isolated networks (compliance requirement)
   - Payment processing enclaves (PCI-DSS Level 1)
   - Central bank settlement systems
   - Cryptocurrency custody solutions

3. **Healthcare (HIPAA Isolated)**
   - Research hospitals with patient data
   - Clinical trial data processing
   - Genomics research facilities
   - Mental health records systems

4. **Critical Infrastructure**
   - Power grid SCADA systems
   - Nuclear facility control systems
   - Water treatment facilities
   - Defense contractor classified networks

### Air-Gap Requirements

Air-gapped operation imposes absolute constraints:

1. **Zero Egress**: No packets may leave the environment under any circumstances
2. **Local AI Inference**: No calls to cloud LLM providers (OpenAI, Anthropic, Google)
3. **Local Cryptographic Operations**: No external HSM/KMS services
4. **Local Identity Resolution**: No cloud-based identity providers
5. **Offline Software Updates**: All updates via physical media or data diodes
6. **Local Audit Storage**: All logs must remain in the enclave
7. **Self-Contained Monitoring**: No cloud telemetry or metrics exporters

### Compliance Drivers

- **FedRAMP High**: SC-7 (Boundary Protection), SC-8 (Transmission Confidentiality)
- **NIST 800-53**: Complete control family for classified systems
- **IL5/IL6**: DoD Impact Level requirements for CUI and classified data
- **HIPAA**: Technical safeguards for isolated PHI environments
- **PCI-DSS**: Network segmentation for cardholder data environments

## Decision Drivers

### Functional Requirements
- **Feature Parity**: All core Enablement Layer capabilities must work offline
- **Performance**: Inference latency comparable to cloud deployment
- **Scalability**: Support 100+ concurrent agents in isolated environment
- **Reliability**: 99.9% uptime without cloud fallback

### Security Requirements
- **Zero Trust**: Assume network compromise, verify everything
- **Tamper Evidence**: Cryptographic proof of audit integrity
- **Key Management**: Secure key lifecycle without cloud HSM
- **Updates**: Verifiable software provenance for offline updates

### Operational Requirements
- **Maintainability**: Updates and patches without internet access
- **Observability**: Complete monitoring within enclave
- **Disaster Recovery**: Self-contained backup and restore
- **Documentation**: Comprehensive offline operational procedures

## Considered Options

### Option A: Stripped-Down Deployment

**Description**: Remove all features requiring external connectivity.

**Pros**:
- Simplest implementation
- Minimal attack surface
- Easy to certify for high-security environments

**Cons**:
- Significant feature gap vs. cloud deployment
- No advanced LLM capabilities (severely limited agent intelligence)
- Poor customer experience
- Requires maintaining separate codebase

**Verdict**: **REJECTED** - Unacceptable functional limitations

### Option B: Proxy/Gateway Pattern

**Description**: Route all external calls through a controlled proxy that can be disconnected.

**Implementation**:
```yaml
architecture:
  enablement_layer: [internal network]
  proxy_gateway: [DMZ]
  external_services: [internet]

  connectivity_modes:
    - connected: all features available
    - disconnected: fallback to degraded mode
```

**Pros**:
- Gradual transition to air-gap
- Flexible connectivity modes
- Easier to implement initially

**Cons**:
- Doesn't satisfy true air-gap requirement (SCIF deployments)
- Introduces single point of failure
- Proxy becomes high-value target
- Complexity of dual-mode operation

**Verdict**: **REJECTED** - Doesn't meet zero-egress requirement for highest security environments

### Option C: Full Local Stack (CHOSEN)

**Description**: Deploy complete, self-contained stack with local LLM inference, cryptography, identity, and audit.

**Components**:
1. Local LLM inference cluster (vLLM + open models)
2. Software HSM for cryptographic operations
3. Local NHI service with pre-provisioned identities
4. Local audit with Merkle tree tamper evidence
5. Offline update mechanism (physical media or one-way data diode)

**Pros**:
- True zero-egress capability
- Full feature parity with cloud (minus provider variety)
- Meets highest compliance requirements (IL6, FedRAMP High)
- Self-contained disaster recovery
- No dependency on external services

**Cons**:
- High capital cost (~$1M+ for inference cluster)
- Operational complexity (manual updates)
- Model capability gap vs. GPT-4o/Claude
- No automatic model updates
- Requires specialized expertise to operate

**Verdict**: **ACCEPTED** - Only option meeting all requirements

## Decision

We will implement **Option C: Full Local Stack** with the following architecture.

## Architecture

### High-Level Stack

```
┌─────────────────────────────────────────────────────────────────────┐
│                       AIR-GAPPED ENCLAVE                            │
│                                                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                    ENABLEMENT LAYER                           │  │
│  │                                                               │  │
│  │  ┌──────────┐  ┌──────────┐  ┌─────────┐  ┌──────────────┐  │  │
│  │  │ Metering │  │Oversight │  │ Runtime │  │  Messaging   │  │  │
│  │  └──────────┘  └──────────┘  └─────────┘  └──────────────┘  │  │
│  │                                                               │  │
│  │  ┌──────────────────────────────────────────────────────────┐ │  │
│  │  │         Local Provider Adapters (vLLM client)           │ │  │
│  │  └──────────────────────────────────────────────────────────┘ │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                               │                                     │
│                               │ gRPC/HTTP (local only)              │
│                               ▼                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │              LOCAL INFERENCE CLUSTER (vLLM)                   │  │
│  │                                                               │  │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │  │
│  │   │  Node 1     │  │  Node 2     │  │  Node 3     │  ...    │  │
│  │   │  8×H100 80GB│  │  8×H100 80GB│  │  8×H100 80GB│         │  │
│  │   │  512GB RAM  │  │  512GB RAM  │  │  512GB RAM  │         │  │
│  │   └─────────────┘  └─────────────┘  └─────────────┘         │  │
│  │                                                               │  │
│  │   Models: Llama 3.1 70B, Qwen2.5 72B, CodeLlama 70B          │  │
│  │   Tensor Parallel: 8-way, Pipeline Parallel: Optional        │  │
│  │   Max Context: 128K tokens, Batch Size: 256                  │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                               │                                     │
│                               │                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │              LOCAL PLATFORM SERVICES                          │  │
│  │                                                               │  │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐   │  │
│  │  │     NHI      │  │   Crypto     │  │      Audit       │   │  │
│  │  │   (local)    │  │  (SoftHSM)   │  │  (PostgreSQL)    │   │  │
│  │  │              │  │              │  │                  │   │  │
│  │  │ • Pre-prov   │  │ • AES-256-GCM│  │ • WAL archiving  │   │  │
│  │  │   identities │  │ • ML-KEM-768 │  │ • Merkle trees   │   │  │
│  │  │ • Offline    │  │ • Ed25519    │  │ • Tamper detect  │   │  │
│  │  │   cert chain │  │ • ML-DSA-65  │  │ • Local only     │   │  │
│  │  └──────────────┘  └──────────────┘  └──────────────────┘   │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                               │                                     │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                     STORAGE LAYER                             │  │
│  │                                                               │  │
│  │  • Model Weights: 2TB NVMe per node (local cache)            │  │
│  │  • Shared Storage: 100TB NFS (NetApp/Pure)                   │  │
│  │  • Audit Logs: PostgreSQL with pg_repack                     │  │
│  │  • Checkpoints: Distributed across nodes                     │  │
│  │  • Backups: Daily to encrypted cold storage                  │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                     │
│   ═══════════════════════════════════════════════════════════════  │
│                          NETWORK AIR GAP                            │
│          (Physical isolation, no cables to external networks)       │
│   ═══════════════════════════════════════════════════════════════  │
└─────────────────────────────────────────────────────────────────────┘
         │
         │ Data Diode (one-way, write-only) OR Sneakernet (USB/DVD)
         ↓
    ┌──────────────────────────┐
    │   Update Staging Zone    │
    │                          │
    │  • Signature verification│
    │  • Malware scanning      │
    │  • Dual-person approval  │
    │  • Audit trail           │
    └──────────────────────────┘
```

### Component Specifications

#### 1. Local Inference Cluster

**Configuration**:
```yaml
inference:
  mode: air_gapped

  server:
    type: vllm
    version: "0.4.2"
    tensor_parallel_size: 8
    pipeline_parallel_size: 1  # Optional for larger models
    max_model_len: 131072      # 128K context
    gpu_memory_utilization: 0.90
    swap_space: 4              # GiB

  models:
    - id: llama-3.1-70b-instruct
      path: /models/llama-3.1-70b-instruct
      default: true
      quantization: null       # FP16 for quality
      max_concurrent: 32

    - id: qwen2.5-72b-instruct
      path: /models/qwen2.5-72b-instruct
      quantization: null
      max_concurrent: 32

    - id: codellama-70b-instruct
      path: /models/codellama-70b-instruct
      quantization: null
      max_concurrent: 16
      specialization: code_generation

  hardware:
    interconnect: nvlink        # 900GB/s inter-GPU
    storage: nvme_local         # 14GB/s read
    network: infiniband_400g    # Low-latency node-to-node
```

**Performance Targets**:
- Time to First Token (TTFT): <200ms
- Tokens Per Second (TPS): >50 tok/s per request
- Throughput: 5000+ tok/s aggregate
- Concurrent Requests: 100+

#### 2. Software HSM (Cryptography)

**Configuration**:
```yaml
crypto:
  hsm_mode: software
  implementation: softhsm2

  key_storage:
    path: /etc/creto/keys
    encryption: aes-256-gcm     # At-rest encryption
    keyfile: /etc/creto/master.key  # Protected by TPM or HSM

  key_rotation:
    mode: manual                # No automatic online rotation
    schedule: annual
    overlap_period: 30d         # Old + new keys valid
    ceremony: dual_person       # Two operators required

  algorithms:
    symmetric: aes-256-gcm
    kem: ml-kem-768             # Post-quantum key exchange
    signature:
      - ed25519                 # Classical
      - ml-dsa-65              # Post-quantum
    hash: sha3-256

  fips_mode: true               # FIPS 140-2 validation

  backup:
    encrypted: true
    split: shamir_3_of_5        # 5 shares, 3 required
    offsite: cold_storage
```

**Key Management Procedures**:
1. **Initial Provisioning**: Dual-person key generation ceremony, documented
2. **Annual Rotation**: Scheduled maintenance window, overlap period
3. **Emergency Revocation**: Out-of-band communication, new key ceremony
4. **Backup Recovery**: Quorum of keyholders, documented recovery

#### 3. Local NHI Service

**Configuration**:
```yaml
nhi:
  mode: air_gapped

  identity_store:
    type: sqlite
    path: /var/lib/creto/nhi.db
    encryption: sqlcipher

  pre_provisioned:
    # Agent identities created during deployment
    - id: agent://enablement/code-reviewer
      public_key: ed25519:AAAA...
      delegation_depth: 3
      valid_until: 2026-12-31T23:59:59Z

    - id: agent://enablement/data-processor
      public_key: ed25519:BBBB...
      delegation_depth: 2
      valid_until: 2026-12-31T23:59:59Z

  certificate_chain:
    root_ca: /etc/creto/ca/root.crt
    intermediate_ca: /etc/creto/ca/intermediate.crt
    auto_renewal: false         # Manual renewal only

  delegation:
    max_depth: 5
    verification: offline       # No OCSP/CRL checks

  attestation:
    tpm_required: true          # Hardware root of trust
    secure_boot: enforced
```

**Identity Lifecycle**:
1. **Provisioning**: During initial deployment, signed by root CA
2. **Delegation**: Agents can delegate to sub-agents (up to max depth)
3. **Renewal**: Annual certificate renewal, manual process
4. **Revocation**: Manual revocation list, distributed via updates

#### 4. Local Audit & Logging

**Configuration**:
```yaml
audit:
  mode: air_gapped

  storage:
    backend: postgresql
    database: creto_audit
    retention: 7_years          # Compliance requirement

  wal_archiving:
    enabled: true
    archive_command: 'cp %p /mnt/audit/wal/%f'

  tamper_evidence:
    merkle_tree: true
    hash_algorithm: sha3-256
    anchor_interval: 1h         # Create new Merkle root hourly
    anchor_storage: /var/lib/creto/merkle_roots/

  log_levels:
    metering: debug             # All token usage
    oversight: info             # All interventions
    runtime: info               # All agent executions
    crypto: info                # All key operations

  encryption:
    at_rest: true
    algorithm: aes-256-gcm
    key_rotation: quarterly

  export:
    schedule: weekly
    format: encrypted_parquet
    destination: /mnt/export/
    verification: sha256_checksum
```

**Tamper Evidence**:
- Merkle tree computed hourly over all audit entries
- Merkle roots stored in append-only log
- Verification tool compares current database against Merkle roots
- Any tampering detected immediately via root mismatch

#### 5. Offline Update Mechanism

**Process**:
```yaml
update:
  delivery_method: [data_diode, sneakernet]

  bundle_format:
    container: tar.gz.gpg
    signature: ed25519 + ml-dsa-65  # Dual signatures
    manifest: sha256sums.txt

  verification_steps:
    - gpg_signature_check
    - checksum_verification
    - malware_scan              # ClamAV or commercial)
    - dual_person_approval
    - audit_log_entry

  rollback:
    snapshots: zfs              # Filesystem snapshots
    max_rollbacks: 3
    retention: 90d

  update_types:
    - security_patches: monthly
    - model_updates: quarterly
    - feature_releases: semi_annual
```

**Update Procedure**:
1. **External Staging**: Update bundle created by vendor, dual-signed
2. **Physical Transfer**: DVD/USB or one-way data diode
3. **Staging Zone**: Bundle verified in isolated staging environment
4. **Approval**: Two authorized personnel approve update
5. **Pre-Install Snapshot**: ZFS snapshot for rollback
6. **Install**: Automated installation with verification
7. **Testing**: Smoke tests confirm functionality
8. **Rollback if Failed**: Automatic rollback on test failure
9. **Audit**: Complete audit trail of update process

### Hardware Requirements

| Component | Specification | Quantity | Purpose |
|-----------|---------------|----------|---------|
| **Inference Nodes** | NVIDIA HGX H100 8-GPU, 512GB RAM, 2TB NVMe, Dual 100Gbps NICs | 4 | LLM inference (tensor parallel) |
| **Storage** | NetApp AFF A400 or Pure FlashArray, 100TB usable, NFS | 1 | Model weights, checkpoints, backups |
| **Control Plane** | 32 vCPU, 128GB RAM, 1TB SSD | 3 | Enablement Layer services (HA) |
| **Database** | 16 vCPU, 64GB RAM, 2TB NVMe | 2 | PostgreSQL HA (primary + replica) |
| **Network Switch** | 400Gbps InfiniBand (inference), 25Gbps Ethernet (management) | 2 | Low-latency interconnect |
| **UPS** | 20kW, 30min runtime | 2 | Power redundancy |
| **Cooling** | Rear-door heat exchanger or in-row cooling | - | GPU thermal management |

**Power & Cooling**:
- Total power: ~15kW (4 nodes × 3kW + switches + storage)
- Cooling: 5 tons of cooling capacity
- Rack space: 8U per node, 42U total

**Cost Estimate**: ~$1.2M USD
- Inference nodes: $800K (4 × $200K)
- Storage: $150K
- Networking: $100K
- Control plane/database: $50K
- UPS/cooling: $100K

## Deployment Topology

### Network Isolation

```
┌─────────────────────────────────────────────────────────────┐
│              MANAGEMENT VLAN (10.1.0.0/24)                  │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │ Control  │  │ Control  │  │ Control  │                  │
│  │ Plane 1  │  │ Plane 2  │  │ Plane 3  │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
└─────────────────────────────────────────────────────────────┘
                        │
┌─────────────────────────────────────────────────────────────┐
│             INFERENCE VLAN (10.2.0.0/24)                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  vLLM    │  │  vLLM    │  │  vLLM    │  │  vLLM    │   │
│  │  Node 1  │  │  Node 2  │  │  Node 3  │  │  Node 4  │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
│       │             │             │             │          │
│  ┌────────────────────────────────────────────────────┐    │
│  │        InfiniBand 400Gbps (NVLink proxy)          │    │
│  └────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                        │
┌─────────────────────────────────────────────────────────────┐
│               DATA VLAN (10.3.0.0/24)                       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │PostgreSQL│  │PostgreSQL│  │  Storage │                  │
│  │ Primary  │  │ Replica  │  │  (NFS)   │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
└─────────────────────────────────────────────────────────────┘

NO EXTERNAL ROUTING - ALL VLANs ISOLATED
```

## Consequences

### Positive

1. **True Zero-Egress**: Meets strictest air-gap requirements (SCIF, IL6)
2. **Compliance**: FedRAMP High, NIST 800-53, HIPAA technical safeguards
3. **Feature Parity**: All Enablement Layer capabilities work offline
4. **Self-Contained DR**: Backup and restore without external dependencies
5. **Performance**: Local inference eliminates internet latency (10-50ms saved)
6. **Data Sovereignty**: All data (including model weights) remain in enclave

### Negative

1. **Capital Cost**: $1M+ upfront investment vs. $0 cloud deployment
2. **Operational Complexity**: Requires ML infrastructure expertise
3. **Model Staleness**: Open models lag GPT-4o/Claude by 6-12 months
4. **Manual Updates**: Software/model updates require physical process
5. **No Provider Variety**: Limited to open models (Llama, Qwen, Mistral)
6. **Hardware Failure Risk**: No cloud burst, requires spare parts inventory

### Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Model capability gap vs. GPT-4o | Medium | High | Accept gap; open models improving rapidly |
| Hardware failure (GPU) | High | Low | N+1 redundancy, spare GPU inventory |
| Key management error | Critical | Low | Dual-person control, documented procedures |
| Update bundle tampering | Critical | Very Low | Dual signatures (Ed25519 + ML-DSA), checksums |
| Insufficient inference capacity | Medium | Medium | Overprovisioning (4 nodes vs. 3 minimum) |
| Audit log tampering | High | Very Low | Merkle trees, append-only logs, offline verification |

## Operational Procedures

### Model Update Procedure

**Frequency**: Quarterly

**Steps**:
1. **Vendor Preparation**: New model weights prepared by vendor (Meta, Alibaba)
2. **Dual Signature**: Vendor signs with Ed25519 + ML-DSA-65
3. **Physical Transfer**: Model bundle (50-150GB) via encrypted USB or data diode
4. **Staging Verification**:
   ```bash
   gpg --verify model-bundle.tar.gz.sig model-bundle.tar.gz
   sha256sum -c checksums.txt
   clamscan -r model-bundle/
   ```
5. **Dual-Person Approval**: Two authorized ops personnel approve
6. **Snapshot**: ZFS snapshot before deployment
7. **Deployment**: Unpack to `/models/`, restart vLLM
8. **Smoke Test**: Run 100 test prompts, verify quality
9. **Rollback if Failed**: `zfs rollback` to previous snapshot
10. **Audit**: Log complete update trail

### Key Rotation Procedure

**Frequency**: Annual

**Steps**:
1. **Schedule**: 30-day advance notice to stakeholders
2. **Generate New Keys**: Dual-person ceremony, HSM generates new key pair
3. **Overlap Period**: Both old and new keys valid for 30 days
4. **Update Configs**: Distribute new public keys to all agents
5. **Transition**: After 30 days, old key revoked
6. **Backup**: New key backed up via Shamir secret sharing (3-of-5)
7. **Audit**: Document entire rotation in audit log

### Software Patch Procedure

**Frequency**: Monthly (security patches)

**Steps**:
1. **Patch Bundling**: Vendor creates signed patch bundle
2. **Transfer**: Physical delivery to staging zone
3. **Verification**: Signature check, malware scan, dual approval
4. **Snapshot**: Pre-patch ZFS snapshot
5. **Apply**: `apt-get update && apt-get upgrade` or equivalent
6. **Test**: Automated smoke tests
7. **Rollback if Failed**: ZFS rollback
8. **Audit**: Patch application logged

### Audit Export Procedure

**Frequency**: Weekly

**Steps**:
1. **Export**: PostgreSQL dump to encrypted Parquet format
2. **Verification**: Merkle tree verification against stored roots
3. **Encryption**: AES-256-GCM encryption
4. **Transfer**: Copy to cold storage (offline SAN or tape)
5. **Checksum**: SHA-256 checksum logged
6. **Retention**: 7-year retention per compliance

## Compliance Mapping

| Requirement | Standard | Implementation |
|-------------|----------|----------------|
| **Boundary Protection** | FedRAMP SC-7 | Physical air gap, no external network cables |
| **Transmission Confidentiality** | FedRAMP SC-8 | All traffic encrypted (TLS 1.3), local only |
| **Data at Rest** | FedRAMP SC-28 | AES-256-GCM for all storage (logs, keys, data) |
| **Audit & Accountability** | FedRAMP AU-* | PostgreSQL audit log, Merkle tree integrity |
| **Cryptographic Module** | FIPS 140-2 | SoftHSM2 with FIPS mode enabled |
| **CUI Handling** | IL5/IL6 | All data remains in enclave, no cloud egress |
| **Technical Safeguards** | HIPAA | Encryption, access control, audit logging |
| **Cardholder Data** | PCI-DSS | Network segmentation, no external access |

## Integration with Existing ADRs

- **ADR-001 (Hybrid Signatures)**: Software HSM supports Ed25519 + ML-DSA-65
- **ADR-002 (Storage Strategy)**: Local PostgreSQL + NFS for audit/models
- **ADR-011 (Inference Routing)**: Local vLLM cluster as sole provider
- **ADR-003 (Token Metering)**: All metering data stays local, no cloud export
- **ADR-007 (Audit Trail)**: PostgreSQL with Merkle trees, no external anchoring

## Future Considerations

1. **Federated Learning**: Train custom models on local data without exfiltration
2. **Quantization**: Reduce hardware requirements via INT8/INT4 quantization
3. **Mixture-of-Experts**: Deploy sparse models for better cost/performance
4. **TPU Support**: Google TPU v5 for lower cost than H100 (if available)
5. **Kubernetes**: Container orchestration for easier updates/scaling

## References

- NIST 800-53 Rev 5: Security and Privacy Controls
- FedRAMP High Baseline: https://www.fedramp.gov/
- DoD Cloud Computing SRG: Impact Level 5/6 requirements
- vLLM Documentation: https://docs.vllm.ai/
- SoftHSM2: https://www.opendnssec.org/softhsm/

## Approval

**Approved By**:
- Chief Architect: [Signature]
- CISO: [Signature]
- Director of Infrastructure: [Signature]
- Federal Compliance Officer: [Signature]

**Date**: 2025-12-25
