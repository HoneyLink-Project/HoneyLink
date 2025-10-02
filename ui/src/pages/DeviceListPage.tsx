import { ArrowUpDown, RefreshCw, Search, Signal, Smartphone, Wifi } from 'lucide-react';
import { useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';
import { Button, Card, CardContent, CardHeader, Input, Modal, ModalFooter, Select } from '../components/ui';

/**
 * Device interface matching backend POST /devices response
 */
interface Device {
  id: string;
  name: string;
  type: 'smartphone' | 'tablet' | 'laptop' | 'iot' | 'other';
  signalStrength: 1 | 2 | 3 | 4 | 5; // 1-5 stars
  profiles: string[]; // e.g. ['LL_INPUT', 'RT_AUDIO']
  lastSeen: Date;
  status: 'online' | 'offline' | 'pairing';
}

type SortKey = 'name' | 'signalStrength' | 'lastSeen';
type SortOrder = 'asc' | 'desc';

/**
 * WF-01: Device List Page (Near

by Devices)
 *
 * Features:
 * - Device discovery with search and filters
 * - Signal strength display (1-5 stars)
 * - Supported profiles badge
 * - Sort by name, signal, last seen
 * - Scan retry with loading state
 * - Responsive grid layout
 *
 * TODO (Task 4.3 Part 2):
 * - Integrate with POST /devices API
 * - Real-time device discovery (WebSocket/SSE)
 * - Device detail navigation
 */
export const DeviceListPage = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [searchQuery, setSearchQuery] = useState('');
  const [filterType, setFilterType] = useState<Device['type'] | ''>('');
  const [sortKey, setSortKey] = useState<SortKey>('signalStrength');
  const [sortOrder, setSortOrder] = useState<SortOrder>('desc');
  const [isScanning, setIsScanning] = useState(false);
  const [showScanModal, setShowScanModal] = useState(false);

  // Mock device data (TODO: Replace with API call)
  const [devices] = useState<Device[]>([
    {
      id: 'HL-EDGE-0001',
      name: 'HoneyPad X',
      type: 'tablet',
      signalStrength: 4,
      profiles: ['LL_INPUT', 'RT_AUDIO'],
      lastSeen: new Date(Date.now() - 2 * 60 * 1000), // 2 minutes ago
      status: 'online',
    },
    {
      id: 'HL-EDGE-0002',
      name: 'HoneyPhone Pro',
      type: 'smartphone',
      signalStrength: 5,
      profiles: ['LL_INPUT', 'RT_AUDIO', 'MEDIA_8K'],
      lastSeen: new Date(Date.now() - 5 * 60 * 1000),
      status: 'online',
    },
    {
      id: 'HL-EDGE-0003',
      name: 'HoneyBook Air',
      type: 'laptop',
      signalStrength: 3,
      profiles: ['MEDIA_8K', 'GAMING'],
      lastSeen: new Date(Date.now() - 10 * 60 * 1000),
      status: 'online',
    },
    {
      id: 'HL-IOT-5001',
      name: 'Smart Sensor',
      type: 'iot',
      signalStrength: 2,
      profiles: ['IOT_LOWPOWER'],
      lastSeen: new Date(Date.now() - 15 * 60 * 1000),
      status: 'offline',
    },
  ]);

  const deviceTypes = [
    { value: '', label: t('device_list.filter_all') },
    { value: 'smartphone', label: t('device_list.type_smartphone') },
    { value: 'tablet', label: t('device_list.type_tablet') },
    { value: 'laptop', label: t('device_list.type_laptop') },
    { value: 'iot', label: t('device_list.type_iot') },
    { value: 'other', label: t('device_list.type_other') },
  ];

  const sortOptions = [
    { value: 'name', label: t('device_list.sort_name') },
    { value: 'signalStrength', label: t('device_list.sort_signal') },
    { value: 'lastSeen', label: t('device_list.sort_last_seen') },
  ];

  // Filter and sort devices
  const filteredDevices = useMemo(() => {
    let result = devices;

    // Apply search filter
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      result = result.filter(
        (d) =>
          d.name.toLowerCase().includes(query) ||
          d.id.toLowerCase().includes(query) ||
          d.profiles.some((p) => p.toLowerCase().includes(query))
      );
    }

    // Apply type filter
    if (filterType) {
      result = result.filter((d) => d.type === filterType);
    }

    // Apply sort
    result.sort((a, b) => {
      let comparison = 0;
      switch (sortKey) {
        case 'name':
          comparison = a.name.localeCompare(b.name);
          break;
        case 'signalStrength':
          comparison = a.signalStrength - b.signalStrength;
          break;
        case 'lastSeen':
          comparison = a.lastSeen.getTime() - b.lastSeen.getTime();
          break;
      }
      return sortOrder === 'asc' ? comparison : -comparison;
    });

    return result;
  }, [devices, searchQuery, filterType, sortKey, sortOrder]);

  // Handle scan action
  const handleScan = async () => {
    setIsScanning(true);
    setShowScanModal(false);
    // TODO: Call POST /devices scan endpoint
    await new Promise((resolve) => setTimeout(resolve, 2000)); // Mock delay
    setIsScanning(false);
  };

  // Handle device connection
  const handleConnect = (deviceId: string) => {
    navigate(`/devices/${deviceId}/pair`);
  };

  // Render signal strength stars
  const renderSignalStrength = (strength: number) => {
    return (
      <div className="flex items-center gap-1">
        {[1, 2, 3, 4, 5].map((level) => (
          <Signal
            key={level}
            size={14}
            className={level <= strength ? 'text-success fill-success' : 'text-text-secondary'}
          />
        ))}
      </div>
    );
  };

  // Format last seen time
  const formatLastSeen = (date: Date) => {
    const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
    if (seconds < 60) return t('common.time_just_now');
    const minutes = Math.floor(seconds / 60);
    if (minutes < 60) return t('common.time_minutes_ago', { count: minutes });
    const hours = Math.floor(minutes / 60);
    return t('common.time_hours_ago', { count: hours });
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
        <div>
          <h1 className="text-display font-bold text-text-primary dark:text-text-dark">
            {t('device_list.title')}
          </h1>
          <p className="text-sm text-text-secondary mt-1">
            {t('device_list.devices_found', { count: filteredDevices.length })}
          </p>
        </div>
        <Button
          variant="primary"
          icon={isScanning ? <RefreshCw size={18} className="animate-spin" /> : <Smartphone size={18} />}
          onClick={() => setShowScanModal(true)}
          disabled={isScanning}
        >
          {isScanning ? t('device_list.scanning') : t('device_list.scan_button')}
        </Button>
      </div>

      {/* Search and Filters */}
      <Card>
        <div className="flex flex-col sm:flex-row gap-3">
          <div className="flex-1">
            <Input
              placeholder={t('device_list.search_placeholder')}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              icon={<Search size={18} />}
              fullWidth
            />
          </div>
          <div className="flex gap-3">
            <Select
              options={deviceTypes}
              value={filterType}
              onChange={(e) => setFilterType(e.target.value as Device['type'] | '')}
              placeholder="All Types"
            />
            <Select
              options={sortOptions}
              value={sortKey}
              onChange={(e) => setSortKey(e.target.value as SortKey)}
            />
            <Button
              variant="outline"
              icon={<ArrowUpDown size={16} />}
              onClick={() => setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc')}
              aria-label={`Sort ${sortOrder === 'asc' ? 'descending' : 'ascending'}`}
            >
              {sortOrder === 'asc' ? '↑' : '↓'}
            </Button>
          </div>
        </div>
      </Card>

      {/* Device Grid */}
      {filteredDevices.length === 0 ? (
        <Card>
          <div className="text-center py-12">
            <Wifi size={48} className="mx-auto text-text-secondary mb-4" />
            <h3 className="text-heading font-semibold text-text-primary dark:text-text-dark mb-2">
              {t('device_list.no_devices')}
            </h3>
            <p className="text-text-secondary mb-4">
              {searchQuery || filterType ? t('device_list.adjust_filters') : t('device_list.scan_prompt')}
            </p>
            {!isScanning && (
              <Button variant="primary" icon={<Smartphone size={18} />} onClick={() => setShowScanModal(true)}>
                {t('device_list.scan_button')}
              </Button>
            )}
          </div>
        </Card>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {filteredDevices.map((device) => (
            <Card key={device.id} hoverable>
              <CardHeader
                title={device.name}
                subtitle={device.id}
                action={
                  <span
                    className={`text-sm font-medium ${
                      device.status === 'online'
                        ? 'text-success'
                        : device.status === 'pairing'
                        ? 'text-warning'
                        : 'text-text-secondary'
                    }`}
                  >
                    {device.status.charAt(0).toUpperCase() + device.status.slice(1)}
                  </span>
                }
              />
              <CardContent>
                <div className="space-y-3">
                  {/* Signal Strength */}
                  <div className="flex items-center justify-between">
                    <span className="text-sm text-text-secondary">{t('device_list.signal')}:</span>
                    {renderSignalStrength(device.signalStrength)}
                  </div>

                  {/* Supported Profiles */}
                  <div>
                    <span className="text-sm text-text-secondary block mb-1">{t('device_list.profiles')}:</span>
                    <div className="flex flex-wrap gap-1">
                      {device.profiles.map((profile) => (
                        <span
                          key={profile}
                          className="text-xs px-2 py-1 rounded bg-surface-alt dark:bg-surface-dark text-text-primary dark:text-text-dark border border-primary"
                        >
                          {profile}
                        </span>
                      ))}
                    </div>
                  </div>

                  {/* Last Seen */}
                  <div className="flex items-center justify-between text-sm text-text-secondary">
                    <span>{t('device_list.last_seen')}:</span>
                    <span>{formatLastSeen(device.lastSeen)}</span>
                  </div>

                  {/* Actions */}
                  <div className="flex gap-2 pt-2">
                    <Button
                      variant="primary"
                      size="sm"
                      onClick={() => handleConnect(device.id)}
                      disabled={device.status === 'offline'}
                      className="flex-1"
                    >
                      {t('common.connect')}
                    </Button>
                    <Button variant="outline" size="sm" onClick={() => navigate(`/devices/${device.id}`)}>
                      {t('common.details')}
                    </Button>
                  </div>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}

      {/* Scan Confirmation Modal */}
      <Modal
        isOpen={showScanModal}
        onClose={() => setShowScanModal(false)}
        title={t('device_list.scan_modal_title')}
        footer={
          <ModalFooter
            onCancel={() => setShowScanModal(false)}
            onConfirm={handleScan}
            confirmText={t('device_list.start_scan')}
          />
        }
      >
        <p className="text-text-primary dark:text-text-dark">
          {t('device_list.scan_modal_content')}
        </p>
      </Modal>
    </div>
  );
};
