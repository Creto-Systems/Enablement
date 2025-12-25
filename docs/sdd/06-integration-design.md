---
status: draft
author: Creto Team
created: 2024-12-25
updated: 2024-12-25
reviewers: []
---

# SDD-06: Integration Design

## Purpose

This document defines how the Enablement Layer integrates with Platform and Security layers, external systems, and third-party services.

## Scope

**In Scope:**
- Platform layer integration (NHI, Crypto, Consensus, Audit)
- Security layer integration (AuthZ, Memory, Storage)
- External service integrations (Slack, email, payment processors)
- Integration patterns and error handling

**Out of Scope:**
- Internal Enablement crate integration (see 02-architecture.md)
- API contracts (see 04-api-design.md)

---

## 1. Layer Dependencies

```
┌─────────────────────────────────────────────────────────────────┐
│                    ENABLEMENT LAYER                             │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌─────────┐ │
│  │   Metering   │ │   Oversight  │ │   Runtime    │ │Messaging│ │
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘ └────┬────┘ │
└─────────┼────────────────┼────────────────┼──────────────┼──────┘
          │                │                │              │
          ▼                ▼                ▼              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    SECURITY LAYER                               │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐             │
│  │    AuthZ     │ │    Memory    │ │   Storage    │             │
│  │   (168ns)    │ │  (RuVector)  │ │ (encrypted)  │             │
│  └──────┬───────┘ └──────┬───────┘ └──────┬───────┘             │
└─────────┼────────────────┼────────────────┼─────────────────────┘
          │                │                │
          ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────────┐
│                    PLATFORM LAYER                               │
│  ┌────────┐ ┌──────────┐ ┌───────────┐ ┌─────────┐              │
│  │  NHI   │ │  Crypto  │ │ Consensus │ │  Audit  │              │
│  └────────┘ └──────────┘ └───────────┘ └─────────┘              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Platform Layer Integration

### 2.1 NHI Integration

**Purpose**: Agent identity, delegation chains, key management

```rust
use creto_nhi::{NhiClient, AgentIdentity, DelegationChain};

pub struct NhiIntegration {
    client: NhiClient,
}

impl NhiIntegration {
    /// Verify agent identity from token
    pub async fn verify_identity(&self, token: &str) -> Result<AgentIdentity, Error> {
        self.client.verify_token(token).await
    }

    /// Get delegation chain for attribution
    pub async fn get_delegation_chain(
        &self,
        agent: &AgentIdentity,
    ) -> Result<DelegationChain, Error> {
        self.client.get_delegation_chain(agent).await
    }

    /// Get agent's public keys for encryption/verification
    pub async fn get_public_keys(
        &self,
        agent: &AgentIdentity,
    ) -> Result<AgentPublicKeys, Error> {
        self.client.get_public_keys(agent).await
    }

    /// Delegate signing operation to NHI
    pub async fn sign(
        &self,
        agent: &AgentIdentity,
        data: &[u8],
    ) -> Result<HybridSignature, Error> {
        self.client.sign(agent, data).await
    }

    /// Get delegated secrets for sandbox injection
    pub async fn get_delegated_secrets(
        &self,
        agent: &AgentIdentity,
        secret_refs: &[SecretRef],
    ) -> Result<Vec<Secret>, Error> {
        self.client.get_secrets(agent, secret_refs).await
    }
}
```

**Usage by Enablement crates:**

| Crate | NHI Usage |
|-------|-----------|
| **Metering** | Attribution (agent_nhi, delegation_chain) |
| **Oversight** | Identity context, approver verification |
| **Runtime** | Sandbox binding, secret injection |
| **Messaging** | Key lookup, signing delegation |

### 2.2 Crypto Integration

**Purpose**: Algorithm-agile cryptographic operations

```rust
use creto_crypto::{CryptoProvider, Algorithm};

pub struct CryptoIntegration {
    provider: CryptoProvider,
}

impl CryptoIntegration {
    /// Encrypt with algorithm selection
    pub fn encrypt(
        &self,
        plaintext: &[u8],
        key: &SymmetricKey,
    ) -> Result<EncryptedData, Error> {
        self.provider.encrypt(Algorithm::Aes256Gcm, plaintext, key)
    }

    /// ML-KEM key encapsulation
    pub fn encapsulate(
        &self,
        public_key: &MlKemPublicKey,
    ) -> Result<(SharedSecret, Ciphertext), Error> {
        self.provider.kem_encapsulate(Algorithm::MlKem768, public_key)
    }

    /// Verify hybrid signature
    pub fn verify_signature(
        &self,
        signature: &HybridSignature,
        message: &[u8],
        public_keys: &HybridPublicKey,
    ) -> Result<(), Error> {
        self.provider.verify_hybrid(signature, message, public_keys)
    }

    /// Hash with configurable algorithm
    pub fn hash(&self, data: &[u8]) -> Hash {
        self.provider.hash(self.config.hash_algorithm, data)
    }
}
```

### 2.3 Consensus Integration

**Purpose**: Ordered timestamps, distributed coordination

```rust
use creto_consensus::{ConsensusClient, Timestamp};

pub struct ConsensusIntegration {
    client: ConsensusClient,
}

impl ConsensusIntegration {
    /// Get consensus-ordered timestamp
    pub async fn get_timestamp(&self) -> Result<Timestamp, Error> {
        self.client.get_timestamp().await
    }

    /// Order events for consistent aggregation
    pub async fn order_events(
        &self,
        events: &[EventId],
    ) -> Result<Vec<(EventId, Timestamp)>, Error> {
        self.client.order(events).await
    }
}
```

### 2.4 Audit Integration

**Purpose**: Immutable audit trail, Merkle anchoring

```rust
use creto_audit::{AuditClient, AuditRecord, MerkleProof};

pub struct AuditIntegration {
    client: AuditClient,
}

impl AuditIntegration {
    /// Log operation to audit trail
    pub async fn log(&self, record: AuditRecord) -> Result<AuditId, Error> {
        self.client.log(record).await
    }

    /// Log with signature for non-repudiation
    pub async fn log_signed(
        &self,
        record: AuditRecord,
        signature: HybridSignature,
    ) -> Result<SignedAuditId, Error> {
        self.client.log_signed(record, signature).await
    }

    /// Get Merkle proof for verification
    pub async fn get_proof(&self, id: &AuditId) -> Result<MerkleProof, Error> {
        self.client.get_proof(id).await
    }
}
```

---

## 3. Security Layer Integration

### 3.1 Authorization Integration

**Purpose**: Policy evaluation, quota enforcement, oversight triggers

```rust
use creto_authz::{AuthzClient, CheckRequest, Decision};

pub struct AuthzIntegration {
    client: AuthzClient,
}

impl AuthzIntegration {
    /// Check authorization (168ns policy evaluation)
    pub async fn check(
        &self,
        agent: &AgentIdentity,
        delegation_chain: &[AgentIdentity],
        action: &str,
        resource: &str,
    ) -> Result<Decision, Error> {
        self.client.check(CheckRequest {
            agent: agent.clone(),
            delegation_chain: delegation_chain.to_vec(),
            action: action.to_string(),
            resource: resource.to_string(),
        }).await
    }
}

pub enum Decision {
    Allow,
    Deny { reason: String },
    RequiresOversight { policy_id: PolicyId },
    QuotaExceeded {
        limit: u64,
        used: u64,
        retry_after: Option<Duration>
    },
    RateLimited { retry_after: Duration },
}
```

**Integration points:**

| Crate | Authorization Usage |
|-------|---------------------|
| **Metering** | `QuotaEnforcer` registered as inline extension |
| **Oversight** | `OversightPolicy` evaluated, triggers request creation |
| **Runtime** | Egress checks via `check("network_egress", destination)` |
| **Messaging** | Delivery checks via `check("send_message", recipient)` |

### 3.2 Memory Integration

**Purpose**: Agent context for oversight decisions

```rust
use creto_memory::{MemoryClient, Query, MemorySnippet};

pub struct MemoryIntegration {
    client: MemoryClient,
}

impl MemoryIntegration {
    /// Get context for oversight request
    pub async fn get_context(
        &self,
        agent: &AgentIdentity,
        action_description: &str,
    ) -> Result<Vec<MemorySnippet>, Error> {
        self.client.query(Query {
            agent: agent.clone(),
            query: action_description.to_string(),
            limit: 5,
            relevance_threshold: 0.7,
        }).await
    }

    /// Get reasoning behind agent action
    pub async fn get_reasoning(
        &self,
        agent: &AgentIdentity,
        action_id: &str,
    ) -> Result<Option<String>, Error> {
        self.client.get_reasoning(agent, action_id).await
    }
}
```

### 3.3 Storage Integration

**Purpose**: Large message payloads, sandbox artifacts

```rust
use creto_storage::{StorageClient, StorageRef, Object};

pub struct StorageIntegration {
    client: StorageClient,
}

impl StorageIntegration {
    /// Store large message payload
    pub async fn store_payload(
        &self,
        payload: &[u8],
        agent: &AgentIdentity,
    ) -> Result<StorageRef, Error> {
        self.client.store(Object {
            data: payload.to_vec(),
            owner: agent.clone(),
            classification: Classification::AgentData,
        }).await
    }

    /// Store sandbox checkpoint
    pub async fn store_checkpoint(
        &self,
        checkpoint: &Checkpoint,
        sandbox: &SandboxHandle,
    ) -> Result<StorageRef, Error> {
        self.client.store(Object {
            data: checkpoint.serialize()?,
            owner: sandbox.agent_nhi.clone(),
            classification: Classification::SandboxState,
        }).await
    }
}
```

---

## 4. External Service Integration

### 4.1 Notification Channels (Oversight)

#### Slack Integration

```rust
pub struct SlackChannel {
    client: SlackClient,
    config: SlackConfig,
}

impl NotificationChannel for SlackChannel {
    async fn send(&self, request: &OversightRequest) -> Result<NotificationId, Error> {
        let message = self.format_message(request);
        let blocks = self.build_blocks(request);  // Approve/Deny buttons

        let response = self.client.post_message(PostMessageRequest {
            channel: self.config.channel_id.clone(),
            text: message,
            blocks: Some(blocks),
            metadata: Some(json!({
                "request_id": request.id.to_string(),
            })),
        }).await?;

        Ok(NotificationId::Slack(response.ts))
    }

    async fn handle_interaction(&self, payload: InteractionPayload) -> Result<ApprovalResponse, Error> {
        let action = payload.actions.first().ok_or(Error::InvalidPayload)?;
        let request_id = payload.request_id()?;

        let decision = match action.value.as_str() {
            "approve" => ApprovalDecision::Approve,
            "deny" => ApprovalDecision::Deny,
            _ => return Err(Error::InvalidAction),
        };

        Ok(ApprovalResponse {
            approver: self.resolve_user(&payload.user)?,
            decision,
            timestamp: Timestamp::now(),
            channel: ChannelId::Slack,
            // Note: Slack response is not cryptographically signed
            // We record channel-level verification only
        })
    }
}
```

#### Email Integration

```rust
pub struct EmailChannel {
    mailer: Mailer,
    config: EmailConfig,
}

impl NotificationChannel for EmailChannel {
    async fn send(&self, request: &OversightRequest) -> Result<NotificationId, Error> {
        let email = self.build_email(request)?;

        // Email includes unique approve/deny links
        let approve_link = self.generate_action_link(request, ApprovalDecision::Approve);
        let deny_link = self.generate_action_link(request, ApprovalDecision::Deny);

        let message_id = self.mailer.send(email).await?;
        Ok(NotificationId::Email(message_id))
    }

    fn generate_action_link(&self, request: &OversightRequest, decision: ApprovalDecision) -> String {
        let token = self.sign_action_token(request.id, decision);
        format!("{}/oversight/respond?token={}", self.config.base_url, token)
    }
}
```

#### Webhook Integration

```rust
pub struct WebhookChannel {
    client: HttpClient,
    config: WebhookConfig,
}

impl NotificationChannel for WebhookChannel {
    async fn send(&self, request: &OversightRequest) -> Result<NotificationId, Error> {
        let payload = WebhookPayload {
            request_id: request.id.clone(),
            agent_nhi: request.agent_nhi.clone(),
            action: request.pending_action.clone(),
            context: request.context.clone(),
            callback_url: format!("{}/oversight/callback", self.config.callback_base),
        };

        let signature = self.sign_payload(&payload)?;

        let response = self.client.post(&self.config.url)
            .header("X-Creto-Signature", signature)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(NotificationId::Webhook(response.json::<WebhookResponse>()?.id))
        } else {
            Err(Error::WebhookFailed(response.status()))
        }
    }
}
```

### 4.2 Payment Processors (Metering)

#### Stripe Integration

```rust
pub struct StripeIntegration {
    client: StripeClient,
    config: StripeConfig,
}

impl StripeIntegration {
    /// Sync customer from Creto to Stripe
    pub async fn sync_customer(
        &self,
        subscription: &Subscription,
    ) -> Result<StripeCustomerId, Error> {
        let customer = self.client.customers.create(CreateCustomer {
            email: subscription.contact_email.as_deref(),
            metadata: hashmap! {
                "creto_customer_id" => subscription.customer_id.clone(),
                "creto_subscription_id" => subscription.id.to_string(),
            },
        }).await?;

        Ok(StripeCustomerId(customer.id))
    }

    /// Create invoice in Stripe from Creto invoice
    pub async fn create_invoice(
        &self,
        invoice: &Invoice,
        stripe_customer_id: &StripeCustomerId,
    ) -> Result<StripeInvoiceId, Error> {
        // Create invoice items
        for item in &invoice.line_items {
            self.client.invoice_items.create(CreateInvoiceItem {
                customer: stripe_customer_id.0.clone(),
                amount: item.amount.to_stripe_cents(),
                currency: item.amount.currency.to_stripe(),
                description: Some(item.description.clone()),
            }).await?;
        }

        // Create and finalize invoice
        let stripe_invoice = self.client.invoices.create(CreateInvoice {
            customer: Some(stripe_customer_id.0.clone()),
            auto_advance: Some(true),
            metadata: hashmap! {
                "creto_invoice_id" => invoice.id.to_string(),
            },
        }).await?;

        Ok(StripeInvoiceId(stripe_invoice.id))
    }
}
```

### 4.3 Container Runtimes (Runtime)

#### gVisor Integration

```rust
pub struct GVisorBackend {
    runsc_path: PathBuf,
    config: GVisorConfig,
}

impl SandboxRuntime for GVisorBackend {
    async fn spawn(&self, spec: &SandboxSpec) -> Result<SandboxHandle, Error> {
        // Generate OCI runtime spec
        let oci_spec = self.generate_oci_spec(spec)?;

        // Create sandbox with runsc
        let sandbox_id = self.generate_sandbox_id();
        let result = Command::new(&self.runsc_path)
            .args(["create", "--bundle", &oci_spec.path, &sandbox_id])
            .output()
            .await?;

        if !result.status.success() {
            return Err(Error::SpawnFailed(String::from_utf8_lossy(&result.stderr).to_string()));
        }

        // Start sandbox
        Command::new(&self.runsc_path)
            .args(["start", &sandbox_id])
            .output()
            .await?;

        Ok(SandboxHandle {
            id: SandboxId(sandbox_id),
            spec: spec.clone(),
            status: SandboxStatus::Running { pid: self.get_pid(&sandbox_id)? },
            runtime_backend: "gvisor".to_string(),
            ..Default::default()
        })
    }

    async fn attest(&self, handle: &SandboxHandle) -> Result<Attestation, Error> {
        // gVisor attestation: hash of config + image
        let config_hash = self.hash_config(&handle.spec)?;
        let image_hash = self.get_image_hash(&handle.spec.image)?;

        Ok(Attestation {
            sandbox_id: handle.id.clone(),
            agent_nhi: handle.spec.agent_nhi.clone().unwrap(),
            config_hash,
            image_hash,
            platform: AttestationPlatform::GVisor {
                version: self.get_version()?
            },
            timestamp: Timestamp::now(),
            signature: self.sign_attestation(...)?,
        })
    }
}
```

---

## 5. Integration Patterns

### 5.1 Circuit Breaker

```rust
pub struct CircuitBreaker<T> {
    client: T,
    state: AtomicState,
    config: CircuitBreakerConfig,
}

impl<T> CircuitBreaker<T> {
    pub async fn call<F, R>(&self, f: F) -> Result<R, Error>
    where
        F: FnOnce(&T) -> Future<Output = Result<R, Error>>,
    {
        match self.state.load() {
            State::Closed => {
                match f(&self.client).await {
                    Ok(r) => Ok(r),
                    Err(e) if e.is_transient() => {
                        self.record_failure();
                        Err(e)
                    }
                    Err(e) => Err(e),
                }
            }
            State::Open => {
                if self.should_attempt_reset() {
                    self.state.store(State::HalfOpen);
                    self.call(f).await
                } else {
                    Err(Error::CircuitOpen)
                }
            }
            State::HalfOpen => {
                match f(&self.client).await {
                    Ok(r) => {
                        self.state.store(State::Closed);
                        Ok(r)
                    }
                    Err(e) => {
                        self.state.store(State::Open);
                        Err(e)
                    }
                }
            }
        }
    }
}
```

### 5.2 Retry with Backoff

```rust
pub async fn with_retry<F, T>(
    operation: F,
    config: RetryConfig,
) -> Result<T, Error>
where
    F: Fn() -> Future<Output = Result<T, Error>>,
{
    let mut attempts = 0;
    let mut delay = config.initial_delay;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if e.is_retryable() && attempts < config.max_attempts => {
                attempts += 1;
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, config.max_delay);
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 5.3 Fallback

```rust
pub async fn with_fallback<P, F, T>(
    primary: P,
    fallback: F,
) -> Result<T, Error>
where
    P: FnOnce() -> Future<Output = Result<T, Error>>,
    F: FnOnce() -> Future<Output = Result<T, Error>>,
{
    match primary().await {
        Ok(result) => Ok(result),
        Err(e) => {
            warn!("Primary failed, using fallback: {}", e);
            fallback().await
        }
    }
}
```

---

## 6. Error Handling

### 6.1 Error Categorization

| Category | Handling | Example |
|----------|----------|---------|
| **Transient** | Retry with backoff | Network timeout, rate limit |
| **Permanent** | Return to caller | Invalid input, not found |
| **Degraded** | Fallback or circuit break | Service unavailable |
| **Security** | Log, alert, deny | Auth failure, tampering |

### 6.2 Error Mapping

```rust
impl From<NhiError> for Error {
    fn from(e: NhiError) -> Self {
        match e {
            NhiError::NotFound => Error::AgentNotFound,
            NhiError::TokenExpired => Error::Unauthorized("token expired"),
            NhiError::Unavailable => Error::ServiceUnavailable("nhi"),
            _ => Error::Internal(e.to_string()),
        }
    }
}

impl From<AuthzError> for Error {
    fn from(e: AuthzError) -> Self {
        match e {
            AuthzError::Denied { reason } => Error::PermissionDenied(reason),
            AuthzError::RequiresOversight { policy_id } => Error::OversightRequired(policy_id),
            AuthzError::QuotaExceeded { .. } => Error::QuotaExceeded,
            AuthzError::Unavailable => Error::ServiceUnavailable("authz"),
            _ => Error::Internal(e.to_string()),
        }
    }
}
```

---

## 7. Configuration

### 7.1 Integration Configuration

```toml
# /etc/creto/enablement.toml

[integrations.nhi]
endpoint = "grpc://nhi.creto-platform.svc:50051"
timeout_ms = 100
retry_max_attempts = 3

[integrations.authz]
endpoint = "grpc://authz.creto-security.svc:50051"
timeout_ms = 10  # Must be fast for inline checks
cache_ttl_seconds = 60

[integrations.audit]
endpoint = "grpc://audit.creto-platform.svc:50051"
async_writes = true
batch_size = 100
flush_interval_ms = 1000

[integrations.slack]
enabled = true
signing_secret = "${SLACK_SIGNING_SECRET}"
bot_token = "${SLACK_BOT_TOKEN}"

[integrations.stripe]
enabled = false
api_key = "${STRIPE_API_KEY}"
webhook_secret = "${STRIPE_WEBHOOK_SECRET}"
```

---

## 8. Decisions

| Decision | Rationale |
|----------|-----------|
| gRPC for platform/security | Performance, streaming, typed |
| HTTP for external services | Compatibility, existing SDKs |
| Circuit breaker for all integrations | Resilience, prevent cascade |
| Async audit writes | Don't block hot path |

---

## 9. Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2024-12-25 | 0.1 | Creto Team | Initial draft |
