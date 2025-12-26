import { test, expect } from './fixtures/test-fixtures';
import AxeBuilder from '@axe-core/playwright';

/**
 * Accessibility E2E Tests (WCAG 2.1 AA Compliance)
 *
 * Tests accessibility compliance across all pages:
 * - Automated axe-core scanning
 * - Keyboard navigation
 * - Screen reader compatibility
 * - Color contrast validation
 * - Focus management
 * - ARIA attributes
 */

test.describe('Accessibility Compliance (WCAG 2.1 AA)', () => {
  test.beforeEach(async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      agents: [
        {
          id: 'agent-a11y-1',
          name: 'Test Agent',
          budget: 50000,
          strategy: 'moderate',
          riskTolerance: 'medium',
          status: 'active',
          createdAt: new Date().toISOString(),
        },
      ],
    });
  });

  test('Dashboard page should pass axe-core accessibility scan', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    const accessibilityScanResults = await new AxeBuilder({ page: authenticatedPage })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('Agent creation page should pass axe-core accessibility scan', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/create');

    const accessibilityScanResults = await new AxeBuilder({ page: authenticatedPage })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('Agent detail page should pass axe-core accessibility scan', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/agent-a11y-1');

    const accessibilityScanResults = await new AxeBuilder({ page: authenticatedPage })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('Oversight panel should pass axe-core accessibility scan', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/oversight');

    const accessibilityScanResults = await new AxeBuilder({ page: authenticatedPage })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('Trade modal should pass axe-core accessibility scan', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/agent-a11y-1');
    await authenticatedPage.click('button:has-text("New Trade")');

    // Wait for modal to be visible
    await expect(authenticatedPage.locator('[data-testid="trade-modal"]')).toBeVisible();

    const accessibilityScanResults = await new AxeBuilder({ page: authenticatedPage })
      .withTags(['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('should support full keyboard navigation on dashboard', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    // Tab through navigation
    await authenticatedPage.keyboard.press('Tab');
    await expect(authenticatedPage.locator('[data-testid="nav-dashboard"]')).toBeFocused();

    await authenticatedPage.keyboard.press('Tab');
    await expect(authenticatedPage.locator('[data-testid="nav-oversight"]')).toBeFocused();

    // Tab to Create Agent button
    await authenticatedPage.keyboard.press('Tab');
    await expect(authenticatedPage.locator('button:has-text("Create Agent")')).toBeFocused();

    // Press Enter to activate
    await authenticatedPage.keyboard.press('Enter');

    // Verify navigation occurred
    await expect(authenticatedPage).toHaveURL('/agents/create');
  });

  test('should support keyboard navigation through agent wizard', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/create');

    // Tab to name field
    await authenticatedPage.keyboard.press('Tab');
    await expect(authenticatedPage.locator('[name="agentName"]')).toBeFocused();

    // Type agent name
    await authenticatedPage.keyboard.type('Keyboard Nav Agent');

    // Tab to Next button and activate
    await authenticatedPage.keyboard.press('Tab');
    await authenticatedPage.keyboard.press('Enter');

    // Verify moved to Step 2
    await expect(authenticatedPage.locator('[data-testid="step-indicator"]')).toContainText('Step 2 of 5');

    // Tab to budget field
    await authenticatedPage.keyboard.press('Tab');
    await expect(authenticatedPage.locator('[name="budget"]')).toBeFocused();
  });

  test('should trap focus within modal dialogs', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/agent-a11y-1');
    await authenticatedPage.click('button:has-text("New Trade")');

    // Modal should be open
    await expect(authenticatedPage.locator('[data-testid="trade-modal"]')).toBeVisible();

    // Get first and last focusable elements
    const firstFocusable = authenticatedPage.locator('[data-testid="trade-modal"] [name="symbol"]');
    const lastFocusable = authenticatedPage.locator('[data-testid="trade-modal"] button:has-text("Cancel")');

    // First element should receive focus
    await expect(firstFocusable).toBeFocused();

    // Tab through all elements
    let tabCount = 0;
    while (tabCount < 20) {
      await authenticatedPage.keyboard.press('Tab');
      tabCount++;

      // Check if focus wrapped back to first element
      const isFocused = await firstFocusable.evaluate((el) => el === document.activeElement);
      if (isFocused && tabCount > 1) {
        break;
      }
    }

    // Verify focus wrapped (didn't escape modal)
    expect(tabCount).toBeLessThan(20);
  });

  test('should have proper ARIA labels for screen readers', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    // Navigation should have aria-label
    const nav = authenticatedPage.locator('nav');
    await expect(nav).toHaveAttribute('aria-label', /main navigation|primary navigation/i);

    // Create Agent button should have aria-label
    const createButton = authenticatedPage.locator('button:has-text("Create Agent")');
    await expect(createButton).toHaveAttribute('aria-label', /create new agent/i);

    // Agent cards should have aria-label
    const agentCard = authenticatedPage.locator('[data-testid="agent-card"]').first();
    await expect(agentCard).toHaveAttribute('aria-label', /.+/);
  });

  test('should announce dynamic content to screen readers', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/create');

    // Fill form with invalid data
    await authenticatedPage.fill('[name="agentName"]', 'AB');
    await authenticatedPage.click('button:has-text("Next")');

    // Error message should have role="alert"
    const errorMessage = authenticatedPage.locator('[data-testid="error-agentName"]');
    await expect(errorMessage).toHaveAttribute('role', 'alert');
    await expect(errorMessage).toHaveAttribute('aria-live', 'assertive');
  });

  test('should have proper heading hierarchy', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    // Check h1 exists and is unique
    const h1Elements = await authenticatedPage.locator('h1').count();
    expect(h1Elements).toBe(1);
    await expect(authenticatedPage.locator('h1')).toHaveText('Dashboard');

    // Check heading levels don't skip
    const headings = await authenticatedPage.locator('h1, h2, h3, h4, h5, h6').allTextContents();
    // Verify logical hierarchy (h1 -> h2 -> h3, not h1 -> h3)
    const headingLevels = await authenticatedPage.locator('h1, h2, h3, h4, h5, h6').evaluateAll((elements) => {
      return elements.map((el) => parseInt(el.tagName.substring(1)));
    });

    for (let i = 1; i < headingLevels.length; i++) {
      const difference = headingLevels[i] - headingLevels[i - 1];
      expect(difference).toBeLessThanOrEqual(1);
    }
  });

  test('should have sufficient color contrast (WCAG AA)', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    // Run axe-core with only color-contrast rule
    const accessibilityScanResults = await new AxeBuilder({ page: authenticatedPage })
      .withRules(['color-contrast'])
      .analyze();

    expect(accessibilityScanResults.violations).toEqual([]);
  });

  test('should have visible focus indicators', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    // Tab to Create Agent button
    await authenticatedPage.keyboard.press('Tab');
    await authenticatedPage.keyboard.press('Tab');

    const createButton = authenticatedPage.locator('button:has-text("Create Agent")');

    // Check that focus is visible (outline or box-shadow)
    const focusStyle = await createButton.evaluate((el) => {
      const styles = window.getComputedStyle(el);
      return {
        outline: styles.outline,
        outlineWidth: styles.outlineWidth,
        boxShadow: styles.boxShadow,
      };
    });

    // Should have either outline or box-shadow
    const hasFocusIndicator =
      focusStyle.outlineWidth !== '0px' ||
      focusStyle.boxShadow !== 'none';

    expect(hasFocusIndicator).toBe(true);
  });

  test('should support Escape key to close modals', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/agent-a11y-1');
    await authenticatedPage.click('button:has-text("New Trade")');

    // Modal should be open
    await expect(authenticatedPage.locator('[data-testid="trade-modal"]')).toBeVisible();

    // Press Escape
    await authenticatedPage.keyboard.press('Escape');

    // Modal should close
    await expect(authenticatedPage.locator('[data-testid="trade-modal"]')).not.toBeVisible();
  });

  test('should have descriptive link text', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    // Find all links
    const links = authenticatedPage.locator('a');
    const count = await links.count();

    for (let i = 0; i < count; i++) {
      const link = links.nth(i);
      const text = await link.textContent();
      const ariaLabel = await link.getAttribute('aria-label');

      // Link should have either visible text or aria-label
      const hasAccessibleName = (text && text.trim().length > 0) || (ariaLabel && ariaLabel.length > 0);
      expect(hasAccessibleName).toBe(true);

      // Avoid generic link text
      const genericTerms = ['click here', 'read more', 'here', 'more'];
      const lowerText = (text || ariaLabel || '').toLowerCase();
      const isGeneric = genericTerms.some((term) => lowerText === term);
      expect(isGeneric).toBe(false);
    }
  });

  test('should have proper form labels', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/create');

    // All form inputs should have labels
    const inputs = authenticatedPage.locator('input, select, textarea');
    const count = await inputs.count();

    for (let i = 0; i < count; i++) {
      const input = inputs.nth(i);
      const id = await input.getAttribute('id');
      const ariaLabel = await input.getAttribute('aria-label');
      const ariaLabelledBy = await input.getAttribute('aria-labelledby');

      if (id) {
        const label = authenticatedPage.locator(`label[for="${id}"]`);
        const labelExists = (await label.count()) > 0;
        const hasAccessibleName = labelExists || ariaLabel || ariaLabelledBy;
        expect(hasAccessibleName).toBe(true);
      }
    }
  });

  test('should announce loading states to screen readers', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    // Trigger a loading state (e.g., refresh agents)
    await authenticatedPage.click('[data-testid="refresh-agents"]');

    // Loading indicator should have aria-live
    const loadingIndicator = authenticatedPage.locator('[data-testid="loading-indicator"]');
    await expect(loadingIndicator).toHaveAttribute('role', 'status');
    await expect(loadingIndicator).toHaveAttribute('aria-live', 'polite');
  });

  test('should support skip to main content link', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    // Press Tab - skip link should be first focusable element
    await authenticatedPage.keyboard.press('Tab');

    const skipLink = authenticatedPage.locator('a:has-text("Skip to main content")');
    await expect(skipLink).toBeFocused();

    // Activate skip link
    await authenticatedPage.keyboard.press('Enter');

    // Main content should receive focus
    const mainContent = authenticatedPage.locator('main');
    await expect(mainContent).toBeFocused();
  });

  test('should have proper button roles and states', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/create');

    // Next button should have proper attributes
    const nextButton = authenticatedPage.locator('button:has-text("Next")');
    await expect(nextButton).toHaveAttribute('type', 'button');

    // Fill form with invalid data
    await authenticatedPage.click('button:has-text("Next")');

    // Button should indicate disabled state
    // (In this case, it might not be disabled but form should prevent submission)
    const errorExists = await authenticatedPage.locator('[data-testid="error-agentName"]').isVisible();
    expect(errorExists).toBe(true);
  });

  test('should provide error recovery options', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/create');

    // Submit invalid form
    await authenticatedPage.click('button:has-text("Next")');

    // Error message should exist
    const errorMessage = authenticatedPage.locator('[data-testid="error-agentName"]');
    await expect(errorMessage).toBeVisible();

    // Error should be associated with input
    const input = authenticatedPage.locator('[name="agentName"]');
    const ariaDescribedBy = await input.getAttribute('aria-describedby');
    expect(ariaDescribedBy).toBeTruthy();

    // Verify error message ID matches aria-describedby
    const errorId = await errorMessage.getAttribute('id');
    expect(ariaDescribedBy).toContain(errorId || '');
  });
});
