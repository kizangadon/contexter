import type { Skill } from '@/api/types';
import { Badge } from '@/components/ui/Badge';

export interface SkillCardProps {
  skill: Skill;
  onClick: (id: string) => void;
}

function getCategoryVariant(category: string) {
  const map: Record<string, 'info' | 'success' | 'warning' | 'error' | 'pending'> = {
    'code-review': 'info',
    debugging: 'warning',
    refactoring: 'success',
    testing: 'info',
    architecture: 'pending',
  };
  return map[category] ?? 'info';
}

export function SkillCard({ skill, onClick }: SkillCardProps) {
  return (
    <button
      type="button"
      data-testid="skill-card"
      onClick={() => onClick(skill.id)}
      className="group flex flex-col gap-3 rounded-lg border border-border bg-surface p-4 text-left transition-all duration-150 hover:border-border-hover hover:bg-surface-hover focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent"
    >
      {/* Top row: name + category */}
      <div className="flex items-start justify-between gap-2">
        <h3 className="text-sm font-semibold text-text-primary">{skill.name}</h3>
        <Badge variant={getCategoryVariant(skill.category)} size="sm">
          {skill.category}
        </Badge>
      </div>

      {/* Effectiveness bar */}
      <div className="flex flex-col gap-1">
        <div className="flex items-center justify-between text-xs">
          <span className="text-text-secondary">Effectiveness</span>
          <span className="font-medium text-text-primary">{skill.effectiveness_score}%</span>
        </div>
        <div className="h-2 w-full overflow-hidden rounded-full bg-bg-tertiary">
          <div
            data-testid="effectiveness-bar-fill"
            className="h-full rounded-full bg-accent transition-all duration-300"
            style={{ width: `${skill.effectiveness_score}%` }}
          />
        </div>
      </div>

      {/* Usage count */}
      <p className="text-xs text-text-tertiary">
        {skill.usage_count} use{skill.usage_count !== 1 ? 's' : ''}
      </p>
    </button>
  );
}
