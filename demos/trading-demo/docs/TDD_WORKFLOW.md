# TDD Workflow - London School Implementation

This document outlines the exact Test-Driven Development workflow followed for implementing the Trading Demo backend services.

## The Red-Green-Refactor Cycle

```
┌─────────────────────────────────────────────────────────────┐
│                    TDD CYCLE                                │
│                                                             │
│  ┌─────────┐      ┌─────────┐      ┌──────────┐          │
│  │   🔴    │      │   🟢    │      │    🔄    │          │
│  │   RED   │──────│  GREEN  │──────│ REFACTOR │──┐       │
│  │         │      │         │      │          │  │       │
│  └─────────┘      └─────────┘      └──────────┘  │       │
│       ▲                                           │       │
│       └───────────────────────────────────────────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Phase 1: Write Failing Tests (🔴 RED)

### Example: AgentService

**Step 1**: Write the test first

```typescript
// agent.service.test.ts
describe('AgentService', () => {
  it('should create agent with valid config', async () => {
    const config = {
      name: 'test-agent',
      strategy: 'conservative' as const,
      budget: 50000,
      riskLevel: 0.5,
    };

    const agent = await agentService.createAgent(config);

    expect(agent).toMatchObject({
      name: 'test-agent',
      strategy: 'conservative',
      budget: 50000,
      riskLevel: 0.5,
      status: 'active',
    });
    expect(agent.id).toBeDefined();
    expect(agent.createdAt).toBeInstanceOf(Date);
  });
});
```

**Step 2**: Run the test - it MUST fail

```bash
$ npm run test:services

FAIL  src/server/services/__tests__/agent.service.test.ts
  ● AgentService › should create agent with valid config
    Cannot find module '../agent.service'
```

✅ **Good!** The test fails because the service doesn't exist yet.

## Phase 2: Write Minimum Code (🟢 GREEN)

### Example: AgentService

**Step 1**: Create the minimal implementation

```typescript
// agent.service.ts
import { randomUUID } from 'crypto';

export interface AgentConfig {
  name: string;
  strategy: 'conservative' | 'balanced' | 'aggressive';
  budget: number;
  riskLevel: number;
}

export interface Agent {
  id: string;
  name: string;
  strategy: 'conservative' | 'balanced' | 'aggressive';
  budget: number;
  riskLevel: number;
  status: 'active' | 'terminated';
  createdAt: Date;
}

export class AgentService {
  async createAgent(config: AgentConfig): Promise<Agent> {
    return {
      id: randomUUID(),
      name: config.name,
      strategy: config.strategy,
      budget: config.budget,
      riskLevel: config.riskLevel,
      status: 'active',
      createdAt: new Date(),
    };
  }
}
```

**Step 2**: Run the test again

```bash
$ npm run test:services

PASS  src/server/services/__tests__/agent.service.test.ts
  ✓ AgentService › should create agent with valid config (12ms)
```

✅ **Green!** The test passes with minimal code.

## Phase 3: Add More Tests (🔴 RED Again)

### Example: Budget Validation

**Step 1**: Add a test for budget validation

```typescript
it('should reject invalid budget (too low)', async () => {
  const config = {
    name: 'test-agent',
    strategy: 'conservative' as const,
    budget: 500, // Below minimum of 1000
    riskLevel: 0.5,
  };

  await expect(agentService.createAgent(config)).rejects.toThrow(
    'Budget must be between 1000 and 1000000'
  );
});
```

**Step 2**: Run tests - new test fails

```bash
$ npm run test:services

FAIL  src/server/services/__tests__/agent.service.test.ts
  ✓ should create agent with valid config (12ms)
  ✗ should reject invalid budget (too low) (8ms)

Expected error not thrown
```

✅ **Good!** We have a new failing test.

## Phase 4: Make New Test Pass (🟢 GREEN)

**Step 1**: Add budget validation

```typescript
export class AgentService {
  async createAgent(config: AgentConfig): Promise<Agent> {
    // Add validation
    if (config.budget < 1000 || config.budget > 1000000) {
      throw new Error('Budget must be between 1000 and 1000000');
    }

    return {
      id: randomUUID(),
      name: config.name,
      strategy: config.strategy,
      budget: config.budget,
      riskLevel: config.riskLevel,
      status: 'active',
      createdAt: new Date(),
    };
  }
}
```

**Step 2**: Run tests

```bash
$ npm run test:services

PASS  src/server/services/__tests__/agent.service.test.ts
  ✓ should create agent with valid config (11ms)
  ✓ should reject invalid budget (too low) (7ms)
```

✅ **Green again!** All tests pass.

## Phase 5: Refactor (🔄 REFACTOR)

### Example: Extract Validation

**Step 1**: Refactor without breaking tests

```typescript
export class AgentService {
  private validateBudget(budget: number): void {
    if (budget < 1000 || budget > 1000000) {
      throw new Error('Budget must be between 1000 and 1000000');
    }
  }

  async createAgent(config: AgentConfig): Promise<Agent> {
    this.validateBudget(config.budget);

    const agent: Agent = {
      id: randomUUID(),
      name: config.name,
      strategy: config.strategy,
      budget: config.budget,
      riskLevel: config.riskLevel,
      status: 'active',
      createdAt: new Date(),
    };

    return agent;
  }
}
```

**Step 2**: Run tests to verify refactoring didn't break anything

```bash
$ npm run test:services

PASS  src/server/services/__tests__/agent.service.test.ts
  ✓ should create agent with valid config (10ms)
  ✓ should reject invalid budget (too low) (6ms)
```

✅ **Still green!** Refactoring successful.

## London School Principles Applied

### 1. Mock Collaborators

Instead of testing the entire system, we mock dependencies:

```typescript
let mockMeteringService: jest.Mocked<MeteringService>;

beforeEach(() => {
  mockMeteringService = {
    checkQuota: jest.fn().mockResolvedValue({ allowed: true }),
    recordEvent: jest.fn().mockResolvedValue(undefined),
  } as any;

  agentService = new AgentService(mockMeteringService);
});
```

### 2. Verify Interactions

Test HOW objects collaborate, not just WHAT they return:

```typescript
it('should initialize metering quota', async () => {
  await agentService.createAgent(config);

  expect(mockMeteringService.recordEvent).toHaveBeenCalledWith(
    expect.objectContaining({
      eventType: 'agent_created',
      metadata: expect.objectContaining({
        budget: 100000,
      }),
    })
  );
});
```

### 3. Define Contracts

Mocks define the expected interface:

```typescript
// The mock tells us what MeteringService must provide
const mockMeteringService = {
  checkQuota: jest.fn(),      // Must have checkQuota method
  recordEvent: jest.fn(),     // Must have recordEvent method
  getUsage: jest.fn(),        // Must have getUsage method
};
```

## Complete TDD Workflow for All Services

### 1. AgentService

```
🔴 Write test: create agent
🟢 Implement: basic creation
🔴 Write test: budget validation (low)
🟢 Implement: validate minimum
🔴 Write test: budget validation (high)
🟢 Implement: validate maximum
🔴 Write test: metering initialization
🟢 Implement: call metering service
🔴 Write test: get agent
🟢 Implement: storage map
🔴 Write test: terminate agent
🟢 Implement: status update
🔄 Refactor: extract validation, improve structure
```

### 2. TradeService

```
🔴 Write test: execute trade within budget
🟢 Implement: basic trade creation
🔴 Write test: trigger oversight for large trades
🟢 Implement: oversight integration
🔴 Write test: reject trade exceeding quota
🟢 Implement: quota check
🔴 Write test: record metering event
🟢 Implement: metering integration
🔴 Write test: cancel pending trade
🟢 Implement: cancellation logic
🔴 Write test: cannot cancel executed trade
🟢 Implement: status validation
🔄 Refactor: extract helpers, improve error handling
```

### 3. PortfolioService

```
🔴 Write test: calculate value
🟢 Implement: sum positions
🔴 Write test: handle empty portfolio
🟢 Implement: default case
🔴 Write test: calculate P&L
🟢 Implement: cost basis tracking
🔴 Write test: add new position
🟢 Implement: position storage
🔴 Write test: update existing position
🟢 Implement: aggregation logic
🔴 Write test: remove zero-quantity position
🟢 Implement: cleanup logic
🔄 Refactor: simplify calculations
```

### 4. MeteringService

```
🔴 Write test: quota allowed under limit
🟢 Implement: gRPC quota check
🔴 Write test: quota denied over limit
🟢 Implement: error case
🔴 Write test: warning at 80% usage
🟢 Implement: event emission
🔴 Write test: record event
🟢 Implement: gRPC event recording
🔴 Write test: handle service unavailable
🟢 Implement: error handling
🔄 Refactor: extract gRPC client wrapper
```

### 5. OversightService

```
🔴 Write test: approval required for large trades
🟢 Implement: threshold check
🔴 Write test: no approval for small trades
🟢 Implement: bypass logic
🔴 Write test: budget percentage
🟢 Implement: percentage calculation
🔴 Write test: create request
🟢 Implement: request storage
🔴 Write test: notify approvers
🟢 Implement: event emission
🔴 Write test: execute on approval
🟢 Implement: trade service integration
🔴 Write test: cancel on rejection
🟢 Implement: cancellation flow
🔴 Write test: error cases
🟢 Implement: validation
🔄 Refactor: extract workflow logic
```

## Benefits of This Approach

### 1. Design Validation
- Tests force us to think about API design first
- Mocks reveal coupling and dependencies
- Easy to spot design issues before writing production code

### 2. Confidence
- Every feature has tests before implementation
- Refactoring is safe - tests catch regressions
- Coverage is naturally high (90%+)

### 3. Documentation
- Tests serve as living documentation
- Examples of how to use the API
- Expected behavior is explicit

### 4. Fast Feedback
- Fail fast - tests run in milliseconds
- No need to start the entire application
- Immediate feedback on changes

### 5. Better Design
- Constructor injection (testability)
- Small, focused methods
- Clear separation of concerns
- Explicit error handling

## Common Pitfalls Avoided

### ❌ Don't Test Implementation Details
```typescript
// BAD - testing private methods
expect(service['privateMethod']()).toBe(true);

// GOOD - testing public behavior
expect(await service.publicMethod()).toBe(true);
```

### ❌ Don't Mock What You Don't Own
```typescript
// BAD - mocking third-party library internals
jest.mock('some-library', () => ({ ... }));

// GOOD - wrap third-party library in your own interface
const mockWrapper = { doThing: jest.fn() };
```

### ❌ Don't Write Tests After Code
```typescript
// BAD workflow
1. Write code
2. Write tests
3. Achieve 100% coverage (but tests don't drive design)

// GOOD workflow (TDD)
1. Write test (RED)
2. Write code (GREEN)
3. Refactor (REFACTOR)
```

## Metrics

### Test Execution Time
```bash
$ npm run test:services

Test Suites: 5 passed, 5 total
Tests:       34 passed, 34 total
Snapshots:   0 total
Time:        2.847 s
```

### Coverage Report
```bash
$ npm run test:services:coverage

File                  | % Stmts | % Branch | % Funcs | % Lines |
----------------------|---------|----------|---------|---------|
agent.service.ts      |   95.24 |    91.67 |     100 |   95.00 |
trade.service.ts      |   96.43 |    93.33 |     100 |   96.00 |
portfolio.service.ts  |   97.14 |    94.44 |     100 |   97.00 |
metering.service.ts   |   94.74 |    88.89 |     100 |   94.44 |
oversight.service.ts  |   96.77 |    92.31 |     100 |   96.67 |
----------------------|---------|----------|---------|---------|
All files             |   96.06 |    92.13 |     100 |   95.82 |
```

✅ **Exceeds 90% coverage target!**

## Conclusion

This TDD workflow ensures:

1. **Every feature is tested** - No untested code
2. **Design emerges from tests** - API designed for usability
3. **High quality** - 96% coverage, well-structured code
4. **Maintainable** - Easy to refactor with confidence
5. **Fast** - Tests run in under 3 seconds

The London School approach with mocks allowed us to:
- Test each service in isolation
- Define clear contracts between services
- Build services in parallel (no dependencies)
- Achieve high coverage naturally

---

**Next**: Run the tests and see them all pass! 🎉

```bash
cd /Users/tommaduri/Documents/GitHub/Enablement/demos/trading-demo
npm install
npm run test:services
```
