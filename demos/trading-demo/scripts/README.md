# Trading Demo Scripts

Demo data and scenario scripts for investor presentations showcasing all 4 Enablement products.

## 📦 Scripts Overview

### 1. `seed-demo-data.ts`
Populates the database with realistic demo data.

**Features:**
- 3 demo agents with different trading strategies
- Portfolio positions across major tech stocks
- 30 days of trade history
- Oversight requests (pending and resolved)
- Metering usage data and quota configs

**Usage:**
```bash
npm run demo:seed
```

**Demo Agents:**
- **Alpha Trader** - Aggressive momentum strategy, $100K budget, 85% quota used
- **Safe Haven** - Conservative mean reversion, $50K budget, 64% quota used
- **Balanced Growth** - Market making strategy, $75K budget, 78% quota used

### 2. `reset-demo.ts`
Clears all data and re-seeds with fresh demo data.

**Usage:**
```bash
npm run demo:reset
```

**Actions:**
- Deletes existing database
- Clears memory cache
- Resets metering quotas
- Clears oversight history
- Re-seeds with fresh data

### 3. `demo-scenario.ts`
Interactive demo runner with 5 investor-ready scenarios.

**Usage:**
```bash
# Interactive menu
npm run demo:run

# Run specific scenario
npm run demo:run 1    # Create new agent
npm run demo:run 2    # Execute small trade
npm run demo:run 3    # Large trade with oversight
npm run demo:run 4    # Approve pending trade
npm run demo:run 5    # Hit quota limit
npm run demo:run 6    # Run all scenarios
```

## 🎯 Demo Scenarios

### Scenario 1: Create New Agent
**Duration:** ~2 minutes
**Products shown:** Metering, Runtime

**Flow:**
1. Collect agent configuration
2. Initialize metering quota (5,000 API calls/day)
3. Start agent runtime in sandboxed environment

**Key points:**
- Quota automatically initialized
- Agent isolated in runtime sandbox
- Budget allocation tracked

### Scenario 2: Execute Small Trade
**Duration:** ~2 minutes
**Products shown:** Runtime, Metering, Messaging

**Flow:**
1. Agent analyzes market opportunity (AAPL)
2. Check metering quota availability
3. Execute trade ($3,950)
4. Broadcast real-time event updates

**Key points:**
- Trade execution <200ms
- 5 API calls recorded
- 4 event messages delivered
- Real-time WebSocket updates

### Scenario 3: Execute Large Trade (Oversight)
**Duration:** ~3 minutes
**Products shown:** Oversight, Metering, Messaging

**Flow:**
1. Agent proposes large trade (NVDA $50,550)
2. Check auto-approval rules (denied - exceeds $10K limit)
3. Create oversight request
4. Run risk assessment (score: 65/100 - MEDIUM)
5. Notify approvers via email, Slack, SMS

**Key points:**
- Threshold-based oversight triggering
- Automated risk assessment
- Multi-channel notifications
- Expiring approval requests (4 hours)

### Scenario 4: Approve Pending Trade
**Duration:** ~2 minutes
**Products shown:** Oversight, Runtime, Messaging

**Flow:**
1. View pending oversight request
2. Approver reviews risk assessment and agent performance
3. Record approval decision with audit trail
4. Execute approved trade
5. Notify all parties

**Key points:**
- Human-in-the-loop approval
- Full audit trail (IP, duration, reason)
- Automatic trade execution post-approval
- Notification cascade

### Scenario 5: Hit Quota Limit
**Duration:** ~2 minutes
**Products shown:** Metering, Messaging

**Flow:**
1. Agent approaching quota limit (97.5% used)
2. Attempt high-frequency operations
3. Quota enforcement triggers
4. Block/throttle non-essential calls
5. Show quota reset options

**Key points:**
- Real-time quota tracking
- Automatic enforcement at 100%
- Alert notifications
- Daily quota reset (00:00 UTC)

## 📊 Demo Data Summary

### Agents
| Name | Strategy | Budget | Quota Used | Positions |
|------|----------|--------|------------|-----------|
| Alpha Trader | Momentum | $100K | 85.47% | NVDA, TSLA |
| Safe Haven | Mean Reversion | $50K | 64.30% | AAPL, MSFT |
| Balanced Growth | Market Making | $75K | 77.76% | GOOGL, AMZN |

### Trade History
- **4 total trades** across last 30 days
- Mix of: Executed, Pending Approval, Rejected
- Trade values: $3,950 - $50,550
- Symbols: AAPL, TSLA, NVDA, META

### Oversight Requests
- **2 requests** total
- 1 pending (NVDA $50,550, high priority)
- 1 rejected (META $35,500, position size violation)

### Metering Usage
- **Total API calls:** 17,594 across all agents
- **Cost incurred:** $87.98
- **Breakdown:**
  - Market data: 61%
  - Trade execution: 25%
  - Portfolio analysis: 11%
  - Oversight: 3%

## 🎬 Presentation Tips

### Setup (5 minutes before demo)
```bash
# 1. Reset to clean state
npm run demo:reset

# 2. Start the application
npm run dev

# 3. Open browser to http://localhost:5173
```

### Running the Demo

**Option A: Full Interactive Demo (15 minutes)**
```bash
npm run demo:run 6    # Run all scenarios sequentially
```

**Option B: Targeted Scenarios (5-10 minutes)**
```bash
# Pick 2-3 scenarios based on investor interests
npm run demo:run 3    # Oversight workflow
npm run demo:run 4    # Approval process
npm run demo:run 5    # Metering enforcement
```

**Option C: Manual Walkthrough**
Use the live web UI with pre-seeded data:
1. Show agent dashboard (3 active agents)
2. Demonstrate pending oversight request
3. Show metering quota status
4. Execute new trade live

### Key Talking Points

**Metering:**
- "Real-time usage tracking across all API operations"
- "Automatic quota enforcement prevents cost overruns"
- "Granular cost breakdown by operation type"

**Oversight:**
- "Configurable approval thresholds per agent"
- "Automated risk assessment for every high-value trade"
- "Multi-channel notifications with escalation rules"

**Runtime:**
- "Sandboxed execution environment for agent isolation"
- "Sub-second trade execution with full audit trail"
- "Scalable to thousands of concurrent agents"

**Messaging:**
- "Real-time event streaming to all subscribers"
- "Guaranteed message delivery with acknowledgment"
- "WebSocket connections for instant UI updates"

## 🔧 Troubleshooting

### Database not found
```bash
# Re-run seed script
npm run demo:seed
```

### Port already in use
```bash
# Change port in .env or kill existing process
lsof -ti:5173 | xargs kill
```

### Missing dependencies
```bash
# Reinstall all dependencies
npm install
```

### Script execution errors
```bash
# Ensure TypeScript executable is available
npm install -g tsx

# Or use npx
npx tsx scripts/seed-demo-data.ts
```

## 📝 Customization

### Add new demo agent
Edit `/tests/fixtures/mock-data.ts`:
```typescript
export const DEMO_AGENTS = {
  // ... existing agents
  myNewAgent: {
    id: 'agent-custom-004',
    name: 'My Custom Agent',
    // ... configuration
  },
};
```

### Modify trade history
Edit `DEMO_TRADES` array in `/tests/fixtures/mock-data.ts`

### Adjust quota limits
Edit `DEMO_QUOTA_CONFIGS` array in `/tests/fixtures/mock-data.ts`

### Create custom scenario
Add new function to `/scripts/demo-scenario.ts`:
```typescript
async function scenarioN_MyScenario(): Promise<void> {
  printHeader('SCENARIO N: My Custom Scenario');
  // ... implementation
}
```

## 📚 Related Documentation

- [Trading Demo README](../README.md) - Main application documentation
- [API Documentation](../docs/api.md) - REST and WebSocket APIs
- [Type Definitions](../src/types/models.ts) - Complete TypeScript interfaces
- [Test Fixtures](../tests/fixtures/mock-data.ts) - Reusable test data

## 🚀 Next Steps

After running the demo:

1. **Collect feedback** on which scenarios resonated most
2. **Customize data** based on investor interests
3. **Add metrics** relevant to their use case
4. **Prepare Q&A** around architecture and scalability

## 📞 Support

For issues or questions:
- Check existing issues in GitHub repository
- Review main application README
- Contact: [Your Contact Info]
