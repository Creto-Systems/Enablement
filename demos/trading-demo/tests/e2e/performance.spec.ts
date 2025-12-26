import { test, expect } from './fixtures/test-fixtures';

/**
 * Performance E2E Tests
 *
 * Tests performance characteristics:
 * - Page load time < 2s
 * - API response time < 200ms
 * - WebSocket connection < 500ms
 * - Layout stability (CLS < 0.1)
 * - First Contentful Paint (FCP)
 * - Time to Interactive (TTI)
 */

test.describe('Performance Metrics', () => {
  test.beforeEach(async ({ authenticatedPage, mockAPI }) => {
    await mockAPI({
      agents: [
        {
          id: 'agent-perf-1',
          name: 'Performance Test Agent',
          budget: 50000,
          strategy: 'moderate',
          riskTolerance: 'medium',
          status: 'active',
          createdAt: new Date().toISOString(),
        },
      ],
    });
  });

  test('Dashboard should load in under 2 seconds', async ({ authenticatedPage }) => {
    const startTime = Date.now();

    await authenticatedPage.goto('/dashboard', {
      waitUntil: 'networkidle',
    });

    const loadTime = Date.now() - startTime;

    // Verify load time
    expect(loadTime).toBeLessThan(2000);

    // Log performance metrics
    const performanceMetrics = await authenticatedPage.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      return {
        domContentLoaded: navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
        loadComplete: navigation.loadEventEnd - navigation.loadEventStart,
        domInteractive: navigation.domInteractive - navigation.fetchStart,
      };
    });

    console.log('Performance Metrics:', performanceMetrics);
  });

  test('API requests should respond in under 200ms', async ({ authenticatedPage, page }) => {
    const apiTimes: number[] = [];

    // Monitor API requests
    page.on('response', async (response) => {
      if (response.url().includes('/api/')) {
        const timing = response.timing();
        const responseTime = timing.responseEnd;
        apiTimes.push(responseTime);
      }
    });

    await authenticatedPage.goto('/dashboard');

    // Trigger various API calls
    await authenticatedPage.click('[data-testid="refresh-agents"]');
    await authenticatedPage.goto('/agents/agent-perf-1');

    // Wait for all requests to complete
    await authenticatedPage.waitForLoadState('networkidle');

    // Verify all API calls were fast
    apiTimes.forEach((time) => {
      expect(time).toBeLessThan(200);
    });

    // Calculate average
    const avgTime = apiTimes.reduce((a, b) => a + b, 0) / apiTimes.length;
    console.log(`Average API response time: ${avgTime.toFixed(2)}ms`);
    expect(avgTime).toBeLessThan(150);
  });

  test('WebSocket should connect in under 500ms', async ({ authenticatedPage, mockWebSocket }) => {
    const startTime = Date.now();

    // Wait for WebSocket connection
    const wsConnected = new Promise((resolve) => {
      mockWebSocket.on('connection', () => {
        const connectionTime = Date.now() - startTime;
        resolve(connectionTime);
      });
    });

    await authenticatedPage.goto('/dashboard');

    const connectionTime = await wsConnected;

    expect(connectionTime).toBeLessThan(500);
    console.log(`WebSocket connection time: ${connectionTime}ms`);
  });

  test('Page should have minimal layout shift (CLS < 0.1)', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    // Wait for page to be fully loaded
    await authenticatedPage.waitForLoadState('networkidle');

    // Measure Cumulative Layout Shift
    const cls = await authenticatedPage.evaluate(() => {
      return new Promise<number>((resolve) => {
        let clsValue = 0;
        const observer = new PerformanceObserver((list) => {
          for (const entry of list.getEntries()) {
            if ((entry as any).hadRecentInput) {
              continue;
            }
            clsValue += (entry as any).value;
          }
        });

        observer.observe({ type: 'layout-shift', buffered: true });

        // Wait 2 seconds then resolve
        setTimeout(() => {
          observer.disconnect();
          resolve(clsValue);
        }, 2000);
      });
    });

    console.log(`Cumulative Layout Shift: ${cls.toFixed(4)}`);
    expect(cls).toBeLessThan(0.1);
  });

  test('First Contentful Paint should be under 1.5s', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard', {
      waitUntil: 'domcontentloaded',
    });

    const fcp = await authenticatedPage.evaluate(() => {
      const entry = performance.getEntriesByName('first-contentful-paint')[0] as PerformanceEntry;
      return entry ? entry.startTime : 0;
    });

    console.log(`First Contentful Paint: ${fcp.toFixed(2)}ms`);
    expect(fcp).toBeLessThan(1500);
  });

  test('Time to Interactive should be under 3s', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    const tti = await authenticatedPage.evaluate(() => {
      return new Promise<number>((resolve) => {
        const observer = new PerformanceObserver((list) => {
          const entries = list.getEntries();
          const interactiveEntry = entries.find((entry) => entry.name === 'interactive');
          if (interactiveEntry) {
            resolve(interactiveEntry.startTime);
            observer.disconnect();
          }
        });

        observer.observe({ type: 'measure', buffered: true });

        // Fallback: use load event
        setTimeout(() => {
          const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
          resolve(navigation.domInteractive);
        }, 5000);
      });
    });

    console.log(`Time to Interactive: ${tti.toFixed(2)}ms`);
    expect(tti).toBeLessThan(3000);
  });

  test('Large list rendering should be performant', async ({ authenticatedPage, mockAPI }) => {
    // Create 100 agents for testing
    const largeAgentList = Array.from({ length: 100 }, (_, i) => ({
      id: `agent-${i}`,
      name: `Agent ${i}`,
      budget: 10000 + i * 100,
      strategy: ['conservative', 'moderate', 'aggressive'][i % 3] as 'conservative' | 'moderate' | 'aggressive',
      riskTolerance: ['low', 'medium', 'high'][i % 3] as 'low' | 'medium' | 'high',
      status: 'active' as const,
      createdAt: new Date().toISOString(),
    }));

    await mockAPI({ agents: largeAgentList });

    const startTime = Date.now();
    await authenticatedPage.goto('/dashboard');

    // Wait for all agent cards to render
    await expect(authenticatedPage.locator('[data-testid="agent-card"]').first()).toBeVisible();

    const renderTime = Date.now() - startTime;

    console.log(`Rendered 100 agents in: ${renderTime}ms`);
    expect(renderTime).toBeLessThan(3000);

    // Verify scrolling is smooth (no jank)
    await authenticatedPage.mouse.wheel(0, 1000);
    await authenticatedPage.waitForTimeout(100);
    await authenticatedPage.mouse.wheel(0, 1000);

    // No specific assertion, but monitoring for crashes/freezes
  });

  test('Trade modal should open quickly', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/agents/agent-perf-1');

    const startTime = Date.now();
    await authenticatedPage.click('button:has-text("New Trade")');
    await expect(authenticatedPage.locator('[data-testid="trade-modal"]')).toBeVisible();
    const openTime = Date.now() - startTime;

    console.log(`Trade modal opened in: ${openTime}ms`);
    expect(openTime).toBeLessThan(500);
  });

  test('Search/filter should respond instantly', async ({ authenticatedPage, mockAPI }) => {
    // Create agents with diverse names
    const agents = Array.from({ length: 50 }, (_, i) => ({
      id: `agent-${i}`,
      name: `${['Alpha', 'Beta', 'Gamma', 'Delta'][i % 4]} Agent ${i}`,
      budget: 10000,
      strategy: 'moderate' as const,
      riskTolerance: 'medium' as const,
      status: 'active' as const,
      createdAt: new Date().toISOString(),
    }));

    await mockAPI({ agents });
    await authenticatedPage.goto('/dashboard');

    // Type in search box
    const searchInput = authenticatedPage.locator('[data-testid="search-agents"]');

    const startTime = Date.now();
    await searchInput.fill('Alpha');

    // Wait for filtered results
    await expect(authenticatedPage.locator('[data-testid="agent-card"]').first()).toContainText('Alpha');
    const filterTime = Date.now() - startTime;

    console.log(`Filter applied in: ${filterTime}ms`);
    expect(filterTime).toBeLessThan(300);
  });

  test('Memory usage should remain stable during navigation', async ({ authenticatedPage }) => {
    // Get initial memory
    const initialMemory = await authenticatedPage.evaluate(() => {
      return (performance as any).memory ? (performance as any).memory.usedJSHeapSize : 0;
    });

    // Navigate multiple times
    for (let i = 0; i < 10; i++) {
      await authenticatedPage.goto('/dashboard');
      await authenticatedPage.goto('/agents/agent-perf-1');
      await authenticatedPage.goto('/oversight');
    }

    // Get final memory
    const finalMemory = await authenticatedPage.evaluate(() => {
      return (performance as any).memory ? (performance as any).memory.usedJSHeapSize : 0;
    });

    if (initialMemory > 0 && finalMemory > 0) {
      const memoryIncrease = ((finalMemory - initialMemory) / initialMemory) * 100;
      console.log(`Memory increase after 10 navigations: ${memoryIncrease.toFixed(2)}%`);

      // Memory shouldn't increase by more than 50%
      expect(memoryIncrease).toBeLessThan(50);
    }
  });

  test('Bundle size should be reasonable', async ({ authenticatedPage, page }) => {
    const resourceSizes: { [key: string]: number } = {};

    page.on('response', async (response) => {
      const url = response.url();
      if (url.includes('.js') || url.includes('.css')) {
        const buffer = await response.body().catch(() => null);
        if (buffer) {
          resourceSizes[url] = buffer.length;
        }
      }
    });

    await authenticatedPage.goto('/dashboard');
    await authenticatedPage.waitForLoadState('networkidle');

    // Calculate total bundle size
    const totalSize = Object.values(resourceSizes).reduce((sum, size) => sum + size, 0);
    const totalSizeMB = totalSize / (1024 * 1024);

    console.log(`Total bundle size: ${totalSizeMB.toFixed(2)} MB`);
    console.log('Resource breakdown:', Object.entries(resourceSizes).map(([url, size]) => ({
      file: url.split('/').pop(),
      size: `${(size / 1024).toFixed(2)} KB`,
    })));

    // Total bundle should be under 2MB for initial load
    expect(totalSizeMB).toBeLessThan(2);
  });

  test('Real-time updates should not cause performance degradation', async ({ authenticatedPage, mockWebSocket, mockAPI }) => {
    await mockAPI();
    await authenticatedPage.goto('/agents/agent-perf-1');

    // Monitor frame rate
    const frameRates: number[] = [];

    await authenticatedPage.evaluate(() => {
      let lastTime = performance.now();
      const measureFPS = () => {
        const now = performance.now();
        const fps = 1000 / (now - lastTime);
        (window as any).currentFPS = fps;
        lastTime = now;
        requestAnimationFrame(measureFPS);
      };
      requestAnimationFrame(measureFPS);
    });

    // Send 50 WebSocket updates rapidly
    for (let i = 0; i < 50; i++) {
      mockWebSocket.clients.forEach((client) => {
        client.send(JSON.stringify({
          type: 'portfolio_update',
          agentId: 'agent-perf-1',
          data: {
            cash: 50000 - (i * 100),
            positions: [{
              symbol: 'AAPL',
              quantity: i,
              averagePrice: 175.00,
              currentPrice: 175.00 + (i * 0.1),
            }],
          },
          timestamp: new Date().toISOString(),
        }));
      });

      await authenticatedPage.waitForTimeout(50);

      // Sample frame rate
      const fps = await authenticatedPage.evaluate(() => (window as any).currentFPS);
      if (fps) frameRates.push(fps);
    }

    // Average FPS should be above 30 (preferably 60)
    const avgFPS = frameRates.reduce((a, b) => a + b, 0) / frameRates.length;
    console.log(`Average FPS during updates: ${avgFPS.toFixed(2)}`);
    expect(avgFPS).toBeGreaterThan(30);
  });

  test('Image loading should use lazy loading', async ({ authenticatedPage }) => {
    await authenticatedPage.goto('/dashboard');

    // Check all images have loading="lazy"
    const images = authenticatedPage.locator('img');
    const count = await images.count();

    for (let i = 0; i < count; i++) {
      const img = images.nth(i);
      const loading = await img.getAttribute('loading');

      // Images below the fold should have lazy loading
      const isInViewport = await img.evaluate((el) => {
        const rect = el.getBoundingClientRect();
        return rect.top < window.innerHeight;
      });

      if (!isInViewport) {
        expect(loading).toBe('lazy');
      }
    }
  });
});
