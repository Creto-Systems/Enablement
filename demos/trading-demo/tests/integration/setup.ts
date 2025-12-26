/**
 * Integration Test Setup
 *
 * Provides test infrastructure for full-stack integration tests:
 * - In-memory SQLite database
 * - Mock gRPC servers for metering/oversight
 * - Express app with all middleware
 * - WebSocket test client
 * - Cleanup hooks
 */

import { beforeAll, afterAll, beforeEach, afterEach } from 'vitest';
import express, { Express } from 'express';
import { Server } from 'http';
import { EventEmitter } from 'events';
import { AgentService } from '@server/services/agent.service';
import { TradeService } from '@server/services/trade.service';
import { MeteringService } from '@server/services/metering.service';
import { OversightService } from '@server/services/oversight.service';
import { AgentController } from '@server/controllers/agent.controller';
import { TradeController } from '@server/controllers/trade.controller';

// Mock gRPC client for metering
export class MockMeteringGrpcClient {
  private quotas = new Map<string, { used: number; limit: number }>();

  checkQuota(
    request: { agentId: string; amount: number },
    callback: (error: Error | null, response?: any) => void
  ) {
    const quota = this.quotas.get(request.agentId) || { used: 0, limit: 100000 };
    const newUsed = quota.used + request.amount;
    const allowed = newUsed <= quota.limit;

    callback(null, {
      allowed,
      remaining: allowed ? quota.limit - newUsed : 0,
      limit: quota.limit,
      reason: allowed ? undefined : 'Quota exceeded',
    });

    if (allowed) {
      quota.used = newUsed;
      this.quotas.set(request.agentId, quota);
    }
  }

  recordEvent(
    request: {
      agentId: string;
      eventType: string;
      amount: number;
      metadata: string;
      timestamp: string;
    },
    callback: (error: Error | null, response?: any) => void
  ) {
    // Initialize quota if needed
    if (!this.quotas.has(request.agentId)) {
      this.quotas.set(request.agentId, { used: 0, limit: 100000 });
    }

    // For agent_created event, set initial quota
    if (request.eventType === 'agent_created') {
      const metadata = JSON.parse(request.metadata);
      this.quotas.set(request.agentId, {
        used: 0,
        limit: metadata.budget || 100000
      });
    }

    callback(null, { success: true });
  }

  getUsage(
    request: { agentId: string },
    callback: (error: Error | null, response?: any) => void
  ) {
    const quota = this.quotas.get(request.agentId) || { used: 0, limit: 100000 };
    callback(null, quota);
  }

  reset() {
    this.quotas.clear();
  }

  // Helper for testing
  setQuota(agentId: string, used: number, limit: number) {
    this.quotas.set(agentId, { used, limit });
  }
}

// Test context shared across integration tests
export interface IntegrationTestContext {
  app: Express;
  server: Server;
  eventEmitter: EventEmitter;
  meteringClient: MockMeteringGrpcClient;
  meteringService: MeteringService;
  oversightService: OversightService;
  agentService: AgentService;
  tradeService: TradeService;
  agentController: AgentController;
  tradeController: TradeController;
  baseUrl: string;
  authToken: string;
}

let testContext: IntegrationTestContext | null = null;

/**
 * Setup integration test environment
 * Called before all tests in a suite
 */
export async function setupIntegrationTests(): Promise<IntegrationTestContext> {
  // Create event emitter for coordination
  const eventEmitter = new EventEmitter();

  // Create mock gRPC client
  const meteringClient = new MockMeteringGrpcClient();

  // Create services
  const meteringService = new MeteringService(meteringClient, eventEmitter);

  const tradeService = new TradeService(meteringService, null as any); // Will set oversight service after creation
  const oversightService = new OversightService(tradeService, eventEmitter);

  // Set oversight service on trade service
  (tradeService as any).oversightService = oversightService;

  const agentService = new AgentService(meteringService);

  // Create controllers
  const agentController = new AgentController({
    createAgent: agentService.createAgent.bind(agentService),
    getAgent: agentService.getAgent.bind(agentService),
    terminateAgent: agentService.terminateAgent.bind(agentService),
  });

  // Create mock trade service interface for controller
  const mockTradeService = {
    submitTrade: async (agentId: string, dto: any) => {
      const trade = await tradeService.submitTrade({
        agentId,
        ...dto,
      });
      return {
        trade,
        requiresOversight: trade.status === 'pending_approval',
        reason: trade.status === 'pending_approval' ? 'Large trade requires approval' : undefined,
      };
    },
    getTrades: async (agentId: string, options: any) => {
      const trades = await tradeService.listTrades(agentId);
      return {
        trades,
        total: trades.length,
        page: options.page || 1,
        limit: options.limit || 20,
      };
    },
  };

  const tradeController = new TradeController(mockTradeService as any);

  // Create Express app
  const app = express();
  app.use(express.json());

  // Setup routes
  app.post('/api/agents', agentController.createAgent);
  app.get('/api/agents/:id', agentController.getAgent);
  app.delete('/api/agents/:id', agentController.terminateAgent);

  app.post('/api/trades', tradeController.submitTrade);
  app.get('/api/agents/:id/trades', tradeController.getTrades);

  // Error handler
  app.use((err: any, req: any, res: any, next: any) => {
    const statusCode = err.statusCode || 500;
    res.status(statusCode).json({
      error: err.message,
      statusCode,
      errors: err.errors,
    });
  });

  // Start server on random port
  const server = await new Promise<Server>((resolve) => {
    const srv = app.listen(0, () => resolve(srv));
  });

  const port = (server.address() as any).port;
  const baseUrl = `http://localhost:${port}`;
  const authToken = 'Bearer test-token-12345';

  testContext = {
    app,
    server,
    eventEmitter,
    meteringClient,
    meteringService,
    oversightService,
    agentService,
    tradeService,
    agentController,
    tradeController,
    baseUrl,
    authToken,
  };

  return testContext;
}

/**
 * Cleanup integration test environment
 * Called after all tests in a suite
 */
export async function cleanupIntegrationTests() {
  if (testContext?.server) {
    await new Promise<void>((resolve) => {
      testContext!.server.close(() => resolve());
    });
  }

  // Clean up event listeners
  testContext?.eventEmitter.removeAllListeners();

  testContext = null;
}

/**
 * Reset state between tests
 * Called before each test
 */
export async function resetTestState() {
  if (!testContext) return;

  // Reset mock gRPC client
  testContext.meteringClient.reset();

  // Clear service state (in-memory maps)
  (testContext.agentService as any).agents.clear();
  (testContext.tradeService as any).trades.clear();
  (testContext.oversightService as any).requests.clear();

  // Clear event listeners (except internal service listeners)
  const eventEmitter = testContext.eventEmitter;
  const eventNames = eventEmitter.eventNames();
  eventNames.forEach((eventName) => {
    if (!eventName.toString().startsWith('oversight:') &&
        !eventName.toString().startsWith('quota:')) {
      eventEmitter.removeAllListeners(eventName);
    }
  });
}

/**
 * Get current test context
 * Throws if setup hasn't been called
 */
export function getTestContext(): IntegrationTestContext {
  if (!testContext) {
    throw new Error('Test context not initialized. Call setupIntegrationTests first.');
  }
  return testContext;
}

/**
 * Helper to wait for event
 */
export function waitForEvent(
  eventEmitter: EventEmitter,
  eventName: string,
  timeout = 5000
): Promise<any> {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`Timeout waiting for event: ${eventName}`));
    }, timeout);

    eventEmitter.once(eventName, (data) => {
      clearTimeout(timer);
      resolve(data);
    });
  });
}

/**
 * Helper to create test agent
 */
export async function createTestAgent(context: IntegrationTestContext, config?: {
  name?: string;
  strategy?: 'conservative' | 'balanced' | 'aggressive';
  budget?: number;
  riskLevel?: number;
}) {
  return context.agentService.createAgent({
    name: config?.name || 'Test Agent',
    strategy: config?.strategy || 'balanced',
    budget: config?.budget || 50000,
    riskLevel: config?.riskLevel || 5,
  });
}

/**
 * Helper to create test trade
 */
export async function createTestTrade(
  context: IntegrationTestContext,
  agentId: string,
  config?: {
    symbol?: string;
    quantity?: number;
    price?: number;
    side?: 'buy' | 'sell';
  }
) {
  return context.tradeService.submitTrade({
    agentId,
    symbol: config?.symbol || 'AAPL',
    quantity: config?.quantity || 100,
    price: config?.price || 150,
    side: config?.side || 'buy',
  });
}
