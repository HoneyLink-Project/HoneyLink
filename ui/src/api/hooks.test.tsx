/**
 * Unit tests for TanStack Query hooks (API integration)
 * Tests all 13 hooks with mock API responses
 */

import type { QueryClient } from '@tanstack/react-query';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { mockAlerts, mockDevices, mockKPIs, mockStreams } from '../test/mock-data';
import { AllProviders, createTestQueryClient, renderHook, waitFor } from '../test/test-utils';
import { apiClient } from './client';
import {
  useAcknowledgeAlert,
  useCreatePolicy,
  useDeviceDetails,
  useDevices,
  useLatencyHeatmap,
  useMetrics,
  usePairDevice,
  useScanDevices,
  useStreamMetrics,
  useStreams,
  useUnpairDevice,
  useUpdatePolicy,
  useUpdateStreamPriority,
} from './hooks';

// Mock axios
vi.mock('./client', () => ({
  apiClient: {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
  },
}));

describe('API Hooks', () => {
  let queryClient: QueryClient;

  beforeEach(() => {
    queryClient = createTestQueryClient();
    vi.clearAllMocks();
  });

  afterEach(() => {
    queryClient.clear();
  });

  describe('useDevices', () => {
    it('should fetch devices successfully', async () => {
      vi.mocked(apiClient.get).mockResolvedValueOnce({
        data: { devices: mockDevices },
      });

      const { result } = renderHook(() => useDevices(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
      expect(result.current.data).toHaveLength(2);
      expect(result.current.data?.[0].id).toBe('dev-001');
    });

    it('should handle fetch error gracefully', async () => {
      vi.mocked(apiClient.get).mockRejectedValueOnce(new Error('Network error'));

      const { result } = renderHook(() => useDevices(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
      expect(result.current.data).toEqual([]);
    });
  });

  describe('useScanDevices', () => {
    it('should scan devices successfully', async () => {
      vi.mocked(apiClient.post).mockResolvedValueOnce({
        data: { count: 3 },
      });

      const { result } = renderHook(() => useScanDevices(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      result.current.mutate();

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
      expect(result.current.data).toBe(3);
    });
  });

  describe('useStreams', () => {
    it('should fetch streams successfully', async () => {
      vi.mocked(apiClient.get).mockResolvedValueOnce({
        data: { sessions: mockStreams },
      });

      const { result } = renderHook(() => useStreams(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
      expect(result.current.data).toHaveLength(2);
      expect(result.current.data?.[0].profile).toBe('LL_INPUT');
    });
  });

  describe('useMetrics', () => {
    it('should fetch metrics successfully', async () => {
      vi.mocked(apiClient.get).mockResolvedValueOnce({
        data: {
          kpis: mockKPIs,
          alerts: mockAlerts,
          uptime: 99.9,
          mttr: 5,
        },
      });

      const { result } = renderHook(() => useMetrics(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
      expect(result.current.data?.kpis).toHaveLength(3);
      expect(result.current.data?.alerts).toHaveLength(2);
      expect(result.current.data?.uptime).toBe(99.9);
    });
  });

  describe('useDeviceDetails', () => {
    it('should fetch device details', async () => {
      vi.mocked(apiClient.get).mockResolvedValueOnce({
        data: { ...mockDevices[0], securityStatus: 'verified' },
      });

      const { result } = renderHook(() => useDeviceDetails('dev-001'), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
      expect(result.current.data?.id).toBe('dev-001');
    });
  });

  describe('usePairDevice', () => {
    it('should pair device successfully', async () => {
      vi.mocked(apiClient.post).mockResolvedValueOnce({
        data: { sessionId: 'session-123', status: 'paired' },
      });

      const { result } = renderHook(() => usePairDevice(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      result.current.mutate({ deviceId: 'dev-001', profileId: 'LL_OUTPUT' });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
      expect(result.current.data?.sessionId).toBe('session-123');
    });
  });

  describe('useUnpairDevice', () => {
    it('should unpair device successfully', async () => {
      vi.mocked(apiClient.delete).mockResolvedValueOnce({ data: {} });

      const { result } = renderHook(() => useUnpairDevice(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      result.current.mutate('dev-001');

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });
  });

  describe('useUpdateStreamPriority', () => {
    it('should update stream priority', async () => {
      vi.mocked(apiClient.put).mockResolvedValueOnce({
        data: { success: true },
      });

      const { result } = renderHook(() => useUpdateStreamPriority(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      result.current.mutate({ streamId: 'stream-001', priority: 5 });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });
  });

  describe('useStreamMetrics', () => {
    it('should fetch stream metrics', async () => {
      vi.mocked(apiClient.get).mockResolvedValueOnce({
        data: {
          metrics: [
            { timestamp: '2025-01-01T00:00:00Z', latency: 10, jitter: 2 },
            { timestamp: '2025-01-01T00:00:01Z', latency: 12, jitter: 3 },
          ],
        },
      });

      const { result } = renderHook(() => useStreamMetrics('stream-001'), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
      expect(result.current.data).toHaveLength(2);
      expect(result.current.data?.[0].latency).toBe(10);
    });
  });

  describe('useCreatePolicy', () => {
    it('should create policy successfully', async () => {
      vi.mocked(apiClient.post).mockResolvedValueOnce({
        data: { id: 'policy-001', name: 'Low Latency Gaming' },
      });

      const { result } = renderHook(() => useCreatePolicy(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      result.current.mutate({
        name: 'Low Latency Gaming',
        usage: 'gaming',
        latencyTarget: 10,
        bandwidthMin: 100,
        fecMode: 'HEAVY',
        scheduleStart: '18:00',
        scheduleEnd: '23:00',
        priority: 5,
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
      expect(result.current.data?.id).toBe('policy-001');
    });
  });

  describe('useUpdatePolicy', () => {
    it('should update policy successfully', async () => {
      vi.mocked(apiClient.put).mockResolvedValueOnce({
        data: { id: 'policy-001', name: 'Updated Policy' },
      });

      const { result } = renderHook(() => useUpdatePolicy(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      result.current.mutate({
        id: 'policy-001',
        template: {
          name: 'Updated Policy',
          usage: 'gaming',
          latencyTarget: 8,
          bandwidthMin: 150,
          fecMode: 'HEAVY',
          scheduleStart: '18:00',
          scheduleEnd: '23:00',
          priority: 5,
        },
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });
  });

  describe('useAcknowledgeAlert', () => {
    it('should acknowledge alert successfully', async () => {
      vi.mocked(apiClient.put).mockResolvedValueOnce({ data: {} });

      const { result } = renderHook(() => useAcknowledgeAlert(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      result.current.mutate('alert-001');

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
    });
  });

  describe('useLatencyHeatmap', () => {
    it('should fetch heatmap data', async () => {
      vi.mocked(apiClient.get).mockResolvedValueOnce({
        data: {
          heatmap: [
            { x: '00:00', y: 'Device1', value: 10 },
            { x: '00:01', y: 'Device1', value: 12 },
          ],
        },
      });

      const { result } = renderHook(() => useLatencyHeatmap(), {
        wrapper: ({ children }: { children: React.ReactNode }) => (
          <AllProviders queryClient={queryClient}>{children}</AllProviders>
        ),
      });

      await waitFor(() => expect(result.current.isSuccess).toBe(true));
      expect(result.current.data).toHaveLength(2);
      expect(result.current.data?.[0].value).toBe(10);
    });
  });
});
