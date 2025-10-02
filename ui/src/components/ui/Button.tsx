import { Loader2 } from 'lucide-react';
import { ButtonHTMLAttributes, forwardRef, ReactNode } from 'react';

type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost' | 'outline';
type ButtonSize = 'sm' | 'md' | 'lg';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  loading?: boolean;
  icon?: ReactNode;
  children: ReactNode;
}

/**
 * Button component following HoneyLink design system (spec/ui/visual-design.md)
 *
 * Variants:
 * - primary: Main CTA with #F4B400 background
 * - secondary: Sub-CTA with #7F5AF0 background
 * - danger: Destructive actions with #EF476F background
 * - ghost: Transparent background
 * - outline: Border only
 *
 * Accessibility: WCAG 2.2 AA compliant with focus ring and keyboard support
 */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ variant = 'primary', size = 'md', loading = false, icon, children, disabled, className = '', ...props }, ref) => {
    // Base styles
    const baseStyles = 'inline-flex items-center justify-center font-medium rounded-button transition-all focus:outline-none focus:ring-2 focus:ring-secondary focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed';

    // Variant styles
    const variantStyles: Record<ButtonVariant, string> = {
      primary: 'bg-primary text-text-primary hover:bg-primary-dark active:scale-95 shadow-sm hover:shadow-md',
      secondary: 'bg-secondary text-text-inverse hover:bg-secondary/90 active:scale-95 shadow-sm hover:shadow-md',
      danger: 'bg-error text-text-inverse hover:bg-error/90 active:scale-95 shadow-sm hover:shadow-md',
      ghost: 'bg-transparent text-text-primary hover:bg-surface-alt dark:hover:bg-surface-dark/50',
      outline: 'bg-transparent border-2 border-primary text-primary hover:bg-primary hover:text-text-primary',
    };

    // Size styles
    const sizeStyles: Record<ButtonSize, string> = {
      sm: 'text-sm px-3 py-1.5 gap-1.5',
      md: 'text-base px-4 py-2 gap-2',
      lg: 'text-lg px-6 py-3 gap-2.5',
    };

    return (
      <button
        ref={ref}
        disabled={disabled || loading}
        className={`${baseStyles} ${variantStyles[variant]} ${sizeStyles[size]} ${className}`}
        {...props}
      >
        {loading && <Loader2 className="animate-spin" size={size === 'sm' ? 14 : size === 'lg' ? 20 : 16} />}
        {!loading && icon && <span className="inline-flex">{icon}</span>}
        {children}
      </button>
    );
  }
);

Button.displayName = 'Button';
