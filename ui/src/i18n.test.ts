/**
 * Unit tests for i18n functionality
 * Tests language switching, interpolation, and fallback behavior
 */

import { describe, expect, it, beforeEach } from 'vitest';
import i18n from './i18n';

describe('i18n', () => {
  beforeEach(async () => {
    // Reset to Japanese before each test
    await i18n.changeLanguage('ja');
  });

  it('should initialize with Japanese as default language', () => {
    expect(i18n.language).toBe('ja');
  });

  it('should translate device list keys in Japanese', () => {
    expect(i18n.t('device_list.title')).toBe('近接デバイス');
    expect(i18n.t('device_list.scan_button')).toBe('デバイスをスキャン');
    expect(i18n.t('device_list.status.online')).toBe('オンライン');
  });

  it('should switch to English', async () => {
    await i18n.changeLanguage('en');
    expect(i18n.language).toBe('en');
    expect(i18n.t('device_list.title')).toBe('Nearby Devices');
    expect(i18n.t('device_list.scan_button')).toBe('Scan Devices');
  });

  it('should switch to Spanish', async () => {
    await i18n.changeLanguage('es');
    expect(i18n.language).toBe('es');
    expect(i18n.t('device_list.title')).toBe('Dispositivos Cercanos');
    expect(i18n.t('device_list.scan_button')).toBe('Escanear Dispositivos');
  });

  it('should switch to Chinese', async () => {
    await i18n.changeLanguage('zh');
    expect(i18n.language).toBe('zh');
    expect(i18n.t('device_list.title')).toBe('附近设备');
    expect(i18n.t('device_list.scan_button')).toBe('扫描设备');
  });

  it('should interpolate variables', async () => {
    await i18n.changeLanguage('en');
    const result = i18n.t('device_list.toast.scan_success', { count: 3 });
    expect(result).toContain('3');
  });

  it('should handle missing keys with fallback', () => {
    const result = i18n.t('non.existent.key');
    expect(result).toBe('non.existent.key'); // Returns key as fallback
  });

  it('should translate pairing page keys', async () => {
    await i18n.changeLanguage('ja');
    expect(i18n.t('pairing.title')).toBe('ペアリング詳細');
    expect(i18n.t('pairing.security_status')).toBe('セキュリティステータス');
    expect(i18n.t('pairing.disconnect')).toBe('切断');
  });

  it('should translate stream dashboard keys', async () => {
    await i18n.changeLanguage('ja');
    expect(i18n.t('stream_dashboard.title')).toContain('Stream');
    expect(i18n.t('stream_dashboard.metrics.latency')).toBe('Latency');
    expect(i18n.t('stream_dashboard.chart.jitter')).toBe('ジッタ (ms)');
  });

  it('should translate metrics hub keys', async () => {
    await i18n.changeLanguage('ja');
    expect(i18n.t('metrics_hub.title')).toBe('メトリクスハブ');
    expect(i18n.t('metrics_hub.subtitle')).toContain('システム全体');
    expect(i18n.t('metrics_hub.kpis.title')).toBe('主要KPI');
  });

  it('should translate policy builder keys', async () => {
    await i18n.changeLanguage('ja');
    expect(i18n.t('policy_builder.title')).toContain('Policy');
    expect(i18n.t('policy_builder.buttons.save')).toBe('保存');
    expect(i18n.t('policy_builder.buttons.close')).toBe('閉じる');
  });

  it('should handle nested translation keys', async () => {
    await i18n.changeLanguage('en');
    expect(i18n.t('pairing.toast.pair_success')).toContain('paired');
    expect(i18n.t('pairing.toast.pair_error')).toContain('failed');
  });

  it('should persist language across calls', async () => {
    await i18n.changeLanguage('es');
    expect(i18n.t('device_list.title')).toBe('Dispositivos Cercanos');

    // Language should persist
    expect(i18n.t('pairing.title')).toContain('Detalles');
    expect(i18n.language).toBe('es');
  });  it('should translate FEC mode labels', async () => {
    await i18n.changeLanguage('ja');
    expect(i18n.t('policy_builder.fec_modes.none')).toBe('なし (NONE)');
    expect(i18n.t('policy_builder.fec_modes.light')).toBe('軽 (LIGHT - 1/4)');
    expect(i18n.t('policy_builder.fec_modes.medium')).toBe('中 (MEDIUM - 1/2)');
    expect(i18n.t('policy_builder.fec_modes.heavy')).toBe('重 (HEAVY - 3/4)');
  });

  it('should translate alert severity labels', async () => {
    await i18n.changeLanguage('en');
    expect(i18n.t('metrics_hub.severity.info')).toBe('Info');
    expect(i18n.t('metrics_hub.severity.warning')).toBe('Warning');
    expect(i18n.t('metrics_hub.severity.error')).toBe('Error');
  });

  it('should return consistent results for same key', () => {
    const result1 = i18n.t('device_list.title');
    const result2 = i18n.t('device_list.title');
    expect(result1).toBe(result2);
  });
});
