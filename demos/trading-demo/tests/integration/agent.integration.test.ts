/**
 * Agent Lifecycle Integration Tests
 *
 * Tests the complete agent lifecycle including:
 * - Agent creation with quota initialization in metering
 * - Agent retrieval with portfolio data
 * - Agent listing with pagination
 * - Agent termination with cleanup
 */

import { describe, it, expect, beforeAll, afterAll, beforeEach } from 'vitest';
import supertest from 'supertest';
import {
  setupIntegrationTests,
  cleanupIntegrationTests,
  resetTestState,
  getTestContext,
  waitForEvent,
} from './setup';

describe('Agent Integration Tests', () => {
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

  describe('POST /api/agents - Create Agent', () => {
    it('should create agent and initialize quota in metering', async () => {
      const context = getTestContext();

      const response = await request
        .post('/api/agents')
        .set('Authorization', context.authToken)
        .send({
          name: 'Integration Test Agent',
          type: 'trading',
          userId: 'user-123',
          config: {
            strategy: 'balanced',
            budget: 50000,
            riskLevel: 5,
          },
        })
        .expect(201);

      // Verify agent created
      expect(response.body).toMatchObject({
        name: 'Integration Test Agent',
        strategy: 'balanced',
        budget: 50000,
        riskLevel: 5,
        status: 'active',
      });
      expect(response.body.id).toBeDefined();
      expect(response.body.createdAt).toBeDefined();

      // Verify quota initialized in metering
      const usage = await context.meteringService.getUsage(response.body.id);
      expect(usage).toEqual({
        used: 0,
        limit: 50000,
      });
    });

    it('should reject agent creation without authorization', async () => {
      await request
        .post('/api/agents')
        .send({
          name: 'Unauthorized Agent',
          type: 'trading',
          userId: 'user-123',
          config: {
            strategy: 'balanced',
            budget: 50000,
            riskLevel: 5,
          },
        })
        .expect(401);
    });

    it('should reject agent with invalid budget', async () => {
      const context = getTestContext();

      const response = await request
        .post('/api/agents')
        .set('Authorization', context.authToken)
        .send({
          name: 'Invalid Budget Agent',
          type: 'trading',
          userId: 'user-123',
          config: {
            strategy: 'balanced',
            budget: 500, // Below minimum
            riskLevel: 5,
          },
        })
        .expect(500); // Service will throw error

      expect(response.body.error).toContain('Budget must be between');
    });

    it('should create multiple agents independently', async () => {
      const context = getTestContext();

      const agent1Response = await request
        .post('/api/agents')
        .set('Authorization', context.authToken)
        .send({
          name: 'Agent 1',
          type: 'trading',
          userId: 'user-123',
          config: {
            strategy: 'conservative',
            budget: 30000,
            riskLevel: 3,
          },
        })
        .expect(201);

      const agent2Response = await request
        .post('/api/agents')
        .set('Authorization', context.authToken)
        .send({
          name: 'Agent 2',
          type: 'trading',
          userId: 'user-456',
          config: {
            strategy: 'aggressive',
            budget: 100000,
            riskLevel: 8,
          },
        })
        .expect(201);

      expect(agent1Response.body.id).not.toBe(agent2Response.body.id);

      // Verify separate quotas
      const usage1 = await context.meteringService.getUsage(agent1Response.body.id);
      const usage2 = await context.meteringService.getUsage(agent2Response.body.id);

      expect(usage1.limit).toBe(30000);
      expect(usage2.limit).toBe(100000);
    });
  });

  describe('GET /api/agents/:id - Get Agent', () => {
    it('should retrieve agent by id', async () => {
      const context = getTestContext();

      // Create agent
      const createResponse = await request
        .post('/api/agents')
        .set('Authorization', context.authToken)
        .send({
          name: 'Retrievable Agent',
          type: 'trading',
          userId: 'user-123',
          config: {
            strategy: 'balanced',
            budget: 50000,
            riskLevel: 5,
          },
        })
        .expect(201);

      const agentId = createResponse.body.id;

      // Retrieve agent
      const getResponse = await request
        .get(`/api/agents/${agentId}`)
        .set('Authorization', context.authToken)
        .expect(200);

      expect(getResponse.body).toMatchObject({
        id: agentId,
        name: 'Retrievable Agent',
        strategy: 'balanced',
        budget: 50000,
        status: 'active',
      });
    });

    it('should return 404 for non-existent agent', async () => {
      const context = getTestContext();

      await request
        .get('/api/agents/non-existent-id')
        .set('Authorization', context.authToken)
        .expect(404);
    });

    it('should require authorization', async () => {
      await request
        .get('/api/agents/some-id')
        .expect(401);
    });
  });

  describe('DELETE /api/agents/:id - Terminate Agent', () => {
    it('should terminate agent successfully', async () => {
      const context = getTestContext();

      // Create agent
      const createResponse = await request
        .post('/api/agents')
        .set('Authorization', context.authToken)
        .send({
          name: 'Terminable Agent',
          type: 'trading',
          userId: 'user-123',
          config: {
            strategy: 'balanced',
            budget: 50000,
            riskLevel: 5,
          },
        })
        .expect(201);

      const agentId = createResponse.body.id;

      // Terminate agent
      await request
        .delete(`/api/agents/${agentId}`)
        .set('Authorization', context.authToken)
        .expect(204);

      // Verify agent is terminated
      const getResponse = await request
        .get(`/api/agents/${agentId}`)
        .set('Authorization', context.authToken)
        .expect(200);

      expect(getResponse.body.status).toBe('terminated');
      expect(getResponse.body.terminatedAt).toBeDefined();
    });

    it('should return error for non-existent agent', async () => {
      const context = getTestContext();

      await request
        .delete('/api/agents/non-existent-id')
        .set('Authorization', context.authToken)
        .expect(500);
    });
  });

  describe('Agent Lifecycle with Metering Integration', () => {
    it('should track quota throughout agent lifecycle', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await context.agentService.createAgent({
        name: 'Quota Tracking Agent',
        strategy: 'balanced',
        budget: 50000,
        riskLevel: 5,
      });

      // Initial quota
      let usage = await context.meteringService.getUsage(agent.id);
      expect(usage).toEqual({ used: 0, limit: 50000 });

      // Simulate some usage
      await context.meteringService.recordEvent({
        agentId: agent.id,
        eventType: 'trade_executed',
        amount: 10000,
        metadata: { symbol: 'AAPL' },
      });

      // Check quota used
      const quotaCheck = await context.meteringService.checkQuota(agent.id, 10000);
      expect(quotaCheck.allowed).toBe(true);
      expect(quotaCheck.remaining).toBe(30000);

      // Terminate agent
      await context.agentService.terminateAgent(agent.id);

      // Quota should still exist
      usage = await context.meteringService.getUsage(agent.id);
      expect(usage.used).toBe(10000);
    });

    it('should emit quota warning when threshold reached', async () => {
      const context = getTestContext();

      // Create agent
      const agent = await context.agentService.createAgent({
        name: 'Quota Warning Agent',
        strategy: 'balanced',
        budget: 10000,
        riskLevel: 5,
      });

      // Set quota to 85% used (above 80% warning threshold)
      context.meteringClient.setQuota(agent.id, 8500, 10000);

      // Listen for warning event
      const warningPromise = waitForEvent(context.eventEmitter, 'quota:warning');

      // Check quota (should trigger warning)
      await context.meteringService.checkQuota(agent.id, 1000);

      const warning = await warningPromise;
      expect(warning).toMatchObject({
        agentId: agent.id,
        used: 8500,
        limit: 10000,
        percentage: 85,
      });
    });
  });
});
