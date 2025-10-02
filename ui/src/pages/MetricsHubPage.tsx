import { AlertTriangle, BarChart3, CheckCircle, Clock, Shield, TrendingUp, XCircle } from 'lucide-react';
import { useState } from 'react';
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
  const [period, setPeriod] = useState('24h');
  const [role, setRole] = useState('all');
  const [deviceFilter, setDeviceFilter] = useState('all');

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

  // Mock KPI data (TODO: Fetch from API)
  const [kpis] = useState<KPIMetric[]>([
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
  ]);

  // Mock alert data (TODO: Fetch from API)
  const [alerts] = useState<AlertEntry[]>([
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
  ]);

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
      info: { label: 'Info', className: 'text-text-secondary bg-surface-alt dark:bg-surface-dark' },
      warning: { label: 'Warning', className: 'text-warning bg-warning/10' },
      error: { label: 'Error', className: 'text-error bg-error/10' },
    };
    const { label, className } = config[severity];
    return <span className={`px-2 py-1 rounded text-xs font-medium ${className}`}>{label}</span>;
  };

  // Get alert status badge
  const getAlertStatusBadge = (status: AlertEntry['status']) => {
    const config = {
      active: { label: 'Active', className: 'text-error bg-error/10' },
      acknowledged: { label: 'Ack', className: 'text-warning bg-warning/10' },
      resolved: { label: 'Resolved', className: 'text-success bg-success/10' },
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
          Metrics Hub
        </h1>
        <p className="text-sm text-text-secondary mt-1">
          Real-time KPI monitoring and alert management
        </p>
      </div>

      {/* Filters */}
      <Card>
        <CardContent className="py-4">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <Select
              label="期間"
              options={periodOptions}
              value={period}
              onChange={(e) => setPeriod(e.target.value)}
            />
            <Select
              label="ロール"
              options={roleOptions}
              value={role}
              onChange={(e) => setRole(e.target.value)}
            />
            <Select
              label="デバイス"
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

      {/* Heatmap Visualization Placeholder */}
      <Card>
        <CardHeader
          title="可視化"
          subtitle="Latency heatmap over time"
        />
        <CardContent>
          <div className="h-80 flex items-center justify-center bg-surface-alt/30 dark:bg-surface-dark/30 rounded border-2 border-dashed border-text-secondary/30">
            <div className="text-center">
              <BarChart3 size={48} className="mx-auto text-text-secondary mb-3" />
              <div className="text-sm font-medium text-text-primary dark:text-text-dark mb-1">
                Heatmap visualization
              </div>
              <div className="text-xs text-text-secondary">
                TODO: Integrate recharts heatmap for latency distribution
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Alert List */}
      <Card>
        <CardHeader
          title="アラート一覧"
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
                      時刻
                    </div>
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-semibold text-text-primary dark:text-text-dark">
                    種別
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-semibold text-text-primary dark:text-text-dark">
                    詳細
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-semibold text-text-primary dark:text-text-dark">
                    デバイス
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-semibold text-text-primary dark:text-text-dark">
                    対応状況
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
                <span className="text-sm font-medium text-text-secondary">Active Alerts</span>
              </div>
              <div className="text-2xl font-bold text-error">
                {alerts.filter((a) => a.status === 'active').length}
              </div>
            </div>
            <div>
              <div className="flex items-center justify-center gap-2 mb-2">
                <Shield className="text-success" size={20} />
                <span className="text-sm font-medium text-text-secondary">Uptime</span>
              </div>
              <div className="text-2xl font-bold text-success">99.8%</div>
            </div>
            <div>
              <div className="flex items-center justify-center gap-2 mb-2">
                <Clock className="text-text-secondary" size={20} />
                <span className="text-sm font-medium text-text-secondary">MTTR</span>
              </div>
              <div className="text-2xl font-bold text-text-primary dark:text-text-dark">4.2 min</div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
