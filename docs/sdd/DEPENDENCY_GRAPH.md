# Creto Enablement Layer - Dependency Graph & Critical Path Analysis

**Version:** 1.0.0
**Date:** 2025-12-26
**Status:** Draft

---

## 1. External Dependencies (Sibling Repos)

```
CRETO PLATFORM DEPENDENCY TREE

creto-enablement (this repo - orchestration layer)
├── creto-metering (usage tracking & quota enforcement)
│   ├── creto-authz (168ns quota enforcement)
│   │   └── Required for: Real-time usage checks
│   ├── creto-storage (event persistence)
│   │   └── Required for: Durable event log
│   ├── creto-nhi (agent identity)
│   │   └── Required for: Usage attribution
│   ├── creto-crypto (ML-DSA signatures)
│   │   └── Required for: Signed usage records
│   ├── creto-consensus (ordering)
│   │   └── Required for: Consistent event ordering
│   └── creto-audit (Merkle logs)
│       └── Required for: Tamper-proof audit trail
│
├── creto-oversight (approval workflows & policy enforcement)
│   ├── creto-authz (policy triggers)
│   │   └── Required for: Policy decision points
│   ├── creto-memory (agent reasoning)
│   │   └── Required for: Context-aware approvals
│   ├── creto-nhi (delegation chains)
│   │   └── Required for: Approval authority verification
│   ├── creto-crypto (approval signatures)
│   │   └── Required for: Non-repudiation of approvals
│   └── creto-audit (approval logs)
│       └── Required for: Audit trail of decisions
│
├── creto-runtime (sandbox execution environment)
│   ├── creto-authz (egress enforcement)
│   │   └── Required for: Network/API access control
│   ├── creto-vault (secret injection)
│   │   └── Required for: Secure credential delivery
│   ├── creto-nhi (sandbox binding)
│   │   └── Required for: Process-to-identity mapping
│   ├── creto-crypto (attestation)
│   │   └── Required for: Execution attestation
│   └── creto-audit (execution logs)
│       └── Required for: Runtime audit trail
│
└── creto-messaging (agent-to-agent communication)
    ├── creto-authz (delivery policy)
    │   └── Required for: Message authorization
    ├── creto-storage (message persistence)
    │   └── Required for: Durable message queue
    ├── creto-nhi (sender/receiver identity)
    │   └── Required for: E2E identity binding
    ├── creto-crypto (X3DH, Double Ratchet)
    │   └── Required for: E2E encryption
    └── creto-audit (delivery logs)
        └── Required for: Message delivery proof
```

### External Dependency Details

| Repository | Purpose | Critical Interfaces | Latency Requirement |
|------------|---------|---------------------|---------------------|
| **creto-authz** | Authorization decisions | `CheckQuota()`, `EvaluatePolicy()` | 168ns (hot path) |
| **creto-storage** | Event/message persistence | `WriteEvent()`, `ReadEvents()` | 1ms (SSD) |
| **creto-nhi** | Non-Human Identity | `GetIdentity()`, `VerifyDelegation()` | 500ns (cache) |
| **creto-crypto** | Cryptographic operations | `Sign()`, `Verify()`, `Encrypt()` | 10µs (ML-DSA) |
| **creto-consensus** | Event ordering | `ProposeEvent()`, `GetSequence()` | 10ms (Raft) |
| **creto-audit** | Tamper-proof logging | `AppendLog()`, `VerifyProof()` | 5ms (Merkle) |
| **creto-vault** | Secret management | `GetSecret()`, `RenewLease()` | 2ms (unsealed) |
| **creto-memory** | Agent context | `StoreContext()`, `RetrieveContext()` | 1ms (vector DB) |

---

## 2. Internal Dependencies (Build Order)

```mermaid
graph TD
    %% Phase 1: Foundation (Week 1-2)
    common[creto-enablement-common<br/>Types, Interfaces, Utils] --> metering[creto-metering<br/>Usage Tracking]
    common --> oversight[creto-oversight<br/>Approval Workflows]
    common --> runtime[creto-runtime<br/>Sandbox Execution]
    common --> messaging[creto-messaging<br/>Agent Communication]

    %% Phase 2: Core Services (Week 3-4)
    metering --> oversight
    metering --> runtime
    oversight --> runtime
    runtime --> messaging

    %% Phase 3: Orchestration (Week 5)
    metering --> orchestrator[creto-enablement-orchestrator<br/>Workflow Engine]
    oversight --> orchestrator
    runtime --> orchestrator
    messaging --> orchestrator

    %% Phase 4: Demos (Week 6-8)
    orchestrator --> demo1[Demo 1: Trading Agent<br/>Metering + Oversight]
    orchestrator --> demo2[Demo 2: Travel Agent<br/>Metering + Runtime + Messaging]
    orchestrator --> demo3[Demo 3: Healthcare<br/>Oversight + Runtime + Audit]
    orchestrator --> demo4[Demo 4: PSA<br/>All Components]

    %% Styling
    classDef foundation fill:#e1f5ff,stroke:#0288d1,stroke-width:2px
    classDef core fill:#fff3e0,stroke:#f57c00,stroke-width:2px
    classDef orchestration fill:#f3e5f5,stroke:#7b1fa2,stroke-width:2px
    classDef demo fill:#e8f5e9,stroke:#388e3c,stroke-width:2px

    class common foundation
    class metering,oversight,runtime,messaging core
    class orchestrator orchestration
    class demo1,demo2,demo3,demo4 demo
```

### Internal Module Dependencies

| Module | Depends On | Blocks | Build Time | Test Time |
|--------|-----------|--------|------------|-----------|
| `creto-enablement-common` | None | All modules | 5 min | 2 min |
| `creto-metering` | `common`, `creto-authz` | `oversight`, `runtime`, demos | 15 min | 10 min |
| `creto-oversight` | `common`, `metering`, `creto-authz` | `runtime`, demos | 20 min | 15 min |
| `creto-runtime` | `common`, `metering`, `oversight`, `creto-vault` | `messaging`, demos | 25 min | 20 min |
| `creto-messaging` | `common`, `runtime`, `creto-crypto` | Demos | 15 min | 12 min |
| `creto-enablement-orchestrator` | All core modules | Demos | 10 min | 8 min |
| Demo 1 (Trading) | `metering`, `oversight` | None | 5 min | 5 min |
| Demo 2 (Travel) | `metering`, `runtime`, `messaging` | None | 5 min | 5 min |
| Demo 3 (Healthcare) | `oversight`, `runtime` | None | 5 min | 5 min |
| Demo 4 (PSA) | All modules | None | 5 min | 10 min |

---

## 3. Critical Path Analysis

### 3.1 Longest Dependency Chain

**Total Critical Path Duration: 6-8 weeks**

```
Week 1-2: Foundation
creto-enablement-common (5 days)
└─> creto-metering (7 days)
    └─> creto-oversight (7 days)
        └─> creto-runtime (7 days)
            └─> creto-messaging (5 days)
                └─> creto-enablement-orchestrator (3 days)
                    └─> Demo 4: PSA (5 days)
```

### 3.2 Critical Path Bottlenecks

| Bottleneck | Impact | Mitigation Strategy |
|------------|--------|---------------------|
| **creto-authz latency (168ns)** | Blocks metering hot path | Mock with in-memory cache, parallel optimization |
| **creto-vault availability** | Blocks runtime secret injection | Mock with static secrets, implement retry logic |
| **creto-consensus ordering** | Blocks distributed metering | Use local ordering for v1, add consensus in v2 |
| **creto-crypto ML-DSA** | Blocks signature verification | Pre-generate test keypairs, use libsodium for mocks |
| **creto-runtime sandbox isolation** | Blocks execution safety | Use process isolation for v1, WebAssembly for v2 |

### 3.3 What Blocks What (Detailed)

```
creto-enablement-common BLOCKS:
├── creto-metering (needs common types)
├── creto-oversight (needs common types)
├── creto-runtime (needs common types)
└── creto-messaging (needs common types)

creto-metering BLOCKS:
├── creto-oversight (needs usage data for policy triggers)
├── creto-runtime (needs quota checks before execution)
└── Demo 1, Demo 2, Demo 4

creto-oversight BLOCKS:
├── creto-runtime (needs approval before execution)
└── Demo 1, Demo 3, Demo 4

creto-runtime BLOCKS:
├── creto-messaging (needs execution context)
└── Demo 2, Demo 3, Demo 4

creto-messaging BLOCKS:
└── Demo 2, Demo 4

creto-enablement-orchestrator BLOCKS:
└── All demos (orchestrates workflows)
```

---

## 4. Parallel Execution Opportunities

### 4.1 Phase 1: Foundation (Week 1-2)

**Can Build in Parallel:**
- `creto-enablement-common` (blocking)
- Mock implementations of all external dependencies
- CI/CD pipeline setup
- Documentation scaffolding

**Team Allocation:**
- Team A: `creto-enablement-common` + mock infrastructure
- Team B: CI/CD + repository structure
- Team C: External dependency analysis + integration contracts

### 4.2 Phase 2: Core Services (Week 3-4)

**Can Build in Parallel (after `common` completes):**
- `creto-metering` + `creto-messaging` (independent)
- `creto-oversight` (depends on `metering` mock)

**Sequential Constraint:**
- `creto-runtime` must wait for `metering` and `oversight` interfaces

**Team Allocation:**
- Team A: `creto-metering` (full implementation)
- Team B: `creto-messaging` (full implementation)
- Team C: `creto-oversight` (starts after `metering` API is defined)

### 4.3 Phase 3: Integration (Week 5)

**Can Build in Parallel:**
- `creto-enablement-orchestrator` (after core services)
- Demo 1 (Trading) - only needs `metering` + `oversight`
- Demo 2 (Travel) - only needs `metering` + `runtime` + `messaging`

**Sequential Constraint:**
- Demo 3 and Demo 4 need all components

**Team Allocation:**
- Team A: `creto-enablement-orchestrator`
- Team B: Demo 1 (Trading Agent)
- Team C: Demo 2 (Travel Agent)

### 4.4 Phase 4: Demos (Week 6-8)

**Can Build in Parallel:**
- Demo 3 (Healthcare)
- Demo 4 (PSA)
- Performance optimization
- Documentation finalization

**Team Allocation:**
- Team A: Demo 3 + performance tuning
- Team B: Demo 4 + integration testing
- Team C: Documentation + developer experience

---

## 5. Integration Points

### 5.1 External Integration Contracts

| Integration | Provider | Consumer | Contract | SLA | Risk Level |
|-------------|----------|----------|----------|-----|------------|
| **Quota Check** | `creto-authz` | `creto-metering` | `CheckQuota(agentId, resource) -> (allowed, remaining)` | 168ns p99 | 🔴 **High** |
| **Oversight Trigger** | `creto-authz` | `creto-oversight` | `EvaluatePolicy(context, action) -> (approve, deny, escalate)` | 1ms p99 | 🟡 **Medium** |
| **Secret Injection** | `creto-vault` | `creto-runtime` | `GetSecret(path, lease) -> (secret, ttl)` | 2ms p99 | 🔴 **High** |
| **Message Delivery** | `creto-authz` | `creto-messaging` | `CheckDelivery(senderId, recipientId) -> (allowed, policy)` | 500µs p99 | 🟡 **Medium** |
| **Identity Verification** | `creto-nhi` | All modules | `VerifyIdentity(token) -> (agentId, claims)` | 500ns p99 | 🔴 **High** |
| **Event Signature** | `creto-crypto` | `creto-metering` | `Sign(event, privKey) -> signature` | 10µs p99 | 🟡 **Medium** |
| **Event Ordering** | `creto-consensus` | `creto-metering` | `ProposeEvent(event) -> sequence` | 10ms p99 | 🟡 **Medium** |
| **Audit Append** | `creto-audit` | All modules | `AppendLog(entry, proof) -> merkleRoot` | 5ms p99 | 🟢 **Low** |

### 5.2 Internal Integration Contracts

| Integration | Provider | Consumer | Contract | Risk Level |
|-------------|----------|----------|----------|------------|
| **Usage Recording** | `creto-metering` | `creto-runtime` | `RecordUsage(agentId, resource, amount)` | 🟢 **Low** |
| **Approval Request** | `creto-oversight` | `creto-runtime` | `RequestApproval(context, action) -> approvalToken` | 🟡 **Medium** |
| **Message Send** | `creto-messaging` | `creto-runtime` | `SendMessage(senderId, recipientId, payload)` | 🟢 **Low** |
| **Workflow Orchestration** | `orchestrator` | All modules | `ExecuteWorkflow(workflowId, context) -> result` | 🟡 **Medium** |

### 5.3 Risk Mitigation Strategies

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| **creto-authz unavailable** | Metering/oversight blocked | Medium | Circuit breaker, fail-open with audit |
| **creto-vault sealed** | Runtime secret injection fails | Low | Pre-provision secrets, retry with backoff |
| **creto-consensus split-brain** | Event ordering inconsistent | Low | Use single-node for v1, add consensus v2 |
| **creto-crypto key rotation** | Signature verification fails | Medium | Support multiple active keys, grace period |
| **Network partition** | Cross-module communication fails | Medium | Local caching, eventual consistency |

---

## 6. Version Compatibility Matrix

### 6.1 External Repository Compatibility

| Repo | Required Version | Compatible Versions | Breaking Changes | Notes |
|------|------------------|---------------------|------------------|-------|
| **creto-authz** | v1.2.0+ | v1.2.x, v1.3.x | v2.0.0 changes policy DSL | Pin to v1.3.x for stability |
| **creto-storage** | v2.1.0+ | v2.1.x - v2.5.x | v3.0.0 changes event schema | Use v2.3.x (most stable) |
| **creto-nhi** | v1.0.0+ | v1.x.x | None expected | New repo, stable API |
| **creto-crypto** | v0.9.0+ | v0.9.x, v1.0.x | v2.0.0 adds post-quantum | Use v1.0.x for ML-DSA |
| **creto-consensus** | v1.1.0+ | v1.1.x - v1.4.x | v2.0.0 changes leader election | Pin to v1.3.x |
| **creto-audit** | v1.0.0+ | v1.0.x - v1.2.x | v2.0.0 changes Merkle tree | Use v1.1.x |
| **creto-vault** | v1.5.0+ | v1.5.x - v1.8.x | v2.0.0 changes lease API | Use v1.7.x (transit engine) |
| **creto-memory** | v0.8.0+ | v0.8.x - v1.0.x | v1.0.0 stable release | Use v0.9.x (vector DB) |

### 6.2 Internal Module Compatibility

| Module | Version | Depends On | Blocks |
|--------|---------|-----------|--------|
| `creto-enablement-common` | v1.0.0 | None | All modules v1.0.0 |
| `creto-metering` | v1.0.0 | `common@1.0.0`, `creto-authz@1.3.x` | `oversight@1.0.0`, `runtime@1.0.0` |
| `creto-oversight` | v1.0.0 | `common@1.0.0`, `metering@1.0.0`, `creto-authz@1.3.x` | `runtime@1.0.0` |
| `creto-runtime` | v1.0.0 | `common@1.0.0`, `metering@1.0.0`, `oversight@1.0.0`, `creto-vault@1.7.x` | `messaging@1.0.0` |
| `creto-messaging` | v1.0.0 | `common@1.0.0`, `runtime@1.0.0`, `creto-crypto@1.0.x` | Demos |
| `creto-enablement-orchestrator` | v1.0.0 | All modules `@1.0.0` | Demos |

### 6.3 Dependency Lock Strategy

```json
{
  "dependencies": {
    "@creto/authz": "~1.3.0",
    "@creto/storage": "~2.3.0",
    "@creto/nhi": "~1.0.0",
    "@creto/crypto": "~1.0.0",
    "@creto/consensus": "~1.3.0",
    "@creto/audit": "~1.1.0",
    "@creto/vault": "~1.7.0",
    "@creto/memory": "~0.9.0"
  },
  "peerDependencies": {
    "@creto/common": "^1.0.0"
  },
  "engines": {
    "node": ">=18.0.0",
    "npm": ">=9.0.0"
  }
}
```

**Version Policy:**
- Use `~` (tilde) for patch updates (e.g., `~1.3.0` → `1.3.x`)
- Use `^` (caret) for internal modules (e.g., `^1.0.0` → `1.x.x`)
- Pin exact versions for cryptographic libraries
- Quarterly dependency audit and update

---

## 7. Mock Strategy

### 7.1 Mock Implementation Approach

**Principle:** "Mock the boundary, not the behavior"

```typescript
// Mock interface matches real interface exactly
interface ICretoAuthz {
  checkQuota(agentId: string, resource: string): Promise<QuotaResponse>;
  evaluatePolicy(context: Context, action: Action): Promise<PolicyDecision>;
}

// Production implementation
class CretoAuthzClient implements ICretoAuthz {
  async checkQuota(agentId: string, resource: string): Promise<QuotaResponse> {
    // gRPC call to creto-authz service
  }
}

// Mock implementation
class CretoAuthzMock implements ICretoAuthz {
  private quotas = new Map<string, number>();

  async checkQuota(agentId: string, resource: string): Promise<QuotaResponse> {
    const remaining = this.quotas.get(`${agentId}:${resource}`) ?? 1000;
    return { allowed: remaining > 0, remaining };
  }
}
```

### 7.2 Mock Implementations by External Dependency

#### 7.2.1 creto-authz Mock

**Interface:**
```typescript
interface ICretoAuthz {
  checkQuota(agentId: string, resource: string): Promise<QuotaResponse>;
  evaluatePolicy(context: Context, action: Action): Promise<PolicyDecision>;
  registerAgent(agentId: string, quotas: QuotaConfig): Promise<void>;
}
```

**Mock Implementation:**
```typescript
class CretoAuthzMock implements ICretoAuthz {
  private quotas = new Map<string, number>();
  private policies = new Map<string, Policy>();

  async checkQuota(agentId: string, resource: string): Promise<QuotaResponse> {
    // Deterministic mock: always allow with fixed remaining
    const key = `${agentId}:${resource}`;
    const remaining = this.quotas.get(key) ?? 1000;

    // Simulate 168ns latency (actual: sleep not practical, use sync)
    return { allowed: remaining > 0, remaining, timestamp: Date.now() };
  }

  async evaluatePolicy(context: Context, action: Action): Promise<PolicyDecision> {
    // Default: approve unless explicitly denied in test
    const policyId = `${context.agentId}:${action.type}`;
    const policy = this.policies.get(policyId);

    if (policy?.deny) {
      return { decision: 'DENY', reason: policy.reason };
    }

    return { decision: 'APPROVE', reason: 'mock-default-approve' };
  }

  // Test helper: configure mock behavior
  setQuota(agentId: string, resource: string, amount: number): void {
    this.quotas.set(`${agentId}:${resource}`, amount);
  }

  setPolicy(agentId: string, actionType: string, policy: Policy): void {
    this.policies.set(`${agentId}:${actionType}`, policy);
  }
}
```

**Test Fixtures:**
```typescript
// fixtures/authz-fixtures.ts
export const DEFAULT_QUOTA = {
  'agent-001:api-calls': 10000,
  'agent-001:storage-mb': 1024,
  'agent-002:api-calls': 5000
};

export const POLICY_DENY_HIGH_RISK = {
  agentId: 'agent-003',
  actionType: 'transfer-funds',
  deny: true,
  reason: 'high-risk-action-requires-human-approval'
};
```

**Integration Test Approach:**
```typescript
describe('creto-authz integration', () => {
  let authz: ICretoAuthz;

  beforeEach(() => {
    if (process.env.USE_REAL_AUTHZ === 'true') {
      authz = new CretoAuthzClient({ endpoint: 'localhost:50051' });
    } else {
      authz = new CretoAuthzMock();
      (authz as CretoAuthzMock).setQuota('agent-001', 'api-calls', 10000);
    }
  });

  it('should enforce quota limits', async () => {
    const response = await authz.checkQuota('agent-001', 'api-calls');
    expect(response.allowed).toBe(true);
    expect(response.remaining).toBeGreaterThan(0);
  });
});
```

---

#### 7.2.2 creto-vault Mock

**Interface:**
```typescript
interface ICretoVault {
  getSecret(path: string, lease?: string): Promise<SecretResponse>;
  renewLease(leaseId: string): Promise<LeaseResponse>;
  revokeSecret(leaseId: string): Promise<void>;
}
```

**Mock Implementation:**
```typescript
class CretoVaultMock implements ICretoVault {
  private secrets = new Map<string, string>();

  async getSecret(path: string, lease?: string): Promise<SecretResponse> {
    // Return static test secrets
    const secret = this.secrets.get(path) ?? `mock-secret-${path}`;
    return {
      secret,
      leaseId: `mock-lease-${Date.now()}`,
      ttl: 3600
    };
  }

  async renewLease(leaseId: string): Promise<LeaseResponse> {
    return { leaseId, ttl: 3600, renewed: true };
  }

  async revokeSecret(leaseId: string): Promise<void> {
    // No-op in mock
  }

  // Test helper
  setSecret(path: string, value: string): void {
    this.secrets.set(path, value);
  }
}
```

**Test Fixtures:**
```typescript
export const VAULT_SECRETS = {
  'secret/data/agent-001/api-key': 'sk-test-abc123',
  'secret/data/agent-001/database-url': 'postgresql://test:test@localhost:5432/test',
  'secret/data/trading/broker-api': 'broker-key-xyz'
};
```

---

#### 7.2.3 creto-crypto Mock

**Interface:**
```typescript
interface ICretoCrypto {
  sign(message: Buffer, privateKey: Buffer): Promise<Buffer>;
  verify(message: Buffer, signature: Buffer, publicKey: Buffer): Promise<boolean>;
  encrypt(plaintext: Buffer, publicKey: Buffer): Promise<Buffer>;
  decrypt(ciphertext: Buffer, privateKey: Buffer): Promise<Buffer>;
}
```

**Mock Implementation:**
```typescript
import * as crypto from 'crypto';

class CretoCryptoMock implements ICretoCrypto {
  async sign(message: Buffer, privateKey: Buffer): Promise<Buffer> {
    // Use Node.js crypto for deterministic signing in tests
    const sign = crypto.createSign('SHA256');
    sign.update(message);
    return sign.sign(privateKey);
  }

  async verify(message: Buffer, signature: Buffer, publicKey: Buffer): Promise<boolean> {
    const verify = crypto.createVerify('SHA256');
    verify.update(message);
    return verify.verify(publicKey, signature);
  }

  async encrypt(plaintext: Buffer, publicKey: Buffer): Promise<Buffer> {
    return crypto.publicEncrypt(publicKey, plaintext);
  }

  async decrypt(ciphertext: Buffer, privateKey: Buffer): Promise<Buffer> {
    return crypto.privateDecrypt(privateKey, ciphertext);
  }
}
```

**Test Fixtures:**
```typescript
// Pre-generated test keypairs (do NOT use in production)
export const TEST_KEYPAIR_1 = {
  publicKey: Buffer.from('...'),
  privateKey: Buffer.from('...')
};

export const TEST_KEYPAIR_2 = {
  publicKey: Buffer.from('...'),
  privateKey: Buffer.from('...')
};
```

---

#### 7.2.4 creto-consensus Mock

**Interface:**
```typescript
interface ICretoConsensus {
  proposeEvent(event: Event): Promise<SequenceResponse>;
  getSequence(eventId: string): Promise<number>;
  getLeader(): Promise<string>;
}
```

**Mock Implementation:**
```typescript
class CretoConsensusMock implements ICretoConsensus {
  private sequence = 0;
  private events = new Map<string, number>();

  async proposeEvent(event: Event): Promise<SequenceResponse> {
    this.sequence++;
    this.events.set(event.id, this.sequence);
    return { eventId: event.id, sequence: this.sequence, leader: 'mock-node-1' };
  }

  async getSequence(eventId: string): Promise<number> {
    return this.events.get(eventId) ?? -1;
  }

  async getLeader(): Promise<string> {
    return 'mock-node-1';
  }
}
```

---

#### 7.2.5 creto-audit Mock

**Interface:**
```typescript
interface ICretoAudit {
  appendLog(entry: AuditEntry, proof?: MerkleProof): Promise<MerkleRoot>;
  verifyProof(entry: AuditEntry, proof: MerkleProof): Promise<boolean>;
  getEntries(filter: AuditFilter): Promise<AuditEntry[]>;
}
```

**Mock Implementation:**
```typescript
class CretoAuditMock implements ICretoAudit {
  private logs: AuditEntry[] = [];

  async appendLog(entry: AuditEntry, proof?: MerkleProof): Promise<MerkleRoot> {
    this.logs.push(entry);
    // Return mock Merkle root (in real impl, build tree)
    return { root: Buffer.from(`mock-root-${this.logs.length}`) };
  }

  async verifyProof(entry: AuditEntry, proof: MerkleProof): Promise<boolean> {
    // Always valid in mock
    return true;
  }

  async getEntries(filter: AuditFilter): Promise<AuditEntry[]> {
    return this.logs.filter(entry => {
      if (filter.agentId && entry.agentId !== filter.agentId) return false;
      if (filter.startTime && entry.timestamp < filter.startTime) return false;
      return true;
    });
  }
}
```

---

### 7.3 Mock Injection Strategy

**Dependency Injection with Environment Variables:**

```typescript
// src/common/config.ts
export const CONFIG = {
  useMocks: process.env.CRETO_USE_MOCKS === 'true',
  authzEndpoint: process.env.CRETO_AUTHZ_ENDPOINT || 'localhost:50051',
  vaultEndpoint: process.env.CRETO_VAULT_ENDPOINT || 'http://localhost:8200'
};

// src/common/factory.ts
export function createAuthzClient(): ICretoAuthz {
  if (CONFIG.useMocks) {
    return new CretoAuthzMock();
  }
  return new CretoAuthzClient({ endpoint: CONFIG.authzEndpoint });
}
```

**Environment Modes:**

```bash
# Development (uses mocks)
export CRETO_USE_MOCKS=true
npm run dev

# Integration testing (uses real services in Docker)
export CRETO_USE_MOCKS=false
export CRETO_AUTHZ_ENDPOINT=localhost:50051
npm run test:integration

# Production (uses real services)
export CRETO_USE_MOCKS=false
export CRETO_AUTHZ_ENDPOINT=authz.creto.internal:50051
npm start
```

---

### 7.4 Integration Test Strategy

**Three-Tier Testing:**

1. **Unit Tests (Mocks):** Fast, isolated, deterministic
2. **Integration Tests (Docker Compose):** Real services, controlled environment
3. **E2E Tests (Staging):** Full platform, production-like

**Docker Compose for Integration Tests:**

```yaml
# docker-compose.test.yml
version: '3.8'
services:
  creto-authz:
    image: creto/authz:1.3.0
    ports:
      - "50051:50051"
    environment:
      - AUTHZ_MODE=test

  creto-vault:
    image: vault:1.13.0
    ports:
      - "8200:8200"
    environment:
      - VAULT_DEV_ROOT_TOKEN_ID=test-token

  creto-enablement:
    build: .
    depends_on:
      - creto-authz
      - creto-vault
    environment:
      - CRETO_USE_MOCKS=false
      - CRETO_AUTHZ_ENDPOINT=creto-authz:50051
      - CRETO_VAULT_ENDPOINT=http://creto-vault:8200
```

**Test Execution:**

```bash
# Unit tests (fast, mocks)
npm run test

# Integration tests (slower, real services)
docker-compose -f docker-compose.test.yml up -d
npm run test:integration
docker-compose -f docker-compose.test.yml down

# E2E tests (slowest, staging environment)
npm run test:e2e -- --env=staging
```

---

## 8. Dependency Update Strategy

### 8.1 Automated Dependency Monitoring

**Tools:**
- Dependabot (GitHub) for security updates
- Renovate for automated PR creation
- Snyk for vulnerability scanning

**Configuration (.github/dependabot.yml):**

```yaml
version: 2
updates:
  - package-ecosystem: "npm"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 5
    reviewers:
      - "creto-platform-team"
    labels:
      - "dependencies"

  - package-ecosystem: "docker"
    directory: "/"
    schedule:
      interval: "weekly"
```

### 8.2 Update Policy

| Dependency Type | Update Frequency | Approval Process |
|----------------|------------------|------------------|
| **Security patches** | Immediate | Auto-merge if tests pass |
| **Patch versions** | Weekly | Auto-merge if tests pass |
| **Minor versions** | Monthly | Manual review + approval |
| **Major versions** | Quarterly | Architecture review required |
| **Cryptographic libraries** | Immediate | Security team review |

### 8.3 Breaking Change Protocol

**When External Dependency Introduces Breaking Change:**

1. **Detection:** CI fails due to dependency update
2. **Isolation:** Pin old version, create feature branch
3. **Analysis:** Document breaking changes, impact assessment
4. **Adaptation:** Update code, add compatibility layer if needed
5. **Testing:** Run full test suite + manual verification
6. **Rollout:** Staged deployment (dev → staging → prod)

**Example:**

```typescript
// Compatibility layer for creto-authz v2.0.0 policy DSL change
class AuthzPolicyAdapter {
  // Convert v1 policy to v2 format
  static adaptPolicy(v1Policy: V1Policy): V2Policy {
    return {
      version: 2,
      rules: v1Policy.rules.map(rule => ({
        ...rule,
        // Map v1 fields to v2 fields
        conditions: rule.when
      }))
    };
  }
}
```

---

## 9. Dependency Health Dashboard

### 9.1 Metrics to Track

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| **External dependency uptime** | 99.9% | < 99.5% |
| **Average response time (creto-authz)** | 168ns | > 500ns |
| **Average response time (creto-vault)** | 2ms | > 10ms |
| **Dependency staleness** | < 30 days | > 90 days |
| **Known vulnerabilities** | 0 critical | > 0 critical |
| **Test pass rate (with real deps)** | 100% | < 95% |

### 9.2 Monitoring Implementation

```typescript
// src/common/dependency-health.ts
export class DependencyHealthMonitor {
  async checkHealth(): Promise<HealthReport> {
    const checks = await Promise.all([
      this.checkAuthz(),
      this.checkVault(),
      this.checkCrypto(),
      this.checkConsensus(),
      this.checkAudit()
    ]);

    return {
      timestamp: Date.now(),
      overall: checks.every(c => c.healthy) ? 'HEALTHY' : 'DEGRADED',
      checks
    };
  }

  private async checkAuthz(): Promise<HealthCheck> {
    const start = Date.now();
    try {
      await this.authz.checkQuota('health-check', 'ping');
      const latency = Date.now() - start;
      return {
        service: 'creto-authz',
        healthy: latency < 1, // 1ms threshold
        latency,
        message: `Responded in ${latency}ms`
      };
    } catch (error) {
      return {
        service: 'creto-authz',
        healthy: false,
        error: error.message
      };
    }
  }
}
```

---

## 10. Summary and Recommendations

### 10.1 Critical Path Summary

**Total Duration:** 6-8 weeks
**Critical Bottlenecks:**
1. `creto-authz` latency requirement (168ns)
2. `creto-vault` availability for secret injection
3. Sequential dependency chain (common → metering → oversight → runtime → messaging)

**Recommended Mitigation:**
- Start with mock implementations for rapid iteration
- Parallelize development of independent modules (`metering` + `messaging`)
- Implement circuit breakers and fallbacks for external dependencies
- Use feature flags to toggle between mock and real implementations

### 10.2 Build Order Recommendation

**Week 1-2 (Foundation):**
1. `creto-enablement-common` (all teams)
2. Mock implementations (Team A)
3. CI/CD setup (Team B)

**Week 3-4 (Core Services):**
1. `creto-metering` (Team A)
2. `creto-messaging` (Team B)
3. `creto-oversight` (Team C, after `metering` API defined)

**Week 5 (Integration):**
1. `creto-runtime` (Team A)
2. `creto-enablement-orchestrator` (Team B)
3. Demo 1 + Demo 2 (Team C)

**Week 6-8 (Demos + Polish):**
1. Demo 3 + Demo 4 (all teams)
2. Performance optimization
3. Documentation

### 10.3 Risk Register

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| External dependency unavailable | High | Medium | Mock fallbacks, circuit breakers |
| Breaking change in external API | High | Low | Version pinning, compatibility layers |
| Performance regression | Medium | Medium | Continuous benchmarking, alerts |
| Integration test flakiness | Medium | High | Retry logic, test isolation |
| Team coordination overhead | Medium | High | Daily standups, shared backlog |

---

## Appendix A: Dependency Graph Visualization

```
┌─────────────────────────────────────────────────────────────────┐
│                     External Dependencies                        │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌────────┐│
│  │  authz  │  │ storage │  │   nhi   │  │  crypto │  │  vault ││
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘  └───┬────┘│
└───────┼───────────┼──────────────┼────────────┼──────────┼──────┘
        │           │              │            │          │
┌───────┴───────────┴──────────────┴────────────┴──────────┴──────┐
│                  creto-enablement-common                          │
└───────┬───────────┬──────────────┬────────────┬──────────────────┘
        │           │              │            │
   ┌────▼────┐ ┌────▼────┐   ┌────▼────┐  ┌────▼────┐
   │metering │ │oversight│   │ runtime │  │messaging│
   └────┬────┘ └────┬────┘   └────┬────┘  └────┬────┘
        │           │              │            │
        └───────────┴──────────────┴────────────┘
                    │
              ┌─────▼─────┐
              │orchestrator│
              └─────┬──────┘
                    │
        ┌───────────┼───────────┬───────────┐
        │           │           │           │
   ┌────▼────┐ ┌───▼────┐ ┌───▼────┐ ┌────▼────┐
   │ Demo 1  │ │ Demo 2 │ │ Demo 3 │ │ Demo 4  │
   │(Trading)│ │(Travel)│ │(Health)│ │  (PSA)  │
   └─────────┘ └────────┘ └────────┘ └─────────┘
```

---

**Document Control:**
- **Version:** 1.0.0
- **Last Updated:** 2025-12-26
- **Next Review:** 2026-01-26
- **Owner:** Platform Architecture Team
- **Approvers:** CTO, VP Engineering, Security Lead

**Change Log:**
- 2025-12-26: Initial version (v1.0.0)
