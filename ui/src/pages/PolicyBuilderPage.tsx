import { Eye, FileText, Save } from 'lucide-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
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
  const { t } = useTranslation();

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
    { value: 'low_latency', label: t('policy_builder.usage_types.low_latency') },
    { value: 'realtime_audio', label: t('policy_builder.usage_types.realtime_audio') },
    { value: 'media_8k', label: t('policy_builder.usage_types.media_8k') },
    { value: 'gaming', label: t('policy_builder.usage_types.gaming') },
    { value: 'iot_lowpower', label: t('policy_builder.usage_types.iot_lowpower') },
  ];

  // FEC mode options
  const fecModeOptions = [
    { value: 'NONE', label: t('policy_builder.fec_modes.none') },
    { value: 'LIGHT', label: t('policy_builder.fec_modes.light') },
    { value: 'MEDIUM', label: t('policy_builder.fec_modes.medium') },
    { value: 'HEAVY', label: t('policy_builder.fec_modes.heavy') },
  ];

  // Priority options
  const priorityOptions = [
    { value: '1', label: t('policy_builder.priority_levels.1') },
    { value: '2', label: t('policy_builder.priority_levels.2') },
    { value: '3', label: t('policy_builder.priority_levels.3') },
    { value: '4', label: t('policy_builder.priority_levels.4') },
    { value: '5', label: t('policy_builder.priority_levels.5') },
  ];

  // Validate form
  const validateForm = (): boolean => {
    const newErrors: ValidationErrors = {};

    // Name validation
    if (!formData.name.trim()) {
      newErrors.name = t('policy_builder.validation.name_required');
    } else if (formData.name.length < 3) {
      newErrors.name = t('policy_builder.validation.name_min_length');
    }

    // Latency validation (1-50ms from wireframes.md)
    if (formData.latencyTarget < 1 || formData.latencyTarget > 50) {
      newErrors.latencyTarget = t('policy_builder.validation.latency_range');
    }

    // Bandwidth validation (10-5000Mbps from wireframes.md)
    if (formData.bandwidthMin < 10 || formData.bandwidthMin > 5000) {
      newErrors.bandwidthMin = t('policy_builder.validation.bandwidth_range');
    }

    // Schedule validation
    const start = new Date(formData.scheduleStart);
    const end = new Date(formData.scheduleEnd);
    if (start >= end) {
      newErrors.scheduleEnd = t('policy_builder.validation.schedule_end_after_start');
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
          {t('policy_builder.title')}
        </h1>
        <p className="text-sm text-text-secondary mt-1">
          {t('policy_builder.subtitle')}
        </p>
      </div>

      {/* Template Form */}
      <Card>
        <CardHeader
          title={t('policy_builder.card_title')}
          subtitle={t('policy_builder.card_subtitle')}
        />
        <CardContent>
          <div className="space-y-6">
            {/* Basic Information */}
            <div className="space-y-4">
              <h3 className="text-subheading font-semibold text-text-primary dark:text-text-dark">
                {t('policy_builder.form.basic_info')}
              </h3>

              <Input
                label={t('policy_builder.form.template_name')}
                placeholder={t('policy_builder.form.template_name_placeholder')}
                value={formData.name}
                onChange={(e) => handleChange('name', e.target.value)}
                error={errors.name}
                fullWidth
              />

              <Select
                label={t('policy_builder.form.usage')}
                helperText={t('policy_builder.form.usage_help')}
                options={usageOptions}
                value={formData.usage}
                onChange={(e) => handleChange('usage', e.target.value)}
              />
            </div>

            {/* QoS Settings */}
            <div className="space-y-4">
              <h3 className="text-subheading font-semibold text-text-primary dark:text-text-dark">
                {t('policy_builder.form.qos_settings')}
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Input
                  label={t('policy_builder.form.latency_target')}
                  helperText={t('policy_builder.form.latency_help')}
                  type="number"
                  min={1}
                  max={50}
                  value={formData.latencyTarget}
                  onChange={(e) => handleChange('latencyTarget', parseInt(e.target.value, 10))}
                  error={errors.latencyTarget}
                  fullWidth
                />

                <Input
                  label={t('policy_builder.form.bandwidth_min')}
                  helperText={t('policy_builder.form.bandwidth_help')}
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
                label={t('policy_builder.form.fec_mode')}
                helperText={t('policy_builder.form.fec_help')}
                options={fecModeOptions}
                value={formData.fecMode}
                onChange={(e) => handleChange('fecMode', e.target.value as PolicyTemplate['fecMode'])}
              />
            </div>

            {/* Schedule Settings */}
            <div className="space-y-4">
              <h3 className="text-subheading font-semibold text-text-primary dark:text-text-dark">
                {t('policy_builder.form.schedule_settings')}
              </h3>

              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <Input
                  label={t('policy_builder.form.schedule_start')}
                  type="date"
                  value={formData.scheduleStart}
                  onChange={(e) => handleChange('scheduleStart', e.target.value)}
                  error={errors.scheduleStart}
                  fullWidth
                />

                <Input
                  label={t('policy_builder.form.schedule_end')}
                  type="date"
                  value={formData.scheduleEnd}
                  onChange={(e) => handleChange('scheduleEnd', e.target.value)}
                  error={errors.scheduleEnd}
                  fullWidth
                />
              </div>

              <Select
                label={t('policy_builder.form.priority')}
                helperText={t('policy_builder.form.priority_help')}
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
                {t('policy_builder.buttons.preview')}
              </Button>
              <Button
                variant="primary"
                icon={<Save size={18} />}
                onClick={handleSave}
                className="flex-1"
              >
                {t('policy_builder.buttons.save')}
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
                  {t('policy_builder.validation.title')}
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
            <CardHeader title={t('policy_builder.preview.title')} />
            <CardContent>
              <div className="space-y-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-text-secondary">{t('policy_builder.preview.name')}</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">{formData.name}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-text-secondary">{t('policy_builder.preview.usage')}</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">
                    {usageOptions.find((opt) => opt.value === formData.usage)?.label}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-text-secondary">{t('policy_builder.preview.latency_target')}</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">{formData.latencyTarget} ms</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-text-secondary">{t('policy_builder.preview.bandwidth_min')}</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">{formData.bandwidthMin} Mbps</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-text-secondary">{t('policy_builder.preview.fec_mode')}</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">{formData.fecMode}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-text-secondary">{t('policy_builder.preview.priority')}</span>
                  <span className="font-medium text-text-primary dark:text-text-dark">Level {formData.priority}</span>
                </div>
              </div>
              <div className="mt-6 flex justify-end">
                <Button variant="primary" onClick={() => setIsPreviewOpen(false)}>
                  {t('policy_builder.buttons.close')}
                </Button>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  );
};
