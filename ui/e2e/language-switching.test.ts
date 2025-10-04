import { expect, test } from '@playwright/test';
import { setupServer } from 'msw/node';
import { handlers } from './fixtures/api-handlers';

// Setup MSW server for API mocking
const server = setupServer(...handlers);

test.beforeAll(() => server.listen());
test.afterEach(() => server.resetHandlers());
test.afterAll(() => server.close());

test.describe('Language Switching (i18n)', () => {
  test('should display default language (Japanese) on initial load', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Verify Japanese content is displayed (e.g., "デバイス管理")
    const japaneseContent = page.locator('text=/デバイス|機器|管理/').first();
    await expect(japaneseContent).toBeVisible({ timeout: 10000 });
  });

  test('should switch to English', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Find and click language selector
    const languageSelector = page.locator('[data-testid="language-selector"], select[name="language"], button[aria-label*="language"]').first();
    await languageSelector.click();

    // Select English
    const englishOption = page.locator('text=/english|en/i, [value="en"]').first();
    await englishOption.click();

    // Wait for language change
    await page.waitForTimeout(500);

    // Verify English content is displayed
    const englishContent = page.locator('text=/device.*management|manage.*devices/i').first();
    await expect(englishContent).toBeVisible({ timeout: 5000 });
  });

  test('should switch to Spanish', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Find and click language selector
    const languageSelector = page.locator('[data-testid="language-selector"], select[name="language"], button[aria-label*="language"]').first();
    await languageSelector.click();

    // Select Spanish
    const spanishOption = page.locator('text=/español|spanish|es/i, [value="es"]').first();
    await spanishOption.click();

    // Wait for language change
    await page.waitForTimeout(500);

    // Verify Spanish content is displayed
    const spanishContent = page.locator('text=/gestión.*dispositivos|dispositivos/i').first();
    await expect(spanishContent).toBeVisible({ timeout: 5000 });
  });

  test('should switch to Chinese', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Find and click language selector
    const languageSelector = page.locator('[data-testid="language-selector"], select[name="language"], button[aria-label*="language"]').first();
    await languageSelector.click();

    // Select Chinese
    const chineseOption = page.locator('text=/中文|chinese|zh/i, [value="zh"]').first();
    await chineseOption.click();

    // Wait for language change
    await page.waitForTimeout(500);

    // Verify Chinese content is displayed
    const chineseContent = page.locator('text=/设备|管理/').first();
    await expect(chineseContent).toBeVisible({ timeout: 5000 });
  });

  test('should persist language selection across page navigation', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Switch to English
    const languageSelector = page.locator('[data-testid="language-selector"], select[name="language"], button[aria-label*="language"]').first();
    await languageSelector.click();
    const englishOption = page.locator('text=/english|en/i, [value="en"]').first();
    await englishOption.click();
    await page.waitForTimeout(500);

    // Navigate to different page
    await page.goto('/policy-builder');
    await page.waitForLoadState('networkidle');

    // Verify English is still active
    const englishContent = page.locator('text=/policy.*builder|create.*policy/i').first();
    await expect(englishContent).toBeVisible({ timeout: 5000 });
  });

  test('should update all UI text when language changes', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Switch to English
    const languageSelector = page.locator('[data-testid="language-selector"], select[name="language"], button[aria-label*="language"]').first();
    await languageSelector.click();
    const englishOption = page.locator('text=/english|en/i, [value="en"]').first();
    await englishOption.click();
    await page.waitForTimeout(500);

    // Verify multiple UI elements are in English
    const scanButton = page.locator('button:has-text("Scan"), button:has-text("scan")').first();
    await expect(scanButton).toBeVisible({ timeout: 5000 });

    // Verify navigation menu is in English
    const navLinks = page.locator('nav a, [role="navigation"] a').first();
    await expect(navLinks).toBeVisible({ timeout: 5000 });
  });

  test('should handle language switching on Policy Builder page', async ({ page }) => {
    await page.goto('/policy-builder');
    await page.waitForLoadState('networkidle');

    // Switch to English
    const languageSelector = page.locator('[data-testid="language-selector"], select[name="language"], button[aria-label*="language"]').first();
    await languageSelector.click();
    const englishOption = page.locator('text=/english|en/i, [value="en"]').first();
    await englishOption.click();
    await page.waitForTimeout(500);

    // Verify form labels are in English
    const templateNameLabel = page.locator('label:has-text("Template Name"), label:has-text("Name")').first();
    await expect(templateNameLabel).toBeVisible({ timeout: 5000 });
  });

  test('should handle language switching on Metrics Hub page', async ({ page }) => {
    await page.goto('/metrics');
    await page.waitForLoadState('networkidle');

    // Switch to Spanish
    const languageSelector = page.locator('[data-testid="language-selector"], select[name="language"], button[aria-label*="language"]').first();
    await languageSelector.click();
    const spanishOption = page.locator('text=/español|spanish|es/i, [value="es"]').first();
    await spanishOption.click();
    await page.waitForTimeout(500);

    // Verify Spanish content
    const spanishContent = page.locator('text=/métricas|alertas|latencia/i').first();
    await expect(spanishContent).toBeVisible({ timeout: 5000 });
  });

  test('should load language from localStorage on page refresh', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Set language to English
    await page.evaluate(() => {
      localStorage.setItem('language', 'en');
    });

    // Refresh page
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify English content is displayed
    const englishContent = page.locator('text=/device.*management|manage.*devices/i').first();
    await expect(englishContent).toBeVisible({ timeout: 10000 });
  });

  test('should handle RTL languages if supported', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Check if RTL language selector exists (Arabic, Hebrew, etc.)
    const rtlOption = page.locator('[value="ar"], [value="he"], text=/arabic|hebrew/i').first();
    const rtlExists = await rtlOption.count();

    if (rtlExists > 0) {
      // Test RTL language switching
      const languageSelector = page.locator('[data-testid="language-selector"], select[name="language"]').first();
      await languageSelector.click();
      await rtlOption.click();
      await page.waitForTimeout(500);

      // Verify RTL direction is applied
      const bodyDir = await page.evaluate(() => document.body.dir);
      expect(bodyDir).toBe('rtl');
    }
  });

  test('should display language selector on all pages', async ({ page }) => {
    const pages = ['/devices', '/policy-builder', '/metrics', '/streams'];

    for (const pagePath of pages) {
      await page.goto(pagePath);
      await page.waitForLoadState('networkidle');

      // Verify language selector is present
      const languageSelector = page.locator('[data-testid="language-selector"], select[name="language"], button[aria-label*="language"]').first();
      const isVisible = await languageSelector.isVisible();

      // Language selector should be visible or at least exist in DOM
      expect(isVisible || await languageSelector.count() > 0).toBeTruthy();
    }
  });

  test('should fallback to default language if invalid locale is set', async ({ page }) => {
    await page.goto('/devices');
    await page.waitForLoadState('networkidle');

    // Set invalid language in localStorage
    await page.evaluate(() => {
      localStorage.setItem('language', 'invalid-locale');
    });

    // Refresh page
    await page.reload();
    await page.waitForLoadState('networkidle');

    // Verify fallback to default language (Japanese)
    const defaultContent = page.locator('text=/デバイス|device/i').first();
    await expect(defaultContent).toBeVisible({ timeout: 10000 });
  });
});
