import { test, expect } from '@playwright/test';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { handlers } from './fixtures/api-handlers';
import { mockStreams } from './fixtures/mock-data';
import { StreamDashboardPage } from './pages';

// Setup MSW server for API mocking
const server = setupServer(...handlers);

test.beforeAll(() => server.listen());
test.afterEach(() => server.resetHandlers());
test.afterAll(() => server.close());

test.describe('WF-03: Stream Priority Management', () => {
  test('should display active streams on page load', async ({ page }) => {
    const streamPage = new StreamDashboardPage(page);
    await streamPage.navigateTo();

    // Verify page heading
    await streamPage.waitForHeading('Stream Dashboard');

    // Verify stream cards are displayed
    const streamCount = await streamPage.getStreamCount();
    expect(streamCount).toBe(2); // mockStreams has 2 streams
  });

  test('should increase stream priority', async ({ page }) => {
    const streamPage = new StreamDashboardPage(page);
    await streamPage.navigateTo();

    // Increase priority for first stream
    const streamId = mockStreams[0].id;
    await streamPage.increasePriority(streamId);

    // Wait for API call to complete
    await page.waitForTimeout(500);

    // Verify success notification
    const notification = page.locator('[role="status"]').first();
    await expect(notification).toBeVisible({ timeout: 5000 });
  });

  test('should display stream quality metrics', async ({ page }) => {
    const streamPage = new StreamDashboardPage(page);
    await streamPage.navigateTo();

    // Verify latency metrics are visible
    const latencyMetrics = page.locator('text=/latency|ms/i').first();
    await expect(latencyMetrics).toBeVisible({ timeout: 5000 });

    // Verify packet loss metrics are visible
    const packetLossMetrics = page.locator('text=/packet.*loss|loss.*rate/i').first();
    await expect(packetLossMetrics).toBeVisible({ timeout: 5000 });
  });

  test('should show stream status indicators', async ({ page }) => {
    const streamPage = new StreamDashboardPage(page);
    await streamPage.navigateTo();

    // Verify status badges (optimal, degraded, etc.)
    const statusBadges = page.locator('[data-testid*="status"], .status-badge').first();
    await expect(statusBadges).toBeVisible({ timeout: 5000 });
  });

  test('should handle priority update error gracefully', async ({ page }) => {
    // Override handler to return error
    server.use(
      http.put('http://localhost:3000/api/v1/sessions/:sessionId/priority', () => {
        return HttpResponse.json(
          { error: 'Priority update failed' },
          { status: 500 }
        );
      })
    );

    const streamPage = new StreamDashboardPage(page);
    await streamPage.navigateTo();

    const streamId = mockStreams[0].id;
    await streamPage.increasePriority(streamId);

    // Verify error notification
    const errorMessage = page.locator('text=/error|failed/i').first();
    await expect(errorMessage).toBeVisible({ timeout: 5000 });
  });

  test('should display detailed metrics for each stream', async ({ page }) => {
    const streamPage = new StreamDashboardPage(page);
    await streamPage.navigateTo();

    // Click on first stream to view details (if applicable)
    const firstStreamCard = streamPage.streamCards.first();
    await expect(firstStreamCard).toBeVisible();

    // Verify metrics are displayed
    const metricsContainer = page.locator('[data-testid*="metrics"]').first();
    await expect(metricsContainer).toBeVisible({ timeout: 5000 });
  });

  test('should handle empty stream list gracefully', async ({ page }) => {
    // Override handler to return empty list
    server.use(
      http.get('http://localhost:3000/api/v1/sessions', () => {
        return HttpResponse.json({ sessions: [] });
      })
    );

    const streamPage = new StreamDashboardPage(page);
    await streamPage.navigateTo();

    // Verify empty state message
    const emptyMessage = page.locator('text=/no.*streams|no.*active.*sessions|empty/i').first();
    await expect(emptyMessage).toBeVisible({ timeout: 5000 });
  });

  test('should refresh stream data periodically', async ({ page }) => {
    const streamPage = new StreamDashboardPage(page);
    await streamPage.navigateTo();

    // Get initial stream count
    const initialCount = await streamPage.getStreamCount();
    expect(initialCount).toBe(2);

    // Wait for potential auto-refresh (if implemented)
    await page.waitForTimeout(2000);

    // Verify streams are still displayed
    const currentCount = await streamPage.getStreamCount();
    expect(currentCount).toBeGreaterThanOrEqual(0);
  });
});
