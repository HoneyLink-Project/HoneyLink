import { ChevronDown } from 'lucide-react';
import { forwardRef, SelectHTMLAttributes } from 'react';

export interface SelectOption {
  value: string;
  label: string;
  disabled?: boolean;
}

export interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  error?: string;
  helperText?: string;
  options: SelectOption[];
  placeholder?: string;
  fullWidth?: boolean;
}

/**
 * Select component following HoneyLink design system
 *
 * Features:
 * - Label and helper text support
 * - Error state
 * - Placeholder option
 * - Custom dropdown icon
 * - Full width option
 * - Dark mode support
 *
 * Accessibility:
 * - Label association
 * - ARIA attributes for error state
 * - Keyboard navigation (native select)
 */
export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ label, error, helperText, options, placeholder, fullWidth = false, className = '', id, ...props }, ref) => {
    const selectId = id || `select-${Math.random().toString(36).slice(2, 9)}`;
    const hasError = Boolean(error);

    const widthClass = fullWidth ? 'w-full' : '';
    const errorBorder = hasError ? 'border-error focus:ring-error' : 'border-surface-alt dark:border-surface-dark focus:ring-secondary';

    return (
      <div className={`${widthClass}`}>
        {label && (
          <label htmlFor={selectId} className="block text-sm font-medium text-text-primary dark:text-text-dark mb-1">
            {label}
          </label>
        )}
        <div className="relative">
          <select
            ref={ref}
            id={selectId}
            className={`
              px-3 py-2 pr-10 w-full
              bg-surface dark:bg-surface-dark
              text-text-primary dark:text-text-dark
              border-2 ${errorBorder}
              rounded-button
              transition-colors
              focus:outline-none focus:ring-2 focus:ring-offset-1
              disabled:opacity-50 disabled:cursor-not-allowed
              appearance-none cursor-pointer
              ${className}
            `}
            aria-invalid={hasError}
            aria-describedby={hasError ? `${selectId}-error` : helperText ? `${selectId}-helper` : undefined}
            {...props}
          >
            {placeholder && (
              <option value="" disabled>
                {placeholder}
              </option>
            )}
            {options.map((option) => (
              <option key={option.value} value={option.value} disabled={option.disabled}>
                {option.label}
              </option>
            ))}
          </select>
          <div className="absolute right-3 top-1/2 -translate-y-1/2 text-text-secondary pointer-events-none">
            <ChevronDown size={18} />
          </div>
        </div>
        {error && (
          <p id={`${selectId}-error`} className="mt-1 text-sm text-error">
            {error}
          </p>
        )}
        {!error && helperText && (
          <p id={`${selectId}-helper`} className="mt-1 text-sm text-text-secondary">
            {helperText}
          </p>
        )}
      </div>
    );
  }
);

Select.displayName = 'Select';
