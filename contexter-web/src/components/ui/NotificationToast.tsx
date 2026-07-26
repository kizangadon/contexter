import { formatDistanceToNow } from 'date-fns';
import { AlertCircle, AlertTriangle, CheckCircle2, Info, X } from 'lucide-react';
import type { Notification } from '@/api/types';

/* ── Type → icon mapping ───────────────────────────────────── */
const typeIcons: Record<Notification['type'], typeof Info> = {
  info: Info,
  warning: AlertTriangle,
  error: AlertCircle,
  success: CheckCircle2,
};

/* ── Type → accent colour ──────────────────────────────────── */
const typeAccent: Record<Notification['type'], string> = {
  info: 'border-l-info',
  warning: 'border-l-warning',
  error: 'border-l-error',
  success: 'border-l-success',
};

/* ── Props ──────────────────────────────────────────────────── */
export interface NotificationToastProps {
  /** The notification to display */
  notification: Notification;
  /** Called when the user clicks the "mark as read" button */
  onMarkRead?: (id: string) => void;
}

/**
 * Renders a single notification item with icon, title, message,
 * relative timestamp, and an unread indicator dot.
 */
export function NotificationToast({ notification, onMarkRead }: NotificationToastProps) {
  const Icon = typeIcons[notification.type];
  const unread = !notification.read;

  return (
    <div
      className={`flex items-start gap-3 rounded-lg border-l-4 p-4 transition-colors ${
        unread ? 'border-l-accent bg-accent/5' : typeAccent[notification.type]
      } ${unread ? 'border-border' : 'border-border'}`}
      data-testid={`notification-toast-${notification.id}`}
      role="listitem"
    >
      {/* Icon */}
      <div className="mt-0.5 shrink-0">
        <Icon className="h-5 w-5 text-accent" aria-hidden="true" />
      </div>

      {/* Content */}
      <div className="flex min-w-0 flex-1 flex-col gap-1">
        {/* Title row + unread dot */}
        <div className="flex items-center gap-2">
          <span className="truncate text-sm font-medium text-text-primary">
            {notification.title}
          </span>
          {unread && (
            <span
              className="h-2 w-2 shrink-0 rounded-full bg-accent"
              aria-label="Unread"
              data-testid="unread-dot"
            />
          )}
        </div>

        {/* Message */}
        <p className="line-clamp-2 text-sm text-text-secondary">
          {notification.message}
        </p>

        {/* Timestamp */}
        <span className="text-xs text-text-tertiary">
          {formatDistanceToNow(new Date(notification.created_at), { addSuffix: true })}
        </span>
      </div>

      {/* Mark-as-read button (only for unread) */}
      {unread && onMarkRead && (
        <button
          type="button"
          onClick={() => onMarkRead(notification.id)}
          className="mt-0.5 rounded p-0.5 text-text-tertiary transition-colors hover:text-text-primary"
          aria-label="Mark as read"
          data-testid="mark-read-btn"
        >
          <X className="h-4 w-4" />
        </button>
      )}
    </div>
  );
}
