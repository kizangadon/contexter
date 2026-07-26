import { formatDistanceToNow } from 'date-fns';
import type { Session, SessionDetail } from '@/api/types';
import { Badge, type BadgeVariant } from '@/components/ui/Badge';

export interface SessionInfoHeaderProps {
  /** The session to display info for */
  session: SessionDetail;
}

/* ── Status → Badge variant ────────────────────────────────── */
const statusVariant: Record<Session['status'], BadgeVariant> = {
  active: 'success',
  done: 'info',
  error: 'error',
  paused: 'pending',
};

/**
 * Renders session metadata with Status badge, agent, project, date, duration, and turns.
 * Used as the header section on session detail pages.
 */
export function SessionInfoHeader({ session }: SessionInfoHeaderProps) {
  return (
    <div className="mb-lg flex flex-col gap-4 rounded-lg border border-border bg-surface p-4 sm:flex-row sm:items-center sm:justify-between">
      {/* Left: ID + Status + metadata */}
      <div className="flex flex-col gap-3">
        <div className="flex items-center gap-3">
          <span className="font-mono text-xs text-text-tertiary">
            {session.id}
          </span>
          <Badge variant={statusVariant[session.status]} size="md" dot>
            {session.status}
          </Badge>
        </div>
        <div className="flex flex-wrap gap-x-4 gap-y-1 text-sm text-text-secondary">
          <span>
            Agent:{' '}
            <span className="font-medium text-text-primary">
              {session.agent}
            </span>
          </span>
          <span>
            Project:{' '}
            <span className="font-medium text-text-primary">
              {session.project}
            </span>
          </span>
          <span>
            Created:{' '}
            <span className="font-medium text-text-primary">
              {formatDistanceToNow(new Date(session.created_at), {
                addSuffix: true,
              })}
            </span>
          </span>
        </div>
      </div>

      {/* Right: Duration + Turns stats */}
      <div className="flex shrink-0 items-center gap-4 rounded-lg bg-bg-tertiary px-6 py-3">
        <div className="flex flex-col items-center">
          <span className="text-xs font-medium uppercase tracking-wider text-text-secondary">
            Duration
          </span>
          <span className="text-lg font-bold text-text-primary">
            {session.duration_minutes}m
          </span>
        </div>
        <div className="h-8 w-px bg-border" />
        <div className="flex flex-col items-center">
          <span className="text-xs font-medium uppercase tracking-wider text-text-secondary">
            Turns
          </span>
          <span className="text-lg font-bold text-text-primary">
            {session.turn_count}
          </span>
        </div>
      </div>
    </div>
  );
}
