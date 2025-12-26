import { test, expect } from './fixtures/test-fixtures';

/**
 * Agent Creation Wizard E2E Tests
 *
 * Tests the complete agent creation workflow:
 * - Navigation to create agent page
 * - Multi-step wizard completion
 * - Form validation at each step
 * - Agent creation confirmation
 * - Metering quota initialization
 * - Agent appearance on dashboard
 */

test.describe('Agent Creation Wizard', () => {
  test.beforeEach(async ({ authenticatedPage, mockAPI }) => {
    await mockAPI();
    await authenticatedPage.goto('/');
  });

  test('should navigate to agent creation page', async ({ authenticatedPage }) => {
    // Click "Create Agent" button from dashboard
    await authenticatedPage.click('button:has-text("Create Agent")');

    // Verify navigation to wizard
    await expect(authenticatedPage).toHaveURL('/agents/create');
    await expect(authenticatedPage.locator('h1')).toHaveText('Create New Agent');

    // Verify step indicator shows Step 1
    await expect(authenticatedPage.locator('[data-testid="step-indicator"]')).toContainText('Step 1 of 5');
  });

  test('should complete full wizard flow and create agent', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/create');

    // Step 1: Agent Name
    await authenticatedPage.fill('[name="agentName"]', 'Momentum Trading Agent');
    await authenticatedPage.click('button:has-text("Next")');

    // Verify moved to Step 2
    await expect(authenticatedPage.locator('[data-testid="step-indicator"]')).toContainText('Step 2 of 5');

    // Step 2: Budget
    await authenticatedPage.fill('[name="budget"]', '25000');
    await authenticatedPage.selectOption('[name="currency"]', 'USD');
    await authenticatedPage.click('button:has-text("Next")');

    // Step 3: Strategy
    await expect(authenticatedPage.locator('[data-testid="step-indicator"]')).toContainText('Step 3 of 5');
    await authenticatedPage.click('[data-testid="strategy-moderate"]');
    await authenticatedPage.click('button:has-text("Next")');

    // Step 4: Risk Tolerance
    await expect(authenticatedPage.locator('[data-testid="step-indicator"]')).toContainText('Step 4 of 5');
    await authenticatedPage.click('[data-testid="risk-medium"]');
    await authenticatedPage.click('button:has-text("Next")');

    // Step 5: Confirmation
    await expect(authenticatedPage.locator('[data-testid="step-indicator"]')).toContainText('Step 5 of 5');

    // Verify summary displays correct information
    await expect(authenticatedPage.locator('[data-testid="summary-name"]')).toHaveText('Momentum Trading Agent');
    await expect(authenticatedPage.locator('[data-testid="summary-budget"]')).toContainText('$25,000');
    await expect(authenticatedPage.locator('[data-testid="summary-strategy"]')).toContainText('Moderate');
    await expect(authenticatedPage.locator('[data-testid="summary-risk"]')).toContainText('Medium');

    // Submit form
    await authenticatedPage.click('button:has-text("Create Agent")');

    // Verify success message
    await expect(authenticatedPage.locator('[data-testid="success-message"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="success-message"]')).toContainText('Agent created successfully');

    // Verify redirect to dashboard
    await authenticatedPage.waitForURL('/dashboard');

    // Verify new agent appears in list
    await expect(authenticatedPage.locator('[data-testid="agent-card"]').filter({ hasText: 'Momentum Trading Agent' })).toBeVisible();
  });

  test('should validate required fields at each step', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/create');

    // Step 1: Try to proceed without name
    await authenticatedPage.click('button:has-text("Next")');
    await expect(authenticatedPage.locator('[data-testid="error-agentName"]')).toHaveText('Agent name is required');

    // Enter invalid name (too short)
    await authenticatedPage.fill('[name="agentName"]', 'AB');
    await authenticatedPage.click('button:has-text("Next")');
    await expect(authenticatedPage.locator('[data-testid="error-agentName"]')).toHaveText('Agent name must be at least 3 characters');

    // Enter valid name
    await authenticatedPage.fill('[name="agentName"]', 'Valid Agent Name');
    await authenticatedPage.click('button:has-text("Next")');

    // Step 2: Try to proceed without budget
    await authenticatedPage.click('button:has-text("Next")');
    await expect(authenticatedPage.locator('[data-testid="error-budget"]')).toHaveText('Budget is required');

    // Enter invalid budget (too low)
    await authenticatedPage.fill('[name="budget"]', '500');
    await authenticatedPage.click('button:has-text("Next")');
    await expect(authenticatedPage.locator('[data-testid="error-budget"]')).toHaveText('Budget must be at least $1,000');

    // Enter invalid budget (too high)
    await authenticatedPage.fill('[name="budget"]', '1500000');
    await authenticatedPage.click('button:has-text("Next")');
    await expect(authenticatedPage.locator('[data-testid="error-budget"]')).toHaveText('Budget cannot exceed $1,000,000');

    // Enter valid budget
    await authenticatedPage.fill('[name="budget"]', '10000');
    await authenticatedPage.click('button:has-text("Next")');

    // Step 3: Try to proceed without selecting strategy
    await authenticatedPage.click('button:has-text("Next")');
    await expect(authenticatedPage.locator('[data-testid="error-strategy"]')).toHaveText('Please select a trading strategy');
  });

  test('should allow navigation back through wizard steps', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/create');

    // Complete Step 1
    await authenticatedPage.fill('[name="agentName"]', 'Test Agent');
    await authenticatedPage.click('button:has-text("Next")');

    // Complete Step 2
    await authenticatedPage.fill('[name="budget"]', '10000');
    await authenticatedPage.click('button:has-text("Next")');

    // On Step 3, click Back
    await authenticatedPage.click('button:has-text("Back")');

    // Verify returned to Step 2 with preserved values
    await expect(authenticatedPage.locator('[data-testid="step-indicator"]')).toContainText('Step 2 of 5');
    await expect(authenticatedPage.locator('[name="budget"]')).toHaveValue('10000');

    // Click Back again
    await authenticatedPage.click('button:has-text("Back")');

    // Verify returned to Step 1 with preserved values
    await expect(authenticatedPage.locator('[data-testid="step-indicator"]')).toContainText('Step 1 of 5');
    await expect(authenticatedPage.locator('[name="agentName"]')).toHaveValue('Test Agent');
  });

  test('should initialize metering quota for new agent', async ({ authenticatedPage, mockAPI }) => {
    // Setup mock to verify metering initialization
    let meteringInitialized = false;

    await authenticatedPage.route('**/api/metering/initialize', async (route) => {
      const postData = route.request().postDataJSON();
      meteringInitialized = true;

      await route.fulfill({
        status: 201,
        contentType: 'application/json',
        body: JSON.stringify({
          agentId: postData.agentId,
          quota: {
            total: 10000,
            used: 0,
            remaining: 10000,
          },
        }),
      });
    });

    await mockAPI();
    await authenticatedPage.goto('/agents/create');

    // Complete wizard
    await authenticatedPage.fill('[name="agentName"]', 'Test Agent');
    await authenticatedPage.click('button:has-text("Next")');
    await authenticatedPage.fill('[name="budget"]', '10000');
    await authenticatedPage.click('button:has-text("Next")');
    await authenticatedPage.click('[data-testid="strategy-conservative"]');
    await authenticatedPage.click('button:has-text("Next")');
    await authenticatedPage.click('[data-testid="risk-low"]');
    await authenticatedPage.click('button:has-text("Next")');
    await authenticatedPage.click('button:has-text("Create Agent")');

    // Wait for success
    await expect(authenticatedPage.locator('[data-testid="success-message"]')).toBeVisible();

    // Verify metering was initialized
    expect(meteringInitialized).toBe(true);
  });

  test('should handle API errors gracefully', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI();

    // Mock API failure
    await authenticatedPage.route('**/api/agents', async (route) => {
      if (route.request().method() === 'POST') {
        await route.fulfill({
          status: 500,
          contentType: 'application/json',
          body: JSON.stringify({
            error: 'Internal server error',
          }),
        });
      }
    });

    await authenticatedPage.goto('/agents/create');

    // Complete wizard
    await authenticatedPage.fill('[name="agentName"]', 'Test Agent');
    await authenticatedPage.click('button:has-text("Next")');
    await authenticatedPage.fill('[name="budget"]', '10000');
    await authenticatedPage.click('button:has-text("Next")');
    await authenticatedPage.click('[data-testid="strategy-conservative"]');
    await authenticatedPage.click('button:has-text("Next")');
    await authenticatedPage.click('[data-testid="risk-low"]');
    await authenticatedPage.click('button:has-text("Next")');
    await authenticatedPage.click('button:has-text("Create Agent")');

    // Verify error message displayed
    await expect(authenticatedPage.locator('[data-testid="error-message"]')).toBeVisible();
    await expect(authenticatedPage.locator('[data-testid="error-message"]')).toContainText('Failed to create agent');

    // Verify still on confirmation step
    await expect(authenticatedPage.locator('[data-testid="step-indicator"]')).toContainText('Step 5 of 5');
  });

  test('should prevent duplicate agent names', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      agents: [
        {
          id: 'agent-1',
          name: 'Existing Agent',
          budget: 5000,
          strategy: 'conservative',
          riskTolerance: 'low',
          status: 'active',
          createdAt: new Date().toISOString(),
        },
      ],
    });

    await authenticatedPage.goto('/agents/create');

    // Try to create agent with duplicate name
    await authenticatedPage.fill('[name="agentName"]', 'Existing Agent');
    await authenticatedPage.click('button:has-text("Next")');

    // Verify validation error
    await expect(authenticatedPage.locator('[data-testid="error-agentName"]')).toHaveText('An agent with this name already exists');
  });

  test('should support keyboard navigation', async ({ authenticatedPage, mockAPI }) => {
    await mockAPI();
    await authenticatedPage.goto('/agents/create');

    // Use Tab to navigate to name field
    await authenticatedPage.keyboard.press('Tab');
    await authenticatedPage.keyboard.type('Keyboard Test Agent');

    // Press Enter to proceed (should trigger Next button)
    await authenticatedPage.keyboard.press('Enter');

    // Verify moved to Step 2
    await expect(authenticatedPage.locator('[data-testid="step-indicator"]')).toContainText('Step 2 of 5');
  });
});
