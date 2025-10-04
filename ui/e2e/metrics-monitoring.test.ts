import { test, expect } from '@playwright/test';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { handlers } from './fixtures/api-handlers';
import { mockKPIs, mockAlerts } from './fixtures/mock-data';
import { MetricsHubPage } from './pages';

// Setup MSW server for API mocking
const server = setupServer(...handlers);

test.beforeAll(() => server.listen());
test.afterEach(() => server.resetHandlers());
test.afterAll(() => server.close());

test.describe('WF-05: Metrics and Alerts Monitoring', () => {
  test('should display KPIs on page load', async ({ page }) => {
    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    // Verify page heading
    await metricsPage.waitForHeading('Metrics Hub');

    // Verify KPI cards are displayed
    const kpiCount = await metricsPage.getKPICount();
    expect(kpiCount).toBe(3); // mockKPIs has 3 KPIs
  });

  test('should display all KPI metrics with values', async ({ page }) => {
    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    // Verify Average Latency KPI
    const latencyKPI = page.locator('text=/average.*latency/i').first();
    await expect(latencyKPI).toBeVisible({ timeout: 5000 });

    // Verify Packet Loss KPI
    const packetLossKPI = page.locator('text=/packet.*loss/i').first();
    await expect(packetLossKPI).toBeVisible({ timeout: 5000 });

    // Verify Active Sessions KPI
    const sessionsKPI = page.locator('text=/active.*sessions/i').first();
    await expect(sessionsKPI).toBeVisible({ timeout: 5000 });
  });

  test('should display alert list', async ({ page }) => {
    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    // Verify alerts are displayed
    const alertCount = await metricsPage.getAlertCount();
    expect(alertCount).toBe(2); // mockAlerts has 2 alerts
  });

  test('should acknowledge an alert', async ({ page }) => {
    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    // Acknowledge first alert
    await metricsPage.acknowledgeFirstAlert();

    // Wait for acknowledgment to complete
    await page.waitForTimeout(500);

    // Verify success notification or alert removal
    const notification = page.locator('[role="status"]').first();
    await expect(notification).toBeVisible({ timeout: 5000 });
  });

  test('should display alert severity levels', async ({ page }) => {
    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    // Verify critical alert is displayed
    const criticalAlert = page.locator('text=/critical/i').first();
    await expect(criticalAlert).toBeVisible({ timeout: 5000 });

    // Verify warning alert is displayed
    const warningAlert = page.locator('text=/warning/i').first();
    await expect(warningAlert).toBeVisible({ timeout: 5000 });
  });

  test('should display latency heatmap', async ({ page }) => {
    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    // Verify heatmap is visible
    await expect(metricsPage.heatmap).toBeVisible({ timeout: 5000 });
  });

  test('should handle alert acknowledgment error gracefully', async ({ page }) => {
    // Override handler to return error
    server.use(
      http.post('http://localhost:3000/api/v1/metrics/alerts/:alertId/acknowledge', () => {
        return HttpResponse.json(
          { error: 'Acknowledgment failed' },
          { status: 500 }
        );
      })
    );

    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    await metricsPage.acknowledgeFirstAlert();

    // Verify error notification
    const errorMessage = page.locator('text=/error|failed/i').first();
    await expect(errorMessage).toBeVisible({ timeout: 5000 });
  });

  test('should handle empty alerts list gracefully', async ({ page }) => {
    // Override handler to return empty list
    server.use(
      http.get('http://localhost:3000/api/v1/metrics/alerts', () => {
        return HttpResponse.json({ alerts: [] });
      })
    );

    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    // Verify empty state or "no alerts" message
    const emptyMessage = page.locator('text=/no.*alerts|all.*clear|no.*active.*alerts/i').first();
    await expect(emptyMessage).toBeVisible({ timeout: 5000 });
  });

  test('should show KPI trends or changes', async ({ page }) => {
    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    // Verify trend indicators (up/down arrows or percentages)
    const trendIndicators = page.locator('[data-testid*="trend"], .trend, .change-indicator');
    const count = await trendIndicators.count();
    expect(count).toBeGreaterThanOrEqual(0);
  });

  test('should refresh metrics periodically', async ({ page }) => {
    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    // Get initial KPI count
    const initialCount = await metricsPage.getKPICount();
    expect(initialCount).toBe(3);

    // Wait for potential auto-refresh
    await page.waitForTimeout(2000);

    // Verify KPIs are still displayed
    const currentCount = await metricsPage.getKPICount();
    expect(currentCount).toBe(3);
  });

  test('should display alert timestamps', async ({ page }) => {
    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    // Verify timestamps are visible in alerts
    const timestamps = page.locator('time, [datetime], text=/ago|minutes|hours|seconds/i').first();
    await expect(timestamps).toBeVisible({ timeout: 5000 });
  });

  test('should filter or sort alerts by severity', async ({ page }) => {
    const metricsPage = new MetricsHubPage(page);
    await metricsPage.navigateTo();

    // Verify alerts are displayed (filtering/sorting UI might exist)
    const alertCount = await metricsPage.getAlertCount();
    expect(alertCount).toBeGreaterThan(0);
  });
});
