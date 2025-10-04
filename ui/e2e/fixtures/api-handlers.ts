/**
 * MSW API Handlers for E2E Tests
 *
 * Mocks Control Plane REST API responses during E2E testing.
 * No actual network calls are made.
 */

import { http, HttpResponse } from 'msw';
import { mockAlerts, mockDevices, mockKPIs, mockProfiles, mockStreams } from './mock-data';

const API_BASE = 'http://localhost:3000/api/v1';

export const handlers = [
  // WF-01: Device scanning
  http.get(`${API_BASE}/devices`, () => {
    return HttpResponse.json({ devices: mockDevices });
  }),

  http.post(`${API_BASE}/devices/scan`, () => {
    return HttpResponse.json({ devices: mockDevices, count: mockDevices.length });
  }),

  // WF-02: Device pairing
  http.get(`${API_BASE}/devices/:deviceId`, ({ params }) => {
    const device = mockDevices.find((d) => d.id === params.deviceId);
    return device
      ? HttpResponse.json(device)
      : HttpResponse.json({ error: 'Device not found' }, { status: 404 });
  }),

  http.post(`${API_BASE}/devices/:deviceId/pair`, ({ params }) => {
    const device = mockDevices.find((d) => d.id === params.deviceId);
    return device
      ? HttpResponse.json({ success: true, deviceId: params.deviceId })
      : HttpResponse.json({ error: 'Device not found' }, { status: 404 });
  }),

  http.delete(`${API_BASE}/devices/:deviceId/pair`, ({ params }) => {
    return HttpResponse.json({ success: true, deviceId: params.deviceId });
  }),

  // WF-03: Stream management
  http.get(`${API_BASE}/sessions`, () => {
    return HttpResponse.json({ sessions: mockStreams });
  }),

  http.put(`${API_BASE}/sessions/:sessionId/priority`, async ({ params, request }) => {
    const body = (await request.json()) as { priority: number };
    return HttpResponse.json({
      success: true,
      sessionId: params.sessionId,
      priority: body.priority,
    });
  }),

  http.get(`${API_BASE}/sessions/:sessionId/metrics`, ({ params }) => {
    const stream = mockStreams.find((s) => s.id === params.sessionId);
    return stream
      ? HttpResponse.json({
          latency: stream.latency,
          jitter: stream.jitter,
          packetLoss: stream.packetLoss,
          bandwidth: stream.bandwidth,
        })
      : HttpResponse.json({ error: 'Session not found' }, { status: 404 });
  }),

  // WF-04: Policy management
  http.get(`${API_BASE}/policies`, () => {
    return HttpResponse.json({ policies: mockProfiles });
  }),

  http.post(`${API_BASE}/policies`, async ({ request }) => {
    const body = (await request.json()) as { name: string };
    return HttpResponse.json({
      id: `prof-${Date.now()}`,
      ...body,
      success: true,
    }, { status: 201 });
  }),

  http.put(`${API_BASE}/policies/:policyId`, async ({ params, request }) => {
    const body = (await request.json()) as Record<string, unknown>;
    return HttpResponse.json({
      id: params.policyId,
      ...body,
      success: true,
    });
  }),

  // WF-05: Metrics monitoring
  http.get(`${API_BASE}/metrics/kpis`, () => {
    return HttpResponse.json({ kpis: mockKPIs });
  }),

  http.get(`${API_BASE}/metrics/alerts`, () => {
    return HttpResponse.json({ alerts: mockAlerts });
  }),

  http.post(`${API_BASE}/metrics/alerts/:alertId/acknowledge`, ({ params }) => {
    return HttpResponse.json({ success: true, alertId: params.alertId });
  }),

  http.get(`${API_BASE}/metrics/latency/heatmap`, () => {
    // Generate dummy heatmap data (24h x 7 devices)
    const heatmapData = Array.from({ length: 24 }, (_, hour) =>
      mockDevices.map((device) => ({
        deviceId: device.id,
        deviceName: device.name,
        hour,
        latency: Math.floor(Math.random() * 30) + 5,
      }))
    ).flat();

    return HttpResponse.json({ heatmapData });
  }),
];
