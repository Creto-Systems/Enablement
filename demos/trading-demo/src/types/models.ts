/**
 * Trading Demo - Data Models
 *
 * Complete TypeScript interfaces for frontend and backend integration
 * Maps to creto-metering and creto-oversight Rust types
 */

// ============================================================================
// CORE DOMAIN MODELS
// ============================================================================

/**
 * Trading Agent
 * Represents an autonomous AI agent that executes trades
 */
export interface Agent {
  /** Unique agent identifier */
  id: string;

  /** Human-readable agent name */
  name: string;

  /** Current operational status */
  status: AgentStatus;

  /** Allocated trading budget in USD */
  budget: number;

  /** Remaining available budget */
  availableBudget: number;

  /** Agent creation timestamp */
  createdAt: Date;

  /** Last activity timestamp */
  lastActiveAt: Date;

  /** Agent configuration */
  config: AgentConfig;

  /** Associated portfolio ID */
  portfolioId: string;
}

export enum AgentStatus {
  ACTIVE = 'active',
  PAUSED = 'paused',
  SUSPENDED = 'suspended',
  TERMINATED = 'terminated'
}

export interface AgentConfig {
  /** Trading strategy type */
  strategy: TradingStrategy;

  /** Risk tolerance level (0-10) */
  riskTolerance: number;

  /** Maximum position size as % of portfolio */
  maxPositionSize: number;

  /** Daily trade limit */
  dailyTradeLimit: number;

  /** Oversight threshold in USD */
  oversightThreshold: number;

  /** Allowed trading symbols */
  allowedSymbols?: string[];

  /** Auto-approval enabled for small trades */
  autoApprovalEnabled: boolean;

  /** Auto-approval limit in USD */
  autoApprovalLimit: number;
}

export enum TradingStrategy {
  MOMENTUM = 'momentum',
  MEAN_REVERSION = 'mean_reversion',
  ARBITRAGE = 'arbitrage',
  MARKET_MAKING = 'market_making',
  CUSTOM = 'custom'
}

// ============================================================================
// PORTFOLIO & POSITIONS
// ============================================================================

/**
 * Trading Portfolio
 * Aggregates all positions and performance for an agent
 */
export interface Portfolio {
  /** Unique portfolio identifier */
  id: string;

  /** Associated agent ID */
  agentId: string;

  /** All current positions */
  positions: Position[];

  /** Total portfolio value in USD */
  totalValue: number;

  /** Cash available for trading */
  cashBalance: number;

  /** Daily change in USD */
  dailyChange: number;

  /** Daily change percentage */
  dailyChangePercent: number;

  /** Total realized P&L */
  realizedPnL: number;

  /** Total unrealized P&L */
  unrealizedPnL: number;

  /** Performance metrics */
  metrics: PerformanceMetrics;

  /** Last update timestamp */
  updatedAt: Date;
}

/**
 * Individual Position
 * Represents holdings in a specific security
 */
export interface Position {
  /** Position identifier */
  id: string;

  /** Portfolio ID */
  portfolioId: string;

  /** Trading symbol (e.g., AAPL, MSFT) */
  symbol: string;

  /** Number of shares held */
  quantity: number;

  /** Average purchase price per share */
  avgPrice: number;

  /** Current market price per share */
  currentPrice: number;

  /** Current market value */
  marketValue: number;

  /** Unrealized profit/loss in USD */
  pnl: number;

  /** Unrealized profit/loss percentage */
  pnlPercent: number;

  /** Position opened timestamp */
  openedAt: Date;

  /** Last price update timestamp */
  updatedAt: Date;
}

export interface PerformanceMetrics {
  /** Total return percentage */
  totalReturn: number;

  /** Annualized return percentage */
  annualizedReturn: number;

  /** Sharpe ratio */
  sharpeRatio: number;

  /** Maximum drawdown percentage */
  maxDrawdown: number;

  /** Win rate percentage */
  winRate: number;

  /** Average win amount */
  avgWin: number;

  /** Average loss amount */
  avgLoss: number;

  /** Total number of trades */
  totalTrades: number;
}

// ============================================================================
// TRADING OPERATIONS
// ============================================================================

/**
 * Trade Order
 * Represents a buy or sell order
 */
export interface Trade {
  /** Unique trade identifier */
  id: string;

  /** Agent executing the trade */
  agentId: string;

  /** Trading symbol */
  symbol: string;

  /** Buy or sell */
  side: TradeSide;

  /** Order type */
  type: OrderType;

  /** Number of shares */
  quantity: number;

  /** Price per share (null for market orders) */
  price: number | null;

  /** Limit price for limit orders */
  limitPrice?: number;

  /** Stop price for stop orders */
  stopPrice?: number;

  /** Total trade value in USD */
  totalValue: number;

  /** Current order status */
  status: TradeStatus;

  /** Filled quantity */
  filledQuantity: number;

  /** Average fill price */
  avgFillPrice: number | null;

  /** Order submission timestamp */
  timestamp: Date;

  /** Fill timestamp */
  filledAt: Date | null;

  /** Associated oversight request ID */
  oversightRequestId: string | null;

  /** Rejection reason if applicable */
  rejectionReason: string | null;

  /** Execution metadata */
  metadata: TradeMetadata;
}

export enum TradeSide {
  BUY = 'buy',
  SELL = 'sell'
}

export enum OrderType {
  MARKET = 'market',
  LIMIT = 'limit',
  STOP = 'stop',
  STOP_LIMIT = 'stop_limit'
}

export enum TradeStatus {
  PENDING = 'pending',
  SUBMITTED = 'submitted',
  PENDING_APPROVAL = 'pending_approval',
  APPROVED = 'approved',
  REJECTED = 'rejected',
  PARTIALLY_FILLED = 'partially_filled',
  FILLED = 'filled',
  CANCELLED = 'cancelled',
  FAILED = 'failed'
}

export interface TradeMetadata {
  /** Strategy that generated the trade */
  strategy: TradingStrategy;

  /** Confidence score (0-1) */
  confidence: number;

  /** Trade rationale */
  rationale: string;

  /** Expected profit target */
  profitTarget?: number;

  /** Stop loss level */
  stopLoss?: number;

  /** Time in force */
  timeInForce: 'day' | 'gtc' | 'ioc' | 'fok';
}

// ============================================================================
// METERING & USAGE (creto-metering integration)
// ============================================================================

/**
 * Usage Metrics
 * Tracks API calls and quota consumption
 * Maps to creto-metering BillableEvent
 */
export interface UsageMetrics {
  /** Agent identifier */
  agentId: string;

  /** Total API calls made */
  apiCalls: number;

  /** Quota used in current period */
  quotaUsed: number;

  /** Total quota limit */
  quotaLimit: number;

  /** Quota usage percentage */
  quotaPercent: number;

  /** Billing period */
  period: BillingPeriod;

  /** Period start date */
  periodStart: Date;

  /** Period end date */
  periodEnd: Date;

  /** Cost incurred in USD */
  costIncurred: number;

  /** Detailed usage breakdown */
  breakdown: UsageBreakdown;
}

export interface BillingPeriod {
  /** Period type */
  type: 'hourly' | 'daily' | 'weekly' | 'monthly';

  /** Period number */
  number: number;

  /** Year */
  year: number;
}

export interface UsageBreakdown {
  /** Market data API calls */
  marketDataCalls: number;

  /** Trade execution calls */
  tradeExecutionCalls: number;

  /** Portfolio analysis calls */
  portfolioAnalysisCalls: number;

  /** Oversight API calls */
  oversightCalls: number;

  /** Cost per category */
  costs: {
    marketData: number;
    tradeExecution: number;
    portfolioAnalysis: number;
    oversight: number;
  };
}

/**
 * Billable Event
 * Maps to creto-metering BillableEvent Rust struct
 */
export interface BillableEvent {
  /** Event identifier */
  id: string;

  /** Agent/tenant identifier */
  tenantId: string;

  /** Event type */
  eventType: BillableEventType;

  /** Resource consumed */
  resource: string;

  /** Quantity consumed */
  quantity: number;

  /** Cost per unit */
  unitCost: number;

  /** Total cost */
  totalCost: number;

  /** Event timestamp */
  timestamp: Date;

  /** Additional metadata */
  metadata: Record<string, unknown>;
}

export enum BillableEventType {
  API_CALL = 'api_call',
  TRADE_EXECUTION = 'trade_execution',
  DATA_RETRIEVAL = 'data_retrieval',
  OVERSIGHT_REQUEST = 'oversight_request',
  STORAGE = 'storage'
}

/**
 * Quota Configuration
 * Maps to creto-metering QuotaConfig
 */
export interface QuotaConfig {
  /** Tenant/agent identifier */
  tenantId: string;

  /** Resource type */
  resource: string;

  /** Maximum allowed units */
  limit: number;

  /** Reset period */
  resetPeriod: 'hourly' | 'daily' | 'weekly' | 'monthly';

  /** Current usage */
  currentUsage: number;

  /** Last reset timestamp */
  lastReset: Date;

  /** Soft limit warning threshold */
  warningThreshold: number;
}

// ============================================================================
// OVERSIGHT & APPROVALS (creto-oversight integration)
// ============================================================================

/**
 * Oversight Request
 * Request for human approval of high-value trades
 * Maps to creto-oversight types
 */
export interface OversightRequest {
  /** Unique request identifier */
  id: string;

  /** Agent requesting approval */
  agentId: string;

  /** Associated trade ID */
  tradeId: string;

  /** Trade details for review */
  tradeDetails: Trade;

  /** Approval amount in USD */
  amount: number;

  /** Current request status */
  status: OversightStatus;

  /** Required approvers */
  requiredApprovers: string[];

  /** Completed approvals */
  approvals: ApprovalDecision[];

  /** Number of required approvals */
  approvalsRequired: number;

  /** Request priority */
  priority: OversightPriority;

  /** Request reason/justification */
  reason: string;

  /** Risk assessment */
  riskAssessment: RiskAssessment;

  /** Request creation timestamp */
  createdAt: Date;

  /** Expiration timestamp */
  expiresAt: Date;

  /** Resolution timestamp */
  resolvedAt: Date | null;

  /** Auto-approval attempted */
  autoApprovalAttempted: boolean;

  /** Auto-approval failure reason */
  autoApprovalFailureReason: string | null;
}

export enum OversightStatus {
  PENDING = 'pending',
  APPROVED = 'approved',
  REJECTED = 'rejected',
  EXPIRED = 'expired',
  CANCELLED = 'cancelled'
}

export enum OversightPriority {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  URGENT = 'urgent'
}

/**
 * Approval Decision
 * Individual approver's decision
 */
export interface ApprovalDecision {
  /** Decision identifier */
  id: string;

  /** Oversight request ID */
  requestId: string;

  /** Approver identifier */
  approverId: string;

  /** Approver name */
  approverName: string;

  /** Approve or reject */
  decision: 'approve' | 'reject';

  /** Decision reason/comments */
  reason: string;

  /** Decision timestamp */
  timestamp: Date;

  /** Decision metadata */
  metadata: ApprovalMetadata;
}

export interface ApprovalMetadata {
  /** IP address */
  ipAddress: string;

  /** User agent */
  userAgent: string;

  /** Authentication method */
  authMethod: string;

  /** Review duration in seconds */
  reviewDuration: number;
}

/**
 * Oversight Policy
 * Maps to creto-oversight OversightPolicy Rust struct
 */
export interface OversightPolicy {
  /** Policy identifier */
  id: string;

  /** Policy name */
  name: string;

  /** Policy description */
  description: string;

  /** Enabled/disabled */
  enabled: boolean;

  /** Approval threshold in USD */
  threshold: number;

  /** Required number of approvals */
  approvalsRequired: number;

  /** Approval timeout in seconds */
  approvalTimeout: number;

  /** Auto-approval rules */
  autoApprovalRules: AutoApprovalRule[];

  /** Escalation rules */
  escalationRules: EscalationRule[];

  /** Notification settings */
  notifications: NotificationSettings;
}

export interface AutoApprovalRule {
  /** Rule identifier */
  id: string;

  /** Rule name */
  name: string;

  /** Amount threshold for auto-approval */
  maxAmount: number;

  /** Allowed agents */
  allowedAgents?: string[];

  /** Allowed symbols */
  allowedSymbols?: string[];

  /** Time restrictions */
  timeRestrictions?: TimeRestriction[];

  /** Additional conditions */
  conditions: Record<string, unknown>;
}

export interface EscalationRule {
  /** Time in seconds before escalation */
  escalateAfter: number;

  /** Escalation level */
  level: number;

  /** Escalation approvers */
  approvers: string[];

  /** Notification method */
  notificationMethod: 'email' | 'sms' | 'slack' | 'webhook';
}

export interface TimeRestriction {
  /** Days of week (0=Sunday, 6=Saturday) */
  daysOfWeek: number[];

  /** Start time (HH:MM format) */
  startTime: string;

  /** End time (HH:MM format) */
  endTime: string;

  /** Timezone */
  timezone: string;
}

export interface NotificationSettings {
  /** Email notifications enabled */
  email: boolean;

  /** SMS notifications enabled */
  sms: boolean;

  /** Slack notifications enabled */
  slack: boolean;

  /** Webhook notifications enabled */
  webhook: boolean;

  /** Webhook URL */
  webhookUrl?: string;
}

/**
 * Risk Assessment
 * Automated risk analysis for trades
 */
export interface RiskAssessment {
  /** Overall risk score (0-100) */
  riskScore: number;

  /** Risk level */
  riskLevel: 'low' | 'medium' | 'high' | 'critical';

  /** Risk factors */
  factors: RiskFactor[];

  /** Recommendation */
  recommendation: 'approve' | 'reject' | 'review';

  /** Assessment timestamp */
  assessedAt: Date;
}

export interface RiskFactor {
  /** Factor name */
  name: string;

  /** Factor description */
  description: string;

  /** Impact score (0-100) */
  impact: number;

  /** Factor severity */
  severity: 'low' | 'medium' | 'high' | 'critical';
}

// ============================================================================
// VALIDATION SCHEMAS
// ============================================================================

/**
 * Validation Constraints
 */
export const ValidationRules = {
  budget: {
    min: 1000,        // $1K minimum
    max: 1000000,     // $1M maximum
  },
  trade: {
    minAmount: 1,     // $1 minimum
    maxAmount: 1000000, // $1M maximum
  },
  oversight: {
    defaultThreshold: 50000, // $50K default
    minThreshold: 1000,
    maxThreshold: 1000000,
  },
  symbol: {
    pattern: /^[A-Z]{1,5}$/,  // 1-5 uppercase letters
    maxLength: 5,
  },
  agent: {
    maxDailyTrades: 1000,
    maxPositionSize: 0.5, // 50% of portfolio
    minRiskTolerance: 0,
    maxRiskTolerance: 10,
  },
  quota: {
    defaultDailyApiCalls: 10000,
    maxDailyApiCalls: 100000,
  },
} as const;

/**
 * Validation Functions
 */
export const validators = {
  isValidBudget: (amount: number): boolean => {
    return amount >= ValidationRules.budget.min &&
           amount <= ValidationRules.budget.max;
  },

  isValidTradeAmount: (amount: number): boolean => {
    return amount >= ValidationRules.trade.minAmount &&
           amount <= ValidationRules.trade.maxAmount;
  },

  isValidSymbol: (symbol: string): boolean => {
    return ValidationRules.symbol.pattern.test(symbol);
  },

  requiresOversight: (amount: number, threshold: number): boolean => {
    return amount >= threshold;
  },

  isWithinBudget: (amount: number, availableBudget: number): boolean => {
    return amount <= availableBudget;
  },

  isValidRiskTolerance: (level: number): boolean => {
    return level >= ValidationRules.agent.minRiskTolerance &&
           level <= ValidationRules.agent.maxRiskTolerance;
  },
};

// ============================================================================
// DATABASE SCHEMA (SQLite for Demo)
// ============================================================================

/**
 * SQL Schema Definitions
 * For SQLite persistence in demo environment
 */
export const DatabaseSchema = {
  agents: `
    CREATE TABLE IF NOT EXISTS agents (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      status TEXT NOT NULL,
      budget REAL NOT NULL,
      available_budget REAL NOT NULL,
      created_at DATETIME NOT NULL,
      last_active_at DATETIME NOT NULL,
      config TEXT NOT NULL,
      portfolio_id TEXT NOT NULL
    );
  `,

  portfolios: `
    CREATE TABLE IF NOT EXISTS portfolios (
      id TEXT PRIMARY KEY,
      agent_id TEXT NOT NULL,
      total_value REAL NOT NULL,
      cash_balance REAL NOT NULL,
      daily_change REAL NOT NULL,
      daily_change_percent REAL NOT NULL,
      realized_pnl REAL NOT NULL,
      unrealized_pnl REAL NOT NULL,
      metrics TEXT NOT NULL,
      updated_at DATETIME NOT NULL,
      FOREIGN KEY (agent_id) REFERENCES agents(id)
    );
  `,

  positions: `
    CREATE TABLE IF NOT EXISTS positions (
      id TEXT PRIMARY KEY,
      portfolio_id TEXT NOT NULL,
      symbol TEXT NOT NULL,
      quantity REAL NOT NULL,
      avg_price REAL NOT NULL,
      current_price REAL NOT NULL,
      market_value REAL NOT NULL,
      pnl REAL NOT NULL,
      pnl_percent REAL NOT NULL,
      opened_at DATETIME NOT NULL,
      updated_at DATETIME NOT NULL,
      FOREIGN KEY (portfolio_id) REFERENCES portfolios(id)
    );
  `,

  trades: `
    CREATE TABLE IF NOT EXISTS trades (
      id TEXT PRIMARY KEY,
      agent_id TEXT NOT NULL,
      symbol TEXT NOT NULL,
      side TEXT NOT NULL,
      type TEXT NOT NULL,
      quantity REAL NOT NULL,
      price REAL,
      limit_price REAL,
      stop_price REAL,
      total_value REAL NOT NULL,
      status TEXT NOT NULL,
      filled_quantity REAL NOT NULL,
      avg_fill_price REAL,
      timestamp DATETIME NOT NULL,
      filled_at DATETIME,
      oversight_request_id TEXT,
      rejection_reason TEXT,
      metadata TEXT NOT NULL,
      FOREIGN KEY (agent_id) REFERENCES agents(id)
    );
  `,

  usage_metrics: `
    CREATE TABLE IF NOT EXISTS usage_metrics (
      id TEXT PRIMARY KEY,
      agent_id TEXT NOT NULL,
      api_calls INTEGER NOT NULL,
      quota_used INTEGER NOT NULL,
      quota_limit INTEGER NOT NULL,
      quota_percent REAL NOT NULL,
      period_type TEXT NOT NULL,
      period_start DATETIME NOT NULL,
      period_end DATETIME NOT NULL,
      cost_incurred REAL NOT NULL,
      breakdown TEXT NOT NULL,
      FOREIGN KEY (agent_id) REFERENCES agents(id)
    );
  `,

  oversight_requests: `
    CREATE TABLE IF NOT EXISTS oversight_requests (
      id TEXT PRIMARY KEY,
      agent_id TEXT NOT NULL,
      trade_id TEXT NOT NULL,
      amount REAL NOT NULL,
      status TEXT NOT NULL,
      required_approvers TEXT NOT NULL,
      approvals_required INTEGER NOT NULL,
      priority TEXT NOT NULL,
      reason TEXT NOT NULL,
      risk_assessment TEXT NOT NULL,
      created_at DATETIME NOT NULL,
      expires_at DATETIME NOT NULL,
      resolved_at DATETIME,
      auto_approval_attempted INTEGER NOT NULL,
      auto_approval_failure_reason TEXT,
      FOREIGN KEY (agent_id) REFERENCES agents(id),
      FOREIGN KEY (trade_id) REFERENCES trades(id)
    );
  `,

  approval_decisions: `
    CREATE TABLE IF NOT EXISTS approval_decisions (
      id TEXT PRIMARY KEY,
      request_id TEXT NOT NULL,
      approver_id TEXT NOT NULL,
      approver_name TEXT NOT NULL,
      decision TEXT NOT NULL,
      reason TEXT NOT NULL,
      timestamp DATETIME NOT NULL,
      metadata TEXT NOT NULL,
      FOREIGN KEY (request_id) REFERENCES oversight_requests(id)
    );
  `,

  oversight_policies: `
    CREATE TABLE IF NOT EXISTS oversight_policies (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      description TEXT NOT NULL,
      enabled INTEGER NOT NULL,
      threshold REAL NOT NULL,
      approvals_required INTEGER NOT NULL,
      approval_timeout INTEGER NOT NULL,
      auto_approval_rules TEXT NOT NULL,
      escalation_rules TEXT NOT NULL,
      notifications TEXT NOT NULL
    );
  `,

  quota_configs: `
    CREATE TABLE IF NOT EXISTS quota_configs (
      id TEXT PRIMARY KEY,
      tenant_id TEXT NOT NULL,
      resource TEXT NOT NULL,
      limit INTEGER NOT NULL,
      reset_period TEXT NOT NULL,
      current_usage INTEGER NOT NULL,
      last_reset DATETIME NOT NULL,
      warning_threshold REAL NOT NULL
    );
  `,
} as const;

// ============================================================================
// TYPE GUARDS
// ============================================================================

export function isAgent(obj: unknown): obj is Agent {
  const a = obj as Agent;
  return typeof a?.id === 'string' &&
         typeof a?.name === 'string' &&
         typeof a?.budget === 'number';
}

export function isTrade(obj: unknown): obj is Trade {
  const t = obj as Trade;
  return typeof t?.id === 'string' &&
         typeof t?.agentId === 'string' &&
         typeof t?.symbol === 'string';
}

export function isOversightRequest(obj: unknown): obj is OversightRequest {
  const o = obj as OversightRequest;
  return typeof o?.id === 'string' &&
         typeof o?.agentId === 'string' &&
         typeof o?.amount === 'number';
}

// ============================================================================
// UTILITY TYPES
// ============================================================================

/** Partial update for entities */
export type PartialUpdate<T> = Partial<T> & { id: string };

/** API response wrapper */
export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: Date;
}

/** Paginated response */
export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

/** Event notification */
export interface SystemEvent {
  type: string;
  payload: Record<string, unknown>;
  timestamp: Date;
  source: string;
}

// All types are exported inline via their interface/type declarations above
