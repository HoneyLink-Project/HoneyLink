/**
 * TanStack Query hooks for Control Plane API integration
 * 
 * Architecture:
 * - Centralized API calls with error handling
 * - Automatic refetching for real-time screens (WF-03, WF-05)
 * - Optimistic updates for mutations (WF-04 policy creation)
 * - Toast notifications on success/error (to be integrated in Part 4)
 */

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { apiClient } from './client';

// ============================================================================
// Type Definitions (matching backend Control Plane API)
// ============================================================================

export interface Device {
  id: string;
  name: string;
  type: 'smartphone' | 'tablet' | 'laptop' | 'iot' | 'other';
  signalStrength: number; // 1-5
  profiles: string[];
  lastSeen: Date;
  status: 'online' | 'offline' | 'paired';
}

export interface StreamStatus {
  id: string;
  profile: string;
  profileLabel: string;
  latency: number;
  jitter: number;
  packetLoss: number;
  bandwidth: number;
  fecRate: string;
  status: 'optimal' | 'degraded' | 'critical';
}

export interface PolicyTemplate {
  name: string;
  usage: string;
  latencyTarget: number;
  bandwidthMin: number;
  fecMode: 'NONE' | 'LIGHT' | 'MEDIUM' | 'HEAVY';
  scheduleStart: string;
  scheduleEnd: string;
  priority: number;
}

export interface KPIMetric {
  id: string;
  label: string;
  value: string;
  unit?: string;
  target: string;
  status: 'good' | 'warning' | 'critical';
  change: number;
}

export interface AlertEntry {
  timestamp: Date;
  type: string;
  severity: 'info' | 'warning' | 'error';
  details: string;
  device?: string;
  status: 'active' | 'acknowledged' | 'resolved';
}

export interface MetricsResponse {
  kpis: KPIMetric[];
  alerts: AlertEntry[];
  uptime: number;
  mttr: number; // Mean Time To Repair (minutes)
}

// ============================================================================
// WF-01: Device List Hooks
// ============================================================================

/**
 * Fetch all devices (GET /devices)
 * 
 * Refetch interval: 10s (device discovery)
 * Error handling: Returns empty array on failure
 */
export const useDevices = () => {
  return useQuery<Device[]>({
    queryKey: ['devices'],
    queryFn: async () => {
      try {
        const { data } = await apiClient.get<{ devices: Device[] }>('/devices');
        return data.devices.map((d: Device) => ({
          ...d,
          lastSeen: new Date(d.lastSeen),
        }));
      } catch (error) {
        console.error('[useDevices] Failed to fetch devices:', error);
        // Fallback to mock data on error (for development)
        return [];
      }
    },
    refetchInterval: 10000, // 10s auto-refresh
    staleTime: 5000, // Consider data fresh for 5s
  });
};

/**
 * Trigger device scan (POST /devices/scan)
 * 
 * Returns: Number of new devices found
 */
export const useScanDevices = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async () => {
      const { data } = await apiClient.post<{ count: number }>('/devices/scan');
      return data.count;
    },
    onSuccess: () => {
      // Invalidate devices query to refetch
      queryClient.invalidateQueries({ queryKey: ['devices'] });
    },
  });
};

// ============================================================================
// WF-02: Pairing Details Hooks
// ============================================================================

/**
 * Get device pairing details (GET /devices/:id)
 * 
 * Includes: Security status, session logs, current profile
 */
export const useDeviceDetails = (deviceId: string) => {
  return useQuery({
    queryKey: ['device', deviceId],
    queryFn: async () => {
      const { data } = await apiClient.get(`/devices/${deviceId}`);
      return data;
    },
    enabled: !!deviceId, // Only fetch if deviceId exists
  });
};

/**
 * Pair device with profile (POST /devices/:id/pair)
 * 
 * Request: { profileId: string, csr?: string }
 * Response: { sessionId: string, status: 'paired' }
 */
export const usePairDevice = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ deviceId, profileId }: { deviceId: string; profileId: string }) => {
      const { data } = await apiClient.post(`/devices/${deviceId}/pair`, { profileId });
      return data;
    },
    onSuccess: (_, variables) => {
      // Invalidate device details to show updated pairing status
      queryClient.invalidateQueries({ queryKey: ['device', variables.deviceId] });
    },
  });
};

/**
 * Unpair device (DELETE /devices/:id/pair)
 */
export const useUnpairDevice = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (deviceId: string) => {
      await apiClient.delete(`/devices/${deviceId}/pair`);
    },
    onSuccess: (_, deviceId) => {
      queryClient.invalidateQueries({ queryKey: ['device', deviceId] });
      queryClient.invalidateQueries({ queryKey: ['devices'] });
    },
  });
};

// ============================================================================
// WF-03: Stream Dashboard Hooks
// ============================================================================

/**
 * Fetch active streams (GET /sessions)
 * 
 * Refetch interval: 5s (real-time monitoring)
 * Returns: Array of StreamStatus with QoS metrics
 */
export const useStreams = () => {
  return useQuery<StreamStatus[]>({
    queryKey: ['streams'],
    queryFn: async () => {
      try {
        const { data } = await apiClient.get<{ sessions: StreamStatus[] }>('/sessions');
        return data.sessions;
      } catch (error) {
        console.error('[useStreams] Failed to fetch streams:', error);
        return [];
      }
    },
    refetchInterval: 5000, // 5s polling for real-time updates
    staleTime: 2000,
  });
};

/**
 * Update stream priority (PUT /sessions/:id/priority)
 * 
 * Request: { priority: number (1-5) }
 */
export const useUpdateStreamPriority = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async ({ streamId, priority }: { streamId: string; priority: number }) => {
      const { data } = await apiClient.put(`/sessions/${streamId}/priority`, { priority });
      return data;
    },
    onSuccess: () => {
      // Optimistic update: Refetch streams immediately
      queryClient.invalidateQueries({ queryKey: ['streams'] });
    },
  });
};

/**
 * Fetch stream metrics for chart (GET /sessions/:id/metrics)
 * 
 * Returns: Time-series data for recharts LineChart
 * Window: Last 5 minutes (300 data points @ 1s interval)
 */
export const useStreamMetrics = (streamId: string) => {
  return useQuery<{ timestamp: Date; latency: number; jitter: number }[]>({
    queryKey: ['stream-metrics', streamId],
    queryFn: async () => {
      const { data } = await apiClient.get(`/sessions/${streamId}/metrics`);
      return data.metrics.map((m: any) => ({
        ...m,
        timestamp: new Date(m.timestamp),
      }));
    },
    enabled: !!streamId,
    refetchInterval: 1000, // 1s real-time chart updates
    staleTime: 500,
  });
};

// ============================================================================
// WF-04: Policy Builder Hooks
// ============================================================================

/**
 * Create policy template (POST /policies)
 * 
 * Request: PolicyTemplate
 * Response: { id: string, ...PolicyTemplate }
 */
export const useCreatePolicy = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (template: PolicyTemplate) => {
      const { data } = await apiClient.post('/policies', template);
      return data;
    },
    onSuccess: () => {
      // Invalidate policies list (if exists on another screen)
      queryClient.invalidateQueries({ queryKey: ['policies'] });
      // TODO Part 4: toast.success('ポリシーテンプレートを保存しました')
    },
    onError: (error: any) => {
      console.error('[useCreatePolicy] Failed to save policy:', error);
      // TODO Part 4: toast.error('保存に失敗しました: ' + error.message)
    },
  });
};

/**
 * Fetch all policy templates (GET /policies)
 * 
 * For future policy management screen
 */
export const usePolicies = () => {
  return useQuery({
    queryKey: ['policies'],
    queryFn: async () => {
      const { data } = await apiClient.get('/policies');
      return data.policies;
    },
  });
};

// ============================================================================
// WF-05: Metrics Hub Hooks
// ============================================================================

/**
 * Fetch KPI metrics and alerts (GET /metrics)
 * 
 * Query params: period, role, deviceFilter
 * Refetch interval: 30s (dashboard monitoring)
 */
export const useMetrics = (
  period: string = '24h',
  role: string = 'all',
  deviceFilter: string = 'all'
) => {
  return useQuery<MetricsResponse>({
    queryKey: ['metrics', period, role, deviceFilter],
    queryFn: async () => {
      try {
        const { data } = await apiClient.get<MetricsResponse>('/metrics', {
          params: { period, role, device: deviceFilter },
        });
        // Parse alert timestamps
        const alerts = data.alerts.map((a: AlertEntry) => ({
          ...a,
          timestamp: new Date(a.timestamp),
        }));
        return { ...data, alerts };
      } catch (error) {
        console.error('[useMetrics] Failed to fetch metrics:', error);
        // Return empty metrics on error
        return {
          kpis: [],
          alerts: [],
          uptime: 0,
          mttr: 0,
        };
      }
    },
    refetchInterval: 30000, // 30s refresh (less frequent than streams)
    staleTime: 15000,
  });
};

/**
 * Acknowledge alert (PUT /alerts/:id/acknowledge)
 * 
 * Changes alert status: active → acknowledged
 */
export const useAcknowledgeAlert = () => {
  const queryClient = useQueryClient();
  
  return useMutation({
    mutationFn: async (alertId: string) => {
      await apiClient.put(`/alerts/${alertId}/acknowledge`);
    },
    onSuccess: () => {
      // Refetch metrics to update alert status
      queryClient.invalidateQueries({ queryKey: ['metrics'] });
    },
  });
};

/**
 * Fetch latency heatmap data (GET /metrics/heatmap)
 * 
 * For WF-05 recharts heatmap visualization
 * Returns: 2D array of {x: time, y: device, value: latency}
 */
export const useLatencyHeatmap = (period: string = '24h') => {
  return useQuery<{ x: string; y: string; value: number }[]>({
    queryKey: ['heatmap', period],
    queryFn: async () => {
      const { data } = await apiClient.get('/metrics/heatmap', {
        params: { period },
      });
      return data.heatmap;
    },
    refetchInterval: 60000, // 1min refresh (heatmap less real-time)
    staleTime: 30000,
  });
};
