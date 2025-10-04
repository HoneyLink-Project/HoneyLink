/**
 * Page Object Models for E2E Tests
 * 
 * Encapsulates page interactions and selectors for maintainability.
 */

import { type Page, type Locator } from '@playwright/test';

/**
 * Base Page class with common functionality
 */
export class BasePage {
  constructor(protected page: Page) {}

  async goto(path: string) {
    await this.page.goto(path);
    await this.page.waitForLoadState('networkidle');
  }

  async waitForHeading(text: string | RegExp) {
    await this.page.locator('h1').filter({ hasText: text }).waitFor();
  }
}

/**
 * Device List Page (WF-01)
 */
export class DeviceListPage extends BasePage {
  readonly scanButton: Locator;
  readonly deviceCards: Locator;
  readonly searchInput: Locator;

  constructor(page: Page) {
    super(page);
    this.scanButton = page.getByRole('button', { name: /scan/i });
    this.deviceCards = page.locator('[data-testid="device-card"]');
    this.searchInput = page.getByPlaceholder(/search/i);
  }

  async navigateTo() {
    await this.goto('/devices');
  }

  async clickScanDevices() {
    await this.scanButton.click();
  }

  async searchDevices(query: string) {
    await this.searchInput.fill(query);
  }

  async getDeviceCount() {
    return await this.deviceCards.count();
  }

  async clickDevice(deviceName: string) {
    await this.page.getByText(deviceName).first().click();
  }
}

/**
 * Pairing Details Page (WF-02)
 */
export class PairingDetailsPage extends BasePage {
  readonly pairButton: Locator;
  readonly unpairButton: Locator;
  readonly profileSelect: Locator;
  readonly deviceName: Locator;

  constructor(page: Page) {
    super(page);
    this.deviceName = page.locator('h1');
    this.profileSelect = page.getByLabel(/qos profile/i);
    this.pairButton = page.getByRole('button', { name: /add stream/i });
    this.unpairButton = page.getByRole('button', { name: /disconnect/i });
  }

  async navigateTo(deviceId: string) {
    await this.goto(`/devices/${deviceId}/pair`);
  }

  async selectProfile(profileName: string) {
    await this.profileSelect.selectOption(profileName);
  }

  async clickPair() {
    await this.pairButton.click();
  }

  async clickUnpair() {
    await this.unpairButton.click();
  }
}

/**
 * Stream Dashboard Page (WF-03)
 */
export class StreamDashboardPage extends BasePage {
  readonly streamCards: Locator;
  readonly priorityButtons: Locator;

  constructor(page: Page) {
    super(page);
    this.streamCards = page.locator('[data-testid="stream-card"]');
    this.priorityButtons = page.getByRole('button', { name: /priority/i });
  }

  async navigateTo() {
    await this.goto('/streams');
  }

  async getStreamCount() {
    return await this.streamCards.count();
  }

  async increasePriority(streamId: string) {
    const card = this.page.locator(`[data-stream-id="${streamId}"]`);
    await card.getByRole('button', { name: /increase/i }).click();
  }
}

/**
 * Policy Builder Page (WF-04)
 */
export class PolicyBuilderPage extends BasePage {
  readonly nameInput: Locator;
  readonly usageSelect: Locator;
  readonly latencyInput: Locator;
  readonly bandwidthInput: Locator;
  readonly fecSelect: Locator;
  readonly prioritySelect: Locator;
  readonly saveButton: Locator;
  readonly previewButton: Locator;

  constructor(page: Page) {
    super(page);
    this.nameInput = page.getByLabel(/template name/i);
    this.usageSelect = page.getByLabel(/usage type/i);
    this.latencyInput = page.getByLabel(/latency target/i);
    this.bandwidthInput = page.getByLabel(/minimum bandwidth/i);
    this.fecSelect = page.getByLabel(/fec mode/i);
    this.prioritySelect = page.getByLabel(/priority/i);
    this.saveButton = page.getByRole('button', { name: /save/i });
    this.previewButton = page.getByRole('button', { name: /preview/i });
  }

  async navigateTo() {
    await this.goto('/policies');
  }

  async fillPolicyForm(data: {
    name: string;
    usage?: string;
    latency?: string;
    bandwidth?: string;
    fecMode?: string;
    priority?: string;
  }) {
    await this.nameInput.fill(data.name);
    if (data.usage) await this.usageSelect.selectOption(data.usage);
    if (data.latency) await this.latencyInput.fill(data.latency);
    if (data.bandwidth) await this.bandwidthInput.fill(data.bandwidth);
    if (data.fecMode) await this.fecSelect.selectOption(data.fecMode);
    if (data.priority) await this.prioritySelect.selectOption(data.priority);
  }

  async clickSave() {
    await this.saveButton.click();
  }

  async clickPreview() {
    await this.previewButton.click();
  }
}

/**
 * Metrics Hub Page (WF-05)
 */
export class MetricsHubPage extends BasePage {
  readonly kpiCards: Locator;
  readonly alertList: Locator;
  readonly acknowledgeButtons: Locator;
  readonly heatmap: Locator;

  constructor(page: Page) {
    super(page);
    this.kpiCards = page.locator('[data-testid="kpi-card"]');
    this.alertList = page.locator('[data-testid="alert-item"]');
    this.acknowledgeButtons = page.getByRole('button', { name: /acknowledge/i });
    this.heatmap = page.locator('[data-testid="latency-heatmap"]');
  }

  async navigateTo() {
    await this.goto('/metrics');
  }

  async getKPICount() {
    return await this.kpiCards.count();
  }

  async getAlertCount() {
    return await this.alertList.count();
  }

  async acknowledgeFirstAlert() {
    await this.acknowledgeButtons.first().click();
  }
}
