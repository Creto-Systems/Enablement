-- Trading Demo Database Schema
-- SQLite schema for agents, portfolios, positions, trades, oversight, and metering

-- ============================================================================
-- AGENTS
-- ============================================================================

CREATE TABLE IF NOT EXISTS agents (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('active', 'paused', 'suspended', 'terminated')),
  budget REAL NOT NULL,
  available_budget REAL NOT NULL,
  created_at TEXT NOT NULL,
  last_active_at TEXT NOT NULL,
  config TEXT NOT NULL,
  portfolio_id TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_agents_status ON agents(status);
CREATE INDEX IF NOT EXISTS idx_agents_portfolio ON agents(portfolio_id);

-- ============================================================================
-- PORTFOLIOS
-- ============================================================================

CREATE TABLE IF NOT EXISTS portfolios (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  total_value REAL NOT NULL DEFAULT 0,
  cash_balance REAL NOT NULL DEFAULT 0,
  daily_change REAL NOT NULL DEFAULT 0,
  daily_change_percent REAL NOT NULL DEFAULT 0,
  realized_pnl REAL NOT NULL DEFAULT 0,
  unrealized_pnl REAL NOT NULL DEFAULT 0,
  metrics TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_portfolios_agent ON portfolios(agent_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_portfolios_agent_unique ON portfolios(agent_id);

-- ============================================================================
-- POSITIONS
-- ============================================================================

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
  opened_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  FOREIGN KEY (portfolio_id) REFERENCES portfolios(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_positions_portfolio ON positions(portfolio_id);
CREATE INDEX IF NOT EXISTS idx_positions_symbol ON positions(symbol);

-- ============================================================================
-- TRADES
-- ============================================================================

CREATE TABLE IF NOT EXISTS trades (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  symbol TEXT NOT NULL,
  side TEXT NOT NULL CHECK(side IN ('buy', 'sell')),
  type TEXT NOT NULL CHECK(type IN ('market', 'limit', 'stop', 'stop_limit')),
  quantity REAL NOT NULL,
  price REAL,
  limit_price REAL,
  stop_price REAL,
  total_value REAL NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('pending', 'submitted', 'pending_approval', 'approved', 'rejected', 'partially_filled', 'filled', 'cancelled', 'failed')),
  filled_quantity REAL NOT NULL DEFAULT 0,
  avg_fill_price REAL,
  timestamp TEXT NOT NULL,
  filled_at TEXT,
  oversight_request_id TEXT,
  rejection_reason TEXT,
  metadata TEXT NOT NULL,
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_trades_agent ON trades(agent_id);
CREATE INDEX IF NOT EXISTS idx_trades_status ON trades(status);
CREATE INDEX IF NOT EXISTS idx_trades_symbol ON trades(symbol);
CREATE INDEX IF NOT EXISTS idx_trades_timestamp ON trades(timestamp);
CREATE INDEX IF NOT EXISTS idx_trades_oversight ON trades(oversight_request_id);

-- ============================================================================
-- USAGE METRICS (Metering)
-- ============================================================================

CREATE TABLE IF NOT EXISTS usage_metrics (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  api_calls INTEGER NOT NULL DEFAULT 0,
  quota_used INTEGER NOT NULL DEFAULT 0,
  quota_limit INTEGER NOT NULL,
  quota_percent REAL NOT NULL DEFAULT 0,
  period_type TEXT NOT NULL CHECK(period_type IN ('hourly', 'daily', 'weekly', 'monthly')),
  period_number INTEGER NOT NULL,
  period_year INTEGER NOT NULL,
  period_start TEXT NOT NULL,
  period_end TEXT NOT NULL,
  cost_incurred REAL NOT NULL DEFAULT 0,
  breakdown TEXT NOT NULL,
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_usage_agent ON usage_metrics(agent_id);
CREATE INDEX IF NOT EXISTS idx_usage_period ON usage_metrics(period_type, period_number, period_year);

-- ============================================================================
-- BILLABLE EVENTS
-- ============================================================================

CREATE TABLE IF NOT EXISTS billable_events (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  event_type TEXT NOT NULL CHECK(event_type IN ('api_call', 'trade_execution', 'data_retrieval', 'oversight_request', 'storage')),
  resource TEXT NOT NULL,
  quantity REAL NOT NULL,
  unit_cost REAL NOT NULL,
  total_cost REAL NOT NULL,
  timestamp TEXT NOT NULL,
  metadata TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_events_tenant ON billable_events(tenant_id);
CREATE INDEX IF NOT EXISTS idx_events_type ON billable_events(event_type);
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON billable_events(timestamp);

-- ============================================================================
-- QUOTA CONFIGS
-- ============================================================================

CREATE TABLE IF NOT EXISTS quota_configs (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  resource TEXT NOT NULL,
  limit INTEGER NOT NULL,
  reset_period TEXT NOT NULL CHECK(reset_period IN ('hourly', 'daily', 'weekly', 'monthly')),
  current_usage INTEGER NOT NULL DEFAULT 0,
  last_reset TEXT NOT NULL,
  warning_threshold REAL NOT NULL DEFAULT 0.8
);

CREATE INDEX IF NOT EXISTS idx_quota_tenant ON quota_configs(tenant_id);
CREATE UNIQUE INDEX IF NOT EXISTS idx_quota_tenant_resource ON quota_configs(tenant_id, resource);

-- ============================================================================
-- OVERSIGHT REQUESTS
-- ============================================================================

CREATE TABLE IF NOT EXISTS oversight_requests (
  id TEXT PRIMARY KEY,
  agent_id TEXT NOT NULL,
  trade_id TEXT NOT NULL,
  amount REAL NOT NULL,
  status TEXT NOT NULL CHECK(status IN ('pending', 'approved', 'rejected', 'expired', 'cancelled')),
  required_approvers TEXT NOT NULL,
  approvals_required INTEGER NOT NULL,
  priority TEXT NOT NULL CHECK(priority IN ('low', 'medium', 'high', 'urgent')),
  reason TEXT NOT NULL,
  risk_assessment TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  resolved_at TEXT,
  auto_approval_attempted INTEGER NOT NULL DEFAULT 0,
  auto_approval_failure_reason TEXT,
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE,
  FOREIGN KEY (trade_id) REFERENCES trades(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_oversight_agent ON oversight_requests(agent_id);
CREATE INDEX IF NOT EXISTS idx_oversight_trade ON oversight_requests(trade_id);
CREATE INDEX IF NOT EXISTS idx_oversight_status ON oversight_requests(status);
CREATE INDEX IF NOT EXISTS idx_oversight_created ON oversight_requests(created_at);

-- ============================================================================
-- APPROVAL DECISIONS
-- ============================================================================

CREATE TABLE IF NOT EXISTS approval_decisions (
  id TEXT PRIMARY KEY,
  request_id TEXT NOT NULL,
  approver_id TEXT NOT NULL,
  approver_name TEXT NOT NULL,
  decision TEXT NOT NULL CHECK(decision IN ('approve', 'reject')),
  reason TEXT NOT NULL,
  timestamp TEXT NOT NULL,
  metadata TEXT NOT NULL,
  FOREIGN KEY (request_id) REFERENCES oversight_requests(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_decisions_request ON approval_decisions(request_id);
CREATE INDEX IF NOT EXISTS idx_decisions_approver ON approval_decisions(approver_id);

-- ============================================================================
-- OVERSIGHT POLICIES
-- ============================================================================

CREATE TABLE IF NOT EXISTS oversight_policies (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  threshold REAL NOT NULL,
  approvals_required INTEGER NOT NULL,
  approval_timeout INTEGER NOT NULL,
  auto_approval_rules TEXT NOT NULL,
  escalation_rules TEXT NOT NULL,
  notifications TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_policies_enabled ON oversight_policies(enabled);

-- ============================================================================
-- PORTFOLIO HISTORY (for charts)
-- ============================================================================

CREATE TABLE IF NOT EXISTS portfolio_history (
  id TEXT PRIMARY KEY,
  portfolio_id TEXT NOT NULL,
  agent_id TEXT NOT NULL,
  total_value REAL NOT NULL,
  cash_balance REAL NOT NULL,
  timestamp TEXT NOT NULL,
  FOREIGN KEY (portfolio_id) REFERENCES portfolios(id) ON DELETE CASCADE,
  FOREIGN KEY (agent_id) REFERENCES agents(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_history_portfolio ON portfolio_history(portfolio_id);
CREATE INDEX IF NOT EXISTS idx_history_timestamp ON portfolio_history(timestamp);
