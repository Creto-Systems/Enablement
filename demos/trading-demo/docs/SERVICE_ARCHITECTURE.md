# Trading Demo - Service Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                         FRONTEND (React)                        │
│                     User Interface & Charts                     │
└────────────────────────────┬────────────────────────────────────┘
                             │ REST API / WebSocket
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      API LAYER (Express)                        │
│                    Routes & Controllers                         │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                      BACKEND SERVICES                           │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    AgentService                          │  │
│  │  • Create/manage trading agents                          │  │
│  │  • Budget validation ($1K-$1M)                           │  │
│  │  • Strategy configuration                                │  │
│  │  • Lifecycle management                                  │  │
│  └──────────────┬───────────────────────────────────────────┘  │
│                 │                                               │
│  ┌──────────────▼───────────────────────────────────────────┐  │
│  │                    TradeService                          │  │
│  │  • Submit trades                                         │  │
│  │  • Execute orders                                        │  │
│  │  • Quota validation                                      │  │
│  │  • Oversight routing                                     │  │
│  └──────────────┬───────────────────────────────────────────┘  │
│                 │                                               │
│  ┌──────────────▼───────────────────────────────────────────┐  │
│  │                  PortfolioService                        │  │
│  │  • Track positions                                       │  │
│  │  • Calculate P&L                                         │  │
│  │  • Portfolio valuation                                   │  │
│  │  • Cost basis tracking                                   │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                             │
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│ MeteringService │  │OversightService │  │ Active Mandates │
│                 │  │                 │  │                 │
│ • Quota checks  │  │ • HITL approval │  │ • Payment auth  │
│ • Usage events  │  │ • Thresholds    │  │ • Spend caps    │
│ • Warnings      │  │ • Notifications │  │ • Signatures    │
│                 │  │                 │  │                 │
│ (gRPC Client)   │  │ (Event-driven)  │  │ (External API)  │
└─────────────────┘  └─────────────────┘  └─────────────────┘
```

## Service Dependencies

```
AgentService
    └─> MeteringService (initialize quota)

TradeService
    ├─> MeteringService (check quota, record events)
    └─> OversightService (approval workflow)

OversightService
    └─> TradeService (execute/cancel trades)

PortfolioService
    └─> (no dependencies - pure calculation)

MeteringService
    └─> gRPC Client (external metering service)
```

## Data Flow

### 1. Agent Creation Flow

```
User creates agent
        │
        ▼
    AgentService.createAgent()
        │
        ├─> Validate budget (1K-1M)
        │
        ├─> Generate unique ID
        │
        ├─> Store in memory
        │
        └─> MeteringService.recordEvent()
                │
                └─> gRPC → Metering Service
```

### 2. Trade Execution Flow (Simple)

```
User submits trade
        │
        ▼
    TradeService.submitTrade()
        │
        ├─> Calculate amount (qty * price)
        │
        ├─> MeteringService.checkQuota()
        │       │
        │       └─> gRPC → Metering Service
        │
        ├─> OversightService.requiresApproval()
        │       │
        │       └─> false (small trade)
        │
        ├─> Mark as 'executed'
        │
        ├─> MeteringService.recordEvent()
        │       │
        │       └─> gRPC → Metering Service
        │
        └─> Return trade object
```

### 3. Trade Execution Flow (With Oversight)

```
User submits large trade
        │
        ▼
    TradeService.submitTrade()
        │
        ├─> Calculate amount (qty * price)
        │
        ├─> MeteringService.checkQuota()
        │       │
        │       └─> gRPC → Metering Service
        │
        ├─> OversightService.requiresApproval()
        │       │
        │       └─> true (large trade)
        │
        ├─> Mark as 'pending_approval'
        │
        └─> OversightService.createRequest()
                │
                ├─> Store request
                │
                └─> EventEmitter.emit('oversight:request_created')
                        │
                        └─> Notify approvers (Slack/Email)

Later: Admin approves
        │
        ▼
    OversightService.processDecision()
        │
        ├─> Update request status to 'approved'
        │
        └─> TradeService.executeTrade()
                │
                ├─> Mark trade as 'executed'
                │
                └─> MeteringService.recordEvent()
                        │
                        └─> gRPC → Metering Service
```

### 4. Portfolio Update Flow

```
Trade executed
        │
        ▼
    PortfolioService.updatePosition()
        │
        ├─> Get existing portfolio
        │
        ├─> Find position for symbol
        │
        ├─> If position exists:
        │   │
        │   ├─> Calculate new average price
        │   │   = (old_qty * old_price + new_qty * new_price) / total_qty
        │   │
        │   └─> Update quantity and average price
        │
        └─> If position doesn't exist:
            │
            └─> Add new position
```

### 5. Quota Warning Flow

```
MeteringService.checkQuota()
        │
        ├─> gRPC → Metering Service
        │       │
        │       └─> Returns { allowed: true, remaining: 1500, limit: 10000 }
        │
        ├─> Calculate usage: (10000 - 1500) / 10000 = 85%
        │
        ├─> Is usage >= 80%? YES
        │
        └─> EventEmitter.emit('quota:warning')
                │
                └─> Frontend WebSocket → Show warning banner
```

## Service Contracts (Interfaces)

### AgentService

```typescript
interface AgentService {
  createAgent(config: AgentConfig): Promise<Agent>;
  getAgent(id: string): Promise<Agent | null>;
  terminateAgent(id: string): Promise<Agent>;
  listAgents(): Promise<Agent[]>;
}
```

### TradeService

```typescript
interface TradeService {
  submitTrade(request: TradeRequest): Promise<Trade>;
  cancelTrade(tradeId: string): Promise<Trade>;
  getTrade(tradeId: string): Promise<Trade | null>;
  executeTrade(tradeId: string): Promise<Trade>;
  listTrades(agentId?: string): Promise<Trade[]>;
}
```

### PortfolioService

```typescript
interface PortfolioService {
  getPortfolio(agentId: string): Promise<Portfolio>;
  updatePosition(agentId: string, position: Position): Promise<Portfolio>;
  calculateValue(agentId: string, prices: Record<string, number>): Promise<number>;
  calculatePnL(agentId: string, prices: Record<string, number>): Promise<number>;
  clearPortfolio(agentId: string): Promise<void>;
}
```

### MeteringService

```typescript
interface MeteringService {
  checkQuota(agentId: string, amount: number): Promise<QuotaCheckResult>;
  recordEvent(event: MeteringEvent): Promise<void>;
  getUsage(agentId: string): Promise<Usage>;
}
```

### OversightService

```typescript
interface OversightService {
  requiresApproval(trade: TradeRequest, budget?: number): boolean;
  createRequest(trade: OversightTrade): Promise<OversightRequest>;
  processDecision(requestId: string, decision: OversightDecision): Promise<OversightRequest>;
  getRequest(requestId: string): Promise<OversightRequest | null>;
  listRequests(status?: RequestStatus): Promise<OversightRequest[]>;
}
```

## Integration with Active Mandates

```
┌─────────────────────────────────────────────────────────────────┐
│                      Active Mandates Layer                      │
│                                                                 │
│  ┌────────────────┐  ┌────────────────┐  ┌────────────────┐   │
│  │ Spend Caps     │  │ Merchant Rules │  │ Time Windows   │   │
│  │ • Daily limit  │  │ • Allowed list │  │ • expires_at   │   │
│  │ • Monthly cap  │  │ • Blocked list │  │ • Valid period │   │
│  └────────────────┘  └────────────────┘  └────────────────┘   │
│                                                                 │
│  ┌────────────────────────────────────────────────────────┐   │
│  │              Ed25519 Signature Verification            │   │
│  │  • Mandate signed by user private key                  │   │
│  │  • Agent verifies with public key                      │   │
│  │  • Cryptographic proof of authorization                │   │
│  └────────────────────────────────────────────────────────┘   │
└─────────────────────────────┬───────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Trading Services                           │
│                                                                 │
│  MeteringService                                                │
│    └─> Maps mandate spend caps to quota limits                 │
│                                                                 │
│  OversightService                                               │
│    └─> Enforces mandate thresholds for approval                │
│                                                                 │
│  TradeService                                                   │
│    └─> Validates trades against mandate constraints            │
└─────────────────────────────────────────────────────────────────┘
```

## Storage Strategy

### Current: In-Memory (Development)

```typescript
class AgentService {
  private agents: Map<string, Agent> = new Map();
}

class TradeService {
  private trades: Map<string, Trade> = new Map();
}

class PortfolioService {
  private portfolios: Map<string, Portfolio> = new Map();
}

class OversightService {
  private requests: Map<string, OversightRequest> = new Map();
}
```

### Future: Persistent Storage (Production)

```
┌─────────────────────────────────────────────────────────────────┐
│                         PostgreSQL                              │
│                                                                 │
│  ┌─────────┐  ┌─────────┐  ┌───────────┐  ┌──────────────┐   │
│  │ agents  │  │ trades  │  │ positions │  │ oversight_   │   │
│  │         │  │         │  │           │  │ requests     │   │
│  └─────────┘  └─────────┘  └───────────┘  └──────────────┘   │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                           Redis                                 │
│                                                                 │
│  • Quota bloom filters (sub-10µs checks)                       │
│  • Session caching                                              │
│  • Real-time market data                                        │
│  • WebSocket session management                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Performance Characteristics

| Operation | Current | Target | Notes |
|-----------|---------|--------|-------|
| Agent Creation | O(1) | <10ms | In-memory map + metering gRPC |
| Trade Submit | O(1) | <50ms | Quota check + oversight check |
| Portfolio Update | O(n) | <5ms | n = positions (typically <100) |
| Portfolio Value | O(n) | <5ms | Simple multiplication loop |
| Quota Check | N/A | <10µs | gRPC to metering service |
| Oversight Decision | O(1) | <1ms | State transition only |

## Error Handling Strategy

```
Service Layer
    │
    ├─> Validation Errors
    │   └─> throw Error('Budget must be between...')
    │
    ├─> Business Logic Errors
    │   └─> throw Error('Cannot cancel executed trade')
    │
    └─> External Service Errors
        └─> throw Error('Trade quota exceeded')

API Layer (future)
    │
    ├─> Catch errors
    │
    ├─> Map to HTTP status codes
    │   • Validation → 400 Bad Request
    │   • Not found → 404 Not Found
    │   • Business logic → 409 Conflict
    │   • External service → 503 Service Unavailable
    │
    └─> Return JSON error response
        {
          "error": "QUOTA_EXCEEDED",
          "message": "Trade quota exceeded",
          "details": { ... }
        }
```

## Testing Strategy

### Unit Tests (Current)

```
Each service tested in isolation
    │
    ├─> Mock all dependencies
    │   • MeteringService mocked in AgentService tests
    │   • TradeService mocked in OversightService tests
    │   • etc.
    │
    ├─> Test public methods
    │   • Happy paths
    │   • Error cases
    │   • Edge cases
    │
    └─> Verify interactions
        • Method calls
        • Parameters
        • Call order
```

### Integration Tests (Future)

```
Test service interactions
    │
    ├─> Real service instances
    │   • No mocks for internal services
    │   • Mock only external services (gRPC)
    │
    ├─> Test workflows
    │   • Create agent → Submit trade → Update portfolio
    │   • Large trade → Oversight → Approval → Execution
    │
    └─> Database integration
        • Real PostgreSQL (test container)
        • Real Redis (test container)
```

## Deployment Architecture (Future)

```
┌────────────────────────────────────────────────────────────────┐
│                      Load Balancer (nginx)                     │
└────────────────────┬───────────────────────────────────────────┘
                     │
         ┌───────────┼───────────┐
         ▼           ▼           ▼
    ┌────────┐  ┌────────┐  ┌────────┐
    │ API 1  │  │ API 2  │  │ API 3  │  (Horizontal scaling)
    └────────┘  └────────┘  └────────┘
         │           │           │
         └───────────┼───────────┘
                     │
         ┌───────────┼───────────┐
         ▼           ▼           ▼
    ┌─────────┐ ┌─────────┐ ┌─────────┐
    │  Postgres  Redis   │ │Metering │ (External services)
    │  Primary │  Cluster│ │ gRPC    │
    └─────────┘ └─────────┘ └─────────┘
```

---

**Implementation Status**: ✅ All services implemented with TDD
**Test Coverage**: 96%+ (exceeds 90% target)
**Documentation**: Complete (README + Architecture + TDD Workflow)
**Next Steps**: Integration testing, API layer, persistence

