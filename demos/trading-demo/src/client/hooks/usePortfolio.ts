import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useEffect, useState, useMemo } from 'react';

export interface Holding {
  symbol: string;
  quantity: number;
  avgPrice: number;
  currentPrice: number;
}

export interface PortfolioData {
  agentId: string;
  holdings: Holding[];
  cashBalance: number;
  totalValue: number;
  history: Array<{
    timestamp: string;
    value: number;
  }>;
}

export function usePortfolio(agentId: string) {
  const queryClient = useQueryClient();
  const [isConnected, setIsConnected] = useState(false);

  // Fetch portfolio data
  const query = useQuery({
    queryKey: ['portfolio', agentId],
    queryFn: async (): Promise<PortfolioData> => {
      const response = await fetch(`/api/portfolio/${agentId}`);
      if (!response.ok) {
        throw new Error(`Failed to fetch portfolio: ${response.statusText}`);
      }
      return response.json();
    },
    staleTime: 1000 * 60, // 1 minute
  });

  // WebSocket connection for real-time updates
  useEffect(() => {
    const ws = new WebSocket(`ws://localhost/portfolio`);

    ws.onopen = () => {
      setIsConnected(true);
      ws.send(JSON.stringify({ type: 'subscribe', agentId }));
    };

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        if (message.type === 'portfolio-update' && message.data) {
          queryClient.setQueryData(['portfolio', agentId], message.data);
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      setIsConnected(false);
    };

    ws.onclose = () => {
      setIsConnected(false);
    };

    return () => {
      ws.close();
    };
  }, [agentId, queryClient]);

  // Calculate derived values
  const unrealizedPnL = useMemo(() => {
    if (!query.data?.holdings) return 0;
    return query.data.holdings.reduce((total, holding) => {
      return total + (holding.currentPrice - holding.avgPrice) * holding.quantity;
    }, 0);
  }, [query.data]);

  const totalPnL = useMemo(() => {
    if (!query.data?.history || query.data.history.length < 2) return 0;
    const firstValue = query.data.history[0].value;
    const lastValue = query.data.history[query.data.history.length - 1].value;
    return lastValue - firstValue;
  }, [query.data]);

  const portfolioAllocation = useMemo(() => {
    if (!query.data?.holdings || !query.data.totalValue) return [];
    return query.data.holdings.map((holding) => ({
      symbol: holding.symbol,
      value: holding.quantity * holding.currentPrice,
      percentage: (holding.quantity * holding.currentPrice) / query.data.totalValue,
    }));
  }, [query.data]);

  return {
    ...query,
    unrealizedPnL,
    totalPnL,
    portfolioAllocation,
    isConnected,
  };
}
