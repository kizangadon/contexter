import { useState, useCallback } from 'react';
import { RefreshCw, TrendingUp } from 'lucide-react';
import { useEfficiencySessions } from '@/api/hooks';
import type { EfficiencyDetail } from '@/api/types';
import { DataTable, type Column } from '@/components/ui/DataTable';
import { PageHeader } from '@/components/layout/PageHeader';
import { StatCard } from '@/components/ui/StatCard';
import { LoadingSkeleton } from '@/components/ui/LoadingSkeleton';
import { TimeframeFilter } from '@/components/ui/TimeframeFilter';
import { Button } from '@/components/ui/Button';
import { formatNumber } from '@/utils/formatters';

const columns: Column<EfficiencyDetail>[] = [
  {
    key: 'date',
    header: 'Date',
    render: (d) => (
      <span className="font-medium text-text-primary">{d.date}</span>
    ),
  },
  {
    key: 'score',
    header: 'Score',
    render: (d) => (
      <span className="text-text-secondary">{d.score}</span>
    ),
  },
  {
    key: 'tokens',
    header: 'Tokens',
    render: (d) => (
      <span className="text-text-secondary">{formatNumber(d.tokens)}</span>
    ),
  },
  {
    key: 'sessions',
    header: 'Sessions',
    render: (d) => (
      <span className="text-text-secondary">{d.sessions}</span>
    ),
  },
];

export function EfficiencySessionsPage() {
  const [timeframe, setTimeframe] = useState('30d');
  const { data, isLoading, error, refetch } = useEfficiencySessions(timeframe);

  const handleRetry = useCallback(() => {
    refetch();
  }, [refetch]);

  /* ── Loading state ──────────────────────────────────────── */
  if (isLoading && !error) {
    return (
      <div className="flex flex-col gap-lg">
        <PageHeader title="Session Activity">
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
        <PageHeader title="Session Activity">
          <TimeframeFilter value={timeframe} onChange={setTimeframe} />
        </PageHeader>
        <div className="flex flex-col items-center justify-center gap-4 rounded-lg border border-border bg-surface p-8 text-center">
          <div className="rounded-full bg-error/10 p-3">
            <TrendingUp className="h-6 w-6 text-error" aria-hidden="true" />
          </div>
          <h3 className="text-lg font-semibold text-text-primary">
            Failed to load session data
          </h3>
          <p className="max-w-sm text-sm text-text-secondary">
            Unable to retrieve session activity metrics. Please try again.
          </p>
          <Button variant="primary" onClick={handleRetry}>
            <RefreshCw className="h-4 w-4" />
            Retry
          </Button>
        </div>
      </div>
    );
  }

  const sessions: EfficiencyDetail[] = data ?? [];

  /* ── Derived stats ──────────────────────────────────────── */
  const totalSessions = sessions.reduce((sum, s) => sum + s.sessions, 0);
  const avgScore =
    sessions.length > 0
      ? Math.round(
          sessions.reduce((sum, s) => sum + s.score, 0) / sessions.length,
        )
      : 0;
  const totalTokens = sessions.reduce((sum, s) => sum + s.tokens, 0);

  return (
    <div className="flex flex-col gap-lg">
      <PageHeader title="Session Activity">
        <TimeframeFilter value={timeframe} onChange={setTimeframe} />
      </PageHeader>

      {/* Summary stat cards */}
      <div className="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
        <StatCard
          value={formatNumber(totalSessions)}
          label="Total Sessions"
        />
        <StatCard
          value={avgScore}
          label="Avg Score"
        />
        <StatCard
          value={formatNumber(totalTokens)}
          label="Total Tokens"
        />
      </div>

      {/* Sessions table */}
      {sessions.length > 0 ? (
        <DataTable<EfficiencyDetail>
          columns={columns}
          data={sessions}
          pageSize={20}
        />
      ) : (
        <p className="text-sm text-text-tertiary">
          No session activity data available for the selected timeframe.
        </p>
      )}
    </div>
  );
}
