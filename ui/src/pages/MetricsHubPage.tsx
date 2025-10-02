import { AlertTriangle, CheckCircle, Clock, Shield, TrendingUp, XCircle } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { CartesianGrid, Cell, ResponsiveContainer, Scatter, ScatterChart, Tooltip, XAxis, YAxis, ZAxis } from 'recharts';
import { useLatencyHeatmap, useMetrics } from '../api/hooks';
import { Card, CardContent, CardHeader, Select } from '../components/ui';

/**
 * KPI metric interface
 */
interface KPIMetric {
  id: string;
  label: string;
  value: string;
  unit?: string;
  target: string;
  status: 'good' | 'warning' | 'critical';
  change: number; // percentage change (positive/negative)
}

/**
 * Alert entry interface
 */
interface AlertEntry {
  timestamp: Date;
  type: 'latency_spike' | 'packet_loss' | 'fec_degradation' | 'bandwidth_limit' | 'auth_failure';
  severity: 'info' | 'warning' | 'error';
  details: string;
  device?: string;
  status: 'active' | 'acknowledged' | 'resolved';
}

/**
 * WF-05: Metrics Hub Page
 *
 * Features:
 * - Period/Role/Device filters
 * - KPI tiles (success rate, latency, FEC recovery)
 * - Heatmap visualization placeholder
 * - Alert list table with status
 *
 * TODO (Task 4.3 Part 3):
 * - Integrate with GET /metrics API
 * - Heatmap chart (recharts library)
 * - Real-time alert updates (WebSocket/SSE)
 * - Alert acknowledgment (PUT /alerts/{id}/acknowledge)
 */
export const MetricsHubPage = () => {
  const { t } = useTranslation();

  const [period, setPeriod] = useState('24h');
  const [role, setRole] = useState('all');
  const [deviceFilter, setDeviceFilter] = useState('all');

  // Fetch metrics from API (auto-refresh every 30s)
  const { data: metricsData } = useMetrics(period, role, deviceFilter);
  const { data: heatmapDataApi } = useLatencyHeatmap(period);

  // Period options
  const periodOptions = [
    { value: '1h', label: '過去1時間' },
    { value: '24h', label: '過去24時間' },
    { value: '7d', label: '過去7日' },
    { value: '30d', label: '過去30日' },
  ];

  // Role options
  const roleOptions = [
    { value: 'all', label: 'All Roles' },
    { value: 'user', label: 'End User' },
    { value: 'admin', label: 'Administrator' },
    { value: 'sre', label: 'SRE' },
  ];

  // Device filter options
  const deviceFilterOptions = [
    { value: 'all', label: 'All Devices' },
    { value: 'smartphone', label: 'Smartphone' },
    { value: 'tablet', label: 'Tablet' },
    { value: 'laptop', label: 'Laptop' },
    { value: 'iot', label: 'IoT Device' },
  ];

  // Use API data or fallback to mock data
  const mockKpis: KPIMetric[] = [
    {
      id: 'success_rate',
      label: '接続成功率',
      value: '99.6',
      unit: '%',
      target: '≥99%',
      status: 'good',
      change: 0.2,
    },
    {
      id: 'avg_latency',
      label: '平均遅延',
      value: '8',
      unit: 'ms',
      target: '≤10ms',
      status: 'good',
      change: -1.5,
    },
    {
      id: 'fec_recovery',
      label: 'FEC復元率',
      value: '99.9',
      unit: '%',
      target: '≥99.5%',
      status: 'good',
      change: 0.1,
    },
  ];

  const mockAlerts: AlertEntry[] = [
    {
      timestamp: new Date(Date.now() - 600000), // 10 min ago
      type: 'latency_spike',
      severity: 'warning',
      details: 'LL_INPUT latency 遅延スパイク検出 (15ms)',
      device: 'HL-EDGE-0001',
      status: 'active',
    },
    {
      timestamp: new Date(Date.now() - 1200000), // 20 min ago
      type: 'packet_loss',
      severity: 'warning',
      details: 'RT_AUDIO パケットロス 0.5% (閾値超過)',
      device: 'HL-EDGE-0002',
      status: 'acknowledged',
    },
    {
      timestamp: new Date(Date.now() - 1800000), // 30 min ago
      type: 'fec_degradation',
      severity: 'info',
      details: 'FEC率自動調整: 1/4 → 1/2',
      device: 'HL-EDGE-0003',
      status: 'resolved',
    },
  ];

  const kpis = metricsData?.kpis || mockKpis;
  const alerts = metricsData?.alerts || mockAlerts;
  const uptimeValue = metricsData?.uptime || 99.8;
  const mttrValue = metricsData?.mttr || 4.2;

  // Get KPI status icon
  const getKPIStatusIcon = (status: KPIMetric['status']) => {
    switch (status) {
      case 'good':
        return <CheckCircle size={20} className="text-success" />;
      case 'warning':
        return <AlertTriangle size={20} className="text-warning" />;
      case 'critical':
        return <XCircle size={20} className="text-error" />;
    }
  };

  // Get alert severity badge
  const getAlertSeverityBadge = (severity: AlertEntry['severity']) => {
    const config = {
      info: { label: t('metrics_hub.severity.info'), className: 'text-text-secondary bg-surface-alt dark:bg-surface-dark' },
      warning: { label: t('metrics_hub.severity.warning'), className: 'text-warning bg-warning/10' },
      error: { label: t('metrics_hub.severity.error'), className: 'text-error bg-error/10' },
    };
    const { label, className } = config[severity];
    return <span className={`px-2 py-1 rounded text-xs font-medium ${className}`}>{label}</span>;
  };

  // Get alert status badge
  const getAlertStatusBadge = (status: AlertEntry['status']) => {
    const config = {
      active: { label: t('metrics_hub.alert_status.active'), className: 'text-error bg-error/10' },
      acknowledged: { label: t('metrics_hub.alert_status.acknowledged'), className: 'text-warning bg-warning/10' },
      resolved: { label: t('metrics_hub.alert_status.resolved'), className: 'text-success bg-success/10' },
    };
    const { label, className } = config[status];
    return <span className={`px-2 py-1 rounded text-xs font-medium ${className}`}>{label}</span>;
  };

  // Format timestamp
  const formatTimestamp = (date: Date) => {
    return date.toLocaleString('ja-JP', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-display font-bold text-text-primary dark:text-text-dark">
          {t('metrics_hub.title')}
        </h1>
        <p className="text-sm text-text-secondary mt-1">
          {t('metrics_hub.subtitle')}
        </p>
      </div>

      {/* Filters */}
      <Card>
        <CardContent className="py-4">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Select
              label={t('metrics_hub.filters.time_range')}
              options={periodOptions}
              value={period}
              onChange={(e) => setPeriod(e.target.value)}
            />
            <Select
              label={t('metrics_hub.filters.role')}
              options={roleOptions}
              value={role}
              onChange={(e) => setRole(e.target.value)}
            />
            <Select
              label={t('metrics_hub.filters.device')}
              options={deviceFilterOptions}
              value={deviceFilter}
              onChange={(e) => setDeviceFilter(e.target.value)}
            />
          </div>
        </CardContent>
      </Card>

      {/* KPI Tiles */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {kpis.map((kpi) => (
          <Card key={kpi.id}>
            <CardContent className="py-6">
              <div className="flex items-start justify-between mb-4">
                {getKPIStatusIcon(kpi.status)}
                <div className={`text-xs font-medium ${kpi.change >= 0 ? 'text-success' : 'text-error'}`}>
                  {kpi.change >= 0 ? '↑' : '↓'} {Math.abs(kpi.change)}%
                </div>
              </div>
              <div className="text-sm text-text-secondary mb-1">{kpi.label}</div>
              <div className="flex items-baseline gap-1">
                <div className="text-3xl font-bold text-text-primary dark:text-text-dark">
                  {kpi.value}
                </div>
                {kpi.unit && (
                  <div className="text-lg text-text-secondary">{kpi.unit}</div>
                )}
              </div>
              <div className="text-xs text-text-secondary mt-2">
                Target: {kpi.target}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Heatmap Visualization */}
      <Card>
        <CardHeader
          title={t('metrics_hub.heatmap.title')}
          subtitle={t('metrics_hub.heatmap.subtitle')}
        />
        <CardContent>
          {heatmapDataApi && heatmapDataApi.length > 0 ? (
            <ResponsiveContainer width="100%" height={320}>
              <ScatterChart margin={{ top: 10, right: 20, bottom: 20, left: 40 }}>
                <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" opacity={0.3} />
                <XAxis
                  type="category"
                  dataKey="x"
                  name="Time"
                  stroke="#6b7280"
                  fontSize={12}
                  label={{ value: 'Time', position: 'insideBottom', offset: -10, style: { fill: '#6b7280', fontSize: 12 } }}
                />
                <YAxis
                  type="category"
                  dataKey="y"
                  name="Device"
                  stroke="#6b7280"
                  fontSize={12}
                  label={{ value: 'Device', angle: -90, position: 'insideLeft', style: { fill: '#6b7280', fontSize: 12 } }}
                />
                <ZAxis type="number" dataKey="value" range={[100, 500]} name="Latency (ms)" />
                <Tooltip
                  cursor={{ strokeDasharray: '3 3' }}
                  contentStyle={{
                    backgroundColor: '#ffffff',
                    border: '1px solid #e5e7eb',
                    borderRadius: '8px',
                    fontSize: '12px',
                  }}
                  formatter={(value: number) => [`${value.toFixed(1)} ms`, 'Latency']}
                />
                <Scatter data={heatmapDataApi}>
                  {heatmapDataApi.map((entry, index) => {
                    // Color scale: Green (0-8ms) -> Yellow (8-15ms) -> Red (15+ms)
                    const latency = entry.value;
                    let fillColor = '#10b981'; // green (good)
                    if (latency > 15) fillColor = '#ef4444'; // red (critical)
                    else if (latency > 8) fillColor = '#f59e0b'; // yellow/orange (warning)
                    return <Cell key={`cell-${index}`} fill={fillColor} />;
                  })}
                </Scatter>
              </ScatterChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-80 flex items-center justify-center bg-surface-alt/30 dark:bg-surface-dark/30 rounded">
              <div className="text-center text-text-secondary">
                <div className="text-sm font-medium mb-1">{t('metrics_hub.heatmap.no_data')}</div>
                <div className="text-xs">{t('metrics_hub.heatmap.waiting')}</div>
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Alert List */}
      <Card>
        <CardHeader
          title={t('metrics_hub.alerts.title')}
          subtitle={`${alerts.length} alerts in ${periodOptions.find((p) => p.value === period)?.label.toLowerCase()}`}
        />
        <CardContent className="p-0">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-surface-alt dark:bg-surface-dark border-b border-text-secondary/20">
                <tr>
                  <th className="text-left px-4 py-3 text-sm font-semibold text-text-primary dark:text-text-dark">
                    <div className="flex items-center gap-2">
                      <Clock size={16} />
                      {t('metrics_hub.alerts.timestamp')}
                    </div>
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-semibold text-text-primary dark:text-text-dark">
                    {t('metrics_hub.alerts.type')}
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-semibold text-text-primary dark:text-text-dark">
                    {t('metrics_hub.alerts.details')}
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-semibold text-text-primary dark:text-text-dark">
                    {t('metrics_hub.alerts.device')}
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-semibold text-text-primary dark:text-text-dark">
                    {t('metrics_hub.alerts.status')}
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-text-secondary/10">
                {alerts.map((alert, index) => (
                  <tr key={index} className="hover:bg-surface-alt/50 dark:hover:bg-surface-dark/30 transition-colors">
                    <td className="px-4 py-3 text-sm text-text-primary dark:text-text-dark font-mono whitespace-nowrap">
                      {formatTimestamp(alert.timestamp)}
                    </td>
                    <td className="px-4 py-3">
                      {getAlertSeverityBadge(alert.severity)}
                    </td>
                    <td className="px-4 py-3 text-sm text-text-primary dark:text-text-dark">
                      {alert.details}
                    </td>
                    <td className="px-4 py-3 text-sm text-text-secondary">
                      {alert.device || '-'}
                    </td>
                    <td className="px-4 py-3">
                      {getAlertStatusBadge(alert.status)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>

      {/* Summary Footer */}
      <Card>
        <CardContent className="py-4">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6 text-center">
            <div>
              <div className="flex items-center justify-center gap-2 mb-2">
                <TrendingUp className="text-success" size={20} />
                <span className="text-sm font-medium text-text-secondary">{t('metrics_hub.summary.active_alerts')}</span>
              </div>
              <div className="text-2xl font-bold text-error">
                {alerts.filter((a) => a.status === 'active').length}
              </div>
            </div>
            <div>
              <div className="flex items-center justify-center gap-2 mb-2">
                <Shield className="text-success" size={20} />
                <span className="text-sm font-medium text-text-secondary">{t('metrics_hub.summary.uptime')}</span>
              </div>
              <div className="text-2xl font-bold text-success">{uptimeValue.toFixed(1)}%</div>
            </div>
            <div>
              <div className="flex items-center justify-center gap-2 mb-2">
                <Clock className="text-text-secondary" size={20} />
                <span className="text-sm font-medium text-text-secondary">{t('metrics_hub.summary.mttr')}</span>
              </div>
              <div className="text-2xl font-bold text-text-primary dark:text-text-dark">{mttrValue.toFixed(1)} min</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};

// Default export for code splitting (React.lazy)
export default MetricsHubPage;
