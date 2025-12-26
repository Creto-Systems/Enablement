#!/usr/bin/env tsx
/**
 * Interactive Demo Scenario Runner
 *
 * Runs through 5 key demo scenarios for investor presentations:
 * 1. Create new agent (shows metering quota initialization)
 * 2. Execute small trade (shows real-time updates)
 * 3. Execute large trade (shows oversight workflow)
 * 4. Approve pending trade (shows approval flow)
 * 5. Hit quota limit (shows metering enforcement)
 *
 * Usage:
 *   npm run demo:run
 *   tsx scripts/demo-scenario.ts [scenario-number]
 */

import { randomUUID } from 'crypto';
import * as readline from 'readline';

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
});

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

function prompt(question: string): Promise<string> {
  return new Promise((resolve) => {
    rl.question(question, resolve);
  });
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

function formatCurrency(amount: number): string {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency: 'USD',
  }).format(amount);
}

function printHeader(title: string): void {
  console.log('\n' + '='.repeat(70));
  console.log(`  ${title}`);
  console.log('='.repeat(70) + '\n');
}

function printStep(step: number, total: number, description: string): void {
  console.log(`\n[${ step}/${total}] ${description}`);
  console.log('-'.repeat(70));
}

function printSuccess(message: string): void {
  console.log(`✅ ${message}`);
}

function printWarning(message: string): void {
  console.log(`⚠️  ${message}`);
}

function printError(message: string): void {
  console.log(`❌ ${message}`);
}

function printInfo(message: string): void {
  console.log(`ℹ️  ${message}`);
}

// ============================================================================
// SCENARIO 1: Create New Agent
// ============================================================================

async function scenario1_CreateAgent(): Promise<void> {
  printHeader('SCENARIO 1: Create New Agent');
  console.log('Demonstrates: Metering quota initialization');
  console.log('Products shown: Metering, Runtime\n');

  await prompt('Press Enter to start...');

  printStep(1, 3, 'Collecting agent configuration');
  const agentId = randomUUID();
  const config = {
    id: agentId,
    name: 'Demo Momentum Trader',
    strategy: 'momentum' as const,
    budget: 50000,
    riskTolerance: 6,
    dailyTradeLimit: 25,
    oversightThreshold: 25000,
  };

  console.log('\nAgent Configuration:');
  console.log(`  ID: ${config.id}`);
  console.log(`  Name: ${config.name}`);
  console.log(`  Strategy: ${config.strategy}`);
  console.log(`  Budget: ${formatCurrency(config.budget)}`);
  console.log(`  Risk Tolerance: ${config.riskTolerance}/10`);

  await sleep(1000);
  printSuccess('Configuration validated');

  printStep(2, 3, 'Initializing metering quota');
  await sleep(800);

  const quota = {
    tenantId: agentId,
    resource: 'api_calls',
    limit: 5000,
    resetPeriod: 'daily',
    currentUsage: 0,
    warningThreshold: 0.8,
  };

  console.log('\nQuota Configuration:');
  console.log(`  Daily API Call Limit: ${quota.limit.toLocaleString()}`);
  console.log(`  Current Usage: ${quota.currentUsage}`);
  console.log(`  Warning Threshold: ${quota.warningThreshold * 100}%`);
  console.log(`  Reset Period: ${quota.resetPeriod}`);

  await sleep(1000);
  printSuccess('Metering quota initialized');

  printStep(3, 3, 'Starting agent runtime');
  await sleep(1200);

  console.log('\nAgent Status:');
  console.log(`  Status: ACTIVE ✓`);
  console.log(`  Runtime: Sandboxed execution environment ready`);
  console.log(`  API Quota: 5,000 calls available`);
  console.log(`  Budget: ${formatCurrency(config.budget)} allocated`);

  printSuccess('Agent created and running');
  console.log('\n📊 Metering: Agent quota tracked in creto-metering');
  console.log('🚀 Runtime: Agent executing in creto-runtime sandbox');
}

// ============================================================================
// SCENARIO 2: Execute Small Trade
// ============================================================================

async function scenario2_SmallTrade(): Promise<void> {
  printHeader('SCENARIO 2: Execute Small Trade');
  console.log('Demonstrates: Real-time trade execution and updates');
  console.log('Products shown: Runtime, Metering, Messaging\n');

  await prompt('Press Enter to start...');

  printStep(1, 4, 'Agent analyzes market opportunity');
  await sleep(1000);

  const trade = {
    id: randomUUID(),
    agentId: 'agent-alpha-001',
    symbol: 'AAPL',
    side: 'buy' as const,
    quantity: 20,
    price: 197.50,
    totalValue: 3950,
    strategy: 'momentum',
    confidence: 0.82,
  };

  console.log('\nTrade Analysis:');
  console.log(`  Symbol: ${trade.symbol}`);
  console.log(`  Action: ${trade.side.toUpperCase()}`);
  console.log(`  Quantity: ${trade.quantity} shares`);
  console.log(`  Price: ${formatCurrency(trade.price)}`);
  console.log(`  Total Value: ${formatCurrency(trade.totalValue)}`);
  console.log(`  Confidence: ${(trade.confidence * 100).toFixed(0)}%`);

  printSuccess('Market opportunity identified');

  printStep(2, 4, 'Checking metering quota');
  await sleep(800);

  console.log('\nQuota Check:');
  console.log(`  Current Usage: 3,215 / 5,000 calls (64.3%)`);
  console.log(`  Trade Execution Cost: 5 API calls`);
  console.log(`  Post-Trade Usage: 3,220 / 5,000 calls (64.4%)`);

  printSuccess('Quota available - proceeding with trade');

  printStep(3, 4, 'Executing trade');
  await sleep(600);
  console.log('  Submitting order to market...');
  await sleep(800);
  console.log('  Order matched and filled');
  await sleep(400);

  printSuccess('Trade executed successfully');

  printStep(4, 4, 'Broadcasting real-time updates');
  await sleep(500);

  console.log('\nEvent Messages:');
  console.log(`  ✉️  TRADE_SUBMITTED → WebSocket subscribers`);
  await sleep(300);
  console.log(`  ✉️  TRADE_FILLED → Portfolio service`);
  await sleep(300);
  console.log(`  ✉️  POSITION_UPDATED → Analytics service`);
  await sleep(300);
  console.log(`  ✉️  QUOTA_UPDATED → Metering service`);

  printSuccess('All systems updated in real-time');
  console.log('\n📊 Metering: 5 API calls recorded');
  console.log('🚀 Runtime: Trade executed in <200ms');
  console.log('📨 Messaging: 4 event messages delivered');
}

// ============================================================================
// SCENARIO 3: Execute Large Trade (Oversight)
// ============================================================================

async function scenario3_LargeTrade(): Promise<void> {
  printHeader('SCENARIO 3: Execute Large Trade');
  console.log('Demonstrates: Oversight workflow for high-value trades');
  console.log('Products shown: Oversight, Metering, Messaging\n');

  await prompt('Press Enter to start...');

  printStep(1, 5, 'Agent proposes large trade');
  await sleep(1000);

  const trade = {
    id: randomUUID(),
    agentId: 'agent-alpha-001',
    symbol: 'NVDA',
    side: 'buy' as const,
    quantity: 100,
    price: 505.50,
    totalValue: 50550,
    oversightThreshold: 50000,
  };

  console.log('\nProposed Trade:');
  console.log(`  Symbol: ${trade.symbol}`);
  console.log(`  Action: ${trade.side.toUpperCase()}`);
  console.log(`  Quantity: ${trade.quantity} shares`);
  console.log(`  Price: ${formatCurrency(trade.price)}`);
  console.log(`  Total Value: ${formatCurrency(trade.totalValue)}`);
  console.log(`  Oversight Threshold: ${formatCurrency(trade.oversightThreshold)}`);

  await sleep(800);
  printWarning(`Trade value exceeds oversight threshold!`);

  printStep(2, 5, 'Checking auto-approval rules');
  await sleep(1000);

  console.log('\nAuto-Approval Check:');
  console.log(`  Auto-approval limit: $10,000`);
  console.log(`  Trade amount: ${formatCurrency(trade.totalValue)}`);
  console.log(`  Result: DENIED - exceeds limit`);

  printInfo('Human approval required');

  printStep(3, 5, 'Creating oversight request');
  await sleep(1200);

  const oversightRequest = {
    id: randomUUID(),
    tradeId: trade.id,
    amount: trade.totalValue,
    priority: 'high',
    approvalsRequired: 1,
    riskScore: 65,
    riskLevel: 'medium',
  };

  console.log('\nOversight Request Created:');
  console.log(`  Request ID: ${oversightRequest.id}`);
  console.log(`  Priority: ${oversightRequest.priority.toUpperCase()}`);
  console.log(`  Approvals Required: ${oversightRequest.approvalsRequired}`);
  console.log(`  Risk Score: ${oversightRequest.riskScore}/100`);
  console.log(`  Risk Level: ${oversightRequest.riskLevel}`);

  printSuccess('Oversight request submitted');

  printStep(4, 5, 'Running risk assessment');
  await sleep(1500);

  console.log('\nRisk Factors:');
  console.log(`  ⚠️  High Value (impact: 70/100)`);
  console.log(`     Trade value $50,550 exceeds oversight threshold`);
  console.log(`  ⚠️  Market Volatility (impact: 60/100)`);
  console.log(`     NVDA showing high volatility in current session`);
  console.log(`  ⚠️  Position Concentration (impact: 55/100)`);
  console.log(`     Would represent 30% of portfolio value`);

  await sleep(800);
  console.log(`\n  Overall Risk: MEDIUM (score: ${oversightRequest.riskScore}/100)`);
  console.log(`  Recommendation: REVIEW`);

  printSuccess('Risk assessment complete');

  printStep(5, 5, 'Notifying approvers');
  await sleep(800);

  console.log('\nNotifications Sent:');
  console.log(`  📧 Email → approver-1@example.com`);
  await sleep(300);
  console.log(`  💬 Slack → #trading-oversight channel`);
  await sleep(300);
  console.log(`  📱 SMS → Risk Manager`);

  printSuccess('Approvers notified - awaiting decision');
  console.log('\n👁️  Oversight: Request pending in approval queue');
  console.log('📊 Metering: Oversight API calls tracked');
  console.log('📨 Messaging: Multi-channel notifications delivered');
}

// ============================================================================
// SCENARIO 4: Approve Pending Trade
// ============================================================================

async function scenario4_ApproveTradeAsync function scenario4_ApproveTrade(): Promise<void> {
  printHeader('SCENARIO 4: Approve Pending Trade');
  console.log('Demonstrates: Human-in-the-loop approval workflow');
  console.log('Products shown: Oversight, Runtime, Messaging\n');

  await prompt('Press Enter to start...');

  printStep(1, 4, 'Viewing pending oversight request');
  await sleep(1000);

  const request = {
    id: 'oversight-001',
    tradeId: 'trade-001',
    agentId: 'agent-alpha-001',
    symbol: 'NVDA',
    amount: 50550,
    status: 'pending',
    createdAt: new Date(Date.now() - 30 * 60 * 1000), // 30 min ago
    expiresAt: new Date(Date.now() + 3.5 * 60 * 60 * 1000), // 3.5 hours from now
  };

  console.log('\nPending Request:');
  console.log(`  Request ID: ${request.id}`);
  console.log(`  Trade: ${request.symbol} - ${formatCurrency(request.amount)}`);
  console.log(`  Agent: ${request.agentId}`);
  console.log(`  Status: ${request.status.toUpperCase()}`);
  console.log(`  Age: 30 minutes`);
  console.log(`  Expires: 3.5 hours remaining`);

  printInfo('Awaiting approver decision');

  printStep(2, 4, 'Approver reviews request');
  await sleep(1500);

  console.log('\nApprover Dashboard:');
  console.log(`  👤 Approver: Risk Manager`);
  console.log(`  📊 Risk Assessment: MEDIUM (65/100)`);
  console.log(`  📈 Agent Performance: +2.65% total return`);
  console.log(`  💰 Available Budget: $73,500`);
  console.log(`  ✅ Strategy Alignment: High`);

  await sleep(1000);
  printSuccess('Review complete - decision: APPROVE');

  printStep(3, 4, 'Recording approval decision');
  await sleep(800);

  const approval = {
    id: randomUUID(),
    requestId: request.id,
    approverId: 'approver-1',
    approverName: 'Risk Manager',
    decision: 'approve',
    reason: 'Trade aligns with momentum strategy. Risk acceptable given portfolio size.',
    reviewDuration: 180, // 3 minutes
  };

  console.log('\nApproval Decision:');
  console.log(`  Decision: ${approval.decision.toUpperCase()} ✓`);
  console.log(`  Approver: ${approval.approverName}`);
  console.log(`  Reason: ${approval.reason}`);
  console.log(`  Review Time: ${approval.reviewDuration} seconds`);

  printSuccess('Approval recorded');

  printStep(4, 4, 'Executing approved trade');
  await sleep(1000);

  console.log('\nTrade Execution:');
  console.log(`  Status: APPROVED → SUBMITTED`);
  await sleep(600);
  console.log(`  Order sent to market...`);
  await sleep(800);
  console.log(`  Order filled at $505.50`);
  await sleep(400);
  console.log(`  Portfolio updated`);

  printSuccess('Trade executed successfully');

  console.log('\nEvent Messages:');
  console.log(`  ✉️  OVERSIGHT_APPROVED → Agent runtime`);
  await sleep(300);
  console.log(`  ✉️  TRADE_EXECUTED → Portfolio service`);
  await sleep(300);
  console.log(`  ✉️  APPROVAL_NOTIFICATION → Submitting agent`);

  console.log('\n👁️  Oversight: Human approval workflow complete');
  console.log('🚀 Runtime: Trade executed post-approval');
  console.log('📨 Messaging: Approval notifications sent');
}

// ============================================================================
// SCENARIO 5: Hit Quota Limit
// ============================================================================

async function scenario5_QuotaLimit(): Promise<void> {
  printHeader('SCENARIO 5: Hit Quota Limit');
  console.log('Demonstrates: Metering enforcement and quota management');
  console.log('Products shown: Metering, Messaging\n');

  await prompt('Press Enter to start...');

  printStep(1, 4, 'Agent approaching quota limit');
  await sleep(1000);

  const quota = {
    agentId: 'agent-alpha-001',
    limit: 10000,
    currentUsage: 9750,
    warningThreshold: 8000,
    percentUsed: 97.5,
  };

  console.log('\nCurrent Quota Status:');
  console.log(`  Agent: ${quota.agentId}`);
  console.log(`  Usage: ${quota.currentUsage.toLocaleString()} / ${quota.limit.toLocaleString()} calls`);
  console.log(`  Percentage: ${quota.percentUsed.toFixed(1)}%`);
  console.log(`  Warning Threshold: ${quota.warningThreshold.toLocaleString()} calls (80%)`);

  printWarning(`Quota at ${quota.percentUsed}% - approaching limit!`);

  printStep(2, 4, 'Agent attempts high-frequency operations');
  await sleep(1200);

  console.log('\nAPI Call Simulation:');
  for (let i = 1; i <= 5; i++) {
    await sleep(400);
    const newUsage = quota.currentUsage + (i * 50);
    const percentUsed = (newUsage / quota.limit) * 100;

    if (newUsage < quota.limit) {
      console.log(`  ✓ Batch ${i}: ${newUsage.toLocaleString()} / ${quota.limit.toLocaleString()} (${percentUsed.toFixed(1)}%)`);
    } else {
      console.log(`  ❌ Batch ${i}: QUOTA EXCEEDED!`);
      printError('API calls blocked by metering service');
      break;
    }
  }

  printStep(3, 4, 'Metering enforcement triggered');
  await sleep(1000);

  console.log('\nEnforcement Actions:');
  console.log(`  🚫 Block: All non-essential API calls`);
  console.log(`  ⚠️  Throttle: Reduce rate to minimal operational level`);
  console.log(`  📧 Notify: Alert agent owner of quota breach`);
  console.log(`  📊 Log: Record quota violation event`);

  await sleep(800);
  printWarning('Agent operations throttled to prevent quota breach');

  printStep(4, 4, 'Quota reset options');
  await sleep(1000);

  console.log('\nAvailable Actions:');
  console.log(`  1. Wait for daily reset (8 hours remaining)`);
  console.log(`  2. Purchase additional quota (upgrade tier)`);
  console.log(`  3. Optimize API usage (reduce call frequency)`);

  console.log('\nQuota Reset Schedule:');
  console.log(`  Reset Period: Daily (00:00 UTC)`);
  console.log(`  Next Reset: ${new Date(Date.now() + 8 * 60 * 60 * 1000).toLocaleString()}`);
  console.log(`  New Quota: 10,000 calls`);

  printSuccess('Quota management enforced');

  console.log('\n📊 Metering: Real-time quota tracking and enforcement');
  console.log('🚫 Runtime: Agent throttled to prevent over-usage');
  console.log('📨 Messaging: Quota alerts and notifications sent');
}

// ============================================================================
// MENU & MAIN
// ============================================================================

async function showMenu(): Promise<void> {
  console.clear();
  printHeader('Trading Demo - Interactive Scenarios');

  console.log('Available Scenarios:\n');
  console.log('  1. Create New Agent');
  console.log('     → Demonstrates metering quota initialization');
  console.log('     → Products: Metering, Runtime\n');

  console.log('  2. Execute Small Trade');
  console.log('     → Demonstrates real-time execution and updates');
  console.log('     → Products: Runtime, Metering, Messaging\n');

  console.log('  3. Execute Large Trade (Oversight)');
  console.log('     → Demonstrates oversight workflow');
  console.log('     → Products: Oversight, Metering, Messaging\n');

  console.log('  4. Approve Pending Trade');
  console.log('     → Demonstrates human-in-the-loop approval');
  console.log('     → Products: Oversight, Runtime, Messaging\n');

  console.log('  5. Hit Quota Limit');
  console.log('     → Demonstrates metering enforcement');
  console.log('     → Products: Metering, Messaging\n');

  console.log('  6. Run All Scenarios (Full Demo)');
  console.log('  0. Exit\n');
}

async function runScenario(choice: string): Promise<boolean> {
  switch (choice) {
    case '1':
      await scenario1_CreateAgent();
      return true;
    case '2':
      await scenario2_SmallTrade();
      return true;
    case '3':
      await scenario3_LargeTrade();
      return true;
    case '4':
      await scenario4_ApproveTrade();
      return true;
    case '5':
      await scenario5_QuotaLimit();
      return true;
    case '6':
      await scenario1_CreateAgent();
      await prompt('\nPress Enter to continue to next scenario...');
      await scenario2_SmallTrade();
      await prompt('\nPress Enter to continue to next scenario...');
      await scenario3_LargeTrade();
      await prompt('\nPress Enter to continue to next scenario...');
      await scenario4_ApproveTrade();
      await prompt('\nPress Enter to continue to next scenario...');
      await scenario5_QuotaLimit();
      return true;
    case '0':
      console.log('\n👋 Exiting demo...\n');
      return false;
    default:
      printError('Invalid choice. Please try again.');
      await sleep(1500);
      return true;
  }
}

async function main(): Promise<void> {
  // Check if scenario number provided as argument
  const args = process.argv.slice(2);
  if (args.length > 0) {
    const scenarioNum = args[0];
    const continueRunning = await runScenario(scenarioNum);
    if (!continueRunning) {
      rl.close();
      return;
    }
    console.log('\n✨ Scenario complete!\n');
    rl.close();
    return;
  }

  // Interactive menu mode
  let running = true;
  while (running) {
    await showMenu();
    const choice = await prompt('Select scenario (0-6): ');
    running = await runScenario(choice.trim());

    if (running) {
      await prompt('\nPress Enter to return to menu...');
    }
  }

  rl.close();
}

// Run if called directly
if (require.main === module) {
  main();
}

export {
  scenario1_CreateAgent,
  scenario2_SmallTrade,
  scenario3_LargeTrade,
  scenario4_ApproveTrade,
  scenario5_QuotaLimit,
};
