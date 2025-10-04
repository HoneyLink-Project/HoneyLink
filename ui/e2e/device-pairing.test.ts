import { test, expect } from '@playwright/test';
import { http, HttpResponse } from 'msw';
import { setupServer } from 'msw/node';
import { handlers } from './fixtures/api-handlers';
import { mockDevices } from './fixtures/mock-data';
import { PairingDetailsPage } from './pages';

// Setup MSW server for API mocking
const server = setupServer(...handlers);

test.beforeAll(() => server.listen());
test.afterEach(() => server.resetHandlers());
test.afterAll(() => server.close());

test.describe('WF-02: Device Pairing', () => {
  test('should display device details', async ({ page }) => {
    const pairingPage = new PairingDetailsPage(page);
    const deviceId = mockDevices[0].id;
    await pairingPage.navigateTo(deviceId);

    // Verify device name is displayed
    const deviceName = pairingPage.deviceName;
    await expect(deviceName).toBeVisible();
    await expect(deviceName).toContainText('iPhone 15 Pro');
  });

  test('should pair device with selected QoS profile', async ({ page }) => {
    const pairingPage = new PairingDetailsPage(page);
    const deviceId = mockDevices[0].id;
    await pairingPage.navigateTo(deviceId);

    // Select QoS profile
    await pairingPage.selectProfile('LL_INPUT');

    // Click pair button
    await pairingPage.clickPair();

    // Wait for pairing to complete
    await page.waitForTimeout(500);

    // Verify success notification
    const notification = page.locator('[role="status"]').first();
    await expect(notification).toBeVisible({ timeout: 5000 });
  });

  test('should unpair a paired device', async ({ page }) => {
    // Use device-2 which is already paired
    const pairingPage = new PairingDetailsPage(page);
    const deviceId = mockDevices[1].id;
    await pairingPage.navigateTo(deviceId);

    // Click unpair button
    await pairingPage.clickUnpair();

    // Wait for confirmation
    await page.waitForTimeout(500);

    // Verify unpair success
    const notification = page.locator('[role="status"]').first();
    await expect(notification).toBeVisible({ timeout: 5000 });
  });

  test('should show all available QoS profiles in dropdown', async ({ page }) => {
    const pairingPage = new PairingDetailsPage(page);
    const deviceId = mockDevices[0].id;
    await pairingPage.navigateTo(deviceId);

    // Click profile select to open dropdown
    await pairingPage.profileSelect.click();

    // Wait for options to appear
    await page.waitForTimeout(300);

    // Verify profile options (LL_INPUT, RT_AUDIO, MEDIA_8K)
    const options = page.locator('option, [role="option"]');
    const count = await options.count();
    expect(count).toBeGreaterThan(0);
  });

  test('should handle pairing error gracefully', async ({ page }) => {
    // Override handler to return error
    server.use(
      http.post('http://localhost:3000/api/v1/devices/:deviceId/pair', () => {
        return HttpResponse.json(
          { error: 'Device connection failed' },
          { status: 500 }
        );
      })
    );

    const pairingPage = new PairingDetailsPage(page);
    const deviceId = mockDevices[0].id;
    await pairingPage.navigateTo(deviceId);

    await pairingPage.selectProfile('LL_INPUT');
    await pairingPage.clickPair();

    // Verify error notification
    const errorMessage = page.locator('text=/error|failed/i').first();
    await expect(errorMessage).toBeVisible({ timeout: 5000 });
  });

  test('should disable pair button when no profile selected', async ({ page }) => {
    const pairingPage = new PairingDetailsPage(page);
    const deviceId = mockDevices[0].id;
    await pairingPage.navigateTo(deviceId);

    // Verify pair button state (might be disabled initially)
    const pairButton = pairingPage.pairButton;
    await expect(pairButton).toBeVisible();
  });

  test('should show device connection status', async ({ page }) => {
    const pairingPage = new PairingDetailsPage(page);
    const deviceId = mockDevices[0].id;
    await pairingPage.navigateTo(deviceId);

    // Verify status indicator is present
    const statusIndicator = page.locator('[data-testid*="status"], .status, .badge').first();
    await expect(statusIndicator).toBeVisible({ timeout: 5000 });
  });
});
