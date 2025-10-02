import { AlertCircle, ArrowDown, ArrowUp, CheckCircle2, Clock, Settings, TrendingUp } from 'lucide-react';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { CartesianGrid, Legend, Line, LineChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';
import { useStreams, useUpdateStreamPriority } from '../api/hooks';
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
  const { t } = useTranslation();

  // Fetch streams from API (auto-refresh every 5s)
  const { data: apiStreams } = useStreams();
  const updatePriorityMutation = useUpdateStreamPriority();

  // Fallback to mock data if API fails
  const mockStreams: StreamStatus[] = [
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
  ];

  const streams = apiStreams || mockStreams;

  // Generate mock chart data (last 5 minutes, 1s interval = 300 points)
  // TODO: Replace with useStreamMetrics API hook
  const [chartData, setChartData] = useState<{ timestamp: string; ll_input: number; rt_audio: number }[]>(() => {
    const data = [];
    const now = Date.now();
    for (let i = 60; i >= 0; i--) {
      const time = new Date(now - i * 5000); // 5s interval for 5min window
      const hours = time.getHours().toString().padStart(2, '0');
      const minutes = time.getMinutes().toString().padStart(2, '0');
      const seconds = time.getSeconds().toString().padStart(2, '0');
      data.push({
        timestamp: `${hours}:${minutes}:${seconds}`,
        ll_input: 5 + Math.random() * 3, // 5-8ms with noise
        rt_audio: 12 + Math.random() * 4, // 12-16ms with noise
      });
    }
    return data;
  });

  // Simulate real-time chart updates (every 5s)
  useEffect(() => {
    const interval = setInterval(() => {
      setChartData((prev) => {
        const now = new Date();
        const hours = now.getHours().toString().padStart(2, '0');
        const minutes = now.getMinutes().toString().padStart(2, '0');
        const seconds = now.getSeconds().toString().padStart(2, '0');
        const newPoint = {
          timestamp: `${hours}:${minutes}:${seconds}`,
          ll_input: 5 + Math.random() * 3,
          rt_audio: 12 + Math.random() * 4,
        };
        // Keep last 61 points (5 minutes)
        return [...prev.slice(1), newPoint];
      });
    }, 5000); // Update every 5s
    return () => clearInterval(interval);
  }, []);

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
      optimal: { label: t('stream_dashboard.status.optimal'), className: 'text-success bg-success/10' },
      degraded: { label: t('stream_dashboard.status.degraded'), className: 'text-warning bg-warning/10' },
      critical: { label: t('stream_dashboard.status.critical'), className: 'text-error bg-error/10' },
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

  // Handle priority adjustment with API call
  const handlePriorityChange = (streamId: string, direction: 'up' | 'down') => {
    // Determine new priority (current priority not tracked in this mock)
    // In real implementation, track current priority per stream
    const newPriority = direction === 'up' ? 1 : 2; // Simplified logic

    updatePriorityMutation.mutate(
      { streamId, priority: newPriority },
      {
        onSuccess: () => {
          console.log(`Priority updated for ${streamId}: ${direction}`);
          // TODO Part 4: toast.success(`優先度を${direction === 'up' ? '上げました' : '下げました'}`)
        },
        onError: (error) => {
          console.error(`Failed to update priority for ${streamId}:`, error);
          // TODO Part 4: toast.error('優先度の更新に失敗しました')
        },
      }
    );
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-display font-bold text-text-primary dark:text-text-dark">
          {t('stream_dashboard.title')}
        </h1>
        <p className="text-sm text-text-secondary mt-1">
          {t('stream_dashboard.subtitle', { count: streams.length })}
        </p>
      </div>

      {/* KPI Achievement Banner */}
      <Card>
        <CardContent className="flex items-center justify-between py-4">
          <div className="flex items-center gap-3">
            <TrendingUp size={32} className="text-success" />
            <div>
              <div className="text-heading font-semibold text-text-primary dark:text-text-dark">
                {t('stream_dashboard.kpi.title')}
              </div>
              <div className="text-sm text-text-secondary">
                {t('stream_dashboard.kpi.description')}
              </div>
            </div>
          </div>
          <div className="text-right">
            <div className="text-3xl font-bold text-success">{kpiAchievement}%</div>
            <div className="text-xs text-text-secondary">{t('stream_dashboard.kpi.target')}</div>
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
                    <div className="text-xs text-text-secondary mb-1">{t('stream_dashboard.metrics.latency')}</div>
                    <div className="text-lg font-semibold text-text-primary dark:text-text-dark">
                      {stream.latency} ms
                    </div>
                  </div>
                  <div>
                    <div className="text-xs text-text-secondary mb-1">{t('stream_dashboard.metrics.jitter')}</div>
                    <div className="text-lg font-semibold text-text-primary dark:text-text-dark">
                      {stream.jitter} ms
                    </div>
                  </div>
                  <div>
                    <div className="text-xs text-text-secondary mb-1">{t('stream_dashboard.metrics.packet_loss')}</div>
                    <div className="text-lg font-semibold text-text-primary dark:text-text-dark">
                      {stream.packetLoss.toFixed(1)}%
                    </div>
                  </div>
                  <div>
                    <div className="text-xs text-text-secondary mb-1">{t('stream_dashboard.metrics.bandwidth')}</div>
                    <div className="text-lg font-semibold text-text-primary dark:text-text-dark">
                      {stream.bandwidth} Mbps
                    </div>
                  </div>
                </div>

                {/* FEC Rate */}
                <div className="flex items-center justify-between pt-2 border-t border-text-secondary/10">
                  <div className="text-sm text-text-secondary">
                    {t('stream_dashboard.metrics.fec_rate')} <span className="font-medium text-text-primary dark:text-text-dark">{stream.fecRate}</span>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      icon={<ArrowUp size={14} />}
                      onClick={() => handlePriorityChange(stream.id, 'up')}
                      aria-label={t('stream_dashboard.priority.increase')}
                    >
                      {''}
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      icon={<ArrowDown size={14} />}
                      onClick={() => handlePriorityChange(stream.id, 'down')}
                      aria-label={t('stream_dashboard.priority.decrease')}
                    >
                      {''}
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      icon={<Settings size={14} />}
                      aria-label={t('stream_dashboard.priority.settings')}
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

      {/* Real-time Chart */}
      <Card>
        <CardHeader title={t('stream_dashboard.chart.title')} subtitle={t('stream_dashboard.chart.subtitle')} />
        <CardContent>
          <ResponsiveContainer width="100%" height={256}>
            <LineChart data={chartData} margin={{ top: 5, right: 20, left: 0, bottom: 5 }}>
              <CartesianGrid strokeDasharray="3 3" stroke="#e5e7eb" opacity={0.3} />
              <XAxis
                dataKey="timestamp"
                stroke="#6b7280"
                fontSize={12}
                tickLine={false}
                interval="preserveStartEnd"
              />
              <YAxis
                stroke="#6b7280"
                fontSize={12}
                tickLine={false}
                label={{ value: 'Latency (ms)', angle: -90, position: 'insideLeft', style: { fill: '#6b7280', fontSize: 12 } }}
              />
              <Tooltip
                contentStyle={{
                  backgroundColor: '#ffffff',
                  border: '1px solid #e5e7eb',
                  borderRadius: '8px',
                  fontSize: '12px',
                }}
                labelStyle={{ fontWeight: 600, marginBottom: '4px' }}
              />
              <Legend
                wrapperStyle={{ fontSize: '12px', paddingTop: '10px' }}
                iconType="line"
              />
              <Line
                type="monotone"
                dataKey="ll_input"
                stroke="#F4B400"
                strokeWidth={2}
                name="LL_INPUT (低遅延入力)"
                dot={false}
                isAnimationActive={false}
              />
              <Line
                type="monotone"
                dataKey="rt_audio"
                stroke="#7F5AF0"
                strokeWidth={2}
                name="RT_AUDIO (音声)"
                dot={false}
                isAnimationActive={false}
              />
            </LineChart>
          </ResponsiveContainer>
        </CardContent>
      </Card>

      {/* Event Timeline */}
      <Card>
        <CardHeader
          title={t('stream_dashboard.timeline.title')}
          subtitle={t('stream_dashboard.timeline.subtitle', { count: timelineEvents.length })}
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

// Default export for code splitting (React.lazy)
export default StreamDashboardPage;
