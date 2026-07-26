import { useState, useCallback } from 'react';
import { RefreshCw, TrendingUp, Zap } from 'lucide-react';
import { useEfficiencySkills } from '@/api/hooks';
import type { SkillEffectiveness } from '@/api/types';
import { DataTable, type Column } from '@/components/ui/DataTable';
import { PageHeader } from '@/components/layout/PageHeader';
import { StatCard } from '@/components/ui/StatCard';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { EmptyState } from '@/components/ui/EmptyState';
import { TimeframeFilter } from '@/components/ui/TimeframeFilter';
import { Button } from '@/components/ui/Button';
import { formatNumber } from '@/utils/formatters';

const columns: Column<SkillEffectiveness>[] = [
  {
    key: 'skill_name',
    header: 'Skill',
    render: (s) => (
      <span className="font-medium text-text-primary">{s.skill_name}</span>
    ),
  },
  {
    key: 'effectiveness_score',
    header: 'Score',
    render: (s) => (
      <span className="text-text-secondary">{s.effectiveness_score}</span>
    ),
  },
  {
    key: 'usage_count',
    header: 'Usage',
    render: (s) => (
      <span className="text-text-secondary">{formatNumber(s.usage_count)}</span>
    ),
  },
  {
    key: 'trend',
    header: 'Trend',
    render: (s) => {
      const color = s.trend > 0 ? 'text-success' : s.trend < 0 ? 'text-error' : 'text-text-tertiary';
      const sign = s.trend > 0 ? '+' : '';
      return (
        <span className={`font-medium ${color}`}>
          {sign}{s.trend}%
        </span>
      );
    },
  },
];

export function EfficiencySkillsPage() {
  const [timeframe, setTimeframe] = useState('30d');
  const { data, isLoading, error, refetch } = useEfficiencySkills(timeframe);

  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  /* ── Loading state ──────────────────────────────────────── */
  if (isLoading && !error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Skill Effectiveness">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {Array.from({ length: 3 }, (_, i) => (
            <LoadingSkeleton key={i} variant="card" />
          ))}
        </div>
        <LoadingSkeleton variant="card" />
      </div>
    );
  }

  /* ── Error state ────────────────────────────────────────── */
  if (error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Skill Effectiveness">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load skill data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Unable to retrieve skill effectiveness metrics. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  const skills: SkillEffectiveness[] = data ?? [];

  /* ── Derived stats ──────────────────────────────────────── */
  const avgScore =
    skills.length > 0
      ? Math.round(
          skills.reduce((sum, s) => sum + s.effectiveness_score, 0) /
            skills.length,
        )
      : 0;

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Skill Effectiveness">
        <TimeframeFilter value={timeframe} onChange={setTimeframe} />
      </PageHeader>

      {/* Summary stat cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <StatCard
          value={formatNumber(skills.length)}
          label="Total Skills"
        />
        <StatCard
          value={avgScore}
          label="Avg Score"
        />
        <StatCard
          value={formatNumber(skills.reduce((sum, s) => sum + s.usage_count, 0))}
          label="Total Usage"
        />
      </div>

      {/* Skills table or empty state */}
      {skills.length > 0 ? (
        <DataTable<SkillEffectiveness>
          columns={columns}
          data={skills}
          pageSize={20}
        />
      ) : (
        <div className="rounded-lg border border-border">
          <EmptyState
            icon={Zap}
            title="No skill data available"
            message="Skills effectiveness data will appear here once skills have been evaluated."
          />
        </div>
      )}
    </div>
  );
}
