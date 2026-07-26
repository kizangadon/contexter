import { X } from 'lucide-react';

export type TagColor =
  | 'success'
  | 'warning'
  | 'error'
  | 'info'
  | 'pending'
  | 'offline';

export interface TagProps {
  /** Tag label text */
  label: string;
  /** Semantic color variant — maps to V2-DEEP status tokens */
  color?: TagColor;
  /** When provided, renders an X remove button */
  onRemove?: () => void;
  /** Additional CSS class names */
  className?: string;
}

const colorStyles: Record<TagColor, string> = {
  success: 'bg-success/20 text-success',
  warning: 'bg-warning/20 text-warning',
  error: 'bg-error/20 text-error',
  info: 'bg-info/20 text-info',
  pending: 'bg-pending/20 text-pending',
  offline: 'bg-offline/20 text-offline',
};

const baseStyles =
  'inline-flex items-center gap-1 rounded-md px-2 py-0.5 text-xs font-medium max-w-[50ch]';

export function Tag({
  label,
  color,
  onRemove,
  className = '',
}: TagProps) {
  const colorClass = color ? colorStyles[color] : 'bg-bg-tertiary text-text-secondary';

  return (
    <span
      className={`${baseStyles} ${colorClass} truncate ${className}`.trim()}
    >
      <span className="truncate">{label}</span>
      {onRemove && (
        <button
          type="button"
          onClick={onRemove}
          className="ml-0.5 inline-flex shrink-0 items-center justify-center rounded-sm p-0.5 hover:bg-black/20 transition-colors"
          aria-label={`Remove ${label}`}
        >
          <X className="h-3 w-3" />
        </button>
      )}
    </span>
  );
}
