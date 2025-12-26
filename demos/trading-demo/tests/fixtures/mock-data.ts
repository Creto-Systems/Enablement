/**
 * Mock Data Factory for Trading Demo
 *
 * Provides realistic test data for investor demonstrations
 * Showcases all 4 Enablement products:
 * - Metering: Usage quotas and billing
 * - Oversight: Approval workflows
 * - Runtime: Agent execution
 * - Messaging: Event notifications
 */

import {
  Agent,
  AgentStatus,
  TradingStrategy,
  Portfolio,
  Position,
  Trade,
  TradeSide,
  OrderType,
  TradeStatus,
  OversightRequest,
  OversightStatus,
  OversightPriority,
  UsageMetrics,
  BillableEvent,
  BillableEventType,
  QuotaConfig,
  ApprovalDecision,
  RiskAssessment,
  PerformanceMetrics,
} from '../../src/types/models';

// ============================================================================
// DEMO AGENT CONFIGURATIONS
// ============================================================================

export const DEMO_AGENTS = {
  aggressive: {
    id: 'agent-alpha-001',
    name: 'Alpha Trader',
    status: AgentStatus.ACTIVE,
    budget: 100000,
    availableBudget: 73500,
    createdAt: new Date('2024-12-01T09:00:00Z'),
    lastActiveAt: new Date('2024-12-26T14:30:00Z'),
    portfolioId: 'portfolio-alpha-001',
    config: {
      strategy: TradingStrategy.MOMENTUM,
      riskTolerance: 8,
      maxPositionSize: 0.3,
      dailyTradeLimit: 50,
      oversightThreshold: 50000,
      allowedSymbols: ['AAPL', 'GOOGL', 'MSFT', 'AMZN', 'TSLA', 'NVDA', 'META'],
      autoApprovalEnabled: true,
      autoApprovalLimit: 10000,
    },
  },
  conservative: {
    id: 'agent-safehaven-002',
    name: 'Safe Haven',
    status: AgentStatus.ACTIVE,
    budget: 50000,
    availableBudget: 41200,
    createdAt: new Date('2024-12-01T09:00:00Z'),
    lastActiveAt: new Date('2024-12-26T14:15:00Z'),
    portfolioId: 'portfolio-safehaven-002',
    config: {
      strategy: TradingStrategy.MEAN_REVERSION,
      riskTolerance: 3,
      maxPositionSize: 0.15,
      dailyTradeLimit: 20,
      oversightThreshold: 25000,
      allowedSymbols: ['AAPL', 'MSFT', 'JNJ', 'PG', 'KO'],
      autoApprovalEnabled: true,
      autoApprovalLimit: 5000,
    },
  },
  balanced: {
    id: 'agent-balanced-003',
    name: 'Balanced Growth',
    status: AgentStatus.ACTIVE,
    budget: 75000,
    availableBudget: 58300,
    createdAt: new Date('2024-12-01T09:00:00Z'),
    lastActiveAt: new Date('2024-12-26T14:45:00Z'),
    portfolioId: 'portfolio-balanced-003',
    config: {
      strategy: TradingStrategy.MARKET_MAKING,
      riskTolerance: 5,
      maxPositionSize: 0.2,
      dailyTradeLimit: 30,
      oversightThreshold: 35000,
      allowedSymbols: ['AAPL', 'GOOGL', 'MSFT', 'AMZN', 'NVDA'],
      autoApprovalEnabled: true,
      autoApprovalLimit: 7500,
    },
  },
} as const;

// ============================================================================
// PORTFOLIO POSITIONS
// ============================================================================

export const DEMO_POSITIONS: Record<string, Position[]> = {
  'portfolio-alpha-001': [
    {
      id: 'pos-alpha-001',
      portfolioId: 'portfolio-alpha-001',
      symbol: 'NVDA',
      quantity: 50,
      avgPrice: 495.30,
      currentPrice: 505.50,
      marketValue: 25275,
      pnl: 510,
      pnlPercent: 2.06,
      openedAt: new Date('2024-12-15T10:30:00Z'),
      updatedAt: new Date('2024-12-26T14:30:00Z'),
    },
    {
      id: 'pos-alpha-002',
      portfolioId: 'portfolio-alpha-001',
      symbol: 'TSLA',
      quantity: 30,
      avgPrice: 245.80,
      currentPrice: 252.30,
      marketValue: 7569,
      pnl: 195,
      pnlPercent: 2.65,
      openedAt: new Date('2024-12-18T14:20:00Z'),
      updatedAt: new Date('2024-12-26T14:30:00Z'),
    },
  ],
  'portfolio-safehaven-002': [
    {
      id: 'pos-safe-001',
      portfolioId: 'portfolio-safehaven-002',
      symbol: 'AAPL',
      quantity: 25,
      avgPrice: 195.20,
      currentPrice: 197.50,
      marketValue: 4937.50,
      pnl: 57.50,
      pnlPercent: 1.18,
      openedAt: new Date('2024-12-10T11:00:00Z'),
      updatedAt: new Date('2024-12-26T14:15:00Z'),
    },
    {
      id: 'pos-safe-002',
      portfolioId: 'portfolio-safehaven-002',
      symbol: 'MSFT',
      quantity: 15,
      avgPrice: 375.00,
      currentPrice: 380.25,
      marketValue: 5703.75,
      pnl: 78.75,
      pnlPercent: 1.40,
      openedAt: new Date('2024-12-12T09:30:00Z'),
      updatedAt: new Date('2024-12-26T14:15:00Z'),
    },
  ],
  'portfolio-balanced-003': [
    {
      id: 'pos-bal-001',
      portfolioId: 'portfolio-balanced-003',
      symbol: 'GOOGL',
      quantity: 40,
      avgPrice: 142.50,
      currentPrice: 145.30,
      marketValue: 5812,
      pnl: 112,
      pnlPercent: 1.96,
      openedAt: new Date('2024-12-14T13:00:00Z'),
      updatedAt: new Date('2024-12-26T14:45:00Z'),
    },
    {
      id: 'pos-bal-002',
      portfolioId: 'portfolio-balanced-003',
      symbol: 'AMZN',
      quantity: 20,
      avgPrice: 178.90,
      currentPrice: 182.40,
      marketValue: 3648,
      pnl: 70,
      pnlPercent: 1.96,
      openedAt: new Date('2024-12-16T10:15:00Z'),
      updatedAt: new Date('2024-12-26T14:45:00Z'),
    },
  ],
};

// ============================================================================
// TRADE HISTORY (Last 30 Days)
// ============================================================================

export const DEMO_TRADES: Trade[] = [
  // Alpha Trader - Recent large trade requiring oversight
  {
    id: 'trade-001',
    agentId: 'agent-alpha-001',
    symbol: 'NVDA',
    side: TradeSide.BUY,
    type: OrderType.MARKET,
    quantity: 100,
    price: null,
    totalValue: 50550,
    status: TradeStatus.PENDING_APPROVAL,
    filledQuantity: 0,
    avgFillPrice: null,
    timestamp: new Date('2024-12-26T14:00:00Z'),
    filledAt: null,
    oversightRequestId: 'oversight-001',
    rejectionReason: null,
    metadata: {
      strategy: TradingStrategy.MOMENTUM,
      confidence: 0.85,
      rationale: 'Strong bullish momentum with volume surge. Breaking resistance at $500.',
      profitTarget: 520,
      stopLoss: 490,
      timeInForce: 'day',
    },
  },
  // Alpha Trader - Executed trade
  {
    id: 'trade-002',
    agentId: 'agent-alpha-001',
    symbol: 'TSLA',
    side: TradeSide.BUY,
    type: OrderType.LIMIT,
    quantity: 30,
    price: 245.80,
    limitPrice: 246.00,
    totalValue: 7374,
    status: TradeStatus.FILLED,
    filledQuantity: 30,
    avgFillPrice: 245.80,
    timestamp: new Date('2024-12-18T14:20:00Z'),
    filledAt: new Date('2024-12-18T14:25:00Z'),
    oversightRequestId: null,
    rejectionReason: null,
    metadata: {
      strategy: TradingStrategy.MOMENTUM,
      confidence: 0.78,
      rationale: 'Technical breakout pattern confirmed. Entry at support level.',
      profitTarget: 260,
      stopLoss: 238,
      timeInForce: 'day',
    },
  },
  // Safe Haven - Conservative trade
  {
    id: 'trade-003',
    agentId: 'agent-safehaven-002',
    symbol: 'AAPL',
    side: TradeSide.BUY,
    type: OrderType.LIMIT,
    quantity: 25,
    price: 195.20,
    limitPrice: 196.00,
    totalValue: 4880,
    status: TradeStatus.FILLED,
    filledQuantity: 25,
    avgFillPrice: 195.20,
    timestamp: new Date('2024-12-10T11:00:00Z'),
    filledAt: new Date('2024-12-10T11:05:00Z'),
    oversightRequestId: null,
    rejectionReason: null,
    metadata: {
      strategy: TradingStrategy.MEAN_REVERSION,
      confidence: 0.72,
      rationale: 'Mean reversion to moving average. Oversold on RSI.',
      profitTarget: 200,
      stopLoss: 192,
      timeInForce: 'gtc',
    },
  },
  // Balanced Growth - Rejected trade (exceeded risk limits)
  {
    id: 'trade-004',
    agentId: 'agent-balanced-003',
    symbol: 'META',
    side: TradeSide.BUY,
    type: OrderType.MARKET,
    quantity: 100,
    price: null,
    totalValue: 35500,
    status: TradeStatus.REJECTED,
    filledQuantity: 0,
    avgFillPrice: null,
    timestamp: new Date('2024-12-20T10:30:00Z'),
    filledAt: null,
    oversightRequestId: 'oversight-002',
    rejectionReason: 'Exceeds maximum position size limit (20% of portfolio)',
    metadata: {
      strategy: TradingStrategy.MARKET_MAKING,
      confidence: 0.65,
      rationale: 'Market making opportunity with tight bid-ask spread.',
      profitTarget: 365,
      stopLoss: 345,
      timeInForce: 'day',
    },
  },
];

// ============================================================================
// OVERSIGHT REQUESTS
// ============================================================================

export const DEMO_OVERSIGHT_REQUESTS: OversightRequest[] = [
  {
    id: 'oversight-001',
    agentId: 'agent-alpha-001',
    tradeId: 'trade-001',
    tradeDetails: DEMO_TRADES[0],
    amount: 50550,
    status: OversightStatus.PENDING,
    requiredApprovers: ['approver-1', 'approver-2'],
    approvals: [],
    approvalsRequired: 1,
    priority: OversightPriority.HIGH,
    reason: 'Trade value exceeds oversight threshold ($50,000)',
    riskAssessment: {
      riskScore: 65,
      riskLevel: 'medium',
      factors: [
        {
          name: 'High Value',
          description: 'Trade value $50,550 exceeds oversight threshold',
          impact: 70,
          severity: 'medium',
        },
        {
          name: 'Market Volatility',
          description: 'NVDA showing high volatility in current session',
          impact: 60,
          severity: 'medium',
        },
        {
          name: 'Position Concentration',
          description: 'Would represent 30% of portfolio value',
          impact: 55,
          severity: 'medium',
        },
      ],
      recommendation: 'review',
      assessedAt: new Date('2024-12-26T14:00:30Z'),
    },
    createdAt: new Date('2024-12-26T14:00:00Z'),
    expiresAt: new Date('2024-12-26T18:00:00Z'),
    resolvedAt: null,
    autoApprovalAttempted: true,
    autoApprovalFailureReason: 'Amount exceeds auto-approval limit ($10,000)',
  },
  {
    id: 'oversight-002',
    agentId: 'agent-balanced-003',
    tradeId: 'trade-004',
    tradeDetails: DEMO_TRADES[3],
    amount: 35500,
    status: OversightStatus.REJECTED,
    requiredApprovers: ['approver-1'],
    approvals: [
      {
        id: 'approval-001',
        requestId: 'oversight-002',
        approverId: 'approver-1',
        approverName: 'Risk Manager',
        decision: 'reject',
        reason: 'Exceeds maximum position size limit. Would represent 47% of portfolio.',
        timestamp: new Date('2024-12-20T11:00:00Z'),
        metadata: {
          ipAddress: '10.0.1.50',
          userAgent: 'Mozilla/5.0',
          authMethod: 'oauth2',
          reviewDuration: 180,
        },
      },
    ],
    approvalsRequired: 1,
    priority: OversightPriority.MEDIUM,
    reason: 'Trade value requires approval and risk assessment',
    riskAssessment: {
      riskScore: 85,
      riskLevel: 'high',
      factors: [
        {
          name: 'Position Size Violation',
          description: 'Would exceed 20% position size limit',
          impact: 90,
          severity: 'high',
        },
        {
          name: 'Portfolio Concentration',
          description: 'Meta position would be 47% of total portfolio',
          impact: 85,
          severity: 'high',
        },
      ],
      recommendation: 'reject',
      assessedAt: new Date('2024-12-20T10:30:30Z'),
    },
    createdAt: new Date('2024-12-20T10:30:00Z'),
    expiresAt: new Date('2024-12-20T14:30:00Z'),
    resolvedAt: new Date('2024-12-20T11:00:00Z'),
    autoApprovalAttempted: false,
    autoApprovalFailureReason: null,
  },
];

// ============================================================================
// METERING & USAGE DATA
// ============================================================================

export const DEMO_USAGE_METRICS: Record<string, UsageMetrics> = {
  'agent-alpha-001': {
    agentId: 'agent-alpha-001',
    apiCalls: 8547,
    quotaUsed: 8547,
    quotaLimit: 10000,
    quotaPercent: 85.47,
    period: {
      type: 'daily',
      number: 26,
      year: 2024,
    },
    periodStart: new Date('2024-12-26T00:00:00Z'),
    periodEnd: new Date('2024-12-26T23:59:59Z'),
    costIncurred: 42.74,
    breakdown: {
      marketDataCalls: 5200,
      tradeExecutionCalls: 2100,
      portfolioAnalysisCalls: 947,
      oversightCalls: 300,
      costs: {
        marketData: 26.00,
        tradeExecution: 10.50,
        portfolioAnalysis: 4.74,
        oversight: 1.50,
      },
    },
  },
  'agent-safehaven-002': {
    agentId: 'agent-safehaven-002',
    apiCalls: 3215,
    quotaUsed: 3215,
    quotaLimit: 5000,
    quotaPercent: 64.30,
    period: {
      type: 'daily',
      number: 26,
      year: 2024,
    },
    periodStart: new Date('2024-12-26T00:00:00Z'),
    periodEnd: new Date('2024-12-26T23:59:59Z'),
    costIncurred: 16.08,
    breakdown: {
      marketDataCalls: 2100,
      tradeExecutionCalls: 800,
      portfolioAnalysisCalls: 265,
      oversightCalls: 50,
      costs: {
        marketData: 10.50,
        tradeExecution: 4.00,
        portfolioAnalysis: 1.33,
        oversight: 0.25,
      },
    },
  },
  'agent-balanced-003': {
    agentId: 'agent-balanced-003',
    apiCalls: 5832,
    quotaUsed: 5832,
    quotaLimit: 7500,
    quotaPercent: 77.76,
    period: {
      type: 'daily',
      number: 26,
      year: 2024,
    },
    periodStart: new Date('2024-12-26T00:00:00Z'),
    periodEnd: new Date('2024-12-26T23:59:59Z'),
    costIncurred: 29.16,
    breakdown: {
      marketDataCalls: 3500,
      tradeExecutionCalls: 1500,
      portfolioAnalysisCalls: 682,
      oversightCalls: 150,
      costs: {
        marketData: 17.50,
        tradeExecution: 7.50,
        portfolioAnalysis: 3.41,
        oversight: 0.75,
      },
    },
  },
};

// ============================================================================
// QUOTA CONFIGURATIONS
// ============================================================================

export const DEMO_QUOTA_CONFIGS: QuotaConfig[] = [
  {
    tenantId: 'agent-alpha-001',
    resource: 'api_calls',
    limit: 10000,
    resetPeriod: 'daily',
    currentUsage: 8547,
    lastReset: new Date('2024-12-26T00:00:00Z'),
    warningThreshold: 0.8,
  },
  {
    tenantId: 'agent-safehaven-002',
    resource: 'api_calls',
    limit: 5000,
    resetPeriod: 'daily',
    currentUsage: 3215,
    lastReset: new Date('2024-12-26T00:00:00Z'),
    warningThreshold: 0.8,
  },
  {
    tenantId: 'agent-balanced-003',
    resource: 'api_calls',
    limit: 7500,
    resetPeriod: 'daily',
    currentUsage: 5832,
    lastReset: new Date('2024-12-26T00:00:00Z'),
    warningThreshold: 0.8,
  },
];

// ============================================================================
// PERFORMANCE METRICS
// ============================================================================

export const DEMO_PERFORMANCE_METRICS: Record<string, PerformanceMetrics> = {
  'portfolio-alpha-001': {
    totalReturn: 2.65,
    annualizedReturn: 34.45,
    sharpeRatio: 1.85,
    maxDrawdown: -8.2,
    winRate: 67.5,
    avgWin: 1250,
    avgLoss: -680,
    totalTrades: 45,
  },
  'portfolio-safehaven-002': {
    totalReturn: 1.28,
    annualizedReturn: 16.64,
    sharpeRatio: 2.10,
    maxDrawdown: -3.5,
    winRate: 72.0,
    avgWin: 580,
    avgLoss: -320,
    totalTrades: 28,
  },
  'portfolio-balanced-003': {
    totalReturn: 1.96,
    annualizedReturn: 25.48,
    sharpeRatio: 1.95,
    maxDrawdown: -5.8,
    winRate: 69.5,
    avgWin: 920,
    avgLoss: -485,
    totalTrades: 36,
  },
};

// ============================================================================
// EXPORT HELPERS
// ============================================================================

export function getAllAgents(): Agent[] {
  return Object.values(DEMO_AGENTS);
}

export function getAllTrades(): Trade[] {
  return DEMO_TRADES;
}

export function getAllOversightRequests(): OversightRequest[] {
  return DEMO_OVERSIGHT_REQUESTS;
}

export function getAllPositions(): Position[] {
  return Object.values(DEMO_POSITIONS).flat();
}

export function getAgentPositions(portfolioId: string): Position[] {
  return DEMO_POSITIONS[portfolioId] || [];
}

export function getAgentUsage(agentId: string): UsageMetrics | undefined {
  return DEMO_USAGE_METRICS[agentId];
}

export function getAgentQuota(agentId: string): QuotaConfig | undefined {
  return DEMO_QUOTA_CONFIGS.find(q => q.tenantId === agentId);
}

export function getPortfolioMetrics(portfolioId: string): PerformanceMetrics | undefined {
  return DEMO_PERFORMANCE_METRICS[portfolioId];
}
