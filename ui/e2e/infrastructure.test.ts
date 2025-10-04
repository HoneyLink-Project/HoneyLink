/**
 * Playwright E2E Test Example
 * 
 * This file verifies the test infrastructure is working.
 * Will be replaced with actual workflow tests.
 */
import { test, expect } from '@playwright/test';

test('infrastructure check - should load app', async ({ page }) => {
  // Navigate to devices page directly (root redirect happens client-side)
  await page.goto('/devices');
  
  // Wait for page to load (React app hydration)
  await page.waitForLoadState('networkidle');
  
  // Check if main heading is visible (shows app loaded)
  const heading = page.locator('h1').first();
  await expect(heading).toBeVisible({ timeout: 10000 });
});
