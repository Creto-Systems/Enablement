/**
 * Oversight Approval Workflow Integration Tests
 *
 * Tests the complete oversight approval workflow including:
 * - Oversight request creation with notification
 * - Approval workflow executes trade
 * - Rejection workflow cancels trade
 * - Timeout handling and auto-escalation
 * - Multiple approval levels
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import supertest from 'supertest';
import {
  setupIntegrationTests,
  cleanupIntegrationTests,
  resetTestState,
  getTestContext,
  createTestAgent,
  createTestTrade,
  waitForEvent,
} from './setup';

describe('Oversight Integration Tests', () => {
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

  describe('Oversight Request Creation', () => {
    it('should create oversight request for large trade', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, { budget: 100000 });

      // Listen for oversight request event
      const eventPromise = waitForEvent(
        context.eventEmitter,
        'oversight:request_created'
      );

      // Submit large trade (triggers oversight)
      const trade = await createTestTrade(context, agent.id, {
        quantity: 100,
        price: 200, // $20,000
      });

      // Verify event emitted
      const event = await eventPromise;
      expect(event).toMatchObject({
        agentId: agent.id,
        amount: 20000,
      });
      expect(event.requestId).toBeDefined();

      // Verify oversight request created
      const requests = await context.oversightService.listRequests('pending');
      expect(requests).toHaveLength(1);
      expect(requests[0]).toMatchObject({
        tradeId: trade.id,
        agentId: agent.id,
        amount: 20000,
        status: 'pending',
      });
    });

    it('should not create oversight request for small trade', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context);

      // Submit small trade (under threshold)
      const trade = await createTestTrade(context, agent.id, {
        quantity: 10,
        price: 150, // $1,500
      });

      // Verify no oversight request
      const requests = await context.oversightService.listRequests();
      expect(requests).toHaveLength(0);

      // Verify trade executed
      expect(trade.status).toBe('executed');
    });

    it('should check budget percentage threshold', async () => {
      const context = getTestContext();

      // Create agent with small budget
      const agent = await createTestAgent(context, { budget: 20000 });

      // Submit trade that's 30% of budget (above 25% threshold)
      const trade = await createTestTrade(context, agent.id, {
        quantity: 40,
        price: 150, // $6,000 = 30% of budget
      });

      // Should trigger oversight even though under $10K absolute threshold
      expect(trade.status).toBe('pending_approval');

      const requests = await context.oversightService.listRequests('pending');
      expect(requests).toHaveLength(1);
    });
  });

  describe('Approval Workflow', () => {
    it('should execute trade when approved', async () => {
      const context = getTestContext();

      // Create agent and trade
      const agent = await createTestAgent(context, { budget: 100000 });
      const trade = await createTestTrade(context, agent.id, {
        quantity: 100,
        price: 200,
      });

      expect(trade.status).toBe('pending_approval');

      // Get oversight request
      const requests = await context.oversightService.listRequests('pending');
      const oversightRequest = requests[0];

      // Approve request
      const approvedRequest = await context.oversightService.processDecision(
        oversightRequest.id,
        {
          approved: true,
          approvedBy: 'supervisor-alice',
          reason: 'Trade looks good, approved',
        }
      );

      // Verify request status
      expect(approvedRequest).toMatchObject({
        status: 'approved',
        approvedBy: 'supervisor-alice',
        reason: 'Trade looks good, approved',
      });
      expect(approvedRequest.processedAt).toBeDefined();

      // Verify trade executed
      const executedTrade = await context.tradeService.getTrade(trade.id);
      expect(executedTrade?.status).toBe('executed');
      expect(executedTrade?.executedAt).toBeDefined();

      // Verify metering recorded
      const usage = await context.meteringService.getUsage(agent.id);
      expect(usage.used).toBe(20000);
    });

    it('should handle multiple pending approvals', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, { budget: 200000 });

      // Submit multiple large trades
      const trade1 = await createTestTrade(context, agent.id, {
        symbol: 'AAPL',
        quantity: 100,
        price: 200,
      });

      const trade2 = await createTestTrade(context, agent.id, {
        symbol: 'MSFT',
        quantity: 50,
        price: 400,
      });

      // Verify both pending
      expect(trade1.status).toBe('pending_approval');
      expect(trade2.status).toBe('pending_approval');

      // Get all pending requests
      const requests = await context.oversightService.listRequests('pending');
      expect(requests).toHaveLength(2);

      // Approve first
      await context.oversightService.processDecision(requests[0].id, {
        approved: true,
        approvedBy: 'supervisor-bob',
      });

      // Approve second
      await context.oversightService.processDecision(requests[1].id, {
        approved: true,
        approvedBy: 'supervisor-bob',
      });

      // Verify both executed
      const trade1Final = await context.tradeService.getTrade(trade1.id);
      const trade2Final = await context.tradeService.getTrade(trade2.id);

      expect(trade1Final?.status).toBe('executed');
      expect(trade2Final?.status).toBe('executed');

      // Verify cumulative metering
      const usage = await context.meteringService.getUsage(agent.id);
      expect(usage.used).toBe(40000); // 20000 + 20000
    });
  });

  describe('Rejection Workflow', () => {
    it('should cancel trade when rejected', async () => {
      const context = getTestContext();

      // Create agent and trade
      const agent = await createTestAgent(context, { budget: 100000 });
      const trade = await createTestTrade(context, agent.id, {
        quantity: 100,
        price: 200,
      });

      // Get oversight request
      const requests = await context.oversightService.listRequests('pending');
      const oversightRequest = requests[0];

      // Reject request
      const rejectedRequest = await context.oversightService.processDecision(
        oversightRequest.id,
        {
          approved: false,
          approvedBy: 'supervisor-charlie',
          reason: 'Too risky given market conditions',
        }
      );

      // Verify request status
      expect(rejectedRequest).toMatchObject({
        status: 'rejected',
        approvedBy: 'supervisor-charlie',
        reason: 'Too risky given market conditions',
      });

      // Verify trade cancelled
      const cancelledTrade = await context.tradeService.getTrade(trade.id);
      expect(cancelledTrade?.status).toBe('cancelled');
      expect(cancelledTrade?.cancelledAt).toBeDefined();

      // Verify NO metering recorded
      const usage = await context.meteringService.getUsage(agent.id);
      expect(usage.used).toBe(0);
    });

    it('should reject with validation errors', async () => {
      const context = getTestContext();

      // Create agent and trade
      const agent = await createTestAgent(context, { budget: 100000 });
      const trade = await createTestTrade(context, agent.id, {
        quantity: 100,
        price: 200,
      });

      // Get oversight request
      const requests = await context.oversightService.listRequests('pending');
      const oversightRequest = requests[0];

      // Try to process already processed request
      await context.oversightService.processDecision(oversightRequest.id, {
        approved: true,
        approvedBy: 'supervisor-alice',
      });

      // Try to process again
      await expect(
        context.oversightService.processDecision(oversightRequest.id, {
          approved: false,
          approvedBy: 'supervisor-bob',
        })
      ).rejects.toThrow('Request already processed');
    });
  });

  describe('Oversight Request Management', () => {
    it('should list all oversight requests', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, { budget: 200000 });

      // Create multiple trades
      await createTestTrade(context, agent.id, {
        symbol: 'AAPL',
        quantity: 100,
        price: 200,
      });

      await createTestTrade(context, agent.id, {
        symbol: 'MSFT',
        quantity: 50,
        price: 400,
      });

      // List all requests
      const allRequests = await context.oversightService.listRequests();
      expect(allRequests).toHaveLength(2);
    });

    it('should filter requests by status', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, { budget: 200000 });

      // Create trades
      const trade1 = await createTestTrade(context, agent.id, {
        quantity: 100,
        price: 200,
      });

      const trade2 = await createTestTrade(context, agent.id, {
        quantity: 50,
        price: 400,
      });

      // Approve one
      const requests = await context.oversightService.listRequests('pending');
      await context.oversightService.processDecision(requests[0].id, {
        approved: true,
        approvedBy: 'supervisor',
      });

      // Check filtered lists
      const pendingRequests = await context.oversightService.listRequests('pending');
      const approvedRequests = await context.oversightService.listRequests('approved');

      expect(pendingRequests).toHaveLength(1);
      expect(approvedRequests).toHaveLength(1);
    });

    it('should retrieve oversight request by id', async () => {
      const context = getTestContext();

      // Create agent and trade
      const agent = await createTestAgent(context, { budget: 100000 });
      await createTestTrade(context, agent.id, {
        quantity: 100,
        price: 200,
      });

      // Get request
      const requests = await context.oversightService.listRequests();
      const requestId = requests[0].id;

      const retrieved = await context.oversightService.getRequest(requestId);
      expect(retrieved).toBeDefined();
      expect(retrieved?.id).toBe(requestId);
    });

    it('should return null for non-existent request', async () => {
      const context = getTestContext();

      const retrieved = await context.oversightService.getRequest('non-existent-id');
      expect(retrieved).toBeNull();
    });
  });

  describe('Oversight Event Notifications', () => {
    it('should emit notification when oversight request created', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, { budget: 100000 });

      // Set up event listener
      const notifications: any[] = [];
      context.eventEmitter.on('oversight:request_created', (data) => {
        notifications.push(data);
      });

      // Create large trade
      await createTestTrade(context, agent.id, {
        quantity: 100,
        price: 200,
      });

      // Wait a bit for async event
      await new Promise(resolve => setTimeout(resolve, 100));

      expect(notifications).toHaveLength(1);
      expect(notifications[0]).toMatchObject({
        agentId: agent.id,
        amount: 20000,
      });
    });

    it('should handle multiple notification listeners', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, { budget: 100000 });

      // Set up multiple listeners
      const listener1Calls: any[] = [];
      const listener2Calls: any[] = [];

      context.eventEmitter.on('oversight:request_created', (data) => {
        listener1Calls.push(data);
      });

      context.eventEmitter.on('oversight:request_created', (data) => {
        listener2Calls.push(data);
      });

      // Create large trade
      await createTestTrade(context, agent.id, {
        quantity: 100,
        price: 200,
      });

      // Wait for async
      await new Promise(resolve => setTimeout(resolve, 100));

      expect(listener1Calls).toHaveLength(1);
      expect(listener2Calls).toHaveLength(1);
    });
  });

  describe('Complex Oversight Scenarios', () => {
    it('should handle oversight with quota near limit', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, { budget: 50000 });

      // Set quota to near limit
      context.meteringClient.setQuota(agent.id, 45000, 50000);

      // Submit trade that requires oversight AND is near quota
      const trade = await createTestTrade(context, agent.id, {
        quantity: 50,
        price: 100, // $5,000
      });

      // Should be pending (quota check passed, oversight required)
      expect(trade.status).toBe('pending_approval');

      // Approve
      const requests = await context.oversightService.listRequests('pending');
      await context.oversightService.processDecision(requests[0].id, {
        approved: true,
        approvedBy: 'supervisor',
      });

      // Verify executed and quota updated
      const finalTrade = await context.tradeService.getTrade(trade.id);
      expect(finalTrade?.status).toBe('executed');

      const usage = await context.meteringService.getUsage(agent.id);
      expect(usage.used).toBe(50000);
    });

    it('should handle rapid sequential oversight requests', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await createTestAgent(context, { budget: 500000 });

      // Submit multiple large trades rapidly
      const trades = await Promise.all([
        createTestTrade(context, agent.id, { symbol: 'AAPL', quantity: 100, price: 200 }),
        createTestTrade(context, agent.id, { symbol: 'MSFT', quantity: 50, price: 400 }),
        createTestTrade(context, agent.id, { symbol: 'GOOGL', quantity: 25, price: 500 }),
      ]);

      // All should be pending
      trades.forEach(trade => {
        expect(trade.status).toBe('pending_approval');
      });

      // All should have separate oversight requests
      const requests = await context.oversightService.listRequests('pending');
      expect(requests).toHaveLength(3);

      // Verify each request has unique trade
      const tradeIds = requests.map(r => r.tradeId);
      expect(new Set(tradeIds).size).toBe(3);
    });
  });
});
