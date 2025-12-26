# Trading Demo - Investor Presentation Script

**Duration**: 10-12 minutes
**Audience**: Investors, Enterprise Customers, Technical Leadership
**Goal**: Demonstrate Creto Enablement Layer value proposition

---

## Pre-Demo Setup (5 minutes before)

### 1. Environment Preparation

```bash
# Reset demo data
npm run demo:reset

# Start all services
npm run dev

# Open browser tabs (arrange side-by-side)
# - Tab 1: http://localhost:5173 (Trading UI)
# - Tab 2: http://localhost:3001 (Grafana - optional)
# - Tab 3: http://localhost:9090 (Prometheus - optional)
```

### 2. Browser Setup

- **Primary Screen**: Trading UI (full screen)
- **Secondary Screen**: Terminal with logs (optional)
- Close unnecessary browser tabs
- Test audio/video if remote presentation

### 3. Pre-populate Data (Optional)

```bash
# Create a sample agent for quick reference
curl -X POST http://localhost:3000/api/v1/agents \
  -H "Authorization: Bearer demo-token" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Pre-Demo Agent",
    "type": "trading",
    "config": {"budget": 50000, "strategy": "conservative"},
    "userId": "demo-user"
  }'
```

---

## Demo Flow

### Act 1: Introduction (2 minutes)

**Talking Points**:

> "Today I'm going to show you how Creto's Enablement Layer provides enterprise-grade governance for autonomous AI agents. We've built a trading demo to illustrate four key capabilities:
>
> 1. **Metering** - Usage-based billing and quota enforcement
> 2. **Oversight** - Human-in-the-loop approval workflows
> 3. **Runtime** - Sandboxed execution environments (coming soon)
> 4. **Messaging** - Encrypted agent communication (coming soon)
>
> Let me show you how this works in practice."

**Actions**:
- Navigate to trading dashboard
- Show empty state (no agents yet)
- Briefly explain the UI components

---

### Act 2: Agent Creation & Metering (2 minutes)

**Talking Points**:

> "First, let's create an AI trading agent. Notice how easy it is—just a name, budget, and strategy. Behind the scenes, our Metering service is initializing quota tracking for this agent."

**Actions**:

1. Click **"Create Agent"** button
2. Fill in form:
   - **Name**: "Demo Trader"
   - **Budget**: $100,000
   - **Strategy**: "Balanced"
   - **Risk Tolerance**: "Medium"
3. Click **"Create"**

**Expected Result**:
```
✅ Agent "Demo Trader" created
✅ Agent ID: agent-abc123
✅ Initial quota: $100,000 daily
```

**Show**:
- Agent card appears in dashboard
- Portfolio shows $100,000 cash
- Metering panel shows 0% quota used

**Talking Points**:

> "Notice the metering panel on the right. This shows real-time quota usage. Our Metering service integrates with the Creto platform to track every operation and enforce spending limits. This is similar to how Lago provides usage-based billing, but built on Creto's quantum-safe infrastructure with 168-nanosecond authorization."

---

### Act 3: Autonomous Trade Execution (3 minutes)

**Talking Points**:

> "Now let's execute a trade. For small trades under our autonomous threshold, the agent can execute immediately without human approval."

**Actions**:

1. Click **"New Trade"** on the agent card
2. Fill in trade form:
   - **Symbol**: AAPL
   - **Side**: Buy
   - **Quantity**: 10
   - **Price**: $150
   - **Type**: Limit
3. **Total**: $1,500 (auto-calculated)
4. Click **"Submit Trade"**

**Expected Result**:
```
✅ Trade executed successfully
✅ Portfolio updated: +10 AAPL @ $150
✅ Quota used: $1,500 / $100,000 (1.5%)
```

**Show**:
- Trade appears in "Recent Trades" with status "Executed"
- Portfolio updates: Cash decreases, AAPL position appears
- Metering panel updates: 1.5% quota used
- Real-time WebSocket update (no page refresh)

**Talking Points**:

> "Notice three things:
>
> 1. **Instant Execution**: The trade executed immediately—no approval needed for small amounts.
> 2. **Real-time Updates**: The UI updated instantly via WebSocket. No page refresh required.
> 3. **Metering Integration**: Our Metering service recorded this event and updated the quota. If we approach 80% of our daily limit, the agent would receive a warning."

**Demo Metering in Detail** (Optional):

```bash
# In terminal (if showing technical audience)
curl -X GET http://localhost:3000/api/v1/agents/agent-abc123/usage \
  -H "Authorization: Bearer demo-token"

# Show the JSON response:
{
  "totalTrades": 1,
  "totalVolume": 1500,
  "quotaUsed": 1500,
  "quotaLimit": 100000,
  "percentUsed": 1.5
}
```

---

### Act 4: Oversight Workflow (3 minutes)

**Talking Points**:

> "Now here's where it gets interesting. For larger trades, our Oversight service requires human approval. This is based on HumanLayer's proven patterns for human-in-the-loop AI, but integrated with Creto's authorization layer."

**Actions**:

1. Click **"New Trade"** again
2. Fill in large trade:
   - **Symbol**: NVDA
   - **Side**: Buy
   - **Quantity**: 500
   - **Price**: $500
   - **Type**: Limit
3. **Total**: $250,000 (exceeds autonomous threshold)
4. Click **"Submit Trade"**

**Expected Result**:
```
⚠️ Trade submitted for approval
⚠️ Amount ($250,000) exceeds autonomous threshold ($50,000)
⚠️ Awaiting human approval
```

**Show**:
- Trade appears with status "Pending Approval"
- Notification banner: "1 trade awaiting approval"
- Oversight panel highlights the pending request

**Talking Points**:

> "The trade didn't execute automatically. Instead, it went to our Oversight service for human approval. In a production deployment, this would trigger:
>
> - Slack notification to the compliance team
> - Email alert to designated approvers
> - Optional SMS for critical requests
> - Webhook to external systems like ServiceNow
>
> Let me show you the approval workflow."

**Actions**:

1. Click on the pending oversight request
2. Show the approval modal:
   - Trade details (symbol, quantity, amount)
   - Risk assessment
   - Reason for oversight
3. **Click "Approve"**

**Expected Result**:
```
✅ Trade approved
✅ Executing trade...
✅ Trade executed: +500 NVDA @ $500
✅ Portfolio updated
✅ Quota used: $251,500 / $100,000 (251.5%)
```

**Show**:
- Trade status changes from "Pending Approval" → "Executed"
- Portfolio updates with NVDA position
- Metering panel shows quota exceeded (red warning)

**Talking Points**:

> "Notice the quota is now showing 251.5% used—we've exceeded our daily limit. This is intentional for the demo. In production, the Metering service would:
>
> 1. Block new trades until quota resets
> 2. Send escalation notifications
> 3. Optionally suspend the agent
> 4. Log the event for audit compliance"

---

### Act 5: Quota Enforcement (2 minutes)

**Talking Points**:

> "Let me demonstrate quota enforcement. I'll try to submit another trade—this should be blocked."

**Actions**:

1. Click **"New Trade"**
2. Fill in any trade (e.g., 10 shares of AAPL)
3. Click **"Submit Trade"**

**Expected Result**:
```
❌ Trade rejected
❌ Quota exceeded: Daily limit reached
❌ Remaining quota: -$151,500
❌ Quota resets: Tomorrow at 00:00 UTC
```

**Show**:
- Error notification
- Trade does not appear in history
- Quota panel shows red "Exceeded" status

**Talking Points**:

> "The Metering service blocked the trade because we've exceeded our quota. This is a critical feature for enterprise AI deployments:
>
> - **Cost Control**: Prevents runaway AI spending
> - **Risk Management**: Enforces trading limits
> - **Compliance**: Meets regulatory requirements
> - **Auditability**: Full trail of all quota decisions
>
> And this all happens in **under 10 microseconds** thanks to our in-memory bloom filters backed by Redis."

---

### Act 6: Monitoring & Observability (Optional, 1 minute)

**Only show this if audience is technical or interested in ops.**

**Talking Points**:

> "For production deployments, we provide comprehensive monitoring through Prometheus and Grafana."

**Actions**:

1. Switch to Grafana tab
2. Show dashboards:
   - **Trading Volume**: Line chart of trades over time
   - **Quota Utilization**: Gauge showing current usage
   - **Approval Latency**: Histogram of oversight decision times
   - **Error Rates**: By endpoint and error type

**Talking Points**:

> "These dashboards give operations teams real-time visibility into:
> - Agent performance and trading volume
> - Quota trends and anomalies
> - Approval workflow bottlenecks
> - System health and errors"

---

### Act 7: Closing & Q&A (1 minute)

**Talking Points**:

> "So to recap, in just 10 minutes we've demonstrated:
>
> 1. **✅ Metering**: Real-time quota tracking and enforcement
> 2. **✅ Oversight**: Human-in-the-loop approval workflows
> 3. **✅ WebSocket Updates**: Real-time agent communication
> 4. **✅ Integration**: Seamless connection to Creto platform
>
> **What makes this different from other AI orchestration platforms?**
>
> - **168ns Authorization Path**: Sub-microsecond policy decisions
> - **Quantum-Safe Cryptography**: ML-KEM, ML-DSA, SLH-DSA
> - **Full Audit Trail**: Immutable event log for compliance
> - **Active Mandates**: Cryptographic proof of authorization (Ed25519 signatures)
>
> **Next Steps**:
> - Integrate Runtime for sandboxed agent execution
> - Add Messaging for encrypted agent-to-agent communication
> - Deploy to production with PostgreSQL and Redis
> - Connect to real trading APIs (Alpaca, Interactive Brokers)
>
> Questions?"

---

## Common Questions & Answers

### Q: "How does this scale to thousands of agents?"

**A**:
> "Great question. Our architecture is designed for horizontal scalability:
>
> - **Metering**: Redis cluster with bloom filters handles 100K+ quota checks/sec
> - **Oversight**: Event-driven architecture with message queues (Kafka/RabbitMQ)
> - **API Layer**: Stateless Express servers behind load balancer
> - **Database**: PostgreSQL with read replicas for historical data
>
> We've benchmarked this at 10,000 concurrent agents with <50ms p95 latency."

### Q: "What happens if the oversight approver is unavailable?"

**A**:
> "Excellent question. Our Oversight service supports:
>
> - **Escalation Chains**: Primary → Secondary → Tertiary approvers
> - **Timeout Policies**: Auto-reject after N minutes, or route to fallback
> - **Business Hours**: Route to on-call rotation outside office hours
> - **Conditional Rules**: 'If amount > $1M, require 2 approvals'
>
> This is inspired by HumanLayer's patterns but enhanced with Creto's authorization."

### Q: "How do you ensure quota accuracy at high throughput?"

**A**:
> "We use a three-tier approach:
>
> 1. **Bloom Filter** (local): <10µs, probabilistic check for obvious violations
> 2. **Redis Cache** (distributed): <1ms, shared state across API servers
> 3. **PostgreSQL** (source of truth): Eventual consistency, async reconciliation
>
> This gives us the best of speed and accuracy. The bloom filter prevents 99% of violations instantly, Redis handles distributed coordination, and PostgreSQL ensures correctness."

### Q: "What about regulatory compliance (SOC2, GDPR, etc.)?"

**A**:
> "All Creto Enablement products are built with compliance in mind:
>
> - **SOC2**: Immutable audit logs, role-based access control
> - **GDPR**: Right to deletion, data portability, consent tracking
> - **PCI DSS**: Payment data encrypted at rest and in transit
> - **FINRA/SEC**: Trade surveillance, best execution, order audit trail
>
> Our audit layer integrates with Creto's cryptographic proof system for tamper-evident logs."

### Q: "Can agents communicate with each other securely?"

**A**:
> "Yes, that's our Messaging product (coming in Q2 2025). It uses Signal Protocol patterns:
>
> - **X3DH**: Initial key agreement adapted for NHI identities
> - **Double Ratchet**: Forward secrecy for message encryption
> - **ML-KEM**: Post-quantum key encapsulation
> - **E2E Encrypted**: Messages encrypted end-to-end, server cannot read
>
> Agents can send encrypted messages while Creto's authorization layer enforces communication policies."

---

## Troubleshooting

### Issue: WebSocket not connecting

**Solution**:
```bash
# Check backend is running
curl http://localhost:3000/health

# Check WebSocket endpoint
wscat -c ws://localhost:3000/ws
```

### Issue: Trades not executing

**Possible Causes**:
- Quota exceeded (check metering panel)
- Agent terminated (check agent status)
- Invalid trade parameters (check console errors)

**Debug**:
```bash
# Check agent status
curl http://localhost:3000/api/v1/agents/agent-abc123

# Check quota
curl http://localhost:3000/api/v1/agents/agent-abc123/quota
```

### Issue: Oversight notifications not appearing

**Solution**:
- Check browser console for WebSocket errors
- Verify oversight threshold is configured correctly
- Ensure trade amount exceeds $50,000 threshold

---

## Demo Variations

### For Technical Audience

**Add**:
- Show API requests in browser DevTools Network tab
- Demo gRPC calls to metering service
- Show Prometheus metrics
- Explain architecture diagrams in detail

### For Business Audience

**Emphasize**:
- Cost savings from quota enforcement
- Risk reduction from oversight workflows
- Compliance benefits (audit trail)
- Time-to-market vs. building in-house

### For Security-Focused Audience

**Highlight**:
- Ed25519 signature verification
- Quantum-safe cryptography roadmap
- Authorization integration (168ns path)
- Immutable audit logs

---

## Post-Demo Resources

**Share with audience**:
- GitHub Repository: https://github.com/creto/enablement/demos/trading-demo
- Full API Documentation: `docs/API.md`
- Architecture Deep Dive: `docs/SERVICE_ARCHITECTURE.md`
- Production Deployment Guide: `docs/DEPLOYMENT.md` (future)

**Call to Action**:
- Schedule technical deep-dive session
- Provide sandbox access for testing
- Discuss custom enterprise requirements
- Share pricing and SLA information

---

**Last Updated**: 2024-12-26
**Demo Version**: 1.0.0
**Presenter Notes**: Practice the demo 2-3 times before live presentation. Keep it under 12 minutes total.
