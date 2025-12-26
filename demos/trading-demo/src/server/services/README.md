# Trading Demo Backend Services

This directory contains the backend services for the Trading Demo application, implemented using **Test-Driven Development (TDD)** following the **London School (mockist)** approach.

## Overview

The backend services provide the core business logic for:
- Agent management (creation, configuration, lifecycle)
- Trade execution and validation
- Portfolio tracking and P&L calculation
- Usage metering and quota enforcement
- Human oversight and approval workflows

## Services

### 1. AgentService

Manages AI trading agents with configurable strategies and budgets.

**Features:**
- Create agents with strategy (conservative, balanced, aggressive)
- Budget validation (min: $1,000, max: $1,000,000)
- Risk level configuration (0.0 - 1.0)
- Agent lifecycle management (active/terminated)
- Integration with metering service for quota initialization

**Example:**
```typescript
const agentService = new AgentService(meteringService);

const agent = await agentService.createAgent({
  name: 'conservative-trader',
  strategy: 'conservative',
  budget: 50000,
  riskLevel: 0.3,
});
```

### 2. TradeService

Handles trade submission, execution, and lifecycle management.

**Features:**
- Trade submission with quota validation
- Automatic oversight routing for large trades
- Trade status tracking (pending_approval, executed, cancelled)
- Integration with metering for usage tracking
- Trade cancellation (only for non-executed trades)

**Example:**
```typescript
const tradeService = new TradeService(meteringService, oversightService);

const trade = await tradeService.submitTrade({
  agentId: 'agent-123',
  symbol: 'AAPL',
  quantity: 10,
  side: 'buy',
  price: 150.00,
});
```

### 3. PortfolioService

Tracks agent positions and calculates portfolio metrics.

**Features:**
- Position tracking with average cost basis
- Automatic position aggregation (buy/sell)
- Position removal when quantity reaches zero
- Portfolio value calculation with market prices
- P&L (profit & loss) calculation

**Example:**
```typescript
const portfolioService = new PortfolioService();

await portfolioService.updatePosition('agent-123', {
  symbol: 'AAPL',
  quantity: 10,
  averagePrice: 150.00,
});

const value = await portfolioService.calculateValue('agent-123', {
  AAPL: 155.00,
});
```

### 4. MeteringService

Integrates with gRPC metering service for quota enforcement and usage tracking.

**Features:**
- Real-time quota checking (sub-10µs target)
- Usage event recording
- Quota warning emissions at 80% threshold
- Usage statistics retrieval
- gRPC error handling and fallback

**Example:**
```typescript
const meteringService = new MeteringService(grpcClient, eventEmitter);

const quotaCheck = await meteringService.checkQuota('agent-123', 1500);
if (!quotaCheck.allowed) {
  throw new Error('Quota exceeded');
}

await meteringService.recordEvent({
  agentId: 'agent-123',
  eventType: 'trade_executed',
  amount: 1500,
  metadata: { symbol: 'AAPL' },
});
```

### 5. OversightService

Implements human-in-the-loop (HITL) approval workflow for high-value trades.

**Features:**
- Configurable approval thresholds (amount and % of budget)
- Oversight request creation and tracking
- Approval/rejection decision processing
- Automatic trade execution or cancellation based on decision
- Event notification system

**Thresholds:**
- Absolute: $10,000 per trade
- Relative: 25% of agent budget

**Example:**
```typescript
const oversightService = new OversightService(tradeService, eventEmitter);

if (oversightService.requiresApproval(trade, agentBudget)) {
  const request = await oversightService.createRequest({
    tradeId: trade.id,
    agentId: 'agent-123',
    ...tradeDetails,
  });

  // Later, when human approves/rejects:
  await oversightService.processDecision(request.id, {
    approved: true,
    approvedBy: 'admin-123',
    reason: 'Within risk parameters',
  });
}
```

## Test-Driven Development Approach

### London School (Mockist) TDD

This project follows the **London School** of TDD, which emphasizes:

1. **Outside-In Development**: Start with acceptance tests and work inward
2. **Mock-Driven Design**: Use mocks to define collaborator contracts
3. **Behavior Verification**: Focus on interactions between objects
4. **Contract Testing**: Define clear interfaces through mock expectations

### Test Structure

Each service has comprehensive test coverage organized by:

```
__tests__/
├── agent.service.test.ts
├── trade.service.test.ts
├── portfolio.service.test.ts
├── metering.service.test.ts
└── oversight.service.test.ts
```

### Test Categories

**Unit Tests:**
- Test individual service methods in isolation
- Mock all dependencies
- Verify interactions between collaborators
- Target: 90%+ coverage

**Interaction Tests:**
- Verify proper collaboration sequences
- Test mock call order and parameters
- Ensure contract compliance

**Edge Cases:**
- Boundary conditions (min/max values)
- Error handling and validation
- State transitions

### Running Tests

```bash
# Run all service tests
npm run test:services

# Run in watch mode (for TDD)
npm run test:services:watch

# Run with coverage report
npm run test:services:coverage
```

### Coverage Thresholds

```javascript
coverageThreshold: {
  global: {
    branches: 90,
    functions: 90,
    lines: 90,
    statements: 90,
  },
}
```

## TDD Workflow

### Red-Green-Refactor Cycle

1. **🔴 RED**: Write a failing test
   ```typescript
   it('should create agent with valid config', async () => {
     const agent = await agentService.createAgent(validConfig);
     expect(agent.id).toBeDefined();
   });
   ```

2. **🟢 GREEN**: Write minimum code to pass
   ```typescript
   async createAgent(config: AgentConfig): Promise<Agent> {
     return { id: randomUUID(), ...config, status: 'active' };
   }
   ```

3. **🔄 REFACTOR**: Improve code while keeping tests green
   ```typescript
   async createAgent(config: AgentConfig): Promise<Agent> {
     this.validateBudget(config.budget);
     const agent = this.buildAgent(config);
     await this.initializeMetering(agent);
     return agent;
   }
   ```

## Architecture

### Dependency Injection

All services use constructor injection for dependencies:

```typescript
class TradeService {
  constructor(
    private meteringService: MeteringService,
    private oversightService: OversightService
  ) {}
}
```

### Error Handling

Services throw descriptive errors:

```typescript
throw new Error('Budget must be between 1000 and 1000000');
throw new Error('Trade quota exceeded');
throw new Error('Cannot cancel executed trade');
```

### Type Safety

Strict TypeScript configuration ensures type safety:

```typescript
export interface AgentConfig {
  name: string;
  strategy: 'conservative' | 'balanced' | 'aggressive';
  budget: number;
  riskLevel: number;
}
```

## Integration with Active Mandates

These services are designed to integrate with the **Active Mandates** payment system:

- **MeteringService** → Quota enforcement using Active Mandates spend caps
- **OversightService** → Human approval for trades exceeding mandate limits
- **TradeService** → Payment authorization via Active Mandates
- **AgentService** → Budget management tied to mandate periods

## Performance Considerations

- **MeteringService**: Targets sub-10µs quota checks (in-memory + Redis)
- **OversightService**: <1ms state transitions
- **PortfolioService**: O(n) complexity for position updates
- **TradeService**: Async execution for non-blocking operations

## Future Enhancements

- [ ] Persistent storage (PostgreSQL/Redis)
- [ ] Event sourcing for trade history
- [ ] Real-time WebSocket notifications
- [ ] Advanced risk models
- [ ] Multi-agent coordination
- [ ] Machine learning integration

## Contributing

When adding new features:

1. Write tests first (TDD)
2. Follow London School patterns
3. Mock external dependencies
4. Maintain 90%+ coverage
5. Update this README

## References

- [London School TDD](https://martinfowler.com/articles/mocksArentStubs.html)
- [Jest Mocking Guide](https://jestjs.io/docs/mock-functions)
- [TypeScript Best Practices](https://typescript-eslint.io/rules/)
