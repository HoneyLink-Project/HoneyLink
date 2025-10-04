import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright E2E Test Configuration
 *
 * Target: 5 workflows (WF-01 to WF-05) + i18n language switching
 * Strategy: Single browser (Chromium), headless, screenshots on failure
 * Dev Server: Vite (http://localhost:5173)
 */
export default defineConfig({
  testDir: './e2e',

  // Timeout settings (E2E tests can be slower)
  timeout: 30 * 1000, // 30s per test
  expect: {
    timeout: 5 * 1000, // 5s for assertions
  },

  // Fail fast on CI, retry locally
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,

  // Reporter configuration
  reporter: [
    ['html', { outputFolder: 'playwright-report' }],
    ['list'],
  ],

  // Shared settings for all tests
  use: {
    baseURL: 'http://localhost:5173',

    // Screenshots and traces for debugging
    screenshot: 'only-on-failure',
    trace: 'retain-on-failure',

    // Viewport (desktop)
    viewport: { width: 1280, height: 720 },

    // Ignore HTTPS errors (dev server)
    ignoreHTTPSErrors: true,
  },

  // Browser configuration (Chromium only for speed)
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
  ],

  // Dev server configuration
  webServer: {
    command: 'npm run dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000, // 2 minutes for Vite startup
  },
});
