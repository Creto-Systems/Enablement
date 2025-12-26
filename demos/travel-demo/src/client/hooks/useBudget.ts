import { useQuery } from '@tanstack/react-query';
import type { BudgetAnalysis } from '../../shared/types';

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000';

export const useBudget = (tripId?: string) => {
  const { data: budgetAnalysis, isLoading } = useQuery<BudgetAnalysis>({
    queryKey: ['budget', tripId],
    queryFn: async () => {
      if (!tripId) throw new Error('No trip ID');
      const response = await fetch(`${API_BASE}/api/trips/${tripId}/budget`);
      if (!response.ok) throw new Error('Failed to fetch budget analysis');
      return response.json();
    },
    enabled: !!tripId,
    refetchInterval: 5000, // Poll every 5 seconds
  });

  return {
    budgetAnalysis,
    loading: isLoading,
  };
};
