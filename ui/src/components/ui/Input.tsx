import { AlertCircle } from 'lucide-react';
import { forwardRef, InputHTMLAttributes, ReactNode } from 'react';

export interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  label?: string;
  error?: string;
  helperText?: string;
  icon?: ReactNode;
  fullWidth?: boolean;
}

/**
 * Input component following HoneyLink design system
 *
 * Features:
 * - Label and helper text support
 * - Error state with icon
 * - Icon slot (left side)
 * - Full width option
 * - Dark mode support
 *
 * Accessibility:
 * - Label association via htmlFor
 * - ARIA attributes for error state
 * - Focus ring (2px secondary color)
 */
export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ label, error, helperText, icon, fullWidth = false, className = '', id, ...props }, ref) => {
    const inputId = id || `input-${Math.random().toString(36).slice(2, 9)}`;
    const hasError = Boolean(error);

    const widthClass = fullWidth ? 'w-full' : '';
    const iconPadding = icon ? 'pl-10' : 'pl-3';
    const errorBorder = hasError ? 'border-error focus:ring-error' : 'border-surface-alt dark:border-surface-dark focus:ring-secondary';

    return (
      <div className={`${widthClass}`}>
        {label && (
          <label htmlFor={inputId} className="block text-sm font-medium text-text-primary dark:text-text-dark mb-1">
            {label}
          </label>
        )}
        <div className="relative">
          {icon && (
            <div className="absolute left-3 top-1/2 -translate-y-1/2 text-text-secondary pointer-events-none">
              {icon}
            </div>
          )}
          <input
            ref={ref}
            id={inputId}
            className={`
              ${iconPadding} pr-3 py-2 w-full
              bg-surface dark:bg-surface-dark
              text-text-primary dark:text-text-dark
              border-2 ${errorBorder}
              rounded-button
              transition-colors
              focus:outline-none focus:ring-2 focus:ring-offset-1
              disabled:opacity-50 disabled:cursor-not-allowed
              placeholder:text-text-secondary
              ${className}
            `}
            aria-invalid={hasError}
            aria-describedby={hasError ? `${inputId}-error` : helperText ? `${inputId}-helper` : undefined}
            {...props}
          />
          {hasError && (
            <div className="absolute right-3 top-1/2 -translate-y-1/2 text-error pointer-events-none">
              <AlertCircle size={18} />
            </div>
          )}
        </div>
        {error && (
          <p id={`${inputId}-error`} className="mt-1 text-sm text-error flex items-center gap-1">
            {error}
          </p>
        )}
        {!error && helperText && (
          <p id={`${inputId}-helper`} className="mt-1 text-sm text-text-secondary">
            {helperText}
          </p>
        )}
      </div>
    );
  }
);

Input.displayName = 'Input';
