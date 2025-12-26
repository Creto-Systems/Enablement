---
status: accepted
date: 2025-12-25
deciders: [Architecture Team, ML Infrastructure Team, Security Team]
consulted: [Platform Engineering, DevSecOps, Cost Optimization]
informed: [Agent Development Teams, Operations]
---

# ADR-011: Inference Routing Strategy

## Context

The Enablement Layer Runtime provides infrastructure for autonomous AI agents that require Large Language Model (LLM) inference to operate. Our deployment landscape is heterogeneous:

### Deployment Environments

1. **Cloud-Native Environments**
   - Public cloud (AWS, Azure, GCP) with managed AI services
   - Access to best-in-class models (Claude 3.5 Sonnet, GPT-4o, Gemini 2.0)
   - Pay-per-token pricing with varying costs across providers
   - Network latency to external APIs (50-500ms time-to-first-token)
   - Subject to provider rate limits and quotas

2. **Air-Gapped Environments**
   - Classified networks with zero external connectivity
   - On-premises GPU infrastructure (A100, H100 clusters)
   - Local model hosting via vLLM, TGI, or Ollama
   - Data residency requirements (government, defense, healthcare)
   - No dependency on external service availability

3. **Hybrid Environments**
   - Mix of sensitive and non-sensitive workloads
   - Cost optimization through selective cloud usage
   - Regulatory compliance requiring data classification
   - Fallback scenarios (cloud primary, local backup or vice versa)

### Requirements

The inference routing layer must:

- **Unified API**: Agents should use identical code regardless of deployment mode
- **Security**: Classified data must never transit to cloud providers
- **Performance**: Sub-100ms routing decision, minimal overhead
- **Reliability**: Graceful degradation when providers fail
- **Cost Efficiency**: Route to cheapest provider meeting quality requirements
- **Observability**: Track costs, latency, and quality metrics per provider
- **Flexibility**: Support future providers without agent code changes

### Current State

As of December 2025, the agent runtime prototype directly calls cloud provider SDKs:

```rust
// Current problematic approach
let anthropic_client = AnthropicClient::new(api_key);
let response = anthropic_client.messages.create(request).await?;
```

This creates tight coupling and makes air-gap deployment impossible without code changes.

## Decision Drivers

### 1. Security and Compliance

**Driver**: Classified workloads cannot tolerate data exfiltration risk.

- Defense, intelligence, and healthcare agents handle PII, PHI, CUI, and classified data
- Cloud providers represent external network boundaries
- Regulatory frameworks (ITAR, FedRAMP, HIPAA) mandate data residency
- Even metadata leakage (prompt lengths, token counts) can be sensitive

**Implication**: We need per-request routing decisions based on data classification.

### 2. Cost Optimization

**Driver**: Cloud inference costs vary dramatically across providers.

Current pricing (as of Dec 2025):
- **Claude 3.5 Sonnet**: $3.00/M input, $15.00/M output tokens
- **GPT-4o**: $2.50/M input, $10.00/M output tokens
- **Azure OpenAI (committed)**: $1.80/M input, $7.20/M output tokens
- **Bedrock Claude (bulk)**: $2.40/M input, $12.00/M output tokens
- **Local vLLM (amortized)**: ~$0.10/M tokens (infrastructure cost)

For a system processing 10B tokens/month:
- Most expensive: $150,000/month (Anthropic direct)
- Most cost-effective: $18,000/month (Azure committed + local)
- **Potential savings: $132,000/month (88% reduction)**

**Implication**: Intelligent routing based on cost and quality trade-offs.

### 3. Latency and Responsiveness

**Driver**: Agent execution speed impacts user experience.

Time-to-first-token benchmarks:
- **Anthropic API**: 200-400ms (US East)
- **Azure OpenAI**: 150-300ms (co-located)
- **Bedrock**: 250-500ms (cross-region)
- **Local vLLM (A100)**: 50-150ms (on-premises)
- **Local vLLM (H100)**: 30-80ms (on-premises)

For latency-sensitive agents (interactive chat, real-time decision-making), local inference provides 3-5x improvement.

**Implication**: Latency-optimized routing policies for time-critical workloads.

### 4. Model Capability Differences

**Driver**: Different models excel at different tasks.

Capability matrix:
- **Claude 3.5 Sonnet**: Best reasoning, long context (200K tokens), tool use
- **GPT-4o**: Multimodal (vision), creative writing, broad knowledge
- **Gemini 2.0 Pro**: Code generation, mathematical reasoning
- **Local Llama 3.3 70B**: Cost-effective general tasks, acceptable quality
- **Local CodeLlama 70B**: Code-specific, privacy-preserving

**Implication**: Capability-based routing to match task requirements.

### 5. Availability and Resilience

**Driver**: Single provider outages should not halt operations.

Historical incidents:
- Anthropic API outage (2024-09): 4.2 hours
- OpenAI degradation (2024-11): 6.8 hours
- Azure OpenAI throttling (2024-07): Regional capacity exhaustion

**Implication**: Multi-provider failover and circuit breaker patterns.

## Considered Options

### Option A: Cloud-Only with VPN Tunneling

**Description**: Use cloud providers exclusively, with VPN/proxy for air-gapped environments.

```rust
// Air-gapped environments route through VPN gateway
let client = AnthropicClient::new(api_key)
    .proxy("https://airgap-vpn.example.mil:8443");
```

**Advantages**:
- Simple architecture (one code path)
- Access to best models everywhere
- Minimal local infrastructure

**Disadvantages**:
- ❌ Violates true air-gap requirement (network connection exists)
- ❌ VPN represents security boundary risk
- ❌ High latency (VPN overhead + API roundtrip)
- ❌ Single point of failure (VPN gateway)
- ❌ Does not meet compliance requirements (data still leaves enclave)

**Verdict**: **REJECTED** - Fundamentally incompatible with classified workload requirements.

### Option B: Local-Only with No Cloud Integration

**Description**: Deploy local models everywhere, no cloud provider dependencies.

```rust
// All environments use vLLM
let vllm = VLLMClient::new("http://localhost:8000");
let response = vllm.completions.create(request).await?;
```

**Advantages**:
- ✅ True air-gap support
- ✅ Predictable costs (infrastructure-based)
- ✅ Low latency
- ✅ No external dependencies

**Disadvantages**:
- ❌ Misses best-in-class capabilities (Claude 3.5, GPT-4o)
- ❌ Significant infrastructure investment (GPU clusters)
- ❌ Operational burden (model hosting, updates, monitoring)
- ❌ Quality gap vs. frontier models (10-30% worse on reasoning tasks)
- ❌ Cloud environments overpay for local hosting

**Verdict**: **REJECTED** - Sacrifices too much capability in cloud environments.

### Option C: Separate Codepaths with Feature Flags

**Description**: Different code for cloud vs. air-gap, switched via configuration.

```rust
#[cfg(feature = "cloud-inference")]
let provider = AnthropicClient::new(api_key);

#[cfg(feature = "airgap-inference")]
let provider = VLLMClient::new(endpoint);
```

**Advantages**:
- ✅ Full control over each mode
- ✅ No abstraction overhead
- ✅ Compile-time optimization

**Disadvantages**:
- ❌ Code duplication and maintenance burden
- ❌ Divergent behavior between modes
- ❌ Testing complexity (must validate both paths)
- ❌ Agent code must know about deployment mode
- ❌ No hybrid/fallback scenarios
- ❌ Difficult to migrate between modes

**Verdict**: **REJECTED** - Technical debt and inflexibility outweigh benefits.

### Option D: Unified Provider Abstraction (CHOSEN)

**Description**: Single `InferenceProvider` trait with multiple implementations.

```rust
#[async_trait]
pub trait InferenceProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest)
        -> Result<CompletionResponse>;
    async fn stream(&self, request: CompletionRequest)
        -> Result<Stream<CompletionChunk>>;
    fn capabilities(&self) -> ProviderCapabilities;
    fn cost_estimate(&self, request: &CompletionRequest) -> Cost;
}

// Implementations
struct CloudInferenceRouter { providers: Vec<Box<dyn Provider>> }
struct LocalInferenceProvider { vllm_client: VLLMClient }
struct HybridRouter { cloud: CloudRouter, local: LocalProvider, policy: HybridPolicy }
```

**Advantages**:
- ✅ Single API for all deployment modes
- ✅ Agent code is deployment-agnostic
- ✅ Easy testing (mock implementations)
- ✅ Graceful degradation and failover
- ✅ Cost visibility and optimization
- ✅ Supports future providers (Mistral, Cohere, etc.)
- ✅ Configuration-driven mode selection

**Disadvantages**:
- ⚠️ Abstraction overhead (~1-2ms per request)
- ⚠️ Lowest common denominator for capabilities
- ⚠️ Complex configuration for hybrid scenarios

**Verdict**: **ACCEPTED** - Best balance of flexibility, security, and maintainability.

## Decision

We will implement **Option D: Unified Provider Abstraction** as the inference routing strategy.

### Architecture

```rust
// Core abstraction
#[async_trait]
pub trait InferenceProvider: Send + Sync + 'static {
    async fn complete(&self, request: CompletionRequest) -> Result<CompletionResponse>;
    async fn stream(&self, request: CompletionRequest) -> Result<CompletionStream>;
    fn capabilities(&self) -> ProviderCapabilities;
    fn cost_estimate(&self, request: &CompletionRequest) -> CostEstimate;
    async fn health_check(&self) -> HealthStatus;
}

// Cloud router with failover
pub struct CloudInferenceRouter {
    providers: Vec<Arc<dyn InferenceProvider>>,
    policy: RoutingPolicy,
    circuit_breaker: CircuitBreakerRegistry,
    metrics: MetricsCollector,
}

// Local model provider
pub struct LocalInferenceProvider {
    backend: LocalBackend,  // vLLM, TGI, Ollama
    model_config: ModelConfig,
    gpu_pool: GpuResourcePool,
}

// Hybrid router
pub struct HybridRouter {
    cloud_router: Arc<CloudInferenceRouter>,
    local_provider: Arc<LocalInferenceProvider>,
    classifier: DataClassifier,
    policy: HybridPolicy,
}
```

### Routing Policies

```rust
pub enum RoutingPolicy {
    /// Route to cheapest provider meeting quality/latency requirements
    CostOptimized {
        max_latency_ms: u64,
        min_quality_score: f32,
    },

    /// Route to fastest provider (lowest p99 latency)
    LatencyOptimized {
        max_cost_per_1k_tokens: f32,
    },

    /// Route to most capable model for task
    QualityOptimized {
        max_cost_multiplier: f32,
    },

    /// Round-robin with health-based exclusion
    LoadBalanced {
        weights: HashMap<ProviderId, u32>,
    },

    /// Always use specific provider (compliance requirement)
    ProviderPinned {
        provider: ProviderId,
    },
}

pub enum HybridPolicy {
    /// Prefer local, fallback to cloud on failure/overload
    LocalFirst,

    /// Prefer cloud, fallback to local on failure/rate limit
    CloudFirst,

    /// Route based on data classification tags
    ClassificationBased {
        classified_to_local: bool,
        pii_to_local: bool,
        public_to_cloud: bool,
    },

    /// Route based on capability requirements
    CapabilityBased {
        vision_to_cloud: bool,
        long_context_to_cloud: bool,
        code_gen_to_local: bool,
    },
}
```

### Configuration Example

```yaml
# Cloud-native deployment
inference:
  mode: cloud
  routing_policy:
    type: cost_optimized
    max_latency_ms: 500
    min_quality_score: 0.85
  providers:
    - id: anthropic
      api_key_env: ANTHROPIC_API_KEY
      models: [claude-3-5-sonnet-20241022]
      weight: 3
    - id: azure_openai
      endpoint: https://enablement.openai.azure.com/
      models: [gpt-4o]
      weight: 2
    - id: bedrock
      region: us-east-1
      models: [anthropic.claude-3-5-sonnet-v2]
      weight: 1

# Air-gapped deployment
inference:
  mode: local
  backend: vllm
  endpoint: http://vllm-service.ai-infra.svc.cluster.local:8000
  models:
    - llama-3.3-70b-instruct
    - codellama-70b-instruct
  gpu_allocation:
    min_gpus: 2
    max_gpus: 4

# Hybrid deployment
inference:
  mode: hybrid
  policy:
    type: classification_based
    classified_to_local: true
    pii_to_local: true
    public_to_cloud: true
  cloud:
    routing_policy:
      type: cost_optimized
    providers: [anthropic, azure_openai]
  local:
    backend: vllm
    endpoint: http://localhost:8000
```

### Data Classification Integration

```rust
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub model_preferences: ModelPreferences,
    pub classification: DataClassification,  // NEW
    pub routing_hint: Option<RoutingHint>,   // NEW
}

#[derive(Clone, Copy)]
pub enum DataClassification {
    Public,           // Can use any provider
    Internal,         // Company policy may restrict
    Confidential,     // Restricted providers only
    PII,              // GDPR/CCPA considerations
    PHI,              // HIPAA - must stay local
    Classified,       // Government - must stay air-gapped
}
```

## Consequences

### Positive Consequences

1. **Deployment Flexibility**
   - Same agent code runs in cloud, air-gap, and hybrid modes
   - Configuration-driven deployment (no code changes)
   - Easy migration between modes as requirements evolve

2. **Cost Optimization**
   - Transparent cost tracking per provider
   - Automated routing to cheapest provider meeting requirements
   - Estimated savings: $100K-$150K/month at scale

3. **Security Compliance**
   - Classified data never leaves air-gapped environments
   - Data classification enforcement at runtime
   - Audit trail of routing decisions

4. **Operational Resilience**
   - Automatic failover between providers
   - Circuit breaker prevents cascading failures
   - Graceful degradation (cloud → local fallback)

5. **Testing and Development**
   - Mock provider for unit tests (no API calls)
   - Local development without cloud dependencies
   - Consistent behavior across environments

### Negative Consequences

1. **Abstraction Overhead**
   - Additional 1-2ms latency per request (routing decision)
   - Memory overhead for provider registry and metrics
   - **Mitigation**: Acceptable given 150-500ms API latency baseline

2. **Capability Limitations**
   - Abstraction must support lowest common denominator
   - Provider-specific features require extension points
   - **Mitigation**: `ProviderCapabilities` trait for feature detection

3. **Configuration Complexity**
   - Hybrid policies require careful tuning
   - Misconfiguration could route sensitive data to cloud
   - **Mitigation**: Startup validation, required classification tags

4. **Model Quality Variance**
   - Local models may underperform cloud models (10-30% quality gap)
   - Users must accept trade-off for air-gap deployment
   - **Mitigation**: Quality benchmarking, model selection guidance

### Risks and Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Classification bypass (sensitive data to cloud) | Critical | Low | Mandatory classification tags, runtime validation, audit logging |
| Provider cost spike | High | Medium | Cost caps per provider, automatic circuit breaking at budget threshold |
| All providers fail simultaneously | High | Low | Local fallback provider, cached responses for retry scenarios |
| Abstraction performance bottleneck | Medium | Low | Async provider selection, connection pooling, request batching |
| Configuration drift between environments | Medium | Medium | GitOps deployment, config validation in CI/CD |

## Implementation Plan

### Phase 1: Cloud Router (4 weeks)
- Implement `InferenceProvider` trait
- Build `CloudInferenceRouter` with Anthropic + Azure + Bedrock
- Cost tracking and metrics collection
- Circuit breaker and health checks

**Deliverables**:
- `enablement-inference-core` crate
- `CloudInferenceRouter` with 3 providers
- Prometheus metrics and Grafana dashboards

### Phase 2: Local Provider (3 weeks)
- Implement `LocalInferenceProvider` with vLLM backend
- GPU resource pooling and scheduling
- Model loading and health monitoring
- TGI and Ollama backend support

**Deliverables**:
- vLLM integration
- GPU resource management
- Local deployment documentation

### Phase 3: Hybrid Router (4 weeks)
- Implement `HybridRouter` with classification-based routing
- Data classifier integration
- Policy engine for routing decisions
- Security validation and audit logging

**Deliverables**:
- `HybridRouter` with classification policies
- Security compliance documentation
- Audit trail implementation

### Phase 4: Optimization (2 weeks)
- Advanced routing algorithms (quality prediction, latency modeling)
- Cost optimization heuristics
- Performance benchmarking and tuning
- Production rollout

**Deliverables**:
- Cost optimization reports
- Performance benchmarks
- Production runbook

**Total timeline**: 13 weeks

## Validation Criteria

The implementation is considered successful when:

1. **Functional Requirements**
   - [ ] Agents run unchanged in cloud, air-gap, and hybrid modes
   - [ ] Classified data never routes to cloud providers (zero violations in audit logs)
   - [ ] Failover completes within 5 seconds of provider failure
   - [ ] All providers pass health checks on startup

2. **Performance Requirements**
   - [ ] Routing decision completes in <5ms (p99)
   - [ ] Abstraction overhead <2% of total request latency
   - [ ] Local provider achieves <100ms time-to-first-token

3. **Cost Requirements**
   - [ ] Cloud cost reduction of >50% vs. single-provider baseline
   - [ ] Cost tracking accurate within 5% of actual provider bills

4. **Security Requirements**
   - [ ] Classification enforcement tested with penetration testing
   - [ ] Audit logs capture all routing decisions
   - [ ] FedRAMP compliance validation (for government deployments)

## Related Decisions

- **ADR-007: Sandbox Runtime Selection** - Inference provider selection affects sandbox choice (cloud sandboxes use cloud inference)
- **ADR-012: Air-Gap Deployment Patterns** (future) - Will define local model hosting architecture
- **ADR-013: Cost Attribution Model** (future) - How to charge inference costs back to teams/agents

## References

- [vLLM Documentation](https://docs.vllm.ai/)
- [Anthropic API Pricing](https://www.anthropic.com/pricing)
- [Azure OpenAI Service](https://azure.microsoft.com/en-us/products/ai-services/openai-service)
- [FedRAMP AI Requirements](https://www.fedramp.gov/)
- [NIST AI Risk Management Framework](https://www.nist.gov/itl/ai-risk-management-framework)

## Approval

- **Architecture Team**: Approved (2025-12-25)
- **ML Infrastructure Team**: Approved (2025-12-25)
- **Security Team**: Approved with requirement for classification enforcement testing
- **Cost Optimization Team**: Approved with requirement for monthly cost reporting

---

*Document Version: 1.0*
*Last Updated: 2025-12-25*
*Next Review: 2026-03-25 (quarterly)*
