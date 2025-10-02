import { Eye, FileText, Save } from 'lucide-react';
import { useState } from 'react';
import { Button, Card, CardContent, CardHeader, Input, Select } from '../components/ui';

/**
 * Policy template form data interface
 */
interface PolicyTemplate {
  name: string;
  usage: string;
  latencyTarget: number; // ms (1-50)
  bandwidthMin: number; // Mbps (10-5000)
  fecMode: 'NONE' | 'LIGHT' | 'MEDIUM' | 'HEAVY';
  scheduleStart: string; // ISO date
  scheduleEnd: string; // ISO date
  priority: number; // 1-5
}

/**
 * Form validation errors
 */
interface ValidationErrors {
  name?: string;
  latencyTarget?: string;
  bandwidthMin?: string;
  scheduleStart?: string;
  scheduleEnd?: string;
}

/**
 * WF-04: Policy Builder Page
 *
 * Features:
 * - Policy template creation form
 * - QoS settings (latency, bandwidth, FEC mode)
 * - Schedule configuration (start/end date, priority)
 * - Real-time validation (1-50ms latency, 10-5000Mbps bandwidth)
 * - Preview and save actions
 *
 * TODO (Task 4.3 Part 3):
 * - Integrate with POST /policies API
 * - React Hook Form integration for advanced validation
 * - Date picker component (replace native input[type=date])
 * - Template preview modal
 */
export const PolicyBuilderPage = () => {
  const [formData, setFormData] = useState<PolicyTemplate>({
    name: '',
    usage: 'low_latency',
    latencyTarget: 10,
    bandwidthMin: 150,
    fecMode: 'LIGHT',
    scheduleStart: new Date().toISOString().split('T')[0],
    scheduleEnd: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toISOString().split('T')[0], // +30 days
    priority: 2,
  });

  const [errors, setErrors] = useState<ValidationErrors>({});
  const [isPreviewOpen, setIsPreviewOpen] = useState(false);

  // Usage type options
  const usageOptions = [
    { value: 'low_latency', label: '低遅延 (Low Latency)' },
    { value: 'realtime_audio', label: 'リアルタイム音声 (Real-time Audio)' },
    { value: 'media_8k', label: '8Kメディア (8K Media)' },
    { value: 'gaming', label: 'ゲーミング (Gaming)' },
    { value: 'iot_lowpower', label: 'IoT省電力 (IoT Low Power)' },
  ];

  // FEC mode options
  const fecModeOptions = [
    { value: 'NONE', label: 'なし (NONE)' },
    { value: 'LIGHT', label: '軽 (LIGHT - 1/4)' },
    { value: 'MEDIUM', label: '中 (MEDIUM - 1/2)' },
    { value: 'HEAVY', label: '重 (HEAVY - 3/4)' },
  ];

  // Priority options
  const priorityOptions = [
    { value: '1', label: 'Level 1 (最高)' },
    { value: '2', label: 'Level 2 (高)' },
    { value: '3', label: 'Level 3 (中)' },
    { value: '4', label: 'Level 4 (低)' },
    { value: '5', label: 'Level 5 (最低)' },
  ];

  // Validate form
  const validateForm = (): boolean => {
    const newErrors: ValidationErrors = {};

    // Name validation
    if (!formData.name.trim()) {
      newErrors.name = 'テンプレート名を入力してください';
    } else if (formData.name.length < 3) {
      newErrors.name = 'テンプレート名は3文字以上で入力してください';
    }

    // Latency validation (1-50ms from wireframes.md)
    if (formData.latencyTarget < 1 || formData.latencyTarget > 50) {
      newErrors.latencyTarget = '遅延目標は1-50msの範囲で指定してください';
    }

    // Bandwidth validation (10-5000Mbps from wireframes.md)
    if (formData.bandwidthMin < 10 || formData.bandwidthMin > 5000) {
      newErrors.bandwidthMin = '帯域下限は10-5000Mbpsの範囲で指定してください';
    }

    // Schedule validation
    const start = new Date(formData.scheduleStart);
    const end = new Date(formData.scheduleEnd);
    if (start >= end) {
      newErrors.scheduleEnd = '終了日は開始日より後の日付を指定してください';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  // Handle input change
  const handleChange = (field: keyof PolicyTemplate, value: string | number) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    // Clear error for this field
    if (errors[field as keyof ValidationErrors]) {
      setErrors((prev) => ({ ...prev, [field]: undefined }));
    }
  };

  // Handle preview
  const handlePreview = () => {
    if (validateForm()) {
      setIsPreviewOpen(true);
      // TODO: Show preview modal with formatted template
      console.log('Preview:', formData);
    }
  };

  // Handle save
  const handleSave = () => {
    if (validateForm()) {
      // TODO: Call POST /policies API
      console.log('Save policy template:', formData);
      alert('テンプレートを保存しました (Mock)');
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-display font-bold text-text-primary dark:text-text-dark">
          Policy Builder
        </h1>
        <p className="text-sm text-text-secondary mt-1">
          Create and manage QoS profile templates
        </p>
      </div>

      {/* Template Form */}
      <Card>
        <CardHeader
          title="プロファイルテンプレート作成"
          subtitle="用途に応じたQoS設定を定義します"
        />
        <CardContent>
          <div className="space-y-6">
            {/* Basic Information */}
            <div className="space-y-4">
              <h3 className="text-subheading font-semibold text-text-primary dark:text-text-dark">
                基本情報
              </h3>

              <Input
                label="テンプレート名"
                placeholder="例: 低遅延ゲーミング用"
                value={formData.name}
                onChange={(e) => handleChange('name', e.target.value)}
                error={errors.name}
                fullWidth
              />

              <Select
                label="用途"
                helperText="テンプレートの主な用途を選択してください"
                options={usageOptions}
                value={formData.usage}
                onChange={(e) => handleChange('usage', e.target.value)}
              />
            </div>

            {/* QoS Settings */}
            <div className="space-y-4">
              <h3 className="text-subheading font-semibold text-text-primary dark:text-text-dark">
                QoS設定
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Input
                  label="遅延目標 (ms)"
                  helperText="1-50msの範囲で指定"
                  type="number"
                  min={1}
                  max={50}
                  value={formData.latencyTarget}
                  onChange={(e) => handleChange('latencyTarget', parseInt(e.target.value, 10))}
                  error={errors.latencyTarget}
                  fullWidth
                />

                <Input
                  label="帯域下限 (Mbps)"
                  helperText="10-5000Mbpsの範囲で指定"
                  type="number"
                  min={10}
                  max={5000}
                  value={formData.bandwidthMin}
                  onChange={(e) => handleChange('bandwidthMin', parseInt(e.target.value, 10))}
                  error={errors.bandwidthMin}
                  fullWidth
                />
              </div>

              <Select
                label="FECモード"
                helperText="Forward Error Correctionの強度を選択"
                options={fecModeOptions}
                value={formData.fecMode}
                onChange={(e) => handleChange('fecMode', e.target.value as PolicyTemplate['fecMode'])}
              />
            </div>

            {/* Schedule Settings */}
            <div className="space-y-4">
              <h3 className="text-subheading font-semibold text-text-primary dark:text-text-dark">
                スケジュール設定
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Input
                  label="有効期間 (開始)"
                  type="date"
                  value={formData.scheduleStart}
                  onChange={(e) => handleChange('scheduleStart', e.target.value)}
                  error={errors.scheduleStart}
                  fullWidth
                />

                <Input
                  label="有効期間 (終了)"
                  type="date"
                  value={formData.scheduleEnd}
                  onChange={(e) => handleChange('scheduleEnd', e.target.value)}
                  error={errors.scheduleEnd}
                  fullWidth
                />
              </div>

              <Select
                label="優先度"
                helperText="複数のプロファイルが競合する場合の優先順位"
                options={priorityOptions}
                value={formData.priority.toString()}
                onChange={(e) => handleChange('priority', parseInt(e.target.value, 10))}
              />
            </div>

            {/* Actions */}
            <div className="flex flex-col sm:flex-row gap-3 pt-4 border-t border-text-secondary/10">
              <Button
                variant="outline"
                icon={<Eye size={18} />}
                onClick={handlePreview}
                className="flex-1"
              >
                プレビュー
              </Button>
              <Button
                variant="primary"
                icon={<Save size={18} />}
                onClick={handleSave}
                className="flex-1"
              >
                保存
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Validation Messages (Global) */}
      {Object.keys(errors).length > 0 && (
        <Card>
          <CardContent className="py-4">
            <div className="flex items-start gap-3">
              <FileText size={20} className="text-error mt-0.5" />
              <div>
                <div className="font-semibold text-text-primary dark:text-text-dark mb-2">
                  入力内容を確認してください
                </div>
                <ul className="text-sm text-text-secondary space-y-1 list-disc list-inside">
                  {Object.values(errors).map((error, index) => (
                    <li key={index}>{error}</li>
                  ))}
                </ul>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Preview Modal Placeholder */}
      {isPreviewOpen && (
        <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
          <Card className="max-w-2xl w-full m-4">
            <CardHeader title="テンプレートプレビュー" />
            <CardContent>
              <div className="space-y-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-text-secondary">名称:</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">{formData.name}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-text-secondary">用途:</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">
                    {usageOptions.find((opt) => opt.value === formData.usage)?.label}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-text-secondary">遅延目標:</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">{formData.latencyTarget} ms</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-text-secondary">帯域下限:</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">{formData.bandwidthMin} Mbps</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-text-secondary">FECモード:</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">{formData.fecMode}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-text-secondary">優先度:</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">Level {formData.priority}</span>
                </div>
              </div>
              <div className="mt-6 flex justify-end">
                <Button variant="primary" onClick={() => setIsPreviewOpen(false)}>
                  閉じる
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
};
