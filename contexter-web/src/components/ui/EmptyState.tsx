import { type ReactNode } from 'react';
import type { LucideIcon } from 'lucide-react';

export interface EmptyStateProps {
  /** Lucide icon component rendered above the title */
  icon?: LucideIcon;
  /** Primary heading */
  title: string;
  /** Descriptive body text */
  message: string;
  /** Optional call-to-action element (e.g. a Button) */
  action?: ReactNode;
  /** Additional CSS class names */
  className?: string;
}

export function EmptyState({
  icon: Icon,
  title,
  message,
  action,
  className = '',
}: EmptyStateProps) {
  return (
    <div
      className={`flex flex-col items-center justify-center gap-3 px-6 py-12 text-center ${className}`}
    >
      {Icon && (
        <div className="mb-1 rounded-full bg-bg-tertiary p-3">
          <Icon className="h-8 w-8 text-text-tertiary" aria-hidden="true" />
        </div>
      )}

      <h3 className="text-lg font-semibold text-text-primary">{title}</h3>

      <p className="max-w-sm text-sm text-text-secondary">{message}</p>

      {action && <div className="mt-2">{action}</div>}
    </div>
  );
}
