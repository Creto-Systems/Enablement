# Component API Documentation

## Components

### AgentCard

Display trading agent information with budget utilization and performance metrics.

**Props:**
```typescript
interface AgentCardProps {
  agent: Agent;
  onSelect: (agentId: string) => void;
  selected?: boolean;
}

interface Agent {
  id: string;
  name: string;
  status: 'active' | 'inactive' | 'paused';
  budget: {
    total: number;
    used: number;
    remaining: number;
  };
  performance: {
    dailyPnL: number;
    totalPnL: number;
    winRate: number;
  };
}
```

**Example:**
```tsx
import { AgentCard } from './components';

<AgentCard
  agent={{
    id: 'agent-1',
    name: 'Conservative Trader',
    status: 'active',
    budget: { total: 10000, used: 3500, remaining: 6500 },
    performance: { dailyPnL: 234.56, totalPnL: 1250, winRate: 0.68 }
  }}
  onSelect={(id) => console.log('Selected:', id)}
  selected={false}
/>
```

**Features:**
- Color-coded budget utilization bar
- P&L display with conditional coloring
- Win rate percentage
- Keyboard accessible (Enter/Space)
- ARIA compliant

---

### PortfolioChart

Visualize portfolio value over time with performance metrics.

**Props:**
```typescript
interface PortfolioChartProps {
  data: PortfolioDataPoint[];
  height?: number;
}

interface PortfolioDataPoint {
  timestamp: string;
  value: number;
}
```

**Example:**
```tsx
import { PortfolioChart } from './components';

<PortfolioChart
  data={[
    { timestamp: '2024-01-01T00:00:00Z', value: 10000 },
    { timestamp: '2024-01-02T00:00:00Z', value: 10500 },
    { timestamp: '2024-01-03T00:00:00Z', value: 12050 }
  ]}
  height={400}
/>
```

**Features:**
- Responsive line chart
- Interactive tooltip
- Automatic gain/loss calculation
- Empty state handling
- Currency formatting

---

### TradeForm

Form for submitting trade requests with validation and warnings.

**Props:**
```typescript
interface TradeFormProps {
  agentId: string;
  onSubmit: (data: TradeFormData) => void | Promise<void>;
  loading?: boolean;
  maxTradeSize?: number;
}

interface TradeFormData {
  agentId: string;
  symbol: string;
  side: 'buy' | 'sell';
  quantity: number;
  price: number;
  reasoning: string;
}
```

**Example:**
```tsx
import { TradeForm } from './components';

<TradeForm
  agentId="agent-1"
  onSubmit={async (data) => {
    await submitTrade(data);
  }}
  loading={false}
  maxTradeSize={10000}
/>
```

**Features:**
- Real-time validation
- Total value calculation
- Large trade warnings
- Symbol format validation
- Auto-reset on success

---

### UsageMeter

Display resource usage with color-coded thresholds.

**Props:**
```typescript
interface UsageMeterProps {
  used: number;
  total: number;
  label: string;
  showWarning?: boolean;
}
```

**Example:**
```tsx
import { UsageMeter } from './components';

<UsageMeter
  used={3500}
  total={10000}
  label="Budget"
  showWarning={true}
/>
```

**Features:**
- Color thresholds (green < 60%, yellow 60-79%, red ≥ 80%)
- Warning icon at 80%
- Critical alert at 90%
- Accessible progress bar
- Remaining value display

---

### ApprovalCard

Review and approve/reject pending trades with risk assessment.

**Props:**
```typescript
interface ApprovalCardProps {
  approval: PendingApproval;
  onApprove: (approvalId: string, reason?: string) => void | Promise<void>;
  onReject: (approvalId: string, reason: string) => void | Promise<void>;
  loading?: boolean;
}

interface PendingApproval {
  id: string;
  agentId: string;
  symbol: string;
  side: 'buy' | 'sell';
  quantity: number;
  price: number;
  estimatedCost: number;
  timestamp: string;
  reasoning: string;
  riskAssessment: {
    score: number;
    level: 'low' | 'medium' | 'high';
    factors: string[];
  };
}
```

**Example:**
```tsx
import { ApprovalCard } from './components';

<ApprovalCard
  approval={pendingTrade}
  onApprove={async (id, reason) => {
    await approveTrade(id, reason);
  }}
  onReject={async (id, reason) => {
    await rejectTrade(id, reason);
  }}
  loading={false}
/>
```

**Features:**
- Risk level visualization
- Trade details summary
- Modal dialogs for approval/rejection
- Required rejection reason
- Optional approval notes

---

## Custom Hooks

### usePortfolio

Fetch and subscribe to real-time portfolio updates.

**Signature:**
```typescript
function usePortfolio(agentId: string): {
  data?: PortfolioData;
  isLoading: boolean;
  isError: boolean;
  error: Error | null;
  unrealizedPnL: number;
  totalPnL: number;
  portfolioAllocation: Array<{
    symbol: string;
    value: number;
    percentage: number;
  }>;
  isConnected: boolean;
}
```

**Example:**
```tsx
import { usePortfolio } from './hooks';

function Portfolio({ agentId }) {
  const {
    data,
    isLoading,
    unrealizedPnL,
    totalPnL,
    portfolioAllocation,
    isConnected
  } = usePortfolio(agentId);

  if (isLoading) return <div>Loading...</div>;

  return (
    <div>
      <h2>Total Value: ${data.totalValue}</h2>
      <p>Unrealized P&L: ${unrealizedPnL}</p>
      <p>WebSocket: {isConnected ? 'Connected' : 'Disconnected'}</p>
    </div>
  );
}
```

**Features:**
- React Query integration
- WebSocket real-time updates
- Automatic P&L calculations
- Portfolio allocation analysis
- Connection status tracking

---

### useTrade

Submit and manage trades with optimistic updates.

**Signature:**
```typescript
function useTrade(agentId: string): {
  submitTrade: (data: TradeData) => void;
  cancelTrade: (tradeId: string) => Promise<void>;
  updateTradeStatus: (update: { id: string; status: Trade['status'] }) => void;
  calculateTradeMetrics: (data: TradeData) => TradeMetrics;
  pendingTrades: Map<string, Trade>;
  isPending: boolean;
  isSuccess: boolean;
  isError: boolean;
  error: Error | null;
}
```

**Example:**
```tsx
import { useTrade } from './hooks';

function TradingPanel({ agentId }) {
  const {
    submitTrade,
    cancelTrade,
    calculateTradeMetrics,
    pendingTrades,
    isPending
  } = useTrade(agentId);

  const handleSubmit = (data) => {
    const metrics = calculateTradeMetrics(data);
    console.log('Trade will cost:', metrics.netCost);
    submitTrade(data);
  };

  return (
    <div>
      <TradeForm onSubmit={handleSubmit} loading={isPending} />
      <PendingTradesList trades={Array.from(pendingTrades.values())} />
    </div>
  );
}
```

**Features:**
- Optimistic UI updates
- Automatic rollback on error
- Portfolio query invalidation
- Trade validation
- Metrics calculation (fees, total cost)
- Cancel pending trades

---

## Testing

All components include comprehensive tests with:
- Unit tests for logic
- Integration tests for user flows
- Accessibility tests (jest-axe)
- Mock data utilities
- WebSocket mocking

**Run tests:**
```bash
npm test                 # Run all tests
npm test:coverage        # Generate coverage report
npm test:ui              # Open Vitest UI
```

**Coverage targets:** 90% for lines, functions, branches, and statements
