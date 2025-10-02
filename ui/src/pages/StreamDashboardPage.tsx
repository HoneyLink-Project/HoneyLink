import { Activity, AlertCircle, ArrowDown, ArrowUp, CheckCircle2, Clock, Settings, TrendingUp } from 'lucide-react';
import { useState } from 'react';
import { Button, Card, CardContent, CardHeader } from '../components/ui';

/**
 * Stream status interface
 */
interface StreamStatus {
  id: string;
  profile: string;
  profileLabel: string;
  latency: number; // ms
  jitter: number; // ms
  packetLoss: number; // percentage
  bandwidth: number; // Mbps
  fecRate: string; // e.g. '1/4'
  status: 'optimal' | 'degraded' | 'critical';
}

/**
 * Event timeline entry interface
 */
interface TimelineEvent {
  timestamp: Date;
  type: 'qos_update' | 'fec_change' | 'priority_change' | 'alert';
  profile?: string;
  message: string;
  severity: 'info' | 'warning' | 'error';
}

/**
 * WF-03: Stream Dashboard Page
 *
 * Features:
 * - Stream summary cards with QoS metrics
 * - Priority adjustment (up/down arrows)
 * - Event timeline with severity indicators
 * - KPI achievement banner
 * - Real-time chart placeholder
 *
 * TODO (Task 4.3 Part 3):
 * - Integrate with GET /sessions API
 * - Real-time updates (WebSocket/SSE)
 * - Interactive charts (recharts library)
 * - Priority adjustment API (PUT /sessions/{id}/priority)
 */
export const StreamDashboardPage = () => {
  // Mock stream data (TODO: Replace with API call)
  const [streams] = useState<StreamStatus[]>([
    {
      id: 'stream-001',
      profile: 'LL_INPUT',
      profileLabel: '低遅延入力',
      latency: 5,
      jitter: 1,
      packetLoss: 0.0,
      bandwidth: 150,
      fecRate: '1/4',
      status: 'optimal',
    },
    {
      id: 'stream-002',
      profile: 'RT_AUDIO',
      profileLabel: 'リアルタイム音声',
      latency: 12,
      jitter: 2,
      packetLoss: 0.2,
      bandwidth: 80,
      fecRate: '1/2',
      status: 'optimal',
    },
  ]);

  // Mock timeline events (TODO: Fetch from API)
  const [timelineEvents] = useState<TimelineEvent[]>([
    {
      timestamp: new Date(Date.now() - 180000), // 3 min ago
      type: 'qos_update',
      profile: 'LL_INPUT',
      message: 'QoS更新: LL_INPUT → 遅延目標 5ms',
      severity: 'info',
    },
    {
      timestamp: new Date(Date.now() - 120000), // 2 min ago
      type: 'fec_change',
      profile: 'RT_AUDIO',
      message: 'FEC率変更: 1/2 → 1/4 (帯域改善)',
      severity: 'info',
    },
    {
      timestamp: new Date(Date.now() - 60000), // 1 min ago
      type: 'priority_change',
      profile: 'LL_INPUT',
      message: '優先度変更: Level 2 → Level 1',
      severity: 'info',
    },
    {
      timestamp: new Date(Date.now() - 30000), // 30 sec ago
      type: 'alert',
      message: 'RT_AUDIO パケットロス 0.2% (閾値以下)',
      severity: 'warning',
    },
  ]);

  // Calculate overall KPI achievement (mock)
  const kpiAchievement = 98; // 98%

  // Format time as HH:MM
  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('ja-JP', { hour: '2-digit', minute: '2-digit' });
  };

  // Get status badge
  const getStatusBadge = (status: StreamStatus['status']) => {
    const config = {
      optimal: { label: '最適', className: 'text-success bg-success/10' },
      degraded: { label: '劣化', className: 'text-warning bg-warning/10' },
      critical: { label: '警告', className: 'text-error bg-error/10' },
    };
    const { label, className } = config[status];
    return <span className={`px-2 py-1 rounded text-xs font-medium ${className}`}>{label}</span>;
  };

  // Get severity icon
  const getSeverityIcon = (severity: TimelineEvent['severity']) => {
    switch (severity) {
      case 'info':
        return <CheckCircle2 size={16} className="text-success" />;
      case 'warning':
        return <AlertCircle size={16} className="text-warning" />;
      case 'error':
        return <AlertCircle size={16} className="text-error" />;
    }
  };

  // Handle priority adjustment
  const handlePriorityChange = (streamId: string, direction: 'up' | 'down') => {
    // TODO: Call PUT /sessions/{streamId}/priority API
    console.log(`Adjust priority for ${streamId}: ${direction}`);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-display font-bold text-text-primary dark:text-text-dark">
          Stream Dashboard
        </h1>
        <p className="text-sm text-text-secondary mt-1">
          {streams.length} active stream{streams.length !== 1 ? 's' : ''}
        </p>
      </div>

      {/* KPI Achievement Banner */}
      <Card>
        <CardContent className="flex items-center justify-between py-4">
          <div className="flex items-center gap-3">
            <TrendingUp size={32} className="text-success" />
            <div>
              <div className="text-heading font-semibold text-text-primary dark:text-text-dark">
                KPI達成率
              </div>
              <div className="text-sm text-text-secondary">
                Overall performance metric
              </div>
            </div>
          </div>
          <div className="text-right">
            <div className="text-3xl font-bold text-success">{kpiAchievement}%</div>
            <div className="text-xs text-text-secondary">Target: ≥95%</div>
          </div>
        </CardContent>
      </Card>

      {/* Stream Summary Cards */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {streams.map((stream) => (
          <Card key={stream.id}>
            <CardHeader
              title={stream.profileLabel}
              subtitle={stream.profile}
              action={getStatusBadge(stream.status)}
            />
            <CardContent>
              <div className="space-y-3">
                {/* Metrics Grid */}
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <div className="text-xs text-text-secondary mb-1">Latency</div>
                    <div className="text-lg font-semibold text-text-primary dark:text-text-dark">
                      {stream.latency} ms
                    </div>
                  </div>
                  <div>
                    <div className="text-xs text-text-secondary mb-1">Jitter</div>
                    <div className="text-lg font-semibold text-text-primary dark:text-text-dark">
                      {stream.jitter} ms
                    </div>
                  </div>
                  <div>
                    <div className="text-xs text-text-secondary mb-1">Packet Loss</div>
                    <div className="text-lg font-semibold text-text-primary dark:text-text-dark">
                      {stream.packetLoss.toFixed(1)}%
                    </div>
                  </div>
                  <div>
                    <div className="text-xs text-text-secondary mb-1">Bandwidth</div>
                    <div className="text-lg font-semibold text-text-primary dark:text-text-dark">
                      {stream.bandwidth} Mbps
                    </div>
                  </div>
                </div>

                {/* FEC Rate */}
                <div className="flex items-center justify-between pt-2 border-t border-text-secondary/10">
                  <div className="text-sm text-text-secondary">
                    FEC Rate: <span className="font-medium text-text-primary dark:text-text-dark">{stream.fecRate}</span>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      icon={<ArrowUp size={14} />}
                      onClick={() => handlePriorityChange(stream.id, 'up')}
                      aria-label="Increase priority"
                    >
                      {''}
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      icon={<ArrowDown size={14} />}
                      onClick={() => handlePriorityChange(stream.id, 'down')}
                      aria-label="Decrease priority"
                    >
                      {''}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      icon={<Settings size={14} />}
                      aria-label="Stream settings"
                    >
                      {''}
                    </Button>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Real-time Chart Placeholder */}
      <Card>
        <CardHeader title="リアルタイムチャート" subtitle="Latency & Jitter over time" />
        <CardContent>
          <div className="h-64 flex items-center justify-center bg-surface-alt/30 dark:bg-surface-dark/30 rounded border-2 border-dashed border-text-secondary/30">
            <div className="text-center">
              <Activity size={48} className="mx-auto text-text-secondary mb-3" />
              <div className="text-sm font-medium text-text-primary dark:text-text-dark mb-1">
                Chart visualization
              </div>
              <div className="text-xs text-text-secondary">
                TODO: Integrate recharts library for real-time line chart
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Event Timeline */}
      <Card>
        <CardHeader
          title="イベントタイムライン"
          subtitle={`${timelineEvents.length} recent events`}
        />
        <CardContent className="p-0">
          <div className="divide-y divide-text-secondary/10">
            {timelineEvents.map((event, index) => (
              <div key={index} className="px-4 py-3 hover:bg-surface-alt/50 dark:hover:bg-surface-dark/30 transition-colors">
                <div className="flex items-start gap-3">
                  {getSeverityIcon(event.severity)}
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between gap-2">
                      <div className="text-sm font-medium text-text-primary dark:text-text-dark truncate">
                        {event.message}
                      </div>
                      <div className="flex items-center gap-1 text-xs text-text-secondary whitespace-nowrap">
                        <Clock size={12} />
                        {formatTime(event.timestamp)}
                      </div>
                    </div>
                    {event.profile && (
                      <div className="text-xs text-text-secondary mt-1">
                        Profile: {event.profile}
                      </div>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
