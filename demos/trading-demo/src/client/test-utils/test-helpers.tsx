import React from 'react';
import { render, RenderOptions } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

// Create a custom render function that includes providers
export function renderWithProviders(
  ui: React.ReactElement,
  options?: Omit<RenderOptions, 'wrapper'>
) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
      mutations: {
        retry: false,
      },
    },
  });

  function Wrapper({ children }: { children: React.ReactNode }) {
    return (
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    );
  }

  return { ...render(ui, { wrapper: Wrapper, ...options }), queryClient };
}

// Mock agent data
export const mockAgent = {
  id: 'agent-1',
  name: 'Conservative Trader',
  status: 'active' as const,
  budget: {
    total: 10000,
    used: 3500,
    remaining: 6500,
  },
  performance: {
    dailyPnL: 234.56,
    totalPnL: 1250.00,
    winRate: 0.68,
  },
};

// Mock portfolio data
export const mockPortfolio = {
  agentId: 'agent-1',
  holdings: [
    { symbol: 'AAPL', quantity: 10, avgPrice: 150.00, currentPrice: 155.00 },
    { symbol: 'GOOGL', quantity: 5, avgPrice: 2800.00, currentPrice: 2850.00 },
  ],
  cashBalance: 5000.00,
  totalValue: 12050.00,
  history: [
    { timestamp: '2024-01-01T00:00:00Z', value: 10000 },
    { timestamp: '2024-01-02T00:00:00Z', value: 10500 },
    { timestamp: '2024-01-03T00:00:00Z', value: 12050 },
  ],
};

// Mock trade data
export const mockTrade = {
  id: 'trade-1',
  agentId: 'agent-1',
  symbol: 'AAPL',
  side: 'buy' as const,
  quantity: 10,
  price: 155.00,
  timestamp: '2024-01-03T10:00:00Z',
  status: 'pending' as const,
  riskScore: 0.3,
  reasoning: 'Strong technical indicators and positive momentum',
};

// Mock pending approval
export const mockApproval = {
  ...mockTrade,
  estimatedCost: 1550.00,
  riskAssessment: {
    score: 0.3,
    level: 'low' as const,
    factors: [
      'Position size within limits',
      'Strong technical indicators',
      'Diversification maintained',
    ],
  },
};

// Helper to create mock WebSocket events
export function createWSEvent(type: string, data: any): MessageEvent {
  return new MessageEvent('message', {
    data: JSON.stringify({ type, data }),
  });
}

// Helper to wait for async updates
export function waitForAsync() {
  return new Promise(resolve => setTimeout(resolve, 0));
}
