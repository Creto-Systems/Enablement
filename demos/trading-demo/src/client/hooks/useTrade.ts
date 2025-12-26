import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useState, useCallback } from 'react';

export interface TradeData {
  symbol: string;
  side: 'buy' | 'sell';
  quantity: number;
  price: number;
  reasoning: string;
}

export interface Trade extends TradeData {
  id: string;
  agentId: string;
  timestamp: string;
  status: 'pending' | 'approved' | 'rejected' | 'executed' | 'cancelled';
  riskScore?: number;
}

export interface TradeMetrics {
  totalCost: number;
  estimatedFees: number;
  netCost: number;
}

export function useTrade(agentId: string) {
  const queryClient = useQueryClient();
  const [pendingTrades, setPendingTrades] = useState<Map<string, Trade>>(new Map());

  // Validate trade data
  const validateTrade = (data: TradeData) => {
    if (!data.symbol || data.symbol.length === 0) {
      throw new Error('Symbol is required');
    }
    if (data.quantity <= 0) {
      throw new Error('Quantity must be positive');
    }
    if (data.price <= 0) {
      throw new Error('Price must be positive');
    }
    if (!data.reasoning || data.reasoning.length < 10) {
      throw new Error('Reasoning must be at least 10 characters');
    }
  };

  // Submit trade mutation
  const mutation = useMutation({
    mutationFn: async (tradeData: TradeData): Promise<Trade> => {
      validateTrade(tradeData);

      const response = await fetch('/api/trades', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          agentId,
          ...tradeData,
        }),
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.error || `Failed to submit trade: ${response.statusText}`);
      }

      return response.json();
    },
    onMutate: async (tradeData) => {
      // Optimistic update
      const optimisticTrade: Trade = {
        id: `temp-${Date.now()}`,
        agentId,
        ...tradeData,
        timestamp: new Date().toISOString(),
        status: 'pending',
      };

      setPendingTrades((prev) => new Map(prev).set(optimisticTrade.id, optimisticTrade));
      return { optimisticTrade };
    },
    onSuccess: (trade, _, context) => {
      // Remove optimistic trade and add real one
      setPendingTrades((prev) => {
        const next = new Map(prev);
        if (context?.optimisticTrade) {
          next.delete(context.optimisticTrade.id);
        }
        next.set(trade.id, trade);
        return next;
      });

      // Invalidate portfolio to refetch updated data
      queryClient.invalidateQueries({
        queryKey: ['portfolio', agentId],
      });
    },
    onError: (_, __, context) => {
      // Rollback optimistic update
      if (context?.optimisticTrade) {
        setPendingTrades((prev) => {
          const next = new Map(prev);
          next.delete(context.optimisticTrade.id);
          return next;
        });
      }
    },
  });

  // Cancel trade
  const cancelTrade = useCallback(async (tradeId: string) => {
    const response = await fetch(`/api/trades/${tradeId}/cancel`, {
      method: 'POST',
    });

    if (response.ok) {
      setPendingTrades((prev) => {
        const next = new Map(prev);
        next.delete(tradeId);
        return next;
      });
    }
  }, []);

  // Update trade status
  const updateTradeStatus = useCallback(
    (update: { id: string; status: Trade['status'] }) => {
      setPendingTrades((prev) => {
        const trade = prev.get(update.id);
        if (!trade) return prev;

        const next = new Map(prev);
        next.set(update.id, { ...trade, status: update.status });
        return next;
      });
    },
    []
  );

  // Calculate trade metrics
  const calculateTradeMetrics = useCallback((tradeData: TradeData): TradeMetrics => {
    const totalCost = tradeData.quantity * tradeData.price;
    const estimatedFees = totalCost * 0.001; // 0.1% fee
    const netCost = totalCost + estimatedFees;

    return {
      totalCost,
      estimatedFees,
      netCost,
    };
  }, []);

  return {
    submitTrade: mutation.mutate,
    cancelTrade,
    updateTradeStatus,
    calculateTradeMetrics,
    pendingTrades,
    isPending: mutation.isPending,
    isSuccess: mutation.isSuccess,
    isError: mutation.isError,
    error: mutation.error,
  };
}
