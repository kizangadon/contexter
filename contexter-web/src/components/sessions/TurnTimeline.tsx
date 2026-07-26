import type { Turn } from '@/api/types';
import { MessageBubble } from '@/pages/Sessions/components/MessageBubble';

export interface TurnTimelineProps {
  /** The conversation turns to display in chronological order */
  turns: Turn[];
}

/**
 * Renders a timeline of conversation turns.
 * Each turn is displayed as a MessageBubble, ordered chronologically.
 * Shows an empty state message when no turns exist.
 */
export function TurnTimeline({ turns }: TurnTimelineProps) {
  return (
    <div className="flex flex-col gap-3">
      {turns.length === 0 ? (
        <p className="py-8 text-center text-sm text-text-tertiary">
          No turns recorded in this session.
        </p>
      ) : (
        turns.map((turn, index) => (
          <MessageBubble
            key={turn.id}
            turn={turn}
            isUser={turn.role === 'user'}
            turnNumber={index + 1}
          />
        ))
      )}
    </div>
  );
}
