import { zodResolver } from '@hookform/resolvers/zod';
import { Eye, FileText, Save } from 'lucide-react';
import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { useTranslation } from 'react-i18next';
import { z } from 'zod';
import { Button, Card, CardContent, CardHeader, Input, Select } from '../components/ui';

/**
 * Zod validation schema for policy template form
 * Provides type-safe validation with i18n-friendly error messages
 */
const createPolicySchema = (t: (key: string) => string) =>
  z.object({
    name: z
      .string()
      .min(1, t('policy_builder.validation.name_required'))
      .min(3, t('policy_builder.validation.name_min_length')),
    usage: z.string(),
    latencyTarget: z
      .number()
      .min(1, t('policy_builder.validation.latency_range'))
      .max(50, t('policy_builder.validation.latency_range')),
    bandwidthMin: z
      .number()
      .min(10, t('policy_builder.validation.bandwidth_range'))
      .max(5000, t('policy_builder.validation.bandwidth_range')),
    fecMode: z.enum(['NONE', 'LIGHT', 'MEDIUM', 'HEAVY']),
    scheduleStart: z.string(),
    scheduleEnd: z.string(),
    priority: z.number().min(1).max(5),
  })
  .refine(
    (data) => {
      const start = new Date(data.scheduleStart);
      const end = new Date(data.scheduleEnd);
      return start < end;
    },
    {
      message: '',
      path: ['scheduleEnd'],
    }
  );

/**
 * Policy template form data type (inferred from schema)
 */
type PolicyTemplate = z.infer<ReturnType<typeof createPolicySchema>>;

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
export default function PolicyBuilderPage() {
  const { t } = useTranslation();

  // Create schema with current translation function
  const policySchema = createPolicySchema(t);

  // react-hook-form setup with zod validation
  const {
    register,
    handleSubmit,
    formState: { errors, isValid },
    watch,
  } = useForm<PolicyTemplate>({
    resolver: zodResolver(policySchema),
    mode: 'onChange', // Real-time validation
    defaultValues: {
      name: '',
      usage: 'low_latency',
      latencyTarget: 10,
      bandwidthMin: 150,
      fecMode: 'LIGHT',
      scheduleStart: new Date().toISOString().split('T')[0],
      scheduleEnd: new Date(Date.now() + 30 * 24 * 60 * 60 * 1000)
        .toISOString()
        .split('T')[0],
      priority: 2,
    },
  });

  const formData = watch(); // For preview modal
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
  const priorityOptions = [...Array(5)].map((_, i) => ({
    value: i + 1,
    label: `${i + 1} - ${t(`policy_builder.priority_levels.level_${i + 1}`)}`,
  }));

  /**
   * Handle form preview (no validation needed - button is disabled when form invalid)
   */
  const handlePreview = () => {
    setIsPreviewOpen(true);
  };

  /**
   * Handle form submission (validated by react-hook-form)
   */
  const onSubmit = (data: PolicyTemplate) => {
    // TODO: API integration - POST /policies
    console.log('Save policy template:', data);
    alert('テンプレートを保存しました (Mock)');
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
                {...register('name')}
                error={errors.name?.message}
                fullWidth
              />

              <Select
                label={t('policy_builder.form.usage')}
                helperText={t('policy_builder.form.usage_help')}
                options={usageOptions}
                {...register('usage')}
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
                  {...register('latencyTarget', { valueAsNumber: true })}
                  error={errors.latencyTarget?.message}
                  fullWidth
                />

                <Input
                  label={t('policy_builder.form.bandwidth_min')}
                  helperText={t('policy_builder.form.bandwidth_help')}
                  type="number"
                  min={10}
                  max={5000}
                  {...register('bandwidthMin', { valueAsNumber: true })}
                  error={errors.bandwidthMin?.message}
                  fullWidth
                />
              </div>

              <Select
                label={t('policy_builder.form.fec_mode')}
                helperText={t('policy_builder.form.fec_help')}
                options={fecModeOptions}
                {...register('fecMode')}
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
                  {...register('scheduleStart')}
                  error={errors.scheduleStart?.message}
                  fullWidth
                />

                <Input
                  label={t('policy_builder.form.schedule_end')}
                  type="date"
                  {...register('scheduleEnd')}
                  error={errors.scheduleEnd?.message}
                  fullWidth
                />
              </div>

              <Select
                label={t('policy_builder.form.priority')}
                helperText={t('policy_builder.form.priority_help')}
                options={priorityOptions.map(opt => ({ value: opt.value.toString(), label: opt.label }))}
                {...register('priority', { valueAsNumber: true })}
              />
            </div>

            {/* Actions */}
            <div className="flex flex-col sm:flex-row gap-3 pt-4 border-t border-text-secondary/10">
              <Button
                variant="outline"
                icon={<Eye size={18} />}
                onClick={handlePreview}
                disabled={!isValid}
                className="flex-1"
              >
                {t('policy_builder.buttons.preview')}
              </Button>
              <Button
                variant="primary"
                icon={<Save size={18} />}
                onClick={handleSubmit(onSubmit)}
                disabled={!isValid}
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
                    <li key={index}>{error?.message}</li>
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
