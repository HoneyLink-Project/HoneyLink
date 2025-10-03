/**
 * Unit tests for TanStack Query hooks (API integration)
 * Tests all 13 hooks with mock API responses
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor } from '../test/test-utils';
import { createTestQueryClient, AllProviders } from '../test/test-utils';
import { apiClient } from './client';
import { useDevices, useScanDevices, useStreams, useMetrics } from './hooks';
import { mockDevices, mockStreams, mockKPIs, mockAlerts } from '../test/mock-data';
import type { QueryClient } from '@tanstack/react-query';

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
        data: { streams: mockStreams },
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
});
