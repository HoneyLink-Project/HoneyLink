/**
 * Mock data for testing
 * Provides consistent test fixtures for API responses
 */

import type { Device, StreamStatus, PolicyTemplate, KPIMetric, AlertEntry } from '../api/hooks';

/**
 * Mock devices for testing
 */
export const mockDevices: Device[] = [
  {
    id: 'dev-001',
    name: "Aqua's Laptop",
    type: 'laptop',
    signalStrength: 5,
    profiles: ['LL_INPUT', 'MEDIA_OUTPUT'],
    lastSeen: new Date(),
    status: 'online',
  },
  {
    id: 'dev-002',
    name: 'Gaming Desktop',
    type: 'laptop',
    signalStrength: 4,
    profiles: ['LL_INPUT'],
    lastSeen: new Date(Date.now() - 300000),
    status: 'paired',
  },
];

/**
 * Mock stream statuses for testing
 */
export const mockStreams: StreamStatus[] = [
  {
    id: 'stream-001',
    profile: 'LL_INPUT',
    profileLabel: 'Low Latency Input',
    latency: 8.5,
    jitter: 1.2,
    packetLoss: 0.01,
    bandwidth: 250,
    fecRate: '10%',
    status: 'optimal',
  },
  {
    id: 'stream-002',
    profile: 'MEDIA_OUTPUT',
    profileLabel: 'Media Output',
    latency: 15.0,
    jitter: 2.5,
    packetLoss: 0.02,
    bandwidth: 800,
    fecRate: '15%',
    status: 'optimal',
  },
];

/**
 * Mock KPI metrics for testing
 */
export const mockKPIs: KPIMetric[] = [
  {
    id: 'latency',
    label: 'Avg Latency',
    value: '8.5',
    unit: 'ms',
    target: '< 10 ms',
    status: 'good',
    change: -5,
  },
  {
    id: 'jitter',
    label: 'Avg Jitter',
    value: '1.2',
    unit: 'ms',
    target: '< 2 ms',
    status: 'good',
    change: 0,
  },
  {
    id: 'packet-loss',
    label: 'Packet Loss',
    value: '0.01',
    unit: '%',
    target: '< 0.1%',
    status: 'good',
    change: 0,
  },
];

/**
 * Mock alerts for testing
 */
export const mockAlerts: AlertEntry[] = [
  {
    timestamp: new Date(Date.now() - 300000),
    type: 'Latency Spike',
    severity: 'warning',
    details: 'Latency exceeded 15ms on stream-001',
    device: 'dev-001',
    status: 'active',
  },
  {
    timestamp: new Date(Date.now() - 600000),
    type: 'Device Disconnected',
    severity: 'error',
    details: 'Device dev-003 lost connection',
    device: 'dev-003',
    status: 'acknowledged',
  },
];

/**
 * Mock policy templates for testing
 */
export const mockPolicies: PolicyTemplate[] = [
  {
    name: 'Low Latency Gaming',
    usage: 'low_latency',
    latencyTarget: 10,
    bandwidthMin: 200,
    fecMode: 'LIGHT',
    scheduleStart: '2025-01-01',
    scheduleEnd: '2025-12-31',
    priority: 1,
  },
];

/**
 * Mock error responses for testing error states
 */
export const mockErrorResponse = {
  message: 'Internal Server Error',
  status: 500,
};

export const mockNotFoundResponse = {
  message: 'Resource not found',
  status: 404,
};

export const mockValidationErrorResponse = {
  message: 'Validation failed',
  errors: {
    name: 'Name is required',
    latencyTarget: 'Latency must be between 1 and 50',
  },
  status: 400,
};
