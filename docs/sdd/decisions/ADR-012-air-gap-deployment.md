---
status: approved
date: 2025-12-25
author: Inference Architecture Agent
deciders: [Architecture Team, Security Team, Infrastructure Team]
consulted: [Compliance, Operations, ML Infrastructure]
informed: [Platform Team, Customer Success]
---

# ADR-012: Air-Gap Deployment Architecture

## Context

Enterprise and government customers require fully air-gapped deployment where:

1. **No internet connectivity**: Complete network isolation
2. **Data sovereignty**: All data processed locally
3. **Model self-hosting**: Local LLM inference
4. **Offline updates**: Secure update mechanism without connectivity
5. **Compliance**: Meet FedRAMP High, ITAR, classified requirements

## Decision

We will support **full air-gap deployment** with local inference, pre-provisioned identities, and data diode updates.

### Deployment Topology

```
┌─────────────────────────────────────────────────────────────────────┐
│                     AIR-GAPPED ENVIRONMENT                          │
│  ┌───────────────────────────────────────────────────────────────┐  │
│  │                    Creto Runtime Cluster                      │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐   │  │
│  │  │   Runtime   │  │   Runtime   │  │   Control Plane     │   │  │
│  │  │   Node 1    │  │   Node 2    │  │   (HA Pair)         │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘   │  │
│  │         │                │                    │               │  │
│  │  ┌──────┴────────────────┴────────────────────┴────────┐     │  │
│  │  │              Internal Network (Isolated)             │     │  │
│  │  └──────────────────────────┬──────────────────────────┘     │  │
│  └─────────────────────────────┼─────────────────────────────────┘  │
│                                │                                     │
│  ┌─────────────────────────────┼─────────────────────────────────┐  │
│  │              Local Inference Cluster                          │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐   │  │
│  │  │  vLLM Node  │  │  vLLM Node  │  │  Model Registry     │   │  │
│  │  │  (H100×8)   │  │  (H100×8)   │  │  (Harbor)           │   │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘   │  │
│  │  ┌─────────────┐  ┌─────────────┐                            │  │
│  │  │  vLLM Node  │  │  vLLM Node  │                            │  │
│  │  │  (H100×8)   │  │  (H100×8)   │                            │  │
│  │  └─────────────┘  └─────────────┘                            │  │
│  └───────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                    Security Services                             ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐     ││
│  │  │  SoftHSM2   │  │   Vault     │  │   Pre-provisioned   │     ││
│  │  │  (PKCS#11)  │  │  (Secrets)  │  │   NHI Store         │     ││
│  │  └─────────────┘  └─────────────┘  └─────────────────────┘     ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                │                                     │
│  ┌─────────────────────────────┼─────────────────────────────────┐  │
│  │                      DATA DIODE                               │  │
│  │              (Unidirectional: External → Internal)            │  │
│  └─────────────────────────────┬─────────────────────────────────┘  │
└────────────────────────────────┼────────────────────────────────────┘
                                 │ (Updates only, no data out)
┌────────────────────────────────┼────────────────────────────────────┐
│                         EXTERNAL                                     │
│  ┌─────────────────────────────┴─────────────────────────────────┐  │
│  │                    Update Staging Server                       │  │
│  │  - Model weights (signed)                                      │  │
│  │  - Software packages (signed)                                  │  │
│  │  - Policy updates (signed)                                     │  │
│  └───────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────┘
```

### Component Configuration

#### 1. Local Inference (vLLM Cluster)

```yaml
# vllm-deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: vllm-inference
  namespace: creto-inference
spec:
  replicas: 4
  selector:
    matchLabels:
      app: vllm
  template:
    spec:
      containers:
      - name: vllm
        image: harbor.internal/creto/vllm:0.4.0
        args:
          - --model=/models/llama-3.1-70b-instruct
          - --tensor-parallel-size=8
          - --max-model-len=32768
          - --gpu-memory-utilization=0.95
          - --api-key-file=/secrets/api-key
        resources:
          limits:
            nvidia.com/gpu: 8
        volumeMounts:
          - name: model-weights
            mountPath: /models
            readOnly: true
          - name: api-secrets
            mountPath: /secrets
            readOnly: true
      nodeSelector:
        gpu-type: h100
      volumes:
        - name: model-weights
          persistentVolumeClaim:
            claimName: llama-3.1-70b-pvc
        - name: api-secrets
          secret:
            secretName: vllm-api-key
```

#### 2. Software HSM (SoftHSM2)

```rust
/// Software HSM configuration for air-gapped environments
pub struct SoftHsmConfig {
    /// PKCS#11 library path
    pub pkcs11_lib: PathBuf,
    /// Token label
    pub token_label: String,
    /// User PIN (from secure storage)
    pub user_pin: SecretString,
    /// Key slot configuration
    pub slots: Vec<HsmSlot>,
}

pub struct HsmSlot {
    pub slot_id: u64,
    pub key_type: KeyType,
    pub label: String,
}

impl SoftHsmProvider {
    pub fn new(config: SoftHsmConfig) -> Result<Self, HsmError> {
        let ctx = Pkcs11::new(&config.pkcs11_lib)?;
        ctx.initialize(CInitializeArgs::OsThreads)?;

        let slot = ctx.get_slots_with_initialized_token()?
            .into_iter()
            .find(|s| {
                ctx.get_token_info(*s)
                    .map(|t| t.label().trim() == config.token_label)
                    .unwrap_or(false)
            })
            .ok_or(HsmError::TokenNotFound)?;

        let session = ctx.open_rw_session(slot)?;
        session.login(UserType::User, Some(&config.user_pin))?;

        Ok(Self { ctx, session, config })
    }

    pub fn sign(&self, key_label: &str, data: &[u8]) -> Result<Vec<u8>, HsmError> {
        let key = self.find_key(key_label)?;
        let mechanism = Mechanism::EcdsaSha256;
        self.session.sign(&mechanism, key, data)
    }
}
```

#### 3. Pre-provisioned NHI Store

```rust
/// Pre-provisioned agent identities for air-gapped operation
pub struct PreProvisionedNhiStore {
    /// Local database of agent identities
    db: SqliteConnection,
    /// HSM for key operations
    hsm: Arc<SoftHsmProvider>,
    /// Identity cache
    cache: Arc<RwLock<HashMap<AgentId, AgentIdentity>>>,
}

impl PreProvisionedNhiStore {
    /// Load pre-provisioned identities from encrypted backup
    pub async fn load_from_backup(
        backup_path: &Path,
        hsm: Arc<SoftHsmProvider>,
    ) -> Result<Self, NhiError> {
        // Decrypt backup using HSM key
        let decryption_key = hsm.unwrap_key("nhi-backup-key")?;
        let backup_data = decrypt_file(backup_path, &decryption_key)?;

        // Load into SQLite
        let db = SqliteConnection::open_in_memory()?;
        db.execute_batch(&backup_data)?;

        Ok(Self {
            db,
            hsm,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Resolve agent identity locally
    pub async fn resolve(&self, token: &str) -> Result<AgentIdentity, NhiError> {
        // Verify token signature locally
        let claims = self.verify_token_locally(token)?;

        // Look up in local store
        let agent_id = claims.sub;

        if let Some(identity) = self.cache.read().await.get(&agent_id) {
            return Ok(identity.clone());
        }

        let identity = self.db.query_row(
            "SELECT * FROM agents WHERE id = ?",
            [&agent_id],
            |row| AgentIdentity::from_row(row),
        )?;

        self.cache.write().await.insert(agent_id, identity.clone());
        Ok(identity)
    }
}
```

#### 4. Data Diode Update Flow

```rust
/// Secure update mechanism via data diode
pub struct DataDiodeUpdater {
    /// Path to data diode mount point
    diode_path: PathBuf,
    /// Signature verification keys
    signing_keys: Vec<VerifyingKey>,
    /// Update staging directory
    staging_dir: PathBuf,
}

impl DataDiodeUpdater {
    /// Check for and apply updates
    pub async fn check_updates(&self) -> Result<Vec<Update>, UpdateError> {
        let mut updates = Vec::new();

        // Scan diode path for update packages
        for entry in fs::read_dir(&self.diode_path)? {
            let path = entry?.path();
            if path.extension() == Some("crupdate".as_ref()) {
                // Verify signature chain
                let update = self.verify_and_parse(&path).await?;

                // Validate update metadata
                self.validate_update(&update)?;

                updates.push(update);
            }
        }

        Ok(updates)
    }

    /// Verify update package signature
    async fn verify_and_parse(&self, path: &Path) -> Result<Update, UpdateError> {
        let package = fs::read(path)?;

        // Parse outer envelope
        let envelope: UpdateEnvelope = bincode::deserialize(&package)?;

        // Verify signature (requires M-of-N signing keys)
        let verified_count = self.signing_keys.iter()
            .filter(|key| {
                key.verify(&envelope.payload, &envelope.signatures)
                    .map(|_| true)
                    .unwrap_or(false)
            })
            .count();

        if verified_count < envelope.required_signatures {
            return Err(UpdateError::InsufficientSignatures {
                required: envelope.required_signatures,
                verified: verified_count,
            });
        }

        // Parse inner update
        let update: Update = bincode::deserialize(&envelope.payload)?;
        Ok(update)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    pub id: UpdateId,
    pub version: Version,
    pub update_type: UpdateType,
    pub payload: Vec<u8>,
    pub checksum: Hash,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UpdateType {
    ModelWeights { model_id: String },
    SoftwarePackage { package_name: String },
    PolicyUpdate { policy_id: String },
    CertificateRotation,
}
```

## Consequences

### Positive

- **Complete isolation**: No data leaves the environment
- **Compliance ready**: Meets FedRAMP High, ITAR, classified requirements
- **Self-sufficient**: Operates indefinitely without connectivity
- **Secure updates**: Cryptographically verified via data diode

### Negative

- **Higher cost**: ~$1.2M hardware investment for inference cluster
- **Operational complexity**: Local model management, HSM administration
- **Update latency**: Days/weeks for updates vs. immediate in connected mode
- **Reduced model variety**: Limited to models that can be self-hosted

### Neutral

- **Performance trade-off**: Local inference may be faster (no network) or slower (smaller cluster)
- **Staffing requirements**: Need ML infrastructure expertise on-site

## Hardware Requirements

### Minimum Air-Gap Inference Cluster

| Component | Specification | Quantity | Cost Estimate |
|-----------|--------------|----------|---------------|
| **GPU Servers** | 8× H100 80GB per node | 4 nodes | $1,000,000 |
| **Storage** | 100TB NVMe (model weights) | 1 array | $80,000 |
| **Network** | 400GbE InfiniBand | 4 switches | $60,000 |
| **HSM** | SoftHSM2 on dedicated server | 2 (HA) | $10,000 |
| **Data Diode** | Hardware data diode | 1 | $50,000 |
| **Total** | | | **~$1,200,000** |

### Recommended Models for Air-Gap

| Model | Parameters | Context | Use Case | VRAM Required |
|-------|------------|---------|----------|---------------|
| **Llama 3.1 70B** | 70B | 128K | General purpose | 140GB (FP16) |
| **Llama 3.1 405B** | 405B | 128K | High capability | 810GB (FP16) |
| **Qwen2.5 72B** | 72B | 128K | Multilingual | 144GB (FP16) |
| **DeepSeek V2.5** | 236B (MoE) | 128K | Coding | 150GB (FP16) |
| **Mistral Large 2** | 123B | 128K | Enterprise | 246GB (FP16) |

## Security Controls

### Model Integrity

```rust
/// Verify model weights before loading
pub async fn verify_model_integrity(
    model_path: &Path,
    expected_hash: &Hash,
    signature: &HybridSignature,
    signing_key: &VerifyingKey,
) -> Result<(), ModelIntegrityError> {
    // Hash model files
    let actual_hash = hash_directory(model_path, Algorithm::Sha3_256)?;

    if actual_hash != *expected_hash {
        return Err(ModelIntegrityError::HashMismatch {
            expected: expected_hash.clone(),
            actual: actual_hash,
        });
    }

    // Verify signature
    signing_key.verify(&actual_hash.as_bytes(), signature)
        .map_err(|_| ModelIntegrityError::SignatureInvalid)?;

    Ok(())
}
```

### Network Isolation Verification

```rust
/// Verify network isolation at runtime
pub async fn verify_isolation() -> Result<IsolationStatus, IsolationError> {
    // Attempt external DNS resolution (should fail)
    let dns_result = tokio::time::timeout(
        Duration::from_secs(5),
        tokio::net::lookup_host("1.1.1.1:53"),
    ).await;

    if dns_result.is_ok() {
        return Err(IsolationError::ExternalConnectivity {
            detail: "External DNS resolution succeeded".to_string(),
        });
    }

    // Verify no default gateway to external networks
    let routes = get_routing_table()?;
    if routes.has_external_gateway() {
        return Err(IsolationError::ExternalGateway);
    }

    Ok(IsolationStatus::Verified)
}
```

## Related Decisions

- ADR-011: Unified Inference Provider Abstraction
- ADR-001: Hybrid Signature Strategy (for update signing)
- ADR-003: Storage Strategy (for model weight storage)

## References

- [NIST SP 800-53 Rev 5](https://csrc.nist.gov/publications/detail/sp/800-53/rev-5/final)
- [FedRAMP High Baseline](https://www.fedramp.gov/baselines/)
- [Data Diode Technology Guide](https://www.cisa.gov/data-diode)
- [vLLM Offline Deployment](https://docs.vllm.ai/en/latest/serving/offline_inference.html)
