import { useCallback, useState } from 'react';
import { Bell, CheckCheck, RefreshCw } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';
import {
  useNotifications,
  useUnreadCount,
  useMarkNotificationRead,
  useMarkAllRead,
} from '@/api/hooks';
import type { Notification } from '@/api/types';
import { PageHeader } from '@/components/layout/PageHeader';
import { Button } from '@/components/ui/Button';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { Badge, type BadgeVariant } from '@/components/ui/Badge';

/* ─── Type → badge variant ──────────────────────────────────── */
const typeVariant: Record<Notification['type'], BadgeVariant> = {
  info: 'info',
  warning: 'warning',
  error: 'error',
  success: 'success',
};

/* ─── Component ──────────────────────────────────────────────── */

export function NotificationsPage() {
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const notifications = useNotifications();
  const unreadCount = useUnreadCount();
  const markRead = useMarkNotificationRead();
  const markAllRead = useMarkAllRead();

  const isLoading = notifications.isLoading;
  const isError = notifications.isError;
  const data = notifications.data ?? [];
  const count = unreadCount.data?.count ?? 0;
  const isEmpty = !isLoading && !isError && data.length === 0;

  const handleMarkRead = useCallback(
    (id: string) => {
      markRead.mutate(id);
      setSelectedId(null);
    },
    [markRead],
  );

  const handleMarkAllRead = useCallback(() => {
    markAllRead.mutate();
  }, [markAllRead]);

  const handleRetry = useCallback(() => {
    notifications.refetch();
    unreadCount.refetch();
  }, [notifications, unreadCount]);

  /* ── Loading ────────────────────────────────────────────── */
  if (isLoading) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Notifications" />
        <div className="flex flex-col gap-3">
          <LoadingSkeleton variant="card" count={5} />
        </div>
      </div>
    );
  }

  /* ── Error ──────────────────────────────────────────────── */
  if (isError) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Notifications" />
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <Bell className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">Failed to load notifications</h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Something went wrong while fetching notifications.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  /* ── Data ───────────────────────────────────────────────── */
  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Notifications">
        {count > 0 && (
          <>
            <span className="text-sm text-text-secondary">{count} unread</span>
            <Button variant="secondary" onClick={handleMarkAllRead} loading={markAllRead.isPending}>
              <CheckCheck className="h-4 w-4" />
              Mark All Read
            </Button>
          </>
        )}
      </PageHeader>

      {isEmpty ? (
        <div className="rounded-lg border border-border">
          <EmptyState
            icon={Bell}
            title="No notifications"
            message="You're all caught up. Notifications will appear here when something needs your attention."
          />
        </div>
      ) : (
        <div className="flex flex-col gap-3">
          {data.map((notification) => (
            <div
              key={notification.id}
              className={`flex items-start gap-4 rounded-lg border p-4 transition-colors ${
                notification.read
                  ? 'border-border bg-surface'
                  : 'border-accent/20 bg-accent/5'
              } ${selectedId === notification.id ? 'ring-2 ring-accent' : ''}`}
              onClick={() => setSelectedId(notification.id)}
              role="button"
              tabIndex={0}
              aria-label={`${notification.title} — ${notification.read ? 'read' : 'unread'}`}
              onKeyDown={(e) => {
                if (e.key === 'Enter' || e.key === ' ') {
                  e.preventDefault();
                  setSelectedId(notification.id);
                }
              }}
            >
              {/* Type indicator */}
              <div className="mt-0.5">
                <Badge variant={typeVariant[notification.type]} size="sm" dot />
              </div>

              {/* Content */}
              <div className="flex-1">
                <div className="flex items-start justify-between gap-2">
                  <h3
                    className={`text-sm font-medium ${
                      notification.read ? 'text-text-primary' : 'text-text-primary'
                    }`}
                  >
                    {notification.title}
                  </h3>
                  <span className="shrink-0 text-xs text-text-tertiary">
                    {formatDistanceToNow(new Date(notification.created_at), { addSuffix: true })}
                  </span>
                </div>
                <p className={`mt-1 text-sm ${selectedId === notification.id ? 'text-text-primary' : 'text-text-secondary'}`}>
                  {notification.message}
                </p>

                {/* Expanded detail when selected */}
                {selectedId === notification.id && (
                  <div className="mt-3 flex items-center gap-3 border-t border-border pt-3">
                    <span className="text-xs text-text-tertiary">
                      Type: {notification.type}
                    </span>
                    <span className="text-xs text-text-tertiary">
                      ID: {notification.id.slice(0, 8)}…
                    </span>
                    {!notification.read && (
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleMarkRead(notification.id);
                        }}
                      >
                        <CheckCheck className="h-4 w-4" />
                        Mark Read
                      </Button>
                    )}
                  </div>
                )}
              </div>

              {/* Mark read button (collapsed state) — show on the right when not expanded */}
              {!notification.read && selectedId !== notification.id && (
                <div className="shrink-0">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleMarkRead(notification.id);
                    }}
                  >
                    <CheckCheck className="h-4 w-4" />
                    Mark Read
                  </Button>
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
