/**
 * Trading Demo - React Query Hooks
 *
 * TanStack Query hooks for all API endpoints.
 * Provides optimistic updates, caching, and automatic refetching.
 */

import {
  useQuery,
  useMutation,
  useQueryClient,
  type UseQueryOptions,
  type UseMutationOptions,
} from '@tanstack/react-query';

import api, {
  type CreateAgentDTO,
  type ExecuteTradeDTO,
  type ApprovalDTO,
  ApiError,
} from '../api';

import type {
  Agent,
  Trade,
  Portfolio,
  OversightRequest,
  UsageMetrics,
  QuotaConfig,
} from '../../types/models';

// ============================================================================
// QUERY KEYS
// ============================================================================

export const queryKeys = {
  agents: {
    all: ['agents'] as const,
    list: (userId: string) => ['agents', 'list', userId] as const,
    detail: (id: string) => ['agents', 'detail', id] as const,
  },
  trades: {
    all: ['trades'] as const,
    list: (agentId: string) => ['trades', 'list', agentId] as const,
    detail: (agentId: string, tradeId: string) => ['trades', 'detail', agentId, tradeId] as const,
  },
  portfolio: {
    all: ['portfolio'] as const,
    detail: (agentId: string) => ['portfolio', 'detail', agentId] as const,
    history: (agentId: string, days: number) => ['portfolio', 'history', agentId, days] as const,
    positions: (agentId: string) => ['portfolio', 'positions', agentId] as const,
  },
  oversight: {
    all: ['oversight'] as const,
    list: () => ['oversight', 'list'] as const,
    detail: (requestId: string) => ['oversight', 'detail', requestId] as const,
    pending: () => ['oversight', 'pending'] as const,
  },
  metering: {
    all: ['metering'] as const,
    usage: (agentId: string) => ['metering', 'usage', agentId] as const,
    quota: (agentId: string) => ['metering', 'quota', agentId] as const,
  },
};

// ============================================================================
// AGENTS HOOKS
// ============================================================================

/**
 * List all agents for a user
 */
export function useAgents(
  userId: string,
  options?: Omit<UseQueryOptions<Agent[], ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery<Agent[], ApiError>({
    queryKey: queryKeys.agents.list(userId),
    queryFn: () => api.agents.list(userId),
    staleTime: 30000, // 30 seconds
    ...options,
  });
}

/**
 * Get single agent by ID
 */
export function useAgent(
  agentId: string,
  options?: Omit<UseQueryOptions<Agent, ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery<Agent, ApiError>({
    queryKey: queryKeys.agents.detail(agentId),
    queryFn: () => api.agents.get(agentId),
    staleTime: 30000,
    enabled: !!agentId,
    ...options,
  });
}

/**
 * Create new agent
 */
export function useCreateAgent(
  options?: UseMutationOptions<Agent, ApiError, CreateAgentDTO>
) {
  const queryClient = useQueryClient();

  return useMutation<Agent, ApiError, CreateAgentDTO>({
    mutationFn: (dto) => api.agents.create(dto),
    onSuccess: (newAgent, variables) => {
      // Invalidate agents list
      queryClient.invalidateQueries({ queryKey: queryKeys.agents.list(variables.userId) });

      // Set query data for new agent
      queryClient.setQueryData(queryKeys.agents.detail(newAgent.id), newAgent);
    },
    ...options,
  });
}

/**
 * Terminate agent
 */
export function useTerminateAgent(
  options?: UseMutationOptions<void, ApiError, { agentId: string; userId: string }>
) {
  const queryClient = useQueryClient();

  return useMutation<void, ApiError, { agentId: string; userId: string }>({
    mutationFn: ({ agentId }) => api.agents.terminate(agentId),
    onSuccess: (_, variables) => {
      // Invalidate agents list
      queryClient.invalidateQueries({ queryKey: queryKeys.agents.list(variables.userId) });

      // Remove agent from cache
      queryClient.removeQueries({ queryKey: queryKeys.agents.detail(variables.agentId) });
    },
    ...options,
  });
}

// ============================================================================
// TRADES HOOKS
// ============================================================================

/**
 * List trades for an agent
 */
export function useTrades(
  agentId: string,
  params?: { status?: string; limit?: number; offset?: number },
  options?: Omit<UseQueryOptions<Trade[], ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery<Trade[], ApiError>({
    queryKey: [...queryKeys.trades.list(agentId), params],
    queryFn: () => api.trades.list(agentId, params),
    staleTime: 10000, // 10 seconds
    enabled: !!agentId,
    ...options,
  });
}

/**
 * Get single trade
 */
export function useTrade(
  agentId: string,
  tradeId: string,
  options?: Omit<UseQueryOptions<Trade, ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery<Trade, ApiError>({
    queryKey: queryKeys.trades.detail(agentId, tradeId),
    queryFn: () => api.trades.get(agentId, tradeId),
    staleTime: 10000,
    enabled: !!agentId && !!tradeId,
    ...options,
  });
}

/**
 * Execute trade
 */
export function useExecuteTrade(
  agentId: string,
  options?: UseMutationOptions<Trade, ApiError, ExecuteTradeDTO>
) {
  const queryClient = useQueryClient();

  return useMutation<Trade, ApiError, ExecuteTradeDTO>({
    mutationFn: (dto) => api.trades.execute(agentId, dto),
    onSuccess: (newTrade) => {
      // Invalidate trades list
      queryClient.invalidateQueries({ queryKey: queryKeys.trades.list(agentId) });

      // Invalidate portfolio
      queryClient.invalidateQueries({ queryKey: queryKeys.portfolio.detail(agentId) });

      // Set query data for new trade
      queryClient.setQueryData(
        queryKeys.trades.detail(agentId, newTrade.id),
        newTrade
      );
    },
    ...options,
  });
}

/**
 * Cancel trade
 */
export function useCancelTrade(
  options?: UseMutationOptions<void, ApiError, { agentId: string; tradeId: string }>
) {
  const queryClient = useQueryClient();

  return useMutation<void, ApiError, { agentId: string; tradeId: string }>({
    mutationFn: ({ agentId, tradeId }) => api.trades.cancel(agentId, tradeId),
    onSuccess: (_, variables) => {
      // Invalidate trades list
      queryClient.invalidateQueries({ queryKey: queryKeys.trades.list(variables.agentId) });

      // Invalidate specific trade
      queryClient.invalidateQueries({
        queryKey: queryKeys.trades.detail(variables.agentId, variables.tradeId),
      });
    },
    ...options,
  });
}

// ============================================================================
// PORTFOLIO HOOKS
// ============================================================================

/**
 * Get portfolio for an agent
 */
export function usePortfolio(
  agentId: string,
  options?: Omit<UseQueryOptions<Portfolio, ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery<Portfolio, ApiError>({
    queryKey: queryKeys.portfolio.detail(agentId),
    queryFn: () => api.portfolio.get(agentId),
    staleTime: 5000, // 5 seconds
    refetchInterval: 30000, // Refetch every 30 seconds
    enabled: !!agentId,
    ...options,
  });
}

/**
 * Get portfolio history
 */
export function usePortfolioHistory(
  agentId: string,
  days: number = 30,
  options?: Omit<UseQueryOptions<any, ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: queryKeys.portfolio.history(agentId, days),
    queryFn: () => api.portfolio.getHistory(agentId, days),
    staleTime: 60000, // 1 minute
    enabled: !!agentId,
    ...options,
  });
}

/**
 * Get positions for an agent
 */
export function usePortfolioPositions(
  agentId: string,
  options?: Omit<UseQueryOptions<any, ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery({
    queryKey: queryKeys.portfolio.positions(agentId),
    queryFn: () => api.portfolio.getPositions(agentId),
    staleTime: 10000, // 10 seconds
    enabled: !!agentId,
    ...options,
  });
}

// ============================================================================
// OVERSIGHT HOOKS
// ============================================================================

/**
 * List oversight requests
 */
export function useOversightRequests(
  params?: { status?: string; priority?: string; limit?: number },
  options?: Omit<UseQueryOptions<OversightRequest[], ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery<OversightRequest[], ApiError>({
    queryKey: [...queryKeys.oversight.list(), params],
    queryFn: () => api.oversight.list(params),
    staleTime: 5000, // 5 seconds
    refetchInterval: 15000, // Refetch every 15 seconds
    ...options,
  });
}

/**
 * List pending approvals only
 */
export function usePendingApprovals(
  options?: Omit<UseQueryOptions<OversightRequest[], ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery<OversightRequest[], ApiError>({
    queryKey: queryKeys.oversight.pending(),
    queryFn: () => api.oversight.list({ status: 'pending' }),
    staleTime: 5000,
    refetchInterval: 10000, // Refetch every 10 seconds for real-time updates
    ...options,
  });
}

/**
 * Get single oversight request
 */
export function useOversightRequest(
  requestId: string,
  options?: Omit<UseQueryOptions<OversightRequest, ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery<OversightRequest, ApiError>({
    queryKey: queryKeys.oversight.detail(requestId),
    queryFn: () => api.oversight.get(requestId),
    staleTime: 5000,
    enabled: !!requestId,
    ...options,
  });
}

/**
 * Approve oversight request
 */
export function useApproveRequest(
  options?: UseMutationOptions<OversightRequest, ApiError, { requestId: string } & ApprovalDTO>
) {
  const queryClient = useQueryClient();

  return useMutation<OversightRequest, ApiError, { requestId: string } & ApprovalDTO>({
    mutationFn: ({ requestId, ...dto }) => api.oversight.approve(requestId, dto),
    onSuccess: (updatedRequest, variables) => {
      // Invalidate oversight lists
      queryClient.invalidateQueries({ queryKey: queryKeys.oversight.list() });
      queryClient.invalidateQueries({ queryKey: queryKeys.oversight.pending() });

      // Update specific request
      queryClient.setQueryData(
        queryKeys.oversight.detail(variables.requestId),
        updatedRequest
      );

      // Invalidate related trade
      if (updatedRequest.tradeId) {
        queryClient.invalidateQueries({
          queryKey: queryKeys.trades.detail(updatedRequest.agentId, updatedRequest.tradeId),
        });
      }
    },
    ...options,
  });
}

/**
 * Reject oversight request
 */
export function useRejectRequest(
  options?: UseMutationOptions<OversightRequest, ApiError, { requestId: string } & ApprovalDTO>
) {
  const queryClient = useQueryClient();

  return useMutation<OversightRequest, ApiError, { requestId: string } & ApprovalDTO>({
    mutationFn: ({ requestId, ...dto }) => api.oversight.reject(requestId, dto),
    onSuccess: (updatedRequest, variables) => {
      // Invalidate oversight lists
      queryClient.invalidateQueries({ queryKey: queryKeys.oversight.list() });
      queryClient.invalidateQueries({ queryKey: queryKeys.oversight.pending() });

      // Update specific request
      queryClient.setQueryData(
        queryKeys.oversight.detail(variables.requestId),
        updatedRequest
      );

      // Invalidate related trade
      if (updatedRequest.tradeId) {
        queryClient.invalidateQueries({
          queryKey: queryKeys.trades.detail(updatedRequest.agentId, updatedRequest.tradeId),
        });
      }
    },
    ...options,
  });
}

// ============================================================================
// METERING HOOKS
// ============================================================================

/**
 * Get usage metrics for an agent
 */
export function useUsageMetrics(
  agentId: string,
  options?: Omit<UseQueryOptions<UsageMetrics, ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery<UsageMetrics, ApiError>({
    queryKey: queryKeys.metering.usage(agentId),
    queryFn: () => api.metering.getUsage(agentId),
    staleTime: 30000, // 30 seconds
    refetchInterval: 60000, // Refetch every minute
    enabled: !!agentId,
    ...options,
  });
}

/**
 * Get quota configuration for an agent
 */
export function useQuotaCheck(
  agentId: string,
  options?: Omit<UseQueryOptions<QuotaConfig, ApiError>, 'queryKey' | 'queryFn'>
) {
  return useQuery<QuotaConfig, ApiError>({
    queryKey: queryKeys.metering.quota(agentId),
    queryFn: () => api.metering.checkQuota(agentId),
    staleTime: 30000,
    refetchInterval: 60000,
    enabled: !!agentId,
    ...options,
  });
}

// All hooks are exported inline via their function declarations above
