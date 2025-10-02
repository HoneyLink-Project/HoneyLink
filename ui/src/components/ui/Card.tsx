import { HTMLAttributes, ReactNode } from 'react';

export interface CardProps extends HTMLAttributes<HTMLDivElement> {
  children: ReactNode;
  hoverable?: boolean;
  padding?: 'none' | 'sm' | 'md' | 'lg';
}

/**
 * Card component following HoneyLink design system
 *
 * Features:
 * - 16px border radius (radius.card)
 * - Shadow elevation (shadow.elevated)
 * - Hover state with 4px lift (optional)
 * - Configurable padding
 * - Dark mode support
 *
 * Accessibility: Semantic HTML with proper contrast
 */
export function Card({ children, hoverable = false, padding = 'md', className = '', ...props }: CardProps) {
  const paddingStyles = {
    none: '',
    sm: 'p-2',
    md: 'p-3',
    lg: 'p-4',
  };

  const hoverStyles = hoverable
    ? 'transition-all duration-200 hover:-translate-y-1 hover:shadow-card-hover cursor-pointer'
    : '';

  return (
    <div
      className={`bg-surface dark:bg-surface-dark rounded-card shadow-card dark:shadow-card-dark ${paddingStyles[padding]} ${hoverStyles} ${className}`}
      {...props}
    >
      {children}
    </div>
  );
}

export interface CardHeaderProps {
  title: string;
  subtitle?: string;
  action?: ReactNode;
}

export function CardHeader({ title, subtitle, action }: CardHeaderProps) {
  return (
    <div className="flex items-start justify-between mb-3">
      <div>
        <h3 className="text-subheading font-semibold text-text-primary dark:text-text-dark">{title}</h3>
        {subtitle && <p className="text-sm text-text-secondary mt-1">{subtitle}</p>}
      </div>
      {action && <div>{action}</div>}
    </div>
  );
}

export function CardContent({ children, className = '' }: { children: ReactNode; className?: string }) {
  return <div className={`text-body text-text-primary dark:text-text-dark ${className}`}>{children}</div>;
}

export function CardFooter({ children, className = '' }: { children: ReactNode; className?: string }) {
  return <div className={`mt-3 pt-3 border-t border-surface-alt dark:border-surface-dark ${className}`}>{children}</div>;
}
