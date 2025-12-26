import { test as base, Page } from '@playwright/test';
import { Server } from 'ws';

/**
 * Test Fixtures for Trading Demo E2E Tests
 *
 * Provides shared setup and utilities:
 * - Authenticated user fixture
 * - Pre-created agent fixture
 * - Mock WebSocket server
 * - API request interceptors
 */

export interface Agent {
  id: string;
  name: string;
  budget: number;
  strategy: 'conservative' | 'moderate' | 'aggressive';
  riskTolerance: 'low' | 'medium' | 'high';
  status: 'active' | 'paused' | 'stopped';
  createdAt: string;
}

export interface Trade {
  id: string;
  agentId: string;
  symbol: string;
  type: 'BUY' | 'SELL';
  quantity: number;
  price: number;
  total: number;
  status: 'pending' | 'approved' | 'executed' | 'rejected';
  timestamp: string;
  requiresApproval?: boolean;
}

interface MockAPIResponses {
  agents?: Agent[];
  trades?: Trade[];
  portfolio?: {
    cash: number;
    positions: Array<{
      symbol: string;
      quantity: number;
      averagePrice: number;
      currentPrice: number;
    }>;
  };
  metering?: {
    agentId: string;
    quota: {
      total: number;
      used: number;
      remaining: number;
    };
    usage: Array<{
      timestamp: string;
      action: string;
      cost: number;
    }>;
  };
}

export interface TestFixtures {
  authenticatedPage: Page;
  agentPage: Page;
  mockWebSocket: Server;
  mockAPI: (responses?: MockAPIResponses) => Promise<void>;
}

/**
 * Setup authenticated user session
 */
async function setupAuthentication(page: Page): Promise<void> {
  // Set authentication cookies/tokens
  await page.context().addCookies([
    {
      name: 'auth_token',
      value: 'test-auth-token-12345',
      domain: 'localhost',
      path: '/',
      httpOnly: true,
      secure: false,
      sameSite: 'Lax',
    },
  ]);

  // Set localStorage auth data
  await page.addInitScript(() => {
    localStorage.setItem('user', JSON.stringify({
      id: 'test-user-1',
      email: 'test@example.com',
      name: 'Test User',
      role: 'trader',
    }));
  });
}

/**
 * Create a pre-configured agent for testing
 */
async function createTestAgent(page: Page): Promise<Agent> {
  const agent: Agent = {
    id: 'agent-test-123',
    name: 'Test Trading Agent',
    budget: 10000,
    strategy: 'moderate',
    riskTolerance: 'medium',
    status: 'active',
    createdAt: new Date().toISOString(),
  };

  // Store in localStorage for quick access
  await page.addInitScript((agentData) => {
    const existingAgents = JSON.parse(localStorage.getItem('agents') || '[]');
    existingAgents.push(agentData);
    localStorage.setItem('agents', JSON.stringify(existingAgents));
  }, agent);

  return agent;
}

/**
 * Setup Mock WebSocket Server
 */
function createMockWebSocketServer(): Server {
  const wss = new Server({ port: 8080 });

  wss.on('connection', (ws) => {
    console.log('[Mock WS] Client connected');

    // Send initial connection message
    ws.send(JSON.stringify({
      type: 'connection',
      status: 'connected',
      timestamp: new Date().toISOString(),
    }));

    // Handle messages from client
    ws.on('message', (data) => {
      const message = JSON.parse(data.toString());
      console.log('[Mock WS] Received:', message);

      // Echo back for testing
      if (message.type === 'subscribe') {
        ws.send(JSON.stringify({
          type: 'subscribed',
          channel: message.channel,
          timestamp: new Date().toISOString(),
        }));
      }
    });
  });

  return wss;
}

/**
 * Setup API request interceptors
 */
async function setupMockAPI(page: Page, responses: MockAPIResponses = {}): Promise<void> {
  // Mock GET /api/agents
  await page.route('**/api/agents', async (route) => {
    if (route.request().method() === 'GET') {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(responses.agents || [
          {
            id: 'agent-1',
            name: 'Conservative Trader',
            budget: 5000,
            strategy: 'conservative',
            riskTolerance: 'low',
            status: 'active',
            createdAt: new Date().toISOString(),
          },
        ]),
      });
    }
  });

  // Mock POST /api/agents
  await page.route('**/api/agents', async (route) => {
    if (route.request().method() === 'POST') {
      const postData = route.request().postDataJSON();
      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({
          id: `agent-${Date.now()}`,
          ...postData,
          status: 'active',
          createdAt: new Date().toISOString(),
        }),
      });
    }
  });

  // Mock GET /api/trades
  await page.route('**/api/trades*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(responses.trades || []),
    });
  });

  // Mock POST /api/trades
  await page.route('**/api/trades', async (route) => {
    if (route.request().method() === 'POST') {
      const postData = route.request().postDataJSON();
      const requiresApproval = postData.total > 50000;

      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({
          id: `trade-${Date.now()}`,
          ...postData,
          status: requiresApproval ? 'pending' : 'executed',
          requiresApproval,
          timestamp: new Date().toISOString(),
        }),
      });
    }
  });

  // Mock GET /api/portfolio
  await page.route('**/api/portfolio*', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(responses.portfolio || {
        cash: 100000,
        positions: [],
      }),
    });
  });

  // Mock GET /api/metering
  await page.route('**/api/metering*', async (route) => {
    const url = new URL(route.request().url());
    const agentId = url.searchParams.get('agentId') || 'agent-1';

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(responses.metering || {
        agentId,
        quota: {
          total: 10000,
          used: 2500,
          remaining: 7500,
        },
        usage: [
          {
            timestamp: new Date().toISOString(),
            action: 'trade_execution',
            cost: 100,
          },
        ],
      }),
    });
  });

  // Mock POST /api/oversight/approve
  await page.route('**/api/oversight/approve', async (route) => {
    if (route.request().method() === 'POST') {
      const postData = route.request().postDataJSON();
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          tradeId: postData.tradeId,
          status: 'approved',
          approvedBy: 'test-user-1',
          approvedAt: new Date().toISOString(),
        }),
      });
    }
  });

  // Mock POST /api/oversight/reject
  await page.route('**/api/oversight/reject', async (route) => {
    if (route.request().method() === 'POST') {
      const postData = route.request().postDataJSON();
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          tradeId: postData.tradeId,
          status: 'rejected',
          rejectedBy: 'test-user-1',
          rejectedAt: new Date().toISOString(),
          reason: postData.reason || 'Manual rejection',
        }),
      });
    }
  });
}

/**
 * Extended test with fixtures
 */
export const test = base.extend<TestFixtures>({
  authenticatedPage: async ({ page }, use) => {
    await setupAuthentication(page);
    await use(page);
  },

  agentPage: async ({ page }, use) => {
    await setupAuthentication(page);
    await createTestAgent(page);
    await use(page);
  },

  mockWebSocket: async ({}, use) => {
    const wss = createMockWebSocketServer();
    await use(wss);
    wss.close();
  },

  mockAPI: async ({ page }, use) => {
    const setupMock = async (responses?: MockAPIResponses) => {
      await setupMockAPI(page, responses);
    };
    await use(setupMock);
  },
});

export { expect } from '@playwright/test';
