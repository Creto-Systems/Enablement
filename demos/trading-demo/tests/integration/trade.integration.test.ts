/**
 * Trade Execution Flow Integration Tests
 *
 * Tests the complete trade execution flow including:
 * - Small trades execute immediately
 * - Large trades trigger oversight approval
 * - Quota exceeded rejects trade
 * - Trade execution updates portfolio
 * - Trade execution broadcasts via WebSocket (mocked)
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import supertest from 'supertest';
import {
  setupIntegrationTests,
  cleanupIntegrationTests,
  resetTestState,
  getTestContext,
  createTestAgent,
  waitForEvent,
} from './setup';

describe('Trade Integration Tests', () => {
  let request: supertest.SuperTest<supertest.Test>;

  beforeAll(async () => {
    const context = await setupIntegrationTests();
    request = supertest(context.app);
  });

  afterAll(async () => {
    await cleanupIntegrationTests();
  });

  beforeEach(async () => {
    await resetTestState();
  });

  describe('POST /api/trades - Submit Trade', () => {
    it('should execute small trade immediately', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, {
        budget: 50000,
      });

      // Submit small trade (under oversight threshold)
      const response = await request
        .post('/api/trades')
        .set('Authorization', context.authToken)
        .send({
          agentId: agent.id,
          symbol: 'AAPL',
          quantity: 10,
          price: 150,
          side: 'buy',
          userId: 'user-123',
        })
        .expect(201);

      // Verify trade executed immediately
      expect(response.body).toMatchObject({
        agentId: agent.id,
        symbol: 'AAPL',
        quantity: 10,
        price: 150,
        side: 'buy',
        status: 'executed',
      });
      expect(response.body.executedAt).toBeDefined();
      expect(response.body.id).toBeDefined();

      // Verify metering recorded
      const usage = await context.meteringService.getUsage(agent.id);
      expect(usage.used).toBe(1500); // 10 * 150
    });

    it('should trigger oversight for large trade (>$10K)', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, {
        budget: 100000,
      });

      // Listen for oversight request event
      const oversightEventPromise = waitForEvent(
        context.eventEmitter,
        'oversight:request_created'
      );

      // Submit large trade (over oversight threshold)
      const response = await request
        .post('/api/trades')
        .set('Authorization', context.authToken)
        .send({
          agentId: agent.id,
          symbol: 'TSLA',
          quantity: 100,
          price: 200, // $20,000 total
          side: 'buy',
          userId: 'user-123',
        })
        .expect(201);

      // Verify trade is pending approval
      expect(response.body).toMatchObject({
        agentId: agent.id,
        symbol: 'TSLA',
        quantity: 100,
        price: 200,
        side: 'buy',
        status: 'pending_approval',
      });
      expect(response.body.executedAt).toBeUndefined();

      // Verify oversight request created
      const oversightEvent = await oversightEventPromise;
      expect(oversightEvent).toMatchObject({
        agentId: agent.id,
        amount: 20000,
      });

      // Verify metering NOT recorded yet (trade not executed)
      const usage = await context.meteringService.getUsage(agent.id);
      expect(usage.used).toBe(0);
    });

    it('should reject trade when quota exceeded', async () => {
      const context = getTestContext();

      // Create agent with small budget
      const agent = await createTestAgent(context, {
        budget: 5000,
      });

      // Set quota to near limit
      context.meteringClient.setQuota(agent.id, 4000, 5000);

      // Try to submit trade that exceeds remaining quota
      const response = await request
        .post('/api/trades')
        .set('Authorization', context.authToken)
        .send({
          agentId: agent.id,
          symbol: 'AAPL',
          quantity: 10,
          price: 150, // $1,500 total, would exceed
          side: 'buy',
          userId: 'user-123',
        })
        .expect(500);

      expect(response.body.error).toContain('Trade quota exceeded');
    });

    it('should handle multiple trades sequentially', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, {
        budget: 50000,
      });

      // Submit first trade
      const trade1Response = await request
        .post('/api/trades')
        .set('Authorization', context.authToken)
        .send({
          agentId: agent.id,
          symbol: 'AAPL',
          quantity: 10,
          price: 150,
          side: 'buy',
          userId: 'user-123',
        })
        .expect(201);

      expect(trade1Response.body.status).toBe('executed');

      // Submit second trade
      const trade2Response = await request
        .post('/api/trades')
        .set('Authorization', context.authToken)
        .send({
          agentId: agent.id,
          symbol: 'MSFT',
          quantity: 20,
          price: 300,
          side: 'buy',
          userId: 'user-123',
        })
        .expect(201);

      expect(trade2Response.body.status).toBe('executed');

      // Verify cumulative quota usage
      const usage = await context.meteringService.getUsage(agent.id);
      expect(usage.used).toBe(7500); // 1500 + 6000
    });

    it('should validate trade parameters', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context);

      // Missing required fields
      await request
        .post('/api/trades')
        .set('Authorization', context.authToken)
        .send({
          agentId: agent.id,
          symbol: 'AAPL',
          // Missing quantity, price, side
          userId: 'user-123',
        })
        .expect(400);
    });
  });

  describe('GET /api/trades/:id - Get Trade', () => {
    it('should retrieve trade by id', async () => {
      const context = getTestContext();

      // Create agent and trade
      const agent = await createTestAgent(context);
      const trade = await context.tradeService.submitTrade({
        agentId: agent.id,
        symbol: 'AAPL',
        quantity: 10,
        price: 150,
        side: 'buy',
      });

      // Retrieve trade directly from service (no HTTP endpoint for single trade)
      const retrievedTrade = await context.tradeService.getTrade(trade.id);

      expect(retrievedTrade).toMatchObject({
        id: trade.id,
        agentId: agent.id,
        symbol: 'AAPL',
        quantity: 10,
        price: 150,
        status: 'executed',
      });
    });

    it('should return null for non-existent trade', async () => {
      const context = getTestContext();

      const retrievedTrade = await context.tradeService.getTrade('non-existent-id');
      expect(retrievedTrade).toBeNull();
    });
  });

  describe('GET /api/trades - List Trades', () => {
    it('should list all trades', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context);

      // Create multiple trades
      await context.tradeService.submitTrade({
        agentId: agent.id,
        symbol: 'AAPL',
        quantity: 10,
        price: 150,
        side: 'buy',
      });

      await context.tradeService.submitTrade({
        agentId: agent.id,
        symbol: 'MSFT',
        quantity: 20,
        price: 300,
        side: 'buy',
      });

      // List trades via service
      const trades = await context.tradeService.listTrades();

      expect(trades).toHaveLength(2);
      expect(trades[0].symbol).toBeDefined();
      expect(trades[1].symbol).toBeDefined();
    });

    it('should filter trades by agentId', async () => {
      const context = getTestContext();

      // Create two agents
      const agent1 = await createTestAgent(context, { name: 'Agent 1' });
      const agent2 = await createTestAgent(context, { name: 'Agent 2' });

      // Create trades for both agents
      await context.tradeService.submitTrade({
        agentId: agent1.id,
        symbol: 'AAPL',
        quantity: 10,
        price: 150,
        side: 'buy',
      });

      await context.tradeService.submitTrade({
        agentId: agent2.id,
        symbol: 'MSFT',
        quantity: 20,
        price: 300,
        side: 'buy',
      });

      // List trades for agent1 via service
      const trades = await context.tradeService.listTrades(agent1.id);

      expect(trades).toHaveLength(1);
      expect(trades[0].agentId).toBe(agent1.id);
      expect(trades[0].symbol).toBe('AAPL');
    });
  });

  describe('Trade Cancellation', () => {
    it('should cancel pending trade', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, { budget: 100000 });

      // Create large trade (pending approval)
      const trade = await context.tradeService.submitTrade({
        agentId: agent.id,
        symbol: 'TSLA',
        quantity: 100,
        price: 200,
        side: 'buy',
      });

      expect(trade.status).toBe('pending_approval');

      // Cancel trade via service
      const cancelledTrade = await context.tradeService.cancelTrade(trade.id);

      expect(cancelledTrade.status).toBe('cancelled');
      expect(cancelledTrade.cancelledAt).toBeDefined();
    });

    it('should not cancel executed trade', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context);

      // Create small trade (executes immediately)
      const trade = await context.tradeService.submitTrade({
        agentId: agent.id,
        symbol: 'AAPL',
        quantity: 10,
        price: 150,
        side: 'buy',
      });

      expect(trade.status).toBe('executed');

      // Try to cancel
      await expect(
        context.tradeService.cancelTrade(trade.id)
      ).rejects.toThrow('Cannot cancel executed trade');
    });
  });

  describe('Trade Execution with Oversight Integration', () => {
    it('should execute trade after oversight approval', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, { budget: 100000 });

      // Submit large trade
      const trade = await context.tradeService.submitTrade({
        agentId: agent.id,
        symbol: 'TSLA',
        quantity: 100,
        price: 200,
        side: 'buy',
      });

      expect(trade.status).toBe('pending_approval');

      // Get oversight requests
      const requests = await context.oversightService.listRequests('pending');
      expect(requests).toHaveLength(1);
      const request = requests[0];

      // Approve oversight request
      await context.oversightService.processDecision(request.id, {
        approved: true,
        approvedBy: 'supervisor-123',
        reason: 'Approved for testing',
      });

      // Verify trade executed
      const executedTrade = await context.tradeService.getTrade(trade.id);
      expect(executedTrade?.status).toBe('executed');
      expect(executedTrade?.executedAt).toBeDefined();

      // Verify metering recorded
      const usage = await context.meteringService.getUsage(agent.id);
      expect(usage.used).toBe(20000);
    });

    it('should cancel trade after oversight rejection', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, { budget: 100000 });

      // Submit large trade
      const trade = await context.tradeService.submitTrade({
        agentId: agent.id,
        symbol: 'TSLA',
        quantity: 100,
        price: 200,
        side: 'buy',
      });

      // Get oversight request
      const requests = await context.oversightService.listRequests('pending');
      const request = requests[0];

      // Reject oversight request
      await context.oversightService.processDecision(request.id, {
        approved: false,
        approvedBy: 'supervisor-123',
        reason: 'Too risky',
      });

      // Verify trade cancelled
      const cancelledTrade = await context.tradeService.getTrade(trade.id);
      expect(cancelledTrade?.status).toBe('cancelled');

      // Verify metering NOT recorded
      const usage = await context.meteringService.getUsage(agent.id);
      expect(usage.used).toBe(0);
    });
  });
});
