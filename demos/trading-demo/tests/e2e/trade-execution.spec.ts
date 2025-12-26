import { test, expect } from './fixtures/test-fixtures';

/**
 * Trade Execution E2E Tests
 *
 * Tests the complete trading workflow:
 * - Agent selection from dashboard
 * - Trade modal interactions
 * - Buy/Sell order submission
 * - Trade history verification
 * - Real-time portfolio updates via WebSocket
 */

test.describe('Trade Execution', () => {
  test.beforeEach(async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      agents: [
        {
          id: 'agent-test-1',
          name: 'Test Trading Agent',
          budget: 50000,
          strategy: 'moderate',
          riskTolerance: 'medium',
          status: 'active',
          createdAt: new Date().toISOString(),
        },
      ],
      portfolio: {
        cash: 50000,
        positions: [],
      },
    });

    await authenticatedPage.goto('/dashboard');
  });

  test('should select agent and open trade modal', async ({ authenticatedPage }) => {
    // Click on agent card
    await authenticatedPage.click('[data-testid="agent-card"]:has-text("Test Trading Agent")');

    // Verify navigation to agent detail page
    await expect(authenticatedPage).toHaveURL(/\/agents\/agent-test-1/);
    await expect(authenticatedPage.locator('h1')).toContainText('Test Trading Agent');

    // Click "New Trade" button
    await authenticatedPage.click('button:has-text("New Trade")');

    // Verify trade modal opens
    await expect(authenticatedPage.locator('[data-testid="trade-modal"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="trade-modal-title"]')).toHaveText('Execute Trade');
  });

  test('should submit BUY order and verify execution', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      trades: [],
    });

    await authenticatedPage.goto('/agents/agent-test-1');
    await authenticatedPage.click('button:has-text("New Trade")');

    // Fill trade form
    await authenticatedPage.fill('[name="symbol"]', 'AAPL');
    await authenticatedPage.fill('[name="quantity"]', '100');
    await authenticatedPage.selectOption('[name="type"]', 'BUY');

    // Mock price lookup
    await authenticatedPage.route('**/api/prices/AAPL', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          symbol: 'AAPL',
          price: 175.50,
          timestamp: new Date().toISOString(),
        }),
      });
    });

    // Trigger price lookup (blur from quantity field)
    await authenticatedPage.locator('[name="quantity"]').blur();

    // Wait for price to populate
    await expect(authenticatedPage.locator('[data-testid="current-price"]')).toHaveText('$175.50');
    await expect(authenticatedPage.locator('[data-testid="total-cost"]')).toHaveText('$17,550.00');

    // Submit trade
    await authenticatedPage.click('button:has-text("Submit Trade")');

    // Verify success message
    await expect(authenticatedPage.locator('[data-testid="trade-success"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="trade-success"]')).toContainText('Trade executed successfully');

    // Verify modal closes
    await expect(authenticatedPage.locator('[data-testid="trade-modal"]')).not.toBeVisible();

    // Verify trade appears in history
    await expect(authenticatedPage.locator('[data-testid="trade-history"]')).toContainText('AAPL');
    await expect(authenticatedPage.locator('[data-testid="trade-history"]')).toContainText('BUY');
    await expect(authenticatedPage.locator('[data-testid="trade-history"]')).toContainText('100');
  });

  test('should submit SELL order for existing position', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      portfolio: {
        cash: 30000,
        positions: [
          {
            symbol: 'AAPL',
            quantity: 100,
            averagePrice: 170.00,
            currentPrice: 175.50,
          },
        ],
      },
    });

    await authenticatedPage.goto('/agents/agent-test-1');
    await authenticatedPage.click('button:has-text("New Trade")');

    // Fill sell order
    await authenticatedPage.fill('[name="symbol"]', 'AAPL');
    await authenticatedPage.fill('[name="quantity"]', '50');
    await authenticatedPage.selectOption('[name="type"]', 'SELL');

    // Mock price lookup
    await authenticatedPage.route('**/api/prices/AAPL', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          symbol: 'AAPL',
          price: 175.50,
          timestamp: new Date().toISOString(),
        }),
      });
    });

    await authenticatedPage.locator('[name="quantity"]').blur();

    // Verify sell calculation
    await expect(authenticatedPage.locator('[data-testid="total-proceeds"]')).toHaveText('$8,775.00');

    // Submit trade
    await authenticatedPage.click('button:has-text("Submit Trade")');

    // Verify success
    await expect(authenticatedPage.locator('[data-testid="trade-success"]')).toBeVisible();
  });

  test('should validate insufficient funds for BUY order', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      portfolio: {
        cash: 5000,
        positions: [],
      },
    });

    await authenticatedPage.goto('/agents/agent-test-1');
    await authenticatedPage.click('button:has-text("New Trade")');

    // Try to buy more than available cash
    await authenticatedPage.fill('[name="symbol"]', 'AAPL');
    await authenticatedPage.fill('[name="quantity"]', '100');
    await authenticatedPage.selectOption('[name="type"]', 'BUY');

    // Mock price lookup
    await authenticatedPage.route('**/api/prices/AAPL', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          symbol: 'AAPL',
          price: 175.50,
          timestamp: new Date().toISOString(),
        }),
      });
    });

    await authenticatedPage.locator('[name="quantity"]').blur();

    // Verify validation error
    await expect(authenticatedPage.locator('[data-testid="error-insufficient-funds"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="error-insufficient-funds"]')).toContainText('Insufficient funds');

    // Verify submit button is disabled
    await expect(authenticatedPage.locator('button:has-text("Submit Trade")')).toBeDisabled();
  });

  test('should validate insufficient shares for SELL order', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      portfolio: {
        cash: 50000,
        positions: [
          {
            symbol: 'AAPL',
            quantity: 50,
            averagePrice: 170.00,
            currentPrice: 175.50,
          },
        ],
      },
    });

    await authenticatedPage.goto('/agents/agent-test-1');
    await authenticatedPage.click('button:has-text("New Trade")');

    // Try to sell more than owned
    await authenticatedPage.fill('[name="symbol"]', 'AAPL');
    await authenticatedPage.fill('[name="quantity"]', '100');
    await authenticatedPage.selectOption('[name="type"]', 'SELL');

    await authenticatedPage.locator('[name="quantity"]').blur();

    // Verify validation error
    await expect(authenticatedPage.locator('[data-testid="error-insufficient-shares"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="error-insufficient-shares"]')).toContainText('Insufficient shares');

    // Verify submit button is disabled
    await expect(authenticatedPage.locator('button:has-text("Submit Trade")')).toBeDisabled();
  });

  test('should update portfolio in real-time via WebSocket', async ({ authenticatedPage, mockWebSocket, mockAPI }) => {
    await mockAPI();

    await authenticatedPage.goto('/agents/agent-test-1');

    // Initial portfolio state
    await expect(authenticatedPage.locator('[data-testid="portfolio-cash"]')).toContainText('$50,000.00');

    // Simulate WebSocket update
    mockWebSocket.clients.forEach((client) => {
      client.send(JSON.stringify({
        type: 'portfolio_update',
        agentId: 'agent-test-1',
        data: {
          cash: 32450,
          positions: [
            {
              symbol: 'AAPL',
              quantity: 100,
              averagePrice: 175.50,
              currentPrice: 176.20,
            },
          ],
        },
        timestamp: new Date().toISOString(),
      }));
    });

    // Verify portfolio updated
    await expect(authenticatedPage.locator('[data-testid="portfolio-cash"]')).toContainText('$32,450.00');
    await expect(authenticatedPage.locator('[data-testid="position-AAPL"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="position-AAPL-quantity"]')).toContainText('100');
  });

  test('should handle invalid stock symbol', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI();

    await authenticatedPage.goto('/agents/agent-test-1');
    await authenticatedPage.click('button:has-text("New Trade")');

    // Enter invalid symbol
    await authenticatedPage.fill('[name="symbol"]', 'INVALID');
    await authenticatedPage.fill('[name="quantity"]', '10');

    // Mock invalid symbol response
    await authenticatedPage.route('**/api/prices/INVALID', async (route) => {
      await route.fulfill({
        status: 404,
        contentType: 'application/json',
        body: JSON.stringify({
          error: 'Symbol not found',
        }),
      });
    });

    await authenticatedPage.locator('[name="quantity"]').blur();

    // Verify error message
    await expect(authenticatedPage.locator('[data-testid="error-symbol"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="error-symbol"]')).toContainText('Invalid symbol');
  });

  test('should display trade confirmation with all details', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI();

    await authenticatedPage.goto('/agents/agent-test-1');
    await authenticatedPage.click('button:has-text("New Trade")');

    // Fill trade
    await authenticatedPage.fill('[name="symbol"]', 'AAPL');
    await authenticatedPage.fill('[name="quantity"]', '100');
    await authenticatedPage.selectOption('[name="type"]', 'BUY');

    // Mock price
    await authenticatedPage.route('**/api/prices/AAPL', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          symbol: 'AAPL',
          price: 175.50,
          timestamp: new Date().toISOString(),
        }),
      });
    });

    await authenticatedPage.locator('[name="quantity"]').blur();

    // Click submit
    await authenticatedPage.click('button:has-text("Submit Trade")');

    // Verify confirmation dialog
    await expect(authenticatedPage.locator('[data-testid="trade-confirmation"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="confirm-symbol"]')).toHaveText('AAPL');
    await expect(authenticatedPage.locator('[data-testid="confirm-type"]')).toHaveText('BUY');
    await expect(authenticatedPage.locator('[data-testid="confirm-quantity"]')).toHaveText('100');
    await expect(authenticatedPage.locator('[data-testid="confirm-price"]')).toHaveText('$175.50');
    await expect(authenticatedPage.locator('[data-testid="confirm-total"]')).toHaveText('$17,550.00');

    // Confirm trade
    await authenticatedPage.click('button:has-text("Confirm")');

    // Verify success
    await expect(authenticatedPage.locator('[data-testid="trade-success"]')).toBeVisible();
  });

  test('should support canceling trade submission', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI();

    await authenticatedPage.goto('/agents/agent-test-1');
    await authenticatedPage.click('button:has-text("New Trade")');

    // Fill trade
    await authenticatedPage.fill('[name="symbol"]', 'AAPL');
    await authenticatedPage.fill('[name="quantity"]', '100');

    // Cancel modal
    await authenticatedPage.click('button:has-text("Cancel")');

    // Verify modal closed
    await expect(authenticatedPage.locator('[data-testid="trade-modal"]')).not.toBeVisible();

    // Verify no trade was created
    await expect(authenticatedPage.locator('[data-testid="trade-history"]')).not.toContainText('AAPL');
  });
});
