import { test, expect } from './fixtures/test-fixtures';

/**
 * Oversight Approval Workflow E2E Tests
 *
 * Tests the human-in-the-loop approval process:
 * - Large trade submission (>$50K threshold)
 * - Oversight panel navigation
 * - Pending approval view
 * - Trade approval flow
 * - Trade rejection flow
 * - Post-approval execution
 */

test.describe('Oversight Approval Workflow', () => {
  test.beforeEach(async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      agents: [
        {
          id: 'agent-large-trader',
          name: 'Large Position Agent',
          budget: 500000,
          strategy: 'aggressive',
          riskTolerance: 'high',
          status: 'active',
          createdAt: new Date().toISOString(),
        },
      ],
      portfolio: {
        cash: 500000,
        positions: [],
      },
    });

    await authenticatedPage.goto('/dashboard');
  });

  test('should require approval for large trades over $50K', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/agent-large-trader');
    await authenticatedPage.click('button:has-text("New Trade")');

    // Submit large trade
    await authenticatedPage.fill('[name="symbol"]', 'TSLA');
    await authenticatedPage.fill('[name="quantity"]', '500');
    await authenticatedPage.selectOption('[name="type"]', 'BUY');

    // Mock price lookup
    await authenticatedPage.route('**/api/prices/TSLA', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          symbol: 'TSLA',
          price: 250.00,
          timestamp: new Date().toISOString(),
        }),
      });
    });

    await authenticatedPage.locator('[name="quantity"]').blur();

    // Total should be $125,000 (over $50K threshold)
    await expect(authenticatedPage.locator('[data-testid="total-cost"]')).toHaveText('$125,000.00');

    // Verify approval required notice
    await expect(authenticatedPage.locator('[data-testid="approval-notice"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="approval-notice"]')).toContainText('This trade requires oversight approval');

    // Submit trade
    await authenticatedPage.click('button:has-text("Submit Trade")');

    // Verify pending status message
    await expect(authenticatedPage.locator('[data-testid="trade-pending"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="trade-pending"]')).toContainText('Trade submitted for approval');

    // Verify trade appears with pending status
    await expect(authenticatedPage.locator('[data-testid="trade-history"]').filter({ hasText: 'TSLA' })).toContainText('Pending Approval');
  });

  test('should navigate to oversight panel and view pending trades', async ({ authenticatedPage, mockAPI }) => {
    // Mock pending trades
    await mockAPI({
      trades: [
        {
          id: 'trade-pending-1',
          agentId: 'agent-large-trader',
          symbol: 'TSLA',
          type: 'BUY',
          quantity: 500,
          price: 250.00,
          total: 125000,
          status: 'pending',
          requiresApproval: true,
          timestamp: new Date().toISOString(),
        },
        {
          id: 'trade-pending-2',
          agentId: 'agent-large-trader',
          symbol: 'NVDA',
          type: 'BUY',
          quantity: 200,
          price: 450.00,
          total: 90000,
          status: 'pending',
          requiresApproval: true,
          timestamp: new Date().toISOString(),
        },
      ],
    });

    // Navigate to oversight panel
    await authenticatedPage.click('[data-testid="nav-oversight"]');

    // Verify navigation
    await expect(authenticatedPage).toHaveURL('/oversight');
    await expect(authenticatedPage.locator('h1')).toHaveText('Oversight Panel');

    // Verify pending trades count
    await expect(authenticatedPage.locator('[data-testid="pending-count"]')).toHaveText('2');

    // Verify trade cards displayed
    await expect(authenticatedPage.locator('[data-testid="pending-trade-trade-pending-1"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="pending-trade-trade-pending-2"]')).toBeVisible();

    // Verify trade details
    const trade1 = authenticatedPage.locator('[data-testid="pending-trade-trade-pending-1"]');
    await expect(trade1.locator('[data-testid="trade-symbol"]')).toHaveText('TSLA');
    await expect(trade1.locator('[data-testid="trade-type"]')).toHaveText('BUY');
    await expect(trade1.locator('[data-testid="trade-quantity"]')).toHaveText('500');
    await expect(trade1.locator('[data-testid="trade-total"]')).toHaveText('$125,000.00');
  });

  test('should approve pending trade and verify execution', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      trades: [
        {
          id: 'trade-approve-1',
          agentId: 'agent-large-trader',
          symbol: 'TSLA',
          type: 'BUY',
          quantity: 500,
          price: 250.00,
          total: 125000,
          status: 'pending',
          requiresApproval: true,
          timestamp: new Date().toISOString(),
        },
      ],
    });

    await authenticatedPage.goto('/oversight');

    // Click on trade to view details
    await authenticatedPage.click('[data-testid="pending-trade-trade-approve-1"]');

    // Verify detail view
    await expect(authenticatedPage.locator('[data-testid="trade-detail-modal"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="detail-symbol"]')).toHaveText('TSLA');
    await expect(authenticatedPage.locator('[data-testid="detail-agent"]')).toHaveText('Large Position Agent');

    // Verify risk assessment displayed
    await expect(authenticatedPage.locator('[data-testid="risk-assessment"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="risk-level"]')).toContainText('High');

    // Add approval comment
    await authenticatedPage.fill('[name="approvalComment"]', 'Approved based on market analysis');

    // Click Approve button
    await authenticatedPage.click('button:has-text("Approve Trade")');

    // Verify confirmation dialog
    await expect(authenticatedPage.locator('[data-testid="approve-confirmation"]')).toBeVisible();
    await authenticatedPage.click('button:has-text("Confirm Approval")');

    // Verify success message
    await expect(authenticatedPage.locator('[data-testid="approval-success"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="approval-success"]')).toContainText('Trade approved and executed');

    // Verify trade removed from pending list
    await expect(authenticatedPage.locator('[data-testid="pending-trade-trade-approve-1"]')).not.toBeVisible();

    // Verify pending count updated
    await expect(authenticatedPage.locator('[data-testid="pending-count"]')).toHaveText('0');
  });

  test('should reject pending trade with reason', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      trades: [
        {
          id: 'trade-reject-1',
          agentId: 'agent-large-trader',
          symbol: 'GME',
          type: 'BUY',
          quantity: 1000,
          price: 200.00,
          total: 200000,
          status: 'pending',
          requiresApproval: true,
          timestamp: new Date().toISOString(),
        },
      ],
    });

    await authenticatedPage.goto('/oversight');
    await authenticatedPage.click('[data-testid="pending-trade-trade-reject-1"]');

    // Click Reject button
    await authenticatedPage.click('button:has-text("Reject Trade")');

    // Verify rejection modal
    await expect(authenticatedPage.locator('[data-testid="reject-modal"]')).toBeVisible();

    // Fill rejection reason (required)
    await authenticatedPage.fill('[name="rejectionReason"]', 'Excessive exposure to volatile stock');

    // Confirm rejection
    await authenticatedPage.click('button:has-text("Confirm Rejection")');

    // Verify success message
    await expect(authenticatedPage.locator('[data-testid="rejection-success"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="rejection-success"]')).toContainText('Trade rejected');

    // Verify trade removed from pending
    await expect(authenticatedPage.locator('[data-testid="pending-trade-trade-reject-1"]')).not.toBeVisible();
  });

  test('should require rejection reason', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      trades: [
        {
          id: 'trade-reject-2',
          agentId: 'agent-large-trader',
          symbol: 'GME',
          type: 'BUY',
          quantity: 1000,
          price: 200.00,
          total: 200000,
          status: 'pending',
          requiresApproval: true,
          timestamp: new Date().toISOString(),
        },
      ],
    });

    await authenticatedPage.goto('/oversight');
    await authenticatedPage.click('[data-testid="pending-trade-trade-reject-2"]');
    await authenticatedPage.click('button:has-text("Reject Trade")');

    // Try to submit without reason
    await authenticatedPage.click('button:has-text("Confirm Rejection")');

    // Verify validation error
    await expect(authenticatedPage.locator('[data-testid="error-rejection-reason"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="error-rejection-reason"]')).toHaveText('Rejection reason is required');

    // Verify reject button disabled
    await expect(authenticatedPage.locator('button:has-text("Confirm Rejection")')).toBeDisabled();
  });

  test('should display agent context and portfolio impact', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      trades: [
        {
          id: 'trade-context-1',
          agentId: 'agent-large-trader',
          symbol: 'AAPL',
          type: 'BUY',
          quantity: 400,
          price: 175.00,
          total: 70000,
          status: 'pending',
          requiresApproval: true,
          timestamp: new Date().toISOString(),
        },
      ],
      portfolio: {
        cash: 500000,
        positions: [
          {
            symbol: 'AAPL',
            quantity: 100,
            averagePrice: 170.00,
            currentPrice: 175.00,
          },
        ],
      },
    });

    await authenticatedPage.goto('/oversight');
    await authenticatedPage.click('[data-testid="pending-trade-trade-context-1"]');

    // Verify agent context displayed
    await expect(authenticatedPage.locator('[data-testid="agent-budget"]')).toContainText('$500,000');
    await expect(authenticatedPage.locator('[data-testid="agent-strategy"]')).toContainText('Aggressive');
    await expect(authenticatedPage.locator('[data-testid="agent-risk"]')).toContainText('High');

    // Verify current portfolio
    await expect(authenticatedPage.locator('[data-testid="current-cash"]')).toContainText('$500,000');
    await expect(authenticatedPage.locator('[data-testid="current-position-AAPL"]')).toContainText('100 shares');

    // Verify portfolio impact
    await expect(authenticatedPage.locator('[data-testid="impact-new-cash"]')).toContainText('$430,000');
    await expect(authenticatedPage.locator('[data-testid="impact-new-position-AAPL"]')).toContainText('500 shares');
    await expect(authenticatedPage.locator('[data-testid="impact-concentration"]')).toContainText('14%');
  });

  test('should filter pending trades by agent', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      agents: [
        {
          id: 'agent-1',
          name: 'Agent 1',
          budget: 100000,
          strategy: 'moderate',
          riskTolerance: 'medium',
          status: 'active',
          createdAt: new Date().toISOString(),
        },
        {
          id: 'agent-2',
          name: 'Agent 2',
          budget: 200000,
          strategy: 'aggressive',
          riskTolerance: 'high',
          status: 'active',
          createdAt: new Date().toISOString(),
        },
      ],
      trades: [
        {
          id: 'trade-1',
          agentId: 'agent-1',
          symbol: 'AAPL',
          type: 'BUY',
          quantity: 300,
          price: 175.00,
          total: 52500,
          status: 'pending',
          requiresApproval: true,
          timestamp: new Date().toISOString(),
        },
        {
          id: 'trade-2',
          agentId: 'agent-2',
          symbol: 'TSLA',
          type: 'BUY',
          quantity: 500,
          price: 250.00,
          total: 125000,
          status: 'pending',
          requiresApproval: true,
          timestamp: new Date().toISOString(),
        },
      ],
    });

    await authenticatedPage.goto('/oversight');

    // Verify both trades visible initially
    await expect(authenticatedPage.locator('[data-testid="pending-count"]')).toHaveText('2');

    // Filter by Agent 1
    await authenticatedPage.selectOption('[name="agentFilter"]', 'agent-1');

    // Verify only Agent 1 trade visible
    await expect(authenticatedPage.locator('[data-testid="pending-trade-trade-1"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="pending-trade-trade-2"]')).not.toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="filtered-count"]')).toHaveText('1');

    // Clear filter
    await authenticatedPage.selectOption('[name="agentFilter"]', 'all');

    // Verify both visible again
    await expect(authenticatedPage.locator('[data-testid="pending-count"]')).toHaveText('2');
  });

  test('should update pending list in real-time via WebSocket', async ({ authenticatedPage, mockWebSocket, mockAPI }) => {
    await mockAPI({
      trades: [],
    });

    await authenticatedPage.goto('/oversight');

    // Initial state: no pending trades
    await expect(authenticatedPage.locator('[data-testid="pending-count"]')).toHaveText('0');
    await expect(authenticatedPage.locator('[data-testid="empty-state"]')).toBeVisible();

    // Simulate new pending trade via WebSocket
    mockWebSocket.clients.forEach((client) => {
      client.send(JSON.stringify({
        type: 'trade_pending',
        data: {
          id: 'trade-realtime-1',
          agentId: 'agent-large-trader',
          symbol: 'NVDA',
          type: 'BUY',
          quantity: 200,
          price: 450.00,
          total: 90000,
          status: 'pending',
          requiresApproval: true,
          timestamp: new Date().toISOString(),
        },
        timestamp: new Date().toISOString(),
      }));
    });

    // Verify trade appears
    await expect(authenticatedPage.locator('[data-testid="pending-trade-trade-realtime-1"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="pending-count"]')).toHaveText('1');
    await expect(authenticatedPage.locator('[data-testid="empty-state"]')).not.toBeVisible();
  });

  test('should support keyboard shortcuts for approval actions', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      trades: [
        {
          id: 'trade-keyboard-1',
          agentId: 'agent-large-trader',
          symbol: 'AAPL',
          type: 'BUY',
          quantity: 400,
          price: 175.00,
          total: 70000,
          status: 'pending',
          requiresApproval: true,
          timestamp: new Date().toISOString(),
        },
      ],
    });

    await authenticatedPage.goto('/oversight');
    await authenticatedPage.click('[data-testid="pending-trade-trade-keyboard-1"]');

    // Press 'A' for approve
    await authenticatedPage.keyboard.press('a');

    // Verify approval dialog opened
    await expect(authenticatedPage.locator('[data-testid="approve-confirmation"]')).toBeVisible();

    // Press Escape to cancel
    await authenticatedPage.keyboard.press('Escape');

    // Verify dialog closed
    await expect(authenticatedPage.locator('[data-testid="approve-confirmation"]')).not.toBeVisible();

    // Press 'R' for reject
    await authenticatedPage.keyboard.press('r');

    // Verify rejection modal opened
    await expect(authenticatedPage.locator('[data-testid="reject-modal"]')).toBeVisible();
  });
});
