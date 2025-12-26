import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { useTrade } from '../useTrade';
import { mockTrade } from '../../test-utils/test-helpers';

const mockFetch = vi.fn();
global.fetch = mockFetch;

function createWrapper() {
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

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

describe('useTrade', () => {
  beforeEach(() => {
    mockFetch.mockClear();
  });

  it('submits trade optimistically', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ ...mockTrade, status: 'pending' }),
    });

    const { result } = renderHook(() => useTrade('agent-1'), {
      wrapper: createWrapper(),
    });

    const tradeData = {
      symbol: 'AAPL',
      side: 'buy' as const,
      quantity: 10,
      price: 150.00,
      reasoning: 'Good opportunity',
    };

    result.current.submitTrade(tradeData);

    // Should immediately update UI (optimistic)
    expect(result.current.isPending).toBe(true);

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(mockFetch).toHaveBeenCalledWith('/api/trades', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        agentId: 'agent-1',
        ...tradeData,
      }),
    });
  });

  it('rolls back on failure', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 400,
      statusText: 'Bad Request',
      json: async () => ({ error: 'Invalid trade' }),
    });

    const { result } = renderHook(() => useTrade('agent-1'), {
      wrapper: createWrapper(),
    });

    const tradeData = {
      symbol: 'AAPL',
      side: 'buy' as const,
      quantity: 10,
      price: 150.00,
      reasoning: 'Good opportunity',
    };

    result.current.submitTrade(tradeData);

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(result.current.error).toBeDefined();
  });

  it('invalidates portfolio query', async () => {
    const queryClient = new QueryClient({
      defaultOptions: {
        queries: { retry: false, gcTime: 0 },
        mutations: { retry: false },
      },
    });

    const invalidateSpy = vi.spyOn(queryClient, 'invalidateQueries');

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockTrade,
    });

    const wrapper = ({ children }: { children: React.ReactNode }) => (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );

    const { result } = renderHook(() => useTrade('agent-1'), { wrapper });

    const tradeData = {
      symbol: 'AAPL',
      side: 'buy' as const,
      quantity: 10,
      price: 150.00,
      reasoning: 'Good opportunity',
    };

    result.current.submitTrade(tradeData);

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(invalidateSpy).toHaveBeenCalledWith({
      queryKey: ['portfolio', 'agent-1'],
    });
  });

  it('handles network errors', async () => {
    mockFetch.mockRejectedValueOnce(new Error('Network error'));

    const { result } = renderHook(() => useTrade('agent-1'), {
      wrapper: createWrapper(),
    });

    const tradeData = {
      symbol: 'AAPL',
      side: 'buy' as const,
      quantity: 10,
      price: 150.00,
      reasoning: 'Good opportunity',
    };

    result.current.submitTrade(tradeData);

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(result.current.error?.message).toContain('Network error');
  });

  it('validates trade data before submission', async () => {
    const { result } = renderHook(() => useTrade('agent-1'), {
      wrapper: createWrapper(),
    });

    const invalidTradeData = {
      symbol: '',
      side: 'buy' as const,
      quantity: -10,
      price: 0,
      reasoning: '',
    };

    expect(() => {
      result.current.submitTrade(invalidTradeData);
    }).toThrow();

    expect(mockFetch).not.toHaveBeenCalled();
  });

  it('supports cancelling pending trades', async () => {
    mockFetch
      .mockResolvedValueOnce({
        ok: true,
        json: async () => mockTrade,
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ ...mockTrade, status: 'cancelled' }),
      });

    const { result } = renderHook(() => useTrade('agent-1'), {
      wrapper: createWrapper(),
    });

    const tradeData = {
      symbol: 'AAPL',
      side: 'buy' as const,
      quantity: 10,
      price: 150.00,
      reasoning: 'Good opportunity',
    };

    result.current.submitTrade(tradeData);

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    // Cancel the trade
    result.current.cancelTrade(mockTrade.id);

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(`/api/trades/${mockTrade.id}/cancel`, {
        method: 'POST',
      });
    });
  });

  it('updates trade status', async () => {
    const { result } = renderHook(() => useTrade('agent-1'), {
      wrapper: createWrapper(),
    });

    const statusUpdate = {
      id: mockTrade.id,
      status: 'executed' as const,
    };

    result.current.updateTradeStatus(statusUpdate);

    // Should update local cache
    expect(result.current.pendingTrades.get(mockTrade.id)?.status).toBe('executed');
  });

  it('provides pending trades list', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => [mockTrade],
    });

    const { result } = renderHook(() => useTrade('agent-1'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.pendingTrades.size).toBeGreaterThan(0);
    });

    expect(result.current.pendingTrades.get(mockTrade.id)).toEqual(mockTrade);
  });

  it('calculates trade metrics', async () => {
    const { result } = renderHook(() => useTrade('agent-1'), {
      wrapper: createWrapper(),
    });

    const tradeData = {
      symbol: 'AAPL',
      side: 'buy' as const,
      quantity: 10,
      price: 150.00,
      reasoning: 'Good opportunity',
    };

    const metrics = result.current.calculateTradeMetrics(tradeData);

    expect(metrics.totalCost).toBe(1500.00);
    expect(metrics.estimatedFees).toBeDefined();
    expect(metrics.netCost).toBeDefined();
  });
});
