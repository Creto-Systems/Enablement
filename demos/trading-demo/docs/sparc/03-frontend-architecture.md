# Frontend Architecture - Trading Demo

## Architecture Decision Records (ADRs)

### ADR-001: React 18 with TypeScript
**Status**: Accepted
**Date**: 2025-12-26

**Context**: Need a modern, type-safe frontend framework with excellent performance and developer experience.

**Decision**: Use React 18 with TypeScript, leveraging concurrent features and automatic batching.

**Rationale**:
- React 18's concurrent rendering improves responsiveness for real-time data
- TypeScript provides compile-time safety and excellent IDE support
- Large ecosystem with mature tooling
- Team expertise and hiring pool

**Consequences**:
- Positive: Type safety, better refactoring, improved DX
- Negative: Learning curve for advanced concurrent features
- Mitigation: Gradual adoption of concurrent features, comprehensive training

---

### ADR-002: Zustand for State Management
**Status**: Accepted
**Date**: 2025-12-26

**Context**: Need lightweight, performant state management without Redux boilerplate.

**Decision**: Use Zustand for global state management.

**Rationale**:
- Minimal boilerplate compared to Redux
- Built-in TypeScript support
- No context provider wrapping needed
- Excellent performance with automatic shallow equality
- Easy to debug and test

**Consequences**:
- Positive: Faster development, less code, better performance
- Negative: Less middleware ecosystem than Redux
- Mitigation: Use middleware pattern for persistence, logging

**Alternatives Considered**:
- Redux Toolkit: Too much boilerplate for our scale
- Jotai/Recoil: Atomic approach not needed for our data model
- Context + useReducer: Performance concerns with frequent updates

---

### ADR-003: TanStack Query for Server State
**Status**: Accepted
**Date**: 2025-12-26

**Context**: Need robust data fetching, caching, and synchronization with backend.

**Decision**: Use TanStack Query (React Query) for all server state management.

**Rationale**:
- Automatic caching and invalidation
- Optimistic updates for trades
- Excellent TypeScript support
- Built-in retry, polling, and refetching
- DevTools for debugging

**Consequences**:
- Positive: Less manual cache management, better UX
- Negative: Additional dependency
- Mitigation: Well-maintained, industry standard library

---

### ADR-004: Recharts for Data Visualization
**Status**: Accepted
**Date**: 2025-12-26

**Context**: Need accessible, performant charts for portfolio and usage data.

**Decision**: Use Recharts for all data visualizations.

**Rationale**:
- Built on D3 with simpler API
- Composable React components
- Responsive by default
- Good accessibility support
- Active maintenance

**Consequences**:
- Positive: Fast implementation, good DX
- Negative: Less customization than raw D3
- Mitigation: Use D3 directly for complex custom visualizations if needed

**Alternatives Considered**:
- Victory: Less popular, smaller ecosystem
- Chart.js: Imperative API, harder to integrate
- Nivo: Heavier bundle size

---

## System Architecture

### High-Level Component Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         App Shell                            │
│  ┌─────────────────────────────────────────────────────┐   │
│  │                    Error Boundary                    │   │
│  │  ┌───────────────────────────────────────────────┐  │   │
│  │  │              Query Client Provider            │  │   │
│  │  │  ┌─────────────────────────────────────────┐ │  │   │
│  │  │  │            WebSocket Provider           │ │  │   │
│  │  │  │  ┌───────────────────────────────────┐ │ │  │   │
│  │  │  │  │          Theme Provider          │ │ │  │   │
│  │  │  │  │  ┌────────────────────────────┐ │ │ │  │   │
│  │  │  │  │  │        Router          │ │ │ │  │   │
│  │  │  │  │  │  ┌──────────────────┐ │ │ │ │  │   │
│  │  │  │  │  │  │   Layout        │ │ │ │ │  │   │
│  │  │  │  │  │  │  - Header       │ │ │ │ │  │   │
│  │  │  │  │  │  │  - Sidebar      │ │ │ │ │  │   │
│  │  │  │  │  │  │  - Main Content │ │ │ │ │  │   │
│  │  │  │  │  │  │  - Modals       │ │ │ │ │  │   │
│  │  │  │  │  │  └──────────────────┘ │ │ │ │  │   │
│  │  │  │  │  └────────────────────────┘ │ │ │  │   │
│  │  │  │  └───────────────────────────────┘ │ │  │   │
│  │  │  └─────────────────────────────────────┘ │  │   │
│  │  └───────────────────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Detailed Component Hierarchy

### 1. Application Shell

```typescript
// src/App.tsx
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactQueryDevtools } from '@tanstack/react-query-devtools';
import { BrowserRouter } from 'react-router-dom';
import { ErrorBoundary } from '@/components/ErrorBoundary';
import { WebSocketProvider } from '@/providers/WebSocketProvider';
import { ThemeProvider } from '@/providers/ThemeProvider';
import { AppRouter } from '@/routes';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60 * 1000, // 1 minute
      retry: 3,
      refetchOnWindowFocus: true,
    },
  },
});

function App() {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <WebSocketProvider>
          <ThemeProvider>
            <BrowserRouter>
              <AppRouter />
            </BrowserRouter>
          </ThemeProvider>
        </WebSocketProvider>
        <ReactQueryDevtools initialIsOpen={false} />
      </QueryClientProvider>
    </ErrorBoundary>
  );
}
```

### 2. Layout Components

```
Layout/
├── AppLayout.tsx          # Main application layout
├── Header/
│   ├── Header.tsx
│   ├── Navigation.tsx
│   ├── UserMenu.tsx
│   └── NotificationBell.tsx
├── Sidebar/
│   ├── Sidebar.tsx
│   ├── AgentList.tsx
│   ├── QuickActions.tsx
│   └── SystemStatus.tsx
└── Main/
    └── MainContent.tsx
```

**AppLayout Component**:
```typescript
// src/components/Layout/AppLayout.tsx
interface AppLayoutProps {
  children: React.ReactNode;
}

export function AppLayout({ children }: AppLayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(true);

  return (
    <div className="app-layout">
      <Header onMenuToggle={() => setSidebarOpen(!sidebarOpen)} />
      <div className="app-body">
        <Sidebar isOpen={sidebarOpen} />
        <main
          className="main-content"
          role="main"
          aria-label="Main content"
        >
          {children}
        </main>
      </div>
      <ModalContainer />
      <ToastContainer />
    </div>
  );
}
```

### 3. Page Components

```
pages/
├── Dashboard/
│   ├── Dashboard.tsx
│   ├── PortfolioOverview.tsx
│   ├── ActiveAgentsPanel.tsx
│   ├── RecentTradesPanel.tsx
│   └── PerformanceChart.tsx
├── AgentCreate/
│   ├── AgentCreateWizard.tsx
│   ├── steps/
│   │   ├── BasicInfoStep.tsx
│   │   ├── StrategyStep.tsx
│   │   ├── RiskStep.tsx
│   │   └── ReviewStep.tsx
│   └── AgentCreateContext.tsx
├── AgentDetail/
│   ├── AgentDetail.tsx
│   ├── AgentOverview.tsx
│   ├── AgentPerformance.tsx
│   ├── AgentTrades.tsx
│   ├── AgentSettings.tsx
│   └── AgentLogs.tsx
├── OversightQueue/
│   ├── OversightQueue.tsx
│   ├── ApprovalList.tsx
│   ├── ApprovalCard.tsx
│   └── ApprovalDetail.tsx
└── MeteringDashboard/
    ├── MeteringDashboard.tsx
    ├── UsageChart.tsx
    ├── CostBreakdown.tsx
    └── BudgetAlerts.tsx
```

### 4. Feature Components

```
features/
├── agents/
│   ├── components/
│   │   ├── AgentCard.tsx
│   │   ├── AgentStatusBadge.tsx
│   │   └── AgentActions.tsx
│   ├── hooks/
│   │   ├── useAgents.ts
│   │   ├── useAgentCreate.ts
│   │   └── useAgentUpdate.ts
│   └── types/
│       └── agent.types.ts
├── trades/
│   ├── components/
│   │   ├── TradeTable.tsx
│   │   ├── TradeRow.tsx
│   │   ├── TradeModal.tsx
│   │   └── TradeStatusBadge.tsx
│   ├── hooks/
│   │   ├── useTrades.ts
│   │   ├── useTradeSubmit.ts
│   │   └── useTradeHistory.ts
│   └── types/
│       └── trade.types.ts
├── portfolio/
│   ├── components/
│   │   ├── PortfolioChart.tsx
│   │   ├── AllocationPieChart.tsx
│   │   ├── AssetTable.tsx
│   │   └── PerformanceMetrics.tsx
│   ├── hooks/
│   │   ├── usePortfolio.ts
│   │   └── usePortfolioHistory.ts
│   └── types/
│       └── portfolio.types.ts
└── oversight/
    ├── components/
    │   ├── ApprovalQueue.tsx
    │   ├── ApprovalModal.tsx
    │   └── ApprovalHistory.tsx
    ├── hooks/
    │   ├── useApprovals.ts
    │   └── useApprovalActions.ts
    └── types/
        └── oversight.types.ts
```

---

## State Management Architecture

### Zustand Store Structure

```typescript
// src/stores/index.ts
export { useAgentStore } from './agentStore';
export { useTradeStore } from './tradeStore';
export { useUIStore } from './uiStore';
export { useUserStore } from './userStore';
```

**Agent Store**:
```typescript
// src/stores/agentStore.ts
import { create } from 'zustand';
import { devtools, persist } from 'zustand/middleware';

interface AgentStore {
  // State
  selectedAgentId: string | null;
  agentFilters: AgentFilters;

  // Actions
  selectAgent: (id: string | null) => void;
  setFilters: (filters: Partial<AgentFilters>) => void;
  clearFilters: () => void;
}

export const useAgentStore = create<AgentStore>()(
  devtools(
    persist(
      (set) => ({
        selectedAgentId: null,
        agentFilters: {
          status: 'all',
          strategy: 'all',
          sortBy: 'created_at',
        },

        selectAgent: (id) => set({ selectedAgentId: id }),
        setFilters: (filters) =>
          set((state) => ({
            agentFilters: { ...state.agentFilters, ...filters }
          })),
        clearFilters: () => set({ agentFilters: DEFAULT_FILTERS }),
      }),
      { name: 'agent-store' }
    )
  )
);
```

**UI Store**:
```typescript
// src/stores/uiStore.ts
interface UIStore {
  // Modal state
  activeModal: ModalType | null;
  modalData: any;

  // Sidebar state
  sidebarOpen: boolean;
  sidebarCollapsed: boolean;

  // Theme
  theme: 'light' | 'dark' | 'system';

  // Actions
  openModal: (type: ModalType, data?: any) => void;
  closeModal: () => void;
  toggleSidebar: () => void;
  setTheme: (theme: 'light' | 'dark' | 'system') => void;
}

export const useUIStore = create<UIStore>()(
  persist(
    (set) => ({
      activeModal: null,
      modalData: null,
      sidebarOpen: true,
      sidebarCollapsed: false,
      theme: 'system',

      openModal: (type, data) => set({ activeModal: type, modalData: data }),
      closeModal: () => set({ activeModal: null, modalData: null }),
      toggleSidebar: () => set((state) => ({
        sidebarOpen: !state.sidebarOpen
      })),
      setTheme: (theme) => set({ theme }),
    }),
    { name: 'ui-store' }
  )
);
```

### TanStack Query Structure

```typescript
// src/api/queries/agents.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { agentApi } from '@/api/services/agentService';

export const agentKeys = {
  all: ['agents'] as const,
  lists: () => [...agentKeys.all, 'list'] as const,
  list: (filters: AgentFilters) => [...agentKeys.lists(), filters] as const,
  details: () => [...agentKeys.all, 'detail'] as const,
  detail: (id: string) => [...agentKeys.details(), id] as const,
  trades: (id: string) => [...agentKeys.detail(id), 'trades'] as const,
  portfolio: (id: string) => [...agentKeys.detail(id), 'portfolio'] as const,
};

export function useAgents(filters: AgentFilters) {
  return useQuery({
    queryKey: agentKeys.list(filters),
    queryFn: () => agentApi.getAgents(filters),
    staleTime: 30 * 1000, // 30 seconds
  });
}

export function useAgent(id: string) {
  return useQuery({
    queryKey: agentKeys.detail(id),
    queryFn: () => agentApi.getAgent(id),
    enabled: !!id,
  });
}

export function useCreateAgent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: agentApi.createAgent,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: agentKeys.lists() });
    },
  });
}

export function useAgentTrades(agentId: string) {
  return useQuery({
    queryKey: agentKeys.trades(agentId),
    queryFn: () => agentApi.getAgentTrades(agentId),
    enabled: !!agentId,
    refetchInterval: 5000, // Poll every 5 seconds
  });
}
```

---

## Routing Structure

### Route Configuration

```typescript
// src/routes/index.tsx
import { Routes, Route, Navigate } from 'react-router-dom';
import { AppLayout } from '@/components/Layout/AppLayout';
import { Dashboard } from '@/pages/Dashboard';
import { AgentCreateWizard } from '@/pages/AgentCreate';
import { AgentDetail } from '@/pages/AgentDetail';
import { OversightQueue } from '@/pages/OversightQueue';
import { MeteringDashboard } from '@/pages/MeteringDashboard';
import { NotFound } from '@/pages/NotFound';

export function AppRouter() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route path="/" element={<Dashboard />} />

        {/* Agent Routes */}
        <Route path="/agents">
          <Route index element={<Navigate to="/" replace />} />
          <Route path="new" element={<AgentCreateWizard />} />
          <Route path=":agentId" element={<AgentDetail />}>
            <Route index element={<Navigate to="overview" replace />} />
            <Route path="overview" element={<AgentOverview />} />
            <Route path="trades" element={<AgentTrades />} />
            <Route path="performance" element={<AgentPerformance />} />
            <Route path="settings" element={<AgentSettings />} />
            <Route path="logs" element={<AgentLogs />} />
          </Route>
        </Route>

        {/* Oversight Routes */}
        <Route path="/oversight">
          <Route index element={<OversightQueue />} />
          <Route path=":approvalId" element={<ApprovalDetail />} />
        </Route>

        {/* Metering Routes */}
        <Route path="/metering" element={<MeteringDashboard />} />

        {/* 404 */}
        <Route path="*" element={<NotFound />} />
      </Route>
    </Routes>
  );
}
```

---

## Custom Hooks

### WebSocket Hook

```typescript
// src/hooks/useWebSocket.ts
import { useEffect, useRef, useCallback } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { agentKeys } from '@/api/queries/agents';

interface WebSocketMessage {
  type: 'TRADE_UPDATE' | 'PORTFOLIO_UPDATE' | 'AGENT_STATUS';
  payload: any;
}

export function useWebSocket(url: string) {
  const ws = useRef<WebSocket | null>(null);
  const queryClient = useQueryClient();
  const reconnectTimeout = useRef<NodeJS.Timeout>();

  const connect = useCallback(() => {
    ws.current = new WebSocket(url);

    ws.current.onmessage = (event) => {
      const message: WebSocketMessage = JSON.parse(event.data);

      switch (message.type) {
        case 'TRADE_UPDATE':
          queryClient.invalidateQueries({
            queryKey: agentKeys.trades(message.payload.agentId)
          });
          break;

        case 'PORTFOLIO_UPDATE':
          queryClient.setQueryData(
            agentKeys.portfolio(message.payload.agentId),
            message.payload.portfolio
          );
          break;

        case 'AGENT_STATUS':
          queryClient.invalidateQueries({
            queryKey: agentKeys.detail(message.payload.agentId)
          });
          break;
      }
    };

    ws.current.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    ws.current.onclose = () => {
      // Reconnect after 5 seconds
      reconnectTimeout.current = setTimeout(connect, 5000);
    };
  }, [url, queryClient]);

  useEffect(() => {
    connect();

    return () => {
      if (reconnectTimeout.current) {
        clearTimeout(reconnectTimeout.current);
      }
      ws.current?.close();
    };
  }, [connect]);

  return ws;
}
```

### Portfolio Hook

```typescript
// src/hooks/usePortfolio.ts
import { useQuery } from '@tanstack/react-query';
import { portfolioApi } from '@/api/services/portfolioService';
import { useWebSocket } from './useWebSocket';

export function usePortfolio(agentId: string) {
  const { data, isLoading, error } = useQuery({
    queryKey: agentKeys.portfolio(agentId),
    queryFn: () => portfolioApi.getPortfolio(agentId),
    enabled: !!agentId,
  });

  // Real-time updates via WebSocket
  useWebSocket(`${WS_URL}/portfolio/${agentId}`);

  return {
    portfolio: data,
    isLoading,
    error,
  };
}
```

### Trade Submission Hook

```typescript
// src/hooks/useTradeSubmit.ts
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { tradeApi } from '@/api/services/tradeService';
import { agentKeys } from '@/api/queries/agents';
import { useToast } from './useToast';

export function useTradeSubmit(agentId: string) {
  const queryClient = useQueryClient();
  const toast = useToast();

  return useMutation({
    mutationFn: tradeApi.submitTrade,

    // Optimistic update
    onMutate: async (newTrade) => {
      await queryClient.cancelQueries({
        queryKey: agentKeys.trades(agentId)
      });

      const previousTrades = queryClient.getQueryData(
        agentKeys.trades(agentId)
      );

      queryClient.setQueryData(
        agentKeys.trades(agentId),
        (old: any) => [...(old || []), {
          ...newTrade,
          id: 'temp-' + Date.now(),
          status: 'pending'
        }]
      );

      return { previousTrades };
    },

    onError: (err, newTrade, context) => {
      queryClient.setQueryData(
        agentKeys.trades(agentId),
        context?.previousTrades
      );
      toast.error('Trade submission failed');
    },

    onSuccess: () => {
      toast.success('Trade submitted successfully');
    },

    onSettled: () => {
      queryClient.invalidateQueries({
        queryKey: agentKeys.trades(agentId)
      });
    },
  });
}
```

### Oversight Hook

```typescript
// src/hooks/useOversight.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { oversightApi } from '@/api/services/oversightService';

const oversightKeys = {
  all: ['oversight'] as const,
  pending: () => [...oversightKeys.all, 'pending'] as const,
  detail: (id: string) => [...oversightKeys.all, 'detail', id] as const,
};

export function usePendingApprovals() {
  return useQuery({
    queryKey: oversightKeys.pending(),
    queryFn: oversightApi.getPendingApprovals,
    refetchInterval: 10000, // Poll every 10 seconds
  });
}

export function useApproveAction() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ id, decision }: { id: string; decision: 'approve' | 'deny' }) =>
      oversightApi.approveAction(id, decision),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: oversightKeys.pending() });
    },
  });
}
```

---

## Chart Components

### Portfolio Chart (Recharts)

```typescript
// src/components/charts/PortfolioChart.tsx
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer
} from 'recharts';

interface PortfolioChartProps {
  data: PortfolioHistory[];
  timeRange: '1D' | '1W' | '1M' | '3M' | '1Y';
}

export function PortfolioChart({ data, timeRange }: PortfolioChartProps) {
  return (
    <ResponsiveContainer width="100%" height={400}>
      <LineChart
        data={data}
        margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
      >
        <CartesianGrid strokeDasharray="3 3" />
        <XAxis
          dataKey="timestamp"
          tickFormatter={(ts) => formatDate(ts, timeRange)}
        />
        <YAxis
          tickFormatter={(value) => `$${value.toLocaleString()}`}
        />
        <Tooltip
          labelFormatter={(ts) => formatDate(ts, 'full')}
          formatter={(value: number) => [`$${value.toFixed(2)}`, 'Value']}
        />
        <Legend />
        <Line
          type="monotone"
          dataKey="totalValue"
          stroke="#8884d8"
          strokeWidth={2}
          dot={false}
        />
      </LineChart>
    </ResponsiveContainer>
  );
}
```

### Allocation Pie Chart

```typescript
// src/components/charts/AllocationPieChart.tsx
import { PieChart, Pie, Cell, Tooltip, Legend, ResponsiveContainer } from 'recharts';

interface AllocationData {
  symbol: string;
  value: number;
  percentage: number;
}

const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042', '#8884D8'];

export function AllocationPieChart({ data }: { data: AllocationData[] }) {
  return (
    <ResponsiveContainer width="100%" height={300}>
      <PieChart>
        <Pie
          data={data}
          cx="50%"
          cy="50%"
          labelLine={false}
          label={({ symbol, percentage }) => `${symbol} ${percentage}%`}
          outerRadius={80}
          fill="#8884d8"
          dataKey="value"
        >
          {data.map((entry, index) => (
            <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
          ))}
        </Pie>
        <Tooltip formatter={(value: number) => `$${value.toLocaleString()}`} />
        <Legend />
      </PieChart>
    </ResponsiveContainer>
  );
}
```

---

## Form Handling

### Agent Create Form (Multi-Step Wizard)

```typescript
// src/pages/AgentCreate/AgentCreateWizard.tsx
import { useForm, FormProvider } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { agentCreateSchema } from '@/schemas/agentSchema';
import { useState } from 'react';

const STEPS = [
  { id: 'basic', title: 'Basic Information', component: BasicInfoStep },
  { id: 'strategy', title: 'Trading Strategy', component: StrategyStep },
  { id: 'risk', title: 'Risk Management', component: RiskStep },
  { id: 'review', title: 'Review & Create', component: ReviewStep },
];

export function AgentCreateWizard() {
  const [currentStep, setCurrentStep] = useState(0);
  const createAgent = useCreateAgent();

  const methods = useForm({
    resolver: zodResolver(agentCreateSchema),
    mode: 'onChange',
    defaultValues: {
      name: '',
      description: '',
      strategy: 'balanced',
      initialBalance: 10000,
      riskTolerance: 'medium',
      maxDrawdown: 0.2,
      stopLoss: 0.05,
    },
  });

  const onSubmit = async (data: AgentCreateData) => {
    try {
      await createAgent.mutateAsync(data);
      toast.success('Agent created successfully');
      navigate('/');
    } catch (error) {
      toast.error('Failed to create agent');
    }
  };

  const CurrentStepComponent = STEPS[currentStep].component;

  return (
    <FormProvider {...methods}>
      <form onSubmit={methods.handleSubmit(onSubmit)}>
        <WizardHeader steps={STEPS} currentStep={currentStep} />

        <CurrentStepComponent />

        <WizardFooter
          currentStep={currentStep}
          totalSteps={STEPS.length}
          onNext={() => setCurrentStep(prev => prev + 1)}
          onPrev={() => setCurrentStep(prev => prev - 1)}
          isValid={methods.formState.isValid}
          isSubmitting={createAgent.isPending}
        />
      </form>
    </FormProvider>
  );
}
```

**Form Schema (Zod)**:
```typescript
// src/schemas/agentSchema.ts
import { z } from 'zod';

export const agentCreateSchema = z.object({
  name: z.string()
    .min(3, 'Name must be at least 3 characters')
    .max(50, 'Name must be less than 50 characters'),

  description: z.string()
    .min(10, 'Description must be at least 10 characters')
    .max(500, 'Description must be less than 500 characters'),

  strategy: z.enum(['aggressive', 'balanced', 'conservative']),

  initialBalance: z.number()
    .min(1000, 'Minimum balance is $1,000')
    .max(1000000, 'Maximum balance is $1,000,000'),

  riskTolerance: z.enum(['low', 'medium', 'high']),

  maxDrawdown: z.number()
    .min(0.05, 'Minimum drawdown is 5%')
    .max(0.5, 'Maximum drawdown is 50%'),

  stopLoss: z.number()
    .min(0.01, 'Minimum stop loss is 1%')
    .max(0.2, 'Maximum stop loss is 20%'),
});
```

---

## Accessibility Implementation

### ARIA Labels and Roles

```typescript
// src/components/agents/AgentCard.tsx
export function AgentCard({ agent }: { agent: Agent }) {
  return (
    <article
      className="agent-card"
      role="article"
      aria-labelledby={`agent-${agent.id}-name`}
      aria-describedby={`agent-${agent.id}-desc`}
    >
      <h3 id={`agent-${agent.id}-name`}>{agent.name}</h3>
      <p id={`agent-${agent.id}-desc`}>{agent.description}</p>

      <div role="status" aria-live="polite">
        <AgentStatusBadge status={agent.status} />
      </div>

      <nav aria-label="Agent actions">
        <button
          onClick={() => handleView(agent.id)}
          aria-label={`View details for ${agent.name}`}
        >
          View Details
        </button>
        <button
          onClick={() => handleEdit(agent.id)}
          aria-label={`Edit ${agent.name}`}
        >
          Edit
        </button>
      </nav>
    </article>
  );
}
```

### Keyboard Navigation

```typescript
// src/hooks/useKeyboardNav.ts
export function useKeyboardNav(items: any[], onSelect: (item: any) => void) {
  const [focusIndex, setFocusIndex] = useState(0);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      switch (e.key) {
        case 'ArrowDown':
          e.preventDefault();
          setFocusIndex(prev => Math.min(prev + 1, items.length - 1));
          break;
        case 'ArrowUp':
          e.preventDefault();
          setFocusIndex(prev => Math.max(prev - 1, 0));
          break;
        case 'Enter':
        case ' ':
          e.preventDefault();
          onSelect(items[focusIndex]);
          break;
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [items, focusIndex, onSelect]);

  return { focusIndex, setFocusIndex };
}
```

### Screen Reader Announcements

```typescript
// src/hooks/useAnnouncement.ts
export function useAnnouncement() {
  const announce = useCallback((message: string, priority: 'polite' | 'assertive' = 'polite') => {
    const announcement = document.createElement('div');
    announcement.setAttribute('role', 'status');
    announcement.setAttribute('aria-live', priority);
    announcement.className = 'sr-only';
    announcement.textContent = message;

    document.body.appendChild(announcement);

    setTimeout(() => {
      document.body.removeChild(announcement);
    }, 1000);
  }, []);

  return { announce };
}

// Usage
function TradeSubmit() {
  const { announce } = useAnnouncement();
  const submitTrade = useTradeSubmit();

  const handleSubmit = async (data) => {
    await submitTrade.mutateAsync(data);
    announce('Trade submitted successfully', 'polite');
  };
}
```

---

## Performance Optimization

### Code Splitting Strategy

```typescript
// src/routes/index.tsx
import { lazy, Suspense } from 'react';
import { LoadingSpinner } from '@/components/LoadingSpinner';

// Route-based code splitting
const Dashboard = lazy(() => import('@/pages/Dashboard'));
const AgentCreateWizard = lazy(() => import('@/pages/AgentCreate/AgentCreateWizard'));
const AgentDetail = lazy(() => import('@/pages/AgentDetail'));
const OversightQueue = lazy(() => import('@/pages/OversightQueue'));
const MeteringDashboard = lazy(() => import('@/pages/MeteringDashboard'));

export function AppRouter() {
  return (
    <Routes>
      <Route element={<AppLayout />}>
        <Route
          path="/"
          element={
            <Suspense fallback={<LoadingSpinner />}>
              <Dashboard />
            </Suspense>
          }
        />
        {/* Other routes... */}
      </Route>
    </Routes>
  );
}
```

### Component Memoization

```typescript
// src/components/agents/AgentCard.tsx
import { memo } from 'react';

interface AgentCardProps {
  agent: Agent;
  onSelect: (id: string) => void;
}

export const AgentCard = memo(function AgentCard({
  agent,
  onSelect
}: AgentCardProps) {
  return (
    <article className="agent-card">
      {/* Component content */}
    </article>
  );
}, (prevProps, nextProps) => {
  // Custom comparison function
  return (
    prevProps.agent.id === nextProps.agent.id &&
    prevProps.agent.status === nextProps.agent.status &&
    prevProps.agent.updatedAt === nextProps.agent.updatedAt
  );
});
```

### Virtual Scrolling for Large Lists

```typescript
// src/components/trades/TradeList.tsx
import { useVirtualizer } from '@tanstack/react-virtual';
import { useRef } from 'react';

export function TradeList({ trades }: { trades: Trade[] }) {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: trades.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 60, // Estimated row height
    overscan: 5, // Number of items to render outside viewport
  });

  return (
    <div
      ref={parentRef}
      style={{ height: '600px', overflow: 'auto' }}
    >
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative',
        }}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => (
          <div
            key={virtualRow.key}
            style={{
              position: 'absolute',
              top: 0,
              left: 0,
              width: '100%',
              height: `${virtualRow.size}px`,
              transform: `translateY(${virtualRow.start}px)`,
            }}
          >
            <TradeRow trade={trades[virtualRow.index]} />
          </div>
        ))}
      </div>
    </div>
  );
}
```

### Query Prefetching

```typescript
// src/components/agents/AgentCard.tsx
import { useQueryClient } from '@tanstack/react-query';

export function AgentCard({ agent }: { agent: Agent }) {
  const queryClient = useQueryClient();

  const handleMouseEnter = () => {
    // Prefetch agent details on hover
    queryClient.prefetchQuery({
      queryKey: agentKeys.detail(agent.id),
      queryFn: () => agentApi.getAgent(agent.id),
    });
  };

  return (
    <article onMouseEnter={handleMouseEnter}>
      {/* Component content */}
    </article>
  );
}
```

---

## Testing Strategy

### Component Testing (Vitest + React Testing Library)

```typescript
// src/components/agents/__tests__/AgentCard.test.tsx
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { AgentCard } from '../AgentCard';

describe('AgentCard', () => {
  const mockAgent = {
    id: '123',
    name: 'Test Agent',
    status: 'active',
    strategy: 'balanced',
  };

  it('renders agent information correctly', () => {
    render(<AgentCard agent={mockAgent} onSelect={vi.fn()} />);

    expect(screen.getByText('Test Agent')).toBeInTheDocument();
    expect(screen.getByText('Active')).toBeInTheDocument();
  });

  it('calls onSelect when view button is clicked', () => {
    const onSelect = vi.fn();
    render(<AgentCard agent={mockAgent} onSelect={onSelect} />);

    fireEvent.click(screen.getByText('View Details'));
    expect(onSelect).toHaveBeenCalledWith('123');
  });

  it('is keyboard accessible', () => {
    const onSelect = vi.fn();
    render(<AgentCard agent={mockAgent} onSelect={onSelect} />);

    const button = screen.getByText('View Details');
    button.focus();
    expect(button).toHaveFocus();

    fireEvent.keyDown(button, { key: 'Enter' });
    expect(onSelect).toHaveBeenCalledWith('123');
  });
});
```

### Integration Testing

```typescript
// src/pages/__tests__/Dashboard.integration.test.tsx
import { describe, it, expect, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { Dashboard } from '../Dashboard';
import { setupServer } from 'msw/node';
import { rest } from 'msw';

const server = setupServer(
  rest.get('/api/agents', (req, res, ctx) => {
    return res(ctx.json([
      { id: '1', name: 'Agent 1', status: 'active' },
      { id: '2', name: 'Agent 2', status: 'paused' },
    ]));
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

describe('Dashboard Integration', () => {
  it('fetches and displays agents', async () => {
    const queryClient = new QueryClient();

    render(
      <QueryClientProvider client={queryClient}>
        <Dashboard />
      </QueryClientProvider>
    );

    await waitFor(() => {
      expect(screen.getByText('Agent 1')).toBeInTheDocument();
      expect(screen.getByText('Agent 2')).toBeInTheDocument();
    });
  });
});
```

---

## Build Configuration

### Vite Configuration

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

export default defineConfig({
  plugins: [react()],

  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },

  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          // Vendor chunks
          'react-vendor': ['react', 'react-dom', 'react-router-dom'],
          'query-vendor': ['@tanstack/react-query'],
          'chart-vendor': ['recharts'],
          'form-vendor': ['react-hook-form', '@hookform/resolvers', 'zod'],
        },
      },
    },
    chunkSizeWarningLimit: 1000,
  },

  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8000',
        changeOrigin: true,
      },
      '/ws': {
        target: 'ws://localhost:8000',
        ws: true,
      },
    },
  },
});
```

---

## File Structure Summary

```
frontend/
├── public/
│   └── assets/
├── src/
│   ├── api/
│   │   ├── queries/
│   │   │   ├── agents.ts
│   │   │   ├── trades.ts
│   │   │   ├── portfolio.ts
│   │   │   └── oversight.ts
│   │   └── services/
│   │       ├── agentService.ts
│   │       ├── tradeService.ts
│   │       ├── portfolioService.ts
│   │       └── oversightService.ts
│   ├── components/
│   │   ├── Layout/
│   │   ├── charts/
│   │   ├── forms/
│   │   ├── ui/
│   │   └── ErrorBoundary.tsx
│   ├── features/
│   │   ├── agents/
│   │   ├── trades/
│   │   ├── portfolio/
│   │   └── oversight/
│   ├── hooks/
│   │   ├── useWebSocket.ts
│   │   ├── usePortfolio.ts
│   │   ├── useAnnouncement.ts
│   │   └── useKeyboardNav.ts
│   ├── pages/
│   │   ├── Dashboard/
│   │   ├── AgentCreate/
│   │   ├── AgentDetail/
│   │   ├── OversightQueue/
│   │   └── MeteringDashboard/
│   ├── providers/
│   │   ├── WebSocketProvider.tsx
│   │   └── ThemeProvider.tsx
│   ├── routes/
│   │   └── index.tsx
│   ├── schemas/
│   │   ├── agentSchema.ts
│   │   └── tradeSchema.ts
│   ├── stores/
│   │   ├── agentStore.ts
│   │   ├── tradeStore.ts
│   │   ├── uiStore.ts
│   │   └── userStore.ts
│   ├── styles/
│   │   ├── globals.css
│   │   └── tokens.css
│   ├── types/
│   │   ├── agent.types.ts
│   │   ├── trade.types.ts
│   │   └── api.types.ts
│   ├── utils/
│   │   ├── formatters.ts
│   │   ├── validators.ts
│   │   └── constants.ts
│   ├── App.tsx
│   └── main.tsx
├── package.json
├── tsconfig.json
├── vite.config.ts
└── vitest.config.ts
```

---

## Quality Attributes

### Performance Targets
- **Time to Interactive**: < 3 seconds
- **First Contentful Paint**: < 1.5 seconds
- **Bundle Size**: < 300KB (gzipped)
- **Chart Rendering**: < 100ms for 1000 data points

### Scalability
- Virtual scrolling for 10,000+ trades
- Lazy loading for code splitting
- Optimistic updates for responsiveness
- WebSocket for real-time sync

### Accessibility
- WCAG 2.1 AA compliance
- Keyboard navigation for all features
- Screen reader compatibility
- Focus management

### Maintainability
- TypeScript for type safety
- Component-driven architecture
- Comprehensive test coverage (>80%)
- Clear separation of concerns

---

## Deployment Strategy

### Production Build
```bash
npm run build
# Output: dist/
```

### Environment Configuration
```typescript
// src/config/env.ts
export const config = {
  apiUrl: import.meta.env.VITE_API_URL,
  wsUrl: import.meta.env.VITE_WS_URL,
  environment: import.meta.env.MODE,
};
```

### Docker Deployment
```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

---

## Next Steps

1. **Implement Design System** - Create reusable UI component library
2. **Setup Testing Infrastructure** - Configure Vitest and Testing Library
3. **Build Core Features** - Dashboard, Agent Management, Trades
4. **Integrate WebSocket** - Real-time portfolio updates
5. **Accessibility Audit** - WCAG 2.1 AA compliance verification
6. **Performance Testing** - Lighthouse, Web Vitals monitoring
7. **E2E Testing** - Playwright for critical user flows

---

## References

- [React 18 Documentation](https://react.dev)
- [TanStack Query](https://tanstack.com/query)
- [Zustand](https://github.com/pmndrs/zustand)
- [Recharts](https://recharts.org)
- [React Hook Form](https://react-hook-form.com)
- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
