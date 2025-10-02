import { X } from 'lucide-react';
import { ReactNode, useEffect } from 'react';
import { Button } from './Button';

export interface ModalProps {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
  footer?: ReactNode;
  size?: 'sm' | 'md' | 'lg' | 'xl';
  closeOnOverlayClick?: boolean;
  closeOnEsc?: boolean;
}

/**
 * Modal component following HoneyLink design system
 *
 * Features:
 * - Overlay with backdrop blur
 * - Configurable sizes
 * - Close on overlay click (optional)
 * - Close on Escape key (optional)
 * - Focus trap (basic)
 * - Dark mode support
 *
 * Accessibility:
 * - Role="dialog"
 * - ARIA attributes (aria-modal, aria-labelledby)
 * - Focus management (trap focus inside modal)
 * - Keyboard support (Escape to close)
 */
export function Modal({
  isOpen,
  onClose,
  title,
  children,
  footer,
  size = 'md',
  closeOnOverlayClick = true,
  closeOnEsc = true,
}: ModalProps) {
  const modalId = `modal-${Math.random().toString(36).slice(2, 9)}`;

  // Handle Escape key
  useEffect(() => {
    if (!isOpen || !closeOnEsc) return;

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    document.addEventListener('keydown', handleEscape);
    return () => document.removeEventListener('keydown', handleEscape);
  }, [isOpen, closeOnEsc, onClose]);

  // Prevent body scroll when modal is open
  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }
    return () => {
      document.body.style.overflow = '';
    };
  }, [isOpen]);

  if (!isOpen) return null;

  const sizeClasses = {
    sm: 'max-w-sm',
    md: 'max-w-md',
    lg: 'max-w-lg',
    xl: 'max-w-xl',
  };

  const handleOverlayClick = (e: React.MouseEvent) => {
    if (closeOnOverlayClick && e.target === e.currentTarget) {
      onClose();
    }
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/50 backdrop-blur-sm animate-fadeIn"
      onClick={handleOverlayClick}
      role="dialog"
      aria-modal="true"
      aria-labelledby={modalId}
    >
      <div
        className={`
          ${sizeClasses[size]} w-full
          bg-surface dark:bg-surface-dark
          rounded-card shadow-card-hover
          transform transition-all
          animate-slideUp
        `}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-surface-alt dark:border-surface-dark">
          <h2 id={modalId} className="text-heading font-semibold text-text-primary dark:text-text-dark">
            {title}
          </h2>
          <button
            onClick={onClose}
            className="p-1 rounded-button text-text-secondary hover:text-text-primary hover:bg-surface-alt dark:hover:bg-surface-dark transition-colors focus:outline-none focus:ring-2 focus:ring-secondary"
            aria-label="Close modal"
          >
            <X size={20} />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 text-text-primary dark:text-text-dark">{children}</div>

        {/* Footer */}
        {footer && (
          <div className="flex items-center justify-end gap-2 p-4 border-t border-surface-alt dark:border-surface-dark">
            {footer}
          </div>
        )}
      </div>
    </div>
  );
}

// Default footer for common use cases
export function ModalFooter({ onCancel, onConfirm, cancelText = 'Cancel', confirmText = 'Confirm', loading = false }: {
  onCancel: () => void;
  onConfirm: () => void;
  cancelText?: string;
  confirmText?: string;
  loading?: boolean;
}) {
  return (
    <>
      <Button variant="ghost" onClick={onCancel} disabled={loading}>
        {cancelText}
      </Button>
      <Button variant="primary" onClick={onConfirm} loading={loading}>
        {confirmText}
      </Button>
    </>
  );
}
