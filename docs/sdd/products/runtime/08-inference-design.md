---
status: draft
author: Inference Architecture Agent
created: 2025-12-25
updated: 2025-12-25
reviewers: []
github_issue: "#TBD"
---

# RTM-08: Inference Layer Design

## Table of Contents
1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Provider Abstraction](#provider-abstraction)
4. [Cloud Integration](#cloud-integration)
5. [Air-Gap Mode](#air-gap-mode)
6. [Routing & Load Balancing](#routing--load-balancing)
7. [Security Controls](#security-controls)
8. [Operations](#operations)

---

## Overview

### Purpose

The inference layer enables sandboxed agents to access AI model inference capabilities through a unified interface that supports both cloud providers and fully air-gapped local inference.

### Scope

**In Scope:**
- Unified inference provider trait
- Cloud provider integrations (Anthropic, Azure, Bedrock, Vertex AI, OpenAI)
- Local inference runtimes (vLLM, TGI, Ollama)
- Intelligent routing between providers
- Prompt injection detection
- Audit logging and observability

**Out of Scope:**
- Model training (see separate ML platform)
- Fine-tuning infrastructure
- RAG/vector store integration (handled by creto-memory)

### Requirements Mapping

| Requirement | Description | Implementation |
|-------------|-------------|----------------|
| REQ-RT-INF-001 | Multi-provider inference | InferenceProvider trait |
| REQ-RT-INF-002 | Air-gap local inference | LocalInferenceProvider |
| REQ-RT-INF-003 | <100ms routing overhead | InferenceRouter |
| REQ-RT-INF-004 | Prompt injection detection | PromptScanner |
| REQ-RT-INF-005 | Full audit trail | AuditMiddleware |

---

## Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Sandbox Runtime                              │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                       InferenceProxy                            ││
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ ││
│  │  │   Request   │  │   Prompt    │  │        Response         │ ││
│  │  │ Interceptor │→ │   Scanner   │→ │       Processor         │ ││
│  │  └─────────────┘  └─────────────┘  └─────────────────────────┘ ││
│  └──────────────────────────┬──────────────────────────────────────┘│
└─────────────────────────────┼───────────────────────────────────────┘
                              │
┌─────────────────────────────┼───────────────────────────────────────┐
│                     InferenceRouter                                  │
│  ┌──────────────────────────┴──────────────────────────────────────┐│
│  │                     Routing Policy                               ││
│  │  [CloudFirst | LocalOnly | ClassificationBased | CostOptimized] ││
│  └──────────────────────────┬──────────────────────────────────────┘│
│                              │                                       │
│  ┌───────────────────────────┼───────────────────────────────────┐  │
│  │                    Provider Pool                               │  │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐  │  │
│  │  │Anthropic │ │  Azure   │ │ Bedrock  │ │  Local (vLLM)    │  │  │
│  │  │  Claude  │ │  OpenAI  │ │  Claude  │ │  Llama/Mistral   │  │  │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────────────┘  │  │
│  └───────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

| Component | Responsibility |
|-----------|---------------|
| **InferenceProxy** | Entry point, request validation, audit |
| **PromptScanner** | Injection detection, content filtering |
| **InferenceRouter** | Provider selection, failover, load balancing |
| **Provider Pool** | Manages provider connections and health |
| **CloudProvider** | Implements cloud API integrations |
| **LocalProvider** | Implements local inference (vLLM, TGI) |

---

## Provider Abstraction

### Core Trait

```rust
use async_trait::async_trait;
use tokio_stream::Stream;

/// Unified inference provider interface
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    /// Provider identifier
    fn id(&self) -> ProviderId;

    /// Capabilities this provider supports
    fn capabilities(&self) -> &ProviderCapabilities;

    /// Execute completion request
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, InferenceError>;

    /// Stream completion (for long responses)
    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<impl Stream<Item = Result<CompletionChunk, InferenceError>> + Send, InferenceError>;

    /// Generate embeddings
    async fn embed(&self, texts: &[String]) -> Result<Vec<Embedding>, InferenceError>;

    /// Health status
    async fn health(&self) -> HealthStatus;
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub model: ModelId,
    pub messages: Vec<Message>,
    pub max_tokens: u32,
    pub temperature: f32,
    pub stop_sequences: Vec<String>,
    pub metadata: RequestMetadata,
}

#[derive(Debug, Clone)]
pub struct RequestMetadata {
    pub sandbox_id: SandboxId,
    pub agent_nhi: AgentIdentity,
    pub trace_id: TraceId,
    pub classification: Option<DataClassification>,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub id: ResponseId,
    pub content: String,
    pub model: ModelId,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
    pub latency_ms: u64,
}
```

### Provider Capabilities

```rust
#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    /// Supported models
    pub models: Vec<ModelInfo>,
    /// Maximum context window
    pub max_context: u32,
    /// Supports streaming responses
    pub streaming: bool,
    /// Supports function/tool calling
    pub function_calling: bool,
    /// Supports vision inputs
    pub vision: bool,
    /// Supports embeddings
    pub embeddings: bool,
    /// Typical latency range
    pub latency_p50_ms: u32,
    pub latency_p99_ms: u32,
    /// Cost per 1M tokens (input/output)
    pub cost_per_million_input: Decimal,
    pub cost_per_million_output: Decimal,
}

#[derive(Debug, Clone)]
pub struct ModelInfo {
    pub id: ModelId,
    pub name: String,
    pub context_window: u32,
    pub capabilities: ModelCapabilities,
}
```

---

## Cloud Integration

### Anthropic Claude Provider

```rust
pub struct AnthropicProvider {
    client: anthropic::Client,
    config: AnthropicConfig,
    rate_limiter: RateLimiter,
}

#[async_trait]
impl InferenceProvider for AnthropicProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Anthropic
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &ANTHROPIC_CAPABILITIES
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, InferenceError> {
        // Rate limiting
        self.rate_limiter.acquire().await?;

        // Map to Anthropic API format
        let api_request = anthropic::MessageCreateParams {
            model: self.map_model(&request.model)?,
            messages: request.messages.iter()
                .map(|m| self.map_message(m))
                .collect(),
            max_tokens: request.max_tokens,
            temperature: Some(request.temperature),
            stop_sequences: Some(request.stop_sequences.clone()),
            ..Default::default()
        };

        // Execute with timeout
        let start = Instant::now();
        let response = tokio::time::timeout(
            self.config.timeout,
            self.client.messages.create(api_request),
        ).await
            .map_err(|_| InferenceError::Timeout)?
            .map_err(|e| InferenceError::ProviderError(e.to_string()))?;

        Ok(CompletionResponse {
            id: ResponseId::new(),
            content: response.content.into_text(),
            model: request.model,
            usage: TokenUsage {
                input_tokens: response.usage.input_tokens,
                output_tokens: response.usage.output_tokens,
            },
            finish_reason: self.map_finish_reason(response.stop_reason),
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<impl Stream<Item = Result<CompletionChunk, InferenceError>> + Send, InferenceError> {
        // Similar implementation with streaming
        todo!()
    }

    async fn embed(&self, _texts: &[String]) -> Result<Vec<Embedding>, InferenceError> {
        Err(InferenceError::NotSupported("Anthropic does not support embeddings".into()))
    }

    async fn health(&self) -> HealthStatus {
        match self.client.messages.create_minimal_test().await {
            Ok(_) => HealthStatus::Healthy,
            Err(e) if e.is_rate_limit() => HealthStatus::Degraded {
                reason: "Rate limited".into(),
            },
            Err(_) => HealthStatus::Unhealthy {
                reason: "API unreachable".into(),
            },
        }
    }
}
```

### Provider Configuration

```rust
#[derive(Debug, Clone, Deserialize)]
pub struct CloudProvidersConfig {
    pub anthropic: Option<AnthropicConfig>,
    pub azure_openai: Option<AzureOpenAIConfig>,
    pub bedrock: Option<BedrockConfig>,
    pub vertex_ai: Option<VertexAIConfig>,
    pub openai: Option<OpenAIConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicConfig {
    /// API key (from secret store)
    pub api_key_ref: SecretRef,
    /// Base URL override
    pub base_url: Option<String>,
    /// Request timeout
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    /// Max concurrent requests
    pub max_concurrent: usize,
    /// Rate limit (requests per minute)
    pub rate_limit_rpm: u32,
}
```

---

## Air-Gap Mode

### Local Inference Provider

```rust
pub struct LocalInferenceProvider {
    /// vLLM/TGI endpoint pool
    endpoints: Vec<LocalEndpoint>,
    /// Load balancer
    balancer: RoundRobinBalancer,
    /// Model registry
    registry: ModelRegistry,
    /// Health checker
    health: HealthChecker,
}

#[derive(Debug, Clone)]
pub struct LocalEndpoint {
    pub url: Url,
    pub model: ModelId,
    pub gpu_count: u8,
    pub max_concurrent: usize,
}

#[async_trait]
impl InferenceProvider for LocalInferenceProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Local
    }

    fn capabilities(&self) -> &ProviderCapabilities {
        &self.registry.capabilities()
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, InferenceError> {
        // Select endpoint
        let endpoint = self.balancer.next(&request.model)?;

        // vLLM uses OpenAI-compatible API
        let client = reqwest::Client::new();
        let api_request = OpenAIRequest {
            model: request.model.as_str(),
            messages: request.messages.iter()
                .map(|m| OpenAIMessage::from(m))
                .collect(),
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stop: request.stop_sequences.clone(),
        };

        let start = Instant::now();
        let response: OpenAIResponse = client
            .post(format!("{}/v1/chat/completions", endpoint.url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&api_request)
            .send()
            .await?
            .json()
            .await?;

        Ok(CompletionResponse {
            id: ResponseId::new(),
            content: response.choices[0].message.content.clone(),
            model: request.model,
            usage: TokenUsage {
                input_tokens: response.usage.prompt_tokens,
                output_tokens: response.usage.completion_tokens,
            },
            finish_reason: FinishReason::from_openai(&response.choices[0].finish_reason),
            latency_ms: start.elapsed().as_millis() as u64,
        })
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Embedding>, InferenceError> {
        let endpoint = self.balancer.next_embedding()?;

        let response: EmbeddingResponse = reqwest::Client::new()
            .post(format!("{}/v1/embeddings", endpoint.url))
            .json(&EmbeddingRequest {
                model: "text-embedding",
                input: texts.to_vec(),
            })
            .send()
            .await?
            .json()
            .await?;

        Ok(response.data.into_iter().map(|d| d.embedding).collect())
    }

    async fn health(&self) -> HealthStatus {
        self.health.check_all().await
    }
}
```

### Model Registry

```rust
pub struct ModelRegistry {
    /// Available models (loaded from disk)
    models: HashMap<ModelId, ModelMetadata>,
    /// Model weights path
    weights_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ModelMetadata {
    pub id: ModelId,
    pub name: String,
    pub size_bytes: u64,
    pub hash: Hash,
    pub signature: HybridSignature,
    pub capabilities: ModelCapabilities,
    pub loaded_at: Option<DateTime<Utc>>,
}

impl ModelRegistry {
    /// Load and verify model from disk
    pub async fn load_model(
        &mut self,
        model_id: &ModelId,
        signing_key: &VerifyingKey,
    ) -> Result<(), ModelError> {
        let metadata = self.models.get(model_id)
            .ok_or(ModelError::NotFound)?;

        // Verify integrity before loading
        let model_path = self.weights_path.join(&model_id.0);
        let actual_hash = hash_directory(&model_path, Algorithm::Sha3_256)?;

        if actual_hash != metadata.hash {
            return Err(ModelError::IntegrityCheckFailed {
                expected: metadata.hash.clone(),
                actual: actual_hash,
            });
        }

        // Verify signature
        signing_key.verify(actual_hash.as_bytes(), &metadata.signature)
            .map_err(|_| ModelError::SignatureInvalid)?;

        Ok(())
    }
}
```

---

## Routing & Load Balancing

### Inference Router

```rust
pub struct InferenceRouter {
    /// Available providers
    providers: Vec<Arc<dyn InferenceProvider>>,
    /// Routing policy
    policy: RoutingPolicy,
    /// Data classifier for routing decisions
    classifier: Option<DataClassifier>,
    /// Circuit breakers per provider
    circuits: HashMap<ProviderId, CircuitBreaker>,
    /// Metrics collector
    metrics: RouterMetrics,
}

impl InferenceRouter {
    pub async fn route(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, InferenceError> {
        // Select provider based on policy
        let provider = self.select_provider(&request).await?;

        // Check circuit breaker
        let circuit = self.circuits.get(&provider.id())
            .ok_or(InferenceError::ProviderUnavailable)?;

        if !circuit.allow_request() {
            // Try fallback provider
            let fallback = self.select_fallback(&request).await?;
            return self.execute_with_fallback(request, fallback).await;
        }

        // Execute request
        let start = Instant::now();
        let result = provider.complete(request.clone()).await;

        // Update circuit breaker
        match &result {
            Ok(_) => circuit.record_success(),
            Err(e) if e.is_transient() => circuit.record_failure(),
            Err(_) => {} // Permanent errors don't trip circuit
        }

        // Record metrics
        self.metrics.record_request(
            provider.id(),
            start.elapsed(),
            result.is_ok(),
        );

        result
    }

    async fn select_provider(
        &self,
        request: &CompletionRequest,
    ) -> Result<Arc<dyn InferenceProvider>, InferenceError> {
        match &self.policy {
            RoutingPolicy::CloudFirst { fallback_to_local } => {
                // Try cloud providers first
                for provider in &self.providers {
                    if provider.id().is_cloud()
                        && provider.capabilities().supports_model(&request.model)
                        && self.circuits.get(&provider.id()).map(|c| c.allow_request()).unwrap_or(false)
                    {
                        return Ok(provider.clone());
                    }
                }

                if *fallback_to_local {
                    self.select_local_provider(request).await
                } else {
                    Err(InferenceError::NoProviderAvailable)
                }
            }

            RoutingPolicy::LocalOnly => {
                self.select_local_provider(request).await
            }

            RoutingPolicy::ClassificationBased { local_classifications } => {
                // Check data classification
                let classification = self.classifier
                    .as_ref()
                    .map(|c| c.classify(&request))
                    .unwrap_or(DataClassification::Public);

                if local_classifications.contains(&classification) {
                    self.select_local_provider(request).await
                } else {
                    self.select_cloud_provider(request).await
                }
            }

            RoutingPolicy::CostOptimized { max_cost_per_token } => {
                // Select cheapest provider that meets requirements
                self.providers.iter()
                    .filter(|p| {
                        p.capabilities().cost_per_million_input / Decimal::from(1_000_000)
                            <= *max_cost_per_token
                    })
                    .min_by_key(|p| p.capabilities().cost_per_million_input)
                    .cloned()
                    .ok_or(InferenceError::NoProviderAvailable)
            }

            RoutingPolicy::LatencyOptimized { max_latency_ms } => {
                self.providers.iter()
                    .filter(|p| p.capabilities().latency_p99_ms <= *max_latency_ms as u32)
                    .min_by_key(|p| p.capabilities().latency_p50_ms)
                    .cloned()
                    .ok_or(InferenceError::NoProviderAvailable)
            }
        }
    }
}
```

---

## Security Controls

### Prompt Injection Detection

```rust
pub struct PromptScanner {
    /// Regex patterns for known injection techniques
    patterns: Vec<CompiledPattern>,
    /// ML-based classifier (optional)
    ml_classifier: Option<InjectionClassifier>,
    /// Action on detection
    on_detection: InjectionAction,
}

#[derive(Debug, Clone)]
pub enum InjectionRisk {
    None,
    Low { indicators: Vec<String> },
    Medium { indicators: Vec<String>, confidence: f32 },
    High { indicators: Vec<String>, confidence: f32 },
}

impl PromptScanner {
    pub async fn scan(&self, request: &CompletionRequest) -> Result<InjectionRisk, ScanError> {
        let mut risk = InjectionRisk::None;
        let mut indicators = Vec::new();

        // Combine all text content
        let text = request.messages.iter()
            .map(|m| m.content.as_str())
            .collect::<Vec<_>>()
            .join("\n");

        // Pattern matching
        for pattern in &self.patterns {
            if pattern.regex.is_match(&text) {
                indicators.push(pattern.name.clone());
            }
        }

        // ML classification if available
        let ml_confidence = if let Some(classifier) = &self.ml_classifier {
            classifier.predict(&text).await?
        } else {
            0.0
        };

        // Determine risk level
        risk = match (indicators.len(), ml_confidence) {
            (0, c) if c < 0.3 => InjectionRisk::None,
            (1..=2, c) if c < 0.5 => InjectionRisk::Low { indicators },
            (_, c) if c < 0.8 => InjectionRisk::Medium { indicators, confidence: c },
            _ => InjectionRisk::High { indicators, confidence: ml_confidence },
        };

        Ok(risk)
    }
}

/// Known injection patterns
const INJECTION_PATTERNS: &[(&str, &str)] = &[
    ("ignore_instructions", r"(?i)(ignore|disregard|forget).*(previous|above|prior|system).*(instructions?|prompt)"),
    ("system_override", r"(?i)(you are now|act as|pretend to be|roleplay as)"),
    ("jailbreak_attempt", r"(?i)(DAN|do anything now|developer mode|sudo mode)"),
    ("prompt_leak", r"(?i)(show|reveal|display|print).*(system prompt|instructions|rules)"),
    ("delimiter_injection", r"(```|<\|im_start\|>|<\|im_end\|>|\[INST\]|\[/INST\])"),
];
```

### Audit Middleware

```rust
pub struct AuditMiddleware {
    logger: AsyncAuditLogger,
}

impl AuditMiddleware {
    pub async fn wrap<F, R>(
        &self,
        request: &CompletionRequest,
        operation: F,
    ) -> Result<R, InferenceError>
    where
        F: Future<Output = Result<R, InferenceError>>,
    {
        // Log request start
        self.logger.log(AuditRecord {
            who: request.metadata.agent_nhi.clone(),
            delegation_chain: vec![],
            what: "inference_request".to_string(),
            resource: format!("model:{}", request.model),
            why: None,
            outcome: Outcome::InProgress,
            timestamp: Timestamp::now(),
            signature: None,
        });

        let start = Instant::now();
        let result = operation.await;

        // Log completion
        let outcome = match &result {
            Ok(_) => Outcome::Success,
            Err(e) => Outcome::Failure(e.to_string()),
        };

        self.logger.log(AuditRecord {
            who: request.metadata.agent_nhi.clone(),
            delegation_chain: vec![],
            what: "inference_complete".to_string(),
            resource: format!("model:{}", request.model),
            why: Some(format!("latency_ms:{}", start.elapsed().as_millis())),
            outcome,
            timestamp: Timestamp::now(),
            signature: None,
        });

        result
    }
}
```

---

## Operations

### Health Checks

```rust
pub struct InferenceHealthCheck {
    router: Arc<InferenceRouter>,
}

impl HealthCheck for InferenceHealthCheck {
    async fn check(&self) -> HealthStatus {
        let mut statuses = Vec::new();

        for provider in self.router.providers() {
            let status = provider.health().await;
            statuses.push((provider.id(), status));
        }

        // All unhealthy = unhealthy
        if statuses.iter().all(|(_, s)| matches!(s, HealthStatus::Unhealthy { .. })) {
            return HealthStatus::Unhealthy {
                reason: "All inference providers unavailable".into(),
            };
        }

        // Any unhealthy = degraded
        if statuses.iter().any(|(_, s)| matches!(s, HealthStatus::Unhealthy { .. })) {
            return HealthStatus::Degraded {
                reason: "Some inference providers unavailable".into(),
            };
        }

        HealthStatus::Healthy
    }
}
```

### Metrics

```rust
pub struct RouterMetrics {
    /// Request count by provider
    requests_total: IntCounterVec,
    /// Latency histogram by provider
    latency_seconds: HistogramVec,
    /// Error rate by provider
    errors_total: IntCounterVec,
    /// Token usage by provider
    tokens_total: IntCounterVec,
}

impl RouterMetrics {
    pub fn new(registry: &Registry) -> Self {
        Self {
            requests_total: register_int_counter_vec_with_registry!(
                "inference_requests_total",
                "Total inference requests",
                &["provider", "model"],
                registry
            ).unwrap(),
            latency_seconds: register_histogram_vec_with_registry!(
                "inference_latency_seconds",
                "Inference request latency",
                &["provider", "model"],
                vec![0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
                registry
            ).unwrap(),
            errors_total: register_int_counter_vec_with_registry!(
                "inference_errors_total",
                "Total inference errors",
                &["provider", "error_type"],
                registry
            ).unwrap(),
            tokens_total: register_int_counter_vec_with_registry!(
                "inference_tokens_total",
                "Total tokens processed",
                &["provider", "direction"],
                registry
            ).unwrap(),
        }
    }
}
```

### Configuration

```toml
# /etc/creto/inference.toml

[inference]
# Default routing policy
routing_policy = "cloud_first"
fallback_to_local = true

# Prompt injection settings
[inference.security]
injection_detection = true
injection_action = "block"  # block | warn | log
injection_threshold = 0.7

[inference.providers.anthropic]
enabled = true
api_key_ref = "secrets/anthropic-api-key"
timeout = "30s"
rate_limit_rpm = 1000
models = ["claude-3-5-sonnet", "claude-3-opus"]

[inference.providers.local]
enabled = true
endpoints = [
    { url = "http://vllm-1:8000", model = "llama-3.1-70b", gpus = 8 },
    { url = "http://vllm-2:8000", model = "llama-3.1-70b", gpus = 8 },
]
health_check_interval = "10s"
```

---

## Revision History

| Date | Version | Author | Changes |
|------|---------|--------|---------|
| 2025-12-25 | 0.1 | Inference Architecture Agent | Initial inference layer design |
