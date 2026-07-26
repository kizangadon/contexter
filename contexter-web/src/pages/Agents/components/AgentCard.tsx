import type { Agent } from '@/api/types';
import { Badge } from '@/components/ui/Badge';
import { Tag } from '@/components/ui/Tag';

/* ── Status → Badge variant map ───────────────────────────── */
const statusVariant: Record<
  Agent['status'],
  'success' | 'pending' | 'error' | 'offline'
> = {
  active: 'success',
  idle: 'pending',
  error: 'error',
  offline: 'offline',
};

/* ── Efficiency bar color by score tier ────────────────────── */
function barColor(score: number): string {
  if (score >= 80) return 'bg-success';
  if (score >= 50) return 'bg-warning';
  return 'bg-error';
}

export interface AgentCardProps {
  /** The agent to display */
  agent: Agent;
  /** Called when the card is clicked */
  onClick: () => void;
}

export function AgentCard({ agent, onClick }: AgentCardProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="flex w-full flex-col gap-3 rounded-lg border border-border bg-surface p-4 text-left transition-colors duration-150 hover:border-border-hover hover:bg-surface-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
    >
      {/* Header: name + status */}
      <div className="flex items-start justify-between gap-2">
        <h3 className="truncate text-base font-semibold text-text-primary">
          {agent.name}
        </h3>
        <Badge variant={statusVariant[agent.status]} size="sm" dot>
          {agent.status}
        </Badge>
      </div>

      {/* Capability tags */}
      <div className="flex flex-wrap gap-1">
        {agent.capabilities.map((cap) => (
          <Tag key={cap} label={cap} />
        ))}
      </div>

      {/* Efficiency bar */}
      <div className="flex flex-col gap-1">
        <div className="flex items-center justify-between text-xs">
          <span className="text-text-secondary">Efficiency</span>
          <span className="font-semibold text-text-primary">
            {agent.efficiency_score}%
          </span>
        </div>
        <div className="h-2 w-full overflow-hidden rounded-full bg-bg-tertiary">
          <div
            data-testid="efficiency-bar"
            className={`h-full rounded-full transition-all duration-300 ${barColor(agent.efficiency_score)}`}
            style={{ width: `${agent.efficiency_score}%` }}
            role="progressbar"
            aria-valuenow={agent.efficiency_score}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-label={`Efficiency ${agent.efficiency_score}%`}
          />
        </div>
      </div>

      {/* Sessions count */}
      <p className="text-xs text-text-tertiary">
        {agent.sessions_count} session{agent.sessions_count !== 1 ? 's' : ''}
      </p>
    </button>
  );
}
