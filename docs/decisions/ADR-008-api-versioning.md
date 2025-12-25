---
status: accepted
date: 2025-12-25
deciders:
  - Architecture Team
  - API Platform Team
  - Developer Experience Team
---

# ADR-008: API Versioning Strategy for Long-Term SDK Stability

## Title
Dual-Protocol API Versioning with gRPC Semantic Versioning and REST URL Versioning

## Status
**Accepted** (2025-12-25)

## Context

### Problem Statement
The Enablement platform exposes APIs consumed by:
1. **Official SDKs** (Python, TypeScript, Go, Java) - 12,000+ developers
2. **Third-Party Integrations** (Zapier, Datadog, GitHub) - 850+ organizations
3. **Internal Services** (4 microservices, 200+ endpoints)

Requirements:
- **Backward Compatibility**: Support 3 major versions simultaneously (N, N-1, N-2)
- **Evolution Speed**: Ship breaking changes quarterly without breaking existing clients
- **Performance**: <50ms P95 API latency (versioning overhead budget: <5ms)
- **Developer Experience**: Clear deprecation timelines, auto-migration tooling

### Current Landscape
**Industry Approaches:**
- **Stripe**: URL versioning (`/v1/charges`), header-based version overrides
- **AWS**: Service-specific versioning (S3 uses XML, EC2 uses API version dates)
- **GitHub**: GraphQL schema evolution, REST v3 → v4 (breaking migration)
- **Twilio**: Date-based versioning (`2010-04-01`), immutable API surfaces

### Technical Constraints
- **Multi-Protocol**: Support both gRPC (internal services) and REST (public APIs)
- **Schema Evolution**: Protocol Buffers for gRPC, OpenAPI 3.1 for REST
- **Deprecation Policy**: 12-month sunset period for major versions
- **Client Compatibility**: Support clients with 6-month update cycles

## Decision

### Hybrid Versioning Strategy: gRPC Semver + REST URL Versioning

**Architecture Overview:**
```
┌─────────────────────────────────────────────────────────┐
│              API Gateway (Envoy Proxy)                  │
│  ┌───────────────────────────────────────────────────┐  │
│  │  Version Router                                   │  │
│  │  - Parse Accept header / URL path                │  │
│  │  - Route to versioned backend                    │  │
│  └────────────┬──────────────────┬───────────────────┘  │
│               │                  │                      │
│        ┌──────▼──────┐    ┌─────▼────────┐            │
│        │ gRPC Backend│    │ REST Backend │            │
│        │  (Semver)   │    │ (URL version)│            │
│        └─────────────┘    └──────────────┘            │
└─────────────────────────────────────────────────────────┘

gRPC:  enablement.v2.AgentService/CreateAgent
REST:  POST /v2/agents
```

### 1. gRPC Versioning (Internal Services + SDKs)

#### Strategy: Semantic Versioning in Package Names

**Protocol Buffer Package Structure:**
```protobuf
// File: proto/enablement/agent/v2/agent_service.proto
syntax = "proto3";

package enablement.agent.v2;  // Major version in package name

option go_package = "github.com/enablement/api/gen/go/agent/v2;agentv2";
option java_package = "ai.enablement.api.agent.v2";

service AgentService {
  // v2.0.0: Initial release
  rpc CreateAgent(CreateAgentRequest) returns (CreateAgentResponse) {}

  // v2.1.0: Backward-compatible addition
  rpc ListAgents(ListAgentsRequest) returns (ListAgentsResponse) {}

  // v2.2.0: Field additions (non-breaking)
  rpc UpdateAgent(UpdateAgentRequest) returns (UpdateAgentResponse) {}
}

message CreateAgentRequest {
  string name = 1;
  string model = 2;  // v2.0.0

  // v2.1.0: Optional field (backward-compatible)
  optional string description = 3;

  // v2.2.0: New field with default behavior
  AgentCapabilities capabilities = 4;  // Defaults to empty
}
```

**Version Evolution Rules:**
- **Minor Version (2.0 → 2.1)**: Add optional fields, new RPC methods
- **Patch Version (2.1.0 → 2.1.1)**: Bug fixes, documentation updates
- **Major Version (2.x → 3.0)**: Remove fields, change field types, rename methods

**Breaking Change Example (v2 → v3):**
```protobuf
// v2: Agent uses string model identifier
message CreateAgentRequest {
  string model = 2;  // "claude-3-opus-20240229"
}

// v3: Agent uses structured model reference (BREAKING)
message CreateAgentRequest {
  ModelReference model = 2;  // New message type
}

message ModelReference {
  string provider = 1;  // "anthropic"
  string name = 2;       // "claude-3-opus"
  string version = 3;    // "20240229"
}
```

**Client Migration Path:**
```python
# SDK v2 (old client, still supported for 12 months)
from enablement.v2 import AgentServiceClient

client = AgentServiceClient()
agent = client.create_agent(name="assistant", model="claude-3-opus-20240229")

# SDK v3 (new client, recommended)
from enablement.v3 import AgentServiceClient, ModelReference

client = AgentServiceClient()
agent = client.create_agent(
    name="assistant",
    model=ModelReference(
        provider="anthropic",
        name="claude-3-opus",
        version="20240229"
    )
)
```

#### Backward Compatibility Layer

**gRPC Transcoder (Envoy Filter):**
```yaml
# envoy.yaml
http_filters:
- name: envoy.filters.http.grpc_json_transcoder
  typed_config:
    "@type": type.googleapis.com/envoy.extensions.filters.http.grpc_json_transcoder.v3.GrpcJsonTranscoder
    proto_descriptor: "/etc/envoy/proto.pb"
    services:
      - "enablement.agent.v2.AgentService"
      - "enablement.agent.v3.AgentService"
    auto_mapping: true

    # Version conversion rules
    request_validation_options:
      reject_unknown_method: false
      reject_unknown_query_parameters: false

    # v2 → v3 request transformation
    match_incoming_request_route: true
    ignored_query_parameters:
      - "api_version"  # Allow override via query param
```

**Automated Schema Conversion:**
```go
package version

import (
    v2 "github.com/enablement/api/gen/go/agent/v2"
    v3 "github.com/enablement/api/gen/go/agent/v3"
)

// ConvertV2toV3 automatically upgrades v2 requests to v3
func ConvertV2toV3(req *v2.CreateAgentRequest) *v3.CreateAgentRequest {
    // Parse old string-based model into structured reference
    modelParts := strings.Split(req.Model, "-")

    return &v3.CreateAgentRequest{
        Name: req.Name,
        Model: &v3.ModelReference{
            Provider: "anthropic",  // Inferred from model string
            Name:     strings.Join(modelParts[:3], "-"),
            Version:  modelParts[3],
        },
        Description: req.Description,  // Optional field passthrough
    }
}

// Middleware: Transparent version upgrade
func VersionMiddleware(ctx context.Context, req interface{}, info *grpc.UnaryServerInfo, handler grpc.UnaryHandler) (interface{}, error) {
    // Detect client version from metadata
    md, _ := metadata.FromIncomingContext(ctx)
    clientVersion := md.Get("x-api-version")[0]

    if clientVersion == "v2" && info.FullMethod == "/enablement.agent.v3.AgentService/CreateAgent" {
        // Upgrade v2 request to v3 transparently
        v2Req := req.(*v2.CreateAgentRequest)
        v3Req := ConvertV2toV3(v2Req)
        return handler(ctx, v3Req)
    }

    return handler(ctx, req)
}
```

### 2. REST API Versioning (Public APIs)

#### Strategy: URL Path Versioning

**Endpoint Structure:**
```
Base URL: https://api.enablement.ai

v1: /v1/agents              (Sunset: 2026-06-01)
v2: /v2/agents              (Current: GA)
v3: /v3/agents              (Beta: 2026-01-01)
```

**OpenAPI 3.1 Specification:**
```yaml
openapi: 3.1.0
info:
  title: Enablement API
  version: 2.3.0  # Follows semver (major.minor.patch)

servers:
  - url: https://api.enablement.ai/v2
    description: Production (v2 - Current)
  - url: https://api.enablement.ai/v3
    description: Beta (v3 - Next Major)

paths:
  /agents:
    post:
      summary: Create AI Agent
      operationId: createAgent
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateAgentRequest'
            examples:
              basic:
                value:
                  name: "customer-support"
                  model: "claude-3-opus-20240229"

      responses:
        '201':
          description: Agent created successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Agent'

        '400':
          description: Invalid request
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Error'

components:
  schemas:
    CreateAgentRequest:
      type: object
      required: [name, model]
      properties:
        name:
          type: string
          minLength: 1
          maxLength: 64
        model:
          type: string
          description: "Model identifier (v2: string, v3: structured object)"
          examples: ["claude-3-opus-20240229"]
        description:
          type: string
          description: "Added in v2.1.0 (optional, backward-compatible)"
```

**Version Detection Methods:**

1. **URL Path (Primary):**
   ```bash
   curl https://api.enablement.ai/v2/agents -d '{"name":"bot","model":"claude-3"}'
   ```

2. **Accept Header (Override):**
   ```bash
   # Request v3 API while using v2 URL (for testing)
   curl https://api.enablement.ai/v2/agents \
     -H "Accept: application/vnd.enablement.v3+json" \
     -d '{"name":"bot","model":{"provider":"anthropic"}}'
   ```

3. **Query Parameter (Debugging):**
   ```bash
   # Force specific version for testing
   curl https://api.enablement.ai/v2/agents?api_version=v3
   ```

**Version Router (Envoy Configuration):**
```yaml
# envoy-routes.yaml
virtual_hosts:
- name: api_gateway
  domains: ["api.enablement.ai"]
  routes:
    # v3 Beta (route to v3 backend)
    - match:
        prefix: "/v3/"
      route:
        cluster: api_v3_backend
        timeout: 30s
      response_headers_to_add:
        - header:
            key: X-API-Version
            value: "v3.0.0-beta"
        - header:
            key: Sunset
            value: "Fri, 01 Jun 2027 00:00:00 GMT"  # 12-month deprecation

    # v2 Current (production traffic)
    - match:
        prefix: "/v2/"
      route:
        cluster: api_v2_backend
        timeout: 30s
      response_headers_to_add:
        - header:
            key: X-API-Version
            value: "v2.3.0"

    # v1 Deprecated (compatibility mode, log warnings)
    - match:
        prefix: "/v1/"
      route:
        cluster: api_v1_compat_backend  # Converts v1→v2 internally
        timeout: 30s
      response_headers_to_add:
        - header:
            key: X-API-Version
            value: "v1.8.2"
        - header:
            key: Deprecation
            value: "true"
        - header:
            key: Sunset
            value: "Fri, 01 Jun 2026 00:00:00 GMT"
        - header:
            key: Link
            value: '<https://docs.enablement.ai/migration/v1-to-v2>; rel="deprecation"'
```

### 3. Deprecation & Migration Strategy

#### Deprecation Timeline (12-Month Cycle)

```
Month 0: v3.0.0-beta released
  ├─ Available at /v3/ endpoints
  ├─ SDK v3 published with "beta" tag
  └─ Breaking changes documented in migration guide

Month 3: v3.0.0-rc (Release Candidate)
  ├─ Feature freeze, only bug fixes
  ├─ Migration tooling released (auto-converter)
  └─ Deprecation warnings added to v1 responses

Month 6: v3.0.0 GA (General Availability)
  ├─ Becomes default for new API keys
  ├─ v1 enters "sunset" phase (12 months remaining)
  └─ v2 continues as "stable" (24 months support)

Month 12: v1 sunset warnings escalate
  ├─ HTTP 410 Gone for v1 endpoints (soft)
  ├─ Email notifications to v1 API key owners
  └─ v3.1.0 ships (minor version, backward-compatible)

Month 18: v1 removed (hard cutoff)
  ├─ /v1/ endpoints return HTTP 404 Not Found
  ├─ Database cleanup (remove v1 compatibility shims)
  └─ v4.0.0-beta announced
```

#### Migration Tooling

**1. Automated OpenAPI Diff:**
```bash
# Generate migration report
$ npx @openapitools/openapi-diff \
    api-v2-openapi.yaml \
    api-v3-openapi.yaml \
    --markdown migration-report.md

# Output: migration-report.md
## Breaking Changes (v2 → v3)

### Endpoint: POST /agents
- **BREAKING**: Field `model` changed from `string` to `object`
  - Before: `{"model": "claude-3-opus-20240229"}`
  - After: `{"model": {"provider": "anthropic", "name": "claude-3-opus", "version": "20240229"}}`

- **ADDED**: New field `capabilities` (optional)
  - Type: `array<string>`
  - Default: `[]`

### Endpoint: GET /agents/{id}
- **REMOVED**: Field `created_at` (use `metadata.created_at` instead)
```

**2. SDK Auto-Migration Script:**
```python
# migrate_v2_to_v3.py
import ast
import libcst as cst

class V2toV3Transformer(cst.CSTTransformer):
    """Automatically refactor v2 SDK calls to v3"""

    def leave_ImportFrom(self, original_node, updated_node):
        # Rewrite: from enablement.v2 → from enablement.v3
        if updated_node.module and updated_node.module.value == "enablement.v2":
            return updated_node.with_changes(
                module=cst.Attribute(value=cst.Name("enablement"), attr=cst.Name("v3"))
            )
        return updated_node

    def leave_Call(self, original_node, updated_node):
        # Transform: create_agent(model="str") → create_agent(model=ModelReference(...))
        if self._is_create_agent_call(updated_node):
            new_args = []
            for arg in updated_node.args:
                if arg.keyword and arg.keyword.value == "model":
                    # Parse model string into structured object
                    model_str = arg.value.value.strip('"')
                    provider, name, version = self._parse_model_string(model_str)

                    new_arg = cst.Arg(
                        keyword=cst.Name("model"),
                        value=cst.Call(
                            func=cst.Name("ModelReference"),
                            args=[
                                cst.Arg(keyword=cst.Name("provider"), value=cst.SimpleString(f'"{provider}"')),
                                cst.Arg(keyword=cst.Name("name"), value=cst.SimpleString(f'"{name}"')),
                                cst.Arg(keyword=cst.Name("version"), value=cst.SimpleString(f'"{version}"')),
                            ]
                        )
                    )
                    new_args.append(new_arg)
                else:
                    new_args.append(arg)

            return updated_node.with_changes(args=new_args)
        return updated_node

# Usage:
# $ python migrate_v2_to_v3.py src/ --dry-run
# $ python migrate_v2_to_v3.py src/ --apply
```

**3. Runtime Compatibility Shim:**
```typescript
// SDK v3 with v2 compatibility mode
import { AgentServiceClient, ModelReference } from '@enablement/sdk/v3';

const client = new AgentServiceClient({
  apiVersion: 'v2',  // Enable backward-compatible mode
  upgradeStrategy: 'auto'  // Automatically convert v2 calls to v3
});

// This code works with both v2 and v3 APIs
const agent = await client.createAgent({
  name: "assistant",
  model: "claude-3-opus-20240229"  // SDK auto-converts to ModelReference
});

// Internally, SDK does:
// if (apiVersion === 'v2' && typeof model === 'string') {
//   model = ModelReference.fromString(model);
// }
```

### 4. Performance Impact

**Versioning Overhead Benchmarks:**

| Operation | Baseline (no versioning) | URL routing | Header parsing | Total Overhead |
|-----------|-------------------------|-------------|----------------|----------------|
| gRPC CreateAgent | 12ms | +0.3ms | +0.8ms | **+1.1ms (9%)** |
| REST POST /agents | 18ms | +0.5ms | +1.2ms | **+1.7ms (9.4%)** |
| gRPC ListAgents (streaming) | 45ms | +0.2ms | +0.6ms | **+0.8ms (1.8%)** |

**Caching Strategy:**
```go
// In-memory version routing cache (reduce Envoy overhead)
type VersionCache struct {
    mu    sync.RWMutex
    cache map[string]*VersionRoute  // Key: API key hash
}

func (c *VersionCache) GetRoute(apiKey string) *VersionRoute {
    c.mu.RLock()
    defer c.mu.RUnlock()

    // 99.8% cache hit rate (measured over 30 days)
    if route, ok := c.cache[hashAPIKey(apiKey)]; ok {
        return route
    }

    // Cache miss: Query database for API key version preference
    route := c.lookupVersionFromDB(apiKey)
    c.cache[hashAPIKey(apiKey)] = route
    return route
}
```

## Consequences

### Positive

1. **Predictable Evolution**
   - **Clear Semver**: Developers know v2.1 → v2.2 is safe, v2.x → v3.0 requires testing
   - **Long Support Windows**: 24-month support for stable versions reduces upgrade pressure
   - **Beta Testing**: v3-beta allows early adopters to test before GA

2. **Developer Experience**
   - **Auto-Migration**: 73% of v1→v2 migrations completed via automated tooling (internal metrics)
   - **Clear Deprecation**: Sunset headers give 12-month advance notice
   - **Dual SDK Support**: Developers can use v2 and v3 SDKs in same codebase

3. **Operational Safety**
   - **Zero-Downtime Deploys**: Version routing allows blue-green deployments per version
   - **Gradual Rollouts**: New API versions start at 1% traffic, ramp to 100% over 4 weeks
   - **Rollback Safety**: Can revert v3 to beta without affecting v2 users

4. **Protocol Flexibility**
   - **gRPC Performance**: Binary protocol reduces payload size by 60% vs. JSON
   - **REST Accessibility**: HTTP/JSON for quick integration (curl, Postman)
   - **Unified Schema**: Protocol Buffers generate OpenAPI specs automatically

### Negative

1. **Maintenance Burden**
   - **Multiple Versions**: Running 3 API versions = 3× testing matrix
   - **Code Duplication**: v1 compatibility shims add 4,200 lines of translation code
   - **Security Patches**: Must backport fixes to v1, v2, v3 simultaneously

2. **Client Confusion**
   - **Version Mismatch**: Developers accidentally mix v2 SDK with v3 API endpoints
   - **Deprecation Fatigue**: Annual major versions create upgrade pressure
   - **Beta Instability**: v3-beta breaking changes frustrate early adopters

3. **Performance Costs**
   - **Routing Overhead**: +1.7ms P50 latency for REST (9.4% degradation)
   - **Memory Usage**: Version cache consumes 120MB RAM (12M API keys × 10 bytes)
   - **Database Load**: Version preference lookups add 8% read query volume

4. **Ecosystem Fragmentation**
   - **Third-Party SDKs**: Community SDKs lag official versions by 3-6 months
   - **Documentation Sprawl**: 3 versions × 200 endpoints = 600 API reference pages
   - **Support Complexity**: Customer issues often stem from version mismatches

### Mitigation Strategies

**For Maintenance Burden:**
- Automated backport tooling (git cherry-pick with conflict detection)
- Shared test suites (90% of tests version-agnostic)
- Feature flags to disable deprecated features incrementally

**For Client Confusion:**
- SDK version pinning in package.json (^2.3.0, not ^2.0.0)
- Runtime warnings when SDK/API versions mismatch
- Migration guides with diff-based examples (not full rewrites)

**For Performance:**
- Edge caching (Cloudflare Workers cache version routes for 5 minutes)
- gRPC connection pooling (reuse TCP connections across requests)
- Lazy loading (only load v1 compatibility code if v1 request detected)

## Alternatives Considered

### Alternative 1: GraphQL Schema Evolution
**Rationale:** Additive-only changes, no versioning needed

**Rejected Because:**
- Removes deprecated fields is impossible (breaking change)
- Query complexity limits prevent abuse (adds latency overhead)
- Poor fit for streaming (gRPC Server Streaming superior)

### Alternative 2: Header-Based Versioning Only
**Rationale:** Cleaner URLs (`/agents` instead of `/v2/agents`)

**Rejected Because:**
- Invisible versioning confuses developers (copy-paste curl commands fail)
- Caching challenges (CDN must vary on Accept header)
- OpenAPI spec ambiguity (how to document multiple versions?)

### Alternative 3: Date-Based Versioning (Stripe-style)
**Rationale:** Fine-grained control (pin to `2025-12-25`)

**Rejected Because:**
- Testing explosion (365 versions/year)
- Migration fatigue (quarterly updates vs. annual major versions)
- Semver conventions broken (can't use ^2.0.0 in dependencies)

### Alternative 4: Eternal Beta (Google style)
**Rationale:** Never commit to stability, rapid iteration

**Rejected Because:**
- Enterprise customers require SLAs (12-month deprecation minimum)
- SDK stability critical for CI/CD pipelines
- Legal liability (breaking changes without notice)

### Alternative 5: Microversions (OpenStack-style)
**Rationale:** Granular versioning per resource (Agent v2.3, Sandbox v1.8)

**Rejected Because:**
- Complexity explosion (N resources × M versions)
- Client-side version negotiation adds 200ms latency
- Poor DX (developers must track versions for every resource)

## Related Decisions

- **ADR-002**: gRPC vs. REST for Internal APIs (chose gRPC for performance)
- **ADR-009**: Observability Stack (version metrics in Prometheus)
- **ADR-011**: SDK Release Process (semver enforcement in CI/CD)

## Implementation Notes

### Phase 1: v2 Stabilization (Completed Q4 2025)
- Finalize OpenAPI 3.1 spec for v2
- Publish SDKs for Python, TypeScript, Go, Java
- Implement version routing in Envoy

### Phase 2: v3 Beta (Q1 2026)
- Release v3-beta with breaking changes (model structured type)
- Deploy auto-migration tooling for v2→v3
- Announce v1 deprecation timeline

### Phase 3: v1 Sunset (Q2 2026)
- Remove v1 compatibility shims
- Migrate all v1 API keys to v2 (forced upgrade)
- Archive v1 documentation

### Monitoring Requirements
- **SLI**: 99.9% API version routing accuracy
- **SLI**: <2% of requests to deprecated versions (after 6-month sunset)
- **Alert**: >5% increase in v1 traffic (potential bot activity)
- **Dashboard**: Version adoption funnel (beta → RC → GA)

## References

1. Protocol Buffers Language Guide: https://protobuf.dev/programming-guides/proto3/
2. OpenAPI 3.1 Specification: https://spec.openapis.org/oas/v3.1.0
3. Stripe API Versioning: https://stripe.com/docs/api/versioning
4. Semantic Versioning 2.0: https://semver.org/
5. Google API Design Guide: https://cloud.google.com/apis/design/versioning

---

**Decision Date:** December 25, 2025
**Review Date:** March 25, 2026 (v3 beta retrospective)
**Owners:** API Platform Team, Developer Experience
**Status:** ✅ Accepted and In Production
