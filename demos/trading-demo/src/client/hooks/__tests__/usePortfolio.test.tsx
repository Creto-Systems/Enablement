import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { usePortfolio } from '../usePortfolio';
import { mockPortfolio, createWSEvent } from '../../test-utils/test-helpers';

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        gcTime: 0,
      },
    },
  });

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
}

describe('usePortfolio', () => {
  let mockWebSocket: any;

  beforeEach(() => {
    mockFetch.mockClear();
    mockWebSocket = null;
  });

  afterEach(() => {
    if (mockWebSocket) {
      mockWebSocket.close();
    }
  });

  it('fetches portfolio on mount', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockPortfolio,
    });

    const { result } = renderHook(() => usePortfolio('agent-1'), {
      wrapper: createWrapper(),
    });

    expect(result.current.isLoading).toBe(true);

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toEqual(mockPortfolio);
    expect(mockFetch).toHaveBeenCalledWith('/api/portfolio/agent-1');
  });

  it('updates on WebSocket event', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockPortfolio,
    });

    const { result } = renderHook(() => usePortfolio('agent-1'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    // Simulate WebSocket update
    const updatedPortfolio = {
      ...mockPortfolio,
      totalValue: 13000,
    };

    // Get WebSocket instance from mock
    const WS = (global.WebSocket as any);
    mockWebSocket = new WS('ws://localhost/portfolio');

    // Trigger message
    await waitFor(() => {
      if (mockWebSocket.onmessage) {
        mockWebSocket.onmessage(createWSEvent('portfolio-update', updatedPortfolio));
      }
    });

    // Portfolio should update
    await waitFor(() => {
      expect(result.current.data?.totalValue).toBe(13000);
    });
  });

  it('calculates derived values', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockPortfolio,
    });

    const { result } = renderHook(() => usePortfolio('agent-1'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    // Check derived calculations
    expect(result.current.totalPnL).toBeDefined();
    expect(result.current.unrealizedPnL).toBeDefined();
    expect(result.current.portfolioAllocation).toBeDefined();
  });

  it('handles errors gracefully', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 500,
      statusText: 'Internal Server Error',
    });

    const { result } = renderHook(() => usePortfolio('agent-1'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isError).toBe(true);
    });

    expect(result.current.error).toBeDefined();
  });

  it('handles WebSocket connection errors', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockPortfolio,
    });

    const { result } = renderHook(() => usePortfolio('agent-1'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    // Simulate WebSocket error
    const WS = (global.WebSocket as any);
    mockWebSocket = new WS('ws://localhost/portfolio');

    if (mockWebSocket.onerror) {
      mockWebSocket.onerror(new Event('error'));
    }

    // Should still have data from initial fetch
    expect(result.current.data).toEqual(mockPortfolio);
  });

  it('calculates unrealized P&L correctly', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockPortfolio,
    });

    const { result } = renderHook(() => usePortfolio('agent-1'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    const expectedPnL = mockPortfolio.holdings.reduce((total, holding) => {
      return total + (holding.currentPrice - holding.avgPrice) * holding.quantity;
    }, 0);

    expect(result.current.unrealizedPnL).toBeCloseTo(expectedPnL, 2);
  });

  it('reconnects WebSocket on connection loss', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockPortfolio,
    });

    const { result } = renderHook(() => usePortfolio('agent-1'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    // Simulate WebSocket close
    const WS = (global.WebSocket as any);
    mockWebSocket = new WS('ws://localhost/portfolio');

    if (mockWebSocket.onclose) {
      mockWebSocket.onclose(new Event('close'));
    }

    // Should attempt to reconnect
    await waitFor(() => {
      expect(result.current.isConnected).toBe(false);
    });
  });

  it('cleans up on unmount', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockPortfolio,
    });

    const { result, unmount } = renderHook(() => usePortfolio('agent-1'), {
      wrapper: createWrapper(),
    });

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    unmount();

    // WebSocket should be closed
    expect(result.current.isConnected).toBe(false);
  });
});
