/**
 * Trading Demo API Client
 *
 * Centralized API client for all backend endpoints.
 * Provides type-safe wrappers for agents, trades, portfolio, oversight, and metering.
 */

import type {
  Agent,
  Trade,
  Portfolio,
  Position,
  OversightRequest,
  UsageMetrics,
  QuotaConfig,
  TradeSide,
  OrderType,
  TradingStrategy,
} from '../../types/models';

// ============================================================================
// CONFIGURATION
// ============================================================================

const API_BASE = '/api/v1';

/**
 * API Error
 */
export class ApiError extends Error {
  constructor(
    public statusCode: number,
    public message: string,
    public errors?: Array<{ field: string; message: string }>
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

/**
 * Generic fetch wrapper with error handling
 */
async function apiFetch<T>(
  endpoint: string,
  options: RequestInit = {}
): Promise<T> {
  const url = `${API_BASE}${endpoint}`;

  const response = await fetch(url, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options.headers,
    },
  });

  if (!response.ok) {
    const errorData = await response.json().catch(() => ({
      message: response.statusText,
    }));

    throw new ApiError(
      response.status,
      errorData.message || `Request failed with status ${response.status}`,
      errorData.errors
    );
  }

  // Handle 204 No Content
  if (response.status === 204) {
    return undefined as T;
  }

  return response.json();
}

// ============================================================================
// AGENTS API
// ============================================================================

export interface CreateAgentDTO {
  name: string;
  type: TradingStrategy;
  userId: string;
  config: {
    strategy: TradingStrategy;
    riskTolerance: number;
    maxPositionSize: number;
    dailyTradeLimit: number;
    oversightThreshold: number;
    allowedSymbols?: string[];
    autoApprovalEnabled: boolean;
    autoApprovalLimit: number;
  };
}

export const agentsApi = {
  /**
   * List all agents for a user
   */
  list: async (userId: string): Promise<Agent[]> => {
    return apiFetch<Agent[]>(`/agents?userId=${encodeURIComponent(userId)}`);
  },

  /**
   * Get agent by ID
   */
  get: async (agentId: string): Promise<Agent> => {
    return apiFetch<Agent>(`/agents/${agentId}`);
  },

  /**
   * Create new agent
   */
  create: async (dto: CreateAgentDTO): Promise<Agent> => {
    return apiFetch<Agent>('/agents', {
      method: 'POST',
      body: JSON.stringify(dto),
    });
  },

  /**
   * Terminate agent
   */
  terminate: async (agentId: string): Promise<void> => {
    return apiFetch<void>(`/agents/${agentId}`, {
      method: 'DELETE',
    });
  },
};

// ============================================================================
// TRADES API
// ============================================================================

export interface ExecuteTradeDTO {
  symbol: string;
  side: TradeSide;
  type: OrderType;
  quantity: number;
  price?: number;
  limitPrice?: number;
  stopPrice?: number;
  metadata: {
    strategy: TradingStrategy;
    confidence: number;
    rationale: string;
    profitTarget?: number;
    stopLoss?: number;
    timeInForce: 'day' | 'gtc' | 'ioc' | 'fok';
  };
}

export const tradesApi = {
  /**
   * List trades for an agent
   */
  list: async (agentId: string, params?: {
    status?: string;
    limit?: number;
    offset?: number;
  }): Promise<Trade[]> => {
    const queryParams = new URLSearchParams();
    if (params?.status) queryParams.append('status', params.status);
    if (params?.limit) queryParams.append('limit', params.limit.toString());
    if (params?.offset) queryParams.append('offset', params.offset.toString());

    const query = queryParams.toString();
    const endpoint = `/agents/${agentId}/trades${query ? `?${query}` : ''}`;

    return apiFetch<Trade[]>(endpoint);
  },

  /**
   * Get trade by ID
   */
  get: async (agentId: string, tradeId: string): Promise<Trade> => {
    return apiFetch<Trade>(`/agents/${agentId}/trades/${tradeId}`);
  },

  /**
   * Execute new trade
   */
  execute: async (agentId: string, dto: ExecuteTradeDTO): Promise<Trade> => {
    return apiFetch<Trade>(`/agents/${agentId}/trades`, {
      method: 'POST',
      body: JSON.stringify(dto),
    });
  },

  /**
   * Cancel trade
   */
  cancel: async (agentId: string, tradeId: string): Promise<void> => {
    return apiFetch<void>(`/agents/${agentId}/trades/${tradeId}`, {
      method: 'DELETE',
    });
  },
};

// ============================================================================
// PORTFOLIO API
// ============================================================================

export interface PortfolioHistoryResponse {
  agentId: string;
  days: number;
  history: Array<{
    timestamp: string;
    value: number;
  }>;
}

export interface PositionsResponse {
  agentId: string;
  positions: Position[];
  totalPositions: number;
}

export const portfolioApi = {
  /**
   * Get portfolio for an agent
   */
  get: async (agentId: string): Promise<Portfolio> => {
    return apiFetch<Portfolio>(`/portfolio/${agentId}`);
  },

  /**
   * Get portfolio history
   */
  getHistory: async (agentId: string, days: number = 30): Promise<PortfolioHistoryResponse> => {
    return apiFetch<PortfolioHistoryResponse>(
      `/portfolio/${agentId}/history?days=${days}`
    );
  },

  /**
   * Get positions for an agent
   */
  getPositions: async (agentId: string): Promise<PositionsResponse> => {
    return apiFetch<PositionsResponse>(`/portfolio/${agentId}/positions`);
  },
};

// ============================================================================
// OVERSIGHT API
// ============================================================================

export interface ApprovalDTO {
  approverId: string;
  approverName: string;
  decision: 'approve' | 'reject';
  reason: string;
}

export const oversightApi = {
  /**
   * List pending approval requests
   */
  list: async (params?: {
    status?: string;
    priority?: string;
    limit?: number;
  }): Promise<OversightRequest[]> => {
    const queryParams = new URLSearchParams();
    if (params?.status) queryParams.append('status', params.status);
    if (params?.priority) queryParams.append('priority', params.priority);
    if (params?.limit) queryParams.append('limit', params.limit.toString());

    const query = queryParams.toString();
    const endpoint = `/oversight/requests${query ? `?${query}` : ''}`;

    return apiFetch<OversightRequest[]>(endpoint);
  },

  /**
   * Get oversight request by ID
   */
  get: async (requestId: string): Promise<OversightRequest> => {
    return apiFetch<OversightRequest>(`/oversight/requests/${requestId}`);
  },

  /**
   * Approve request
   */
  approve: async (requestId: string, dto: ApprovalDTO): Promise<OversightRequest> => {
    return apiFetch<OversightRequest>(`/oversight/requests/${requestId}/approve`, {
      method: 'POST',
      body: JSON.stringify(dto),
    });
  },

  /**
   * Reject request
   */
  reject: async (requestId: string, dto: ApprovalDTO): Promise<OversightRequest> => {
    return apiFetch<OversightRequest>(`/oversight/requests/${requestId}/reject`, {
      method: 'POST',
      body: JSON.stringify(dto),
    });
  },
};

// ============================================================================
// METERING API
// ============================================================================

export const meteringApi = {
  /**
   * Get usage metrics for an agent
   */
  getUsage: async (agentId: string): Promise<UsageMetrics> => {
    return apiFetch<UsageMetrics>(`/metering/agents/${agentId}/usage`);
  },

  /**
   * Check quota for an agent
   */
  checkQuota: async (agentId: string): Promise<QuotaConfig> => {
    return apiFetch<QuotaConfig>(`/metering/agents/${agentId}/quota`);
  },
};

// ============================================================================
// EXPORT ALL
// ============================================================================

export default {
  agents: agentsApi,
  trades: tradesApi,
  portfolio: portfolioApi,
  oversight: oversightApi,
  metering: meteringApi,
};
