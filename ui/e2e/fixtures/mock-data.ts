/**
 * E2E Test Fixtures - Mock Data
 * 
 * Provides consistent test data for all E2E workflows.
 * Aligned with spec/ui/mock-data.md
 */

// Inline type definitions for E2E tests
type Device = {
  id: string;
  name: string;
  type: string;
  rssi: number;
  profileIds: string[];
  status: string;
  lastSeen: string;
  capabilities: string[];
};

type Stream = {
  id: string;
  deviceId: string;
  deviceName: string;
  profileId: string;
  profileName: string;
  latency: number;
  jitter: number;
  packetLoss: number;
  bandwidth: number;
  fecRate: number;
  priority: number;
  status: string;
  duration: number;
};

type QoSProfile = {
  id: string;
  name: string;
  usage: string;
  latencyTarget: number;
  bandwidthMin: number;
  fecMode: string;
  scheduleStart: string;
  scheduleEnd: string;
  priority: number;
};

type KPI = {
  name: string;
  value: number;
  unit: string;
  target: number;
  trend: string;
  change: number;
};

type Alert = {
  id: string;
  severity: string;
  deviceId: string;
  deviceName: string;
  message: string;
  timestamp: string;
  acknowledged: boolean;
};

/**
 * Mock devices for device scanning and pairing tests
 */
export const mockDevices: Device[] = [
  {
    id: 'dev-001',
    name: 'iPhone 15 Pro',
    type: 'smartphone',
    rssi: -45,
    profileIds: ['prof-ll', 'prof-rt'],
    status: 'online',
    lastSeen: new Date().toISOString(),
    capabilities: ['BT5.3', 'WiFi6E'],
  },
  {
    id: 'dev-002',
    name: 'MacBook Pro M3',
    type: 'laptop',
    rssi: -38,
    profileIds: ['prof-8k'],
    status: 'paired',
    lastSeen: new Date(Date.now() - 300000).toISOString(),
    capabilities: ['BT5.3', 'WiFi6E', 'UWB'],
  },
  {
    id: 'dev-003',
    name: 'Galaxy Buds Pro',
    type: 'iot',
    rssi: -62,
    profileIds: ['prof-rt'],
    status: 'offline',
    lastSeen: new Date(Date.now() - 3600000).toISOString(),
    capabilities: ['BT5.2'],
  },
];

/**
 * Mock streams for stream dashboard tests
 */
export const mockStreams: Stream[] = [
  {
    id: 'stream-001',
    deviceId: 'dev-001',
    deviceName: 'iPhone 15 Pro',
    profileId: 'prof-ll',
    profileName: 'LL_INPUT',
    latency: 8,
    jitter: 1.2,
    packetLoss: 0.01,
    bandwidth: 25.5,
    fecRate: 0.05,
    priority: 1,
    status: 'optimal',
    duration: 3600,
  },
  {
    id: 'stream-002',
    deviceId: 'dev-002',
    deviceName: 'MacBook Pro M3',
    profileId: 'prof-8k',
    profileName: 'MEDIA_8K',
    latency: 15,
    jitter: 2.5,
    packetLoss: 0.02,
    bandwidth: 180.0,
    fecRate: 0.12,
    priority: 2,
    status: 'degraded',
    duration: 7200,
  },
];

/**
 * Mock QoS profiles for policy builder tests
 */
export const mockProfiles: QoSProfile[] = [
  {
    id: 'prof-ll',
    name: 'LL_INPUT',
    usage: 'low_latency',
    latencyTarget: 10,
    bandwidthMin: 20,
    fecMode: 'LIGHT',
    scheduleStart: '2025-01-01',
    scheduleEnd: '2025-12-31',
    priority: 1,
  },
  {
    id: 'prof-rt',
    name: 'RT_AUDIO',
    usage: 'realtime_audio',
    latencyTarget: 20,
    bandwidthMin: 50,
    fecMode: 'MEDIUM',
    scheduleStart: '2025-01-01',
    scheduleEnd: '2025-12-31',
    priority: 2,
  },
  {
    id: 'prof-8k',
    name: 'MEDIA_8K',
    usage: 'media_8k',
    latencyTarget: 50,
    bandwidthMin: 200,
    fecMode: 'HEAVY',
    scheduleStart: '2025-01-01',
    scheduleEnd: '2025-12-31',
    priority: 3,
  },
];

/**
 * Mock KPIs for metrics hub tests
 */
export const mockKPIs: KPI[] = [
  {
    name: 'Average Latency',
    value: 12.5,
    unit: 'ms',
    target: 15,
    trend: 'down',
    change: -2.3,
  },
  {
    name: 'Packet Loss',
    value: 0.015,
    unit: '%',
    target: 0.5,
    trend: 'stable',
    change: 0.001,
  },
  {
    name: 'Active Sessions',
    value: 12,
    unit: '',
    target: 20,
    trend: 'up',
    change: 3,
  },
];

/**
 * Mock alerts for metrics hub tests
 */
export const mockAlerts: Alert[] = [
  {
    id: 'alert-001',
    severity: 'critical',
    deviceId: 'dev-002',
    deviceName: 'MacBook Pro M3',
    message: 'Latency exceeded threshold',
    timestamp: new Date(Date.now() - 600000).toISOString(),
    acknowledged: false,
  },
  {
    id: 'alert-002',
    severity: 'warning',
    deviceId: 'dev-001',
    deviceName: 'iPhone 15 Pro',
    message: 'Packet loss detected',
    timestamp: new Date(Date.now() - 1200000).toISOString(),
    acknowledged: true,
  },
];
