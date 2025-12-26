---
status: approved
date: 2025-12-25
author: Inference Architecture Agent
deciders: [Architecture Team, Security Team, ML Infrastructure Team]
consulted: [Cloud Provider Integration, Air-Gap Deployment]
informed: [Platform Team, Operations]
---

# ADR-011: Unified Inference Provider Abstraction

## Context

The Creto Runtime needs to support AI model inference for agent workloads. Requirements include:

1. **Multi-provider support**: Anthropic Claude, Azure OpenAI, AWS Bedrock, Google Vertex AI, OpenAI
2. **Air-gapped deployment**: Fully offline operation with local models (vLLM, TGI, Ollama)
3. **Hybrid routing**: Classification-based routing between cloud and local
4. **Performance**: <100ms P99 for cached responses, <2s P99 for inference
5. **Security**: Prompt injection detection, API key protection, audit logging

## Decision

We will implement a **unified InferenceProvider trait** that abstracts cloud and local inference behind a common interface, with intelligent routing.

### Core Trait Design

```rust
/// Unified inference provider abstraction
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    /// Get provider identifier
    fn provider_id(&self) -> ProviderId;

    /// Check provider capabilities
    fn capabilities(&self) -> ProviderCapabilities;

    /// Execute completion request
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, InferenceError>;

    /// Stream completion response
    async fn complete_stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<CompletionChunk, InferenceError>> + Send>>, InferenceError>;

    /// Generate embeddings
    async fn embed(
        &self,
        texts: &[String],
    ) -> Result<Vec<Embedding>, InferenceError>;

    /// Health check
    async fn health_check(&self) -> HealthStatus;
}

#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    pub max_tokens: u32,
    pub supports_streaming: bool,
    pub supports_embeddings: bool,
    pub supports_vision: bool,
    pub supports_function_calling: bool,
    pub models: Vec<ModelInfo>,
}
```

### Routing Strategy

```rust
pub enum RoutingPolicy {
    /// Always use cloud providers (default for connected environments)
    CloudFirst { fallback_to_local: bool },

    /// Always use local inference (air-gapped mode)
    LocalOnly,

    /// Route based on data classification
    ClassificationBased {
        /// PII/sensitive data stays local
        local_classifications: Vec<DataClassification>,
    },

    /// Route based on model capability requirements
    CapabilityBased {
        /// Prefer providers with specific capabilities
        required_capabilities: ProviderCapabilities,
    },

    /// Cost-optimized routing
    CostOptimized {
        max_cost_per_token: Decimal,
    },

    /// Latency-optimized routing
    LatencyOptimized {
        max_latency_ms: u64,
    },
}

pub struct InferenceRouter {
    providers: Vec<Arc<dyn InferenceProvider>>,
    policy: RoutingPolicy,
    classifier: Option<DataClassifier>,
    metrics: RouterMetrics,
}

impl InferenceRouter {
    pub async fn route(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, InferenceError> {
        let provider = self.select_provider(&request).await?;

        // Pre-process: prompt injection scan
        self.scan_for_injection(&request).await?;

        // Execute with timeout and retry
        let response = with_retry(
            || provider.complete(request.clone()),
            RetryConfig::default(),
        ).await?;

        // Post-process: audit logging
        self.audit_inference(&request, &response, provider.provider_id()).await;

        Ok(response)
    }
}
```

## Consequences

### Positive

- **Provider agnostic**: Agents don't need to know which provider handles requests
- **Seamless failover**: Automatic fallback between providers
- **Air-gap compatible**: Same API works in disconnected environments
- **Security integrated**: Injection detection and audit built-in
- **Observable**: Unified metrics across all providers

### Negative

- **Abstraction overhead**: ~1-2ms routing overhead per request
- **Lowest common denominator**: Provider-specific features require escape hatches
- **Complexity**: Router logic adds maintenance burden

### Neutral

- **Configuration complexity**: Routing policies need careful tuning
- **Model mapping**: Need to maintain model equivalence tables

## Alternatives Considered

### 1. Direct Provider Integration

Each agent calls providers directly with provider-specific SDKs.

**Rejected because**:
- No unified audit trail
- No air-gap support
- Security controls scattered across codebase
- High integration burden per provider

### 2. LLM Gateway (LiteLLM/OpenRouter)

Use existing LLM gateway projects.

**Rejected because**:
- External dependency for critical path
- No native NHI integration
- Limited air-gap support
- Security controls outside our boundary

### 3. Provider Plugins

Dynamic plugin loading for providers.

**Rejected because**:
- Complexity of plugin ABI stability
- Security concerns with dynamic loading
- Harder to audit and certify

## Implementation Notes

### Cloud Providers

| Provider | SDK | Models | Notes |
|----------|-----|--------|-------|
| **Anthropic** | `anthropic-rs` | Claude 3.5, Claude 4 | Primary provider |
| **Azure OpenAI** | `azure-openai-rs` | GPT-4o, GPT-4 | Enterprise compliance |
| **AWS Bedrock** | `aws-sdk-bedrockruntime` | Claude, Llama, Titan | AWS-native |
| **Google Vertex AI** | `gcp-vertex-ai` | Gemini Pro, PaLM | GCP-native |
| **OpenAI** | `async-openai` | GPT-4o, o1 | Fallback |

### Local Providers (Air-Gap)

| Runtime | Protocol | Models | Hardware |
|---------|----------|--------|----------|
| **vLLM** | OpenAI-compatible | Llama 3.1 70B/405B, Qwen2.5 | H100/A100 |
| **TGI** | OpenAI-compatible | Mistral Large 2, Mixtral | H100/A100 |
| **Ollama** | REST | Llama 3.1, Mistral | Any GPU |
| **llama.cpp** | REST | GGUF models | CPU/GPU |

### Performance Targets

| Metric | Cloud | Local | Notes |
|--------|-------|-------|-------|
| P50 latency | <500ms | <1s | First token |
| P99 latency | <2s | <5s | Full response |
| Throughput | 1K req/s | 100 req/s | Per cluster |
| Availability | 99.9% | 99.99% | Air-gap higher |

## Related Decisions

- ADR-012: Air-Gap Deployment Architecture
- ADR-001: Hybrid Signature Strategy (for request signing)
- ADR-008: Inline Authorization (for inference authorization)

## References

- [vLLM Documentation](https://docs.vllm.ai/)
- [Text Generation Inference](https://huggingface.co/docs/text-generation-inference/)
- [Anthropic API Reference](https://docs.anthropic.com/claude/reference/)
- [OpenAI API Reference](https://platform.openai.com/docs/api-reference)
