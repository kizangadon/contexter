import { formatDistanceToNow } from 'date-fns';
import type { Turn } from '@/api/types';
import { Badge } from '@/components/ui/Badge';

export interface MessageBubbleProps {
  /** The turn data to render */
  turn: Turn;
  /** Whether this turn is from the user (right-aligned) or agent (left-aligned) */
  isUser: boolean;
  /** Turn number for display (1-indexed) */
  turnNumber?: number;
}

/**
 * Renders a single conversation turn as a chat bubble.
 * User messages appear right-aligned; agent messages appear left-aligned
 * with agent name and optional latency badge.
 */
export function MessageBubble({ turn, isUser, turnNumber }: MessageBubbleProps) {
  return (
    <div className={`flex flex-col ${isUser ? 'items-end' : 'items-start'}`}>
      {/* Turn number label */}
      {turnNumber != null && (
        <span className="mb-1 px-1 text-[10px] font-semibold uppercase tracking-wider text-text-tertiary">
          Turn {turnNumber}
        </span>
      )}
      <div
        className={`flex max-w-[80%] flex-col gap-1 rounded-lg px-4 py-3 ${
          isUser
            ? 'bg-accent/10 text-text-primary'
            : 'border border-border bg-surface text-text-primary'
        }`}
      >
        {/* Agent name + latency for non-user messages */}
        {!isUser && turn.agent && (
          <div className="flex items-center gap-2">
            <span className="text-xs font-semibold text-accent">
              {turn.agent}
            </span>
            {turn.latency_ms != null && (
              <Badge variant="info" size="sm">
                {turn.latency_ms}ms
              </Badge>
            )}
          </div>
        )}

        {/* Message content */}
        <p className="whitespace-pre-wrap text-sm">{turn.content}</p>

        {/* Timestamp */}
        <span className="text-xs text-text-tertiary">
          {formatDistanceToNow(new Date(turn.created_at), { addSuffix: true })}
        </span>
      </div>
    </div>
  );
}
