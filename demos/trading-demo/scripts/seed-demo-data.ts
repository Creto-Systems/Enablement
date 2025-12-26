#!/usr/bin/env tsx
/**
 * Seed Demo Data Script
 *
 * Populates the trading demo with realistic data for investor presentations.
 * Demonstrates all 4 Enablement products:
 * - Metering: API quotas and billing
 * - Oversight: Approval workflows
 * - Runtime: Agent execution
 * - Messaging: Event notifications
 *
 * Usage:
 *   npm run demo:seed
 *   tsx scripts/seed-demo-data.ts
 */

import Database from 'better-sqlite3';
import { existsSync, mkdirSync } from 'fs';
import { join } from 'path';
import {
  getAllAgents,
  getAllTrades,
  getAllOversightRequests,
  getAllPositions,
  DEMO_USAGE_METRICS,
  DEMO_QUOTA_CONFIGS,
  DEMO_PERFORMANCE_METRICS,
} from '../tests/fixtures/mock-data';

const DB_PATH = join(process.cwd(), 'data');
const DB_FILE = join(DB_PATH, 'trading-demo.db');

// ============================================================================
// DATABASE SETUP
// ============================================================================

function initializeDatabase(): Database {
  console.log('🔧 Initializing database...');

  // Ensure data directory exists
  if (!existsSync(DB_PATH)) {
    mkdirSync(DB_PATH, { recursive: true });
  }

  const db = new Database(DB_FILE);

  // Enable foreign keys
  db.pragma('foreign_keys = ON');

  console.log('✅ Database initialized at:', DB_FILE);
  return db;
}

function createTables(db: Database): void {
  console.log('📋 Creating tables...');

  // Agents table
  db.exec(`
    CREATE TABLE IF NOT EXISTS agents (
      id TEXT PRIMARY KEY,
      name TEXT NOT NULL,
      status TEXT NOT NULL,
      budget REAL NOT NULL,
      available_budget REAL NOT NULL,
      created_at TEXT NOT NULL,
      last_active_at TEXT NOT NULL,
      config TEXT NOT NULL,
      portfolio_id TEXT NOT NULL
    );
  `);

  // Portfolios table
  db.exec(`
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
      updated_at TEXT NOT NULL,
      FOREIGN KEY (agent_id) REFERENCES agents(id)
    );
  `);

  // Positions table
  db.exec(`
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
      FOREIGN KEY (portfolio_id) REFERENCES portfolios(id)
    );
  `);

  // Trades table
  db.exec(`
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
      timestamp TEXT NOT NULL,
      filled_at TEXT,
      oversight_request_id TEXT,
      rejection_reason TEXT,
      metadata TEXT NOT NULL,
      FOREIGN KEY (agent_id) REFERENCES agents(id)
    );
  `);

  // Oversight requests table
  db.exec(`
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
      created_at TEXT NOT NULL,
      expires_at TEXT NOT NULL,
      resolved_at TEXT,
      auto_approval_attempted INTEGER NOT NULL,
      auto_approval_failure_reason TEXT,
      FOREIGN KEY (agent_id) REFERENCES agents(id),
      FOREIGN KEY (trade_id) REFERENCES trades(id)
    );
  `);

  // Approval decisions table
  db.exec(`
    CREATE TABLE IF NOT EXISTS approval_decisions (
      id TEXT PRIMARY KEY,
      request_id TEXT NOT NULL,
      approver_id TEXT NOT NULL,
      approver_name TEXT NOT NULL,
      decision TEXT NOT NULL,
      reason TEXT NOT NULL,
      timestamp TEXT NOT NULL,
      metadata TEXT NOT NULL,
      FOREIGN KEY (request_id) REFERENCES oversight_requests(id)
    );
  `);

  // Usage metrics table
  db.exec(`
    CREATE TABLE IF NOT EXISTS usage_metrics (
      id TEXT PRIMARY KEY,
      agent_id TEXT NOT NULL,
      api_calls INTEGER NOT NULL,
      quota_used INTEGER NOT NULL,
      quota_limit INTEGER NOT NULL,
      quota_percent REAL NOT NULL,
      period_type TEXT NOT NULL,
      period_start TEXT NOT NULL,
      period_end TEXT NOT NULL,
      cost_incurred REAL NOT NULL,
      breakdown TEXT NOT NULL,
      FOREIGN KEY (agent_id) REFERENCES agents(id)
    );
  `);

  // Quota configs table
  db.exec(`
    CREATE TABLE IF NOT EXISTS quota_configs (
      id TEXT PRIMARY KEY,
      tenant_id TEXT NOT NULL,
      resource TEXT NOT NULL,
      limit_value INTEGER NOT NULL,
      reset_period TEXT NOT NULL,
      current_usage INTEGER NOT NULL,
      last_reset TEXT NOT NULL,
      warning_threshold REAL NOT NULL
    );
  `);

  console.log('✅ Tables created successfully');
}

// ============================================================================
// DATA SEEDING
// ============================================================================

function seedAgents(db: Database): void {
  console.log('👥 Seeding agents...');

  const insertAgent = db.prepare(`
    INSERT INTO agents (
      id, name, status, budget, available_budget,
      created_at, last_active_at, config, portfolio_id
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const agents = getAllAgents();
  for (const agent of agents) {
    insertAgent.run(
      agent.id,
      agent.name,
      agent.status,
      agent.budget,
      agent.availableBudget,
      agent.createdAt.toISOString(),
      agent.lastActiveAt.toISOString(),
      JSON.stringify(agent.config),
      agent.portfolioId
    );
    console.log(`  ✓ ${agent.name} (${agent.config.strategy})`);
  }
}

function seedPortfolios(db: Database): void {
  console.log('💼 Seeding portfolios...');

  const insertPortfolio = db.prepare(`
    INSERT INTO portfolios (
      id, agent_id, total_value, cash_balance,
      daily_change, daily_change_percent,
      realized_pnl, unrealized_pnl, metrics, updated_at
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const agents = getAllAgents();
  for (const agent of agents) {
    const positions = getAllPositions().filter(p => p.portfolioId === agent.portfolioId);
    const totalPositionValue = positions.reduce((sum, p) => sum + p.marketValue, 0);
    const totalPnl = positions.reduce((sum, p) => sum + p.pnl, 0);
    const metrics = DEMO_PERFORMANCE_METRICS[agent.portfolioId];

    insertPortfolio.run(
      agent.portfolioId,
      agent.id,
      totalPositionValue + agent.availableBudget,
      agent.availableBudget,
      totalPnl,
      (totalPnl / (totalPositionValue - totalPnl)) * 100,
      0,
      totalPnl,
      JSON.stringify(metrics),
      new Date().toISOString()
    );
    console.log(`  ✓ Portfolio ${agent.portfolioId} ($${(totalPositionValue + agent.availableBudget).toFixed(2)})`);
  }
}

function seedPositions(db: Database): void {
  console.log('📊 Seeding positions...');

  const insertPosition = db.prepare(`
    INSERT INTO positions (
      id, portfolio_id, symbol, quantity, avg_price,
      current_price, market_value, pnl, pnl_percent,
      opened_at, updated_at
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const positions = getAllPositions();
  for (const position of positions) {
    insertPosition.run(
      position.id,
      position.portfolioId,
      position.symbol,
      position.quantity,
      position.avgPrice,
      position.currentPrice,
      position.marketValue,
      position.pnl,
      position.pnlPercent,
      position.openedAt.toISOString(),
      position.updatedAt.toISOString()
    );
    console.log(`  ✓ ${position.symbol}: ${position.quantity} shares @ $${position.currentPrice}`);
  }
}

function seedTrades(db: Database): void {
  console.log('📈 Seeding trades...');

  const insertTrade = db.prepare(`
    INSERT INTO trades (
      id, agent_id, symbol, side, type, quantity,
      price, limit_price, stop_price, total_value,
      status, filled_quantity, avg_fill_price,
      timestamp, filled_at, oversight_request_id,
      rejection_reason, metadata
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const trades = getAllTrades();
  for (const trade of trades) {
    insertTrade.run(
      trade.id,
      trade.agentId,
      trade.symbol,
      trade.side,
      trade.type,
      trade.quantity,
      trade.price,
      trade.limitPrice || null,
      trade.stopPrice || null,
      trade.totalValue,
      trade.status,
      trade.filledQuantity,
      trade.avgFillPrice,
      trade.timestamp.toISOString(),
      trade.filledAt?.toISOString() || null,
      trade.oversightRequestId,
      trade.rejectionReason,
      JSON.stringify(trade.metadata)
    );
    console.log(`  ✓ ${trade.side} ${trade.quantity} ${trade.symbol} - ${trade.status}`);
  }
}

function seedOversight(db: Database): void {
  console.log('👁️  Seeding oversight requests...');

  const insertRequest = db.prepare(`
    INSERT INTO oversight_requests (
      id, agent_id, trade_id, amount, status,
      required_approvers, approvals_required, priority,
      reason, risk_assessment, created_at, expires_at,
      resolved_at, auto_approval_attempted, auto_approval_failure_reason
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const insertApproval = db.prepare(`
    INSERT INTO approval_decisions (
      id, request_id, approver_id, approver_name,
      decision, reason, timestamp, metadata
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const requests = getAllOversightRequests();
  for (const request of requests) {
    insertRequest.run(
      request.id,
      request.agentId,
      request.tradeId,
      request.amount,
      request.status,
      JSON.stringify(request.requiredApprovers),
      request.approvalsRequired,
      request.priority,
      request.reason,
      JSON.stringify(request.riskAssessment),
      request.createdAt.toISOString(),
      request.expiresAt.toISOString(),
      request.resolvedAt?.toISOString() || null,
      request.autoApprovalAttempted ? 1 : 0,
      request.autoApprovalFailureReason
    );

    for (const approval of request.approvals) {
      insertApproval.run(
        approval.id,
        approval.requestId,
        approval.approverId,
        approval.approverName,
        approval.decision,
        approval.reason,
        approval.timestamp.toISOString(),
        JSON.stringify(approval.metadata)
      );
    }

    console.log(`  ✓ Request ${request.id} - ${request.status} ($${request.amount})`);
  }
}

function seedMetering(db: Database): void {
  console.log('📊 Seeding metering data...');

  const insertUsage = db.prepare(`
    INSERT INTO usage_metrics (
      id, agent_id, api_calls, quota_used, quota_limit,
      quota_percent, period_type, period_start, period_end,
      cost_incurred, breakdown
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
  `);

  const insertQuota = db.prepare(`
    INSERT INTO quota_configs (
      id, tenant_id, resource, limit_value, reset_period,
      current_usage, last_reset, warning_threshold
    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
  `);

  // Insert usage metrics
  for (const [agentId, metrics] of Object.entries(DEMO_USAGE_METRICS)) {
    insertUsage.run(
      `usage-${agentId}`,
      agentId,
      metrics.apiCalls,
      metrics.quotaUsed,
      metrics.quotaLimit,
      metrics.quotaPercent,
      metrics.period.type,
      metrics.periodStart.toISOString(),
      metrics.periodEnd.toISOString(),
      metrics.costIncurred,
      JSON.stringify(metrics.breakdown)
    );
    console.log(`  ✓ ${agentId}: ${metrics.quotaPercent.toFixed(1)}% quota used`);
  }

  // Insert quota configs
  for (const quota of DEMO_QUOTA_CONFIGS) {
    insertQuota.run(
      `quota-${quota.tenantId}`,
      quota.tenantId,
      quota.resource,
      quota.limit,
      quota.resetPeriod,
      quota.currentUsage,
      quota.lastReset.toISOString(),
      quota.warningThreshold
    );
  }
}

// ============================================================================
// MAIN
// ============================================================================

function main(): void {
  console.log('🌱 Starting demo data seed...\n');

  try {
    const db = initializeDatabase();

    createTables(db);
    console.log();

    seedAgents(db);
    console.log();

    seedPortfolios(db);
    console.log();

    seedPositions(db);
    console.log();

    seedTrades(db);
    console.log();

    seedOversight(db);
    console.log();

    seedMetering(db);
    console.log();

    db.close();

    console.log('✨ Demo data seeded successfully!\n');
    console.log('📊 Summary:');
    console.log(`  • 3 demo agents with different strategies`);
    console.log(`  • 3 portfolios with active positions`);
    console.log(`  • ${getAllPositions().length} portfolio positions`);
    console.log(`  • ${getAllTrades().length} trade history records`);
    console.log(`  • ${getAllOversightRequests().length} oversight requests`);
    console.log(`  • Usage metrics and quota configs`);
    console.log('\n🚀 Ready for investor demo!\n');

  } catch (error) {
    console.error('❌ Error seeding data:', error);
    process.exit(1);
  }
}

// Run automatically
main();
