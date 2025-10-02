import { ArrowLeft, Shield, CheckCircle, Plus, PhoneOff, Clock, FileText, CheckCircle2, XCircle } from 'lucide-react';
import { useState } from 'react';
import { useNavigate, useParams } from 'react-router-dom';
import { Button, Card, CardHeader, CardContent, Select } from '../components/ui';

/**
 * Session log entry interface
 */
interface SessionLogEntry {
  timestamp: Date;
  event: string;
  result: 'success' | 'warning' | 'error';
  details?: string;
}

/**
 * WF-02: Pairing Details Page
 * 
 * Features:
 * - Device pairing status display
 * - Security status indicator (mutual authentication)
 * - Profile selection dropdown
 * - Session event log table
 * - Stream addition and disconnection actions
 * 
 * TODO (Task 4.3 Part 3):
 * - Integrate with POST /devices/{id}/pair API
 * - Real-time session log updates (WebSocket/SSE)
 * - CSR-based pairing flow
 * - Stream management (POST /sessions)
 */
export const PairingDetailsPage = () => {
  const { id: deviceId } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [selectedProfile, setSelectedProfile] = useState('LL_INPUT');
  const [isPaired, setIsPaired] = useState(true); // TODO: Get from API

  // Mock device data (TODO: Fetch from API)
  const device = {
    id: deviceId || 'HL-EDGE-0001',
    name: 'HoneyPad X',
    type: 'tablet' as const,
    securityStatus: 'mutual_auth_complete' as const, // 相互認証済
    supportedProfiles: ['LL_INPUT', 'RT_AUDIO', 'MEDIA_8K', 'GAMING'],
  };

  // Profile options with descriptions
  const profileOptions = [
    { value: 'LL_INPUT', label: '低遅延入力 (LL_INPUT)' },
    { value: 'RT_AUDIO', label: 'リアルタイム音声 (RT_AUDIO)' },
    { value: 'MEDIA_8K', label: '8Kメディア (MEDIA_8K)' },
    { value: 'GAMING', label: 'ゲーミング (GAMING)' },
  ];

  // Mock session log (TODO: Fetch from API)
  const [sessionLog] = useState<SessionLogEntry[]>([
    {
      timestamp: new Date(Date.now() - 300000), // 5 min ago
      event: '接続開始',
      result: 'success',
      details: 'Device discovery completed',
    },
    {
      timestamp: new Date(Date.now() - 295000),
      event: '鍵交換',
      result: 'success',
      details: 'X25519 ECDH completed',
    },
    {
      timestamp: new Date(Date.now() - 290000),
      event: '相互認証',
      result: 'success',
      details: 'mTLS handshake verified',
    },
    {
      timestamp: new Date(Date.now() - 285000),
      event: 'プロファイル確定',
      result: 'success',
      details: 'LL_INPUT',
    },
    {
      timestamp: new Date(Date.now() - 280000),
      event: 'セッション確立',
      result: 'success',
      details: 'Session ID: sess-2025-abc123',
    },
  ]);

  // Format timestamp to HH:MM:SS
  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('ja-JP', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
  };

  // Handle stream addition
  const handleAddStream = () => {
    // TODO: Call POST /sessions endpoint
    console.log('Add stream with profile:', selectedProfile);
    navigate('/streams');
  };

  // Handle disconnection
  const handleDisconnect = () => {
    // TODO: Call DELETE /devices/{id}/pair or disconnect endpoint
    console.log('Disconnect device:', deviceId);
    setIsPaired(false);
    setTimeout(() => navigate('/devices'), 1000);
  };

  // Render result badge
  const renderResultBadge = (result: SessionLogEntry['result']) => {
    const config = {
      success: { icon: CheckCircle2, className: 'text-success', label: '成功' },
      warning: { icon: FileText, className: 'text-warning', label: '警告' },
      error: { icon: XCircle, className: 'text-error', label: 'エラー' },
    };
    const { icon: Icon, className, label } = config[result];
    return (
      <span className={`inline-flex items-center gap-1 text-sm font-medium ${className}`}>
        <Icon size={16} />
        {label}
      </span>
    );
  };

  return (
    <div className="space-y-6">
      {/* Header with Back Button */}
      <div className="flex items-center gap-4">
        <Button variant="ghost" icon={<ArrowLeft size={18} />} onClick={() => navigate('/devices')}>
          Back
        </Button>
        <div>
          <h1 className="text-display font-bold text-text-primary dark:text-text-dark">
            {device.name}
          </h1>
          <p className="text-sm text-text-secondary mt-1">{device.id}</p>
        </div>
      </div>

      {/* Security Status Card */}
      <Card>
        <CardHeader
          title="セキュリティステータス"
          action={
            <div className="flex items-center gap-2 text-success">
              <Shield size={20} className="fill-success" />
              <span className="font-medium">相互認証済</span>
            </div>
          }
        />
        <CardContent>
          <div className="flex items-center gap-2 text-sm text-text-secondary">
            <CheckCircle size={16} className="text-success" />
            <span>mTLS handshake verified with ed25519 certificate</span>
          </div>
          <div className="flex items-center gap-2 text-sm text-text-secondary mt-2">
            <CheckCircle size={16} className="text-success" />
            <span>X25519 key exchange completed</span>
          </div>
          <div className="flex items-center gap-2 text-sm text-text-secondary mt-2">
            <CheckCircle size={16} className="text-success" />
            <span>ChaCha20-Poly1305 encryption active</span>
          </div>
        </CardContent>
      </Card>

      {/* Profile Selection Card */}
      <Card>
        <CardHeader title="プロファイル選択" />
        <CardContent>
          <div className="space-y-4">
            <div>
              <Select
                label="QoSプロファイル"
                helperText="用途に応じた最適なプロファイルを選択してください"
                options={profileOptions}
                value={selectedProfile}
                onChange={(e) => setSelectedProfile(e.target.value)}
              />
            </div>
            <div className="flex flex-col sm:flex-row gap-3">
              <Button
                variant="primary"
                icon={<Plus size={18} />}
                onClick={handleAddStream}
                disabled={!isPaired}
                className="flex-1"
              >
                ストリーム追加
              </Button>
              <Button variant="danger" icon={<PhoneOff size={18} />} onClick={handleDisconnect} disabled={!isPaired}>
                切断
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Session Log Table */}
      <Card>
        <CardHeader
          title="セッションログ"
          subtitle={`${sessionLog.length}件のイベント`}
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
                    <div className="flex items-center gap-2">
                      <FileText size={16} />
                      イベント
                    </div>
                  </th>
                  <th className="text-left px-4 py-3 text-sm font-semibold text-text-primary dark:text-text-dark">
                    結果
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-text-secondary/10">
                {sessionLog.map((entry, index) => (
                  <tr key={index} className="hover:bg-surface-alt/50 dark:hover:bg-surface-dark/30 transition-colors">
                    <td className="px-4 py-3 text-sm text-text-primary dark:text-text-dark font-mono">
                      {formatTime(entry.timestamp)}
                    </td>
                    <td className="px-4 py-3 text-sm text-text-primary dark:text-text-dark">
                      <div className="font-medium">{entry.event}</div>
                      {entry.details && (
                        <div className="text-xs text-text-secondary mt-1">{entry.details}</div>
                      )}
                    </td>
                    <td className="px-4 py-3">
                      {renderResultBadge(entry.result)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </CardContent>
      </Card>
    </div>
  );
};
